//! Bounded, data-only local workflows. One approved tool call, local checks between every action.
use crate::{adb::{Adb,AdbStats},package::{valid_package,Result,Store},ui::{self,Selector}};
use agent_kernel::ToolCtx;
use chrono::Utc;
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{collections::BTreeMap,thread,time::{Duration,Instant}};

#[derive(Debug,Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
pub struct Parameter {pub description:String,pub max_bytes:usize}
#[derive(Debug,Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
pub struct Workflow {
    pub schema:u32,
    pub package_name:String,
    pub name:String,
    pub description:String,
    pub app_versions:Vec<String>,
    #[serde(default)] pub parameters:BTreeMap<String,Parameter>,
    pub steps:Vec<Step>,
}
#[derive(Debug,Deserialize,Serialize)]
#[serde(tag="action",rename_all="snake_case",deny_unknown_fields)]
pub enum Step {
    Launch,
    Wait {selector:Selector,#[serde(default="wait_timeout")] timeout_ms:u64},
    Tap {selector:Selector,#[serde(default="wait_timeout")] timeout_ms:u64},
    Fill {selector:Selector,parameter:String,#[serde(default="wait_timeout")] timeout_ms:u64},
    VerifyFields {parameters:Vec<String>},
    Yield {reason:String},
}
fn wait_timeout()->u64 {8000}
fn valid_selector(selector:&Selector)->bool {
    let fields=[&selector.text,&selector.description,&selector.resource_id,&selector.class];
    fields.iter().any(|field|field.is_some()) && fields.iter().all(|field|field.as_ref().is_none_or(|s|!s.is_empty()&&s.len()<=512))
}
impl Workflow {
    pub fn parse(content:&str,package:&str)->Result<Self> {
        let flow:Self=serde_yaml_ng::from_str(content).map_err(|e|format!("invalid workflow: {e}"))?;
        if flow.schema!=1 || flow.package_name!=package || !valid_package(package) || flow.name.is_empty()
            || flow.description.len()>2000 || flow.app_versions.is_empty() || flow.app_versions.len()>50
            || flow.app_versions.iter().any(|version|version.is_empty()||version.len()>100)
            || flow.steps.is_empty() || flow.steps.len()>32 || flow.parameters.len()>8 {
            return Err("invalid workflow identity, supported versions or size".into());
        }
        for (name,param) in &flow.parameters {
            if name.is_empty()||name.len()>64||!name.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_')
                || !(1..=12000).contains(&param.max_bytes)||param.description.len()>1000 {
                return Err("invalid workflow parameter declaration".into());
            }
        }
        for (index,step) in flow.steps.iter().enumerate() {
            match step {
                Step::Wait {selector,timeout_ms}|Step::Tap {selector,timeout_ms}|Step::Fill {selector,timeout_ms,..}=> {
                    if !valid_selector(selector)||!(100..=15000).contains(timeout_ms) {return Err("invalid workflow selector or wait timeout".into());}
                },
                Step::Yield {reason} if reason.trim().is_empty()||reason.len()>1000||index+1!=flow.steps.len()=>return Err("yield must be the final step with a reason".into()),
                _=>{},
            }
            if let Step::Fill {selector,parameter,..}=step {
                if !flow.parameters.contains_key(parameter)||selector.class.as_deref()!=Some("android.widget.EditText") {
                    return Err("fill requires a declared parameter and an exact EditText class".into());
                }
            }
            if let Step::VerifyFields {parameters}=step {
                let unique=parameters.iter().collect::<std::collections::BTreeSet<_>>();
                if parameters.is_empty()||parameters.len()>8||unique.len()!=parameters.len()||parameters.iter().any(|p|!flow.parameters.contains_key(p)) {
                    return Err("verify_fields requires unique declared parameters in expected editor order".into());
                }
            }
        }
        if !matches!(flow.steps.last(),Some(Step::Wait {..}|Step::VerifyFields {..}|Step::Yield {..})) {
            return Err("workflow must end with an observed postcondition or an explicit yield".into());
        }
        if serde_json::to_vec(&flow).map_err(|e|e.to_string())?.len()>12000 {
            return Err("workflow review exceeds inline budget; split it into smaller workflows".into());
        }
        Ok(flow)
    }
    pub fn validate_parameters(&self,parameters:&BTreeMap<String,String>)->Result<()> {
        if parameters.len()!=self.parameters.len() {return Err("provide exactly the declared workflow parameters".into());}
        for (name,param) in &self.parameters {
            let value=parameters.get(name).ok_or_else(||format!("missing parameter: {name}"))?;
            if value.is_empty()||value.len()>param.max_bytes||value.chars().any(|c|c=='\0'||(c.is_control()&&c!='\n'&&c!='\t')) {
                return Err(format!("invalid parameter: {name}"));
            }
        }
        Ok(())
    }
}

pub fn inspect(store:&Store,package:&str,path:&str,app_version:Option<&str>)->Result<Value> {
    let (source,content)=store.workflow_document(package,path,app_version,Utc::now())?;
    let workflow=Workflow::parse(&content,package)?;
    Ok(json!({"source":source,"sha256":format!("{:x}",Sha256::digest(content.as_bytes())),"workflow":workflow,
        "review":"Check every action against the user request. A pack cannot grant device or publishing permission."}))
}

#[derive(Serialize)]
struct StepMetric {step:usize,action:&'static str,elapsed_ms:u64,commands:u64,ok:bool}
fn milliseconds(duration:Duration)->u64 {duration.as_millis().min(u64::MAX as u128) as u64}

/// Errors after a dispatched action return a checkpoint, never restart the workflow automatically.
pub fn run(store:&Store,adb:&Adb,serial:&str,package:&str,path:&str,sha256:&str,parameters:BTreeMap<String,String>,ctx:&ToolCtx)->Result<Value> {
    let before=adb.stats();let start=Instant::now();
    if ctx.is_cancelled() {return Err("cancelled before workflow inspection".into());}
    // Read the installed version, not a model-supplied compatibility claim.
    let version=adb.app_version(serial,package,ctx)?;
    let (source,content)=store.workflow_document(package,path,Some(&version),Utc::now())?;
    if sha256.len()!=64 || format!("{:x}",Sha256::digest(content.as_bytes()))!=sha256.to_lowercase() {
        return Err("workflow changed or a different version patch applies; inspect again before running".into());
    }
    let flow=Workflow::parse(&content,package)?;
    if !flow.app_versions.contains(&version) {return Err(format!("workflow does not declare support for installed app version {version}"));}
    flow.validate_parameters(&parameters)?;
    let mut metrics=Vec::new();let mut cached:Option<(String,Instant)>=None;
    for (index,step) in flow.steps.iter().enumerate() {
        if ctx.is_cancelled() || start.elapsed()>Duration::from_secs(90) {
            return Ok(report("needs_model",index,"cancelled or workflow time budget reached; inspect current state before any continuation",&metrics,adb.stats().since(before),start,&source));
        }
        let step_start=Instant::now();let commands=adb.stats().commands;
        let action=match step {Step::Launch=>"launch",Step::Wait {..}=>"wait",Step::Tap {..}=>"tap",Step::Fill {..}=>"fill",Step::VerifyFields {..}=>"verify_fields",Step::Yield {..}=>"yield"};
        let result=(||->Result<Option<String>> {
            match step {
                Step::Launch=>{cached=None;adb.action(serial,package,"launch",&json!({}),ctx)?;},
                Step::Wait {selector,timeout_ms}=>{cached=Some(wait(adb,serial,package,selector,*timeout_ms,start,ctx,cached.take())?);},
                Step::Tap {selector,timeout_ms}=>{
                    let (xml,_)=wait(adb,serial,package,selector,*timeout_ms,start,ctx,cached.take())?;
                    check_budget(start,ctx)?;adb.tap_observed(serial,package,&xml,selector,ctx)?;
                },
                Step::Fill {selector,parameter,timeout_ms}=>{
                    let value=parameters.get(parameter).ok_or("missing parameter")?;
                    let (xml,_)=wait(adb,serial,package,selector,*timeout_ms,start,ctx,cached.take())?;
                    check_budget(start,ctx)?;adb.tap_observed(serial,package,&xml,selector,ctx)?;
                    // Never type based only on a tap returning zero; confirm the intended editor is focused.
                    let mut focused=selector.clone();focused.focused=Some(true);
                    wait(adb,serial,package,&focused,*timeout_ms,start,ctx,None)?;
                    check_budget(start,ctx)?;adb.action(serial,package,"text",&json!({"text":value}),ctx)?;
                    let mut expected=focused;expected.text=None;
                    let xml=wait_text(adb,serial,package,&expected,value,*timeout_ms,start,ctx)?;
                    cached=Some((xml,Instant::now()));
                },
                Step::Yield {reason}=>return Ok(Some(reason.clone())),
                Step::VerifyFields {parameters:fields}=>{
                    check_budget(start,ctx)?;
                    let xml=adb.checked_ui(serial,package,ctx)?;
                    check_budget(start,ctx)?;
                    let actual=ui::editable_values(&xml)?;
                    if actual.len()!=fields.len()||actual.iter().zip(fields).any(|(value,name)| {
                        parameters.get(name).is_none_or(|expected|value!=expected&&value!=&expected.replace('\n',"\\n").replace('\t',"\\t"))
                    }) {return Err("final editor field order or contents differ from requested parameters".into());}
                    cached=Some((xml,Instant::now()));
                },
            }
            Ok(None)
        })();
        metrics.push(StepMetric {step:index,action,elapsed_ms:milliseconds(step_start.elapsed()),commands:adb.stats().commands.saturating_sub(commands),ok:result.is_ok()});
        match result {
            Ok(Some(reason))=>return Ok(report("needs_model",index,&reason,&metrics,adb.stats().since(before),start,&source)),
            Err(error)=>return Ok(report("needs_model",index,&format!("{error}; do not replay prior steps automatically"),&metrics,adb.stats().since(before),start,&source)),
            Ok(None)=>{},
        }
    }
    Ok(report("completed",flow.steps.len(),"declared workflow postconditions satisfied; this does not mean a note was published",&metrics,adb.stats().since(before),start,&source))
}

fn check_budget(start:Instant,ctx:&ToolCtx)->Result<()> {
    if ctx.is_cancelled()||start.elapsed()>Duration::from_secs(90) {Err("workflow cancelled or timed out".into())} else {Ok(())}
}
fn wait(adb:&Adb,serial:&str,package:&str,selector:&Selector,timeout_ms:u64,run_start:Instant,ctx:&ToolCtx,cached:Option<(String,Instant)>)->Result<(String,Instant)> {
    if let Some((xml,at))=cached {
        if at.elapsed()<Duration::from_millis(250)&&ui::target(&xml,selector).is_ok() {return Ok((xml,at));}
    }
    let deadline=Instant::now()+Duration::from_millis(timeout_ms);
    loop {
        check_budget(run_start,ctx)?;
        let xml=adb.checked_ui(serial,package,ctx)?;
        check_budget(run_start,ctx)?;
        if Instant::now()>deadline {return Err("target-state wait timed out".into());}
        match ui::target(&xml,selector) {
            Ok(_)=>return Ok((xml,Instant::now())),
            Err(error) if error.contains("matched 0 nodes")=>{},
            Err(error)=>return Err(error),
        }
        if Instant::now()>=deadline {return Err("target-state wait timed out".into());}
        for _ in 0..10 {check_budget(run_start,ctx)?;thread::sleep(Duration::from_millis(20));}
    }
}
fn report(status:&str,step:usize,reason:&str,metrics:&[StepMetric],commands:AdbStats,start:Instant,source:&str)->Value {
    json!({"status":status,"checkpoint":step,"reason":reason,"source":source,"elapsed_ms":milliseconds(start.elapsed()),
        "steps":metrics,"adb":commands,"model_calls_inside_workflow":0,"automatic_replay":false})
}

fn wait_text(adb:&Adb,serial:&str,package:&str,selector:&Selector,value:&str,timeout_ms:u64,start:Instant,ctx:&ToolCtx)->Result<String> {
    let deadline=Instant::now()+Duration::from_millis(timeout_ms);
    let escaped=value.replace('\n',"\\n").replace('\t',"\\t");
    loop {
        check_budget(start,ctx)?;
        let (xml,_)=wait(adb,serial,package,selector,timeout_ms,start,ctx,None)?;
        let target=ui::target(&xml,selector)?;
        if Instant::now()>deadline {return Err("text verification timed out".into());}
        if target.text==value||target.text==escaped {return Ok(xml);}
        for _ in 0..10 {check_budget(start,ctx)?;thread::sleep(Duration::from_millis(20));}
    }
}
