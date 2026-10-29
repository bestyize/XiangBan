//! 需要长期保存的状态：这台设备的编号。没有任何密钥，丢了只是重新登记成一台新设备。
//! 宿主决定存哪里：[`FileStore`] 是一个 JSON 文件；测试用 [`MemoryStore`]。
use crate::Error;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Mutex};

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct DeviceIdentity {
    /// 中转分配的设备编号
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    /// 首次运行时随机生成，登记时带上，让重复登记拿到同一个设备编号。
    #[serde(default)]
    pub install_id: String,
    #[serde(default)]
    pub device: Option<DeviceIdentity>,
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
            // 旧版本（配对 + 密钥）留下的字段会被忽略。
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
        std::fs::rename(&tmp, &self.path).map_err(|e| Error::Store(e.to_string()))
    }
}
