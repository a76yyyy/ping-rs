use pyo3::prelude::*;
use std::time::Duration;

/// 验证 `interval_ms` 参数并转换为 u64
///
/// 由于 ping 命令的 -i 参数格式化为一位小数，所以 `interval_ms` 必须是 100ms 的倍数且不小于 100ms
pub fn validate_interval_ms(value: i64, param_name: &str) -> PyResult<u64> {
    if value < 0 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "{param_name} must be a non-negative integer"
        )));
    }
    if value < 100 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "{param_name} must be at least 100ms"
        )));
    }
    if value % 100 != 0 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "{param_name} must be a multiple of 100ms due to ping command's decimal precision"
        )));
    }
    Ok(value as u64)
}

/// 验证 count 参数并转换为 usize
pub fn validate_count(count: i32, param_name: &str) -> PyResult<usize> {
    if count <= 0 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "{param_name} ({count}) must be a positive integer"
        )));
    }
    Ok(count as usize)
}

/// 验证 `timeout_ms` 参数并转换为 Duration
///
/// 如果 `timeout_ms` 为 None，返回 None
/// 否则验证 `timeout_ms` 必须大于等于 `interval_ms`
pub fn validate_timeout_ms(timeout_ms: Option<i64>, interval_ms: u64, param_name: &str) -> PyResult<Option<Duration>> {
    match timeout_ms {
        Some(timeout) => {
            let timeout_ms_u64 = validate_interval_ms(timeout, param_name)?;

            // 确保 timeout_ms 大于 interval_ms
            if timeout_ms_u64 < interval_ms {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "{param_name} ({timeout_ms_u64} ms) must be greater than or equal to interval_ms ({interval_ms} ms)"
                )));
            }

            Ok(Some(Duration::from_millis(timeout_ms_u64)))
        }
        None => Ok(None),
    }
}
