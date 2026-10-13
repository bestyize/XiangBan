use agent_kernel::{Agent, HarnessError, MemoryStore, Message, ModelProvider, Part, Role, SessionStore};
use agent_kernel::provider::{ChatRequest, Reply};
use std::sync::{Arc, Mutex, atomic::AtomicBool};
struct Fails { requests: Mutex<Vec<ChatRequest>> }
impl ModelProvider for Fails {
    fn chat(&self, request: &ChatRequest, _: &mut dyn FnMut(&str), _: &AtomicBool) -> Result<Reply, HarnessError> {
        self.requests.lock().unwrap().push(request.clone());
        Err(HarnessError::provider("model_not_found", false))
    }
}
#[test]
fn retry_reuses_images_and_does_not_append_another_user_message() {
    let store = Arc::new(MemoryStore::new());
    let provider = Arc::new(Fails { requests: Mutex::new(Vec::new()) });
    let agent = Agent::builder(provider.clone(), store.clone()).build().unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let input = vec![Part::Text { text: "describe".into() }, Part::Image { url: "data:image/png;base64,x".into() }];
    assert!(agent.run_parts("s", input.clone(), &mut |_| {}, &cancel).is_err());
    assert!(agent.retry_last("s", &mut |_| {}, &cancel).is_err());
    let history = store.load("s").unwrap();
    assert_eq!(history.iter().filter(|m| m.role == Role::User).count(), 1);
    assert_eq!(history[0].parts, input);
    let requests = provider.requests.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].messages, requests[1].messages);
}
#[test]
fn retry_refuses_a_turn_that_has_persisted_output_or_tool_actions() {
    let store = Arc::new(MemoryStore::new());
    store.append("s", &[Message::user_text("hello"), Message::assistant("already answered", vec![])]).unwrap();
    let provider = Arc::new(Fails { requests: Mutex::new(Vec::new()) });
    let agent = Agent::builder(provider.clone(), store).build().unwrap();
    assert!(agent.retry_last("s", &mut |_| {}, &Arc::new(AtomicBool::new(false))).is_err());
    assert!(provider.requests.lock().unwrap().is_empty());
}
