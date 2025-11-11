use crate::protocols::icmp::platform;
use crate::types::result::PingResult;
use crate::utils::conversion::{create_ping_options, extract_target};
use crate::utils::validation::{validate_interval_ms, validate_timeout_ms};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;
use std::time::{Duration, Instant};

use super::helpers::calculate_timeout_info;

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

            let interval_duration = std::time::Duration::from_millis(interval_ms);

            // 获取异步通道
            let mut receiver = platform::execute_ping_async(options)
                .await
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to start ping: {}", e)))?;

            // 使用 interval 作为超时时间（单次 ping 的最大等待时间）
            match tokio::time::timeout(interval_duration, receiver.recv()).await {
                Ok(Some(result)) => {
                    let ping_result: PingResult = result.into();
                    Ok(ping_result)
                }
                Ok(None) => {
                    // 通道关闭，可能是进程异常退出
                    Err(PyErr::new::<PyRuntimeError, _>("Ping process exited unexpectedly"))
                }
                Err(_) => {
                    // 超时
                    Ok(PingResult::Timeout {
                        line: "Request timeout for icmp_seq 0".to_string(),
                    })
                }
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
            // 不传递 count 给底层 ping 命令，由 Rust 层控制接收数量
            let options = create_ping_options(&target, interval_ms, interface, ipv4, ipv6);
            let start_time = Instant::now();

            // 获取异步通道
            let mut receiver = platform::execute_ping_async(options)
                .await
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to start ping: {}", e)))?;

            let mut results = Vec::new();
            let mut received_count = 0;

            let mut should_stop = false;

            while received_count < count && !should_stop {
                // 计算剩余时间，支持 interval 浮动
                let remaining_time = if let Some(timeout_duration) = timeout {
                    let (should_timeout, remaining, timeout_result) =
                        calculate_timeout_info(start_time, timeout_duration, interval_ms, count, received_count);

                    if should_timeout {
                        // 已经过了超时时间
                        if let Some(result) = timeout_result {
                            results.push(result);
                        }
                        // 标记应该停止，但先尝试接收已经在通道中的结果
                        should_stop = true;
                        // 使用 0 超时来尝试接收已经准备好的结果
                        Some(Duration::from_millis(0))
                    } else {
                        remaining
                    }
                } else {
                    // 没有设置 timeout，无限等待
                    None
                };

                // 使用 timeout 等待下一个结果
                let recv_result = if let Some(timeout_dur) = remaining_time {
                    tokio::time::timeout(timeout_dur, receiver.recv()).await
                } else {
                    Ok(receiver.recv().await)
                };

                match recv_result {
                    Ok(Some(result)) => {
                        let ping_result: PingResult = result.into();

                        // 处理 PingExited
                        if matches!(ping_result, PingResult::PingExited { .. }) {
                            results.push(ping_result);
                            break;
                        }

                        results.push(ping_result);
                        received_count += 1;
                    }
                    Ok(None) => break, // 通道关闭
                    Err(_) => {
                        // tokio::time::timeout 超时
                        if should_stop {
                            // 已经标记要停止，现在真的停止
                            break;
                        }

                        // 检查是否真的到达了总超时时间
                        if let Some(timeout_duration) = timeout {
                            let (should_timeout, _, timeout_result) = calculate_timeout_info(
                                start_time,
                                timeout_duration,
                                interval_ms,
                                count,
                                received_count,
                            );

                            if should_timeout {
                                // 已经到达总超时时间
                                if let Some(result) = timeout_result {
                                    results.push(result);
                                }
                                break;
                            }
                            // 否则继续循环
                        } else {
                            // 没有设置 timeout，不应该超时
                            break;
                        }
                    }
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
