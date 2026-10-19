//! Convert persisted widget content into the existing native reminder scheduler.
//! No WebView timer is used and hidden cards continue to remind.
use chrono::{DateTime,Datelike,Local,NaiveDateTime,TimeZone};
use sha2::{Digest,Sha256};
use crate::{reminders::Rule,widgets::Layout};

fn rule(id:String,text:String,at:String,repeat:&str,notify:bool,done:bool,completed:&str,now:DateTime<Local>)->Option<Rule>{
    if !notify || done && repeat=="none" || repeat!="none" && completed==now.format("%Y-%m-%d").to_string(){return None;}
    let date=NaiveDateTime::parse_from_str(&at,"%Y-%m-%dT%H:%M").ok()?;
    let not_before=Local.from_local_datetime(&date).earliest()?.timestamp();
    let weekdays=match repeat {"weekdays"=>vec![1,2,3,4,5],"weekly"=>vec![date.weekday().number_from_monday()],_=>vec![]};
    let hash=Sha256::digest(format!("{id}\n{text}\n{at}\n{repeat}"));
    let revision=hash[..6].iter().fold(0u64,|a,b|a*256+*b as u64);
    Some(Rule{id,revision,enabled:true,kind:"custom".into(),repeat:match repeat{"none"=>"once","weekly"=>"weekdays",v=>v}.into(),time:date.format("%H:%M").to_string(),once_at:at,weekdays,text,not_before,..Rule::default()})
}
pub fn rules(layout:&Layout,now:DateTime<Local>)->Vec<Rule>{
    let mut result=Vec::new();
    for widget in &layout.widgets {
        if widget.kind=="notes" {if let Some(note)=&widget.note {for item in &note.items {
            if let Some(r)=rule(format!("widget-{}-{}",widget.id,item.id),format!("待办：{}",item.text),format!("{}T{}",item.date,item.time),&item.repeat,item.notify,item.done,&item.completed_on,now){result.push(r);}
        }}}
        if widget.kind=="reminders" {if let Some(living)=&widget.living {for item in &living.reminders {
            let text=format!("日程：{}{}",item.title,if item.location.is_empty(){String::new()}else{format!(" · {}",item.location)});
            if let Some(r)=rule(format!("widget-{}-{}",widget.id,item.id),text,item.at.clone(),&item.repeat,item.notify,item.done,&item.completed_on,now){result.push(r);}
        }}}
    }
    result
}
#[cfg(test)] mod tests {
 use super::*;use serde_json::json;
 fn layout()->Layout{serde_json::from_value(json!({"version":2,"widgets":[{"id":"one","kind":"notes","theme":"ivory","scale":0.5,"x":0.,"y":0.,"enabled":false,"city":null,"note":{"title":"待办","mode":"todo","text":"","items":[{"id":"first","text":"读书","done":false,"date":"2026-10-19","time":"10:00","notify":true,"repeat":"daily"}]}}]})).unwrap()}
 #[test]fn hidden_cards_schedule_and_revision_is_stable(){let l=layout();let now=Local.with_ymd_and_hms(2026,10,19,10,0,0).unwrap();let a=rules(&l,now);assert_eq!(a.len(),1);assert_eq!(a[0].revision,rules(&l,now)[0].revision);let mut edited=l;edited.widgets[0].note.as_mut().unwrap().items[0].text="写作".into();assert_ne!(a[0].revision,rules(&edited,now)[0].revision);}
 #[test]fn completion_only_skips_the_current_recurring_day(){let mut l=layout();l.widgets[0].note.as_mut().unwrap().items[0].completed_on="2026-10-19".into();assert!(rules(&l,Local.with_ymd_and_hms(2026,10,19,10,0,0).unwrap()).is_empty());assert_eq!(rules(&l,Local.with_ymd_and_hms(2026,10,20,10,0,0).unwrap()).len(),1);}
 #[test]fn legacy_timed_todos_do_not_implicitly_enable_notifications(){let mut l=layout();l.widgets[0].note.as_mut().unwrap().items[0].notify=false;assert!(rules(&l,Local::now()).is_empty());}
}
