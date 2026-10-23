//! Trusted host HTTP policy and cancellation. Proxy settings never enter tool schemas.
use crate::HarnessError;
use std::{
    sync::{OnceLock, RwLock},
    time::{Duration, Instant},
};
#[derive(Clone, Default)]
pub struct NetworkPolicy {
    pub mode: String,
    pub proxy_url: String,
}
static POLICY: OnceLock<RwLock<NetworkPolicy>> = OnceLock::new();
fn policy() -> &'static RwLock<NetworkPolicy> {
    POLICY.get_or_init(|| RwLock::new(NetworkPolicy::default()))
}
pub fn set_policy(p: NetworkPolicy) -> Result<(), String> {
    if p.mode == "manual" {
        reqwest::Proxy::all(&p.proxy_url).map_err(|e| e.to_string())?;
    }
    *policy().write().unwrap_or_else(|e| e.into_inner()) = p;
    Ok(())
}
pub fn async_builder(b: reqwest::ClientBuilder) -> Result<reqwest::ClientBuilder, String> {
    let p = policy().read().unwrap_or_else(|e| e.into_inner());
    Ok(match p.mode.as_str() {
        "direct" => b.no_proxy(),
        "manual" => b
            .no_proxy()
            .proxy(reqwest::Proxy::all(&p.proxy_url).map_err(|e| e.to_string())?),
        _ => b,
    })
}
pub fn blocking_builder(
    b: reqwest::blocking::ClientBuilder,
) -> Result<reqwest::blocking::ClientBuilder, String> {
    let p = policy().read().unwrap_or_else(|e| e.into_inner());
    Ok(match p.mode.as_str() {
        "direct" => b.no_proxy(),
        "manual" => b
            .no_proxy()
            .proxy(reqwest::Proxy::all(&p.proxy_url).map_err(|e| e.to_string())?),
        _ => b,
    })
}
pub(crate) async fn wait<F: std::future::Future>(
    future: F,
    cancel: impl Fn() -> bool,
    timeout: Duration,
) -> Result<F::Output, HarnessError> {
    use std::{future::Future, task::Poll};
    let mut future = Box::pin(future);
    let mut tick = Box::pin(tokio::time::sleep(Duration::from_millis(20)));
    let end = Instant::now() + timeout;
    std::future::poll_fn(move |cx| {
        if cancel() {
            return Poll::Ready(Err(HarnessError::Cancelled));
        }
        if Instant::now() >= end {
            return Poll::Ready(Err(HarnessError::provider(
                "HTTP transport timed out",
                true,
            )));
        }
        if let Poll::Ready(v) = future.as_mut().poll(cx) {
            return Poll::Ready(Ok(v));
        }
        if tick.as_mut().poll(cx).is_ready() {
            tick.as_mut()
                .reset(tokio::time::Instant::now() + Duration::from_millis(20));
            cx.waker().wake_by_ref();
        }
        Poll::Pending
    })
    .await
}
