use super::*;

#[tokio::test]
async fn model_snapshots_only_expose_published_past_rows_and_keep_token_precision() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    let date = (chrono::Utc::now() - chrono::Duration::days(1)).date_naive().to_string();
    let future = (chrono::Utc::now() + chrono::Duration::days(1)).date_naive().to_string();
    let tokens = "900719925474099312345";
    for (key, status, delay) in [(&date, "published", -1), (&future, "published", 1)] {
        sqlx::query("INSERT INTO discover_items(kind,key,status,published_at,data) VALUES('model_usage',$1,$2,now()+$3*interval '1 day',$4)")
            .bind(key).bind(status).bind(delay).bind(json!({"date":key,"total_tokens":tokens,"models":[]})).execute(&h.pool).await?;
    }
    sqlx::query("INSERT INTO discover_drafts(kind,key,data,updated_by) VALUES('model_usage','private',$1,'test')")
        .bind(json!({"date":"private","total_tokens":"42"})).execute(&h.pool).await?;
    let (s, latest) = call(&h.app,"GET","/api/discover/model-usage",None,Value::Null).await;
    assert_eq!(s,StatusCode::OK);
    assert_eq!(latest["date"],date);
    assert_eq!(latest["total_tokens"],tokens);
    assert!(latest["updated_at"].is_string());
    let (s, missing) = call(&h.app,"GET",&format!("/api/discover/model-usage?date={future}"),None,Value::Null).await;
    assert_eq!(s,StatusCode::NOT_FOUND,"{missing}");
    let (s, dates) = call(&h.app,"GET","/api/discover/model-usage/dates",None,Value::Null).await;
    assert_eq!(s,StatusCode::OK);
    assert_eq!(dates["items"].as_array().unwrap().len(),1);
    assert_eq!(dates["items"][0]["date"],date);
    for bad in ["2026-02-30","2026-1-1","bad"] {
        assert_eq!(call(&h.app,"GET",&format!("/api/discover/model-usage?date={bad}"),None,Value::Null).await.0,StatusCode::BAD_REQUEST);
    }
    assert_eq!(call(&h.app,"GET","/api/discover/models/v2",None,Value::Null).await.0,StatusCode::NOT_FOUND);
    sqlx::query("INSERT INTO discover_items(kind,key,status,data) VALUES('model_board','artificial-analysis','published',$1)")
        .bind(json!({"models":[{"id":"missing-score","metrics":{"intelligence":null}}],"index_version":"5"})).execute(&h.pool).await?;
    let (s, board) = call(&h.app,"GET","/api/discover/models/v2",None,Value::Null).await;
    assert_eq!(s,StatusCode::OK);
    assert!(board["models"][0]["metrics"]["intelligence"].is_null());
    h.finish().await
}
