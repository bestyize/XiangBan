use std::path::{Path, PathBuf};

/// 安装包按内容寻址：`packages/<前两位>/<sha256>.zip`。
pub fn package_path(data_dir: &Path, sha256: &str) -> PathBuf {
    data_dir.join("packages").join(&sha256[..2]).join(format!("{sha256}.zip"))
}

pub async fn store_package(data_dir: &Path, sha256: &str, bytes: &[u8]) -> std::io::Result<()> {
    let path = package_path(data_dir, sha256);
    if tokio::fs::try_exists(&path).await? {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let tmp = path.with_extension(format!("tmp-{}", uuid::Uuid::new_v4().simple()));
    tokio::fs::write(&tmp, bytes).await?;
    tokio::fs::rename(&tmp, &path).await
}

/// 头像等小资源也按内容寻址：`assets/<前两位>/<sha256>`，同样的图只存一份。
pub fn asset_path(data_dir: &Path, sha256: &str) -> PathBuf {
    data_dir.join("assets").join(&sha256[..2]).join(sha256)
}

pub async fn store_asset(data_dir: &Path, sha256: &str, bytes: &[u8]) -> std::io::Result<()> {
    let path = asset_path(data_dir, sha256);
    if tokio::fs::try_exists(&path).await? {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let tmp = path.with_extension(format!("tmp-{}", uuid::Uuid::new_v4().simple()));
    tokio::fs::write(&tmp, bytes).await?;
    tokio::fs::rename(&tmp, &path).await
}
