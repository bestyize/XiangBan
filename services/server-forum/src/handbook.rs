//! Public first-party evidence for product and community guide articles, not invented events.
use crate::{error::{AppError, AppResult}, state::AppState};
use axum::{extract::Path, routing::get, response::Html, Json, Router};
use serde_json::{json,Value};
use std::sync::OnceLock;
fn data() -> &'static Value {
    static VALUE: OnceLock<Value> = OnceLock::new();
    VALUE.get_or_init(||{
        let source=std::env::var("FORUM_EDITORIAL_HANDBOOK").ok().map(|path|std::fs::read_to_string(path).expect("read public editorial handbook"));
        serde_json::from_str(source.as_deref().unwrap_or(include_str!("../editorial/handbook.json"))).expect("valid first-party handbook")
    })
}
fn source(id:&str) -> AppResult<Value> {
    let entry=data()["entries"].as_array().unwrap().iter().find(|entry|entry["id"]==id).ok_or_else(||AppError::not_found("资料"))?;
    let documents=entry["documents"].as_array().unwrap().iter().filter_map(|name|data()["documents"].get(name.as_str()?)).cloned().collect::<Vec<_>>();
    Ok(json!({"id":id,"title":entry["title"],"focus":entry["focus"],"boundary":entry["boundary"],"documents":documents}))
}
pub fn api_routes()->Router<AppState>{Router::new().route("/community/sources/{id}",get(api_source))}
pub fn page_routes()->Router<AppState>{Router::new().route("/community/sources/{id}",get(page))}
async fn api_source(Path(id):Path<String>)->AppResult<Json<Value>>{Ok(Json(source(&id)?))}
fn escape(s:&str)->String{s.replace('&',"&amp;").replace('<',"&lt;").replace('>',"&gt;").replace('"',"&quot;")}
async fn page(Path(id):Path<String>)->AppResult<Html<String>>{
    let source=source(&id)?;
    let title=escape(source["title"].as_str().unwrap());
    let mut body=format!("<p><a href=\"/\">社区首页</a> / 公开资料</p><h1>{title}</h1><p>{}</p>",escape(source["boundary"].as_str().unwrap()));
    for doc in source["documents"].as_array().unwrap(){
        body+=&format!("<section><h2>{}</h2><p><a href=\"{}\">对应产品或规则页面</a></p><div>{}</div></section>",escape(doc["title"].as_str().unwrap()),escape(doc["url"].as_str().unwrap()),crate::markdown::render(doc["text"].as_str().unwrap(),true));
    }
    Ok(Html(format!("<!doctype html><html lang=\"zh-CN\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta name=\"robots\" content=\"noindex\"><title>{title} · 小伴公开资料</title><style>body{{max-width:800px;margin:30px auto;padding:0 20px;font:16px/1.8 system-ui;color:#242424}}pre{{overflow:auto;background:#f5f5f5;padding:16px}}table{{border-collapse:collapse}}td,th{{border:1px solid #ddd;padding:8px}}a{{color:#ac3247}}section{{margin:30px 0}}</style><main>{body}</main></html>")))
}
