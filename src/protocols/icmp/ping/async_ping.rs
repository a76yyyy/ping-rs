use crate::protocols::icmp::platform;
use crate::types::result::PingResult;
use crate::utils::conversion::{create_ping_options, extract_target};
use crate::utils::validation::{validate_interval_ms, validate_timeout_ms};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;

/// Python 包装的异步 Pinger 类
#[pyclass]
pub struct AsyncPinger {
    target: String,
    interval_ms: u64,
    interface: Option<String>,
    ipv4: bool,
    ipv6: bool,
}

#[pymethods]
impl AsyncPinger {
    #[new]
    #[pyo3(signature = (target, interval_ms=1000, interface=None, ipv4=false, ipv6=false))]
    pub fn new(
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

    /// 异步执行单次ping
    pub fn ping_once<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let target = self.target.clone();
        let interval_ms = self.interval_ms;
        let interface = self.interface.clone();
        let ipv4 = self.ipv4;
        let ipv6 = self.ipv6;

        future_into_py(py, async move {
            let options = create_ping_options(&target, interval_ms, interface, ipv4, ipv6);

            // 获取异步通道
            let mut receiver = platform::execute_ping_async(options)
                .await
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to start ping: {}", e)))?;

            // ✅ 直接 await，无需 spawn_blocking
            match receiver.recv().await {
                Some(result) => {
                    let ping_result: PingResult = result.into();
                    Ok(ping_result)
                }
                None => Err(PyErr::new::<PyRuntimeError, _>("Failed to receive ping result")),
            }
        })
    }

    /// 异步执行多次 ping
    #[pyo3(signature = (count=4, timeout_ms=None))]
    pub fn ping_multiple<'py>(
        &self,
        py: Python<'py>,
        count: i32,
        timeout_ms: Option<i64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        // 验证 count 参数
        let count = crate::utils::validation::validate_count(count, "count")?;

        // 验证 timeout_ms 参数
        let timeout = validate_timeout_ms(timeout_ms, self.interval_ms, "timeout_ms")?;

        let target = self.target.clone();
        let interval_ms = self.interval_ms;
        let interface = self.interface.clone();
        let ipv4 = self.ipv4;
        let ipv6 = self.ipv6;

        future_into_py(py, async move {
            let options = create_ping_options(&target, interval_ms, interface, ipv4, ipv6);
            let start_time = std::time::Instant::now();

            // 获取异步通道
            let mut receiver = platform::execute_ping_async(options)
                .await
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to start ping: {}", e)))?;

            let mut results = Vec::new();
            let mut received_count = 0;

            while received_count < count {
                // 检查总超时
                if let Some(timeout_duration) = timeout {
                    if start_time.elapsed() >= timeout_duration {
                        break;
                    }
                }

                // ✅ 直接 await，无需 spawn_blocking
                match receiver.recv().await {
                    Some(result) => {
                        let ping_result: PingResult = result.into();
                        results.push(ping_result.clone());

                        // 如果是退出信号，跳出循环
                        if matches!(ping_result, PingResult::PingExited { .. }) {
                            break;
                        }

                        received_count += 1;
                    }
                    None => break, // 通道关闭
                }
            }

            Ok(results)
        })
    }

    pub fn __repr__(&self) -> String {
        format!(
            "AsyncPinger(target='{}', interval_ms={}, ipv4={}, ipv6={})",
            self.target, self.interval_ms, self.ipv4, self.ipv6
        )
    }
}
