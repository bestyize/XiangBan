use super::Runtime;
use crate::error::AppError;
use std::time::Duration;

const MAX_ATTEMPTS: usize = 3;

/// A provider's way out to the internet. It picks an API key and (optionally) a proxy for every
/// request, and when a key is rate limited or a proxy is unreachable it moves on to the next one.
#[derive(Clone)]
pub struct Upstream {
    runtime: Runtime,
    provider: &'static str,
}

impl Upstream {
    pub(super) fn new(runtime: Runtime, provider: &'static str) -> Self {
        Self { runtime, provider }
    }

    pub fn is_configured(&self) -> bool {
        self.runtime.has_keys(self.provider)
    }

    fn error(&self, message: impl Into<String>) -> AppError {
        AppError::Provider {
            provider: self.provider,
            message: message.into(),
        }
    }

    /// Sends a request built by `build`, which receives the HTTP client to use and the API key
    /// chosen for this attempt. The caller decides where the key goes (header or query string).
    pub async fn send<F>(&self, build: F) -> Result<reqwest::Response, AppError>
    where
        F: Fn(&reqwest::Client, &str) -> reqwest::RequestBuilder,
    {
        let mut last_error: Option<AppError> = None;

        for attempt in 1..=MAX_ATTEMPTS {
            let Some(key) = self.runtime.pick_key(self.provider) else {
                return Err(last_error.unwrap_or_else(|| {
                    if self.is_configured() {
                        self.error("All API keys are resting after rate limits or errors; try again shortly")
                    } else {
                        self.error("No API key configured. Add one in the admin console")
                    }
                }));
            };

            let proxy = self.runtime.pick_proxy(self.provider);
            let proxy_id = proxy.as_ref().map(|p| p.id);
            let client = match &proxy {
                Some(p) => p.client.clone(),
                None => self.runtime.direct_client(),
            };

            match build(&client, &key.value).send().await {
                Ok(response) => {
                    let status = response.status().as_u16();
                    self.runtime
                        .record_response(key.id, proxy_id, status, retry_after(&response));

                    let key_rejected = matches!(status, 401 | 403 | 429);
                    if key_rejected && attempt < MAX_ATTEMPTS && self.runtime.has_available_key(self.provider) {
                        continue;
                    }
                    return Ok(response);
                }
                Err(e) => {
                    let message = describe(e);
                    self.runtime.record_network_error(proxy_id, &message);
                    let error = self.error(format!("Request failed: {message}"));
                    // Another proxy may work; a direct connection has no alternative to try.
                    if proxy_id.is_some() && attempt < MAX_ATTEMPTS {
                        last_error = Some(error);
                        continue;
                    }
                    return Err(error);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| self.error("Request failed")))
    }

    /// Sends an unauthenticated or public request, routing through the provider's proxy if
    /// configured, or direct client otherwise. Retries across proxies on network error.
    pub async fn send_public<F>(&self, build: F) -> Result<reqwest::Response, AppError>
    where
        F: Fn(&reqwest::Client) -> reqwest::RequestBuilder,
    {
        let mut last_error: Option<AppError> = None;

        for attempt in 1..=MAX_ATTEMPTS {
            let proxy = self.runtime.pick_proxy(self.provider);
            let proxy_id = proxy.as_ref().map(|p| p.id);
            let client = match &proxy {
                Some(p) => p.client.clone(),
                None => self.runtime.direct_client(),
            };

            match build(&client).send().await {
                Ok(response) => {
                    let status = response.status().as_u16();
                    if let Some(id) = proxy_id {
                        self.runtime.record_proxy_response(id, status);
                    }
                    return Ok(response);
                }
                Err(e) => {
                    let message = describe(e);
                    self.runtime.record_network_error(proxy_id, &message);
                    let error = self.error(format!("Request failed: {message}"));
                    if proxy_id.is_some() && attempt < MAX_ATTEMPTS {
                        last_error = Some(error);
                        continue;
                    }
                    return Err(error);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| self.error("Request failed")))
    }
}

fn retry_after(response: &reqwest::Response) -> Option<Duration> {
    response
        .headers()
        .get(reqwest::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()
        .map(Duration::from_secs)
}

/// Error text with the request URL removed (Pixabay keys travel in the query string) and the
/// underlying cause appended, since "error sending request" alone says nothing useful.
pub(super) fn describe(error: reqwest::Error) -> String {
    let error = error.without_url();
    let mut message = error.to_string();
    let mut source = std::error::Error::source(&error);
    while let Some(cause) = source {
        message.push_str(": ");
        message.push_str(&cause.to_string());
        source = cause.source();
    }
    message
}
