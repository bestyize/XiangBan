//! 两种限流：连接内的令牌桶（消息数与字节数），以及按键的滑动窗口（配对码兑换、申请）。
use std::{
    collections::{HashMap, VecDeque},
    sync::Mutex,
    time::{Duration, Instant},
};

/// 令牌桶：容量为速率的 2 倍，允许短时突发。
pub struct Bucket {
    rate: f64,
    capacity: f64,
    tokens: f64,
    last: Instant,
}

impl Bucket {
    pub fn new(rate: f64) -> Self {
        Self { rate, capacity: rate * 2.0, tokens: rate * 2.0, last: Instant::now() }
    }

    pub fn take(&mut self, amount: f64) -> bool {
        self.take_at(amount, Instant::now())
    }

    pub fn take_at(&mut self, amount: f64, now: Instant) -> bool {
        let elapsed = now.saturating_duration_since(self.last).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.rate).min(self.capacity);
        self.last = now;
        if self.tokens >= amount {
            self.tokens -= amount;
            true
        } else {
            false
        }
    }
}

/// 滑动窗口计数：`window` 内最多 `limit` 次。
#[derive(Default)]
pub struct WindowLimiter {
    hits: Mutex<HashMap<String, VecDeque<Instant>>>,
}

impl WindowLimiter {
    /// 允许则记一次并返回 None；超限返回需要等待的秒数。
    pub fn hit(&self, key: &str, limit: usize, window: Duration) -> Option<i64> {
        self.hit_at(key, limit, window, Instant::now())
    }

    pub fn hit_at(&self, key: &str, limit: usize, window: Duration, now: Instant) -> Option<i64> {
        let mut map = self.hits.lock().unwrap();
        let q = map.entry(key.to_string()).or_default();
        while q.front().is_some_and(|t| now.saturating_duration_since(*t) >= window) {
            q.pop_front();
        }
        if q.len() >= limit {
            let wait = window.saturating_sub(now.saturating_duration_since(*q.front().unwrap()));
            return Some(wait.as_secs() as i64 + 1);
        }
        q.push_back(now);
        // 顺手清理空键，避免无限增长。
        if map.len() > 10_000 {
            map.retain(|_, v| v.back().is_some_and(|t| now.saturating_duration_since(*t) < window));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_allows_burst_then_refills() {
        let mut b = Bucket::new(10.0);
        let t0 = Instant::now();
        assert!(b.take_at(20.0, t0));
        assert!(!b.take_at(1.0, t0));
        assert!(b.take_at(5.0, t0 + Duration::from_millis(500)));
        assert!(!b.take_at(6.0, t0 + Duration::from_millis(500)));
    }

    #[test]
    fn window_limits_and_expires() {
        let l = WindowLimiter::default();
        let t0 = Instant::now();
        let w = Duration::from_secs(60);
        for _ in 0..3 {
            assert_eq!(l.hit_at("a", 3, w, t0), None);
        }
        assert!(l.hit_at("a", 3, w, t0).is_some());
        assert_eq!(l.hit_at("b", 3, w, t0), None, "不同键互不影响");
        assert_eq!(l.hit_at("a", 3, w, t0 + Duration::from_secs(61)), None);
    }
}
