use super::{lock_run, record, settle};
use crate::{
    error::AppError, extract::AuthUser, handlers::user::is_responses_only_model, proxy, search,
    state::AppState,
};
use agent_kernel::{
    provider::{openai::OpenAiCompat, responses::OpenAiResponses, wire::ProviderProfile},
    Agent, Approval, ApprovalRequest, Event, HarnessError, MemoryStore, Message, ModelProvider,
    Permission, PermissionMode, Risk, SessionStore, Tool, ToolCtx, ToolSpec,
};
use serde_json::{json, Value};
use sqlx::{types::Json, PgPool};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::runtime::Handle;
use uuid::Uuid;

#[derive(Clone)]
struct Job {
    pool: PgPool,
    session: Uuid,
    run: Uuid,
    rt: Handle,
}

impl Job {
    // A provider may deliver streamed fragments from inside its own Tokio runtime.
    // The synchronous kernel waits for the service runtime to commit each structural event.
    fn call<T: Send + 'static>(
        &self,
        operation: impl std::future::Future<Output = Result<T, AppError>> + Send + 'static,
    ) -> Result<T, AppError> {
        let (send, recv) = std::sync::mpsc::sync_channel(1);
        self.rt.spawn(async move {
            let _ = send.send(operation.await);
        });
        recv.recv()
            .map_err(|_| AppError::Internal("workbench database task stopped".into()))?
    }

    async fn append(&self, event: Value) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;
        if lock_run(&mut tx, self.session, self.run).await? != "running" {
            return Err(AppError::Conflict("回答已停止".into()));
        }
        record(&mut tx, self.session, self.run, event).await?;
        tx.commit().await?;
        Ok(())
    }
    async fn finish(&self, result: Result<Value, AppError>) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;
        if lock_run(&mut tx, self.session, self.run).await? == "running" {
            let (status, event, history) = match result {
                Ok(history) => ("done", json!({"type":"done"}), history),
                Err(err) => (
                    "error",
                    json!({"type":"error","message":err.parts().2}),
                    json!([]),
                ),
            };
            settle(&mut tx, self.session, self.run, status, event, history).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn ask(&self, id: Uuid, request: Value) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;
        if lock_run(&mut tx, self.session, self.run).await? != "running" {
            return Err(AppError::Conflict("回答已停止".into()));
        }
        sqlx::query("INSERT INTO workbench_approvals(id,run_id) VALUES($1,$2)")
            .bind(id)
            .bind(self.run)
            .execute(&mut *tx)
            .await?;
        record(&mut tx, self.session, self.run, request).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn wait_for_decision(&self, id: Uuid) -> Result<Approval, AppError> {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(600);
        loop {
            let row: Option<(Option<String>, String)> = sqlx::query_as(
                "SELECT a.decision,r.status FROM workbench_approvals a
                 JOIN workbench_runs r ON a.run_id=r.id WHERE a.id=$1",
            )
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
            match row {
                Some((Some(choice), status)) if status == "running" => {
                    return Ok(if choice == "once" {
                        Approval::Once
                    } else {
                        Approval::Deny
                    });
                }
                Some((None, status)) if status == "running" => (),
                _ => return Ok(Approval::Deny),
            }
            if tokio::time::Instant::now() >= deadline {
                let mut tx = self.pool.begin().await?;
                if lock_run(&mut tx, self.session, self.run).await? != "running" {
                    return Ok(Approval::Deny);
                }
                let expired = sqlx::query("UPDATE workbench_approvals SET decision='deny' WHERE id=$1 AND decision IS NULL")
                    .bind(id).execute(&mut *tx).await?.rows_affected() == 1;
                if expired {
                    record(
                        &mut tx,
                        self.session,
                        self.run,
                        json!({"type":"approval_resolved","id":id}),
                    )
                    .await?;
                }
                tx.commit().await?;
                if expired {
                    return Ok(Approval::Deny);
                }
                continue; // A click won the race with the deadline. Read the committed decision.
            }
            tokio::time::sleep(Duration::from_millis(350)).await;
        }
    }

    fn approval(&self, request: &ApprovalRequest<'_>) -> Approval {
        let id = Uuid::new_v4();
        let risk = match request.risk {
            Risk::Read => "read",
            Risk::Write => "write",
            Risk::Network => "network",
            Risk::Exec => "exec",
        };
        let event = json!({"type":"approval_requested","request":{
            "id": id, "session": self.session, "tool": request.tool, "risk": risk,
            "arguments": request.arguments, "tainted": request.tainted, "canRemember": false
        }});
        let job = self.clone();
        if self.call(async move { job.ask(id, event).await }).is_err() {
            return Approval::Deny;
        }
        let (send, receive) = std::sync::mpsc::channel();
        let job = self.clone();
        self.rt.spawn(async move {
            let decision = job.wait_for_decision(id).await.unwrap_or(Approval::Deny);
            let _ = send.send(decision);
        });
        request
            .wait_for_approval(&receive)
            .unwrap_or(Approval::Deny)
    }
}

struct WebSearch {
    state: AppState,
    auth: AuthUser,
    job: Job,
    spec: ToolSpec,
}
impl Tool for WebSearch {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }
    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        if ctx.is_cancelled() {
            return Err("回答已停止".into());
        }
        let state = self.state.clone();
        let auth = self.auth.clone();
        let run = self.job.run;
        self.job
            .call(async move {
                let active: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM workbench_runs WHERE id=$1 AND status='running')",
                )
                .bind(run)
                .fetch_one(&state.pool)
                .await?;
                if !active {
                    return Err(AppError::Conflict("回答已停止".into()));
                }
                let request = search::parse_search(args.to_string().as_bytes())?;
                Ok(proxy::search_for_user(&state, auth, &request)
                    .await?
                    .to_string())
            })
            .map_err(|e| e.parts().2)
    }
}

pub(super) async fn execute(
    state: AppState,
    auth: AuthUser,
    session: Uuid,
    run: Uuid,
    input: String,
    tier: String,
) {
    let job = Job {
        pool: state.pool.clone(),
        session,
        run,
        rt: Handle::current(),
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let monitor = tokio::spawn({
        let pool = job.pool.clone();
        let cancel = cancel.clone();
        async move {
            loop {
                let result = sqlx::query(
                    "UPDATE workbench_runs SET heartbeat=now() WHERE id=$1 AND status='running'",
                )
                .bind(run)
                .execute(&pool)
                .await;
                if !matches!(result, Ok(row) if row.rows_affected()==1) {
                    cancel.store(true, Ordering::Release);
                    break;
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    });
    let result = run_agent(&job, state, auth, input, tier, cancel).await;
    if let Err(err) = job.finish(result).await {
        tracing::warn!("workbench finish {run}: {err}");
    }
    monitor.abort();
}

async fn run_agent(
    job: &Job,
    state: AppState,
    auth: AuthUser,
    input: String,
    tier_name: String,
    cancel: Arc<AtomicBool>,
) -> Result<Value, AppError> {
    let (tier, binding) = proxy::model_access(&state, &auth, Some(&tier_name)).await?;
    let prior = sqlx::query_as::<_, PreviousTurn>(
        "SELECT r.input,r.history,CASE WHEN r.history='[]'::jsonb THEN
         (SELECT string_agg(e.payload->>'text','' ORDER BY e.seq) FROM workbench_events e
          WHERE e.run_id=r.id AND e.payload->>'type'='text_delta') END AS partial
         FROM workbench_runs r WHERE r.session_id=$1 AND r.id<>$2 ORDER BY r.created_at,r.id",
    )
    .bind(job.session)
    .bind(job.run)
    .fetch_all(&job.pool)
    .await?;
    let mut profile = ProviderProfile::new(
        format!("{}/v1", state.newapi.base_url().trim_end_matches('/')),
        format!("sk-{}", binding.key),
        &tier.model,
    );
    // An approval can leave a turn suspended while a user's balance or tier configuration changes.
    // New API enforces the user's total wallet. Recheck the product's paid-tier rules per model call.
    let service = state.clone();
    let owner = auth.clone();
    let access_job = job.clone();
    let model = tier.model.clone();
    let first_key = std::sync::Mutex::new(Some(std::mem::take(&mut profile.api_key)));
    profile = profile.with_checked_api_key_source(Arc::new(move || {
        if let Some(key) = first_key.lock().unwrap_or_else(|e| e.into_inner()).take() {
            return Ok(key);
        }
        let (service, owner, tier_name, model) = (
            service.clone(),
            owner.clone(),
            tier_name.clone(),
            model.clone(),
        );
        access_job
            .call(async move {
                let (current, binding) =
                    proxy::model_access(&service, &owner, Some(&tier_name)).await?;
                if current.model != model {
                    return Err(AppError::Conflict("模型档位已更新，请重新发送。".into()));
                }
                Ok(format!("sk-{}", binding.key))
            })
            .map_err(|_| HarnessError::Config("workbench_access_changed".into()))
    }));
    profile.proxy_mode = "direct".into();
    let caps = tier.capabilities.as_deref();
    profile.supports_tools = caps.and_then(|c| c.supports_tools).unwrap_or(true);
    profile.max_output_tokens = caps.and_then(|c| c.max_output_tokens);
    profile.max_request_bytes = Some(8 * 1024 * 1024);
    let use_responses = is_responses_only_model(&tier.model);
    let search = if profile.supports_tools && state.search.available() {
        let spec = ToolSpec::new("web_search", "搜索公开网页", json!({
            "type":"object", "properties":{"query":{"type":"string","minLength":1,"maxLength":search::MAX_QUERY_CHARS}},
            "required":["query"], "additionalProperties":false
        })).risk(Risk::Network).always_ask().untrusted_output();
        Some(WebSearch {
            state,
            auth,
            job: job.clone(),
            spec,
        })
    } else {
        None
    };
    let job = job.clone();
    tokio::task::spawn_blocking(move || {
        let provider: Arc<dyn ModelProvider> = if use_responses {
            Arc::new(OpenAiResponses::new(profile).map_err(model_error)?)
        } else {
            Arc::new(OpenAiCompat::new(profile).map_err(model_error)?)
        };
        let session = job.session.to_string();
        let history = Arc::new(MemoryStore::new());
        let mut before = 0;
        for turn in prior {
            let mut messages = turn.history.0;
            if messages.is_empty() {
                messages.push(Message::user_text(turn.input));
                if let Some(text) = turn.partial { messages.push(Message::assistant(text,vec![])); }
            }
            before += messages.len();
            history.append(&session,&messages).map_err(model_error)?;
        }
        let mut builder = Agent::builder(provider,history.clone()).max_turns(8)
            .system_prompt("你是小伴，正在网页版工作台与用户对话。使用当前会话提供的工具。用户文件和桌面只能在用户设备上访问。");
        if let Some(tool) = search {
            let approver = job.clone();
            builder = builder.tool(tool).middleware(Permission::new(PermissionMode::Ask,
                move |request: &ApprovalRequest<'_>| approver.approval(request)));
        }
        let agent = builder.build().map_err(model_error)?;
        let mut sink = EventWriter {job:&job,buffer:None,last:None};
        let mut failed = false;
        let result = agent.run(&session,&input,&mut |event| {
            if !failed && sink.push(event).is_err() {
                failed = true;
                cancel.store(true,Ordering::Release);
            }
        }, &cancel);
        sink.flush()?;
        if failed { return Err(AppError::ServiceUnavailable("保存回答时连接中断".into())); }
        result.map_err(model_error)?;
        let messages = history.load(&session).map_err(model_error)?;
        Ok(json!(&messages[before..]))
    }).await.map_err(|_| AppError::Internal("workbench worker stopped".into()))?
}

#[derive(sqlx::FromRow)]
struct PreviousTurn {
    input: String,
    history: Json<Vec<Message>>,
    partial: Option<String>,
}

fn model_error(error: HarnessError) -> AppError {
    // Provider error strings can contain a real model identifier or upstream connection details.
    let message = match error {
        HarnessError::Config(ref code) if code == "workbench_access_changed" => {
            "模型档位或可用额度已变化，请检查账号后重试。"
        }
        HarnessError::RequestTooLarge { .. } => "对话超过模型请求上限，请缩短内容或新建对话。",
        HarnessError::MaxTurns(_) | HarnessError::BudgetExceeded(_) => {
            "本次任务已达到执行上限，可以继续提问。"
        }
        HarnessError::Cancelled => "本次回答已停止。",
        _ => "模型未能完成回答，请稍后重试。",
    };
    AppError::BadGateway(message.into())
}

/// Batch adjacent token fragments. Structural events flush the buffer so replay order is stable.
struct EventWriter<'a> {
    job: &'a Job,
    buffer: Option<Event>,
    last: Option<Instant>,
}
impl EventWriter<'_> {
    fn flush(&mut self) -> Result<(), AppError> {
        if let Some(event) = self.buffer.take() {
            let job = self.job.clone();
            self.job
                .call(async move { job.append(json!(event)).await })?;
            self.last = Some(Instant::now());
        }
        Ok(())
    }
    fn push(&mut self, event: Event) -> Result<(), AppError> {
        let event = match event {
            Event::Done
            | Event::Cancelled
            | Event::Error { .. }
            | Event::UserMessage { initial: true, .. } => return self.flush(),
            Event::Retrying {
                attempt, delay_ms, ..
            } => Event::Retrying {
                attempt,
                delay_ms,
                message: "正在重新连接模型".into(),
            },
            other => other,
        };
        match (&mut self.buffer, &event) {
            (Some(Event::TextDelta { text }), Event::TextDelta { text: next }) => {
                text.push_str(next)
            }
            (
                Some(Event::ReasoningDelta { kind, text }),
                Event::ReasoningDelta {
                    kind: k,
                    text: next,
                },
            ) if k == kind => text.push_str(next),
            _ => {
                self.flush()?;
                self.buffer = Some(event);
            }
        }
        if !matches!(
            self.buffer,
            Some(Event::TextDelta { .. } | Event::ReasoningDelta { .. })
        ) || self
            .last
            .is_none_or(|t| t.elapsed() >= Duration::from_millis(80))
        {
            self.flush()?;
        }
        Ok(())
    }
}
