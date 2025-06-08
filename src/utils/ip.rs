use pinger::PingCreationError;
use pinger::PingOptions;
use std::net::{IpAddr, ToSocketAddrs};

/// 处理IP地址解析和验证的工具函数
pub fn resolve_ip(target: &str, is_ipv4: bool, is_ipv6: bool) -> Result<IpAddr, PingCreationError> {
    // 解析目标地址
    let socket_addrs_result = (target.to_string(), 0).to_socket_addrs();
    if socket_addrs_result.is_err() {
        return Err(PingCreationError::HostnameError(target.to_string()));
    }

    // 根据IP版本过滤地址
    let selected_ips: Vec<_> = socket_addrs_result
        .unwrap()
        .filter(|addr| {
            if is_ipv6 {
                matches!(addr.ip(), IpAddr::V6(_))
            } else if is_ipv4 {
                matches!(addr.ip(), IpAddr::V4(_))
            } else {
                true // 如果没有指定版本，接受任何版本
            }
        })
        .collect();

    if selected_ips.is_empty() {
        return Err(PingCreationError::HostnameError(target.to_string()));
    }

    Ok(selected_ips[0].ip())
}

/// 处理目标解析并返回有效IP或错误
pub fn process_target(options: &PingOptions) -> (Result<IpAddr, PingCreationError>, bool, bool) {
    let target = &options.target.to_string();
    let is_ipv4 = target.parse::<IpAddr>().is_ok() && target.parse::<std::net::Ipv4Addr>().is_ok();
    let is_ipv6 = options.target.is_ipv6();

    let ip_result = resolve_ip(target, is_ipv4, is_ipv6);
    (ip_result, is_ipv4, is_ipv6)
}
