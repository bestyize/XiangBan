use agent_phone_use_tools::{adb::{Adb,Transport,ProcessTransport,shell_quote,validate_endpoint},seeds,Store};
use agent_kernel::{Part,ToolCtx,Risk};
use base64::{engine::general_purpose::STANDARD,Engine};
use serde_json::{json,Value};
use std::sync::{Arc,Mutex};

const PACKAGE:&str="com.xingin.xhs";
const PNG:&str="iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";
#[derive(Default)]
struct Fake {
    calls:Mutex<Vec<(Vec<String>,Vec<u8>)>>,
    wrong_foreground:bool,
    unicode_ime:bool,
}
impl Transport for Fake {
    fn run(&self,args:&[String],input:&[u8],_:&ToolCtx,_:usize)->Result<Vec<u8>,String> {
        self.calls.lock().unwrap().push((args.to_vec(),input.to_vec()));
        if args.first().is_some_and(|s|s=="devices") {
            return Ok(b"List of devices attached\nusb-one device product:test model:Test\nusb-two unauthorized\n".to_vec());
        }
        let joined=args.join(" ");
        let text=if joined.contains("'dumpsys' 'activity'") {
            if self.wrong_foreground {"mResumedActivity: ActivityRecord{123 u0 com.other.app/.Main t12}"}
            else {"mResumedActivity: ActivityRecord{123 u0 com.xingin.xhs/.Main t12}"}
        } else if joined.contains("'dumpsys' 'package'") {"versionCode=123\nversionName=8.1\n"}
        else if joined.contains("'wm' 'size'") {"Physical size: 1080x1920"}
        else if joined.contains("'uiautomator'") {"<?xml version=\"1.0\"?><hierarchy><node text=\"你好\" /></hierarchy>\nUI hierarchy dumped to: /dev/tty"}
        else if joined.contains("default_input_method") {
            if self.unicode_ime {"com.android.adbkeyboard/.AdbIME\n"} else {"com.example.keyboard/.IME\n"}
        }
        else if joined.contains("exec-out screencap") {return Ok(STANDARD.decode(PNG).unwrap());}
        else if args.first().is_some_and(|s|s=="pair") {"Successfully paired"}
        else {""};
        Ok(text.as_bytes().to_vec())
    }
}
fn ctx()->ToolCtx {ToolCtx::detached("phone-test")}

#[test]
fn foreground_mismatch_blocks_input_and_unauthorized_device_blocks_all_actions() {
    let fake=Arc::new(Fake {wrong_foreground:true,..Default::default()}); let adb=Adb::with_transport(fake.clone());
    let error=adb.action("usb-one",PACKAGE,"tap",&json!({"x":100,"y":200}),&ctx()).unwrap_err();
    assert!(error.contains("foreground changed"));
    assert!(!fake.calls.lock().unwrap().iter().any(|(a,_)|a.join(" ").contains("'input'")));
    assert!(adb.action("usb-two",PACKAGE,"launch",&json!({}),&ctx()).is_err());
    assert!(adb.action("-s evil",PACKAGE,"launch",&json!({}),&ctx()).is_err());
}

#[test]
fn ascii_input_is_remote_shell_quoted_without_command_interpolation() {
    let fake=Arc::new(Fake::default()); let adb=Adb::with_transport(fake.clone());
    let input="hello ' ; $(touch /sdcard/evil) &";
    adb.action("usb-one",PACKAGE,"text",&json!({"text":input}),&ctx()).unwrap();
    let calls=fake.calls.lock().unwrap(); let last=&calls.last().unwrap().0;
    assert_eq!(last.len(),4);
    assert_eq!(last[3],format!("'input' 'text' {}",shell_quote(&input.replace(' ',"%s"))));
    assert!(last[3].contains("'\\''"));
}

#[test]
fn unicode_requires_selected_ime_and_uses_base64_targeted_broadcast() {
    let fake=Arc::new(Fake::default()); let adb=Adb::with_transport(fake.clone());
    assert!(adb.action("usb-one",PACKAGE,"text",&json!({"text":"你好，小伴"}),&ctx()).unwrap_err().contains("ADB Keyboard"));
    assert!(!fake.calls.lock().unwrap().iter().any(|(a,_)|a.join(" ").contains("broadcast")));
    let fake=Arc::new(Fake {unicode_ime:true,..Default::default()}); let adb=Adb::with_transport(fake.clone());
    adb.action("usb-one",PACKAGE,"text",&json!({"text":"你好，小伴"}),&ctx()).unwrap();
    let calls=fake.calls.lock().unwrap(); let command=calls.last().unwrap().0.join(" ");
    assert!(command.contains("'ADB_INPUT_B64' '-p' 'com.android.adbkeyboard'"));
    assert!(command.contains(&STANDARD.encode("你好，小伴")));
    assert!(!command.contains("你好"));
}

#[test]
fn pair_code_is_stdin_not_a_command_argument_and_invalid_endpoints_never_dispatch() {
    let fake=Arc::new(Fake::default()); let adb=Adb::with_transport(fake.clone());
    adb.connect("pair","192.168.1.2:37000",Some("123456"),&ctx()).unwrap();
    let calls=fake.calls.lock().unwrap();
    assert_eq!(calls[0].0,vec!["pair","192.168.1.2:37000"]);
    assert_eq!(calls[0].1,b"123456\n"); drop(calls);
    for bad in ["host","host:0","host:65536","-s:123","host;echo:123","host:123\n"] {
        assert!(validate_endpoint(bad).is_err(),"accepted {bad}");
    }
    assert!(adb.connect("pair","192.168.1.2:37000",Some("12;456"),&ctx()).is_err());
    assert_eq!(fake.calls.lock().unwrap().len(),1);
}

#[test]
fn status_returns_actual_foreground_version_and_ui_is_complete() {
    let adb=Adb::with_transport(Arc::new(Fake::default()));
    let status=adb.status("usb-one",None,&ctx()).unwrap();
    assert_eq!(status["foreground_package"],PACKAGE); assert_eq!(status["app_version"],"8.1");
    assert!(adb.ui("usb-one",&ctx()).unwrap().ends_with("</hierarchy>"));
}

#[test]
fn screenshot_reaches_tool_result_as_media_not_base64_text() {
    let temp=tempfile::tempdir().unwrap(); let store=Store::new(temp.path().join("packs"));
    let tools=agent_phone_use_tools::tools(store,Adb::with_transport(Arc::new(Fake::default())),temp.path().to_path_buf());
    let tool=tools.iter().find(|t|t.spec().name=="phone_observe").unwrap();
    let result=tool.execute_result(json!({"serial":"usb-one","kind":"screenshot"}),&ctx());
    assert!(!result.is_error,"{}",result.output); assert!(!result.output.contains(PNG));
    assert!(matches!(&result.parts[0],Part::Image {url} if url.ends_with(PNG)));
    assert_eq!(result.structured_content.unwrap()["width"],1);
}

#[test]
fn seeds_can_be_discovered_read_and_planned_without_touching_adb() {
    let temp=tempfile::tempdir().unwrap(); let store=Store::new(temp.path().join("packs"));
    let fake=Arc::new(Fake::default());
    let tools=agent_phone_use_tools::tools(store.clone(),Adb::with_transport(fake.clone()),temp.path().to_path_buf());
    let find=tools.iter().find(|t|t.spec().name=="phone_pack_find").unwrap();
    let result=find.execute_result(json!({"query":"小红书"}),&ctx()); assert!(!result.is_error,"{}",result.output);
    let found:Value=serde_json::from_str(&result.output).unwrap(); assert_eq!(found["total"],1);
    assert!(found["packs"][0]["verified_app_versions"].as_array().unwrap().is_empty());
    let graph=store.routes(PACKAGE,"flows/publishing",None,false,chrono::Utc::now()).unwrap();
    assert_eq!(graph.plan("home","result").unwrap()["steps"].as_array().unwrap().len(),3);
    for seed in seeds::SEEDS {
        let graph=store.routes(seed.package,if seed.package==PACKAGE {"flows/publishing"} else {"flows/playing"},None,false,chrono::Utc::now()).unwrap();
        for node in graph.nodes {assert!(!store.read(seed.package,&node.page,None,0,8192,chrono::Utc::now()).unwrap().content.is_empty());}
    }
    assert!(fake.calls.lock().unwrap().is_empty());
}

#[test]
fn data_tools_have_distinct_permissions_and_native_device_actions_are_not_self_managed() {
    let temp=tempfile::tempdir().unwrap();
    let tools=agent_phone_use_tools::tools(Store::new(temp.path().join("packs")),Adb::with_transport(Arc::new(Fake::default())),temp.path().to_path_buf());
    assert_eq!(tools.len(),12);
    for tool in tools {
        let spec=tool.spec(); assert!(spec.untrusted_output); assert!(!spec.parallel_safe);
        if matches!(spec.name.as_str(),"phone_action"|"phone_push_image"|"phone_connect") {assert_eq!(spec.risk,Risk::Exec);assert!(!spec.self_managed);}
        if spec.name=="phone_pack_install" {assert!(spec.always_ask);}
    }
}

#[test]
fn match3_finds_valid_swap_and_rejects_unsettled_or_malformed_boards() {
    let board=vec![vec![1,2,1],vec![2,1,3],vec![3,1,2]];
    let result=agent_phone_use_tools::match3::candidates(&board).unwrap();
    assert!(result["total"].as_u64().unwrap()>0);
    assert!(result["candidates"].as_array().unwrap().iter().all(|v|v["score"].as_u64().unwrap()>=3));
    for bad in [vec![],vec![vec![1,1,1];3],vec![vec![1,2,3],vec![2],vec![3,1,2]]] {
        assert!(agent_phone_use_tools::match3::candidates(&bad).is_err());
    }
    let no_moves=agent_phone_use_tools::match3::candidates(&[vec![0;3],vec![0;3],vec![0;3]]).unwrap();
    assert_eq!(no_moves["total"],0);
}

#[test]
fn image_transfer_cannot_read_outside_workspace() {
    let temp=tempfile::tempdir().unwrap(); let outside=tempfile::tempdir().unwrap();
    let path=outside.path().join("private.png");std::fs::write(&path,b"private").unwrap();
    let fake=Arc::new(Fake::default());let adb=Adb::with_transport(fake.clone());
    assert!(adb.push_image("usb-one",&path,temp.path(),&ctx()).is_err());
    assert!(!fake.calls.lock().unwrap().iter().any(|(a,_)|a.iter().any(|s|s=="push")));
}

#[test]
fn actual_process_transport_bounds_output_and_reports_spawn_errors() {
    #[cfg(windows)] let (exe,args)=("powershell",vec!["-NoProfile","-NonInteractive","-Command","[Console]::Out.Write('x' * 1000)"]);
    #[cfg(not(windows))] let (exe,args)=("sh",vec!["-c","printf '%01000d' 1"]);
    let transport=ProcessTransport::new(exe);
    let args=args.into_iter().map(str::to_string).collect::<Vec<_>>();
    assert!(transport.run(&args,&[],&ctx(),16).unwrap_err().contains("budget"));
    assert!(ProcessTransport::new("nonexistent-adb-for-test").run(&[],&[],&ctx(),100).unwrap_err().contains("Platform Tools"));
}

#[test]
fn timed_out_tool_cancels_the_actual_client_process_promptly() {
    use agent_kernel::{Agent,Tool,ToolResult,ToolSpec,MemoryStore};
    use agent_kernel::provider::scripted::{ScriptedProvider,Step};
    use std::time::{Duration,Instant};
    struct Wait {spec:ToolSpec}
    impl Tool for Wait {
        fn spec(&self)->&ToolSpec {&self.spec}
        fn cancel_grace(&self)->Duration {Duration::from_secs(3)}
        fn execute_result(&self,_:Value,ctx:&ToolCtx)->ToolResult {
            #[cfg(windows)] let (exe,args)=("powershell",vec!["-NoProfile","-NonInteractive","-Command","Start-Sleep -Seconds 10"]);
            #[cfg(not(windows))] let (exe,args)=("sleep",vec!["10"]);
            match ProcessTransport::new(exe).run(&args.into_iter().map(str::to_string).collect::<Vec<_>>(),&[],ctx,100) {
                Ok(_)=>ToolResult::ok("unexpected completion"), Err(error)=>ToolResult::error(error),
            }
        }
    }
    let provider=Arc::new(ScriptedProvider::new(vec![Step::tool_call("wait","client_wait","{}"),Step::text("stopped")]));
    let agent=Agent::builder(provider.clone(),Arc::new(MemoryStore::new()))
        .tool(Wait {spec:ToolSpec::new("client_wait","test process cancellation",json!({"type":"object"})).risk(Risk::Read).timeout(Duration::from_millis(200))})
        .build().unwrap();
    let started=Instant::now();
    agent.run("cancel-test","run the test client",&mut |_|{},&Arc::new(std::sync::atomic::AtomicBool::new(false))).unwrap();
    assert!(started.elapsed()<Duration::from_secs(5));
    assert!(provider.requests().last().unwrap().messages.iter().any(|m|m.text().contains("cancelled or timed out")));
}
