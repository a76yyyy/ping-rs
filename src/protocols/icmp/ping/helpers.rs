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

    // 已经超过 timeout，计算是否需要宽限期
    let interval_duration = Duration::from_millis(interval_ms);

    // 计算最后一个"应该已经发送"的包的序号（从0开始）
    // 使用 (elapsed - 1) 来避免边界问题
    let last_sent_seq = if elapsed.as_millis() > 0 {
        ((elapsed.as_millis() - 1) / interval_ms as u128) as usize
    } else {
        0
    };
    let last_sent_seq = last_sent_seq.min(count - 1);

    // 最后一个包的理论发送时间
    let last_sent_time = start_time + interval_duration * last_sent_seq as u32;

    // 应该等到的时间 = 最后发送时间 + interval（给响应时间）
    let grace_deadline = last_sent_time + interval_duration;

    if now >= grace_deadline {
        // 已经过了宽限期
        let timeout_result = if received_count <= last_sent_seq {
            Some(PingResult::Timeout {
                line: format!("Request timeout for icmp_seq {}", last_sent_seq),
            })
        } else {
            None
        };
        (true, None, timeout_result)
    } else {
        // 还在宽限期内，继续等待
        (false, Some(grace_deadline - now), None)
    }
}
