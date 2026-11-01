//! Internal scheduled-job accounting against the existing product quota buckets.
//! Off by default. No prices, product IDs, model IDs, or entitlements are seeded.
use crate::{state::AppState,error::AppError,quota,ledger::{allocate,BucketView}};
use axum::{extract::{Path,State},http::{HeaderMap,StatusCode},Json};
use serde::Deserialize;
use serde_json::{json,Value};
use uuid::Uuid;
use sqlx::{PgPool,FromRow};
use chrono::{DateTime,Utc};

pub async fn init(pool:&PgPool)->Result<(),AppError>{
 sqlx::raw_sql("CREATE TABLE IF NOT EXISTS information_job_reservations (edition_id UUID PRIMARY KEY, subscription_id UUID NOT NULL, binding_id BIGINT NOT NULL REFERENCES bindings(id), status TEXT NOT NULL, reserved BIGINT NOT NULL, settled BIGINT, policy JSONB NOT NULL, usage JSONB, created_at TIMESTAMPTZ NOT NULL DEFAULT now()); CREATE TABLE IF NOT EXISTS information_job_allocations (edition_id UUID NOT NULL REFERENCES information_job_reservations(edition_id),bucket_id BIGINT NOT NULL REFERENCES quota_buckets(id),amount BIGINT NOT NULL,PRIMARY KEY(edition_id,bucket_id));").execute(pool).await?;Ok(())
}
#[derive(Clone,Deserialize,serde::Serialize)]
#[serde(deny_unknown_fields)]
struct Policy { product_id:String, reserve_units:i64, model:String, input_per_million:i64, output_per_million:i64, search_units:i64, email_units:i64 }
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
pub struct Request {edition_id:Uuid,subscription_id:Uuid,owner:Uuid,#[serde(default)] usage:Vec<Value>}
fn authorize(headers:&HeaderMap)->Result<Policy,AppError>{
 let key=std::env::var("AI_INFORMATION_SERVICE_TOKEN").ok().filter(|s|s.len()>=32).ok_or_else(||AppError::ServiceUnavailable("后台任务额度尚未开放".into()))?;
 let given=headers.get("authorization").and_then(|v|v.to_str().ok()).and_then(|s|s.strip_prefix("Bearer ")).unwrap_or("");
 let signing=ring::hmac::Key::new(ring::hmac::HMAC_SHA256,key.as_bytes());
 let tag=ring::hmac::sign(&signing,b"information-jobs");
 let candidate=ring::hmac::Key::new(ring::hmac::HMAC_SHA256,given.as_bytes());
 if ring::hmac::verify(&candidate,b"information-jobs",tag.as_ref()).is_err(){return Err(AppError::Unauthorized("服务凭证无效".into()));}
 let policy:Policy=serde_json::from_str(&std::env::var("AI_INFORMATION_POLICY").unwrap_or_default()).map_err(|_|AppError::ServiceUnavailable("后台任务价格策略尚未配置".into()))?;
 if policy.reserve_units<=0||policy.product_id.is_empty()||policy.model.is_empty()||[policy.input_per_million,policy.output_per_million,policy.search_units,policy.email_units].iter().any(|v|*v<0){return Err(AppError::ServiceUnavailable("后台任务价格策略无效".into()));}
 Ok(policy)
}
fn cost(policy:&Policy,usage:&[Value])->Result<i64,AppError>{
 let mut units:i128=i128::from(policy.email_units);
 for item in usage {
  let stage=item.get("stage").and_then(Value::as_str).unwrap_or("");let value=&item["usage"];
  match stage {
   "platform_search"=>units+=i128::from(value["requests"].as_u64().unwrap_or(0))*i128::from(policy.search_units),
   "platform_model"=>{
    if value["model"].as_str()!=Some(&policy.model){return Err(AppError::BadRequest("计量模型与价格策略不匹配".into()));}
    let u=&value["provider_usage"];
    let input=u["prompt_tokens"].as_u64().or_else(||u["input_tokens"].as_u64()).ok_or_else(||AppError::BadRequest("缺少模型计量".into()))?;
    let output=u["completion_tokens"].as_u64().or_else(||u["output_tokens"].as_u64()).ok_or_else(||AppError::BadRequest("缺少模型计量".into()))?;
    // Cached / reasoning token accounting needs a policy-specific adapter before enabling those modes.
    if u.get("cache_read_input_tokens").and_then(Value::as_u64).unwrap_or(0)>0||u.get("cache_creation_input_tokens").and_then(Value::as_u64).unwrap_or(0)>0{return Err(AppError::BadRequest("当前策略未配置缓存计量".into()));}
    units+=(i128::from(input)*i128::from(policy.input_per_million)+i128::from(output)*i128::from(policy.output_per_million)+999_999)/1_000_000;
   },_=>{}
  }
 }
 i64::try_from(units).map_err(|_|AppError::BadRequest("计量溢出".into()))
}
#[derive(FromRow)] struct Reservation {subscription_id:Uuid,binding_id:i64,status:String,reserved:i64,policy:Value}
pub async fn action(State(s):State<AppState>,headers:HeaderMap,Path(action):Path<String>,Json(input):Json<Request>)->Result<Json<Value>,AppError>{
 let policy=authorize(&headers)?;
 if !["reserve","settle","release"].contains(&action.as_str()){return Err(AppError::NotFound("操作不存在".into()));}
 transact(&s.pool,&policy,&action,&input).await?;
 Ok(Json(json!({"ok":true})))
}
async fn transact(pool:&PgPool,policy:&Policy,action:&str,input:&Request)->Result<(),AppError>{
 let mut tx=pool.begin().await?;
 let binding:Option<(i64,)>=sqlx::query_as("SELECT b.id FROM bindings b JOIN products p ON p.product_id=b.product_id WHERE b.uid=$1 AND b.product_id=$2 AND b.status='active' AND p.enabled").bind(input.owner).bind(&policy.product_id).fetch_optional(&mut *tx).await?;
 let binding=binding.ok_or_else(||AppError::Coded{status:StatusCode::PAYMENT_REQUIRED,code:"insufficient_quota",message:"请先开通小伴额度".into()})?.0;
 // Same lock as gateway reconciliation, grants, and redemption.
 sqlx::query("SELECT pg_advisory_xact_lock($1)").bind(binding).execute(&mut *tx).await?;
 let old:Option<Reservation>=sqlx::query_as("SELECT subscription_id,binding_id,status,reserved,policy FROM information_job_reservations WHERE edition_id=$1 FOR UPDATE").bind(input.edition_id).fetch_optional(&mut *tx).await?;
 if let Some(ref old)=old {
  if old.binding_id!=binding||old.subscription_id!=input.subscription_id{return Err(AppError::Conflict("期刊归属不一致".into()));}
  if old.status=="settled" {return if action=="release"{Err(AppError::Conflict("期刊已结算".into()))}else{Ok(())};}
  if old.status=="released" {return if action=="release"{Ok(())}else{Err(AppError::Conflict("期刊预约已释放".into()))};}
 }
 if action=="reserve" {
  if old.is_some(){return Ok(());}
  let live:Vec<(i64,i64,Option<DateTime<Utc>>)>=sqlx::query_as(&format!("SELECT id,remaining,expires_at FROM quota_buckets WHERE binding_id=$1 AND {} FOR UPDATE",quota::LIVE)).bind(binding).fetch_all(&mut *tx).await?;
  let buckets:Vec<_>=live.into_iter().map(|(id,remaining,expires_at)|BucketView{id,remaining,expires_at}).collect();
  let (takes,short)=allocate(policy.reserve_units,&buckets);
  if short>0{return Err(AppError::Coded{status:StatusCode::PAYMENT_REQUIRED,code:"insufficient_quota",message:"小伴额度不足".into()});}
  sqlx::query("INSERT INTO information_job_reservations(edition_id,subscription_id,binding_id,status,reserved,policy) VALUES($1,$2,$3,'reserved',$4,$5)").bind(input.edition_id).bind(input.subscription_id).bind(binding).bind(policy.reserve_units).bind(json!(policy)).execute(&mut *tx).await?;
  for (bucket,amount) in takes {
   sqlx::query("UPDATE quota_buckets SET remaining=remaining-$2 WHERE id=$1").bind(bucket).bind(amount).execute(&mut *tx).await?;
   sqlx::query("INSERT INTO information_job_allocations VALUES($1,$2,$3)").bind(input.edition_id).bind(bucket).bind(amount).execute(&mut *tx).await?;
   quota::add_ledger(&mut tx,binding,Some(bucket),"job_reserve",-amount,&input.edition_id.to_string()).await?;
  }
 }else if let Some(old)=old {
  let frozen:Policy=serde_json::from_value(old.policy).map_err(|_|AppError::Internal("invalid reservation policy".into()))?;
  let used=if action=="release"{0}else{cost(&frozen,&input.usage)?};
  if used>old.reserved{return Err(AppError::Conflict("实际用量超过预约上限，待核实；不会额外扣款".into()));}
  let mut refund=old.reserved-used;
  let allocations:Vec<(i64,i64)>=sqlx::query_as("SELECT bucket_id,amount FROM information_job_allocations WHERE edition_id=$1 ORDER BY bucket_id DESC").bind(input.edition_id).fetch_all(&mut *tx).await?;
  for (bucket,amount) in allocations {
   let give=refund.min(amount);refund-=give;if give==0{continue;}
   // Preserve original expiration and administrator revocation; never resurrect expired credit.
   let changed=sqlx::query("UPDATE quota_buckets SET remaining=remaining+$2,closed_at=NULL,closed_reason=NULL WHERE id=$1 AND (closed_at IS NULL OR closed_reason='spent') AND (expires_at IS NULL OR expires_at>now())").bind(bucket).bind(give).execute(&mut *tx).await?;
   quota::add_ledger(&mut tx,binding,Some(bucket),"job_release",if changed.rows_affected()>0{give}else{0},&input.edition_id.to_string()).await?;
  }
  quota::add_ledger(&mut tx,binding,None,"job_settle",0,&input.edition_id.to_string()).await?;
  sqlx::query("UPDATE information_job_reservations SET status=$2,settled=$3,usage=$4 WHERE edition_id=$1").bind(input.edition_id).bind(if action=="release"{"released"}else{"settled"}).bind(used).bind(json!(input.usage)).execute(&mut *tx).await?;
 }else {return Ok(());}
 sqlx::query("UPDATE bindings SET change_seq=change_seq+1 WHERE id=$1").bind(binding).execute(&mut *tx).await?;
 tx.commit().await?;Ok(())
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn provider_rates_and_missing_usage(){
  let p=Policy{product_id:"test".into(),model:"configured-test-model".into(),reserve_units:100,input_per_million:2_000_000,output_per_million:3_000_000,search_units:4,email_units:5};
  let usage=vec![json!({"stage":"platform_model","usage":{"model":p.model,"provider_usage":{"prompt_tokens":2,"completion_tokens":3}}}),json!({"stage":"platform_search","usage":{"requests":1}})];
  assert_eq!(cost(&p,&usage).unwrap(),22);
  assert!(cost(&p,&[json!({"stage":"platform_model","usage":{}})]).is_err());
 }
}
