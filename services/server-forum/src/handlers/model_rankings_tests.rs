use super::*;

#[tokio::test]
async fn content_status_is_owner_only_and_schemes_hide_unavailable_references() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    let uid = Uuid::new_v4();
    sqlx::query("INSERT INTO users(uid,nickname,trust_level) VALUES($1,'Owner',1)").bind(uid).execute(&h.pool).await?;
    let token = (h.make_token)(uid);
    let other = h.member().await?;
    sqlx::query("INSERT INTO pack_uploads(file_key,uid,kind,size,sha256) VALUES('packs/upload-qa',$1,'effect',1,$2)")
        .bind(uid).bind("a".repeat(64)).execute(&h.pool).await?;
    let status_path = "/api/packs/upload-status?file_key=packs/upload-qa";
    assert_eq!(call(&h.app,"GET",status_path,None,Value::Null).await.0,StatusCode::UNAUTHORIZED);
    assert_eq!(call(&h.app,"GET",status_path,Some(&other),Value::Null).await.0,StatusCode::NOT_FOUND);
    let (s, body) = call(&h.app,"GET",status_path,Some(&token),Value::Null).await;
    assert_eq!(s,StatusCode::OK);
    assert_eq!(body,json!({"file_key":"packs/upload-qa","verified":false,"used":false}));
    sqlx::query("UPDATE pack_uploads SET verified=true,used=true WHERE file_key='packs/upload-qa'").execute(&h.pool).await?;
    assert_eq!(call(&h.app,"GET",status_path,Some(&token),Value::Null).await.1["used"],true);
    let (id,): (i64,) = sqlx::query_as("INSERT INTO schemes(author_uid,title,items) VALUES($1,'Public scheme',$2) RETURNING id")
        .bind(uid).bind(json!([{"pack_id":999999,"version":1,"slug":"old-private-slug","kind":"task"}])).fetch_one(&h.pool).await?;
    let path = format!("/api/schemes/{id}");
    let (s, scheme) = call(&h.app,"GET",&path,None,Value::Null).await;
    assert_eq!(s,StatusCode::OK);
    assert_eq!(scheme["items"][0],json!({"pack_id":999999,"version":1,"available":false}));
    sqlx::query("UPDATE users SET status='banned' WHERE uid=$1").bind(uid).execute(&h.pool).await?;
    assert_eq!(call(&h.app,"GET",&path,None,Value::Null).await.0,StatusCode::NOT_FOUND);
    let (_, listed) = call(&h.app,"GET","/api/schemes",None,Value::Null).await;
    assert_eq!(listed["total"],0);
    h.finish().await
}

#[tokio::test]
async fn pack_links_and_scheme_references_follow_live_visibility() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    let uid = Uuid::new_v4();
    sqlx::query("INSERT INTO users(uid,nickname,trust_level) VALUES($1,'Creator',1)").bind(uid).execute(&h.pool).await?;
    let token = (h.make_token)(uid);
    let other = h.member().await?;
    let (pack,): (i64,) = sqlx::query_as("INSERT INTO packs(kind,slug,title,author_uid) VALUES('task','visibility-test','Visible',$1) RETURNING id")
        .bind(uid).fetch_one(&h.pool).await?;
    sqlx::query("INSERT INTO pack_versions(pack_id,version,label,file_key,sha256,size,manifest) VALUES($1,1,'1','packs/test',$2,1,'{}')")
        .bind(pack).bind("a".repeat(64)).execute(&h.pool).await?;
    let (topic,): (i64,) = sqlx::query_as("INSERT INTO topics(author_uid,category_id,title,body_md,body_html,excerpt) SELECT $1,id,'Topic','body','body','body' FROM categories LIMIT 1 RETURNING id")
        .bind(uid).fetch_one(&h.pool).await?;
    sqlx::query("UPDATE packs SET topic_id=$2 WHERE id=$1").bind(pack).bind(topic).execute(&h.pool).await?;
    let payload = json!({"title":"Scheme","items":[{"pack_id":pack,"version":1}]});
    let (status,created) = call(&h.app,"POST","/api/schemes",Some(&token),payload.clone()).await;
    assert_eq!(status,StatusCode::OK,"{created}");
    let scheme=created["id"].as_i64().unwrap();
    let (status,_) = call(&h.app,"POST","/api/schemes",Some(&token),json!({"title":"Bad version","items":[{"pack_id":pack,"version":123}]})).await;
    assert_eq!(status,StatusCode::BAD_REQUEST);
    assert_eq!(call(&h.app,"GET",&format!("/api/packs/{pack}"),None,Value::Null).await.0,StatusCode::OK);
    sqlx::query("UPDATE topics SET min_read_level=1 WHERE id=$1").bind(topic).execute(&h.pool).await?;
    for path in [format!("/api/packs/{pack}"),format!("/api/packs/{pack}/download"),format!("/api/share/pack/{pack}")] {
        assert_eq!(call(&h.app,"GET",&path,None,Value::Null).await.0,StatusCode::NOT_FOUND,"{path}");
    }
    assert_eq!(call(&h.app,"GET","/api/packs",None,Value::Null).await.1["total"],0);
    let refs = call(&h.app,"GET",&format!("/api/schemes/{scheme}"),None,Value::Null).await.1;
    assert_eq!(refs["items"][0],json!({"pack_id":pack,"version":1,"available":false}));
    let listed=call(&h.app,"GET","/api/schemes",None,Value::Null).await.1;
    assert_eq!(listed["items"][0]["items"][0],refs["items"][0]);
    assert_eq!(call(&h.app,"POST","/api/schemes",Some(&token),payload).await.0,StatusCode::BAD_REQUEST);
    sqlx::query("UPDATE topics SET min_read_level=0 WHERE id=$1").bind(topic).execute(&h.pool).await?;
    sqlx::query("UPDATE users SET status='banned' WHERE uid=$1").bind(uid).execute(&h.pool).await?;
    assert_eq!(call(&h.app,"GET",&format!("/api/packs/{pack}"),None,Value::Null).await.0,StatusCode::NOT_FOUND);
    assert_eq!(call(&h.app,"GET",&format!("/api/share/scheme/{scheme}"),None,Value::Null).await.0,StatusCode::NOT_FOUND);
    sqlx::query("UPDATE users SET status='normal' WHERE uid=$1").bind(uid).execute(&h.pool).await?;
    assert_eq!(call(&h.app,"GET","/api/me/schemes",Some(&token),Value::Null).await.1["items"][0]["id"],scheme);
    assert_eq!(call(&h.app,"POST",&format!("/api/schemes/{scheme}/remove"),Some(&other),Value::Null).await.0,StatusCode::NOT_FOUND);
    assert_eq!(call(&h.app,"POST",&format!("/api/schemes/{scheme}/remove"),Some(&token),Value::Null).await.0,StatusCode::OK);
    assert_eq!(call(&h.app,"GET",&format!("/api/schemes/{scheme}"),None,Value::Null).await.0,StatusCode::NOT_FOUND);
    h.finish().await
}

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
