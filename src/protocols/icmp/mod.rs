// ICMP协议实现
pub mod ping;
pub mod platform;
pub mod stream;

use crate::types::options::DnsPreResolveOptions;
use pinger::{PingOptions, PingResult};
use std::sync::mpsc;

/// 执行ping操作的统一接口，返回标准库的通道
///
/// 所有平台统一使用 pinger 库的实现
///
/// # 参数
///
/// - `options`: Ping 选项配置
/// - `dns_options`: DNS 预解析选项配置
///
/// # 特殊处理
///
/// 将 `PingCreationError::HostnameError` 转换为 `PingResult::PingExited`，
/// 以保持与测试和用户期望的一致性。这样做的原因是:
///
/// - Linux/macOS: 主机名解析失败由 ping 命令处理，返回错误输出
/// - Windows: 主机名解析在 Rust 层完成，需要手动转换为结果
///
/// 其他类型的 `PingCreationError` (如 `UnknownPing`, `SpawnError`, `NotSupported`)
/// 仍然会作为错误返回，因为它们表示环境问题而非目标问题。
///
/// # 示例
///
/// ```rust
/// use ping_rs::types::options::DnsPreResolveOptions;
/// use ping_rs::protocols::icmp::execute_ping;
///
/// // 使用默认 DNS 预解析选项（启用，超时为 options.interval）
/// // let receiver = execute_ping(options, DnsPreResolveOptions::default())?;
///
/// // 禁用 DNS 预解析
/// // let dns_opts = DnsPreResolveOptions { enable: false, timeout: None };
/// // let receiver = execute_ping(options, dns_opts)?;
/// ```
pub fn execute_ping(
    mut options: PingOptions,
    dns_options: DnsPreResolveOptions,
) -> Result<mpsc::Receiver<PingResult>, pinger::PingCreationError> {
    // 尝试预解析主机名，以避免 ping 命令解析超时或卡住
    if dns_options.enable {
        if let pinger::target::Target::Hostname { .. } = &options.target {
            let target = options.target.clone();
            let resolve_timeout = dns_options.timeout.unwrap_or(options.interval);

            let (tx_resolve, rx_resolve) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let result = pinger::utils::resolve_target(&target);
                let _ = tx_resolve.send(result);
            });

            match rx_resolve.recv_timeout(resolve_timeout) {
                Ok(Ok(ip)) => {
                    // 解析成功，更新 target 为 IP，避免 ping 命令再次解析
                    options.target = pinger::target::Target::IP(ip);
                }
                Ok(Err(e)) => {
                    // 解析失败，直接返回 PingExited
                    let (tx, rx) = mpsc::channel();
                    let _ = tx.send(PingResult::PingExited(
                        std::process::ExitStatus::default(),
                        e.to_string(),
                    ));
                    return Ok(rx);
                }
                Err(_) => {
                    // 解析超时，直接返回 PingExited
                    let (tx, rx) = mpsc::channel();
                    let _ = tx.send(PingResult::PingExited(
                        std::process::ExitStatus::default(),
                        "Hostname resolution timeout".to_string(),
                    ));
                    return Ok(rx);
                }
            }
        }
    }

    match pinger::ping(options) {
        Ok(rx) => Ok(rx),
        Err(e @ pinger::PingCreationError::HostnameError(_)) => {
            // 主机名解析失败，创建一个返回错误结果的接收器
            let (tx, rx) = mpsc::channel();
            let _ = tx.send(PingResult::PingExited(
                std::process::ExitStatus::default(),
                e.to_string(),
            ));
            Ok(rx)
        }
        Err(e) => Err(e), // 其他错误继续传播
    }
}

/// 异步执行ping操作，返回 tokio 异步通道
///
/// 所有平台统一使用 pinger 库的实现
///
/// # 参数
///
/// - `options`: Ping 选项配置
/// - `dns_options`: DNS 预解析选项配置
///
/// # 特殊处理
///
/// 将 `PingCreationError::HostnameError` 转换为 `PingResult::PingExited`，
/// 以保持与测试和用户期望的一致性。这样做的原因是:
///
/// - Linux/macOS: 主机名解析失败由 ping 命令处理，返回错误输出
/// - Windows: 主机名解析在 Rust 层完成，需要手动转换为结果
///
/// 其他类型的 `PingCreationError` (如 `UnknownPing`, `SpawnError`, `NotSupported`)
/// 仍然会作为错误返回，因为它们表示环境问题而非目标问题。
///
/// # 示例
///
/// ```rust
/// use ping_rs::types::options::DnsPreResolveOptions;
/// use ping_rs::protocols::icmp::execute_ping_async;
///
/// // 使用默认 DNS 预解析选项（启用，超时为 options.interval）
/// // let receiver = execute_ping_async(options, DnsPreResolveOptions::default()).await?;
///
/// // 禁用 DNS 预解析
/// // let dns_opts = DnsPreResolveOptions { enable: false, timeout: None };
/// // let receiver = execute_ping_async(options, dns_opts).await?;
/// ```
pub async fn execute_ping_async(
    mut options: PingOptions,
    dns_options: DnsPreResolveOptions,
) -> Result<tokio::sync::mpsc::UnboundedReceiver<PingResult>, pinger::PingCreationError> {
    // 尝试预解析主机名，以避免 ping 命令解析超时或卡住
    if dns_options.enable {
        if let pinger::target::Target::Hostname { .. } = &options.target {
            let resolve_timeout = dns_options.timeout.unwrap_or(options.interval);
            match tokio::time::timeout(resolve_timeout, pinger::utils::resolve_target_async(&options.target)).await {
                Ok(Ok(ip)) => {
                    // 解析成功，更新 target 为 IP，避免 ping 命令再次解析
                    options.target = pinger::target::Target::IP(ip);
                }
                Ok(Err(e)) => {
                    // 解析失败，直接返回 PingExited
                    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
                    let _ = tx.send(PingResult::PingExited(
                        std::process::ExitStatus::default(),
                        e.to_string(),
                    ));
                    return Ok(rx);
                }
                Err(_) => {
                    // 解析超时，返回错误
                    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
                    let _ = tx.send(PingResult::PingExited(
                        std::process::ExitStatus::default(),
                        "Hostname resolution timeout".to_string(),
                    ));
                    return Ok(rx);
                }
            }
        }
    }

    match pinger::ping_async(options).await {
        Ok(rx) => Ok(rx),
        Err(e @ pinger::PingCreationError::HostnameError(_)) => {
            // 主机名解析失败，创建一个返回错误结果的接收器
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            let _ = tx.send(PingResult::PingExited(
                std::process::ExitStatus::default(),
                e.to_string(),
            ));
            Ok(rx)
        }
        Err(e) => Err(e), // 其他错误继续传播
    }
}
