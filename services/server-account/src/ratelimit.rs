use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

/// 进程内的按键固定窗口计数，只用来挡住无需登录的查询接口（如用户名查重）被批量探测。
/// 多实例部署时各自计数，Nginx 的 limit_req 仍是第一道防线。
pub struct WindowLimiter {
    max: u32,
    window: Duration,
    hits: Mutex<HashMap<String, (Instant, u32)>>,
}

impl WindowLimiter {
    pub fn new(max: u32, window: Duration) -> Self {
        Self { max, window, hits: Mutex::new(HashMap::new()) }
    }

    /// 允许则返回 `Ok`，超限返回需要等待的秒数。
    pub fn check(&self, key: &str) -> Result<(), i64> {
        let now = Instant::now();
        let mut hits = self.hits.lock().unwrap_or_else(|e| e.into_inner());
        if hits.len() > 10_000 {
            hits.retain(|_, (started, _)| now.duration_since(*started) < self.window);
        }
        let entry = hits.entry(key.to_string()).or_insert((now, 0));
        if now.duration_since(entry.0) >= self.window {
            *entry = (now, 0);
        }
        if entry.1 >= self.max {
            let wait = self.window.saturating_sub(now.duration_since(entry.0)).as_secs() as i64;
            return Err(wait.max(1));
        }
        entry.1 += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_after_the_limit_per_key_and_recovers_after_the_window() {
        let limiter = WindowLimiter::new(2, Duration::from_millis(40));
        assert!(limiter.check("a").is_ok());
        assert!(limiter.check("a").is_ok());
        assert!(limiter.check("a").is_err());
        assert!(limiter.check("b").is_ok(), "other keys are independent");
        std::thread::sleep(Duration::from_millis(50));
        assert!(limiter.check("a").is_ok());
    }
}
