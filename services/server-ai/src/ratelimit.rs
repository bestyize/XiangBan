use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

/// 进程内的失败次数计数：只有失败才记一笔，成功不占额度，所以一次兑换十张码的正常用户不会被锁。
/// 多实例部署时各自计数，Nginx 的 limit_req 仍是第一道防线。
pub struct FailureLimiter {
    max: u32,
    window: Duration,
    hits: Mutex<HashMap<String, (Instant, u32)>>,
}

impl FailureLimiter {
    pub fn new(max: u32, window: Duration) -> Self {
        Self { max, window, hits: Mutex::new(HashMap::new()) }
    }

    /// 已经失败太多次时返回还需要等待的秒数，否则返回 `None`。只读，不计数。
    pub fn blocked(&self, key: &str) -> Option<i64> {
        let now = Instant::now();
        let hits = self.hits.lock().unwrap_or_else(|e| e.into_inner());
        let (started, count) = hits.get(key)?;
        let age = now.duration_since(*started);
        if age >= self.window || *count < self.max {
            return None;
        }
        Some(self.window.saturating_sub(age).as_secs().max(1) as i64)
    }

    pub fn record_failure(&self, key: &str) {
        let now = Instant::now();
        let mut hits = self.hits.lock().unwrap_or_else(|e| e.into_inner());
        if hits.len() > 10_000 {
            hits.retain(|_, (started, _)| now.duration_since(*started) < self.window);
        }
        let entry = hits.entry(key.to_string()).or_insert((now, 0));
        if now.duration_since(entry.0) >= self.window {
            *entry = (now, 0);
        }
        entry.1 += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_recorded_failures_count_and_the_block_lifts_after_the_window() {
        let limiter = FailureLimiter::new(2, Duration::from_millis(60));
        for _ in 0..10 {
            assert!(limiter.blocked("a").is_none(), "checking must not consume the allowance");
        }
        limiter.record_failure("a");
        assert!(limiter.blocked("a").is_none());
        limiter.record_failure("a");
        assert!(limiter.blocked("a").is_some_and(|wait| wait >= 1));
        assert!(limiter.blocked("b").is_none(), "other keys are independent");
        std::thread::sleep(Duration::from_millis(70));
        assert!(limiter.blocked("a").is_none());
        limiter.record_failure("a");
        assert!(limiter.blocked("a").is_none(), "a new window starts counting from zero");
    }
}
