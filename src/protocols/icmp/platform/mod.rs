use pinger::{PingOptions, PingResult};
use std::sync::mpsc;

// 平台特定实现
#[cfg(target_os = "windows")]
pub mod windows;

/// 执行ping操作的统一接口，返回标准库的通道
/// 抽象平台差异，为同步操作提供基础
pub fn execute_ping(options: PingOptions) -> Result<mpsc::Receiver<PingResult>, pinger::PingCreationError> {
    #[cfg(target_os = "windows")]
    {
        windows::ping(options)
    }

    #[cfg(not(target_os = "windows"))]
    {
        pinger::ping(options)
    }
}

/// 异步执行ping操作，在支持的平台上使用纯异步实现
/// 为异步操作提供基础
///
/// 返回 tokio 异步通道（unbounded）
pub async fn execute_ping_async(
    options: PingOptions,
) -> Result<tokio::sync::mpsc::UnboundedReceiver<PingResult>, pinger::PingCreationError> {
    // 在Windows平台使用优化的异步实现
    #[cfg(target_os = "windows")]
    {
        windows::ping_async(options).await
    }

    // 在其他平台上使用 pinger::ping_async()
    #[cfg(not(target_os = "windows"))]
    {
        // 直接返回 pinger 的 unbounded receiver，无需转发
        pinger::ping_async(options).await
    }
}
