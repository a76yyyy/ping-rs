/// Ping 辅助函数模块
///
/// 包含各种辅助函数，如超时计算等
use crate::types::result::PingResult;
use std::time::{Duration, Instant};

/// 计算超时相关的信息
///
/// # 参数
/// - `start_time`: 开始时间
/// - `timeout_duration`: 总超时时间
/// - `interval_ms`: ping 间隔（毫秒）
/// - `count`: 总共要发送的包数量
/// - `received_count`: 已经收到的响应数量
///
/// # 返回
/// - `bool`: 是否应该超时（已过宽限期）
/// - `Option<Duration>`: 剩余等待时间
/// - `Option<PingResult>`: 如果需要构造 Timeout，返回 Timeout 结果
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
        ((elapsed.as_millis() - 1) / interval_ms as u128) as usize
    } else {
        0
    };
    let last_completed_seq = last_completed_seq.min(count - 1);

    // 如果已经收到了所有应该完成的包,就不需要超时结果
    if received_count > last_completed_seq {
        return (true, None, None);
    }

    // 构造超时结果
    let timeout_result = Some(PingResult::Timeout {
        line: format!("Request timeout for icmp_seq {}", last_completed_seq),
    });

    (true, None, timeout_result)
}
