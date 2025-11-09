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

/// 异步执行ping操作，在支持的平台上使用纯异步实现，否则使用兼容的方式
/// 为异步操作提供基础
///
/// 返回 tokio 异步通道，避免在上层频繁使用 spawn_blocking
pub async fn execute_ping_async(
    options: PingOptions,
) -> Result<tokio::sync::mpsc::UnboundedReceiver<PingResult>, pinger::PingCreationError> {
    // 在Windows平台使用优化的异步实现
    #[cfg(target_os = "windows")]
    {
        // Windows 平台直接返回 tokio 通道
        windows::ping_async(options).await
    }

    // 在其他平台上使用标准实现，通过一次 spawn_blocking 转换为 tokio 通道
    #[cfg(not(target_os = "windows"))]
    {
        let receiver = execute_ping(options)?;
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        // ✅ 只需要一次 spawn_blocking，在后台持续转发消息
        tokio::task::spawn_blocking(move || {
            while let Ok(result) = receiver.recv() {
                // 当发送失败，说明接收端已关闭，退出循环
                if tx.send(result).is_err() {
                    break;
                }
            }
        });

        Ok(rx)
    }
}
