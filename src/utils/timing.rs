use std::time::{Duration, Instant};

/// 计算下次 ping 的等待时间
pub fn wait_for_next_interval(last_ping_time: Instant, interval: Duration) -> Duration {
    let now = Instant::now();
    let elapsed = now.duration_since(last_ping_time);
    if elapsed < interval {
        interval - elapsed
    } else {
        Duration::from_millis(0)
    }
}
