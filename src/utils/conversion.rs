use pinger::PingOptions;
use pyo3::prelude::*;
use std::net::IpAddr;
use std::time::Duration;

/// 从 Python 对象中提取 IP 地址字符串
pub fn extract_target(target: &Bound<PyAny>) -> PyResult<String> {
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

/// 创建 PingOptions 配置
pub fn create_ping_options(
    target: &str,
    interval_ms: u64,
    interface: Option<String>,
    ipv4: bool,
    ipv6: bool,
) -> PingOptions {
    let interval = Duration::from_millis(interval_ms);

    if ipv4 {
        PingOptions::new_ipv4(target, interval, interface)
    } else if ipv6 {
        PingOptions::new_ipv6(target, interval, interface)
    } else {
        PingOptions::new(target, interval, interface)
    }
}
