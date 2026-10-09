use serde::Deserialize;
use std::time::Duration;
use tracing::info;

#[derive(Debug, Default, Clone)]
pub struct CollectedVersion {
    pub version: Option<String>,
    pub mac_download_url: Option<String>,
    pub win_download_url: Option<String>,
    pub linux_download_url: Option<String>,
    pub file_size: Option<String>,
    pub release_notes: Option<String>,
}

#[derive(Deserialize)]
struct BrewCaskResponse {
    version: Option<String>,
    url: Option<String>,
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: Option<String>,
    name: Option<String>,
    body: Option<String>,
    assets: Option<Vec<GitHubAsset>>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

fn build_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .unwrap_or_default()
}

pub async fn collect_app_version(
    slug: &str,
    brew_cask: Option<&str>,
    github_url: Option<&str>,
) -> Option<CollectedVersion> {
    let client = build_client();
    let mut result = CollectedVersion::default();

    // 1. 特殊已知官方直接下载规则处理
    match slug {
        "claude-desktop" => {
            result.version = Some("2.31226.0".to_string());
            result.mac_download_url = Some("https://downloads.claude.ai/releases/darwin/universal/2.31226.0/Claude-eb794d1033f2a59c2cd89aa29ed1f8b5de12aa73.zip".to_string());
            result.win_download_url = Some("https://downloads.claude.ai/releases/win32/x64/2.31226.0/Claude-Setup-x64.exe".to_string());
            result.file_size = Some("118 MB".to_string());
            result.release_notes = Some("Anthropic 官方桌面版，支持 Artifacts 实时渲染与工程模式".to_string());
            return Some(result);
        }
        "codex-editor" => {
            result.version = Some("0.45.0".to_string());
            result.mac_download_url = Some("https://downloader.cursor.sh/mac/installer/universal".to_string());
            result.win_download_url = Some("https://downloader.cursor.sh/windows/installer/x64".to_string());
            result.linux_download_url = Some("https://downloader.cursor.sh/linux/appImage/x64".to_string());
            result.file_size = Some("135 MB".to_string());
            result.release_notes = Some("新一代 AI 编程编辑器，项目级全库代码索引与智能补全".to_string());
            return Some(result);
        }
        "ollama" => {
            result.version = Some("0.5.11".to_string());
            result.mac_download_url = Some("https://ollama.com/download/Ollama-darwin.zip".to_string());
            result.win_download_url = Some("https://ollama.com/download/OllamaSetup.exe".to_string());
            result.linux_download_url = Some("https://ollama.com/download/ollama-linux-amd64.tar.gz".to_string());
            result.file_size = Some("64 MB".to_string());
            result.release_notes = Some("支持本地大模型一键拉取与并发运行".to_string());
            return Some(result);
        }
        _ => {}
    }

    // 2. 尝试从 Homebrew Cask API 采集直接下载链接与版本
    if let Some(cask) = brew_cask {
        let clean_cask = cask.trim();
        if !clean_cask.is_empty() {
            let url = format!("https://formulae.brew.sh/api/cask/{clean_cask}.json");
            match client.get(&url).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    match resp.json::<BrewCaskResponse>().await {
                        Ok(cask_data) => {
                            if let Some(ver) = cask_data.version {
                                let clean_ver = ver.split(',').next().unwrap_or(&ver).trim().to_string();
                                result.version = Some(clean_ver);
                            }
                            if let Some(dl_url) = cask_data.url {
                                result.mac_download_url = Some(dl_url);
                            }
                            info!("Successfully collected version from Homebrew Cask for {}: {:?}", slug, result.version);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse JSON from {url} (status {status}): {e}");
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to request Homebrew Cask at {url}: {e}");
                }
            }
        }
    }

    // 3. 尝试从 GitHub Releases 提取多平台安装包 (.dmg, .exe, .AppImage)
    if let Some(gh) = github_url {
        if gh.contains("github.com/") {
            let parts: Vec<&str> = gh.trim_end_matches('/').split("github.com/").collect();
            if parts.len() == 2 {
                let repo_path = parts[1];
                let api_url = format!("https://api.github.com/repos/{repo_path}/releases/latest");
                if let Ok(resp) = client.get(&api_url).send().await {
                    if let Ok(release) = resp.json::<GitHubRelease>().await {
                        if result.version.is_none() {
                            let tag = release.tag_name.or(release.name).unwrap_or_default();
                            let clean_tag = tag.trim_start_matches('v').trim().to_string();
                            if !clean_tag.is_empty() {
                                result.version = Some(clean_tag);
                            }
                        }

                        if let Some(body) = release.body {
                            let short_body: String = body.chars().take(200).collect();
                            result.release_notes = Some(short_body);
                        }

                        if let Some(assets) = release.assets {
                            for asset in assets {
                                let name_lower = asset.name.to_lowercase();
                                if (name_lower.ends_with(".dmg") || name_lower.ends_with(".app.zip")) && result.mac_download_url.is_none() {
                                    result.mac_download_url = Some(asset.browser_download_url.clone());
                                    let mb = asset.size / 1024 / 1024;
                                    result.file_size = Some(format!("{mb} MB"));
                                } else if (name_lower.ends_with(".exe") || name_lower.ends_with(".msi")) && result.win_download_url.is_none() {
                                    result.win_download_url = Some(asset.browser_download_url.clone());
                                } else if (name_lower.ends_with(".appimage") || name_lower.ends_with(".deb")) && result.linux_download_url.is_none() {
                                    result.linux_download_url = Some(asset.browser_download_url.clone());
                                }
                            }
                        }
                        info!("Successfully collected release assets from GitHub for {}: {:?}", slug, result.version);
                    }
                }
            }
        }
    }

    if result.version.is_some() || result.mac_download_url.is_some() {
        Some(result)
    } else {
        None
    }
}
