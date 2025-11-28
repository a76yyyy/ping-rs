use pinger::PingResult as RustPingResult;
use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Python 包装的 `PingResult` 枚举
#[pyclass]
#[derive(Debug, Clone)]
pub enum PingResult {
    /// 成功的 ping 响应，包含延迟时间（毫秒）和原始行
    Pong {
        /// 延迟时间（毫秒）
        duration_ms: f64,
        /// 原始输出行
        line: String,
    },
    /// 超时
    Timeout {
        /// 原始输出行
        line: String,
    },
    /// 未知响应
    Unknown {
        /// 原始输出行
        line: String,
    },
    /// Ping 进程退出
    PingExited {
        /// 退出代码
        exit_code: i32,
        /// 标准错误输出
        stderr: String,
    },
}

#[pymethods]
impl PingResult {
    /// Python `__repr__` method for string representation
    pub fn __repr__(&self) -> String {
        match self {
            Self::Pong { duration_ms, line } => {
                format!("PingResult.Pong(duration_ms={duration_ms}ms, line='{line}')")
            }
            Self::Timeout { line } => format!("PingResult.Timeout(line='{line}')"),
            Self::Unknown { line } => format!("PingResult.Unknown(line='{line}')"),
            Self::PingExited { exit_code, stderr } => {
                format!("PingResult.PingExited(exit_code={exit_code}, stderr='{stderr}')")
            }
        }
    }

    /// Python `__str__` method for string representation
    pub fn __str__(&self) -> String {
        self.__repr__()
    }

    /// 获取延迟时间（毫秒），如果不是 Pong 则返回 None
    #[getter]
    pub fn duration_ms(&self) -> Option<f64> {
        match self {
            Self::Pong { duration_ms, .. } => Some(*duration_ms),
            _ => None,
        }
    }

    /// 获取原始行内容
    #[getter]
    pub fn line(&self) -> String {
        match self {
            Self::Pong { line, .. } => line.clone(),
            Self::Timeout { line } => line.clone(),
            Self::Unknown { line } => line.clone(),
            Self::PingExited { stderr, .. } => stderr.clone(),
        }
    }

    /// 获取退出代码，如果不是 `PingExited` 则返回 None
    #[getter]
    pub fn exit_code(&self) -> Option<i32> {
        match self {
            Self::PingExited { exit_code, .. } => Some(*exit_code),
            _ => None,
        }
    }

    /// 获取标准错误输出，如果不是 `PingExited` 则返回 None
    #[getter]
    pub fn stderr(&self) -> Option<String> {
        match self {
            Self::PingExited { stderr, .. } => Some(stderr.clone()),
            _ => None,
        }
    }

    /// 检查是否为成功的 ping
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Pong { .. })
    }

    /// 检查是否为超时
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout { .. })
    }

    /// 检查是否为未知响应
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown { .. })
    }

    /// 检查是否为 ping 进程退出
    pub fn is_exited(&self) -> bool {
        matches!(self, Self::PingExited { .. })
    }

    /// 获取 `PingResult` 的类型名称
    #[getter]
    pub fn type_name(&self) -> String {
        match self {
            Self::Pong { .. } => "Pong".to_string(),
            Self::Timeout { .. } => "Timeout".to_string(),
            Self::Unknown { .. } => "Unknown".to_string(),
            Self::PingExited { .. } => "PingExited".to_string(),
        }
    }

    /// 将 `PingResult` 转换为字典
    ///
    /// # Errors
    /// - `PyErr`: If failed to set dictionary items (rare, usually indicates memory issues)
    pub fn to_dict(&self, py: Python) -> PyResult<Py<PyAny>> {
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
        }

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
