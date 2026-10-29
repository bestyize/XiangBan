//! 在线连接与路由。全部在内存里：谁在线、谁能给谁发消息（同账号的手机与电脑）、离线时暂存的通知。
//! 这里只处理已经序列化好的 JSON 文本帧。
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
    time::{Duration, Instant},
};
use std::sync::Arc;
use tokio::sync::{mpsc, Notify};

#[derive(Clone)]
pub struct Conn {
    pub id: u64,
    pub tx: mpsc::Sender<String>,
    /// 通知这个连接的读循环退出（被顶替、设备被撤销时用）。
    pub kick: Arc<Notify>,
}

impl Conn {
    /// 先把最后一帧排进发送队列，再让连接退出；队列里的帧会在关闭前发完。
    pub fn close_with(self, last_frame: String) {
        let _ = self.tx.try_send(last_frame);
        self.kick.notify_one();
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Sent {
    Delivered,
    Offline,
    /// 对方的发送队列满了（读得太慢），视为不可达。
    Congested,
}

#[derive(Default)]
struct Inner {
    conns: HashMap<String, Conn>,
    peers: HashMap<String, HashSet<String>>,
    notify: HashMap<String, VecDeque<(Instant, String)>>,
}

pub struct Hub {
    inner: Mutex<Inner>,
    next_id: AtomicU64,
    notify_cap: usize,
    notify_ttl: Duration,
}

impl Hub {
    pub fn new(notify_cap: usize, notify_ttl: Duration) -> Self {
        Self { inner: Mutex::new(Inner::default()), next_id: AtomicU64::new(1), notify_cap, notify_ttl }
    }

    /// 登记连接。同一设备只保留最新的一个连接，旧的返回给调用方去关闭。
    pub fn register(&self, device: &str, tx: mpsc::Sender<String>, kick: Arc<Notify>, peers: HashSet<String>) -> (u64, Option<Conn>) {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let mut g = self.inner.lock().unwrap();
        let old = g.conns.insert(device.to_string(), Conn { id, tx, kick });
        g.peers.insert(device.to_string(), peers);
        (id, old)
    }

    /// 连接结束时注销；只有当前登记的就是这个连接才会移除（被新连接顶替的不算）。
    pub fn unregister(&self, device: &str, conn_id: u64) -> bool {
        let mut g = self.inner.lock().unwrap();
        if g.conns.get(device).is_some_and(|c| c.id == conn_id) {
            g.conns.remove(device);
            g.peers.remove(device);
            true
        } else {
            false
        }
    }

    pub fn online_count(&self) -> usize {
        self.inner.lock().unwrap().conns.len()
    }

    pub fn is_online(&self, device: &str) -> bool {
        self.inner.lock().unwrap().conns.contains_key(device)
    }

    pub fn is_paired(&self, from: &str, to: &str) -> bool {
        self.inner.lock().unwrap().peers.get(from).is_some_and(|p| p.contains(to))
    }

    pub fn peers_of(&self, device: &str) -> Vec<String> {
        self.inner.lock().unwrap().peers.get(device).map(|p| p.iter().cloned().collect()).unwrap_or_default()
    }

    /// 连接建立后补充可通信的对端（并入，不覆盖：查库期间新登记的设备可能已经被 `add_pair` 加进来）。
    pub fn extend_peers(&self, device: &str, peers: impl IntoIterator<Item = String>) {
        if let Some(p) = self.inner.lock().unwrap().peers.get_mut(device) {
            p.extend(peers);
        }
    }

    pub fn add_pair(&self, a: &str, b: &str) {
        let mut g = self.inner.lock().unwrap();
        if let Some(p) = g.peers.get_mut(a) { p.insert(b.to_string()); }
        if let Some(p) = g.peers.get_mut(b) { p.insert(a.to_string()); }
    }

    /// 解除关系：双方的路由关系删掉，并清掉各自给对方暂存的通知。
    pub fn remove_pair(&self, a: &str, b: &str) {
        let mut g = self.inner.lock().unwrap();
        if let Some(p) = g.peers.get_mut(a) { p.remove(b); }
        if let Some(p) = g.peers.get_mut(b) { p.remove(a); }
        g.notify.remove(a);
        g.notify.remove(b);
    }

    /// 撤销设备：断开连接（调用方先发完告知帧）并清掉它的暂存。
    pub fn drop_device(&self, device: &str) -> Option<Conn> {
        let mut g = self.inner.lock().unwrap();
        g.peers.remove(device);
        g.notify.remove(device);
        g.conns.remove(device)
    }

    pub fn send(&self, device: &str, frame: String) -> Sent {
        let tx = self.inner.lock().unwrap().conns.get(device).map(|c| c.tx.clone());
        match tx {
            None => Sent::Offline,
            Some(tx) => match tx.try_send(frame) {
                Ok(()) => Sent::Delivered,
                Err(_) => Sent::Congested,
            },
        }
    }

    /// 暂存给离线设备的通知帧：每个设备最多 `notify_cap` 条，挤掉最旧的；过期的不再补发。
    pub fn store_notify(&self, to: &str, frame: String) {
        self.store_notify_at(to, frame, Instant::now());
    }

    pub fn store_notify_at(&self, to: &str, frame: String, now: Instant) {
        let mut g = self.inner.lock().unwrap();
        let q = g.notify.entry(to.to_string()).or_default();
        q.push_back((now, frame));
        while q.len() > self.notify_cap {
            q.pop_front();
        }
    }

    pub fn take_notify(&self, to: &str) -> Vec<String> {
        self.take_notify_at(to, Instant::now())
    }

    pub fn take_notify_at(&self, to: &str, now: Instant) -> Vec<String> {
        let mut g = self.inner.lock().unwrap();
        let q = g.notify.remove(to).unwrap_or_default();
        q.into_iter().filter(|(t, _)| now.saturating_duration_since(*t) < self.notify_ttl).map(|(_, f)| f).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> (mpsc::Sender<String>, mpsc::Receiver<String>) { mpsc::channel(4) }

    #[test]
    fn newer_connection_replaces_older_and_old_cleanup_is_ignored() {
        let hub = Hub::new(3, Duration::from_secs(60));
        let (t1, _r1) = conn();
        let (t2, _r2) = conn();
        let (id1, old) = hub.register("d", t1, Arc::new(Notify::new()), HashSet::new());
        assert!(old.is_none());
        let (id2, old) = hub.register("d", t2, Arc::new(Notify::new()), HashSet::new());
        assert_eq!(old.unwrap().id, id1);
        assert!(!hub.unregister("d", id1), "被顶替的旧连接结束时不能把新连接注销");
        assert!(hub.is_online("d"));
        assert!(hub.unregister("d", id2));
        assert!(!hub.is_online("d"));
    }

    #[test]
    fn routing_needs_a_peer_relation_and_reports_offline_or_congested() {
        let hub = Hub::new(3, Duration::from_secs(60));
        let (t, mut r) = conn();
        hub.register("a", t, Arc::new(Notify::new()), ["b".to_string()].into());
        assert!(hub.is_paired("a", "b"));
        assert!(!hub.is_paired("a", "c"));
        assert_eq!(hub.send("b", "x".into()), Sent::Offline);
        assert_eq!(hub.send("a", "1".into()), Sent::Delivered);
        assert_eq!(r.try_recv().unwrap(), "1");
        for i in 0..4 { assert_eq!(hub.send("a", i.to_string()), Sent::Delivered); }
        assert_eq!(hub.send("a", "overflow".into()), Sent::Congested);
    }

    #[test]
    fn notify_queue_is_capped_expires_and_cleared_when_removed() {
        let hub = Hub::new(3, Duration::from_secs(60));
        let t0 = Instant::now();
        for i in 0..5 { hub.store_notify_at("p", format!("n{i}"), t0); }
        assert_eq!(hub.take_notify_at("p", t0), vec!["n2", "n3", "n4"]);
        assert!(hub.take_notify_at("p", t0).is_empty(), "取走后清空");
        hub.store_notify_at("p", "old".into(), t0);
        assert!(hub.take_notify_at("p", t0 + Duration::from_secs(61)).is_empty(), "过期不补发");
        hub.store_notify_at("p", "x".into(), t0);
        hub.remove_pair("d", "p");
        assert!(hub.take_notify_at("p", t0).is_empty());
    }
}
