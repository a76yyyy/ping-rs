use pinger::{PingOptions, PingResult as RustPingResult};
use pyo3::exceptions::{PyStopAsyncIteration, PyStopIteration};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3_async_runtimes::tokio::future_into_py;
use std::net::IpAddr;
use std::sync::mpsc;
use std::sync::OnceLock;
use std::time::Duration;

// 平台特定实现
#[cfg(target_os = "windows")]
mod windows;

#[cfg(not(target_os = "windows"))]
use pinger::ping;

// =================== 基础 ping 操作 ===================

/// 执行ping操作的统一接口，返回标准库的通道
/// 抽象平台差异，为同步操作提供基础
fn execute_ping(options: PingOptions) -> Result<mpsc::Receiver<RustPingResult>, pinger::PingCreationError> {
    #[cfg(target_os = "windows")]
    {
        windows::ping(options)
    }

    #[cfg(not(target_os = "windows"))]
    {
        ping(options)
    }
}

/// 异步执行ping操作，在支持的平台上使用纯异步实现，否则使用兼容的方式
/// 为异步操作提供基础
async fn execute_ping_async(
    options: PingOptions,
) -> Result<std::sync::Arc<std::sync::Mutex<mpsc::Receiver<RustPingResult>>>, pinger::PingCreationError> {
    // 在Windows平台使用优化的异步实现
    #[cfg(target_os = "windows")]
    {
        // 获取tokio通道
        let mut receiver = windows::ping_async(options).await?;

        // 创建标准库通道
        let (tx, rx) = mpsc::channel();

        // 创建一个任务来转发消息
        // 这个任务将会在标准通道的发送端关闭时退出
        tokio::spawn(async move {
            while let Some(result) = receiver.recv().await {
                // 当发送失败，说明接收端已关闭，退出循环
                if tx.send(result).is_err() {
                    break;
                }
            }
        });

        // 直接返回包装后的接收端
        // 当 rx 被丢弃时，tx 就会关闭，上面的任务会自然结束
        Ok(std::sync::Arc::new(std::sync::Mutex::new(rx)))
    }

    // 在其他平台上使用标准实现并包装为Arc<Mutex>
    #[cfg(not(target_os = "windows"))]
    {
        let receiver = execute_ping(options)?;
        Ok(std::sync::Arc::new(std::sync::Mutex::new(receiver)))
    }
}

// =================== 辅助函数 ===================

/// 验证 interval_ms 参数并转换为 u64
///
/// 由于 ping 命令的 -i 参数格式化为一位小数，所以 interval_ms 必须是 100ms 的倍数且不小于 100ms
fn validate_interval_ms(value: i64, param_name: &str) -> PyResult<u64> {
    if value < 0 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "{} must be a non-negative integer",
            param_name
        )));
    }
    if value < 100 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "{} must be at least 100ms",
            param_name
        )));
    }
    if value % 100 != 0 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "{} must be a multiple of 100ms due to ping command's decimal precision",
            param_name
        )));
    }
    Ok(value as u64)
}

/// 从 Python 对象中提取 IP 地址字符串
fn extract_target(target: &Bound<PyAny>) -> PyResult<String> {
    // 首先尝试直接提取为 IpAddr（包含 IPv4 和 IPv6）
    if let Ok(ip_addr) = target.extract::<IpAddr>() {
        return Ok(ip_addr.to_string());
    }

    // 尝试作为字符串提取
    if let Ok(s) = target.extract::<String>() {
        return Ok(s);
    }

    Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
        "Expected target to be a string, IPv4Address, or IPv6Address",
    ))
}

/// 创建 PingOptions 配置
fn create_ping_options(
    target: &str,
    interval_ms: u64,
    interface: Option<String>,
    ipv4: bool,
    ipv6: bool,
) -> PingOptions {
    let interval = Duration::from_millis(interval_ms);

    if ipv4 {
        PingOptions::new_ipv4(target, interval, interface)
    } else if ipv6 {
        PingOptions::new_ipv6(target, interval, interface)
    } else {
        PingOptions::new(target, interval, interface)
    }
}

// =================== Python 类型定义 ===================

/// Python 包装的 PingResult 枚举
#[pyclass]
#[derive(Debug, Clone)]
pub enum PingResult {
    /// 成功的 ping 响应，包含延迟时间（毫秒）和原始行
    Pong { duration_ms: f64, line: String },
    /// 超时
    Timeout { line: String },
    /// 未知响应
    Unknown { line: String },
    /// Ping 进程退出
    PingExited { exit_code: i32, stderr: String },
}

#[pymethods]
impl PingResult {
    fn __repr__(&self) -> String {
        match self {
            Self::Pong { duration_ms, line } => {
                format!("PingResult.Pong(duration_ms={}ms, line='{}')", duration_ms, line)
            }
            Self::Timeout { line } => format!("PingResult.Timeout(line='{}')", line),
            Self::Unknown { line } => format!("PingResult.Unknown(line='{}')", line),
            Self::PingExited { exit_code, stderr } => {
                format!("PingResult.PingExited(exit_code={}, stderr='{}')", exit_code, stderr)
            }
        }
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    /// 获取延迟时间（毫秒），如果不是 Pong 则返回 None
    #[getter]
    fn duration_ms(&self) -> Option<f64> {
        match self {
            Self::Pong { duration_ms, .. } => Some(*duration_ms),
            _ => None,
        }
    }

    /// 获取原始行内容
    #[getter]
    fn line(&self) -> String {
        match self {
            Self::Pong { line, .. } => line.clone(),
            Self::Timeout { line } => line.clone(),
            Self::Unknown { line } => line.clone(),
            Self::PingExited { stderr, .. } => stderr.clone(),
        }
    }

    /// 获取退出代码，如果不是 PingExited 则返回 None
    #[getter]
    fn exit_code(&self) -> Option<i32> {
        match self {
            Self::PingExited { exit_code, .. } => Some(*exit_code),
            _ => None,
        }
    }

    /// 获取标准错误输出，如果不是 PingExited 则返回 None
    #[getter]
    fn stderr(&self) -> Option<String> {
        match self {
            Self::PingExited { stderr, .. } => Some(stderr.clone()),
            _ => None,
        }
    }

    /// 检查是否为成功的 ping
    fn is_success(&self) -> bool {
        matches!(self, Self::Pong { .. })
    }

    /// 检查是否为超时
    fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout { .. })
    }

    /// 检查是否为未知响应
    fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown { .. })
    }

    /// 检查是否为 ping 进程退出
    fn is_exited(&self) -> bool {
        matches!(self, Self::PingExited { .. })
    }

    /// 获取 PingResult 的类型名称
    #[getter]
    fn type_name(&self) -> String {
        match self {
            Self::Pong { .. } => "Pong".to_string(),
            Self::Timeout { .. } => "Timeout".to_string(),
            Self::Unknown { .. } => "Unknown".to_string(),
            Self::PingExited { .. } => "PingExited".to_string(),
        }
    }

    /// 将 PingResult 转换为字典
    fn to_dict(&self, py: Python) -> PyResult<PyObject> {
        let dict = PyDict::new(py);

        match self {
            Self::Pong { duration_ms, line } => {
                dict.set_item("type", "Pong")?;
                dict.set_item("duration_ms", *duration_ms)?;
                dict.set_item("line", line.clone())?;
            }
            Self::Timeout { line } => {
                dict.set_item("type", "Timeout")?;
                dict.set_item("line", line.clone())?;
            }
            Self::Unknown { line } => {
                dict.set_item("type", "Unknown")?;
                dict.set_item("line", line.clone())?;
            }
            Self::PingExited { exit_code, stderr } => {
                dict.set_item("type", "PingExited")?;
                dict.set_item("exit_code", *exit_code)?;
                dict.set_item("stderr", stderr.clone())?;
            }
        };

        Ok(dict.into())
    }
}

impl From<RustPingResult> for PingResult {
    fn from(result: RustPingResult) -> Self {
        match result {
            RustPingResult::Pong(duration, line) => Self::Pong {
                duration_ms: duration.as_secs_f64() * 1000.0,
                line,
            },
            RustPingResult::Timeout(line) => Self::Timeout { line },
            RustPingResult::Unknown(line) => Self::Unknown { line },
            RustPingResult::PingExited(status, stderr) => Self::PingExited {
                exit_code: status.code().unwrap_or(-1),
                stderr,
            },
        }
    }
}

/// Python 包装的 Pinger 类
#[pyclass]
pub struct Pinger {
    target: String,
    interval_ms: u64,
    interface: Option<String>,
    ipv4: bool,
    ipv6: bool,
}

#[pymethods]
impl Pinger {
    #[new]
    #[pyo3(signature = (target, interval_ms=1000, interface=None, ipv4=false, ipv6=false))]
    fn new(
        target: &Bound<PyAny>,
        interval_ms: i64,
        interface: Option<String>,
        ipv4: bool,
        ipv6: bool,
    ) -> PyResult<Self> {
        let target_str = extract_target(target)?;

        // 验证 interval_ms 参数
        let interval_ms_u64 = validate_interval_ms(interval_ms, "interval_ms")?;

        Ok(Self {
            target: target_str,
            interval_ms: interval_ms_u64,
            interface,
            ipv4,
            ipv6,
        })
    }

    /// 同步执行单次 ping
    fn ping_once(&self) -> PyResult<PingResult> {
        let options = create_ping_options(
            &self.target,
            self.interval_ms,
            self.interface.clone(),
            self.ipv4,
            self.ipv6,
        );

        // 执行ping并等待第一个结果
        let receiver = execute_ping(options)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to start ping: {}", e)))?;

        // 等待第一个结果
        match receiver.recv() {
            Ok(result) => Ok(result.into()),
            Err(_) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Failed to receive ping result",
            )),
        }
    }

    /// 异步执行单次ping
    fn ping_once_async<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let target = self.target.clone();
        let interval_ms = self.interval_ms;
        let interface = self.interface.clone();
        let ipv4 = self.ipv4;
        let ipv6 = self.ipv6;

        future_into_py(py, async move {
            let options = create_ping_options(&target, interval_ms, interface, ipv4, ipv6);

            // 在异步上下文中执行ping
            let receiver = execute_ping_async(options).await.map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to start ping: {}", e))
            })?;

            // 接收单个结果
            tokio::task::spawn_blocking(move || {
                let guard = receiver.lock().unwrap();
                match guard.recv() {
                    Ok(result) => Ok::<PingResult, pyo3::PyErr>(result.into()), // 指定类型参数
                    Err(_) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                        "Failed to receive ping result",
                    )),
                }
            })
            .await
            .unwrap_or_else(|e| {
                Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Task error: {}",
                    e
                )))
            })
        })
    }

    /// 同步执行多次 ping
    #[pyo3(signature = (count=4, timeout_ms=None))]
    fn ping_multiple(&self, count: i32, timeout_ms: Option<i64>) -> PyResult<Vec<PingResult>> {
        // 验证 count 参数
        if count <= 0 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "count ({}) must be a positive integer",
                count
            )));
        }
        let count = count as usize;

        // 验证 timeout_ms 参数
        let timeout = if let Some(timeout) = timeout_ms {
            let timeout_ms_u64 = validate_interval_ms(timeout, "timeout_ms")?;
            Some(Duration::from_millis(timeout_ms_u64))
        } else {
            None
        };

        let options = create_ping_options(
            &self.target,
            self.interval_ms,
            self.interface.clone(),
            self.ipv4,
            self.ipv6,
        );

        // 执行ping
        let receiver = execute_ping(options)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to start ping: {}", e)))?;

        let mut results = Vec::new();
        let mut received_count = 0;
        let start_time = std::time::Instant::now();

        while let Ok(result) = receiver.recv() {
            let ping_result: PingResult = result.into();

            // 添加到结果列表
            results.push(ping_result.clone());

            // 如果是退出信号，跳出循环
            if matches!(ping_result, PingResult::PingExited { .. }) {
                break;
            }

            received_count += 1;

            // 检查是否达到指定数量
            if received_count >= count {
                break;
            }

            // 检查是否超时
            if let Some(timeout_duration) = timeout {
                if start_time.elapsed() >= timeout_duration {
                    break;
                }
            }
        }

        Ok(results)
    }

    /// 异步执行多次 ping
    #[pyo3(signature = (count=4, timeout_ms=None))]
    fn ping_multiple_async<'py>(
        &self,
        py: Python<'py>,
        count: i32,
        timeout_ms: Option<i64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        // 验证 count 参数
        if count <= 0 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "count ({}) must be a positive integer",
                count
            )));
        }
        let count = count as usize;

        // 验证 timeout_ms 参数
        let timeout = if let Some(timeout) = timeout_ms {
            let timeout_ms_u64 = validate_interval_ms(timeout, "timeout_ms")?;
            Some(Duration::from_millis(timeout_ms_u64))
        } else {
            None
        };

        let target = self.target.clone();
        let interval_ms = self.interval_ms;
        let interface = self.interface.clone();
        let ipv4 = self.ipv4;
        let ipv6 = self.ipv6;

        future_into_py(py, async move {
            let options = create_ping_options(&target, interval_ms, interface, ipv4, ipv6);
            let start_time = std::time::Instant::now();

            // 在异步上下文中执行ping
            let receiver = execute_ping_async(options).await.map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to start ping: {}", e))
            })?;

            let mut results = Vec::new();
            let mut received_count = 0;

            while received_count < count {
                // 检查是否超时
                if let Some(timeout_duration) = timeout {
                    if start_time.elapsed() >= timeout_duration {
                        break;
                    }
                }

                // 克隆接收器用于当前迭代
                let receiver_clone = receiver.clone();

                // 使用线程池处理阻塞接收
                let result = match tokio::task::spawn_blocking(move || {
                    let guard = receiver_clone.lock().unwrap();
                    guard.recv()
                })
                .await
                {
                    Ok(Ok(result)) => result,
                    Ok(Err(e)) => {
                        return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                            "Channel error: {}",
                            e
                        )))
                    }
                    Err(e) => {
                        return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                            "Task error: {}",
                            e
                        )))
                    }
                };

                let ping_result: PingResult = result.into();
                results.push(ping_result.clone());

                // 如果是退出信号，跳出循环
                if matches!(ping_result, PingResult::PingExited { .. }) {
                    break;
                }

                received_count += 1;

                // 短暂等待避免过度占用CPU
                tokio::time::sleep(Duration::from_millis(10)).await;
            }

            Ok(results)
        })
    }

    fn __repr__(&self) -> String {
        format!(
            "Pinger(target='{}', interval_ms={}, ipv4={}, ipv6={})",
            self.target, self.interval_ms, self.ipv4, self.ipv6
        )
    }
}

// 非阻塞 ping 流处理器
#[pyclass]
pub struct PingStream {
    receiver: Option<std::sync::Arc<std::sync::Mutex<mpsc::Receiver<RustPingResult>>>>,
    max_count: Option<usize>,
    current_count: usize,
}

#[pymethods]
impl PingStream {
    /// 创建新的 PingStream 实例
    #[new]
    #[pyo3(signature = (target, interval_ms=1000, interface=None, ipv4=false, ipv6=false, max_count=None))]
    fn new(
        target: &Bound<PyAny>,
        interval_ms: i64,
        interface: Option<String>,
        ipv4: bool,
        ipv6: bool,
        max_count: Option<usize>,
    ) -> PyResult<Self> {
        // 提取目标地址
        let target_str = extract_target(target)?;

        // 验证 interval_ms 参数
        let interval_ms_u64 = validate_interval_ms(interval_ms, "interval_ms")?;

        // 创建 ping 选项
        let options = create_ping_options(&target_str, interval_ms_u64, interface, ipv4, ipv6);

        // 执行 ping 并获取接收器
        let receiver = match execute_ping(options) {
            Ok(rx) => rx,
            Err(e) => {
                return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Failed to start ping: {}",
                    e
                )))
            }
        };

        // 将接收器包装到 PingStream 中
        Ok(PingStream {
            receiver: Some(std::sync::Arc::new(std::sync::Mutex::new(receiver))),
            max_count,
            current_count: 0,
        })
    }

    fn _recv(&mut self, non_blocking: bool, iter: bool) -> PyResult<Option<PingResult>> {
        // 检查是否达到最大数量
        if let Some(max) = self.max_count {
            if self.current_count >= max {
                self.receiver = None;
                if iter {
                    return Err(PyStopIteration::new_err("Stream exhausted"));
                } else {
                    return Ok(None);
                }
            }
        }
        if let Some(receiver) = &self.receiver {
            let result = {
                let receiver_guard = match receiver.lock() {
                    Ok(guard) => guard,
                    Err(_) => {
                        return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                            "Failed to lock receiver",
                        ))
                    }
                };
                if iter {
                    // 阻塞接收
                    match receiver_guard.recv() {
                        Ok(result) => Ok(Some(result.into())),
                        Err(_) => Err(PyStopIteration::new_err("Stream exhausted")),
                    }
                } else if non_blocking {
                    match receiver_guard.try_recv() {
                        Ok(result) => Ok(Some(result.into())),
                        Err(mpsc::TryRecvError::Empty) => Ok(None),
                        Err(mpsc::TryRecvError::Disconnected) => Ok(None),
                    }
                } else {
                    // 阻塞接收
                    match receiver_guard.recv() {
                        Ok(result) => Ok(Some(result.into())),
                        Err(_) => Ok(None),
                    }
                }
            };

            // 如果接收器已断开连接，则在锁释放后设置 receiver 为 None
            if let Ok(Some(PingResult::PingExited { .. })) = &result {
                self.receiver = None;
                self.current_count += 1;
            } else if let Ok(None) = &result {
                if !non_blocking {
                    // 如果是阻塞接收且没有结果，清空接收器
                    self.receiver = None;
                    self.current_count += 1;
                }
            } else {
                self.current_count += 1;
            }

            result
        } else if iter {
            Err(PyStopIteration::new_err("Stream exhausted"))
        } else {
            Ok(None)
        }
    }

    /// 获取下一个 ping 结果（非阻塞）
    fn try_recv(&mut self) -> PyResult<Option<PingResult>> {
        self._recv(true, false)
    }

    /// 阻塞等待下一个 ping 结果
    fn recv(&mut self) -> PyResult<Option<PingResult>> {
        self._recv(false, false)
    }

    fn __iter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    fn __next__(&mut self) -> PyResult<Option<PingResult>> {
        self._recv(false, true)
    }

    /// 检查流是否仍然活跃
    fn is_active(&self) -> bool {
        if let Some(max) = self.max_count {
            if self.current_count >= max {
                return false;
            }
        }
        self.receiver.is_some()
    }
}

async fn next_ping_stream(
    receiver: std::sync::Arc<std::sync::Mutex<mpsc::Receiver<RustPingResult>>>,
    sync: bool,
) -> PyResult<PingResult> {
    let result = match tokio::task::spawn_blocking(move || {
        let guard = receiver.lock().unwrap();
        guard.recv()
    })
    .await
    {
        Ok(Ok(result)) => result,
        Ok(Err(e)) => {
            return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Channel error: {}",
                e
            )))
        }
        Err(e) => {
            return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Task error: {}",
                e
            )))
        }
    };

    let ping_result: PingResult = result.into();

    // 如果是退出信号，跳出循环
    if matches!(ping_result, PingResult::PingExited { .. }) {
        if sync {
            Err(PyStopIteration::new_err("Stream exhausted"))
        } else {
            Err(PyStopAsyncIteration::new_err("Stream exhausted"))
        }
    } else {
        Ok(ping_result)
    }
}

// 为 AsyncPingStream 创建内部状态结构
struct AsyncPingStreamState {
    options: PingOptions,
    receiver: Option<std::sync::Arc<std::sync::Mutex<mpsc::Receiver<RustPingResult>>>>,
    max_count: Option<usize>,
    current_count: usize,
}

#[pyclass]
pub struct AsyncPingStream {
    // 使用 tokio::sync::Mutex 替换 std::sync::Mutex
    state: std::sync::Arc<tokio::sync::Mutex<AsyncPingStreamState>>,
}

#[pymethods]
impl AsyncPingStream {
    /// 创建新的 AsyncPingStream 实例
    #[new]
    #[pyo3(signature = (target, interval_ms=1000, interface=None, ipv4=false, ipv6=false, max_count=None))]
    fn new(
        target: &Bound<PyAny>,
        interval_ms: i64,
        interface: Option<String>,
        ipv4: bool,
        ipv6: bool,
        max_count: Option<usize>,
    ) -> PyResult<AsyncPingStream> {
        // 提取目标地址
        let target_str = extract_target(target)?;

        // 验证 interval_ms 参数
        let interval_ms_u64 = validate_interval_ms(interval_ms, "interval_ms")?;

        // 创建 ping 选项
        let options = create_ping_options(&target_str, interval_ms_u64, interface, ipv4, ipv6);

        // 创建内部状态
        let state = AsyncPingStreamState {
            options,
            receiver: None,
            max_count,
            current_count: 0,
        };

        // 将状态包装到 Arc<tokio::sync::Mutex<>> 中
        Ok(AsyncPingStream {
            state: std::sync::Arc::new(tokio::sync::Mutex::new(state)),
        })
    }

    // 实现 Python 异步迭代器协议
    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __anext__<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        // 获取状态的克隆，以便在异步闭包中使用
        let state_clone = self.state.clone();

        future_into_py(py, async move {
            // 使用 tokio::sync::Mutex 的 .lock().await 异步锁定状态
            let mut state = state_clone.lock().await;

            // 检查是否达到最大数量
            if let Some(max) = state.max_count {
                if state.current_count >= max {
                    state.receiver = None; // 清空接收器
                    return Err(PyStopAsyncIteration::new_err("Stream exhausted"));
                }
            }

            if let Some(receiver) = &state.receiver {
                let result = next_ping_stream(receiver.clone(), false).await;
                if result.is_ok() {
                    state.current_count += 1;
                }
                result
            } else {
                // 如果接收器不存在，创建新的接收器
                let receiver = execute_ping_async(state.options.clone()).await.map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to start ping: {}", e))
                })?;

                state.receiver = Some(receiver.clone());
                let result = next_ping_stream(receiver, false).await;
                if result.is_ok() {
                    state.current_count += 1;
                }
                result
            }
        })
    }
}

// =================== 模块级函数 ===================

/// 创建非阻塞 ping 流
#[pyfunction]
#[pyo3(signature = (target, interval_ms=1000, interface=None, ipv4=false, ipv6=false, count=None))]
fn create_ping_stream(
    target: &Bound<PyAny>,
    interval_ms: i64,
    interface: Option<String>,
    ipv4: bool,
    ipv6: bool,
    count: Option<usize>,
) -> PyResult<PingStream> {
    // 直接使用 PingStream 的构造函数
    PingStream::new(target, interval_ms, interface, ipv4, ipv6, count)
}

/// 执行单次 ping（同步版本）
#[pyfunction]
#[pyo3(signature = (target, timeout_ms=5000, interface=None, ipv4=false, ipv6=false))]
fn ping_once(
    target: &Bound<PyAny>,
    timeout_ms: i64,
    interface: Option<String>,
    ipv4: bool,
    ipv6: bool,
) -> PyResult<PingResult> {
    // 创建 Pinger 实例
    let pinger = Pinger::new(target, timeout_ms, interface, ipv4, ipv6)?;

    // 执行 ping_once
    pinger.ping_once()
}

/// 执行单次 ping（异步版本）
#[pyfunction]
#[pyo3(signature = (target, timeout_ms=5000, interface=None, ipv4=false, ipv6=false))]
fn ping_once_async<'py>(
    py: Python<'py>,
    target: &Bound<PyAny>,
    timeout_ms: i64,
    interface: Option<String>,
    ipv4: bool,
    ipv6: bool,
) -> PyResult<Bound<'py, PyAny>> {
    // 创建 Pinger 实例
    let pinger = Pinger::new(target, timeout_ms, interface, ipv4, ipv6)?;

    // 执行异步 ping_once
    pinger.ping_once_async(py)
}

/// 执行多次 ping（同步版本）
#[pyfunction]
#[pyo3(signature = (target, count=4, interval_ms=1000, timeout_ms=None, interface=None, ipv4=false, ipv6=false))]
fn ping_multiple(
    target: &Bound<PyAny>,
    count: i32,
    interval_ms: i64,
    timeout_ms: Option<i64>,
    interface: Option<String>,
    ipv4: bool,
    ipv6: bool,
) -> PyResult<Vec<PingResult>> {
    // 创建 Pinger 实例
    let pinger = Pinger::new(target, interval_ms, interface, ipv4, ipv6)?;

    // 执行 ping_multiple
    pinger.ping_multiple(count, timeout_ms)
}

/// 执行多次 ping（异步版本）
#[pyfunction]
#[pyo3(signature = (target, count=4, interval_ms=1000, timeout_ms=None, interface=None, ipv4=false, ipv6=false))]
#[allow(clippy::too_many_arguments)]  // 添加允许多参数的属性
fn ping_multiple_async<'py>(
    py: Python<'py>,
    target: &Bound<PyAny>,
    count: i32,
    interval_ms: i64,
    timeout_ms: Option<i64>,
    interface: Option<String>,
    ipv4: bool,
    ipv6: bool,
) -> PyResult<Bound<'py, PyAny>> {
    // 创建 Pinger 实例
    let pinger = Pinger::new(target, interval_ms, interface, ipv4, ipv6)?;

    // 执行异步 ping_multiple
    pinger.ping_multiple_async(py, count, timeout_ms)
}

pub fn get_ping_rs_version() -> &'static str {
    static VERSION: OnceLock<String> = OnceLock::new();

    VERSION.get_or_init(|| {
        let version = env!("CARGO_PKG_VERSION");
        // cargo uses "1.0-alpha1" etc. while python uses "1.0.0a1", this is not full compatibility,
        // but it's good enough for now
        // see https://docs.rs/semver/1.0.9/semver/struct.Version.html#method.parse for rust spec
        // see https://peps.python.org/pep-0440/ for python spec
        // it seems the dot after "alpha/beta" e.g. "-alpha.1" is not necessary, hence why this works
        version.replace("-alpha", "a").replace("-beta", "b")
    })
}

/// Python 模块定义
#[pymodule]
fn _ping_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // 初始化日志
    pyo3_log::init();

    // 添加类
    m.add_class::<PingResult>()?;
    m.add_class::<Pinger>()?;
    m.add_class::<PingStream>()?;
    m.add_class::<AsyncPingStream>()?; // 添加 AsyncPingStream

    // 添加函数
    m.add_function(wrap_pyfunction!(ping_once, m)?)?;
    m.add_function(wrap_pyfunction!(ping_once_async, m)?)?;
    m.add_function(wrap_pyfunction!(ping_multiple, m)?)?;
    m.add_function(wrap_pyfunction!(ping_multiple_async, m)?)?;
    m.add_function(wrap_pyfunction!(create_ping_stream, m)?)?;

    // 添加版本信息
    m.add("__version__", get_ping_rs_version())?;

    Ok(())
}
