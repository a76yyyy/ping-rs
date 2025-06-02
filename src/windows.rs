use pinger::{PingCreationError, PingOptions, PingResult};
use std::net::{IpAddr, ToSocketAddrs};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::time::sleep;
use winping::Error;
use winping::{AsyncPinger as AsyncWinPinger, Buffer};

// Windows平台专用的ping实现
pub fn ping(options: PingOptions) -> Result<mpsc::Receiver<PingResult>, PingCreationError> {
    let interval = options.interval;
    let target = options.target.to_string();
    let is_ipv4 = target.parse::<IpAddr>().is_ok() && target.parse::<std::net::Ipv4Addr>().is_ok();
    let is_ipv6 = options.target.is_ipv6();

    // 创建通道
    let (tx, rx) = mpsc::channel();

    // 解析目标地址
    let socket_addrs_result = (target.clone(), 0).to_socket_addrs();
    if socket_addrs_result.is_err() {
        // 当解析失败时，发送 PingExited 结果而不是返回错误
        let _ = tx.send(PingResult::PingExited(
            std::process::ExitStatus::default(),
            PingCreationError::HostnameError(target).to_string(),
        ));
        return Ok(rx);
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
        // 发送 PingExited 结果
        let _ = tx.send(PingResult::PingExited(
            std::process::ExitStatus::default(),
            PingCreationError::HostnameError(target).to_string(),
        ));
        return Ok(rx);
    }

    let parsed_ip = selected_ips[0].ip();

    // 创建一个全局的静态tokio运行时
    static RUNTIME: once_cell::sync::Lazy<Runtime> =
        once_cell::sync::Lazy::new(|| Runtime::new().expect("Failed to create tokio runtime"));

    // 使用全局运行时执行异步ping任务
    RUNTIME.spawn(async move {
        // 创建AsyncWinPinger实例
        let mut pinger = AsyncWinPinger::new();

        // 设置超时时间
        let timeout_ms = options.interval.as_millis() as u32;
        pinger.set_timeout(timeout_ms);

        let mut last_ping_time = Instant::now();

        loop {
            // 执行ping操作
            let buffer = Buffer::new();
            let ping_future = pinger.send(parsed_ip, buffer);

            // 异步等待ping结果
            let async_result = ping_future.await;

            match async_result.result {
                Ok(rtt) => {
                    if tx
                        .send(PingResult::Pong(
                            Duration::from_millis(rtt as u64),
                            format!("Reply from {}: time={}ms", parsed_ip, rtt),
                        ))
                        .is_err()
                    {
                        break;
                    }
                }
                Err(e) => {
                    // 判断是否为超时错误类型 Error::Timeout
                    if let Error::Timeout = e {
                        // timeout时不应break，而是继续ping
                        if tx.send(PingResult::Timeout(e.to_string())).is_err() {
                            break;
                        }
                    }
                    // 其他错误类型直接发送错误结果
                    else {
                        let _ = tx.send(PingResult::PingExited(
                            std::process::ExitStatus::default(),
                            e.to_string(),
                        ));
                        break;
                    }
                }
            }

            // 计算下一次ping的等待时间
            // 更新最后ping时间为当前时间
            let now = Instant::now();
            let elapsed = now.duration_since(last_ping_time);

            // 如果已经过去的时间小于间隔，则等待剩余时间
            if elapsed < interval {
                let wait_time = interval - elapsed;
                sleep(wait_time).await;
            }

            // 更新最后ping时间
            last_ping_time = Instant::now();
        }
    });

    Ok(rx)
}
