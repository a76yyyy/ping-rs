/// Ping 辅助函数模块
///
/// 包含各种辅助函数，如超时计算等
use crate::types::result::PingResult;
use std::time::{Duration, Instant};

/// Unix/Linux/macOS 平台的超时计算
///
/// 在这些平台上,ping 命令持续运行,我们在 Rust 层控制接收数量和超时
/// 需要计算"已完成等待的包"
#[cfg(not(target_os = "windows"))]
pub fn calculate_timeout_info(
    start_time: Instant,
    timeout_duration: Duration,
    interval_ms: u64,
    count: usize,
    received_count: usize,
) -> (bool, Option<Duration>, Option<PingResult>) {
    let now = Instant::now();
    let elapsed = now.duration_since(start_time);

    if elapsed < timeout_duration {
        // 还没到 timeout，正常等待
        return (false, Some(timeout_duration - elapsed), None);
    }

    // 已经超过 timeout，计算最后一个已完成等待的包
    // 计算最后一个"已经完成等待"的包的序号（从0开始）
    // 在时刻 t,已经完成等待的包是那些发送时间 <= t - interval 的包
    // 例如: t=3000ms, interval=500ms
    //   - seq 0 在 0ms 发送,在 500ms 完成等待
    //   - seq 5 在 2500ms 发送,在 3000ms 完成等待
    //   - seq 6 在 3000ms 发送,还在等待中
    // 所以 last_completed_seq = (3000 - 1) / 500 = 5
    let last_completed_seq = if elapsed.as_millis() > 0 {
        ((elapsed.as_millis() - 1) / u128::from(interval_ms)) as usize
    } else {
        0
    };
    let last_completed_seq = last_completed_seq.min(count - 1);

    // 如果已经收到了所有应该完成的包,就不需要超时结果
    // received_count 是已收到的包数量,last_completed_seq 是最后一个应该完成的包的序号
    // 例如: received_count=6 表示收到了 seq 0-5 共 6 个包
    //      last_completed_seq=6 表示应该完成 seq 0-6 共 7 个包
    // 所以应该检查 received_count > last_completed_seq
    if received_count > last_completed_seq {
        return (true, None, None);
    }

    // 构造超时结果,返回第一个未收到的包
    // 例如: received_count=5, last_completed_seq=6
    //      应该返回 seq 5 的超时结果
    let timeout_seq = received_count;
    let timeout_result = Some(PingResult::Timeout {
        line: format!("Request timeout for icmp_seq {timeout_seq}"),
    });

    (true, None, timeout_result)
}

/// Windows 平台的超时计算
///
/// 在 Windows 上,每次 ping 调用都会立即返回(成功或超时)
/// 我们在循环中多次调用 ping,需要计算"应该发送的包数"
#[cfg(target_os = "windows")]
pub fn calculate_timeout_info(
    start_time: Instant,
    timeout_duration: Duration,
    interval_ms: u64,
    count: usize,
    received_count: usize,
) -> (bool, Option<Duration>, Option<PingResult>) {
    let now = Instant::now();
    let elapsed = now.duration_since(start_time);

    if elapsed < timeout_duration {
        // 还没到 timeout，正常等待
        return (false, Some(timeout_duration - elapsed), None);
    }

    // 已经超过 timeout
    // 在 Windows 上,计算应该发送的包数
    // 第一个包在 t=0 发送,第二个包在 t=interval 发送,以此类推
    // 在时刻 timeout_duration,应该发送的包数是 ceil(timeout_duration / interval)
    // 例如: timeout=3200ms, interval=500ms
    //   - seq 0 在 0ms 发送
    //   - seq 1 在 500ms 发送
    //   - seq 2 在 1000ms 发送
    //   - seq 3 在 1500ms 发送
    //   - seq 4 在 2000ms 发送
    //   - seq 5 在 2500ms 发送
    //   - seq 6 在 3000ms 发送
    //   - seq 7 在 3500ms 发送 (超过 3200ms,不应该发送)
    // 所以应该发送 7 个包 (seq 0-6)
    // 使用向上取整: ceil(timeout / interval) = (timeout + interval - 1) / interval
    let timeout_ms = timeout_duration.as_millis() as u64;
    let expected_count = ((timeout_ms + interval_ms - 1) / interval_ms) as usize;
    let expected_count = expected_count.min(count);

    // 如果已经收到了足够的包,就不需要超时结果
    if received_count >= expected_count {
        return (true, None, None);
    }

    // 构造超时结果,返回第一个未收到的包
    let timeout_seq = received_count;
    let timeout_result = Some(PingResult::Timeout {
        line: format!("Request timeout for icmp_seq {}", timeout_seq),
    });

    (true, None, timeout_result)
}
