use pinger::{PingOptions, PingResult};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

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
pub async fn execute_ping_async(
    options: PingOptions,
) -> Result<Arc<Mutex<mpsc::Receiver<PingResult>>>, pinger::PingCreationError> {
    // 在Windows平台使用优化的异步实现
    #[cfg(target_os = "windows")]
    {
        // 获取tokio通道
        let mut receiver = windows::ping_async(options).await?;

        // 创建标准库通道
        let (tx, rx) = mpsc::channel();

        // 创建一个任务来转发消息
        // 这个任务将会在标准通道的发送端关闭时退出
        tokio::spawn(async move {
            while let Some(result) = receiver.recv().await {
                // 当发送失败，说明接收端已关闭，退出循环
                if tx.send(result).is_err() {
                    break;
                }
            }
        });

        // 直接返回包装后的接收端
        // 当 rx 被丢弃时，tx 就会关闭，上面的任务会自然结束
        Ok(Arc::new(Mutex::new(rx)))
    }

    // 在其他平台上使用标准实现并包装为Arc<Mutex>
    #[cfg(not(target_os = "windows"))]
    {
        let receiver = execute_ping(options)?;
        Ok(Arc::new(Mutex::new(receiver)))
    }
}
