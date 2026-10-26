//! Bounded, cancellable network work. Dropping the request stops it before publication.
use reqwest::{redirect::Policy, Client};
use std::{future::Future, net::IpAddr, time::{Duration, Instant}};

pub(crate) const SEARCH_BUDGET: Duration = Duration::from_secs(45);
pub(crate) const REVIEW_BUDGET: Duration = Duration::from_secs(180);
pub(crate) const INSTALL_BUDGET: Duration = Duration::from_secs(300);
const POLL: Duration = Duration::from_millis(25);

pub(crate) struct Download<'a> {
    cancelled: &'a (dyn Fn() -> bool + Sync),
    deadline: Instant,
}
impl<'a> Download<'a> {
    pub(crate) fn new(cancelled: &'a (dyn Fn() -> bool + Sync), budget: Duration) -> Self {
        Self { cancelled, deadline: Instant::now() + budget }
    }
    pub(crate) fn check(&self) -> Result<(), String> {
        if (self.cancelled)() { return Err("SkillHub operation cancelled; no further files will be published".into()); }
        if Instant::now() >= self.deadline { return Err("SkillHub operation timed out before publication; retry is safe after checking installed state".into()); }
        Ok(())
    }
    async fn wait<T>(&self, future: impl Future<Output = Result<T, String>>) -> Result<T, String> {
        let mut future = std::pin::pin!(future);
        loop {
            self.check()?;
            if let Ok(result) = tokio::time::timeout(POLL, future.as_mut()).await { return result; }
        }
    }
    pub(crate) fn bytes(&self, url: &str, max: u64) -> Result<Vec<u8>, String> {
        self.check()?;
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|e| e.to_string())?;
        let result = runtime.block_on(self.wait(async {
            let client = agent_kernel::net::async_builder(Client::builder()
                .timeout(Duration::from_secs(30)).connect_timeout(Duration::from_secs(8))
                .redirect(Policy::custom(|attempt| {
                    let u = attempt.url(); let host = u.host_str().unwrap_or("");
                    if attempt.previous().len() >= 4 || u.scheme() != "https" || host.is_empty()
                        || host.parse::<IpAddr>().is_ok() || host == "localhost"
                        || host.ends_with(".local") || host.ends_with(".internal") { attempt.stop() } else { attempt.follow() }
                })))?.build().map_err(|e| e.to_string())?;
            let mut response = client.get(url).send().await.map_err(|e| e.to_string())?;
            if !response.status().is_success() { return Err(format!("SkillHub HTTP {}", response.status())); }
            if response.content_length().is_some_and(|size| size > max) { return Err("SkillHub response exceeds size limit".into()); }
            let mut bytes = Vec::new();
            while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
                if chunk.len() as u64 > max.saturating_sub(bytes.len() as u64) { return Err("SkillHub response exceeds size limit".into()); }
                bytes.extend_from_slice(&chunk);
            }
            Ok(bytes)
        }));
        runtime.shutdown_timeout(Duration::from_millis(50));
        result
    }
    pub(crate) fn files(&self, plan: &crate::package::Plan, read: &(dyn Fn(&crate::package::Item, &Download<'_>) -> Result<Vec<u8>,String> + Sync)) -> Result<std::collections::BTreeMap<String,Vec<u8>>,String> {
        use std::sync::{Mutex,atomic::{AtomicBool,AtomicUsize,Ordering}};
        let next=AtomicUsize::new(0);let stopped=AtomicBool::new(false);
        let files=Mutex::new(std::collections::BTreeMap::new());let failure=Mutex::new(None);
        std::thread::scope(|scope| -> Result<(),String> {
            let mut workers=Vec::new();
            for _ in 0..4.min(plan.files.len()) {
                workers.push(scope.spawn(|| {
                    let cancelled=||(self.cancelled)() || stopped.load(Ordering::SeqCst);
                    let download=Download {cancelled:&cancelled,deadline:self.deadline};
                    loop {
                        let index=next.fetch_add(1,Ordering::SeqCst);
                        let Some(item)=plan.files.get(index) else {break};
                        let result=download.check().and_then(|_|read(item,&download)).and_then(|bytes|{
                            use sha2::{Digest,Sha256};
                            if bytes.len() as u64!=item.size || format!("{:x}",Sha256::digest(&bytes))!=item.sha256 {
                                Err(format!("SkillHub canonical file SHA256 or size mismatch: {}",item.path))
                            } else {Ok(bytes)}
                        });
                        match result {
                            Ok(bytes)=>{files.lock().unwrap_or_else(|e|e.into_inner()).insert(item.path.clone(),bytes);},
                            Err(error)=>{let mut first=failure.lock().unwrap_or_else(|e|e.into_inner());if first.is_none(){*first=Some(error);}stopped.store(true,Ordering::SeqCst);break;},
                        }
                    }
                }));
            }
            for worker in workers {if worker.join().is_err(){stopped.store(true,Ordering::SeqCst);return Err("SkillHub download worker failed; package was not published".into());}}
            Ok(())
        })?;
        if let Some(error)=failure.into_inner().unwrap_or_else(|e|e.into_inner()){return Err(error);}
        self.check()?;
        let files=files.into_inner().unwrap_or_else(|e|e.into_inner());
        crate::package::canonical_files(plan,|item|files.get(&item.path).cloned().ok_or_else(||"incomplete canonical download".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::{Arc, atomic::{AtomicBool, Ordering}}, thread};
    #[test]
    fn pending_network_work_acknowledges_cancellation_without_waiting_for_http_timeout() {
        let cancel = Arc::new(AtomicBool::new(false)); let signal = cancel.clone();
        let worker = thread::spawn(move || { thread::sleep(Duration::from_millis(60)); signal.store(true, Ordering::SeqCst); });
        let cancelled = || cancel.load(Ordering::SeqCst);
        let download = Download::new(&cancelled, INSTALL_BUDGET);
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let started = Instant::now();
        let result: Result<(), String> = runtime.block_on(download.wait(std::future::pending()));
        assert!(result.unwrap_err().contains("cancelled")); assert!(started.elapsed() < Duration::from_secs(1));
        worker.join().unwrap();
    }
    #[test]
    fn operation_budget_bounds_multiple_requests_and_is_distinct_from_http_timeout() {
        let download = Download::new(&|| false, Duration::ZERO);
        assert!(download.check().unwrap_err().contains("timed out"));
        assert!(INSTALL_BUDGET > Duration::from_secs(30));
    }
    #[test]
    fn canonical_fallback_has_bounded_concurrency_and_verifies_every_file() {
        use sha2::{Digest,Sha256};use std::sync::atomic::AtomicUsize;
        let values=(0..32).map(|i|serde_json::json!({"path":if i==0{"SKILL.md".to_string()}else{format!("references/{i}.md")},"size":4,"sha256":format!("{:x}",Sha256::digest(b"data"))})).collect::<Vec<_>>();
        let plan=crate::package::plan("1.0",&values).unwrap();
        let active=AtomicUsize::new(0);let peak=AtomicUsize::new(0);
        let files=Download::new(&||false,INSTALL_BUDGET).files(&plan,&|_,_|{
            let count=active.fetch_add(1,Ordering::SeqCst)+1;peak.fetch_max(count,Ordering::SeqCst);
            thread::sleep(Duration::from_millis(10));active.fetch_sub(1,Ordering::SeqCst);Ok(b"data".to_vec())
        }).unwrap();
        assert_eq!(files.len(),32);assert!(peak.load(Ordering::SeqCst)<=4);assert!(peak.load(Ordering::SeqCst)>1);
        assert!(Download::new(&||false,INSTALL_BUDGET).files(&plan,&|_,_|Ok(b"bad".to_vec())).is_err());
    }
    #[test]
    fn cancellation_during_http_headers_or_body_returns_a_known_error() {
        use std::{net::TcpListener,io::{Read,Write}};
        for send_headers in [false,true] {
            let server=TcpListener::bind("127.0.0.1:0").unwrap();let address=server.local_addr().unwrap();
            let cancel=Arc::new(AtomicBool::new(false));let signal=cancel.clone();
            let worker=thread::spawn(move||{
                let (mut stream,_)=server.accept().unwrap();let mut request=[0;2048];let _=stream.read(&mut request);
                if send_headers {let _=stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 10000\r\n\r\npartial");}
                signal.store(true,Ordering::SeqCst);thread::sleep(Duration::from_millis(200));
            });
            let cancelled=||cancel.load(Ordering::SeqCst);let download=Download::new(&cancelled,INSTALL_BUDGET);
            let result=download.bytes(&format!("http://{address}/slow"),20000);
            assert!(result.unwrap_err().contains("cancelled"));worker.join().unwrap();
        }
    }
}
