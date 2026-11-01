//! Platform SMTP only. Same STARTTLS/implicit TLS conventions as server-account.
//! SMTP has no idempotency API: after starting a send, an ambiguous result is never auto-retried.
use lettre::{message::{Mailbox, MultiPart}, transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use uuid::Uuid;
pub fn configured()->bool {std::env::var("INFORMATION_MAIL_ENABLED").as_deref()==Ok("true") && ["INFORMATION_SMTP_HOST","INFORMATION_MAIL_FROM"].iter().all(|k|std::env::var(k).is_ok_and(|v|!v.is_empty()))}
pub fn escape(s:&str)->String{s.replace('&',"&amp;").replace('<',"&lt;").replace('>',"&gt;").replace('"',"&quot;").replace('\'',"&#39;")}
pub async fn send(to:&str,title:&str,html:&str,text:&str,id:Uuid)->Result<String,()> {
    if !configured(){return Err(());}
    let from:Mailbox=std::env::var("INFORMATION_MAIL_FROM").map_err(|_|())?.parse().map_err(|_|())?;
    let email=Message::builder().from(from).to(to.parse().map_err(|_|())?).subject(title)
        .message_id(Some(format!("{id}@information.xiangban.xyz")))
        .multipart(MultiPart::alternative_plain_html(text.to_string(),html.to_string())).map_err(|_|())?;
    let host=std::env::var("INFORMATION_SMTP_HOST").map_err(|_|())?;
    let port=std::env::var("INFORMATION_SMTP_PORT").ok().and_then(|s|s.parse().ok()).unwrap_or(587);
    let builder=if port==465{AsyncSmtpTransport::<Tokio1Executor>::relay(&host)}else{AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)}.map_err(|_|())?;
    let mut builder=builder.port(port).timeout(Some(std::time::Duration::from_secs(30)));
    if let Ok(user)=std::env::var("INFORMATION_SMTP_USER"){builder=builder.credentials(Credentials::new(user,std::env::var("INFORMATION_SMTP_PASSWORD").map_err(|_|())?));}
    builder.build().send(email).await.map_err(|_|())?;
    Ok(id.to_string())
}
#[cfg(test)] mod tests {
    #[test] fn escaping(){assert_eq!(super::escape("<script a=\"&\">'"),"&lt;script a=&quot;&amp;&quot;&gt;&#39;");}
}
