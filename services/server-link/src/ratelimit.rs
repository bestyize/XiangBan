//! 连接内的令牌桶限流（消息数与字节数）。
use std::time::Instant;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn bucket_allows_burst_then_refills() {
        let mut b = Bucket::new(10.0);
        let t0 = Instant::now();
        assert!(b.take_at(20.0, t0));
        assert!(!b.take_at(1.0, t0));
        assert!(b.take_at(5.0, t0 + Duration::from_millis(500)));
        assert!(!b.take_at(6.0, t0 + Duration::from_millis(500)));
    }
}
