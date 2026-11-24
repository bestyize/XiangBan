use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Native {
    spec: ToolSpec,
    invoke: Box<dyn Fn(Value) -> ToolResult + Send + Sync>,
}
impl Tool for Native {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }
    fn execute_result(&self, args: Value, _: &ToolCtx) -> ToolResult {
        (self.invoke)(args)
    }
}
fn native(name: &str, invoke: impl Fn(Value) -> ToolResult + Send + Sync + 'static) -> NativeTool {
    Arc::new(Native {
        spec: ToolSpec::new(name, "fixture", json!({})),
        invoke: Box::new(invoke),
    })
}
fn window(id: u32) -> Window {
    Window {
        id,
        title: "Board".into(),
        bounds: json!({"x":1,"y":2,"w":100,"h":50}),
    }
}
fn frame() -> ToolResult {
    let header = *b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\0d\0\0\0\x32";
    ToolResult::ok("App=lab (pid 2)\nWindow: \"Board\", App: Lab.\n0 standard window Board\n\t1 menu button Tools\n2 menu bar")
        .with_part(Part::Image {url:format!("data:image/png;base64,{}",STANDARD.encode(header))})
}
fn path_tool(
    ids: Vec<u32>,
    click: impl Fn(Value) -> ToolResult + Send + Sync + 'static,
) -> WindowTool {
    let n = AtomicUsize::new(0);
    let query = native("query", move |_| {
        let index = n.fetch_add(1, Ordering::SeqCst);
        let id = ids[index.min(ids.len() - 1)];
        ToolResult::ok(json!({"window_id":id,"matches":[{"role":"AXWindow","title":"Board","bounds":window(id).bounds}]}).to_string())
    });
    let inner = native("click", click);
    WindowTool {
        spec: inner.spec().clone(),
        inner,
        query,
        key: None,
        activity: None,
        state: native("get_app_state", |args| {
            let mut state = frame();
            if args["include_screenshot"] == false {
                state.parts.clear();
            }
            state
        }),
        observed: Arc::new(Mutex::new(Some(Observation {
            app: "lab".into(),
            window: window(7),
            pixels: (100, 50),
        }))),
    }
}
fn plan() -> Value {
    json!({"app":"lab","window_id":7,"points":[{"x":1,"y":2},{"x":60,"y":2,"click_count":2}]})
}

#[test]
fn ordered_path_uses_background_clicks_and_only_returns_last_frame() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let received = calls.clone();
    let tool = path_tool(vec![7], move |args| {
        received.lock().unwrap().push(args);
        frame()
    });
    let result = tool.execute_result(plan(), &ToolCtx::detached("s"));
    assert!(!result.is_error, "{}", result.output);
    assert_eq!(result.parts.len(), 1);
    let log = calls.lock().unwrap();
    assert_eq!(
        log[0],
        json!({"app":"lab","x":1,"y":2,"click_count":1,"click_method":"sky_click"})
    );
    assert_eq!(log[1]["click_count"], 2);
}

#[test]
fn invalid_later_point_or_stale_binding_never_dispatches_the_first_point() {
    let tool = || path_tool(vec![7], |_| panic!("should not click"));
    let ctx = ToolCtx::detached("s");
    for k in ["x", "click_method", "mouse_button"] {
        let mut p = plan();
        p[k] = json!("global");
        assert!(tool().execute_result(p, &ctx).is_error);
    }
    let mut p = plan();
    p["points"][1]["x"] = json!(100);
    assert!(tool().execute_result(p, &ctx).is_error);
    let mut p = plan();
    p["window_id"] = json!(8);
    assert!(tool().execute_result(p, &ctx).is_error);
    let mut p = plan();
    p["points"][1]["click_count"] = json!(3);
    assert!(tool().execute_result(p, &ctx).is_error);
}

#[test]
fn same_title_different_window_stops_remaining_actions_without_replay() {
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let tool = path_tool(vec![7, 7, 8], move |_| {
        count.fetch_add(1, Ordering::SeqCst);
        frame()
    });
    let r = tool.execute_result(plan(), &ToolCtx::detached("s"));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(r.is_error && r.output.contains("前序落点数 1"));
    assert_eq!(
        r.structured_content.unwrap()["harness_status"],
        "outcome_unknown"
    );
}

#[test]
fn resized_preflight_frame_stops_before_coordinate_input() {
    let mut tool = path_tool(vec![7], |_| panic!("should not click"));
    tool.observed.lock().unwrap().as_mut().unwrap().pixels.0 = 120;
    tool.state = native("get_app_state", |args| {
        assert_eq!(args["include_screenshot"], true);
        frame()
    });
    let r = tool.execute_result(plan(), &ToolCtx::detached("s"));
    assert!(r.is_error && r.output.contains("前序落点数 0"));
}

#[test]
fn modal_or_unknown_native_result_stops_path() {
    for unknown in [false, true] {
        let calls = Arc::new(AtomicUsize::new(0));
        let count = calls.clone();
        let tool = path_tool(vec![7], move |_| {
            count.fetch_add(1, Ordering::SeqCst);
            if unknown {
                return ToolResult::unknown("transport interrupted");
            }
            let mut r = frame();
            r.output.push_str("\n\t3 sheet Save");
            r
        });
        let r = tool.execute_result(plan(), &ToolCtx::detached("s"));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            r.structured_content.unwrap()["harness_status"],
            "outcome_unknown"
        );
    }
}

#[test]
fn preparation_resolves_live_buttons_uniquely_inside_the_pinned_window() {
    for ambiguous in [false, true] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let seen = calls.clone();
        let mut tool = path_tool(vec![7], move |a| {
            seen.lock().unwrap().push(a);
            frame()
        });
        let default_query = tool.query.clone();
        tool.query = native("query", move |a| {
            if a["role"] == "AXWindow" {
                return default_query.execute_result(a, &ToolCtx::detached("fixture"));
            }
            assert_eq!(a["window_id"], 7);
            assert_eq!(a["exact"], true);
            let mut hits = vec![json!({"index":31,"role":"AXButton"})];
            if ambiguous {
                hits.push(hits[0].clone());
            }
            ToolResult::ok(json!({"window_id":7,"truncated":false,"matches":hits}).to_string())
        });
        let mut p = plan();
        p["prepare_buttons"] = json!(["Pen"]);
        let r = tool.execute_result(p, &ToolCtx::detached("s"));
        let clicks = calls.lock().unwrap();
        if ambiguous {
            assert!(r.is_error);
            assert!(clicks.is_empty());
        } else {
            assert!(!r.is_error, "{}", r.output);
            assert_eq!(clicks.len(), 3);
            assert_eq!(clicks[0]["element_index"], "31");
            assert_eq!(clicks[0]["click_method"], "accessibility");
            assert_eq!(clicks[1]["x"], 1);
        }
    }
}

#[test]
fn freeform_exits_the_old_and_new_paths_in_the_same_approved_action() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let clicks = log.clone();
    let mut tool = path_tool(vec![7], move |a| {
        clicks.lock().unwrap().push(a["click_method"].as_str().unwrap().to_owned());
        frame()
    });
    let keys = log.clone();
    tool.key = Some(native("press_key", move |a| {
        assert_eq!(a["app"], "com.apple.freeform");
        assert_eq!(a["key_method"], "sky_key");
        keys.lock().unwrap().push(a["key"].as_str().unwrap().to_owned());
        frame()
    }));
    let windows = tool.query.clone();
    tool.query = native("query", move |args| {
        if args["role"] == "AXWindow" { return windows.execute_result(args, &ToolCtx::detached("test")); }
        ToolResult::ok(json!({"window_id":7, "truncated":false, "matches":[{"index":3,"role":"AXButton"}]}).to_string())
    });
    let mut p = plan();
    p["freeform_new_stroke"] = json!(true);
    p["prepare_buttons"] = json!(["Draw with Pen"]);
    let ctx = ToolCtx::detached("s");
    assert!(tool.path(&p, &ctx).is_error);
    assert!(log.lock().unwrap().is_empty());
    tool.observed.lock().unwrap().as_mut().unwrap().app = "com.apple.freeform".into();
    p["app"] = json!("com.apple.freeform");
    let result = tool.path(&p, &ctx);
    assert!(!result.is_error, "{}", result.output);
    assert_eq!(*log.lock().unwrap(), ["Escape", "accessibility", "sky_click", "sky_click", "Escape"]);
    assert_eq!(result.parts.len(), 1);
}
