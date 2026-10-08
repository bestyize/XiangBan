use crate::{
    error::{ApiError, Result},
    protocol::{resource_url, Resource},
};
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};

pub fn public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let denied = [
                "0.0.0.0/8",
                "10.0.0.0/8",
                "100.64.0.0/10",
                "127.0.0.0/8",
                "169.254.0.0/16",
                "172.16.0.0/12",
                "192.0.0.0/24",
                "192.0.2.0/24",
                "192.168.0.0/16",
                "198.18.0.0/15",
                "198.51.100.0/24",
                "203.0.113.0/24",
                "224.0.0.0/4",
                "240.0.0.0/4",
            ];
            !denied
                .iter()
                .any(|net| net.parse::<ipnet::Ipv4Net>().unwrap().contains(&ip))
        }
        IpAddr::V6(ip) => {
            if let Some(v4) = ip.to_ipv4_mapped() {
                return public_ip(IpAddr::V4(v4));
            }
            // Global unicast only; deny documentation, transition and translation ranges.
            "2000::/3".parse::<ipnet::Ipv6Net>().unwrap().contains(&ip)
                && !["2001::/23", "2001:db8::/32", "2002::/16"]
                    .iter()
                    .any(|n| n.parse::<ipnet::Ipv6Net>().unwrap().contains(&ip))
        }
    }
}
pub async fn inspect(resource: &Resource, limit: u64, allow_loopback: bool) -> Result<Resource> {
    tokio::time::timeout(
        Duration::from_secs(120),
        inspect_inner(resource, limit, allow_loopback),
    )
    .await
    .map_err(|_| ApiError::bad("资源探测超时，未保存任何媒体文件"))?
}
async fn inspect_inner(resource: &Resource, limit: u64, allow_loopback: bool) -> Result<Resource> {
    let mut url = resource_url(&resource.url, allow_loopback)?;
    for redirects in 0..=4 {
        let host = url
            .host_str()
            .ok_or_else(|| ApiError::bad("地址缺少主机"))?
            .trim_matches(['[', ']'])
            .to_owned();
        let port = url
            .port_or_known_default()
            .ok_or_else(|| ApiError::bad("资源端口不支持"))?;
        let addresses: Vec<SocketAddr> = tokio::net::lookup_host((host.as_str(), port))
            .await
            .map_err(|_| ApiError::bad("资源域名无法解析"))?
            .collect();
        if addresses.is_empty()
            || addresses
                .iter()
                .any(|a| !(public_ip(a.ip()) || allow_loopback && a.ip().is_loopback()))
        {
            return Err(ApiError::bad(
                "资源探测仅访问公网地址，不访问内网或系统元数据地址",
            ));
        }
        // Pin validated DNS addresses on every redirect to prevent DNS rebinding.
        let client = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .resolve_to_addrs(&host, &addresses)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(120))
            .user_agent("XiaoBanWallpaperMetadata/0.1")
            .build()
            .map_err(ApiError::internal)?;
        let response = client
            .get(url.clone())
            .header("Accept-Encoding", "identity")
            .send()
            .await
            .map_err(|_| ApiError::bad("无法访问资源下载地址"))?;
        if response.status().is_redirection() {
            if redirects == 4 {
                return Err(ApiError::bad("资源重定向超过 4 次"));
            }
            let location = response
                .headers()
                .get("location")
                .and_then(|h| h.to_str().ok())
                .ok_or_else(|| ApiError::bad("重定向缺少地址"))?;
            let next = url
                .join(location)
                .map_err(|_| ApiError::bad("重定向地址不合法"))?;
            url = resource_url(next.as_str(), allow_loopback)?;
            continue;
        }
        if response.status().as_u16() != 200 {
            return Err(ApiError::bad(format!(
                "下载地址需要返回完整文件，当前 HTTP {}",
                response.status().as_u16()
            )));
        }
        let actual_mime = response
            .headers()
            .get("content-type")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("")
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_lowercase();
        if ["text/html", "application/xhtml+xml"].contains(&actual_mime.as_str()) {
            return Err(ApiError::bad(
                "下载地址返回了网页，请提供媒体文件的直接下载地址",
            ));
        }
        let encoding = response
            .headers()
            .get("content-encoding")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("identity");
        if encoding != "identity" {
            return Err(ApiError::bad(
                "资源需提供未压缩传输的原始字节，请关闭下载地址的 HTTP 内容压缩",
            ));
        }
        let mime = if resource.mime.starts_with("image/")
            && ["image/png", "image/jpeg", "image/webp", "image/avif"]
                .contains(&actual_mime.as_str())
        {
            actual_mime
        } else {
            resource.mime.clone()
        };
        if response.content_length().is_some_and(|n| n > limit) {
            return Err(ApiError::bad("资源超过允许大小"));
        }
        let mut stream = response.bytes_stream();
        let mut hash = Sha256::new();
        let mut bytes = 0u64;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|_| ApiError::bad("资源读取中断"))?;
            bytes = bytes
                .checked_add(chunk.len() as u64)
                .ok_or_else(|| ApiError::bad("资源过大"))?;
            if bytes > limit {
                return Err(ApiError::bad("资源超过允许大小"));
            }
            hash.update(&chunk); // Each chunk is dropped after hashing. No files or media BLOBs.
        }
        if bytes == 0 {
            return Err(ApiError::bad("资源为空文件"));
        }
        return Ok(Resource {
            url: resource.url.clone(),
            mime,
            bytes,
            sha256: hex::encode(hash.finalize()),
        });
    }
    Err(ApiError::bad("资源重定向异常"))
}
