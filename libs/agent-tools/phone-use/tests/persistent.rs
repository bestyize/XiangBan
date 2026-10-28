use agent_phone_use_tools::{adb::Transport,persistent::PersistentTransport};
use agent_kernel::{ToolCtx,Tool,ToolSpec,ToolResult,Agent,MemoryStore,Risk};
use agent_kernel::provider::scripted::{ScriptedProvider,Step};
use serde_json::{json,Value};
use std::{sync::Arc,time::{Duration,Instant}};

fn transport(log:&std::path::Path)->PersistentTransport {
    PersistentTransport::new(if cfg!(windows) {"python"} else {"python3"}).launch_prefix(vec![
        format!("{}/tests/fixtures/fake_adb.py",env!("CARGO_MANIFEST_DIR")),"--log".into(),log.display().to_string()])
}
fn run(transport:&PersistentTransport,serial:&str,command:&str,max:usize)->Result<Vec<u8>,String> {
    transport.run(&["-s".into(),serial.into(),"shell".into(),command.into()],&[],&ToolCtx::detached("persistent-test"),max)
}

#[test]
fn channel_reuses_one_process_and_preserves_empty_multiline_and_markerlike_output() {
    let dir=tempfile::tempdir().unwrap();let log=dir.path().join("commands.log");let client=transport(&log);
    assert_eq!(run(&client,"fake-one","OK",100).unwrap(),b"ok");
    assert_eq!(run(&client,"fake-one","EMPTY",100).unwrap(),b"");
    assert_eq!(run(&client,"fake-one","MULTI",100).unwrap(),b"alpha\nbeta\n");
    assert_eq!(run(&client,"fake-one","MARKERLIKE",100).unwrap(),b"xb-other:END:0\nkept");
    assert_eq!(client.stats().shell_starts,1);assert_eq!(client.stats().shell_reuses,3);
    client.close().unwrap();
    assert_eq!(std::fs::read_to_string(log).unwrap().lines().filter(|line|line.starts_with("start")).count(),1);
}

#[test]
fn two_devices_have_separate_channels_and_invalid_serials_do_not_spawn() {
    let dir=tempfile::tempdir().unwrap();let client=transport(&dir.path().join("commands.log"));
    run(&client,"fake-one","OK",100).unwrap();run(&client,"fake-two","OK",100).unwrap();
    assert_eq!(client.stats().shell_starts,2);
    assert!(run(&client,"-s evil","OK",100).is_err());assert_eq!(client.stats().shell_starts,2);
    client.close().unwrap();
}

#[test]
fn disconnect_never_replays_the_dispatched_command_and_new_calls_use_a_clean_channel() {
    let dir=tempfile::tempdir().unwrap();let log=dir.path().join("commands.log");let client=transport(&log);
    assert!(run(&client,"fake-one","DROP",100).unwrap_err().contains("outcome unknown"));
    assert_eq!(client.stats().shell_starts,1);
    assert_eq!(run(&client,"fake-one","OK",100).unwrap(),b"ok");
    assert_eq!(client.stats().shell_starts,2);client.close().unwrap();
    assert_eq!(std::fs::read_to_string(log).unwrap().lines().filter(|line|line.ends_with(" DROP")).count(),1);
}

#[test]
fn output_overflow_and_nonzero_exit_close_the_channel() {
    let dir=tempfile::tempdir().unwrap();let client=transport(&dir.path().join("commands.log"));
    assert!(run(&client,"fake-one","BIG",16).unwrap_err().contains("budget"));
    assert!(run(&client,"fake-one","FAIL",100).unwrap_err().contains("exited 9"));
    assert_eq!(run(&client,"fake-one","OK",100).unwrap(),b"ok");
    assert_eq!(client.stats().shell_starts,3);client.close().unwrap();
}

#[test]
fn cancellation_kills_the_resident_client_and_a_later_call_does_not_read_old_output() {
    struct Call {spec:ToolSpec,transport:Arc<PersistentTransport>}
    impl Tool for Call {
        fn spec(&self)->&ToolSpec {&self.spec}
        fn cancel_grace(&self)->Duration {Duration::from_secs(3)}
        fn execute_result(&self,_:Value,ctx:&ToolCtx)->ToolResult {
            match self.transport.run(&["-s".into(),"fake-one".into(),"shell".into(),"SLEEP".into()],&[],ctx,100) {
                Ok(_)=>ToolResult::ok("unexpected completion"),Err(error)=>ToolResult::error(error),
            }
        }
    }
    let dir=tempfile::tempdir().unwrap();let client=Arc::new(transport(&dir.path().join("commands.log")));
    let provider=Arc::new(ScriptedProvider::new(vec![Step::tool_call("call","resident_test","{}"),Step::text("stopped")]));
    let agent=Agent::builder(provider,Arc::new(MemoryStore::new())).tool(Call {spec:ToolSpec::new("resident_test","local simulator",json!({"type":"object"})).risk(Risk::Read).timeout(Duration::from_millis(200)),transport:client.clone()}).build().unwrap();
    let start=Instant::now();agent.run("test","test cancellation",&mut |_|{},&Arc::new(std::sync::atomic::AtomicBool::new(false))).unwrap();
    assert!(start.elapsed()<Duration::from_secs(5));
    assert_eq!(run(&client,"fake-one","OK",100).unwrap(),b"ok");
    assert_eq!(client.stats().shell_starts,2);client.close().unwrap();
}
