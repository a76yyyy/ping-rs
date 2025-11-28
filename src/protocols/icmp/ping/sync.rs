use crate::protocols::icmp::platform;
use crate::types::result::PingResult;
use crate::utils::conversion::{create_ping_options, extract_target};
use crate::utils::validation::{validate_interval_ms, validate_timeout_ms};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

use super::helpers::calculate_timeout_info;

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

    /// 同步执行单次 ping
    pub fn ping_once(&self) -> PyResult<PingResult> {
        let options = create_ping_options(
            &self.target,
            self.interval_ms,
            self.interface.clone(),
            self.ipv4,
            self.ipv6,
        );

        // 执行ping并等待第一个结果
        let receiver = platform::execute_ping(options)
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to start ping: {e}")))?;

        // 使用 interval 作为超时时间
        let timeout = std::time::Duration::from_millis(self.interval_ms);

        // 等待第一个结果（带超时）
        match receiver.recv_timeout(timeout) {
            Ok(result) => {
                let ping_result: PingResult = result.into();
                Ok(ping_result)
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // 超时，主动构造 Timeout 结果
                Ok(PingResult::Timeout {
                    line: "Request timeout for icmp_seq 0".to_string(),
                })
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                // 通道断开，可能是进程异常退出
                Err(PyErr::new::<PyRuntimeError, _>("Ping process disconnected"))
            }
        }
    }

    /// 同步执行多次 ping
    #[pyo3(signature = (count=4, timeout_ms=None))]
    pub fn ping_multiple(&self, count: i32, timeout_ms: Option<i64>) -> PyResult<Vec<PingResult>> {
        // 验证 count 参数
        let count = crate::utils::validation::validate_count(count, "count")?;

        // 验证 timeout_ms 参数
        let timeout = validate_timeout_ms(timeout_ms, self.interval_ms, "timeout_ms")?;

        // 不传递 count 给底层 ping 命令，由 Rust 层控制接收数量
        let options = create_ping_options(
            &self.target,
            self.interval_ms,
            self.interface.clone(),
            self.ipv4,
            self.ipv6,
        );

        // 执行ping
        let receiver = platform::execute_ping(options)
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to start ping: {e}")))?;

        let mut results = Vec::new();
        let mut received_count = 0;
        let start_time = std::time::Instant::now();

        loop {
            // 检查是否达到指定数量
            if received_count >= count {
                break;
            }

            // 计算剩余时间
            let remaining_timeout = if let Some(timeout_duration) = timeout {
                let (should_timeout, remaining, timeout_result) =
                    calculate_timeout_info(start_time, timeout_duration, self.interval_ms, count, received_count);

                if should_timeout {
                    // 已经过了宽限期
                    if let Some(result) = timeout_result {
                        results.push(result);
                    }
                    break;
                }
                remaining
            } else {
                // 没有设置 timeout，使用一个较大的值
                Some(std::time::Duration::from_secs(3600))
            };

            // 等待下一个结果（带超时）
            let recv_result = if let Some(timeout_dur) = remaining_timeout {
                receiver.recv_timeout(timeout_dur)
            } else {
                receiver
                    .recv()
                    .map_err(|_| std::sync::mpsc::RecvTimeoutError::Disconnected)
            };

            match recv_result {
                Ok(result) => {
                    let ping_result: PingResult = result.into();

                    // 如果收到 PingExited，说明进程异常退出（因为我们不使用 -c 参数）
                    // 这通常表示网络错误或权限问题
                    if matches!(ping_result, PingResult::PingExited { .. }) {
                        results.push(ping_result);
                        break;
                    }

                    // 添加到结果列表
                    results.push(ping_result);
                    received_count += 1;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // 超时，检查是否需要构造最后一个包的 Timeout
                    if let Some(timeout_duration) = timeout {
                        let (_, _, timeout_result) = calculate_timeout_info(
                            start_time,
                            timeout_duration,
                            self.interval_ms,
                            count,
                            received_count,
                        );

                        if let Some(result) = timeout_result {
                            results.push(result);
                        }
                    }
                    break;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    // 通道断开，退出循环
                    break;
                }
            }
        }

        Ok(results)
    }

    pub fn __repr__(&self) -> String {
        format!(
            "Pinger(target='{}', interval_ms={}, ipv4={}, ipv6={})",
            self.target, self.interval_ms, self.ipv4, self.ipv6
        )
    }
}
