use crate::utils::ip::process_target;
use crate::utils::timing::wait_for_next_interval;
use pinger::{PingCreationError, PingOptions, PingResult};
use std::net::IpAddr;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use winping::Error;
use winping::{AsyncPinger as AsyncWinPinger, Buffer, Pinger as WinPinger};

// 创建并配置 WinPinger
fn create_pinger(interval: Duration) -> WinPinger {
    let mut pinger = WinPinger::new().unwrap();
    let timeout_ms = interval.as_millis() as u32;
    pinger.set_timeout(timeout_ms);
    pinger
}

// 创建并配置 AsyncWinPinger
fn create_pinger_async(interval: Duration) -> AsyncWinPinger {
    let mut pinger = AsyncWinPinger::new();
    let timeout_ms = interval.as_millis() as u32;
    pinger.set_timeout(timeout_ms);
    pinger
}

// 处理 ping 超时和错误的通用函数
fn handle_ping_error(error: Error, send_timeout: impl FnOnce() -> bool, send_error: impl FnOnce() -> bool) -> bool {
    if let Error::Timeout = error {
        // timeout - 继续ping
        send_timeout()
    } else {
        // 其他错误 - 发送错误并退出
        send_error();
        false
    }
}

/// Windows平台专用的 ping 实现（通过标准库的channel通信）
pub fn ping(options: PingOptions) -> Result<mpsc::Receiver<PingResult>, PingCreationError> {
    let interval = options.interval;

    // 创建通道
    let (tx, rx) = mpsc::channel();

    // 解析IP地址
    let (ip_result, _, _) = process_target(&options);

    let parsed_ip = match ip_result {
        Ok(ip) => ip,
        Err(e) => {
            // 当解析失败时，发送错误结果并返回接收器
            let _ = tx.send(PingResult::PingExited(
                std::process::ExitStatus::default(),
                e.to_string(),
            ));
            return Ok(rx);
        }
    };

    // 执行ping操作
    thread::spawn(move || {
        // 创建 pinger
        let pinger = create_pinger(interval);
        process_ping_task(pinger, parsed_ip, interval, tx);
    });

    Ok(rx)
}

// 处理同步风格的ping任务
fn process_ping_task(pinger: WinPinger, target_ip: IpAddr, interval: Duration, tx: mpsc::Sender<PingResult>) {
    let mut last_ping_time = Instant::now();

    loop {
        let buffer = &mut Buffer::new();

        // 发送ping请求
        match pinger.send(target_ip, buffer) {
            Ok(rtt) => {
                if tx
                    .send(PingResult::Pong(
                        Duration::from_millis(rtt as u64),
                        format!("Reply from {}: time={}ms", target_ip, rtt),
                    ))
                    .is_err()
                {
                    break;
                }
            }
            Err(e) => {
                let should_continue = handle_ping_error(
                    e,
                    || tx.send(PingResult::Timeout(e.to_string())).is_ok(),
                    || {
                        let _ = tx.send(PingResult::PingExited(
                            std::process::ExitStatus::default(),
                            e.to_string(),
                        ));
                        true
                    },
                );

                if !should_continue {
                    break;
                }
            }
        }

        // 计算等待时间
        let wait_time = wait_for_next_interval(last_ping_time, interval);
        thread::sleep(wait_time);
        last_ping_time = Instant::now();
    }
}

/// Windows平台专用的异步ping实现（返回tokio通道）
pub async fn ping_async(
    options: PingOptions,
) -> Result<tokio::sync::mpsc::UnboundedReceiver<PingResult>, PingCreationError> {
    let interval = options.interval;

    // 创建tokio通道
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    // 解析IP地址
    let (ip_result, _, _) = process_target(&options);

    let parsed_ip = match ip_result {
        Ok(ip) => ip,
        Err(e) => {
            // 当解析失败时，发送错误结果并返回接收器
            let _ = tx.send(PingResult::PingExited(
                std::process::ExitStatus::default(),
                e.to_string(),
            ));
            return Ok(rx);
        }
    };

    // 创建pinger
    let pinger = create_pinger_async(interval);

    // 在单独的tokio任务中执行ping
    tokio::spawn(async move {
        process_ping_async_task(pinger, parsed_ip, interval, tx).await;
    });

    Ok(rx)
}

// 处理异步风格的ping任务
async fn process_ping_async_task(
    pinger: AsyncWinPinger,
    target_ip: IpAddr,
    interval: Duration,
    tx: tokio::sync::mpsc::UnboundedSender<PingResult>,
) {
    let mut last_ping_time = Instant::now();

    loop {
        let buffer = Buffer::new();
        let ping_future = pinger.send(target_ip, buffer);

        // 等待ping结果
        let async_result = ping_future.await;

        match async_result.result {
            Ok(rtt) => {
                let ping_result = PingResult::Pong(
                    Duration::from_millis(rtt as u64),
                    format!("Reply from {}: time={}ms", target_ip, rtt),
                );

                if tx.send(ping_result).is_err() {
                    break;
                }
            }
            Err(e) => {
                let should_continue = handle_ping_error(
                    e,
                    || tx.send(PingResult::Timeout(e.to_string())).is_ok(),
                    || {
                        let _ = tx.send(PingResult::PingExited(
                            std::process::ExitStatus::default(),
                            e.to_string(),
                        ));
                        true
                    },
                );

                if !should_continue {
                    break;
                }
            }
        }

        // 计算等待时间
        let wait_time = wait_for_next_interval(last_ping_time, interval);
        tokio::time::sleep(wait_time).await;
        last_ping_time = Instant::now();
    }
}
