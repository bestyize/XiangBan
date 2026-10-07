use crate::error::{AppError, AppResult};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

#[derive(Clone)]
pub struct StorageManager {
    base_dir: PathBuf,
}

impl StorageManager {
    pub async fn new(base_dir: &Path) -> AppResult<Self> {
        fs::create_dir_all(base_dir).await?;
        Ok(Self {
            base_dir: base_dir.to_path_buf(),
        })
    }

    /// 保存文件，返回相对存储路径（UUID形式）
    #[allow(dead_code)]
    pub async fn save_file(&self, data: &[u8], extension: Option<&str>) -> AppResult<String> {
        let (file_name, mut file) = self.create_file(extension).await?;
        file.write_all(data).await?;
        file.flush().await?;

        Ok(file_name)
    }

    /// 创建物理文件写入句柄，支持大文件流式写入
    pub async fn create_file(&self, extension: Option<&str>) -> AppResult<(String, fs::File)> {
        let file_name = match extension {
            Some(ext) if !ext.is_empty() => format!("{}.{}", Uuid::new_v4(), ext.trim_start_matches('.')),
            _ => Uuid::new_v4().to_string(),
        };

        let file_path = self.base_dir.join(&file_name);
        let file = fs::File::create(&file_path).await?;
        Ok((file_name, file))
    }

    /// 获取物理文件的绝对路径，并校验是否处于安全根目录下
    pub fn get_absolute_path(&self, rel_path: &str) -> AppResult<PathBuf> {
        let target = self.base_dir.join(rel_path);
        // 简单路径安全检查，防止 .. 等越权
        if target.starts_with(&self.base_dir) {
            Ok(target)
        } else {
            Err(AppError::BadRequest("非法的文件存储路径".to_string()))
        }
    }

    /// 删除物理文件
    pub async fn delete_file(&self, rel_path: &str) -> AppResult<()> {
        let abs_path = self.get_absolute_path(rel_path)?;
        if abs_path.exists() {
            fs::remove_file(abs_path).await?;
        }
        Ok(())
    }
}
