use agent_phone_use_tools::{package::{PatchMeta, Store}, routes::{Edge, Graph, Node, Role}};
use chrono::{Duration, Utc};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::io::{Cursor, Write};

const PACKAGE: &str = "com.example.phone";
fn archive(version: &str, extra: &[(&str,&str)]) -> Vec<u8> {
    let metadata = json!({"package_name":PACKAGE,"name":"测试应用","description":"发布图文","version":version,"verified_app_versions":[]}).to_string();
    let mut entries = vec![("skill.md", "先发现任务，再读页面。"), ("basic/package.json", metadata.as_str()),
        ("basic/pages/home.md", "基础首页"), ("basic/routes/core.yaml", "nodes: []\nedges: []")];
    entries.extend_from_slice(extra);
    zip_entries(&entries)
}
fn zip_entries(entries: &[(&str,&str)]) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, data) in entries {
        zip.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
        zip.write_all(data.as_bytes()).unwrap();
    }
    zip.finish().unwrap().into_inner()
}
fn install(store: &Store, bytes: Vec<u8>) {
    let sha = format!("{:x}", Sha256::digest(&bytes));
    store.install(PACKAGE, &bytes, &sha).unwrap();
}
fn patch(version: &str) -> PatchMeta {
    PatchMeta { app_version: "8.1".into(), based_on: version.into(), observed_at: Utc::now()-Duration::seconds(5),
        evidence: "实机观察：当前页面的新入口位于工具栏，执行后已检查目标页。".into(), expires_at: None }
}
fn node(id: &str, role: Role, confidence: f64) -> Node {
    Node { id:id.into(), title:id.into(), page:format!("pages/{id}.md"), role, priority:1.0,
        stability:1.0, confidence, enabled:true, expires_at:None }
}
fn edge(id: &str, from: &str, to: &str, confidence: f64) -> Edge {
    Edge { id:id.into(), from:from.into(), to:to.into(), action:"观察入口后点击".into(), role:Role::Feature,
        cost:1, stability:1.0, confidence, enabled:true, expires_at:None }
}

#[test]
fn install_discover_and_read_utf8_by_page() {
    let temp = tempfile::tempdir().unwrap();
    let store = Store::new(temp.path());
    install(&store, archive("1", &[]));
    assert_eq!(store.discover("发布").unwrap()["total"], 1);
    let first = store.read(PACKAGE,"pages/home.md",None,0,4,Utc::now()).unwrap();
    assert_eq!(first.content, "基");
    assert!(first.has_more);
    let rest = store.read(PACKAGE,"pages/home.md",None,first.next_offset,100,Utc::now()).unwrap();
    assert_eq!(rest.content,"础首页");
    assert!(!rest.verified_app_version);
    assert!(store.read(PACKAGE,"pages/home.md",None,1,100,Utc::now()).is_err());
}

#[test]
fn exact_version_patch_overrides_but_update_keeps_and_invalidates_it() {
    let temp = tempfile::tempdir().unwrap();
    let store = Store::new(temp.path());
    install(&store,archive("1",&[]));
    store.save_patch(PACKAGE,"pages/home.md","修正后的首页",patch("1")).unwrap();
    assert_eq!(store.read(PACKAGE,"pages/home.md",Some("8.1"),0,100,Utc::now()).unwrap().source,"patch/pages/home.md");
    for version in [None,Some("8.2")] {
        assert_eq!(store.read(PACKAGE,"pages/home.md",version,0,100,Utc::now()).unwrap().source,"basic/pages/home.md");
    }
    install(&store,archive("2",&[]));
    assert!(temp.path().join(PACKAGE).join("patch/pages/home.md").exists());
    assert_eq!(store.read(PACKAGE,"pages/home.md",Some("8.1"),0,100,Utc::now()).unwrap().source,"basic/pages/home.md");
    assert_eq!(store.patches(PACKAGE,Some("8.1"),Utc::now()).unwrap()["patches"][0]["applicable"],false);
}

#[test]
fn expired_patch_and_new_task_discovery() {
    let temp=tempfile::tempdir().unwrap(); let store=Store::new(temp.path()); install(&store,archive("1",&[]));
    let mut info=patch("1"); info.expires_at=Some(Utc::now()+Duration::minutes(1));
    store.save_patch(PACKAGE,"tasks/new.md","新学会的任务",info).unwrap();
    assert_eq!(store.patches(PACKAGE,Some("8.1"),Utc::now()).unwrap()["patches"][0]["path"],"tasks/new.md");
    assert!(store.read(PACKAGE,"tasks/new.md",Some("8.1"),0,100,Utc::now()+Duration::minutes(2)).is_err());
}

#[test]
fn corrupt_or_hostile_zip_leaves_installed_base_unchanged() {
    let temp=tempfile::tempdir().unwrap(); let store=Store::new(temp.path()); install(&store,archive("1",&[]));
    for name in ["../escape.md","basic/../../escape.md","patch/pages/home.md","basic/CON.md","basic/a:stream.md","basic/a\\b.md","basic/run.ps1","/absolute.md","basic/a. /x.md","basic/package.JSON"] {
        let bytes=archive("2",&[(name,"bad")]); let hash=format!("{:x}",Sha256::digest(&bytes));
        assert!(store.install(PACKAGE,&bytes,&hash).is_err(),"accepted {name}");
        assert_eq!(store.metadata(PACKAGE).unwrap().version,"1");
    }
    let bytes=archive("2",&[]);
    assert!(store.install(PACKAGE,&bytes,&"0".repeat(64)).is_err());
    assert!(!temp.path().join("escape.md").exists());
}

#[test]
fn single_package_wrapper_is_supported_and_identity_is_checked() {
    let temp=tempfile::tempdir().unwrap(); let store=Store::new(temp.path());
    let meta=json!({"package_name":PACKAGE,"name":"包","description":"摘要","version":"1","verified_app_versions":[]}).to_string();
    let bytes=zip_entries(&[("com.example.phone/skill.md","入口"),("com.example.phone/basic/package.json",&meta)]);
    install(&store,bytes);
    let bytes=archive("2",&[]); let hash=format!("{:x}",Sha256::digest(&bytes));
    assert!(store.install("com.other.app",&bytes,&hash).is_err());
}

#[test]
fn basic_rejects_temporary_routes_and_invalid_weights() {
    let mut graph=Graph { nodes:vec![node("campaign",Role::Temporary,1.0)],edges:vec![] };
    assert!(graph.validate(true).is_err());
    graph.nodes[0].role=Role::Core; graph.nodes[0].confidence=f64::NAN;
    assert!(graph.validate(false).is_err());
    graph.nodes[0].confidence=1.0;
    graph.edges.push(edge("broken","campaign","missing",1.0));
    assert!(graph.validate_resolved().is_err());
}

#[test]
fn route_overlay_preserves_other_ids_filters_expiry_and_honors_tombstones() {
    let mut graph=Graph { nodes:vec![node("home",Role::Core,1.0),node("editor",Role::Feature,0.2)],
        edges:vec![edge("open","home","editor",0.2)] };
    let mut updated=node("editor",Role::Feature,0.9); updated.title="新编辑页".into();
    let mut campaign=node("campaign",Role::Temporary,1.0); campaign.expires_at=Some(Utc::now()-Duration::seconds(1));
    graph.overlay(Graph {nodes:vec![updated,campaign],edges:vec![edge("activity","home","campaign",1.0)]});
    graph.filter(true,Utc::now());
    assert_eq!(graph.nodes.len(),2); assert_eq!(graph.edges.len(),1);
    assert_eq!(graph.nodes[1].title,"新编辑页");
    let mut removed=graph.nodes[1].clone(); removed.enabled=false;
    graph.overlay(Graph {nodes:vec![removed],edges:vec![]}); graph.filter(false,Utc::now());
    assert_eq!(graph.nodes.len(),1); assert!(graph.edges.is_empty());
}

#[test]
fn planner_prefers_reliable_path_over_unverified_shortcut() {
    let graph=Graph { nodes:vec![node("home",Role::Core,1.0),node("mid",Role::Feature,1.0),node("editor",Role::Feature,1.0)],
        edges:vec![edge("shortcut","home","editor",0.0),edge("first","home","mid",1.0),edge("second","mid","editor",1.0)] };
    // Equal weighted cost uses stable iteration; make the shortcut explicitly unstable too.
    let mut graph=graph; graph.edges[0].stability=0.0;
    let plan=graph.plan("home","editor").unwrap();
    assert_eq!(plan["steps"][0]["id"],"first"); assert_eq!(plan["requires_live_verification"],true);
}

#[test]
fn route_store_merges_by_id_and_temporary_requires_expiry() {
    let temp=tempfile::tempdir().unwrap(); let store=Store::new(temp.path()); install(&store,archive("1",&[]));
    let graph=Graph { nodes:vec![node("activity",Role::Temporary,0.8)],edges:vec![] };
    let yaml=serde_yaml_ng::to_string(&graph).unwrap();
    assert!(store.save_patch(PACKAGE,"routes/temporary/activity.yaml",&yaml,patch("1")).is_err());
    let mut info=patch("1"); info.expires_at=Some(Utc::now()+Duration::hours(1));
    store.save_patch(PACKAGE,"routes/temporary/activity.yaml",&yaml,info).unwrap();
    assert!(store.routes(PACKAGE,"temporary/activity",Some("8.1"),false,Utc::now()).unwrap().nodes.is_empty());
    assert_eq!(store.routes(PACKAGE,"temporary/activity",Some("8.1"),true,Utc::now()).unwrap().nodes.len(),1);
    assert!(store.routes(PACKAGE,"temporary/activity",Some("8.1"),true,Utc::now()+Duration::hours(2)).is_err());
    assert!(store.read(PACKAGE,"routes/core.yaml",None,0,100,Utc::now()).is_err());
}

#[test]
fn patch_rejects_traversal_missing_evidence_and_wrong_base() {
    let temp=tempfile::tempdir().unwrap(); let store=Store::new(temp.path()); install(&store,archive("1",&[]));
    for path in ["../skill.md","pages/../../skill.md","skill.md","pages/a.ps1","tasks/index.md"] {
        assert!(store.save_patch(PACKAGE,path,"body",patch("1")).is_err());
    }
    assert!(store.save_patch(PACKAGE,"pages/home.md","body",patch("old")).is_err());
    let mut info=patch("1"); info.evidence.clear();
    assert!(store.save_patch(PACKAGE,"pages/home.md","body",info).is_err());
}

#[cfg(unix)]
#[test]
fn linked_patch_directory_cannot_write_outside_pack() {
    let temp=tempfile::tempdir().unwrap(); let outside=tempfile::tempdir().unwrap();
    let store=Store::new(temp.path()); install(&store,archive("1",&[]));
    std::os::unix::fs::symlink(outside.path(),temp.path().join(PACKAGE).join("patch/pages")).unwrap();
    assert!(store.save_patch(PACKAGE,"pages/home.md","body",patch("1")).is_err());
    assert!(store.read(PACKAGE,"pages/home.md",Some("8.1"),0,100,Utc::now()).is_err());
    assert!(std::fs::read_dir(outside.path()).unwrap().next().is_none());
}
