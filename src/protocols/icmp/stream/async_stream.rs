use crate::protocols::icmp::platform;
use crate::types::result::PingResult;
use crate::utils::conversion::{create_ping_options, extract_target};
use crate::utils::validation::validate_interval_ms;
use pinger::PingOptions;
use pinger::PingResult as RustPingResult;
use pyo3::exceptions::{PyRuntimeError, PyStopAsyncIteration};
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;
use std::sync::mpsc;
use std::sync::Arc;

async fn next_ping_stream(receiver: Arc<std::sync::Mutex<mpsc::Receiver<RustPingResult>>>) -> PyResult<PingResult> {
    let result = match tokio::task::spawn_blocking(move || {
        let guard = receiver.lock().unwrap();
        guard.recv()
    })
    .await
    {
        Ok(Ok(result)) => result,
        Ok(Err(e)) => return Err(PyErr::new::<PyRuntimeError, _>(format!("Channel error: {}", e))),
        Err(e) => return Err(PyErr::new::<PyRuntimeError, _>(format!("Task error: {}", e))),
    };

    let ping_result: PingResult = result.into();

    // 如果是退出信号，跳出循环
    if matches!(ping_result, PingResult::PingExited { .. }) {
        Err(PyStopAsyncIteration::new_err("Stream exhausted"))
    } else {
        Ok(ping_result)
    }
}

// 为 AsyncPingStream 创建内部状态结构
struct AsyncPingStreamState {
    options: PingOptions,
    receiver: Option<Arc<std::sync::Mutex<mpsc::Receiver<RustPingResult>>>>,
    max_count: Option<usize>,
    current_count: usize,
}

#[pyclass]
pub struct AsyncPingStream {
    // 使用 tokio::sync::Mutex 替换 std::sync::Mutex
    state: Arc<tokio::sync::Mutex<AsyncPingStreamState>>,
}

#[pymethods]
impl AsyncPingStream {
    /// 创建新的 AsyncPingStream 实例
    #[new]
    #[pyo3(signature = (target, interval_ms=1000, interface=None, ipv4=false, ipv6=false, max_count=None))]
    pub fn new(
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

        // 验证 max_count 如果有的话
        if let Some(count) = max_count {
            crate::utils::validation::validate_count(count.try_into().unwrap(), "max_count")?;
        }

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
            state: Arc::new(tokio::sync::Mutex::new(state)),
        })
    }

    // 实现 Python 异步迭代器协议
    pub fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __anext__<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
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
                let result = next_ping_stream(receiver.clone()).await;
                if result.is_ok() {
                    state.current_count += 1;
                }
                result
            } else {
                // 如果接收器不存在，创建新的接收器
                let receiver = platform::execute_ping_async(state.options.clone())
                    .await
                    .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to start ping: {}", e)))?;

                state.receiver = Some(receiver.clone());
                let result = next_ping_stream(receiver).await;
                if result.is_ok() {
                    state.current_count += 1;
                }
                result
            }
        })
    }
}
