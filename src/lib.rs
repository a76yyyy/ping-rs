use pinger::{PingOptions, PingResult as RustPingResult};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3_async_runtimes::tokio::future_into_py;
use std::net::IpAddr;
use std::sync::mpsc;
use std::sync::OnceLock;
use std::time::Duration;

// 添加条件编译模块
#[cfg(target_os = "windows")]
mod windows;

#[cfg(not(target_os = "windows"))]
use pinger::ping;

// 根据平台选择适当的 ping 函数
#[cfg(target_os = "windows")]
fn platform_ping(options: PingOptions) -> Result<mpsc::Receiver<RustPingResult>, pinger::PingCreationError> {
    windows::ping(options)
}

#[cfg(not(target_os = "windows"))]
fn platform_ping(options: PingOptions) -> Result<mpsc::Receiver<RustPingResult>, pinger::PingCreationError> {
    ping(options)
}

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
            PingResult::Pong { duration_ms, line } => {
                format!("PingResult.Pong(duration_ms={}ms, line='{}')", duration_ms, line)
            }
            PingResult::Timeout { line } => format!("PingResult.Timeout(line='{}')", line),
            PingResult::Unknown { line } => format!("PingResult.Unknown(line='{}')", line),
            PingResult::PingExited { exit_code, stderr } => {
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
            PingResult::Pong { duration_ms, .. } => Some(*duration_ms),
            _ => None,
        }
    }

    /// 获取原始行内容
    #[getter]
    fn line(&self) -> String {
        match self {
            PingResult::Pong { line, .. } => line.clone(),
            PingResult::Timeout { line } => line.clone(),
            PingResult::Unknown { line } => line.clone(),
            PingResult::PingExited { stderr, .. } => stderr.clone(),
        }
    }

    /// 获取退出代码，如果不是 PingExited 则返回 None
    #[getter]
    fn exit_code(&self) -> Option<i32> {
        match self {
            PingResult::PingExited { exit_code, .. } => Some(*exit_code),
            _ => None,
        }
    }

    /// 获取标准错误输出，如果不是 PingExited 则返回 None
    #[getter]
    fn stderr(&self) -> Option<String> {
        match self {
            PingResult::PingExited { stderr, .. } => Some(stderr.clone()),
            _ => None,
        }
    }

    /// 检查是否为成功的 ping
    fn is_success(&self) -> bool {
        matches!(self, PingResult::Pong { .. })
    }

    /// 检查是否为超时
    fn is_timeout(&self) -> bool {
        matches!(self, PingResult::Timeout { .. })
    }

    /// 检查是否为未知响应
    fn is_unknown(&self) -> bool {
        matches!(self, PingResult::Unknown { .. })
    }

    /// 检查是否为 ping 进程退出
    fn is_exited(&self) -> bool {
        matches!(self, PingResult::PingExited { .. })
    }

    /// 获取 PingResult 的类型名称
    #[getter]
    fn type_name(&self) -> String {
        match self {
            PingResult::Pong { .. } => "Pong".to_string(),
            PingResult::Timeout { .. } => "Timeout".to_string(),
            PingResult::Unknown { .. } => "Unknown".to_string(),
            PingResult::PingExited { .. } => "PingExited".to_string(),
        }
    }

    /// 将 PingResult 转换为字典
    fn to_dict(&self, py: Python) -> PyResult<PyObject> {
        let dict = match self {
            PingResult::Pong { duration_ms, line } => {
                let dict = PyDict::new(py);
                dict.set_item("type", "Pong")?;
                dict.set_item("duration_ms", *duration_ms)?;
                dict.set_item("line", line.clone())?;
                dict
            }
            PingResult::Timeout { line } => {
                let dict = PyDict::new(py);
                dict.set_item("type", "Timeout")?;
                dict.set_item("line", line.clone())?;
                dict
            }
            PingResult::Unknown { line } => {
                let dict = PyDict::new(py);
                dict.set_item("type", "Unknown")?;
                dict.set_item("line", line.clone())?;
                dict
            }
            PingResult::PingExited { exit_code, stderr } => {
                let dict = PyDict::new(py);
                dict.set_item("type", "PingExited")?;
                dict.set_item("exit_code", *exit_code)?;
                dict.set_item("stderr", stderr.clone())?;
                dict
            }
        };
        Ok(dict.into())
    }
}

impl From<RustPingResult> for PingResult {
    fn from(result: RustPingResult) -> Self {
        match result {
            RustPingResult::Pong(duration, line) => PingResult::Pong {
                duration_ms: duration.as_secs_f64() * 1000.0,
                line,
            },
            RustPingResult::Timeout(line) => PingResult::Timeout { line },
            RustPingResult::Unknown(line) => PingResult::Unknown { line },
            RustPingResult::PingExited(status, stderr) => PingResult::PingExited {
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
        let interval = Duration::from_millis(self.interval_ms);

        let options = if self.ipv4 {
            PingOptions::new_ipv4(&self.target, interval, self.interface.clone())
        } else if self.ipv6 {
            PingOptions::new_ipv6(&self.target, interval, self.interface.clone())
        } else {
            PingOptions::new(&self.target, interval, self.interface.clone())
        };

        // 使用平台特定的 ping 函数
        let receiver = platform_ping(options)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to start ping: {}", e)))?;

        // 等待第一个结果
        match receiver.recv() {
            Ok(result) => Ok(result.into()),
            Err(_) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Failed to receive ping result",
            )),
        }
    }

    /// 异步执行连续 ping
    #[pyo3(signature = (count=None))]
    fn ping_stream<'py>(&self, py: Python<'py>, count: Option<usize>) -> PyResult<Bound<'py, PyAny>> {
        let target = self.target.clone();
        let interval_ms = self.interval_ms;
        let interface = self.interface.clone();
        let ipv4 = self.ipv4;
        let ipv6 = self.ipv6;

        future_into_py(py, async move {
            let interval = Duration::from_millis(interval_ms);

            let options = if ipv4 {
                PingOptions::new_ipv4(&target, interval, interface)
            } else if ipv6 {
                PingOptions::new_ipv6(&target, interval, interface)
            } else {
                PingOptions::new(&target, interval, interface)
            };

            // 使用平台特定的 ping 函数
            let receiver = platform_ping(options).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to start ping: {}", e))
            })?;

            let mut results = Vec::<PingResult>::new();
            let mut received_count = 0;

            // 创建一个可以在多个任务间共享的接收器
            let receiver = std::sync::Arc::new(std::sync::Mutex::new(receiver));

            // 循环接收ping结果，每次接收都在单独的阻塞任务中执行
            loop {
                // 克隆接收器以在阻塞任务中使用
                let receiver_clone = receiver.clone();

                // 在阻塞线程中执行单次接收操作
                let result = match tokio::task::spawn_blocking(move || {
                    let receiver_guard = receiver_clone.lock().unwrap();
                    receiver_guard.recv()
                })
                .await
                {
                    Ok(result) => result,
                    Err(e) => {
                        return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                            "Failed to receive ping result: {}",
                            e
                        )));
                    }
                };

                // 处理接收到的结果
                match result {
                    // 如果接收出错，返回错误
                    Err(e) => {
                        return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                            "Failed to receive ping result: {}",
                            e
                        )));
                    }
                    // 如果接收成功，处理结果
                    Ok(result) => {
                        // 将结果转换为 Python 类型
                        let ping_result: PingResult = result.into();
                        // 添加到结果列表
                        results.push(ping_result.clone());
                        // 如果是退出信号，跳出循环
                        if matches!(ping_result, PingResult::PingExited { .. }) {
                            break;
                        }

                        received_count += 1;
                    }
                }

                // 如果指定了数量限制，检查是否达到
                if let Some(max_count) = count {
                    if received_count >= max_count {
                        break;
                    }
                }

                // 等待一小段时间再继续，允许其他任务执行
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

/// 便捷函数：执行单次 ping（同步版本）
#[pyfunction]
#[pyo3(signature = (target, timeout_ms=5000, interface=None, ipv4=false, ipv6=false))]
fn ping_once(
    target: &Bound<PyAny>,
    timeout_ms: i64,
    interface: Option<String>,
    ipv4: bool,
    ipv6: bool,
) -> PyResult<PingResult> {
    let target_str = extract_target(target)?;

    // 验证 timeout_ms 参数
    let timeout_ms_u64 = validate_interval_ms(timeout_ms, "timeout_ms")?;

    let pinger = Pinger {
        target: target_str,
        interval_ms: timeout_ms_u64,
        interface,
        ipv4,
        ipv6,
    };

    pinger.ping_once()
}

/// 便捷函数：执行单次 ping（异步版本）
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
    let target_str = extract_target(target)?;

    // 验证 timeout_ms 参数
    let timeout_ms_u64 = validate_interval_ms(timeout_ms, "timeout_ms")?;

    let pinger = Pinger {
        target: target_str,
        interval_ms: timeout_ms_u64,
        interface,
        ipv4,
        ipv6,
    };

    future_into_py(py, async move {
        // 将阻塞的ping_once操作移到专门的线程池中执行
        let handle = tokio::task::spawn_blocking(move || pinger.ping_once());

        // 等待阻塞操作完成
        handle
            .await
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to execute ping: {}", e)))?
    })
}

/// 便捷函数：执行多次 ping（同步版本）
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
    let target_str = extract_target(target)?;

    // 验证 count 参数
    if count <= 0 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "count ({}) must be a positive integer",
            count
        )));
    }
    let count = count as usize; // 转换为 usize 用于后续处理

    // 验证 interval_ms 参数
    let interval_ms_u64 = validate_interval_ms(interval_ms, "interval_ms")?;

    // 验证 timeout_ms 参数
    let timeout_ms_u64 = if let Some(timeout) = timeout_ms {
        // 验证 timeout_ms 不为负数
        let timeout_u64 = validate_interval_ms(timeout, "timeout_ms")?;

        if timeout < interval_ms {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "timeout_ms ({}) must be greater than or equal to interval_ms ({})",
                timeout, interval_ms
            )));
        }
        Some(timeout_u64)
    } else {
        None
    };

    let interval = Duration::from_millis(interval_ms_u64);
    let timeout = timeout_ms_u64.map(Duration::from_millis);
    let start_time = std::time::Instant::now();

    let options = if ipv4 {
        PingOptions::new_ipv4(&target_str, interval, interface.clone())
    } else if ipv6 {
        PingOptions::new_ipv6(&target_str, interval, interface.clone())
    } else {
        PingOptions::new(&target_str, interval, interface.clone())
    };

    // 使用平台特定的 ping 函数
    let receiver = platform_ping(options)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to start ping: {}", e)))?;

    let mut results = Vec::<PingResult>::new();
    let mut received_count = 0;

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

/// 便捷函数：执行多次 ping（异步版本）
#[pyfunction]
#[pyo3(signature = (target, count=4, interval_ms=1000, timeout_ms=None, interface=None, ipv4=false, ipv6=false))]
#[allow(clippy::too_many_arguments)]
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
    let target_str = extract_target(target)?;

    // 验证 count 参数
    if count <= 0 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "count ({}) must be a positive integer",
            count
        )));
    }
    let count = count as usize; // 转换为 usize 用于后续处理

    // 验证 interval_ms 参数
    let interval_ms_u64 = validate_interval_ms(interval_ms, "interval_ms")?;

    // 验证 timeout_ms 参数
    let timeout_ms_u64 = if let Some(timeout) = timeout_ms {
        // 验证 timeout_ms 不为负数
        let timeout_u64 = validate_interval_ms(timeout, "timeout_ms")?;

        if timeout < interval_ms {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "timeout_ms ({}) must be greater than or equal to interval_ms ({})",
                timeout, interval_ms
            )));
        }
        Some(timeout_u64)
    } else {
        None
    };

    future_into_py(py, async move {
        let interval = Duration::from_millis(interval_ms_u64);
        let timeout = timeout_ms_u64.map(Duration::from_millis);
        let options = if ipv4 {
            PingOptions::new_ipv4(&target_str, interval, interface.clone())
        } else if ipv6 {
            PingOptions::new_ipv6(&target_str, interval, interface.clone())
        } else {
            PingOptions::new(&target_str, interval, interface.clone())
        };

        // 使用平台特定的 ping 函数
        let receiver = platform_ping(options)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to start ping: {}", e)))?;

        let start_time = std::time::Instant::now();
        let mut results = Vec::<PingResult>::new();
        let mut received_count = 0;

        // 创建一个可以在多个任务间共享的接收器
        let receiver = std::sync::Arc::new(std::sync::Mutex::new(receiver));

        // 循环接收ping结果，每次接收都在单独的阻塞任务中执行
        while received_count < count {
            // 检查是否超时
            if let Some(timeout_duration) = timeout {
                if start_time.elapsed() >= timeout_duration {
                    break;
                }
            }

            // 克隆接收器以在阻塞任务中使用
            let receiver_clone = receiver.clone();

            // 在阻塞线程中执行单次接收操作
            let result = match tokio::task::spawn_blocking(move || {
                let receiver_guard = receiver_clone.lock().unwrap();
                receiver_guard.recv()
            })
            .await
            {
                Ok(result) => result,
                Err(e) => {
                    return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                        "Failed to receive ping result: {}",
                        e
                    )));
                }
            };

            // 处理接收到的结果
            match result {
                // 如果接收出错，返回错误
                Err(e) => {
                    return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                        "Failed to receive ping result: {}",
                        e
                    )));
                }
                // 如果接收成功，处理结果
                Ok(result) => {
                    // 将结果转换为 Python 类型
                    let ping_result: PingResult = result.into();

                    // 添加到结果列表
                    results.push(ping_result.clone());

                    // 如果是退出信号，跳出循环
                    if matches!(ping_result, PingResult::PingExited { .. }) {
                        break;
                    }

                    received_count += 1;
                }
            }

            // 如果还需要接收更多结果，等待一小段时间再继续
            // 这里使用非阻塞的sleep，允许其他任务执行
            if received_count < count {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }

        Ok(results)
    })
}

/// 非阻塞 ping 流处理器
#[pyclass]
pub struct PingStream {
    receiver: Option<std::sync::Arc<std::sync::Mutex<mpsc::Receiver<RustPingResult>>>>,
}

#[pymethods]
impl PingStream {
    /// 获取下一个 ping 结果（非阻塞）
    fn try_recv(&mut self) -> PyResult<Option<PingResult>> {
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
                match receiver_guard.try_recv() {
                    Ok(result) => Ok(Some(result.into())),
                    Err(mpsc::TryRecvError::Empty) => Ok(None),
                    Err(mpsc::TryRecvError::Disconnected) => Ok(None),
                }
            };

            // 如果接收器已断开连接，则在锁释放后设置 receiver 为 None
            if let Ok(Some(PingResult::PingExited { .. })) = &result {
                self.receiver = None;
            }

            result
        } else {
            Ok(None)
        }
    }

    /// 阻塞等待下一个 ping 结果
    fn recv(&mut self) -> PyResult<Option<PingResult>> {
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
                match receiver_guard.recv() {
                    Ok(result) => Ok(Some(result.into())),
                    Err(_) => Ok(None),
                }
            };

            // 如果接收器已断开连接，则在锁释放后设置 receiver 为 None
            if let Ok(Some(PingResult::PingExited { .. })) = &result {
                self.receiver = None;
            } else if let Ok(None) = &result {
                self.receiver = None;
            }

            result
        } else {
            Ok(None)
        }
    }

    /// 检查流是否仍然活跃
    fn is_active(&self) -> bool {
        self.receiver.is_some()
    }
}

/// 创建非阻塞 ping 流
#[pyfunction]
#[pyo3(signature = (target, interval_ms=1000, interface=None, ipv4=false, ipv6=false))]
fn create_ping_stream(
    target: &Bound<PyAny>,
    interval_ms: i64,
    interface: Option<String>,
    ipv4: bool,
    ipv6: bool,
) -> PyResult<PingStream> {
    let target_str = extract_target(target)?;

    // 验证 interval_ms 参数
    let interval_ms_u64 = validate_interval_ms(interval_ms, "interval_ms")?;

    let interval = Duration::from_millis(interval_ms_u64);
    let options = if ipv4 {
        PingOptions::new_ipv4(&target_str, interval, interface)
    } else if ipv6 {
        PingOptions::new_ipv6(&target_str, interval, interface)
    } else {
        PingOptions::new(&target_str, interval, interface)
    };

    // 使用平台特定的 ping 函数
    let receiver = platform_ping(options)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to start ping: {}", e)))?;

    Ok(PingStream {
        receiver: Some(std::sync::Arc::new(std::sync::Mutex::new(receiver))),
    })
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
