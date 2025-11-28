use crate::protocols::icmp::platform;
use crate::types::result::PingResult;
use crate::utils::conversion::{create_ping_options, extract_target};
use crate::utils::validation::validate_interval_ms;
use pinger::PingOptions;
use pinger::PingResult as RustPingResult;
use pyo3::exceptions::{PyRuntimeError, PyStopAsyncIteration};
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;
use std::sync::Arc;

async fn next_ping_stream(receiver: &mut tokio::sync::mpsc::UnboundedReceiver<RustPingResult>) -> PyResult<PingResult> {
    // ✅ 直接 await，无需 spawn_blocking
    match receiver.recv().await {
        Some(result) => {
            let ping_result: PingResult = result.into();

            // 如果是退出信号，跳出循环
            if matches!(ping_result, PingResult::PingExited { .. }) {
                Err(PyStopAsyncIteration::new_err("Stream exhausted"))
            } else {
                Ok(ping_result)
            }
        }
        None => Err(PyStopAsyncIteration::new_err("Stream exhausted")),
    }
}

// 为 AsyncPingStream 创建内部状态结构
struct AsyncPingStreamState {
    options: PingOptions,
    receiver: Option<tokio::sync::mpsc::UnboundedReceiver<RustPingResult>>,
    max_count: Option<usize>,
    current_count: usize,
}

/// Asynchronous ping stream for continuous ping operations
///
/// This struct provides an async iterator interface for streaming ping results.
#[pyclass]
pub struct AsyncPingStream {
    // 使用 tokio::sync::Mutex 替换 std::sync::Mutex
    state: Arc<tokio::sync::Mutex<AsyncPingStreamState>>,
}

#[pymethods]
impl AsyncPingStream {
    /// 创建新的 `AsyncPingStream` 实例
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
            let count_i32 = count.try_into().map_err(|_| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "max_count ({count}) is too large to convert to i32"
                ))
            })?;
            crate::utils::validation::validate_count(count_i32, "max_count")?;
        }

        // 创建 ping 选项（不传递 count 给底层 ping 命令）
        // max_count 参数保存在 state 中，在 __anext__ 迭代时由 Rust 层控制
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

    /// Python async iterator protocol: return self
    pub fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Python async iterator protocol: get next ping result
    ///
    /// Returns the next ping result or raises `StopAsyncIteration` when the stream is exhausted.
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

            if let Some(receiver) = &mut state.receiver {
                let result = next_ping_stream(receiver).await;
                if result.is_ok() {
                    state.current_count += 1;
                }
                result
            } else {
                // 如果接收器不存在，创建新的接收器
                let mut receiver = platform::execute_ping_async(state.options.clone())
                    .await
                    .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to start ping: {e}")))?;

                let result = next_ping_stream(&mut receiver).await;
                if result.is_ok() {
                    state.current_count += 1;
                }
                state.receiver = Some(receiver);
                result
            }
        })
    }
}
