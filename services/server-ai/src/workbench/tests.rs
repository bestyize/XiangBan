use super::*;
use futures_util::StreamExt;
use std::time::Duration;

pub(super) fn model_reply(body: &Value) -> Response {
    let last = body["messages"].as_array().unwrap().last().unwrap();
    let (delta, finish) = if last["role"] == "user" && last["content"].to_string().contains("联网")
    {
        (
            json!({"tool_calls":[{"index":0,"id":"search-1","type":"function","function":{"name":"web_search","arguments":"{\"query\":\"小伴\"}"}}]}),
            "tool_calls",
        )
    } else {
        (json!({"content":"你好"}), "stop")
    };
    let first = json!({"id":"1","choices":[{"index":0,"delta":delta,"finish_reason":null}]});
    let end = json!({"id":"1","choices":[{"index":0,"delta":{},"finish_reason":finish}]});
    Response::builder()
        .header(header::CONTENT_TYPE, "text/event-stream")
        .body(Body::from(format!(
            "data: {first}\n\ndata: {end}\n\ndata: [DONE]\n\n"
        )))
        .unwrap()
}

async fn wait_event(h: &Harness, run: Uuid, kind: &str) -> (i64, Value) {
    tokio::time::timeout(Duration::from_secs(15),async {
        loop {
            let row:Option<(i64,sqlx::types::Json<Value>)>=sqlx::query_as("SELECT seq,payload FROM workbench_events WHERE run_id=$1 AND payload->>'type'=$2 ORDER BY seq LIMIT 1")
                .bind(run).bind(kind).fetch_optional(&h.state.pool).await.unwrap();
            if let Some((seq,value))=row{return (seq,value.0);}
            tokio::time::sleep(Duration::from_millis(35)).await;
        }
    }).await.unwrap_or_else(|_|panic!("missing event {kind} from {run}"))
}

async fn replay(h: &Harness, path: &str, token: &str) -> String {
    let response = h
        .app
        .clone()
        .oneshot(
            Request::get(path)
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let mut stream = response.into_body().into_data_stream();
    let mut bytes = Vec::new();
    tokio::time::timeout(Duration::from_secs(4), async {
        while let Some(data) = stream.next().await {
            bytes.extend_from_slice(&data.unwrap());
            if String::from_utf8_lossy(&bytes).contains("event: ready") {
                break;
            }
        }
    })
    .await
    .unwrap();
    String::from_utf8(bytes).unwrap()
}

#[tokio::test]
async fn web_workbench_replays_detached_runs_and_deduplicates_concurrent_sends() {
    let Some(h) = Harness::new().await else {
        return;
    };
    h.seed_wallpaper().await;
    h.mock.0.lock().unwrap().chat_mode = ChatMode::Workbench;
    let token = h.token(Uuid::new_v4(), CLIENT);
    let other = h.token(Uuid::new_v4(), CLIENT);
    let id = Uuid::new_v4();
    let base = format!("/api/workbench/sessions/{id}");
    let path = format!("{base}/messages");
    assert_eq!(
        h.call(
            Method::POST,
            "/api/workbench/sessions",
            Some(&token),
            json!({"id":id})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        h.call(
            Method::GET,
            &format!("{base}/events"),
            Some(&other),
            json!(null)
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    let body = json!({"text":"第一条","requestId":Uuid::new_v4(),"tier":"standard"});
    let (a, b) = tokio::join!(
        h.call(Method::POST, &path, Some(&token), body.clone()),
        h.call(Method::POST, &path, Some(&token), body.clone())
    );
    assert_eq!(a.0, StatusCode::OK, "{}", a.1);
    assert_eq!(a.1["runId"], b.1["runId"]);
    let run = Uuid::parse_str(a.1["runId"].as_str().unwrap()).unwrap();
    let (cursor, _) = wait_event(&h, run, "user_message").await;
    wait_event(&h, run, "done").await;
    let all = replay(&h, &format!("{base}/events"), &token).await;
    assert_eq!(all.matches("user_message").count(), 1);
    assert!(all.contains("你好") && all.contains("\"type\":\"done\""));
    let after = replay(&h, &format!("{base}/events?after={cursor}"), &token).await;
    assert!(!after.contains("user_message") && after.contains("你好"));
    assert_eq!(h.mock.0.lock().unwrap().chat_requests.len(), 1);
    let mut changed = body;
    changed["text"] = json!("另一个请求");
    assert_eq!(
        h.call(Method::POST, &path, Some(&token), changed).await.0,
        StatusCode::CONFLICT
    );
    let next = h
        .call(
            Method::POST,
            &path,
            Some(&token),
            json!({"text":"第二条","requestId":Uuid::new_v4(),"tier":"standard"}),
        )
        .await;
    wait_event(
        &h,
        Uuid::parse_str(next.1["runId"].as_str().unwrap()).unwrap(),
        "done",
    )
    .await;
    let messages = h.mock.0.lock().unwrap().chat_requests.last().unwrap().1["messages"].to_string();
    assert!(
        messages.contains("第一条") && messages.contains("你好") && messages.contains("第二条")
    );
    assert_eq!(
        h.call(
            Method::GET,
            "/api/workbench/sessions",
            Some(&other),
            json!(null)
        )
        .await
        .1,
        json!([])
    );
    h.cleanup().await;
}

#[tokio::test]
async fn web_workbench_approval_survives_disconnect_and_cancel_targets_one_run() {
    let Some(h) = Harness::new().await else {
        return;
    };
    h.seed_wallpaper().await;
    h.mock.0.lock().unwrap().chat_mode = ChatMode::Workbench;
    let token = h.token(Uuid::new_v4(), CLIENT);
    let other = h.token(Uuid::new_v4(), CLIENT);
    let id = Uuid::new_v4();
    let base = format!("/api/workbench/sessions/{id}");
    h.call(
        Method::POST,
        "/api/workbench/sessions",
        Some(&token),
        json!({"id":id}),
    )
    .await;
    let mut previous_run = None;
    for decision in ["cancel", "deny", "once"] {
        let (status, result) = h
            .call(
                Method::POST,
                &format!("{base}/messages"),
                Some(&token),
                json!({"text":"帮我联网搜索","tier":"standard","requestId":Uuid::new_v4()}),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "{result}");
        let run = Uuid::parse_str(result["runId"].as_str().unwrap()).unwrap();
        let (_, event) = wait_event(&h, run, "approval_requested").await;
        let key = event["request"]["id"].as_str().unwrap();
        let approval = format!("{base}/runs/{run}/approvals/{key}");
        assert!(replay(&h, &format!("{base}/events"), &token)
            .await
            .contains(key));
        assert_eq!(
            h.call(
                Method::POST,
                &approval,
                Some(&other),
                json!({"decision":"once"})
            )
            .await
            .0,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            h.call(
                Method::POST,
                &format!("{base}/messages"),
                Some(&token),
                json!({"text":"不重复运行","tier":"standard","requestId":Uuid::new_v4()})
            )
            .await
            .0,
            StatusCode::CONFLICT
        );
        if let Some(old) = previous_run {
            h.call(
                Method::POST,
                &format!("{base}/runs/{old}/cancel"),
                Some(&token),
                json!(null),
            )
            .await;
        }
        if decision == "cancel" {
            assert_eq!(
                h.call(
                    Method::POST,
                    &format!("{base}/runs/{run}/cancel"),
                    Some(&token),
                    json!(null)
                )
                .await
                .0,
                StatusCode::OK
            );
            wait_event(&h, run, "cancelled").await;
            assert_eq!(
                h.call(
                    Method::POST,
                    &approval,
                    Some(&token),
                    json!({"decision":"once"})
                )
                .await
                .0,
                StatusCode::CONFLICT
            );
        } else {
            for _ in 0..2 {
                assert_eq!(
                    h.call(
                        Method::POST,
                        &approval,
                        Some(&token),
                        json!({"decision":decision})
                    )
                    .await
                    .0,
                    StatusCode::OK
                );
            }
            wait_event(&h, run, "done").await;
        }
        let searches = h
            .mock
            .0
            .lock()
            .unwrap()
            .relayed
            .iter()
            .filter(|r| r.path == "/v1/tools/search")
            .count();
        assert_eq!(searches, usize::from(decision == "once"));
        previous_run = Some(run);
    }
    h.cleanup().await;
}

#[tokio::test]
async fn web_workbench_crashed_worker_closes_partial_turn_once_and_preserves_product_boundaries() {
    let Some(h) = Harness::new().await else {
        return;
    };
    h.seed_wallpaper().await;
    let admin = h.admin_token().await;
    h.call(Method::PUT,"/api/admin/products/forum",Some(&admin),json!({"name":"论坛","client_ids":[WEB_CLIENT],"trial_quota":0,"trial_days":7,"enabled":true})).await;
    let uid = Uuid::new_v4();
    let token = h.token(uid, CLIENT);
    let other_product = h.token(uid, WEB_CLIENT);
    let id = Uuid::new_v4();
    let run = Uuid::new_v4();
    let base = format!("/api/workbench/sessions/{id}");
    assert_eq!(
        h.call(
            Method::POST,
            "/api/workbench/sessions",
            Some(&token),
            json!({"id":id})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        h.call(
            Method::GET,
            &format!("{base}/events"),
            Some(&other_product),
            json!(null)
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    sqlx::query("INSERT INTO workbench_runs(id,session_id,request_id,input,tier,status,heartbeat) VALUES($1,$2,$3,'hi','standard','running',now()-interval '2 minutes')")
        .bind(run).bind(id).bind(Uuid::new_v4()).execute(&h.state.pool).await.unwrap();
    sqlx::query("INSERT INTO workbench_events(session_id,run_id,payload) VALUES($1,$2,$3)")
        .bind(id)
        .bind(run)
        .bind(sqlx::types::Json(
            json!({"type":"text_delta","text":"已收到的片段"}),
        ))
        .execute(&h.state.pool)
        .await
        .unwrap();
    for _ in 0..2 {
        crate::workbench::reap(&h.state.pool).await.unwrap();
    }
    let stream = replay(&h, &format!("{base}/events"), &token).await;
    assert!(stream.contains("已收到的片段"));
    assert_eq!(stream.matches("\"type\":\"error\"").count(), 1);
    assert!(h.mock.0.lock().unwrap().chat_requests.is_empty());
    h.cleanup().await;
}
