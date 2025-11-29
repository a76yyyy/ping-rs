use pyo3::prelude::*;
use std::time::Duration;

/// 将正整数 i64 转换为 u64，用于 DNS 解析超时等参数
///
/// 如果值为负数或零，返回错误
pub fn i64_to_u64_positive(value: i64, param_name: &str) -> PyResult<u64> {
    if value <= 0 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "{param_name} must be a positive integer, got {value}"
        )));
    }
    // 已验证 value > 0，使用 try_from 进行类型转换
    u64::try_from(value).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{param_name} ({value}) conversion to u64 failed: {e}"))
    })
}

/// 将正整数 i32 转换为 usize，用于 count 等参数
///
/// 如果值为负数或零，返回错误
pub fn i32_to_usize_positive(value: i32, param_name: &str) -> PyResult<usize> {
    if value <= 0 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "{param_name} ({value}) must be a positive integer"
        )));
    }
    // 已验证 value > 0，使用 try_from 进行类型转换
    usize::try_from(value).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "{param_name} ({value}) conversion to usize failed: {e}"
        ))
    })
}

/// 验证 `interval_ms` 参数并转换为 u64
///
/// 由于 ping 命令的 -i 参数格式化为一位小数，所以 `interval_ms` 必须是 100ms 的倍数且不小于 100ms
pub fn validate_interval_ms(value: i64, param_name: &str) -> PyResult<u64> {
    let value_u64 = i64_to_u64_positive(value, param_name)?;

    if value_u64 < 100 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "{param_name} must be at least 100ms"
        )));
    }
    if value_u64 % 100 != 0 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "{param_name} must be a multiple of 100ms due to ping command's decimal precision"
        )));
    }
    Ok(value_u64)
}

/// 验证 count 参数并转换为 usize
pub fn validate_count(count: i32, param_name: &str) -> PyResult<usize> {
    i32_to_usize_positive(count, param_name)
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
