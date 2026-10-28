//! 桌面端手机互联的密钥存储：设备私钥与配对密钥 PRK 存进系统钥匙串（Windows 凭据管理器 / macOS Keychain），
//! 其余非敏感状态与重放进度存 JSON 文件（密钥位置只留占位符）。
//!
//! 读取钥匙串出错（被锁、用户拒绝授权等）与“确实没有这个条目”必须区分：前者向上报错、不改动任何本地状态，
//! 后者才会重置设备身份，否则一次临时失败就会让所有手机重新配对。
//! 非 Windows / macOS 平台只有进程内存实现，仅用于开发，重启后设备身份会丢失。

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard},
};
use xiaoban_link_client::{
    store::{LinkStore, State as StoredState},
    Error as LinkError,
};

pub const KEYCHAIN_SERVICE: &str = "xyz.xiaban.desktop-wallpaper.link";
const KEYCHAIN_PLACEHOLDER: &str = "@keychain";

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

#[cfg(windows)]
mod os_keystore {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;

    const CRED_TYPE_GENERIC: u32 = 1;
    const CRED_PERSIST_LOCAL_MACHINE: u32 = 2;

    #[repr(C)]
    struct FileTime {
        dw_low_date_time: u32,
        dw_high_date_time: u32,
    }

    #[repr(C)]
    struct CredentialW {
        flags: u32,
        cred_type: u32,
        target_name: *mut u16,
        comment: *mut u16,
        last_written: FileTime,
        credential_blob_size: u32,
        credential_blob: *mut u8,
        persist: u32,
        attribute_count: u32,
        attributes: *mut u8,
        target_alias: *mut u16,
        user_name: *mut u16,
    }

    #[link(name = "advapi32")]
    extern "system" {
        fn CredWriteW(credential: *const CredentialW, flags: u32) -> i32;
        fn CredReadW(target_name: *const u16, cred_type: u32, flags: u32, credential: *mut *mut CredentialW) -> i32;
        fn CredDeleteW(target_name: *const u16, cred_type: u32, flags: u32) -> i32;
        fn CredFree(buffer: *mut CredentialW);
    }

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    pub fn set(service: &str, account: &str, secret: &str) -> Result<(), String> {
        let target = format!("{service}/{account}");
        let mut target_w = wide(&target);
        let mut user_w = wide(account);
        let mut blob = secret.as_bytes().to_vec();
        let cred = CredentialW {
            flags: 0,
            cred_type: CRED_TYPE_GENERIC,
            target_name: target_w.as_mut_ptr(),
            comment: ptr::null_mut(),
            last_written: FileTime { dw_low_date_time: 0, dw_high_date_time: 0 },
            credential_blob_size: blob.len() as u32,
            credential_blob: blob.as_mut_ptr(),
            persist: CRED_PERSIST_LOCAL_MACHINE,
            attribute_count: 0,
            attributes: ptr::null_mut(),
            target_alias: ptr::null_mut(),
            user_name: user_w.as_mut_ptr(),
        };
        let ok = unsafe { CredWriteW(&cred, 0) };
        if ok != 0 {
            Ok(())
        } else {
            Err(format!("写入 Windows 凭据管理器失败：{}", std::io::Error::last_os_error()))
        }
    }

    pub fn get(service: &str, account: &str) -> Result<Option<String>, String> {
        let target = format!("{service}/{account}");
        let target_w = wide(&target);
        let mut ptr_cred: *mut CredentialW = ptr::null_mut();
        let ok = unsafe { CredReadW(target_w.as_ptr(), CRED_TYPE_GENERIC, 0, &mut ptr_cred) };
        if ok == 0 {
            let err = std::io::Error::last_os_error();
            // ERROR_NOT_FOUND = 1168
            if err.raw_os_error() == Some(1168) {
                return Ok(None);
            }
            return Err(format!("读取 Windows 凭据管理器失败：{err}"));
        }
        let result = unsafe {
            let cred = &*ptr_cred;
            let slice = if cred.credential_blob.is_null() || cred.credential_blob_size == 0 {
                &[][..]
            } else {
                std::slice::from_raw_parts(cred.credential_blob, cred.credential_blob_size as usize)
            };
            let text = String::from_utf8(slice.to_vec()).map_err(|e| e.to_string());
            CredFree(ptr_cred);
            text
        }?;
        Ok(Some(result))
    }

    pub fn delete(service: &str, account: &str) {
        let target = format!("{service}/{account}");
        let target_w = wide(&target);
        unsafe {
            let _ = CredDeleteW(target_w.as_ptr(), CRED_TYPE_GENERIC, 0);
        }
    }
}

#[cfg(target_os = "macos")]
mod os_keystore {
    use std::io::Write;
    use std::process::{Command, Stdio};

    /// `security` 的退出码：条目不存在。
    const ERR_ITEM_NOT_FOUND: i32 = 44;

    /// 密钥通过 `security -i` 的标准输入传入，不出现在进程参数里（`ps` 看不到）。
    pub fn set(service: &str, account: &str, secret: &str) -> Result<(), String> {
        let line = super::macos_add_command(service, account, secret)?;
        let mut child = Command::new("/usr/bin/security")
            .arg("-i")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("调用 macOS 钥匙串失败：{e}"))?;
        {
            let mut stdin = child.stdin.take().ok_or("无法写入 macOS 钥匙串")?;
            stdin.write_all(line.as_bytes()).map_err(|e| format!("写入 macOS 钥匙串失败：{e}"))?;
            stdin.write_all(b"\nexit\n").map_err(|e| format!("写入 macOS 钥匙串失败：{e}"))?;
        }
        let out = child.wait_with_output().map_err(|e| format!("调用 macOS 钥匙串失败：{e}"))?;
        if !out.status.success() {
            return Err(format!("写入 macOS 钥匙串失败：{}", String::from_utf8_lossy(&out.stderr).trim()));
        }
        // 交互模式下单条命令失败不一定体现在退出码上，写完回读一次确认确实存进去了。
        match get(service, account)? {
            Some(saved) if saved == secret => Ok(()),
            _ => Err("写入 macOS 钥匙串后回读不一致".into()),
        }
    }

    pub fn get(service: &str, account: &str) -> Result<Option<String>, String> {
        let out = Command::new("/usr/bin/security")
            .args(["find-generic-password", "-s", service, "-a", account, "-w"])
            .output()
            .map_err(|e| format!("调用 macOS 钥匙串失败：{e}"))?;
        if out.status.success() {
            return Ok(Some(String::from_utf8_lossy(&out.stdout).trim().to_string()));
        }
        if out.status.code() == Some(ERR_ITEM_NOT_FOUND) {
            return Ok(None);
        }
        // 钥匙串被锁、用户拒绝授权等：不能当作“没有”。
        Err(format!("读取 macOS 钥匙串失败：{}", String::from_utf8_lossy(&out.stderr).trim()))
    }

    pub fn delete(service: &str, account: &str) {
        let _ = Command::new("/usr/bin/security")
            .args(["delete-generic-password", "-s", service, "-a", account])
            .output();
    }
}

/// 生成 `security -i` 的一行 `add-generic-password` 命令。密钥只允许 base64 / base64url 字符，避免任何引号或空白注入。
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub(crate) fn macos_add_command(service: &str, account: &str, secret: &str) -> Result<String, String> {
    let plain = |s: &str, extra: &[char]| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || extra.contains(&c));
    if !plain(service, &['.', '-', '_']) || !plain(account, &['.', '-', '_', ':']) {
        return Err("钥匙串条目名含有不允许的字符".into());
    }
    if !plain(secret, &['-', '_', '=', '+', '/', '.']) {
        return Err("密钥含有不允许的字符".into());
    }
    Ok(format!("add-generic-password -U -s \"{service}\" -a \"{account}\" -w \"{secret}\""))
}

#[cfg(not(any(windows, target_os = "macos")))]
mod os_keystore {
    use std::{collections::HashMap, sync::{Mutex, OnceLock}};
    fn mem() -> &'static Mutex<HashMap<String, String>> {
        static MAP: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
        MAP.get_or_init(Default::default)
    }
    pub fn set(service: &str, account: &str, secret: &str) -> Result<(), String> {
        mem().lock().unwrap().insert(format!("{service}/{account}"), secret.to_string());
        Ok(())
    }
    pub fn get(service: &str, account: &str) -> Result<Option<String>, String> {
        Ok(mem().lock().unwrap().get(&format!("{service}/{account}")).cloned())
    }
    pub fn delete(service: &str, account: &str) {
        mem().lock().unwrap().remove(&format!("{service}/{account}"));
    }
}

/// 钥匙串后端：生产用系统实现，测试里换成可注入失败的假实现。
pub trait SecretBackend: Send + Sync {
    fn set(&self, service: &str, account: &str, secret: &str) -> Result<(), String>;
    /// `Ok(None)` 只表示“确实没有这个条目”；其他失败一律 `Err`。
    fn get(&self, service: &str, account: &str) -> Result<Option<String>, String>;
    fn delete(&self, service: &str, account: &str);
}

pub struct OsBackend;

impl SecretBackend for OsBackend {
    fn set(&self, service: &str, account: &str, secret: &str) -> Result<(), String> {
        os_keystore::set(service, account, secret)
    }
    fn get(&self, service: &str, account: &str) -> Result<Option<String>, String> {
        os_keystore::get(service, account)
    }
    fn delete(&self, service: &str, account: &str) {
        os_keystore::delete(service, account)
    }
}

/// 把设备私钥与配对密钥 PRK 存进系统钥匙串，其余元数据与重放计数存入 JSON 文件。
pub struct KeychainLinkStore {
    path: PathBuf,
    service: String,
    backend: Arc<dyn SecretBackend>,
    cache: Mutex<HashMap<String, String>>,
}

impl KeychainLinkStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self::with_backend(path, KEYCHAIN_SERVICE, Arc::new(OsBackend))
    }

    pub fn with_backend(path: impl Into<PathBuf>, service: impl Into<String>, backend: Arc<dyn SecretBackend>) -> Self {
        Self {
            path: path.into(),
            service: service.into(),
            backend,
            cache: Mutex::new(HashMap::new()),
        }
    }

    fn put_secret(&self, account: &str, secret: &str) -> Result<(), LinkError> {
        if secret.is_empty() || secret == KEYCHAIN_PLACEHOLDER {
            return Ok(());
        }
        {
            let cache = lock(&self.cache);
            if cache.get(account).map(String::as_str) == Some(secret) {
                return Ok(());
            }
        }
        self.backend.set(&self.service, account, secret).map_err(LinkError::Store)?;
        lock(&self.cache).insert(account.to_string(), secret.to_string());
        Ok(())
    }

    /// `Ok(None)`：钥匙串里确实没有；`Err`：读取失败，调用方不得据此改动本地状态。
    fn read_secret(&self, account: &str, fallback: &str) -> Result<Option<String>, LinkError> {
        if !fallback.is_empty() && fallback != KEYCHAIN_PLACEHOLDER {
            // 兼容旧版明文文件：第一次读到时迁入系统钥匙串。
            let _ = self.put_secret(account, fallback);
            return Ok(Some(fallback.to_string()));
        }
        if let Some(cached) = lock(&self.cache).get(account).cloned() {
            return Ok(Some(cached));
        }
        let loaded = self.backend.get(&self.service, account).map_err(LinkError::Store)?;
        if let Some(secret) = &loaded {
            lock(&self.cache).insert(account.to_string(), secret.clone());
        }
        Ok(loaded)
    }

    fn remove_secret(&self, account: &str) {
        lock(&self.cache).remove(account);
        self.backend.delete(&self.service, account);
    }
}

/// 只读状态文件里的非敏感部分（设备编号、手机列表），不碰钥匙串。密钥字段此时是占位符，不要拿来用。
pub fn read_public_state(path: &std::path::Path) -> StoredState {
    fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<StoredState>(&bytes).ok())
        .unwrap_or_default()
}

impl LinkStore for KeychainLinkStore {
    fn load(&self) -> Result<StoredState, LinkError> {
        let mut state: StoredState = match fs::read(&self.path) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| LinkError::Store(format!("状态文件损坏：{e}")))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(StoredState::default()),
            Err(e) => return Err(LinkError::Store(e.to_string())),
        };
        if let Some(device) = &mut state.device {
            let key = format!("device:{}", device.id);
            match self.read_secret(&key, &device.private_key)? {
                Some(secret) => device.private_key = secret,
                None => {
                    // 钥匙串里确实没有设备私钥（被用户清除）：重置本地设备身份以便重新登记。
                    state.device = None;
                    state.peers.clear();
                    state.pending_pair = None;
                    return Ok(state);
                }
            }
        }
        let mut missing = Vec::new();
        for (peer_id, rec) in &mut state.peers {
            let key = format!("peer:{peer_id}");
            match self.read_secret(&key, &rec.prk)? {
                Some(secret) => rec.prk = secret,
                None => missing.push(peer_id.clone()),
            }
        }
        for peer_id in missing {
            state.peers.remove(&peer_id);
        }
        Ok(state)
    }

    fn save(&self, state: &StoredState) -> Result<(), LinkError> {
        let mut scrubbed = state.clone();
        let mut active_accounts = HashSet::new();

        if let Some(device) = &mut scrubbed.device {
            let account = format!("device:{}", device.id);
            self.put_secret(&account, &device.private_key)?;
            active_accounts.insert(account);
            device.private_key = KEYCHAIN_PLACEHOLDER.to_string();
        }
        for (peer_id, rec) in &mut scrubbed.peers {
            let account = format!("peer:{peer_id}");
            self.put_secret(&account, &rec.prk)?;
            active_accounts.insert(account);
            rec.prk = KEYCHAIN_PLACEHOLDER.to_string();
        }

        // 清理已移除对端或已撤销设备的钥匙串条目
        let stale: Vec<String> = lock(&self.cache)
            .keys()
            .filter(|k| !active_accounts.contains(*k))
            .cloned()
            .collect();
        for account in stale {
            self.remove_secret(&account);
        }

        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir).map_err(|e| LinkError::Store(e.to_string()))?;
        }
        let tmp = self.path.with_extension("tmp");
        let bytes = serde_json::to_vec_pretty(&scrubbed).map_err(|e| LinkError::Store(e.to_string()))?;
        fs::write(&tmp, bytes).map_err(|e| LinkError::Store(e.to_string()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600));
        }
        fs::rename(&tmp, &self.path).map_err(|e| LinkError::Store(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use xiaoban_link_client::store::{DeviceIdentity, PeerRecord};

    #[derive(Default)]
    struct FakeBackend {
        map: Mutex<HashMap<String, String>>,
        fail_get: AtomicBool,
        fail_set: AtomicBool,
    }

    impl SecretBackend for FakeBackend {
        fn set(&self, service: &str, account: &str, secret: &str) -> Result<(), String> {
            if self.fail_set.load(Ordering::SeqCst) {
                return Err("锁定".into());
            }
            lock(&self.map).insert(format!("{service}/{account}"), secret.into());
            Ok(())
        }
        fn get(&self, service: &str, account: &str) -> Result<Option<String>, String> {
            if self.fail_get.load(Ordering::SeqCst) {
                return Err("钥匙串已锁定".into());
            }
            Ok(lock(&self.map).get(&format!("{service}/{account}")).cloned())
        }
        fn delete(&self, service: &str, account: &str) {
            lock(&self.map).remove(&format!("{service}/{account}"));
        }
    }

    fn sample_state() -> StoredState {
        let mut state = StoredState {
            device: Some(DeviceIdentity {
                id: "dsk_test1".into(),
                name: "测试电脑".into(),
                private_key: "secret_private_key_32bytes_base64url".into(),
            }),
            start_counter: 3,
            peers: HashMap::new(),
            pending_pair: None,
        };
        state.peers.insert(
            "phn_test1".into(),
            PeerRecord {
                pairing_id: "pair_1".into(),
                peer_id: "phn_test1".into(),
                peer_name: "测试手机".into(),
                peer_public: "pub_1".into(),
                prk: "secret_pairing_prk_32bytes_base64url".into(),
                replay: Some((1, "ep_abc".into(), 9)),
            },
        );
        state
    }

    fn temp_file(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("xiaoban-keychain-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir.join("phone-link-state.json")
    }

    #[test]
    fn keeps_private_key_and_prk_out_of_json_file() {
        let file = temp_file("plain");
        let backend = Arc::new(FakeBackend::default());
        let store = KeychainLinkStore::with_backend(&file, "svc", backend.clone());
        let state = sample_state();
        store.save(&state).unwrap();

        let on_disk = fs::read_to_string(&file).unwrap();
        assert!(!on_disk.contains("secret_private_key_32bytes_base64url"), "私钥不得以明文出现在 JSON 文件中");
        assert!(!on_disk.contains("secret_pairing_prk_32bytes_base64url"), "配对密钥 PRK 不得以明文出现在 JSON 文件中");
        assert!(on_disk.contains(KEYCHAIN_PLACEHOLDER));

        // 新实例（空缓存）从钥匙串恢复完整私钥与 PRK
        let reopened = KeychainLinkStore::with_backend(&file, "svc", backend.clone());
        assert_eq!(reopened.load().unwrap(), state);

        // 公开状态只含设备编号与手机列表
        let public = read_public_state(&file);
        assert_eq!(public.device.unwrap().id, "dsk_test1");
        assert!(public.peers.contains_key("phn_test1"));

        // 移除全部状态后清理钥匙串条目
        reopened.save(&StoredState::default()).unwrap();
        assert!(lock(&backend.map).is_empty());
        let _ = fs::remove_dir_all(file.parent().unwrap());
    }

    #[test]
    fn keychain_read_failure_is_an_error_and_never_wipes_pairings() {
        let file = temp_file("locked");
        let backend = Arc::new(FakeBackend::default());
        KeychainLinkStore::with_backend(&file, "svc", backend.clone()).save(&sample_state()).unwrap();
        let before = fs::read(&file).unwrap();

        backend.fail_get.store(true, Ordering::SeqCst);
        let locked = KeychainLinkStore::with_backend(&file, "svc", backend.clone());
        assert!(locked.load().is_err(), "钥匙串被锁时必须报错而不是当作缺失");
        assert_eq!(fs::read(&file).unwrap(), before, "失败时不得改动状态文件");

        // 钥匙串恢复后一切如常
        backend.fail_get.store(false, Ordering::SeqCst);
        let again = KeychainLinkStore::with_backend(&file, "svc", backend.clone());
        assert_eq!(again.load().unwrap(), sample_state());
        let _ = fs::remove_dir_all(file.parent().unwrap());
    }

    #[test]
    fn genuinely_missing_secrets_reset_only_what_is_missing() {
        let file = temp_file("missing");
        let backend = Arc::new(FakeBackend::default());
        KeychainLinkStore::with_backend(&file, "svc", backend.clone()).save(&sample_state()).unwrap();

        // 只丢了某个手机的配对密钥：该手机被移除，设备身份保留
        backend.delete("svc", "peer:phn_test1");
        let loaded = KeychainLinkStore::with_backend(&file, "svc", backend.clone()).load().unwrap();
        assert!(loaded.device.is_some() && loaded.peers.is_empty());

        // 设备私钥也没了：重置身份
        backend.delete("svc", "device:dsk_test1");
        let loaded = KeychainLinkStore::with_backend(&file, "svc", backend.clone()).load().unwrap();
        assert!(loaded.device.is_none() && loaded.peers.is_empty());
        let _ = fs::remove_dir_all(file.parent().unwrap());
    }

    #[test]
    fn failed_keychain_write_does_not_touch_the_state_file() {
        let file = temp_file("setfail");
        let backend = Arc::new(FakeBackend::default());
        backend.fail_set.store(true, Ordering::SeqCst);
        let store = KeychainLinkStore::with_backend(&file, "svc", backend);
        assert!(store.save(&sample_state()).is_err());
        assert!(!file.exists());
    }

    #[test]
    fn legacy_plaintext_file_is_migrated_into_the_keychain() {
        let file = temp_file("legacy");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, serde_json::to_vec(&sample_state()).unwrap()).unwrap();
        let backend = Arc::new(FakeBackend::default());
        let store = KeychainLinkStore::with_backend(&file, "svc", backend.clone());
        assert_eq!(store.load().unwrap(), sample_state());
        assert!(lock(&backend.map).contains_key("svc/device:dsk_test1"));
        let _ = fs::remove_dir_all(file.parent().unwrap());
    }

    /// 真实系统钥匙串（Windows 凭据管理器 / macOS Keychain）的往返。默认不跑，CI 的 Windows、macOS 机器上用
    /// `cargo test os_backend -- --ignored` 显式运行；Linux 上走进程内存实现，也应通过。
    #[test]
    #[ignore = "触碰真实系统钥匙串，只在平台测试工作流里显式运行"]
    fn os_backend_round_trips_with_the_real_keystore() {
        let service = format!("xyz.xiaban.test.link.{}", std::process::id());
        let backend = OsBackend;
        let account = "device:dsk_ci";
        assert_eq!(backend.get(&service, account).unwrap(), None, "新条目应为缺失而不是错误");
        backend.set(&service, account, "AbC-_=+/.9").unwrap();
        assert_eq!(backend.get(&service, account).unwrap().as_deref(), Some("AbC-_=+/.9"));
        backend.set(&service, account, "second_value-2").unwrap();
        assert_eq!(backend.get(&service, account).unwrap().as_deref(), Some("second_value-2"), "覆盖写入");
        backend.delete(&service, account);
        assert_eq!(backend.get(&service, account).unwrap(), None);
    }

    #[test]
    fn macos_command_keeps_the_secret_out_of_shell_syntax() {
        let line = macos_add_command("xyz.x.link", "device:dsk_1", "AbC-_=+/.9").unwrap();
        assert_eq!(line, "add-generic-password -U -s \"xyz.x.link\" -a \"device:dsk_1\" -w \"AbC-_=+/.9\"");
        for bad in ["a b", "a\"b", "a\nb", "a;b", "", "$(x)", "a\\b"] {
            assert!(macos_add_command("svc", "acct", bad).is_err(), "{bad:?}");
        }
        assert!(macos_add_command("sv c", "acct", "x").is_err());
        assert!(macos_add_command("svc", "ac\"ct", "x").is_err());
    }
}
