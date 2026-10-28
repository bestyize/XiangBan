//! Loopback request validation and process-local, capability-specific consent.
use crate::{Capability, ClientInfo, Consent};
use axum::http::{header, HeaderMap};
use sha2::{Digest, Sha256};
use std::{collections::HashSet, sync::Arc, time::Duration};
use subtle::ConstantTimeEq;
use tokio::sync::Mutex;

/// True only for the three allowed literal HTTP authorities, with an optional port.
pub fn valid_host(host: &str) -> bool {
    let (name, port) = if let Some(rest) = host.strip_prefix("[::1]") {
        ("[::1]", rest)
    } else {
        match host.find(':') {
            Some(at) => (&host[..at], &host[at..]),
            None => (host, ""),
        }
    };
    matches!(
        name.to_ascii_lowercase().as_str(),
        "localhost" | "127.0.0.1" | "[::1]"
    ) && (port.is_empty()
        || port.strip_prefix(':').is_some_and(|p| {
            !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()) && p.parse::<u16>().is_ok()
        }))
}

pub(crate) fn token_digest(token: &str) -> [u8; 32] {
    Sha256::digest(token.as_bytes()).into()
}
pub(crate) fn authenticated(headers: &HeaderMap, expected: &[u8; 32]) -> bool {
    if headers.get_all(header::AUTHORIZATION).iter().count() != 1 {
        return false;
    }
    let Some(value) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    else {
        return false;
    };
    let Some((scheme, token)) = value.split_once(' ') else {
        return false;
    };
    let actual = token_digest(token);
    let matches = bool::from(actual.ct_eq(expected));
    scheme.eq_ignore_ascii_case("Bearer") && matches
}
pub(crate) fn client(headers: &HeaderMap) -> ClientInfo {
    let value = |key| {
        headers
            .get(key)
            .and_then(|v| v.to_str().ok())
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
    };
    let user_agent = value("user-agent");
    ClientInfo {
        name: value("x-client-name")
            .or_else(|| user_agent.clone())
            .unwrap_or_else(|| "unknown".into()),
        user_agent,
    }
}

pub(crate) struct ConsentCache {
    backend: Arc<dyn Consent>,
    // Serialize prompts so simultaneous first requests cannot open duplicate dialogs.
    allowed: Mutex<HashSet<(ClientInfo, Capability)>>,
}
impl ConsentCache {
    pub(crate) fn new(backend: Arc<dyn Consent>) -> Self {
        Self {
            backend,
            allowed: Mutex::new(HashSet::new()),
        }
    }
    pub(crate) async fn allow(&self, client: &ClientInfo, capability: Capability) -> bool {
        // Include waiting for another prompt in the same thirty-second deadline.
        tokio::time::timeout(Duration::from_secs(30), async {
            let mut allowed = self.allowed.lock().await;
            let key = (client.clone(), capability);
            if allowed.contains(&key) {
                return true;
            }
            if self.backend.allow(client, capability).await {
                allowed.insert(key);
                true
            } else {
                false
            }
        })
        .await
        .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Unanswered;
    #[async_trait::async_trait]
    impl Consent for Unanswered {
        async fn allow(&self, _: &ClientInfo, _: Capability) -> bool {
            std::future::pending().await
        }
    }
    #[tokio::test(start_paused = true)]
    async fn unanswered_consent_is_denied_after_thirty_seconds() {
        let cache = Arc::new(ConsentCache::new(Arc::new(Unanswered)));
        let client = ClientInfo {
            name: "test".into(),
            user_agent: None,
        };
        let task = tokio::spawn(async move { cache.allow(&client, Capability::Listen).await });
        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_secs(29)).await;
        assert!(!task.is_finished());
        tokio::time::advance(Duration::from_secs(1)).await;
        assert!(!task.await.unwrap());
    }
}
