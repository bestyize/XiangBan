//! Task-scoped, per-device non-PTY ADB shells. Failed/disconnected commands are NEVER replayed.
use crate::{adb::{ProcessTransport,Transport},package::Result};
use agent_kernel::ToolCtx;
use serde::Serialize;
use std::{collections::BTreeMap,io::{BufRead,BufReader,Read,Write},path::PathBuf,process::{Child,ChildStdin,Command,Stdio},
    sync::{Arc,Mutex,mpsc::{self,Receiver},atomic::{AtomicU64,Ordering}},thread,time::{Duration,Instant}};

const MAX_LINE:u64=1024*1024+1024;
#[derive(Debug,Default,Clone,Copy,Serialize)]
pub struct TransportStats {
    pub shell_starts:u64,
    pub shell_reuses:u64,
    pub shell_commands:u64,
    pub one_shot_starts:u64,
}
impl TransportStats {
    pub fn since(self,old:Self)->Self {Self {shell_starts:self.shell_starts.saturating_sub(old.shell_starts),
        shell_reuses:self.shell_reuses.saturating_sub(old.shell_reuses),shell_commands:self.shell_commands.saturating_sub(old.shell_commands),
        one_shot_starts:self.one_shot_starts.saturating_sub(old.one_shot_starts)}}
}
#[derive(Default)]
struct Counters {starts:AtomicU64,reuses:AtomicU64,commands:AtomicU64,one_shots:AtomicU64}
struct Session {child:Child,stdin:ChildStdin,output:Receiver<Result<Vec<u8>>>,nonce:String,sequence:u64}
impl Drop for Session {fn drop(&mut self) {let _=self.child.kill();let _=self.child.wait();}}
type Slot=Arc<Mutex<Option<Session>>>;

pub struct PersistentTransport {
    executable:PathBuf,
    prefix:Vec<String>,
    sessions:Mutex<BTreeMap<String,Slot>>,
    counters:Counters,
}
impl PersistentTransport {
    pub fn new(executable:impl Into<PathBuf>)->Self {Self {executable:executable.into(),prefix:Vec::new(),sessions:Mutex::new(BTreeMap::new()),counters:Counters::default()}}
    /// Host-controlled launch prefix, useful for a simulated ADB executable. Never exposed to pack files/tools.
    pub fn launch_prefix(mut self,prefix:Vec<String>)->Self {self.prefix=prefix;self}

    fn spawn(&self,serial:&str)->Result<Session> {
        let mut command=Command::new(&self.executable);
        command.args(&self.prefix).args(["-s",serial,"shell","-T"])
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        #[cfg(windows)] {use std::os::windows::process::CommandExt;command.creation_flags(0x08000000);}
        let mut random=[0u8;16];getrandom::fill(&mut random).map_err(|e|e.to_string())?;
        let nonce=random.iter().map(|b|format!("{b:02x}")).collect::<String>();
        let mut child=command.spawn().map_err(|e|format!("unable to open ADB shell: {e}; install Android Platform Tools or configure XIAOBAN_ADB"))?;
        let stdin=child.stdin.take().ok_or("ADB stdin unavailable")?;
        let stdout=child.stdout.take().ok_or("ADB stdout unavailable")?;
        let (sender,output)=mpsc::sync_channel(2);
        thread::spawn(move || {
            let mut reader=BufReader::new(stdout);
            loop {
                let mut line=Vec::new();
                match reader.by_ref().take(MAX_LINE+1).read_until(b'\n',&mut line) {
                    Ok(0)=>break,
                    Ok(_) if line.len() as u64>MAX_LINE=>{let _=sender.send(Err("ADB shell line exceeded budget".into()));break;},
                    Ok(_)=>{if sender.send(Ok(line)).is_err() {break;}},
                    Err(e)=>{let _=sender.send(Err(e.to_string()));break;},
                }
            }
        });
        if let Some(stderr)=child.stderr.take() {
            thread::spawn(move || {let _=std::io::copy(&mut stderr.take(32*1024),&mut std::io::sink());});
        }
        self.counters.starts.fetch_add(1,Ordering::Relaxed);
        Ok(Session {child,stdin,output,nonce,sequence:0})
    }

    fn shell(&self,serial:&str,command:&str,ctx:&ToolCtx,max:usize)->Result<Vec<u8>> {
        if !crate::adb::valid_serial(serial) {return Err("invalid explicit device serial".into());}
        let slot={
            let mut sessions=self.sessions.lock().map_err(|_|"ADB session registry poisoned")?;
            // Bound retained devices, including failed slots, for the lifetime of this task.
            if !sessions.contains_key(serial) && sessions.len()>=8 {return Err("too many device sessions in this task".into());}
            sessions.entry(serial.into()).or_default().clone()
        };
        let start=Instant::now();
        let mut guard=loop {
            if ctx.is_cancelled() || start.elapsed()>Duration::from_secs(45) {return Err("cancelled while waiting for device channel; nothing dispatched".into());}
            match slot.try_lock() {
                Ok(guard)=>break guard,
                Err(std::sync::TryLockError::WouldBlock)=>thread::sleep(Duration::from_millis(10)),
                Err(_)=>return Err("ADB session poisoned".into()),
            }
        };
        if guard.is_none() {*guard=Some(self.spawn(serial)?);}
        else {self.counters.reuses.fetch_add(1,Ordering::Relaxed);}
        if ctx.is_cancelled() {guard.take();return Err("cancelled before shell dispatch".into());}
        let session=guard.as_mut().ok_or("missing ADB session")?;
        session.sequence+=1;
        let marker=format!("xb-{}-{}",session.nonce,session.sequence);
        let begin=format!("{marker}:BEGIN");let end=format!("{marker}:END:");
        // The command was constructed by typed tools, not by an operation-pack shell field.
        let script=format!("printf '%s\\n' '{begin}'; ( {command} ) 2>&1; __xb_code=$?; printf '\\n%s:%s\\n' '{marker}:END' \"$__xb_code\"\n");
        self.counters.commands.fetch_add(1,Ordering::Relaxed);
        if session.stdin.write_all(script.as_bytes()).and_then(|_|session.stdin.flush()).is_err() {
            guard.take();return Err("ADB write failed; execution outcome unknown. Observe before retrying; command was not replayed.".into());
        }
        let mut active=false;let mut output=Vec::new();
        let result=loop {
            if ctx.is_cancelled() || start.elapsed()>Duration::from_secs(45) {
                break Err("ADB shell cancelled or timed out; execution outcome unknown. Observe before retrying; command was not replayed.".into());
            }
            let line=match session.output.recv_timeout(Duration::from_millis(20)) {
                Ok(Ok(line))=>line,
                Ok(Err(e))=>break Err(e),
                Err(mpsc::RecvTimeoutError::Timeout)=>continue,
                Err(_)=>break Err("ADB shell disconnected; execution outcome unknown. Observe before retrying; command was not replayed.".into()),
            };
            let clean=line.strip_suffix(b"\n").unwrap_or(&line);
            let clean=clean.strip_suffix(b"\r").unwrap_or(clean);
            if clean==begin.as_bytes() {active=true;continue;}
            if !active {continue;}
            if let Some(code)=clean.strip_prefix(end.as_bytes()) {
                if output.last()==Some(&b'\n') {output.pop();}
                let code=std::str::from_utf8(code).ok().and_then(|s|s.parse::<i32>().ok());
                break match code {
                    Some(0)=>Ok(output),
                    Some(code)=>Err(format!("ADB command exited {code}: {}",String::from_utf8_lossy(&output))),
                    None=>Err("invalid ADB response frame; outcome unknown".into()),
                };
            }
            if output.len()+line.len()>max.saturating_add(1) {break Err("ADB shell output exceeded budget; outcome unknown, command was not replayed".into());}
            output.extend(line);
        };
        // Close even on a normal nonzero exit, so no error framing can contaminate a later call.
        if result.is_err() {guard.take();}
        result
    }
}
impl Transport for PersistentTransport {
    fn run(&self,args:&[String],input:&[u8],ctx:&ToolCtx,max:usize)->Result<Vec<u8>> {
        if args.len()==4 && args[0]=="-s" && args[2]=="shell" && input.is_empty()
            && !args[3].contains(['\n','\r','\0']) {
            self.shell(&args[1],&args[3],ctx,max)
        } else {
            self.counters.one_shots.fetch_add(1,Ordering::Relaxed);
            let mut combined=self.prefix.clone();combined.extend_from_slice(args);
            ProcessTransport::new(&self.executable).run(&combined,input,ctx,max)
        }
    }
    fn bound_device_channel(&self)->bool {true}
    fn stats(&self)->TransportStats {TransportStats {shell_starts:self.counters.starts.load(Ordering::Relaxed),
        shell_reuses:self.counters.reuses.load(Ordering::Relaxed),shell_commands:self.counters.commands.load(Ordering::Relaxed),one_shot_starts:self.counters.one_shots.load(Ordering::Relaxed)}}
    fn close(&self)->Result<()> {
        let mut sessions=self.sessions.lock().map_err(|_|"ADB registry poisoned")?;
        for slot in sessions.values() {
            let mut session=slot.try_lock().map_err(|_|"ADB channel is still finishing cancellation")?;
            session.take();
        }
        sessions.clear();Ok(())
    }
}
