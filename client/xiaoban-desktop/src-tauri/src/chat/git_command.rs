//! Drain both git pipes while it runs: waiting for exit first can deadlock on a full pipe.
use std::{io::Read, process::{Command, Output, Stdio}, sync::mpsc, time::{Duration, Instant}};

pub(super) fn output(command: &mut Command, timeout: Duration) -> Result<Output, String> {
    let mut child = command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .spawn().map_err(|e| format!("无法启动 git：{e}"))?;
    let end = Instant::now() + timeout;
    let (tx, rx) = mpsc::channel();
    fn drain(pipe: impl Read + Send + 'static, tx: mpsc::Sender<(bool, std::io::Result<Vec<u8>>)>, stderr: bool) {
        std::thread::spawn(move || {
            let mut pipe = pipe;
            let mut bytes = Vec::new();
            let result = pipe.read_to_end(&mut bytes).map(|_| bytes);
            let _ = tx.send((stderr, result));
        });
    }
    drain(child.stdout.take().expect("piped stdout"), tx.clone(), false);
    drain(child.stderr.take().expect("piped stderr"), tx, true);
    let result = (|| {
        let status = loop {
            match child.try_wait().map_err(|e| e.to_string())? {
                Some(status) => break status,
                None if Instant::now() < end => std::thread::sleep(Duration::from_millis(20)),
                None => return Err("git 运行太久，已停止".into()),
            }
        };
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        for _ in 0..2 {
            let (is_stderr, bytes) = rx.recv_timeout(end.saturating_duration_since(Instant::now()))
                .map_err(|_| "读取 git 输出超时，请核对实际执行结果".to_string())?;
            let bytes = bytes.map_err(|e| e.to_string())?;
            if is_stderr { stderr = bytes; } else { stdout = bytes; }
        }
        Ok(Output { status, stdout, stderr })
    })();
    if result.is_err() { let _ = child.kill(); let _ = child.wait(); }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn output_fixture() {
        match std::env::var("XIAOBAN_GIT_OUTPUT_TEST").as_deref() {
            Ok("large") => {
                use std::io::Write;
                let data = vec![b'x'; 1024 * 1024];
                std::io::stdout().write_all(&data).unwrap();
                std::io::stderr().write_all(&data).unwrap();
            }
            Ok("wait") => std::thread::sleep(Duration::from_secs(5)),
            _ => {}
        }
    }
    fn fixture(mode: &str) -> Command {
        let mut cmd = Command::new(std::env::current_exe().unwrap());
        cmd.args(["--exact", &format!("{}::output_fixture", module_path!().split_once("::").unwrap().1), "--nocapture"])
            .env("XIAOBAN_GIT_OUTPUT_TEST", mode);
        cmd
    }
    #[test]
    fn drains_large_stdout_and_stderr_without_waiting_for_exit() {
        let out = output(&mut fixture("large"), Duration::from_secs(5)).unwrap();
        assert!(out.status.success());
        assert!(out.stdout.len() >= 1024 * 1024 && out.stderr.len() >= 1024 * 1024);
    }
    #[test]
    fn timeout_still_stops_a_process_with_open_pipes() {
        let started = Instant::now();
        assert!(output(&mut fixture("wait"), Duration::from_millis(100)).is_err());
        assert!(started.elapsed() < Duration::from_secs(2));
    }
}
