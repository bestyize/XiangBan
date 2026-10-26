//! Pending user interactions are host state, recoverable by a rebuilt window.
use agent_kernel::Approval;
use serde_json::{Value,json};
use std::{collections::HashMap,sync::{Mutex,MutexGuard,atomic::{AtomicU64,Ordering},mpsc::{self,Sender,Receiver}}};
type Waiting<T> = HashMap<String,(String,Sender<T>,Value)>;
#[derive(Default)]struct State { approvals:Waiting<Approval>,questions:Waiting<Option<String>>,revision:u64 }
#[derive(Default)]pub struct Broker {state:Mutex<State>,next:AtomicU64,root:Option<std::path::PathBuf>,recovered:Mutex<HashMap<String,Value>>}
fn lock<T>(m:&Mutex<T>)->MutexGuard<'_,T>{m.lock().unwrap_or_else(|e|e.into_inner())}
impl Broker {
 fn id(&self,p:char)->String{format!("{p}{}-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos(),self.next.fetch_add(1,Ordering::Relaxed)+1)}
 pub fn persistent(root:std::path::PathBuf)->Self {
    let broker=Self{root:Some(root.clone()),..Default::default()};
    if let Ok(entries)=std::fs::read_dir(root){for entry in entries.flatten(){
        let path=entry.path();if path.extension().and_then(|s|s.to_str())!=Some("json"){continue;}
        if entry.metadata().map_or(true,|m|m.len()>16*1024){continue;}
        if let Ok(raw)=std::fs::read(&path){if let Ok(mut value)=serde_json::from_slice::<Value>(&raw){
            if let Some(id)=value["id"].as_str().map(str::to_owned){
                if Self::safe(&id)&&value["session"].as_str().is_some_and(Self::safe)&&value["question"].is_string(){
                    value["recovered"]=json!(true);lock(&broker.recovered).insert(id,value);
                }
            }
        }}
    }}
    broker
 }
 fn safe(id:&str)->bool{!id.is_empty()&&id.len()<=128&&id.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-'||b==b'_')}
 fn clear_saved(&self,id:&str)->Result<(),String>{
    if !Self::safe(id){return Err("invalid question id".into());}
    if let Some(root)=&self.root{match std::fs::remove_file(root.join(format!("{id}.json"))){Ok(())=>{},Err(e)if e.kind()==std::io::ErrorKind::NotFound=>{},Err(e)=>return Err(e.to_string())}}
    Ok(())
 }
 pub fn recovered_question(&self,id:&str)->Option<Value>{lock(&self.recovered).get(id).cloned()}
 pub fn finish_recovered(&self,id:&str)->Result<(),String>{self.clear_saved(id)?;lock(&self.recovered).remove(id);lock(&self.state).revision+=1;Ok(())}
 pub fn open_connector_question(&self, session: &str, run: &str, connectors: Value) -> Result<(String, Receiver<Option<String>>), String> {
    self.open_question_checked(session, json!({"question":"连接服务后继续当前任务", "options":[],
        "kind":"connector_request", "runId":run, "connectors":connectors}))
 }
 pub fn open_question_checked(&self,s:&str,mut value:Value)->Result<(String,Receiver<Option<String>>),String>{
    if !Self::safe(s){return Err("invalid session id".into());}
    let id=self.id('q');value["id"]=json!(id);value["session"]=json!(s);
    if let Some(root)=&self.root{
        std::fs::create_dir_all(root).map_err(|e|e.to_string())?;
        let file=root.join(format!("{id}.json"));let temporary=file.with_extension("tmp");
        let raw=serde_json::to_vec(&value).map_err(|e|e.to_string())?;
        #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;use std::io::Write;let mut f=std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&temporary).map_err(|e|e.to_string())?;f.write_all(&raw).and_then(|_|f.sync_all()).map_err(|e|e.to_string())?;}
        #[cfg(not(unix))]std::fs::write(&temporary,raw).map_err(|e|e.to_string())?;
        std::fs::rename(temporary,file).map_err(|e|e.to_string())?;
    }
    let(tx,rx)=mpsc::channel();let mut state=lock(&self.state);state.questions.insert(id.clone(),(s.into(),tx,value));state.revision+=1;Ok((id,rx))
 }
 pub fn open_approval(&self,s:&str)->(String,Receiver<Approval>){self.open_approval_payload(s,json!({}))}
 pub fn open_approval_payload(&self,s:&str,mut value:Value)->(String,Receiver<Approval>){let id=self.id('a');let(tx,rx)=mpsc::channel();value["id"]=json!(id);value["session"]=json!(s);let mut state=lock(&self.state);state.approvals.insert(id.clone(),(s.into(),tx,value));state.revision+=1;(id,rx)}
 pub fn open_question(&self,s:&str)->(String,Receiver<Option<String>>){self.open_question_payload(s,json!({}))}
 pub fn open_question_payload(&self,s:&str,value:Value)->(String,Receiver<Option<String>>){self.open_question_checked(s,value).expect("valid in-memory question")}

 pub fn answer_approval(&self,id:&str,value:Approval)->Result<(),String>{let mut s=lock(&self.state);let item=s.approvals.remove(id);s.revision+=1;item.ok_or("这个确认已经结束")?.1.send(value).map_err(|_|"这个确认已经结束".into())}
 pub fn answer_question(&self,id:&str,value:Option<String>)->Result<(),String>{let mut s=lock(&self.state);if !s.questions.contains_key(id){return Err("这个问题已经结束".into());}self.clear_saved(id)?;let item=s.questions.remove(id);s.revision+=1;item.ok_or("这个问题已经结束")?.1.send(value).map_err(|_|"这个问题已经结束".into())}
 pub fn close_approval(&self,id:&str){let mut s=lock(&self.state);s.approvals.remove(id);s.revision+=1;}
 pub fn close_question(&self,id:&str){let _=self.clear_saved(id);let mut s=lock(&self.state);s.questions.remove(id);s.revision+=1;}
 pub fn drop_session(&self,id:&str){let mut s=lock(&self.state);s.approvals.retain(|_,(session,_,_)|session!=id);s.questions.retain(|key,(session,_,_)|{if session==id{let _=self.clear_saved(key);false}else{true}});s.revision+=1;}
 pub fn snapshot(&self,id:&str)->Value{let s=lock(&self.state);json!({"revision":s.revision,"approvals":s.approvals.values().filter(|(session,_,_)|session==id).map(|(_,_,v)|v).collect::<Vec<_>>(),"questions":s.questions.values().filter(|(session,_,_)|session==id).map(|(_,_,v)|v.clone()).chain(lock(&self.recovered).values().filter(|v|v["session"]==id).cloned()).collect::<Vec<_>>()})}
 #[cfg(test)]pub fn pending(&self)->usize{let s=lock(&self.state);s.approvals.len()+s.questions.len()}
}
/// A restart resumes with new user input, never by replaying the interrupted tool call.
pub fn recovery_text(saved: &Value, answer: &str, inspection: &agent_kernel::journal::RunInspection) -> Result<String, String> {
    let connector = saved["kind"] == "connector_request" || saved["connectors"].is_array();
    let tool = if connector { "connector_request" } else { "ask_user" };
    if inspection.incomplete_tools.iter().any(|t| t.dispatched && t.name != tool) {
        return Err("上次还有结果不明的操作，请先检查实际结果后再继续".into());
    }
    if inspection.terminal || !inspection.incomplete_tools.iter().any(|t| t.dispatched && t.name == tool) {
        return Err("这个问题已经结束，请跳过后在会话中继续".into());
    }
    if connector {
        if !saved["connectors"].as_array().is_some_and(|items| items.iter().any(|c| c["reference"] == answer)) {
            return Err("连接器选择无效".into());
        }
        Ok(format!("上次执行在等待连接服务时中断。我已处理连接卡片，选择的服务引用：{answer}。请先使用 connector_search 核对当前连接状态和工具，再继续原任务；不要重放已执行或结果不明的操作。"))
    } else {
        Ok(format!("上次执行在等待我回答时中断。问题：{}\n我的回答：{}\n请检查现场后继续；不要重放已执行或结果不明的操作。", saved["question"].as_str().unwrap_or(""), answer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn pending_snapshots_retain_payloads_and_version_consumption(){
        let b=Broker::default();let(id,_)=b.open_approval_payload("s1",json!({"tool":"shell","arguments":{"command":"pwd"}}));
        let before=b.snapshot("s1");assert_eq!(before["approvals"][0]["tool"],"shell");assert_eq!(before["approvals"][0]["id"],id);assert_eq!(b.snapshot("s2")["approvals"],json!([]));
        b.close_approval(&id);let after=b.snapshot("s1");assert!(after["approvals"].as_array().unwrap().is_empty());assert!(after["revision"].as_u64()>before["revision"].as_u64());
    }

    #[test]
    fn an_approval_reaches_the_waiting_run_once() {
        let broker = Broker::default();
        let (id, rx) = broker.open_approval("s1");
        broker.answer_approval(&id, Approval::Always).unwrap();
        assert_eq!(rx.recv_timeout(Duration::from_secs(1)).unwrap(), Approval::Always);
        assert!(broker.answer_approval(&id, Approval::Once).is_err(), "a second answer is refused");
        assert_eq!(broker.pending(), 0);
    }

    #[test]
    fn a_question_can_be_answered_or_dismissed() {
        let broker = Broker::default();
        let (a, rx_a) = broker.open_question("s1");
        let (b, rx_b) = broker.open_question("s1");
        broker.answer_question(&a, Some("蓝色".into())).unwrap();
        broker.answer_question(&b, None).unwrap();
        assert_eq!(rx_a.recv_timeout(Duration::from_secs(1)).unwrap(), Some("蓝色".to_string()));
        assert_eq!(rx_b.recv_timeout(Duration::from_secs(1)).unwrap(), None);
    }

    #[test]
    fn ids_are_unique_and_unknown_or_foreign_ids_are_refused() {
        let broker = Broker::default();
        let (approval, _keep) = broker.open_approval("s1");
        let (question, _keep2) = broker.open_question("s1");
        assert_ne!(approval, question);
        assert!(broker.answer_approval("a999", Approval::Once).is_err());
        assert!(broker.answer_question("q999", None).is_err());
        assert!(broker.answer_approval(&question, Approval::Once).is_err(), "a question id is not an approval id");
        assert!(broker.answer_question(&approval, None).is_err());
        assert_eq!(broker.pending(), 2, "wrong-kind answers do not consume anything");
    }

    #[test]
    fn dropping_a_conversation_ends_only_its_own_waits() {
        let broker = Broker::default();
        let (mine, rx_mine) = broker.open_approval("s1");
        let (other, rx_other) = broker.open_approval("s2");
        let (question, rx_q) = broker.open_question("s1");
        broker.drop_session("s1");
        assert!(rx_mine.recv_timeout(Duration::from_millis(50)).is_err(), "the sender is gone, so the run's wait ends");
        assert!(rx_q.recv_timeout(Duration::from_millis(50)).is_err());
        assert!(broker.answer_approval(&mine, Approval::Once).is_err());
        assert!(broker.answer_question(&question, None).is_err());
        broker.answer_approval(&other, Approval::Deny).unwrap();
        assert_eq!(rx_other.recv_timeout(Duration::from_secs(1)).unwrap(), Approval::Deny);
    }

    #[test]
    fn closing_removes_a_wait_that_nobody_will_answer() {
        let broker = Broker::default();
        let (id, _rx) = broker.open_approval("s1");
        broker.close_approval(&id);
        assert!(broker.answer_approval(&id, Approval::Once).is_err());
        let (q, _rx) = broker.open_question("s1");
        broker.close_question(&q);
        assert!(broker.answer_question(&q, None).is_err());
        assert_eq!(broker.pending(), 0);
    }

    #[test]
    fn answering_after_the_run_stopped_listening_is_reported() {
        let broker = Broker::default();
        let (id, rx) = broker.open_approval("s1");
        drop(rx);
        assert!(broker.answer_approval(&id, Approval::Once).is_err());
    }
}

#[cfg(test)]
mod durable_question_tests {
    use super::*;
    #[test]
    fn unanswered_questions_survive_restart_without_replaying_a_worker() {
        let root=std::env::temp_dir().join(format!("broker-persist-{}",std::process::id()));let _=std::fs::remove_dir_all(&root);
        let first=Broker::persistent(root.clone());
        let(id,_rx)=first.open_question_checked("s1",json!({"question":"选择目录？","options":["A","B"],"runId":"r1"})).unwrap();
        drop(first);
        let second=Broker::persistent(root.clone());
        assert_eq!(second.snapshot("s1")["questions"][0]["id"],id);
        assert_eq!(second.snapshot("s1")["questions"][0]["recovered"],true);
        assert_eq!(second.recovered_question(&id).unwrap()["runId"],"r1");
        assert!(second.answer_question(&id,Some("A".into())).is_err(),"a recovered question has no live tool to replay");
        second.finish_recovered(&id).unwrap();
        assert!(Broker::persistent(root.clone()).snapshot("s1")["questions"].as_array().unwrap().is_empty());
        let _=std::fs::remove_dir_all(root);
    }
    #[test]
    fn a_failed_checkpoint_does_not_create_a_live_question() {
        let root=std::env::temp_dir().join(format!("broker-unwritable-{}",std::process::id()));std::fs::write(&root,"occupied").unwrap();
        let broker=Broker::persistent(root.clone());assert!(broker.open_question_checked("s1",json!({"question":"q"})).is_err());assert_eq!(broker.pending(),0);
        std::fs::remove_file(root).unwrap();
    }
}

#[cfg(test)]
mod recovery_regressions {
    use super::*;
    use agent_kernel::journal::{IncompleteTool, RunInspection};
    fn waiting(name: &str) -> RunInspection {
        RunInspection { last_sequence: 2, state: Some(agent_kernel::RunState::Running), terminal: false,
            incomplete_tools: vec![IncompleteTool { id: "tool-1".into(), name: name.into(), dispatched: true }] }
    }
    #[test]
    fn connector_checkpoint_reopens_with_its_kind_and_run_and_can_be_closed() {
        let root = std::env::temp_dir().join(format!("connector-recovery-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let first = Broker::persistent(root.clone());
        let (id, _rx) = first.open_connector_question("s1", "r1", json!([{"reference":"installed:github"}])).unwrap();
        drop(first);
        let reopened = Broker::persistent(root.clone());
        let saved = reopened.recovered_question(&id).unwrap();
        assert_eq!(saved["runId"], "r1"); assert_eq!(saved["kind"], "connector_request");
        assert!(recovery_text(&saved, "installed:github", &waiting("connector_request")).unwrap().contains("connector_search"));
        assert!(recovery_text(&saved, "installed:other", &waiting("connector_request")).is_err());
        let mut unknown_write = waiting("connector_request");
        unknown_write.incomplete_tools.push(IncompleteTool { id: "write".into(), name: "write_file".into(), dispatched: true });
        assert!(recovery_text(&saved, "installed:github", &unknown_write).is_err());
        reopened.finish_recovered(&id).unwrap();
        assert!(Broker::persistent(root.clone()).snapshot("s1")["questions"].as_array().unwrap().is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn a_legacy_connector_card_can_be_dismissed_without_run_metadata() {
        let root = std::env::temp_dir().join(format!("legacy-connector-{}", std::process::id()));
        let b = Broker::persistent(root.clone());
        let (id, _rx) = b.open_question_checked("s1", json!({"question":"连接服务", "connectors":[]})).unwrap();
        drop(b); let reopened = Broker::persistent(root.clone());
        reopened.finish_recovered(&id).unwrap();
        assert!(reopened.recovered_question(&id).is_none()); std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn a_connector_checkpoint_write_failure_is_returned_without_panicking() {
        let root = std::env::temp_dir().join(format!("connector-checkpoint-file-{}", std::process::id()));
        std::fs::write(&root, "occupied").unwrap(); let b = Broker::persistent(root.clone());
        assert!(b.open_connector_question("s1", "r1", json!([])).is_err()); assert_eq!(b.pending(), 0);
        std::fs::remove_file(root).unwrap();
    }
}
