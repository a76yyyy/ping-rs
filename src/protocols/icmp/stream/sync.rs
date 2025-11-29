use crate::protocols::icmp::execute_ping;
use crate::types::options::DnsPreResolveOptions;
use crate::types::result::PingResult;
use crate::utils::conversion::{create_ping_options, extract_target};
use crate::utils::validation::validate_interval_ms;
use pinger::PingResult as RustPingResult;
use pyo3::exceptions::{PyRuntimeError, PyStopIteration};
use pyo3::prelude::*;
use std::sync::mpsc;

/// Synchronous ping stream for continuous ping operations
///
/// This struct provides an iterator interface for streaming ping results.
#[pyclass]
pub struct PingStream {
    receiver: Option<std::sync::Arc<std::sync::Mutex<mpsc::Receiver<RustPingResult>>>>,
    max_count: Option<usize>,
    current_count: usize,
}

#[pymethods]
impl PingStream {
    /// 创建新的 `PingStream` 实例
    ///
    /// # Errors
    /// - `PyValueError`: If `interval_ms` is negative, less than 100ms, not a multiple of 100ms, or `max_count` is too large
    /// - `PyTypeError`: If the target cannot be converted to a string
    /// - `PyRuntimeError`: If the ping process fails to start
    #[new]
    #[pyo3(signature = (target, interval_ms=1000, interface=None, ipv4=false, ipv6=false, max_count=None, dns_pre_resolve=true, dns_resolve_timeout_ms=None))]
    pub fn new(
        target: &Bound<PyAny>,
        interval_ms: i64,
        interface: Option<String>,
        ipv4: bool,
        ipv6: bool,
        max_count: Option<usize>,
        dns_pre_resolve: bool,
        dns_resolve_timeout_ms: Option<i64>,
    ) -> PyResult<Self> {
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

        // 处理 DNS 超时参数
        let dns_timeout = if let Some(timeout_ms) = dns_resolve_timeout_ms {
            let timeout_u64 = crate::utils::validation::i64_to_u64_positive(timeout_ms, "dns_resolve_timeout_ms")?;
            Some(std::time::Duration::from_millis(timeout_u64))
        } else {
            None
        };

        // 创建 ping 选项（不传递 count 给底层 ping 命令）
        // max_count 参数保存在 state 中，在迭代时由 Rust 层控制
        let options = create_ping_options(&target_str, interval_ms_u64, interface, ipv4, ipv6);

        let dns_options = DnsPreResolveOptions {
            enable: dns_pre_resolve,
            timeout: dns_timeout,
        };

        // 执行 ping 并获取接收器
        let receiver = match execute_ping(options, dns_options) {
            Ok(rx) => rx,
            Err(e) => return Err(PyErr::new::<PyRuntimeError, _>(format!("Failed to start ping: {e}"))),
        };

        // 将接收器包装到 PingStream 中
        Ok(PingStream {
            receiver: Some(std::sync::Arc::new(std::sync::Mutex::new(receiver))),
            max_count,
            current_count: 0,
        })
    }

    #[allow(clippy::used_underscore_items)]
    fn _recv(&mut self, non_blocking: bool, iter: bool) -> PyResult<Option<PingResult>> {
        // 检查是否达到最大数量
        if let Some(max) = self.max_count {
            if self.current_count >= max {
                self.receiver = None;
                if iter {
                    return Err(PyStopIteration::new_err("Stream exhausted"));
                }
                return Ok(None);
            }
        }
        if let Some(receiver) = &self.receiver {
            let result = {
                let Ok(receiver_guard) = receiver.lock() else {
                    return Err(PyErr::new::<PyRuntimeError, _>("Failed to lock receiver"));
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
    ///
    /// # Errors
    /// - `PyRuntimeError`: If the receiver mutex lock fails
    #[allow(clippy::used_underscore_items)]
    pub fn try_recv(&mut self) -> PyResult<Option<PingResult>> {
        self._recv(true, false)
    }

    /// 阻塞等待下一个 ping 结果
    ///
    /// # Errors
    /// - `PyRuntimeError`: If the receiver mutex lock fails
    #[allow(clippy::used_underscore_items)]
    pub fn recv(&mut self) -> PyResult<Option<PingResult>> {
        self._recv(false, false)
    }

    /// Python iterator protocol: return self
    pub fn __iter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    /// Python iterator protocol: get next ping result
    ///
    /// Returns the next ping result or raises `StopIteration` when the stream is exhausted.
    ///
    /// # Errors
    /// - `PyStopIteration`: When the stream is exhausted (`max_count` reached or ping process exited)
    /// - `PyRuntimeError`: If the receiver mutex lock fails
    #[allow(clippy::used_underscore_items)]
    pub fn __next__(&mut self) -> PyResult<Option<PingResult>> {
        self._recv(false, true)
    }

    /// 检查流是否仍然活跃
    pub fn is_active(&self) -> bool {
        if let Some(max) = self.max_count {
            if self.current_count >= max {
                return false;
            }
        }
        self.receiver.is_some()
    }
}
