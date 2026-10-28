//! Endpoint file location and metadata.
use serde::{Deserialize, Serialize};
use std::{env, fs, io, path::PathBuf};
/// Loopback endpoint. The token is a secret and must not be logged.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Endpoint {
    /// Bound TCP port.
    pub port: u16,
    /// Unpredictable URL path token.
    pub token: String,
    /// Server process ID.
    pub pid: u32,
}
/// Resolve the endpoint file, honoring the test override.
pub fn endpoint_path() -> io::Result<PathBuf> {
    if let Some(path) = env::var_os("XIAOBAN_HOOK_ENDPOINT") {
        return Ok(path.into());
    }
    #[cfg(target_os = "windows")]
    let base = env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let base = env::var_os("HOME").map(|p| PathBuf::from(p).join("Library/Application Support"));
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    let base = env::var_os("XDG_DATA_HOME")
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/share")));
    base.map(|p| p.join("xiaoban/hook-endpoint.json"))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "user data directory unavailable"))
}
/// Read endpoint metadata.
pub fn read() -> io::Result<Endpoint> {
    serde_json::from_slice(&fs::read(endpoint_path()?)?).map_err(io::Error::other)
}
/// Write a private endpoint file.
pub(crate) fn write(endpoint: &Endpoint) -> io::Result<()> {
    use std::io::Write;
    let path = endpoint_path()?;
    fs::create_dir_all(path.parent().ok_or_else(|| io::Error::other("no parent"))?)?;
    let mut file = tempfile::NamedTempFile::new_in(
        path.parent().ok_or_else(|| io::Error::other("no parent"))?,
    )?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.as_file()
            .set_permissions(fs::Permissions::from_mode(0o600))?;
    }
    #[cfg(windows)]
    {
        // Restrict the empty temporary file before putting the secret into it.
        // Use the SID instead of a localized account name; fail closed if ACLs
        // cannot be set on this filesystem.
        let output = std::process::Command::new("whoami.exe")
            .args(["/user", "/fo", "csv", "/nh"])
            .output()?;
        if !output.status.success() {
            return Err(io::Error::other("cannot identify current Windows user"));
        }
        let text = String::from_utf8_lossy(&output.stdout);
        let sid = text
            .split('"')
            .find(|s| {
                s.starts_with("S-1-")
                    && s.bytes()
                        .all(|b| b.is_ascii_digit() || b == b'-' || b == b'S')
            })
            .ok_or_else(|| io::Error::other("cannot identify current Windows user SID"))?;
        let grant = format!("*{sid}:(F)");
        let result = std::process::Command::new("icacls.exe")
            .arg(file.path())
            .args(["/grant:r", &grant, "/inheritance:r"])
            .output()?;
        if !result.status.success() {
            return Err(io::Error::other("cannot secure endpoint ACL"));
        }
    }
    file.write_all(&serde_json::to_vec(endpoint).map_err(io::Error::other)?)?;
    file.as_file().sync_all()?;
    file.persist(&path).map_err(|e| e.error)?;
    Ok(())
}
/// Remove only an endpoint that still belongs to this instance.
pub(crate) fn remove_if_owned(endpoint: &Endpoint) {
    if read().ok().as_ref() == Some(endpoint) {
        if let Ok(path) = endpoint_path() {
            let _ = fs::remove_file(path);
        }
    }
}
