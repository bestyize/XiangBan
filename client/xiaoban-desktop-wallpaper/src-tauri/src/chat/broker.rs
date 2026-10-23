//! Pending user interactions are host state, recoverable by a rebuilt window.
use agent_kernel::Approval;
use serde_json::{Value,json};
use std::{collections::HashMap,sync::{Mutex,MutexGuard,atomic::{AtomicU64,Ordering},mpsc::{self,Sender,Receiver}}};
type Waiting<T> = HashMap<String,(String,Sender<T>,Value)>;
#[derive(Default)]struct State { approvals:Waiting<Approval>,questions:Waiting<Option<String>>,revision:u64 }
#[derive(Default)]pub struct Broker {state:Mutex<State>,next:AtomicU64}
fn lock<T>(m:&Mutex<T>)->MutexGuard<'_,T>{m.lock().unwrap_or_else(|e|e.into_inner())}
impl Broker {
 fn id(&self,p:char)->String{format!("{p}{}",self.next.fetch_add(1,Ordering::Relaxed)+1)}
 pub fn open_approval(&self,s:&str)->(String,Receiver<Approval>){self.open_approval_payload(s,json!({}))}
 pub fn open_approval_payload(&self,s:&str,mut value:Value)->(String,Receiver<Approval>){let id=self.id('a');let(tx,rx)=mpsc::channel();value["id"]=json!(id);value["session"]=json!(s);let mut state=lock(&self.state);state.approvals.insert(id.clone(),(s.into(),tx,value));state.revision+=1;(id,rx)}
 pub fn open_question(&self,s:&str)->(String,Receiver<Option<String>>){self.open_question_payload(s,json!({}))}
 pub fn open_question_payload(&self,s:&str,mut value:Value)->(String,Receiver<Option<String>>){let id=self.id('q');let(tx,rx)=mpsc::channel();value["id"]=json!(id);value["session"]=json!(s);let mut state=lock(&self.state);state.questions.insert(id.clone(),(s.into(),tx,value));state.revision+=1;(id,rx)}
 pub fn answer_approval(&self,id:&str,value:Approval)->Result<(),String>{let mut s=lock(&self.state);let item=s.approvals.remove(id);s.revision+=1;item.ok_or("这个确认已经结束")?.1.send(value).map_err(|_|"这个确认已经结束".into())}
 pub fn answer_question(&self,id:&str,value:Option<String>)->Result<(),String>{let mut s=lock(&self.state);let item=s.questions.remove(id);s.revision+=1;item.ok_or("这个问题已经结束")?.1.send(value).map_err(|_|"这个问题已经结束".into())}
 pub fn close_approval(&self,id:&str){let mut s=lock(&self.state);s.approvals.remove(id);s.revision+=1;}
 pub fn close_question(&self,id:&str){let mut s=lock(&self.state);s.questions.remove(id);s.revision+=1;}
 pub fn drop_session(&self,id:&str){let mut s=lock(&self.state);s.approvals.retain(|_,(session,_,_)|session!=id);s.questions.retain(|_,(session,_,_)|session!=id);s.revision+=1;}
 pub fn snapshot(&self,id:&str)->Value{let s=lock(&self.state);json!({"revision":s.revision,"approvals":s.approvals.values().filter(|(session,_,_)|session==id).map(|(_,_,v)|v).collect::<Vec<_>>(),"questions":s.questions.values().filter(|(session,_,_)|session==id).map(|(_,_,v)|v).collect::<Vec<_>>()})}
 #[cfg(test)]pub fn pending(&self)->usize{let s=lock(&self.state);s.approvals.len()+s.questions.len()}
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
