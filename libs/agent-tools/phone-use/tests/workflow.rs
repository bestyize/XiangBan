use agent_phone_use_tools::{adb::{Adb,Transport},package::PatchMeta,seeds,workflow::{self,Workflow},Store};
use agent_kernel::ToolCtx;
use base64::{engine::general_purpose::STANDARD,Engine};
use chrono::Utc;
use serde_json::Value;
use std::{collections::BTreeMap,sync::{Arc,Mutex}};

const PACKAGE:&str="com.xingin.xhs";
#[derive(Default)]
struct State {stage:String,focused:String,title:String,body:String,actions:Vec<String>,dumps:usize,wrong_version:bool,wrong_foreground:bool,fail_tap:bool}
#[derive(Default)]
struct Device {state:Mutex<State>}
fn node(text:&str,desc:&str,y:u32,class:&str,focused:bool)->String {
    let text=text.replace('&',"&amp;").replace('"',"&quot;").replace('<',"&lt;").replace('>',"&gt;").replace('\n',"\\n");
    format!("<node text=\"{text}\" content-desc=\"{desc}\" class=\"{class}\" clickable=\"true\" enabled=\"true\" focused=\"{focused}\" bounds=\"[0,{y}][100,{}]\"/>",y+100)
}
impl Transport for Device {
    fn bound_device_channel(&self)->bool {true}
    fn run(&self,args:&[String],_:&[u8],_:&ToolCtx,_:usize)->Result<Vec<u8>,String> {
        // Test device: only generate state in memory. There is no executable, socket or phone.
        assert_eq!(&args[..3],&["-s","fake-device","shell"]);
        let command=&args[3];let mut state=self.state.lock().unwrap();
        let output=if command.contains("'dumpsys' 'package'") {
            if state.wrong_version {"versionName=999.0".into()} else {"versionName=9.51.0".into()}
        } else if command.contains("'dumpsys' 'activity'") {
            if state.wrong_foreground {"topResumedActivity=ActivityRecord{123 u0 com.other.app/.Main t1}".into()}
            else {format!("topResumedActivity=ActivityRecord{{123 u0 {PACKAGE}/.Main t1}}")}
        } else if command.contains("uiautomator dump") {
            state.dumps+=1;
            let nodes=match state.stage.as_str() {
                "home"=>node("","发布",0,"Button",false),
                "creation"=>node("从相册选择","",100,"Button",false),
                "album"=>node("草稿箱","",200,"Button",false),
                "image"=>node("下一步","",300,"Button",false),
                "editor"=>format!("{}{}",node(if state.title.is_empty(){"添加标题"}else{&state.title},"",400,"android.widget.EditText",state.focused=="title"),node(if state.body.is_empty(){"添加正文"}else{&state.body},"",500,"android.widget.EditText",state.focused=="body")),
                _=>String::new(),
            };
            format!("<hierarchy>{nodes}</hierarchy>")
        } else if command.starts_with("'monkey'") {
            state.actions.push("launch".into());state.stage="home".into();"Events injected: 1".into()
        } else if command.starts_with("'input' 'tap'") {
            state.actions.push("tap".into());
            if state.fail_tap {return Err("simulated disconnect: execution outcome unknown".into());}
            let y=command.split_whitespace().last().unwrap().trim_matches('\'').parse::<u32>().unwrap();
            match (state.stage.as_str(),y) {
                ("home",50)=>state.stage="creation".into(),("creation",150)=>state.stage="album".into(),
                ("image",350)=>state.stage="editor".into(),("editor",450)=>state.focused="title".into(),("editor",550)=>state.focused="body".into(),
                _=>return Err(format!("unexpected simulated tap: {command}")),
            }
            String::new()
        } else if command.contains("default_input_method") {"com.android.adbkeyboard/.AdbIME".into()}
        else if command.contains("'ADB_INPUT_B64'") {
            state.actions.push("input".into());
            let encoded=command.split_whitespace().last().unwrap().trim_matches('\'');
            let text=String::from_utf8(STANDARD.decode(encoded).unwrap()).unwrap();
            match state.focused.as_str() {"title"=>state.title=text,"body"=>state.body=text,_=>return Err("no focused field".into())}
            "Broadcast completed: result=0".into()
        } else {return Err(format!("unexpected simulated command: {command}"));};
        Ok(output.into_bytes())
    }
}
fn setup(stage:&str)->(tempfile::TempDir,Store,Arc<Device>,Adb) {
    let temp=tempfile::tempdir().unwrap();let store=Store::new(temp.path().join("packs"));seeds::ensure(&store).unwrap();
    let device=Arc::new(Device::default());device.state.lock().unwrap().stage=stage.into();
    let adb=Adb::with_transport(device.clone());(temp,store,device,adb)
}
fn execute(store:&Store,adb:&Adb,path:&str,parameters:BTreeMap<String,String>)->Value {
    let inspection=workflow::inspect(store,PACKAGE,path,Some("9.51.0")).unwrap();
    workflow::run(store,adb,"fake-device",PACKAGE,path,inspection["sha256"].as_str().unwrap(),parameters,&ToolCtx::detached("workflow-test")).unwrap()
}

#[test]
fn known_navigation_runs_locally_and_yields_before_ambiguous_image_selection() {
    let (_temp,store,device,adb)=setup("home");
    let report=execute(&store,&adb,"tasks/open-image-picker.yaml",BTreeMap::new());
    assert_eq!(report["status"],"needs_model");assert_eq!(report["checkpoint"],5);
    assert_eq!(report["model_calls_inside_workflow"],0);
    let state=device.state.lock().unwrap();assert_eq!(state.stage,"album");
    assert_eq!(state.actions,vec!["launch","tap","tap"]);
    assert_eq!(state.dumps,3,"wait and immediate tap must share a fresh local observation");
    assert_eq!(report["adb"]["commands"],12);
}

#[test]
fn chinese_and_long_body_are_filled_verified_together_and_never_published() {
    let (_temp,store,device,adb)=setup("image");let body=format!("{}\n第二段", "中文长正文".repeat(100));
    let report=execute(&store,&adb,"tasks/prepare-image-note.yaml",BTreeMap::from([("title".into(),"小伴测试".into()),("body".into(),body.clone())]));
    assert_eq!(report["status"],"completed","{report}");
    assert_eq!(report["model_calls_inside_workflow"],0);
    let state=device.state.lock().unwrap();assert_eq!(state.title,"小伴测试");assert_eq!(state.body,body);
    assert_eq!(state.actions,vec!["tap","tap","input","tap","input"]);
    assert_eq!(report["steps"].as_array().unwrap().last().unwrap()["action"],"verify_fields");
}

#[test]
fn mismatched_hash_or_version_and_bad_parameters_never_dispatch_input() {
    let (_temp,store,device,adb)=setup("image");let path="tasks/prepare-image-note.yaml";
    let inspect=workflow::inspect(&store,PACKAGE,path,Some("9.51.0")).unwrap();
    assert!(workflow::run(&store,&adb,"fake-device",PACKAGE,path,&"0".repeat(64),BTreeMap::new(),&ToolCtx::detached("test")).is_err());
    assert!(workflow::run(&store,&adb,"fake-device",PACKAGE,path,inspect["sha256"].as_str().unwrap(),BTreeMap::new(),&ToolCtx::detached("test")).is_err());
    device.state.lock().unwrap().wrong_version=true;
    assert!(workflow::run(&store,&adb,"fake-device",PACKAGE,path,inspect["sha256"].as_str().unwrap(),BTreeMap::new(),&ToolCtx::detached("test")).is_err());
    assert!(device.state.lock().unwrap().actions.is_empty());
}

#[test]
fn foreground_change_and_unknown_action_outcome_stop_at_checkpoint_without_retry() {
    let (_temp,store,device,adb)=setup("image");
    device.state.lock().unwrap().wrong_foreground=true;
    let parameters=BTreeMap::from([("title".into(),"标题".into()),("body".into(),"正文".into())]);
    let report=execute(&store,&adb,"tasks/prepare-image-note.yaml",parameters.clone());
    assert_eq!(report["status"],"needs_model");assert_eq!(report["checkpoint"],0);
    assert!(device.state.lock().unwrap().actions.is_empty());
    {let mut state=device.state.lock().unwrap();state.wrong_foreground=false;state.fail_tap=true;}
    let report=execute(&store,&adb,"tasks/prepare-image-note.yaml",parameters);
    assert_eq!(report["status"],"needs_model");assert_eq!(report["automatic_replay"],false);
    assert_eq!(device.state.lock().unwrap().actions,vec!["tap"]);
}

#[test]
fn learned_workflow_change_requires_a_new_inspection_hash() {
    let (_temp,store,device,adb)=setup("image");let path="tasks/prepare-image-note.yaml";
    let inspected=workflow::inspect(&store,PACKAGE,path,Some("9.51.0")).unwrap();
    let (_,original)=store.workflow_document(PACKAGE,path,Some("9.51.0"),Utc::now()).unwrap();
    store.save_patch(PACKAGE,path,&format!("# learned correction\n{original}"),PatchMeta {app_version:"9.51.0".into(),based_on:store.metadata(PACKAGE).unwrap().version,
        observed_at:Utc::now(),evidence:"simulation of an update".into(),expires_at:None}).unwrap();
    assert!(workflow::run(&store,&adb,"fake-device",PACKAGE,path,inspected["sha256"].as_str().unwrap(),BTreeMap::new(),&ToolCtx::detached("test")).unwrap_err().contains("inspect again"));
    assert!(device.state.lock().unwrap().actions.is_empty());
}

#[test]
fn malformed_or_unbounded_workflows_are_rejected() {
    for content in [
        "schema: 1\npackage_name: com.xingin.xhs\nname: test\ndescription: test\napp_versions: ['9.51.0']\nsteps: [{action: shell, command: anything}]",
        "schema: 1\npackage_name: com.xingin.xhs\nname: test\ndescription: test\napp_versions: ['9.51.0']\nsteps: [{action: launch}]",
        "schema: 1\npackage_name: com.xingin.xhs\nname: test\ndescription: test\napp_versions: ['9.51.0']\nsteps: [{action: wait, selector: {text: ready}, timeout_ms: 999999}]",
    ] {assert!(Workflow::parse(content,PACKAGE).is_err());}
}

#[test]
fn cancellation_during_a_state_wait_stops_without_input_or_step_replay() {
    use agent_kernel::{Agent,Tool,ToolSpec,ToolResult,MemoryStore,Risk};
    use agent_kernel::provider::scripted::{ScriptedProvider,Step};
    use std::time::{Duration,Instant};
    let (_temp,store,device,adb)=setup("unknown");let path="tasks/wait-only.yaml";
    let content="schema: 1\npackage_name: com.xingin.xhs\nname: wait\ndescription: simulator\napp_versions: ['9.51.0']\nsteps: [{action: wait, selector: {text: ready}}]";
    store.save_patch(PACKAGE,path,content,PatchMeta {app_version:"9.51.0".into(),based_on:store.metadata(PACKAGE).unwrap().version,observed_at:Utc::now(),evidence:"simulation".into(),expires_at:None}).unwrap();
    let inspection=workflow::inspect(&store,PACKAGE,path,Some("9.51.0")).unwrap();
    struct Run {spec:ToolSpec,store:Store,adb:Adb,hash:String}
    impl Tool for Run {
        fn spec(&self)->&ToolSpec {&self.spec}
        fn cancel_grace(&self)->Duration {Duration::from_secs(2)}
        fn execute_result(&self,_:Value,ctx:&ToolCtx)->ToolResult {
            match workflow::run(&self.store,&self.adb,"fake-device",PACKAGE,"tasks/wait-only.yaml",&self.hash,BTreeMap::new(),ctx) {
                Ok(value)=>ToolResult::ok(value.to_string()),Err(error)=>ToolResult::error(error),
            }
        }
    }
    let provider=Arc::new(ScriptedProvider::new(vec![Step::tool_call("w","workflow_test","{}"),Step::text("stopped")]));
    let agent=Agent::builder(provider,Arc::new(MemoryStore::new())).tool(Run {spec:ToolSpec::new("workflow_test","simulated local workflow",serde_json::json!({"type":"object"})).risk(Risk::Read).timeout(Duration::from_millis(100)),store,adb,hash:inspection["sha256"].as_str().unwrap().into()}).build().unwrap();
    let start=Instant::now();agent.run("cancel-flow","local test",&mut |_|{},&Arc::new(std::sync::atomic::AtomicBool::new(false))).unwrap();
    assert!(start.elapsed()<Duration::from_secs(3));assert!(device.state.lock().unwrap().actions.is_empty());
}
