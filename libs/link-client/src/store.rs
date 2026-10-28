//! 需要长期保存的状态：设备身份、配对密钥、重放防护进度、启动计数、正在进行的配对。
//! 宿主决定存哪里：[`FileStore`] 是一个 JSON 文件（Unix 上权限 0600）；
//! 桌面正式版应实现 [`LinkStore`]，把私钥和配对密钥放进系统钥匙串，其余放文件。
use crate::Error;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Mutex};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DeviceIdentity {
    pub id: String,
    pub name: String,
    /// 32 字节私钥，base64url
    pub private_key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PeerRecord {
    pub pairing_id: String,
    pub peer_id: String,
    pub peer_name: String,
    /// 对端公钥，base64url
    pub peer_public: String,
    /// 配对密钥 PRK，base64url
    pub prk: String,
    /// 重放防护进度：(启动计数, ep, 最大序号)
    pub replay: Option<(u64, String, u64)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PendingPair {
    pub code: String,
    /// 二维码里的一次性密钥，base64url
    pub secret: String,
    /// 过期时间（Unix 毫秒）
    pub expires_at_ms: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub device: Option<DeviceIdentity>,
    /// 每次启动加一后落盘，用于生成 `ep`（见协议 §2.3）。
    pub start_counter: u64,
    pub peers: HashMap<String, PeerRecord>,
    pub pending_pair: Option<PendingPair>,
}

pub trait LinkStore: Send + Sync {
    fn load(&self) -> Result<State, Error>;
    fn save(&self, state: &State) -> Result<(), Error>;
}

#[derive(Default)]
pub struct MemoryStore(Mutex<State>);

impl MemoryStore {
    pub fn new() -> Self { Self::default() }
}

impl LinkStore for MemoryStore {
    fn load(&self) -> Result<State, Error> { Ok(self.0.lock().unwrap().clone()) }
    fn save(&self, state: &State) -> Result<(), Error> {
        *self.0.lock().unwrap() = state.clone();
        Ok(())
    }
}

pub struct FileStore {
    path: PathBuf,
}

impl FileStore {
    pub fn new(path: impl Into<PathBuf>) -> Self { Self { path: path.into() } }
}

impl LinkStore for FileStore {
    fn load(&self) -> Result<State, Error> {
        match std::fs::read(&self.path) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| Error::Store(format!("状态文件损坏：{e}"))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(State::default()),
            Err(e) => Err(Error::Store(e.to_string())),
        }
    }

    /// 先写临时文件再改名，避免写到一半断电留下半个文件。
    fn save(&self, state: &State) -> Result<(), Error> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| Error::Store(e.to_string()))?;
        }
        let tmp = self.path.with_extension("tmp");
        let bytes = serde_json::to_vec_pretty(state).map_err(|e| Error::Store(e.to_string()))?;
        std::fs::write(&tmp, bytes).map_err(|e| Error::Store(e.to_string()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600));
        }
        std::fs::rename(&tmp, &self.path).map_err(|e| Error::Store(e.to_string()))
    }
}
