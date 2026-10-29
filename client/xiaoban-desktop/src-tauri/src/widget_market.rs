//! Bounded public chart transport and parsers. Quotes keep the source trading timestamp.
use serde_json::{json,Value};
use std::io::Read;

fn number(value:Option<&Value>)->Option<f64>{value.and_then(|v|v.as_f64().or_else(||v.as_str()?.parse().ok())).filter(|v|v.is_finite())}
fn provider_symbol(symbol:&str)->Result<String,String>{
 let (market,code)=symbol.split_once('.').ok_or("证券代码无效")?;
 let prefix=match market{"116" if code.len()==5=>"hk","1" if code.len()==6=>"sh","0" if code.len()==6=>"sz",_=>return Err("证券代码无效".into())};
 if !code.bytes().all(|v|v.is_ascii_digit()){return Err("证券代码无效".into());}Ok(format!("{prefix}{code}"))
}
fn day(value:&str)->Option<String>{
 let format=if value.len()==8{"%Y%m%d"}else{"%Y-%m-%d"};
 chrono::NaiveDate::parse_from_str(value,format).ok().map(|d|d.format("%Y-%m-%d").to_string())
}
fn minute_rows(data:&Value,values:&mut Vec<(String,f64)>){
 let Some(date)=data.get("date").and_then(Value::as_str).and_then(day)else{return;};
 if let Some(rows)=data.get("data").and_then(Value::as_array){for row in rows {
  let Some(row)=row.as_str()else{continue;};let mut cols=row.split_whitespace();
  let Some(time)=cols.next().and_then(|s|chrono::NaiveTime::parse_from_str(s,"%H%M").ok())else{continue;};
  let Some(price)=cols.next().and_then(|s|s.parse::<f64>().ok()).filter(|v|v.is_finite()&&*v>0.)else{continue;};
  values.push((format!("{date} {}",time.format("%H:%M")),price));
 }}
}
fn parse(value:&Value,symbol:&str,period:&str)->Result<Value,String>{
 let data=value.get("data").and_then(|v|v.get(symbol)).ok_or("行情服务暂未返回数据")?;
 let mut values=Vec::new();
 match period{
  "1D"=>{if let Some(rows)=data.get("data"){minute_rows(rows,&mut values);}},
  "5D"=>{if let Some(days)=data.get("data").and_then(Value::as_array){for rows in days{minute_rows(rows,&mut values);}}},
  "1M"|"1Y"=>{if let Some(rows)=data.get("qfqday").or_else(||data.get("day")).and_then(Value::as_array){for row in rows{
   if let (Some(date),Some(price))=(row.get(0).and_then(Value::as_str).and_then(day),number(row.get(2)).filter(|v|*v>0.)){values.push((date,price));}
  }}},_=>return Err("不支持的行情周期".into())
 }
 values.sort_by(|a,b|a.0.cmp(&b.0));values.dedup_by(|a,b|a.0==b.0);
 let limit=match period{"1M"=>23,"1Y"=>251,_=>usize::MAX};if values.len()>limit{values.drain(..values.len()-limit);}
 let last=values.last().ok_or("行情序列不可用")?;let price=last.1;
 let quote=data.get("qt").and_then(|v|v.get(symbol));
 let prev=number(quote.and_then(|q|q.get(4))).filter(|v|*v>0.).or_else(||if values.len()>1&&["1M","1Y"].contains(&period){Some(values[values.len()-2].1)}else{None}).ok_or("行情缺少昨收盘价")?;
 let step=((values.len()+119)/120).max(1);let mut points:Vec<f64>=values.iter().step_by(step).map(|v|v.1).collect();if points.last()!=Some(&price){points.push(price);}
 Ok(json!({"price":price,"change":price-prev,"percent":100.*(price-prev)/prev,"points":points,"updatedAt":last.0,"source":"腾讯财经公开数据","name":quote.and_then(|q|q.get(1)).and_then(Value::as_str).unwrap_or(symbol)}))
}
pub fn fetch(symbol:&str,period:&str)->Result<Value,String>{
 let code=provider_symbol(symbol)?;
 let url=match period{
  "1D"=>format!("https://web.ifzq.gtimg.cn/appstock/app/minute/query?code={code}"),
  "5D"=>format!("https://web.ifzq.gtimg.cn/appstock/app/day/query?code={code}"),
  "1M"|"1Y"=>format!("https://web.ifzq.gtimg.cn/appstock/app/fqkline/get?param={code},day,,,251,qfq"),
  _=>return Err("不支持的行情周期".into())
 };
 let client=reqwest::blocking::Client::builder().timeout(std::time::Duration::from_secs(12)).redirect(reqwest::redirect::Policy::none()).user_agent("Mozilla/5.0 XiaoBan Desktop").build().map_err(|e|e.to_string())?;
 let response=client.get(url).send().map_err(|e|e.to_string())?.error_for_status().map_err(|e|e.to_string())?;
 let mut bytes=Vec::new();response.take(512*1024+1).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
 if bytes.len()>512*1024{return Err("行情响应过大".into());}
 let value=serde_json::from_slice(&bytes).map_err(|_|"行情服务返回格式无效")?;parse(&value,&code,period)
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn minute_data_preserves_actual_time_and_price(){let value=json!({"data":{"hk00700":{"data":{"date":"20261016","data":["0930 424.000 100","bad - 100","1600 417.800 300"]},"qt":{"hk00700":["100","腾讯控股","00700","417.800","417.800"]}}}});let q=parse(&value,"hk00700","1D").unwrap();assert_eq!(q["price"],417.8);assert_eq!(q["updatedAt"],"2026-10-16 16:00");assert_eq!(q["points"].as_array().unwrap().len(),2);assert_eq!(q["percent"],0.);}
 #[test]fn five_days_are_sorted_by_trading_time(){let value=json!({"data":{"sh000001":{"data":[{"date":"20261016","data":["0930 3002 100"]},{"date":"20261015","data":["0930 3001 100"]}],"qt":{"sh000001":["1","上证指数","000001","3002","3000"]}}}});let q=parse(&value,"sh000001","5D").unwrap();assert_eq!(q["points"],json!([3001.,3002.]));}
 #[test]fn candles_support_adjusted_data_and_reject_empty_prices(){let value=json!({"data":{"sz000001":{"qfqday":[["2026-10-15","10","11"],["2026-10-16","11","12"]]}}});let q=parse(&value,"sz000001","1M").unwrap();assert_eq!(q["price"],12.);assert_eq!(q["change"],1.);assert!(parse(&json!({}),"sz000001","1M").is_err());}
 #[test]fn symbol_translation_cannot_inject_query_parameters(){assert_eq!(provider_symbol("116.00700").unwrap(),"hk00700");assert_eq!(provider_symbol("1.000001").unwrap(),"sh000001");assert!(provider_symbol("116.00&x").is_err());}
}
