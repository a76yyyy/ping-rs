use pinger::{PingOptions, PingResult};
use std::sync::mpsc;

/// 执行ping操作的统一接口，返回标准库的通道
///
/// 所有平台统一使用 pinger 库的实现
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
pub fn execute_ping(options: PingOptions) -> Result<mpsc::Receiver<PingResult>, pinger::PingCreationError> {
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
pub async fn execute_ping_async(
    options: PingOptions,
) -> Result<tokio::sync::mpsc::UnboundedReceiver<PingResult>, pinger::PingCreationError> {
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
