mod common;

use agent_kernel::provider::scripted::Step;
use agent_kernel::provider::{anthropic, responses, wire};
use agent_kernel::*;
use common::*;
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[test]
fn schemas_are_compiled_at_registration_and_cannot_fetch_external_documents() {
    for schema in [
        json!({"type":"unknown"}),
        json!({"$ref":"file:///etc/passwd"}),
        json!({"$ref":"https://example.com/schema.json"}),
    ] {
        let provider = Arc::new(agent_kernel::provider::scripted::ScriptedProvider::new(
            vec![],
        ));
        let tool = FnTool::new(ToolSpec::new("invalid", "d", schema), |_, _| {
            panic!("not called")
        });
        assert!(matches!(
            Agent::builder(provider, Arc::new(MemoryStore::new()))
                .tool(tool)
                .build(),
            Err(HarnessError::Config(_))
        ));
    }
}

#[test]
fn validation_blocks_invalid_arguments_before_approval_or_execution() {
    let executed = Arc::new(AtomicUsize::new(0));
    let approved = Arc::new(AtomicUsize::new(0));
    let count = executed.clone();
    let tool = FnTool::new(
        ToolSpec::new(
            "checked",
            "d",
            json!({
                "type":"object", "additionalProperties": false,
                "$defs":{"item":{"type":"integer","minimum":1}},
                "properties":{"items":{"type":"array","items":{"$ref":"#/$defs/item"},"minItems":1}, "mode":{"enum":["fast","safe"]}},
                "required":["items","mode"]
            }),
        ),
        move |_, _| {
            count.fetch_add(1, Ordering::SeqCst);
            Ok("ran".into())
        },
    );
    let args = [
        "{}",
        "[]",
        r#"{"items":[0],"mode":"fast"}"#,
        r#"{"items":[1],"mode":"wrong"}"#,
        r#"{"items":[1],"mode":"safe","extra":true}"#,
    ];
    let mut steps: Vec<_> = args
        .iter()
        .enumerate()
        .map(|(i, a)| Step::tool_call(&format!("c{i}"), "checked", a))
        .collect();
    steps.push(Step::text("done"));
    let approvals = approved.clone();
    let h = harness(steps, |b| {
        b.tool(tool).middleware(Permission::new(
            PermissionMode::Ask,
            move |_: &ApprovalRequest<'_>| {
                approvals.fetch_add(1, Ordering::SeqCst);
                Approval::Once
            },
        ))
    });
    assert!(run(&h.agent, "go").0.is_ok());
    assert_eq!(executed.load(Ordering::SeqCst), 0);
    assert_eq!(approved.load(Ordering::SeqCst), 0);
    assert!(h
        .store
        .load("s1")
        .unwrap()
        .iter()
        .filter(|m| m.role == Role::Tool)
        .all(|m| m.is_error));
    assert!(tool_messages(&h)[2].1.contains("/items/0"));
    let first = &tool_messages(&h)[0].1;
    assert!(first.contains("\"items\" is a required property") && first.contains("\"mode\" is a required property"));
    let invalid = h.store.load("s1").unwrap().into_iter().find(|m| m.tool_call_id.as_deref() == Some("c0")).unwrap();
    assert_eq!(invalid.structured_content.unwrap()["harness_status"], "invalid_arguments");
}

#[test]
fn defaults_are_conservative_and_unclassified_tools_are_blocked_in_read_only_mode() {
    let s = ToolSpec::new("unclassified", "d", json!({"type":"object"}));
    assert_eq!(s.risk, Risk::Exec);
    assert!(!s.parallel_safe);
    let h = harness(
        vec![Step::tool_call("c", "unclassified", "{}"), Step::text("ok")],
        |b| {
            b.tool(FnTool::new(s, |_, _| panic!("must not run")))
                .middleware(Permission::new(
                    PermissionMode::ReadOnly,
                    |_: &ApprovalRequest<'_>| panic!("must not ask"),
                ))
        },
    );
    assert!(run(&h.agent, "go").0.is_ok());
    assert!(tool_messages(&h)[0].1.contains("Read-only"));
}

struct RichTool(ToolSpec);
impl Tool for RichTool {
    fn spec(&self) -> &ToolSpec {
        &self.0
    }
    fn execute(&self, _: Value, _: &ToolCtx) -> Result<String, String> {
        panic!("harness must use rich entry point")
    }
    fn execute_result(&self, _: Value, _: &ToolCtx) -> ToolResult {
        ToolResult::error("partial result")
            .with_part(Part::Image {
                url: "data:image/png;base64,AAAA".into(),
            })
            .with_structured_content(json!({"reason":"test","retryable":false}))
    }
}

fn rich_result() -> Message {
    RichTool(spec("rich"))
        .execute_result(json!({}), &ToolCtx::detached("s"))
        .message("c")
}

#[test]
fn rich_results_survive_execution_events_and_jsonl_reload_and_old_records_still_load() {
    let h = harness(
        vec![Step::tool_call("c", "rich", "{}"), Step::text("ok")],
        |b| b.tool(RichTool(spec("rich"))),
    );
    let (_, events) = run(&h.agent, "go");
    let result = h
        .store
        .load("s1")
        .unwrap()
        .into_iter()
        .find(|m| m.role == Role::Tool)
        .unwrap();
    assert_eq!(result, rich_result());
    assert!(events.iter().any(|e|matches!(e,Event::ToolEnd{is_error:true,parts,structured_content:Some(_),..} if parts.len()==1)));
    let dir = TempDir::new("rich-store");
    let store = JsonlStore::new(&dir.0);
    store.append("s", &[result.clone()]).unwrap();
    assert_eq!(JsonlStore::new(&dir.0).load("s").unwrap(), vec![result]);
    let old: Message = serde_json::from_value(
        json!({"role":"tool","parts":[{"type":"text","text":"old"}],"tool_call_id":"old"}),
    )
    .unwrap();
    assert!(!old.is_error && old.structured_content.is_none() && old.context_summary.is_none());
}

#[test]
fn three_protocols_preserve_error_json_and_images_without_breaking_tool_pairs() {
    let profile = wire::ProviderProfile::new("http://local", "", "model");
    let request = ChatRequest {
        messages: vec![
            Message::assistant(
                "",
                vec![
                    ToolCall {
                        id: "c".into(),
                        name: "rich".into(),
                        arguments: "{}".into(),
                    },
                    ToolCall {
                        id: "d".into(),
                        name: "text".into(),
                        arguments: "{}".into(),
                    },
                ],
            ),
            rich_result(),
            Message::tool("d", "second result"),
            Message::assistant("done", vec![]),
        ],
        tools: vec![],
    };
    let chat = wire::build_request_body(&profile, &request);
    assert!(chat["messages"][1]["content"]
        .as_str()
        .unwrap()
        .starts_with("Error:"));
    assert!(chat["messages"][1]["content"]
        .as_str()
        .unwrap()
        .contains("retryable"));
    assert_eq!(chat["messages"][2]["tool_call_id"], "d");
    assert_eq!(chat["messages"][3]["role"], "user");
    assert_eq!(
        chat["messages"][3]["content"][1]["image_url"]["url"],
        "data:image/png;base64,AAAA"
    );
    assert_eq!(chat["messages"][4]["role"], "assistant");
    let responses = responses::build_request_body(&profile, &request);
    let output = &responses["input"][2]["output"];
    assert!(output.as_str().unwrap().starts_with("Error:"));
    assert!(output.as_str().unwrap().contains("retryable"));
    assert_eq!(responses["input"][3]["call_id"], "d");
    assert_eq!(responses["input"][4]["role"], "user");
    assert_eq!(responses["input"][4]["content"][1]["image_url"], "data:image/png;base64,AAAA");
    assert_eq!(responses["input"][5]["role"], "assistant");
    let claude = anthropic::build_request_body(&profile, &request);
    let result = &claude["messages"][1]["content"][0];
    assert_eq!(result["is_error"], true);
    assert_eq!(result["content"][1]["source"]["data"], "AAAA");
    assert!(result["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("retryable"));
}

#[test]
fn context_accounts_for_tool_definitions_output_reserve_and_structured_results() {
    let mut turn = Turn {
        session: "s".into(),
        index: 0,
        messages: vec![Message::user_text("go")],
        tools: vec![ToolSchema {
            name: "large".into(),
            description: "x".repeat(1000),
            parameters: json!({"type":"object"}),
        }],
    };
    assert!(ContextTrim::new(500)
        .reserve_output(300)
        .before_model(&mut turn)
        .is_err());
    let plain = Message::tool("c", "tiny");
    let mut rich = plain.clone();
    rich.structured_content = Some(json!({"data":"x".repeat(1000)}));
    assert!(estimate_tokens(&[rich]) > estimate_tokens(&[plain]) + 200);
    let h = harness(vec![Step::text("must not be requested")], |b| {
        b.middleware(ContextTrim::new(10).reserve_output(10))
    });
    assert!(run(&h.agent, "too big").0.is_err());
    assert!(h.provider.requests().is_empty());
}

#[test]
fn result_limits_do_not_persist_corrupt_partial_json_or_image_data() {
    let h = harness(
        vec![Step::tool_call("c", "rich", "{}"), Step::text("done")],
        |b| b.tool(RichTool(spec("rich").max_output(8))),
    );
    run(&h.agent, "go").0.unwrap();
    let result = h
        .store
        .load("s1")
        .unwrap()
        .into_iter()
        .find(|m| m.role == Role::Tool)
        .unwrap();
    assert!(result.is_error);
    assert!(
        result.structured_content.is_none()
            && result.parts.iter().all(|p| matches!(p, Part::Text { .. }))
    );
}

#[test]
fn repeated_screenshots_with_different_images_are_not_treated_as_a_stuck_tool() {
    let guard = RepeatGuard::new();
    let call = ToolCall {
        id: "c".into(),
        name: "screenshot".into(),
        arguments: "{}".into(),
    };
    for i in 0..8 {
        let mut result = ToolResult::ok("screen").with_part(Part::Image {
            url: format!("data:image/png;base64,{i}"),
        });
        guard.after_tool("s", &call, &mut result);
        assert!(!result.output.contains("in a row"));
        assert!(matches!(
            guard.before_tool(
                &ToolCtx::detached("s"),
                &call,
                &json!({}),
                &spec("screenshot")
            ),
            ToolGate::Proceed
        ));
    }
}
