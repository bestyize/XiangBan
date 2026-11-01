//! Internal scheduled-job accounting against the existing product quota buckets.
//! Off by default. No prices, product IDs, model IDs, or entitlements are seeded.
use crate::{
    error::AppError,
    ledger::{allocate, BucketView},
    quota,
    state::AppState,
};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

pub async fn init(pool: &PgPool) -> Result<(), AppError> {
    sqlx::raw_sql("CREATE TABLE IF NOT EXISTS information_job_reservations (edition_id UUID PRIMARY KEY, subscription_id UUID NOT NULL, binding_id BIGINT NOT NULL REFERENCES bindings(id), status TEXT NOT NULL, reserved BIGINT NOT NULL, settled BIGINT, policy JSONB NOT NULL, usage JSONB, created_at TIMESTAMPTZ NOT NULL DEFAULT now()); CREATE TABLE IF NOT EXISTS information_job_allocations (edition_id UUID NOT NULL REFERENCES information_job_reservations(edition_id),bucket_id BIGINT NOT NULL REFERENCES quota_buckets(id),amount BIGINT NOT NULL,PRIMARY KEY(edition_id,bucket_id));").execute(pool).await?;
    sqlx::raw_sql("CREATE TABLE IF NOT EXISTS information_job_cancellations (edition_id UUID PRIMARY KEY, owner UUID NOT NULL, subscription_id UUID NOT NULL);").execute(pool).await?;
    sqlx::raw_sql("ALTER TABLE information_job_reservations ADD COLUMN IF NOT EXISTS budget_month DATE; ALTER TABLE information_job_reservations ADD COLUMN IF NOT EXISTS budget_per_run BIGINT; ALTER TABLE information_job_reservations ADD COLUMN IF NOT EXISTS budget_per_month BIGINT;").execute(pool).await?;
    Ok(())
}
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct Policy {
    product_id: String,
    reserve_units: i64,
    model: String,
    input_per_million: i64,
    output_per_million: i64,
    search_units: i64,
    email_units: i64,
    #[serde(default)]
    execution: Option<ExecutionConfig>,
}
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct ExecutionConfig {
    base_url: String,
    protocol: String,
    credential_env: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    edition_id: Uuid,
    subscription_id: Uuid,
    owner: Uuid,
    #[serde(default)]
    budget_per_run: Option<i64>,
    #[serde(default)]
    budget_per_month: Option<i64>,
    #[serde(default)]
    usage: Vec<Value>,
}
fn authorize(headers: &HeaderMap) -> Result<(), AppError> {
    let key = std::env::var("AI_INFORMATION_SERVICE_TOKEN")
        .ok()
        .filter(|s| s.len() >= 32)
        .ok_or_else(|| AppError::ServiceUnavailable("后台任务额度尚未开放".into()))?;
    let given = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .unwrap_or("");
    let signing = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, key.as_bytes());
    let tag = ring::hmac::sign(&signing, b"information-jobs");
    let candidate = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, given.as_bytes());
    if ring::hmac::verify(&candidate, b"information-jobs", tag.as_ref()).is_err() {
        return Err(AppError::Unauthorized("服务凭证无效".into()));
    }
    Ok(())
}
fn current_policy() -> Result<Policy, AppError> {
    let mut policy: Policy =
        serde_json::from_str(&std::env::var("AI_INFORMATION_POLICY").unwrap_or_default())
            .map_err(|_| AppError::ServiceUnavailable("后台任务价格策略尚未配置".into()))?;
    if policy.reserve_units <= 0
        || policy.product_id.is_empty()
        || policy.model.is_empty()
        || [
            policy.input_per_million,
            policy.output_per_million,
            policy.search_units,
            policy.email_units,
        ]
        .iter()
        .any(|v| *v < 0 || *v > 1_000_000_000_000)
    {
        return Err(AppError::ServiceUnavailable("后台任务价格策略无效".into()));
    }
    if policy.execution.is_none() {
        policy.execution = Some(ExecutionConfig {
            base_url: std::env::var("AI_INFORMATION_MODEL_BASE_URL")
                .map_err(|_| AppError::ServiceUnavailable("后台模型未配置".into()))?,
            protocol: std::env::var("AI_INFORMATION_MODEL_PROTOCOL")
                .unwrap_or_else(|_| "chat_completions".into()),
            credential_env: "AI_INFORMATION_MODEL_KEY".into(),
        });
    }
    Ok(policy)
}
fn cost(policy: &Policy, usage: &[Value]) -> Result<i64, AppError> {
    let mut units: i128 = 0;
    for item in usage {
        let stage = item.get("stage").and_then(Value::as_str).unwrap_or("");
        let value = &item["usage"];
        match stage {
            "platform_email" if value["accepted"] == true => {
                units += i128::from(policy.email_units)
            }
            "platform_search" => {
                units += i128::from(value["requests"].as_u64().unwrap_or(0))
                    * i128::from(policy.search_units)
            }
            "platform_model" => {
                if value["model"].as_str() != Some(&policy.model) {
                    return Err(AppError::BadRequest("计量模型与价格策略不匹配".into()));
                }
                let u = &value["provider_usage"];
                let input = u["prompt_tokens"]
                    .as_u64()
                    .or_else(|| u["input_tokens"].as_u64())
                    .ok_or_else(|| AppError::BadRequest("缺少模型计量".into()))?;
                let output = u["completion_tokens"]
                    .as_u64()
                    .or_else(|| u["output_tokens"].as_u64())
                    .ok_or_else(|| AppError::BadRequest("缺少模型计量".into()))?;
                // Cached / reasoning token accounting needs a policy-specific adapter before enabling those modes.
                if u.get("cache_read_input_tokens")
                    .and_then(Value::as_u64)
                    .unwrap_or(0)
                    > 0
                    || u.get("cache_creation_input_tokens")
                        .and_then(Value::as_u64)
                        .unwrap_or(0)
                        > 0
                {
                    return Err(AppError::BadRequest("当前策略未配置缓存计量".into()));
                }
                units += (i128::from(input) * i128::from(policy.input_per_million)
                    + i128::from(output) * i128::from(policy.output_per_million)
                    + 999_999)
                    / 1_000_000;
            }
            _ => {}
        }
    }
    i64::try_from(units).map_err(|_| AppError::BadRequest("计量溢出".into()))
}
#[derive(FromRow)]
struct Reservation {
    subscription_id: Uuid,
    binding_id: i64,
    status: String,
    reserved: i64,
    policy: Value,
}
pub async fn action(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(action): Path<String>,
    Json(input): Json<Request>,
) -> Result<Json<Value>, AppError> {
    authorize(&headers)?;
    if !["reserve", "settle", "release"].contains(&action.as_str()) {
        return Err(AppError::NotFound("操作不存在".into()));
    }
    let policy = if action == "reserve" {
        current_policy().ok()
    } else {
        None
    };
    transact(&s.pool, policy.as_ref(), &action, &input).await?;
    Ok(Json(json!({"ok":true})))
}
async fn transact(
    pool: &PgPool,
    policy: Option<&Policy>,
    action: &str,
    input: &Request,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    // Serialize this edition even if the configured product changes between requests.
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("information-edition:{}", input.edition_id))
        .execute(&mut *tx)
        .await?;
    // All editions of this subscription share the same monthly exposure ceiling.
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("information-budget:{}", input.subscription_id))
        .execute(&mut *tx)
        .await?;
    let cancelled: Option<(Uuid, Uuid)> = sqlx::query_as(
        "SELECT owner,subscription_id FROM information_job_cancellations WHERE edition_id=$1",
    )
    .bind(input.edition_id)
    .fetch_optional(&mut *tx)
    .await?;
    if let Some((owner, sub)) = cancelled {
        if owner != input.owner || sub != input.subscription_id {
            return Err(AppError::Conflict("期刊归属不一致".into()));
        }
        return if action == "release" {
            Ok(())
        } else {
            Err(AppError::Conflict("期刊预约已释放".into()))
        };
    }
    let existing:Option<(i64,Uuid)>=sqlx::query_as("SELECT r.binding_id,b.uid FROM information_job_reservations r JOIN bindings b ON b.id=r.binding_id WHERE r.edition_id=$1")
        .bind(input.edition_id).fetch_optional(&mut *tx).await?;
    let binding = if let Some((binding, owner)) = existing {
        if owner != input.owner {
            return Err(AppError::Conflict("期刊归属不一致".into()));
        }
        binding
    } else {
        if action == "release" {
            sqlx::query("INSERT INTO information_job_cancellations VALUES($1,$2,$3)")
                .bind(input.edition_id)
                .bind(input.owner)
                .bind(input.subscription_id)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            return Ok(());
        }
        if action != "reserve" {
            return Err(AppError::Conflict("没有有效预约".into()));
        }
        let policy = policy
            .ok_or_else(|| AppError::ServiceUnavailable("后台任务价格策略尚未配置".into()))?;
        let binding:Option<(i64,)>=sqlx::query_as("SELECT b.id FROM bindings b JOIN products p ON p.product_id=b.product_id WHERE b.uid=$1 AND b.product_id=$2 AND b.status='active' AND p.enabled").bind(input.owner).bind(&policy.product_id).fetch_optional(&mut *tx).await?;
        binding
            .ok_or_else(|| AppError::Coded {
                status: StatusCode::PAYMENT_REQUIRED,
                code: "insufficient_quota",
                message: "请先开通小伴额度".into(),
            })?
            .0
    };
    // Existing settlements/releases use the original binding even after disable or policy removal.
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(binding)
        .execute(&mut *tx)
        .await?;
    let old:Option<Reservation>=sqlx::query_as("SELECT subscription_id,binding_id,status,reserved,policy FROM information_job_reservations WHERE edition_id=$1 FOR UPDATE").bind(input.edition_id).fetch_optional(&mut *tx).await?;
    if let Some(ref old) = old {
        if old.binding_id != binding || old.subscription_id != input.subscription_id {
            return Err(AppError::Conflict("期刊归属不一致".into()));
        }
        if old.status == "settled" {
            return if action == "release" {
                Err(AppError::Conflict("期刊已结算".into()))
            } else {
                Ok(())
            };
        }
        if old.status == "released" {
            return if action == "release" {
                Ok(())
            } else {
                Err(AppError::Conflict("期刊预约已释放".into()))
            };
        }
    }
    if action == "reserve" {
        if old.is_some() {
            return Ok(());
        }
        let policy = policy
            .ok_or_else(|| AppError::ServiceUnavailable("后台任务价格策略尚未配置".into()))?;
        let (run_limit, month_limit) = match (input.budget_per_run, input.budget_per_month) {
            (Some(run), Some(month)) if run > 0 && run <= month && month <= 1_000_000_000_000 => {
                (run, month)
            }
            _ => {
                return Err(AppError::BadRequest(
                    "请先设置有效的单次和每月额度预算".into(),
                ))
            }
        };
        let reserve = policy.reserve_units.min(run_limit);
        let used:i64=sqlx::query_scalar("SELECT COALESCE(sum(CASE WHEN r.status='reserved' THEN r.reserved WHEN r.status='settled' THEN r.settled ELSE 0 END),0)::bigint FROM information_job_reservations r JOIN bindings b ON b.id=r.binding_id WHERE r.subscription_id=$1 AND b.uid=$2 AND COALESCE(r.budget_month,date_trunc('month',r.created_at AT TIME ZONE 'UTC')::date)=date_trunc('month',now() AT TIME ZONE 'UTC')::date")
            .bind(input.subscription_id).bind(input.owner).fetch_one(&mut *tx).await?;
        if used > month_limit - reserve {
            return Err(AppError::Coded {
                status: StatusCode::PAYMENT_REQUIRED,
                code: "subscription_budget_exceeded",
                message: "本月订阅预算不足，包含已用及未结算预约".into(),
            });
        }
        let live:Vec<(i64,i64,Option<DateTime<Utc>>)>=sqlx::query_as(&format!("SELECT id,remaining,expires_at FROM quota_buckets WHERE binding_id=$1 AND {} FOR UPDATE",quota::LIVE)).bind(binding).fetch_all(&mut *tx).await?;
        let buckets: Vec<_> = live
            .into_iter()
            .map(|(id, remaining, expires_at)| BucketView {
                id,
                remaining,
                expires_at,
            })
            .collect();
        let (takes, short) = allocate(reserve, &buckets);
        if short > 0 {
            return Err(AppError::Coded {
                status: StatusCode::PAYMENT_REQUIRED,
                code: "insufficient_quota",
                message: "小伴额度不足".into(),
            });
        }
        sqlx::query("INSERT INTO information_job_reservations(edition_id,subscription_id,binding_id,status,reserved,policy,budget_month,budget_per_run,budget_per_month) VALUES($1,$2,$3,'reserved',$4,$5,date_trunc('month',now() AT TIME ZONE 'UTC')::date,$6,$7)").bind(input.edition_id).bind(input.subscription_id).bind(binding).bind(reserve).bind(json!(policy)).bind(run_limit).bind(month_limit).execute(&mut *tx).await?;
        for (bucket, amount) in takes {
            sqlx::query("UPDATE quota_buckets SET remaining=remaining-$2 WHERE id=$1")
                .bind(bucket)
                .bind(amount)
                .execute(&mut *tx)
                .await?;
            sqlx::query("INSERT INTO information_job_allocations VALUES($1,$2,$3)")
                .bind(input.edition_id)
                .bind(bucket)
                .bind(amount)
                .execute(&mut *tx)
                .await?;
            quota::add_ledger(
                &mut tx,
                binding,
                Some(bucket),
                "job_reserve",
                -amount,
                &input.edition_id.to_string(),
            )
            .await?;
        }
    } else if let Some(old) = old {
        let frozen: Policy = serde_json::from_value(old.policy)
            .map_err(|_| AppError::Internal("invalid reservation policy".into()))?;
        let used = if action == "release" {
            0
        } else {
            cost(&frozen, &input.usage)?
        };
        if used > old.reserved {
            return Err(AppError::Conflict(
                "实际用量超过预约上限，待核实；不会额外扣款".into(),
            ));
        }
        let mut refund = old.reserved - used;
        let allocations:Vec<(i64,i64)>=sqlx::query_as("SELECT bucket_id,amount FROM information_job_allocations WHERE edition_id=$1 ORDER BY bucket_id DESC").bind(input.edition_id).fetch_all(&mut *tx).await?;
        for (bucket, amount) in allocations {
            let give = refund.min(amount);
            refund -= give;
            if give == 0 {
                continue;
            }
            // Preserve original expiration and administrator revocation; never resurrect expired credit.
            let changed=sqlx::query("UPDATE quota_buckets SET remaining=remaining+$2,closed_at=NULL,closed_reason=NULL WHERE id=$1 AND (closed_at IS NULL OR closed_reason='spent') AND (expires_at IS NULL OR expires_at>now())").bind(bucket).bind(give).execute(&mut *tx).await?;
            quota::add_ledger(
                &mut tx,
                binding,
                Some(bucket),
                "job_release",
                if changed.rows_affected() > 0 { give } else { 0 },
                &input.edition_id.to_string(),
            )
            .await?;
        }
        quota::add_ledger(
            &mut tx,
            binding,
            None,
            if action == "release" {
                "job_cancel"
            } else {
                "job_settle"
            },
            0,
            &input.edition_id.to_string(),
        )
        .await?;
        sqlx::query("UPDATE information_job_reservations SET status=$2,settled=$3,usage=$4 WHERE edition_id=$1").bind(input.edition_id).bind(if action=="release"{"released"}else{"settled"}).bind(used).bind(json!(input.usage)).execute(&mut *tx).await?;
    } else {
        return Err(AppError::Conflict("没有有效预约".into()));
    }
    sqlx::query("UPDATE bindings SET change_seq=change_seq+1 WHERE id=$1")
        .bind(binding)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn provider_rates_and_missing_usage() {
        let p = Policy {
            product_id: "test".into(),
            model: "configured-test-model".into(),
            reserve_units: 100,
            input_per_million: 2_000_000,
            output_per_million: 3_000_000,
            search_units: 4,
            email_units: 5,
            execution: None,
        };
        let usage = vec![
            json!({"stage":"platform_model","usage":{"model":p.model,"provider_usage":{"prompt_tokens":2,"completion_tokens":3}}}),
            json!({"stage":"platform_search","usage":{"requests":1}}),
            json!({"stage":"platform_email","usage":{"accepted":true}}),
        ];
        assert_eq!(cost(&p, &usage).unwrap(), 22);
        assert!(cost(&p, &[json!({"stage":"platform_model","usage":{}})]).is_err());
    }
}

/// Platform execution is separate from quota transactions and never provisions a user's upstream key.
pub async fn execute(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(action): Path<String>,
    Json(input): Json<ExecutionRequest>,
) -> Result<Json<Value>, AppError> {
    authorize(&headers)?;
    let policy = execution_policy(&s.pool, &input).await?;
    let mut body = input.payload;
    if action == "search" {
        let request = crate::search::parse_search(
            &serde_json::to_vec(&body)
                .map_err(|_| AppError::BadRequest("invalid search".into()))?,
        )?;
        let key = std::env::var("AI_INFORMATION_SEARCH_GATEWAY_KEY").ok();
        let (hits, _) = s
            .search
            .search(
                &request,
                key.as_deref().map(|k| k.strip_prefix("sk-").unwrap_or(k)),
            )
            .await?;
        return Ok(Json(json!({"results":hits})));
    }
    if action != "summary" {
        return Err(AppError::NotFound("操作不存在".into()));
    }
    let execution = policy.execution.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailable("此期刊缺少已冻结的执行配置，请取消后重新创建".into())
    })?;
    let api = match execution.protocol.as_str() {
        "chat_completions" => crate::proxy::Api::ChatCompletions,
        "messages" => crate::proxy::Api::Messages,
        _ => return Err(AppError::ServiceUnavailable("不支持的后台模型协议".into())),
    };
    let base = &execution.base_url;
    let key = std::env::var(&execution.credential_env)
        .map_err(|_| AppError::ServiceUnavailable("原执行配置的凭据不可用".into()))?;
    prepare_model_request(api, &policy.model, &mut body)?;
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(50))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| AppError::Internal("http client".into()))?;
    let mut request = http.post(format!("{}{}", base.trim_end_matches('/'), api.path()));
    request = if api == crate::proxy::Api::Messages {
        request
            .header("x-api-key", key)
            .header("anthropic-version", "2023-06-01")
    } else {
        request.bearer_auth(key)
    };
    let mut response = request
        .json(&body)
        .send()
        .await
        .map_err(|_| AppError::BadGateway("后台模型暂不可用".into()))?;
    if !response.status().is_success() {
        return Err(AppError::BadGateway("后台模型暂不可用".into()));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| AppError::BadGateway("模型响应中断".into()))?
    {
        if bytes.len() + chunk.len() > 256 * 1024 {
            return Err(AppError::BadGateway("模型响应超限".into()));
        }
        bytes.extend_from_slice(&chunk);
    }
    let data: Value =
        serde_json::from_slice(&bytes).map_err(|_| AppError::BadGateway("模型响应无效".into()))?;
    Ok(Json(normalize_model_response(api, &policy.model, &data)?))
}

fn prepare_model_request(
    api: crate::proxy::Api,
    model: &str,
    body: &mut Value,
) -> Result<(), AppError> {
    let object = body
        .as_object_mut()
        .ok_or_else(|| AppError::BadRequest("模型请求无效".into()))?;
    object.insert("model".into(), json!(model));
    object.insert("stream".into(), json!(false));
    object.insert("max_tokens".into(), json!(2000));
    if api == crate::proxy::Api::Messages {
        object.remove("response_format");
        let messages = object
            .get("messages")
            .and_then(Value::as_array)
            .ok_or_else(|| AppError::BadRequest("缺少消息".into()))?;
        let system = messages
            .iter()
            .find(|m| m["role"] == "system")
            .and_then(|m| m["content"].as_str())
            .unwrap_or("")
            .to_string();
        let messages: Vec<_> = messages
            .iter()
            .filter(|m| m["role"] != "system")
            .cloned()
            .collect();
        object.insert("system".into(), json!(system));
        object.insert("messages".into(), json!(messages));
    }
    Ok(())
}
fn normalize_model_response(
    api: crate::proxy::Api,
    model: &str,
    data: &Value,
) -> Result<Value, AppError> {
    let content = if api == crate::proxy::Api::Messages {
        data["content"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|b| b["type"] == "text")
            .filter_map(|b| b["text"].as_str())
            .collect::<Vec<_>>()
            .join("")
    } else {
        data.pointer("/choices/0/message/content")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    };
    if content.is_empty() {
        return Err(AppError::BadGateway("模型未返回摘要".into()));
    }
    Ok(json!({"choices":[{"message":{"content":content}}],"model":model,"usage":data.get("usage")}))
}

#[cfg(test)]
mod database_tests {
    use super::*;
    #[tokio::test]
    async fn common_buckets_concurrent_reservation_settlement_and_release() {
        let Ok(url) = std::env::var("TEST_DATABASE_URL") else {
            eprintln!("skipped: TEST_DATABASE_URL missing");
            return;
        };
        let root = crate::db::connect(&url, None).await.unwrap();
        let schema = format!("information_quota_test_{}", Uuid::new_v4().simple());
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&root)
            .await
            .unwrap();
        let pool = crate::db::connect(&url, Some(&schema)).await.unwrap();
        crate::db::migrate(&pool).await.unwrap();
        let owner = Uuid::new_v4();
        sqlx::query("INSERT INTO products(product_id,name,client_ids) VALUES('shared-test','shared',ARRAY['forum-web','xiaoban-android','xiaoban-desktop-wallpaper'])").execute(&pool).await.unwrap();
        let binding:i64=sqlx::query_scalar("INSERT INTO bindings(uid,product_id,status) VALUES($1,'shared-test','active') RETURNING id").bind(owner).fetch_one(&pool).await.unwrap();
        let mut tx = pool.begin().await.unwrap();
        quota::grant(
            &mut tx,
            binding,
            "monthly",
            100,
            Some(Utc::now() + chrono::Duration::days(10)),
            "test-monthly",
            "",
        )
        .await
        .unwrap();
        quota::grant(&mut tx, binding, "topup", 200, None, "test-topup", "")
            .await
            .unwrap();
        tx.commit().await.unwrap();
        let policy = Policy {
            product_id: "shared-test".into(),
            reserve_units: 150,
            model: "mock".into(),
            input_per_million: 1_000_000,
            output_per_million: 2_000_000,
            search_units: 5,
            email_units: 1,
            execution: Some(ExecutionConfig {
                base_url: "https://original.example.test".into(),
                protocol: "messages".into(),
                credential_env: "ORIGINAL_PLATFORM_KEY".into(),
            }),
        };
        let input = Request {
            edition_id: Uuid::new_v4(),
            subscription_id: Uuid::new_v4(),
            owner,
            budget_per_run: Some(150),
            budget_per_month: Some(300),
            usage: vec![
                json!({"stage":"platform_model","usage":{"model":"mock","provider_usage":{"prompt_tokens":10,"completion_tokens":20}}}),
                json!({"stage":"platform_search","usage":{"requests":1}}),
                json!({"stage":"platform_email","usage":{"accepted":true}}),
            ],
        };
        let (a, b) = tokio::join!(
            transact(&pool, Some(&policy), "reserve", &input),
            transact(&pool, Some(&policy), "reserve", &input)
        );
        a.unwrap();
        b.unwrap();
        let total = quota::live_total(&mut pool.acquire().await.unwrap(), binding)
            .await
            .unwrap();
        assert_eq!(total, 150);
        let allocations:Vec<(String,i64)>=sqlx::query_as("SELECT b.kind,a.amount FROM information_job_allocations a JOIN quota_buckets b ON a.bucket_id=b.id WHERE a.edition_id=$1 ORDER BY b.id").bind(input.edition_id).fetch_all(&pool).await.unwrap();
        assert_eq!(
            allocations,
            vec![("monthly".into(), 100), ("topup".into(), 50)]
        );
        let rotated = Policy {
            product_id: "another-product".into(),
            model: "new-model".into(),
            execution: Some(ExecutionConfig {
                base_url: "https://new.example.test".into(),
                protocol: "chat_completions".into(),
                credential_env: "NEW_PLATFORM_KEY".into(),
            }),
            ..policy.clone()
        };
        // A retry after configuration rotation still uses the reservation's original model and endpoint.
        transact(&pool, Some(&rotated), "reserve", &input)
            .await
            .unwrap();
        let frozen = execution_policy(
            &pool,
            &ExecutionRequest {
                edition_id: input.edition_id,
                subscription_id: input.subscription_id,
                owner,
                payload: json!({}),
            },
        )
        .await
        .unwrap();
        assert_eq!(frozen.model, "mock");
        let execution = frozen.execution.unwrap();
        assert_eq!(execution.base_url, "https://original.example.test");
        assert_eq!(execution.protocol, "messages");
        assert_eq!(execution.credential_env, "ORIGINAL_PLATFORM_KEY");
        sqlx::query("UPDATE products SET enabled=false")
            .execute(&pool)
            .await
            .unwrap();
        let (a, b) = tokio::join!(
            transact(&pool, None, "settle", &input),
            transact(&pool, Some(&rotated), "settle", &input)
        );
        a.unwrap();
        b.unwrap();
        assert_eq!(
            quota::live_total(&mut pool.acquire().await.unwrap(), binding)
                .await
                .unwrap(),
            244
        );
        assert!(transact(&pool, Some(&policy), "release", &input)
            .await
            .is_err());
        let other = Request {
            edition_id: input.edition_id,
            subscription_id: input.subscription_id,
            owner: Uuid::new_v4(),
            budget_per_run: Some(150),
            budget_per_month: Some(300),
            usage: vec![],
        };
        assert!(transact(&pool, Some(&policy), "settle", &other)
            .await
            .is_err());
        let fail = Request {
            edition_id: Uuid::new_v4(),
            subscription_id: input.subscription_id,
            owner,
            budget_per_run: Some(150),
            budget_per_month: Some(300),
            usage: vec![],
        };
        sqlx::query("UPDATE products SET enabled=true")
            .execute(&pool)
            .await
            .unwrap();
        transact(&pool, Some(&policy), "reserve", &fail)
            .await
            .unwrap();
        sqlx::query("UPDATE bindings SET status='disabled'")
            .execute(&pool)
            .await
            .unwrap();
        transact(&pool, None, "release", &fail).await.unwrap();
        transact(&pool, Some(&policy), "release", &fail)
            .await
            .unwrap();
        assert_eq!(
            quota::live_total(&mut pool.acquire().await.unwrap(), binding)
                .await
                .unwrap(),
            244
        );
        let cancelled = Request {
            edition_id: Uuid::new_v4(),
            subscription_id: input.subscription_id,
            owner,
            budget_per_run: Some(150),
            budget_per_month: Some(300),
            usage: vec![],
        };
        transact(&pool, Some(&policy), "release", &cancelled)
            .await
            .unwrap();
        assert!(transact(&pool, Some(&policy), "reserve", &cancelled)
            .await
            .is_err());
        sqlx::query("UPDATE bindings SET status='active'")
            .execute(&pool)
            .await
            .unwrap();
        let expensive = Policy {
            reserve_units: 500,
            ..policy.clone()
        };
        let insufficient = Request {
            edition_id: Uuid::new_v4(),
            subscription_id: input.subscription_id,
            owner,
            budget_per_run: Some(500),
            budget_per_month: Some(1000),
            usage: vec![],
        };
        assert!(matches!(
            transact(&pool, Some(&expensive), "reserve", &insufficient).await,
            Err(AppError::Coded {
                status: StatusCode::PAYMENT_REQUIRED,
                ..
            })
        ));
        // Different editions contend for one user-defined monthly ceiling, not separate wallets.
        let budget_sub = Uuid::new_v4();
        let first = Request {
            edition_id: Uuid::new_v4(),
            subscription_id: budget_sub,
            owner,
            budget_per_run: Some(150),
            budget_per_month: Some(200),
            usage: vec![],
        };
        let second = Request {
            edition_id: Uuid::new_v4(),
            subscription_id: budget_sub,
            owner,
            budget_per_run: Some(150),
            budget_per_month: Some(200),
            usage: vec![],
        };
        let (a, b) = tokio::join!(
            transact(&pool, Some(&policy), "reserve", &first),
            transact(&pool, Some(&policy), "reserve", &second)
        );
        assert_ne!(a.is_ok(), b.is_ok());
        let (held, blocked) = if a.is_ok() {
            (&first, &second)
        } else {
            (&second, &first)
        };
        assert!(matches!(
            transact(&pool, Some(&policy), "reserve", blocked).await,
            Err(AppError::Coded {
                code: "subscription_budget_exceeded",
                ..
            })
        ));
        let retry = Request {
            edition_id: held.edition_id,
            subscription_id: budget_sub,
            owner,
            budget_per_run: Some(1),
            budget_per_month: Some(1),
            usage: vec![],
        };
        transact(&pool, None, "reserve", &retry).await.unwrap();
        assert_eq!(
            quota::live_total(&mut pool.acquire().await.unwrap(), binding)
                .await
                .unwrap(),
            94
        );
        // An old reservation remains in its original UTC month; retries never create a new hold.
        sqlx::query("UPDATE information_job_reservations SET budget_month=(date_trunc('month',now() AT TIME ZONE 'UTC')-interval '1 month')::date WHERE edition_id=$1").bind(held.edition_id).execute(&pool).await.unwrap();
        transact(&pool, None, "reserve", held).await.unwrap();
        let next_month = Request {
            edition_id: Uuid::new_v4(),
            subscription_id: budget_sub,
            owner,
            budget_per_run: Some(50),
            budget_per_month: Some(50),
            usage: vec![],
        };
        transact(&pool, Some(&policy), "reserve", &next_month)
            .await
            .unwrap();
        let reserved: i64 = sqlx::query_scalar(
            "SELECT reserved FROM information_job_reservations WHERE edition_id=$1",
        )
        .bind(next_month.edition_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(reserved, 50);
        transact(&pool, None, "release", &next_month).await.unwrap();
        transact(&pool, None, "release", held).await.unwrap();
        transact(&pool, Some(&policy), "reserve", blocked)
            .await
            .unwrap();
        transact(&pool, None, "release", blocked).await.unwrap();
        let balance: i64 =
            sqlx::query_scalar("SELECT sum(delta)::bigint FROM quota_ledger WHERE binding_id=$1")
                .bind(binding)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(balance, 244);
        pool.close().await;
        sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
            .execute(&root)
            .await
            .unwrap();
        root.close().await;
    }
}

#[cfg(test)]
mod protocol_tests {
    use super::*;
    #[test]
    fn deepseek_openai_and_claude_use_platform_selected_model() {
        for api in [
            crate::proxy::Api::ChatCompletions,
            crate::proxy::Api::Messages,
        ] {
            let mut body = json!({"model":"untrusted-client-choice","stream":true,"max_tokens":999999,"response_format":{"type":"json_object"},"messages":[{"role":"system","content":"editor rules"},{"role":"user","content":"source data"}]});
            prepare_model_request(api, "configured-platform-model", &mut body).unwrap();
            assert_eq!(body["model"], "configured-platform-model");
            assert_eq!(body["stream"], false);
            assert_eq!(body["max_tokens"], 2000);
            if api == crate::proxy::Api::Messages {
                assert_eq!(body["system"], "editor rules");
                assert!(body.get("response_format").is_none());
                assert_eq!(body["messages"].as_array().unwrap().len(), 1);
            }
        }
        let response = json!({"content":[{"type":"thinking","thinking":"private"},{"type":"text","text":"{\"items\":[]}"}],"usage":{"input_tokens":10,"output_tokens":20}});
        let normalized = normalize_model_response(
            crate::proxy::Api::Messages,
            "configured-platform-model",
            &response,
        )
        .unwrap();
        assert_eq!(
            normalized["choices"][0]["message"]["content"],
            "{\"items\":[]}"
        );
        assert_eq!(normalized["usage"]["input_tokens"], 10);
        assert!(!normalized.to_string().contains("private"));
        assert!(
            normalize_model_response(crate::proxy::Api::ChatCompletions, "mock", &json!({}))
                .is_err()
        );
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionRequest {
    edition_id: Uuid,
    subscription_id: Uuid,
    owner: Uuid,
    payload: Value,
}
async fn execution_policy(pool: &PgPool, input: &ExecutionRequest) -> Result<Policy, AppError> {
    let value:Option<(Value,)>=sqlx::query_as("SELECT r.policy FROM information_job_reservations r JOIN bindings b ON b.id=r.binding_id JOIN products p ON p.product_id=b.product_id WHERE r.edition_id=$1 AND r.subscription_id=$2 AND b.uid=$3 AND r.status='reserved' AND b.status='active' AND p.enabled")
        .bind(input.edition_id).bind(input.subscription_id).bind(input.owner).fetch_optional(pool).await?;
    serde_json::from_value(
        value
            .ok_or_else(|| AppError::Forbidden("期刊未预约、已结束或产品已停用".into()))?
            .0,
    )
    .map_err(|_| AppError::Internal("invalid frozen policy".into()))
}
