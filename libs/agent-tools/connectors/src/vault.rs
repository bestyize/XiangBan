//! Local AES-256-GCM vault. Values and cryptographic errors never enter diagnostics.
use base64::{engine::general_purpose::STANDARD, Engine};
use ring::{aead, hkdf};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock, Weak},
};

pub struct Vault {
    dir: PathBuf,
    master: [u8; 32],
    lock: Arc<Mutex<()>>,
}
#[derive(Serialize, Deserialize)]
struct Entry {
    nonce: String,
    data: String,
}
type Entries = BTreeMap<String, Entry>;
fn error() -> String {
    "凭据保险箱损坏、密钥不匹配或无法访问".into()
}
pub(crate) fn regular(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(m) if !m.is_file() || m.file_type().is_symlink() => Err(error()),
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(error()),
    }
}
pub(crate) fn atomic_write(path: &Path, data: &[u8]) -> Result<(), String> {
    regular(path)?;
    super::oauth::private_write(path, data).map_err(|_| error())
}
fn lock_for(dir: &Path) -> Arc<Mutex<()>> {
    static LOCKS: OnceLock<Mutex<HashMap<PathBuf, Weak<Mutex<()>>>>> = OnceLock::new();
    let mut locks = LOCKS
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if let Some(lock) = locks.get(dir).and_then(Weak::upgrade) {
        return lock;
    }
    let lock = Arc::new(Mutex::new(()));
    locks.insert(dir.into(), Arc::downgrade(&lock));
    lock
}
impl Vault {
    pub fn open(dir: impl AsRef<Path>) -> Result<Self, String> {
        let dir = dir.as_ref();
        fs::create_dir_all(dir).map_err(|_| error())?;
        if fs::symlink_metadata(dir)
            .map_err(|_| error())?
            .file_type()
            .is_symlink()
        {
            return Err(error());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(dir, fs::Permissions::from_mode(0o700)).map_err(|_| error())?;
        }
        let dir = dir.canonicalize().map_err(|_| error())?;
        let lock = lock_for(&dir);
        let guard = lock.lock().unwrap_or_else(|e| e.into_inner());
        let path = dir.join("master.key");
        regular(&path)?;
        if !path.exists() {
            // Losing the master must never silently create a new one over existing ciphertext.
            if fs::read_dir(&dir)
                .map_err(|_| error())?
                .filter_map(Result::ok)
                .any(|e| e.file_name().to_string_lossy().ends_with(".json"))
            {
                return Err(error());
            }
            let mut master = [0u8; 32];
            getrandom::fill(&mut master).map_err(|_| error())?;
            let bytes = protect(&master)?;
            let mut options = fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&path).map_err(|_| error())?;
            file.write_all(&bytes)
                .and_then(|_| file.sync_all())
                .map_err(|_| error())?;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).map_err(|_| error())?;
        }
        if fs::metadata(&path).map_err(|_| error())?.len() > 16 * 1024 {
            return Err(error());
        }
        let bytes = fs::read(path).map_err(|_| error())?;
        let master: [u8; 32] = unprotect(&bytes)?.try_into().map_err(|_| error())?;
        drop(guard);
        Ok(Self { dir, master, lock })
    }
    fn path(&self, connector: &str) -> PathBuf {
        self.dir
            .join(format!("{:x}.json", Sha256::digest(connector.as_bytes())))
    }
    fn read(&self, connector: &str) -> Result<Entries, String> {
        let path = self.path(connector);
        regular(&path)?;
        if !path.exists() {
            return Ok(BTreeMap::new());
        }
        let bytes = fs::read(path).map_err(|_| error())?;
        if bytes.len() > 4 * 1024 * 1024 {
            return Err(error());
        }
        serde_json::from_slice(&bytes).map_err(|_| error())
    }
    fn key(&self, connector: &str, field: &str) -> Result<aead::LessSafeKey, String> {
        // JSON tuple avoids ambiguous connector/field boundaries in HKDF info and AAD.
        let info = serde_json::to_vec(&(connector, field)).map_err(|_| error())?;
        let parts = [b"xiaoban/connector-vault/v1".as_slice(), info.as_slice()];
        let prk =
            hkdf::Salt::new(hkdf::HKDF_SHA256, b"xiaoban/connector-vault/v1").extract(&self.master);
        let mut key = [0u8; 32];
        prk.expand(&parts, &aead::AES_256_GCM)
            .map_err(|_| error())?
            .fill(&mut key)
            .map_err(|_| error())?;
        Ok(aead::LessSafeKey::new(
            aead::UnboundKey::new(&aead::AES_256_GCM, &key).map_err(|_| error())?,
        ))
    }
    pub fn set(&self, connector: &str, key: &str, value: &str) -> Result<(), String> {
        if connector.is_empty() || key.is_empty() || value.len() > 64 * 1024 {
            return Err("凭据标识或长度无效".into());
        }
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        let mut entries = self.read(connector)?;
        let mut nonce = [0u8; 12];
        getrandom::fill(&mut nonce).map_err(|_| error())?;
        let mut data = value.as_bytes().to_vec();
        let aad = serde_json::to_vec(&(connector, key)).map_err(|_| error())?;
        self.key(connector, key)?
            .seal_in_place_append_tag(
                aead::Nonce::assume_unique_for_key(nonce),
                aead::Aad::from(aad),
                &mut data,
            )
            .map_err(|_| error())?;
        entries.insert(
            key.into(),
            Entry {
                nonce: STANDARD.encode(nonce),
                data: STANDARD.encode(data),
            },
        );
        atomic_write(
            &self.path(connector),
            &serde_json::to_vec(&entries).map_err(|_| error())?,
        )
    }
    pub fn get(&self, connector: &str, key: &str) -> Result<Option<String>, String> {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        let entries = self.read(connector)?;
        let Some(entry) = entries.get(key) else {
            return Ok(None);
        };
        let nonce: [u8; 12] = STANDARD
            .decode(&entry.nonce)
            .map_err(|_| error())?
            .try_into()
            .map_err(|_| error())?;
        let mut data = STANDARD.decode(&entry.data).map_err(|_| error())?;
        let aad = serde_json::to_vec(&(connector, key)).map_err(|_| error())?;
        let plain = self
            .key(connector, key)?
            .open_in_place(
                aead::Nonce::assume_unique_for_key(nonce),
                aead::Aad::from(aad),
                &mut data,
            )
            .map_err(|_| error())?;
        Ok(Some(
            String::from_utf8(plain.to_vec()).map_err(|_| error())?,
        ))
    }
    pub fn remove_connector(&self, connector: &str) -> Result<(), String> {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        let path = self.path(connector);
        regular(&path)?;
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err(error()),
        }
    }
    pub fn has(&self, connector: &str) -> Result<Vec<String>, String> {
        // Authenticate too: corrupted ciphertext must not be presented as configured.
        let keys: Vec<_> = {
            let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
            self.read(connector)?.into_keys().collect()
        };
        for key in &keys {
            self.get(connector, key)?;
        }
        Ok(keys)
    }
}
impl Drop for Vault {
    fn drop(&mut self) {
        self.master.fill(0);
    }
}
#[cfg(not(windows))]
fn protect(bytes: &[u8]) -> Result<Vec<u8>, String> {
    Ok(bytes.to_vec())
}
#[cfg(not(windows))]
fn unprotect(bytes: &[u8]) -> Result<Vec<u8>, String> {
    Ok(bytes.to_vec())
}
#[cfg(windows)]
fn dpapi(bytes: &[u8], encrypt: bool) -> Result<Vec<u8>, String> {
    use windows_sys::Win32::{
        Foundation::LocalFree,
        Security::Cryptography::{
            CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
        },
    };
    let input = CRYPT_INTEGER_BLOB {
        cbData: bytes.len().try_into().map_err(|_| error())?,
        pbData: bytes.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };
    // DPAPI user scope (no LOCAL_MACHINE flag); buffers are owned by LocalAlloc.
    unsafe {
        let ok = if encrypt {
            CryptProtectData(
                &input,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        } else {
            CryptUnprotectData(
                &input,
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        };
        if ok == 0 {
            return Err(error());
        }
        let result = std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec();
        LocalFree(output.pbData as _);
        Ok(result)
    }
}
#[cfg(windows)]
fn protect(bytes: &[u8]) -> Result<Vec<u8>, String> {
    dpapi(bytes, true)
}
#[cfg(windows)]
fn unprotect(bytes: &[u8]) -> Result<Vec<u8>, String> {
    dpapi(bytes, false)
}
