//! Optional weather transport; text reminders never leave the device.
//! Open-Meteo's free endpoint is opt-in for personal/non-commercial use only.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{io::Read, time::Duration};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct City {
    pub name: String,
    pub country: String,
    pub admin1: String,
    pub latitude: f64,
    pub longitude: f64,
}
impl City {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() || self.name.len() > 160 || !self.latitude.is_finite() || !self.longitude.is_finite()
            || !(-90.0..=90.0).contains(&self.latitude) || !(-180.0..=180.0).contains(&self.longitude) {
            return Err("请选择有效的天气城市".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Source {
    pub provider: String,
    pub forecast_url: String,
    pub geocoding_url: String,
}
impl Default for Source {
    fn default() -> Self {
        Self { provider: "disabled".into(), forecast_url: String::new(), geocoding_url: String::new() }
    }
}
pub fn endpoint(value: &str) -> Result<url::Url, String> {
    let parsed = url::Url::parse(value.trim()).map_err(|_| "天气来源需要有效的 HTTPS 地址")?;
    if parsed.scheme() != "https" || parsed.host_str().is_none() || !parsed.username().is_empty()
        || parsed.password().is_some() || parsed.query().is_some() || parsed.fragment().is_some() {
        return Err("天气来源只支持不含凭据、查询参数的 HTTPS API 地址".into());
    }
    Ok(parsed)
}
impl Source {
    pub fn validate(&self) -> Result<(), String> {
        match self.provider.as_str() {
            "disabled" | "open-meteo" => Ok(()),
            "custom" => { endpoint(&self.forecast_url)?; endpoint(&self.geocoding_url)?; Ok(()) },
            _ => Err("未知的天气来源".into()),
        }
    }
    fn url(&self, geocoding: bool) -> Result<url::Url, String> {
        self.validate()?;
        match self.provider.as_str() {
            "open-meteo" => endpoint(if geocoding { "https://geocoding-api.open-meteo.com/v1/search" } else { "https://api.open-meteo.com/v1/forecast" }),
            "custom" => endpoint(if geocoding { &self.geocoding_url } else { &self.forecast_url }),
            _ => Err("天气来源尚未开启".into()),
        }
    }
}
fn get(url: url::Url) -> Result<Value, String> {
    let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(12))
        .redirect(reqwest::redirect::Policy::none()).user_agent("XiaoBanDesktop/0.1 weather")
        .build().map_err(|e| e.to_string())?;
    let response = client.get(url).send().map_err(|e| format!("天气服务连接失败：{e}"))?
        .error_for_status().map_err(|e| format!("天气服务返回错误：{e}"))?;
    let mut bytes = Vec::new();
    response.take(256 * 1024 + 1).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if bytes.len() > 256 * 1024 { return Err("天气返回数据过大".into()); }
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| "天气服务未返回有效 JSON")?;
    if value.get("error").and_then(Value::as_bool) == Some(true) { return Err("天气服务拒绝了本次查询".into()); }
    Ok(value)
}
pub fn search(source: &Source, query: &str) -> Result<Vec<City>, String> {
    let query = query.trim();
    if query.chars().count() < 2 || query.len() > 160 { return Err("请至少输入两个字的城市名".into()); }
    let mut url = source.url(true)?;
    url.query_pairs_mut().append_pair("name", query).append_pair("count", "10").append_pair("language", "zh");
    let value = get(url)?;
    let results = value.get("results").and_then(Value::as_array).cloned().unwrap_or_default();
    Ok(results.iter().filter_map(|item| {
        let city = City { name: item.get("name")?.as_str()?.into(), country: item.get("country").and_then(Value::as_str).unwrap_or("").into(),
            admin1: item.get("admin1").and_then(Value::as_str).unwrap_or("").into(), latitude: item.get("latitude")?.as_f64()?, longitude: item.get("longitude")?.as_f64()? };
        city.validate().ok().map(|_| city)
    }).collect())
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Weather {
    pub city: City,
    pub text: String,
    pub updated_at: i64,
    pub model_at: i64,
    pub source: String,
}
pub fn fetch(source: &Source, city: &City, now: i64) -> Result<Weather, String> {
    city.validate()?;
    let mut url = source.url(false)?;
    url.query_pairs_mut().append_pair("latitude", &city.latitude.to_string()).append_pair("longitude", &city.longitude.to_string())
        .append_pair("current", "temperature_2m,weather_code").append_pair("daily", "precipitation_probability_max,temperature_2m_max,temperature_2m_min")
        .append_pair("forecast_days", "1").append_pair("timezone", "auto").append_pair("timeformat", "unixtime");
    let value = get(url)?;
    parse_weather(&value, source, city, now)
}
fn parse_weather(value: &Value, source: &Source, city: &City, now: i64) -> Result<Weather, String> {
    let current = value.get("current").ok_or("天气数据缺少当前状况")?;
    let model_at = current.get("time").and_then(Value::as_i64).ok_or("天气数据缺少更新时间")?;
    if now - model_at > 7200 || model_at - now > 1800 { return Err("天气数据已过期或时间无效，暂不发送天气提醒".into()); }
    let temperature = current.get("temperature_2m").and_then(Value::as_f64).filter(|n| n.is_finite() && (-100.0..=70.0).contains(n)).ok_or("天气温度数据无效")?;
    let code = current.get("weather_code").and_then(Value::as_u64).ok_or("天气状况数据无效")?;
    let description = match code { 0 => "晴", 1..=3 => "多云", 45 | 48 => "雾", 51..=67 | 80..=82 => "雨", 71..=77 | 85..=86 => "雪", 95..=99 => "雷雨", _ => return Err("天气状况代码不可识别".into()) };
    let mut text = format!("{}：{}，当前约 {:.0}°C（天气模型数据）。", city.name, description, temperature);
    if let Some(probability) = value.pointer("/daily/precipitation_probability_max/0").and_then(Value::as_f64).filter(|p| (0.0..=100.0).contains(p)) {
        text.push_str(&format!(" 今日最高降水概率 {:.0}%。", probability));
        if probability >= 50.0 { text.push_str(" 出门记得带伞。"); }
    }
    Ok(Weather { city: city.clone(), text, updated_at: now, model_at, source: if source.provider == "open-meteo" { "Open-Meteo · CC BY 4.0".into() } else { source.forecast_url.clone() } })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn weather_source_defaults_to_no_network() { assert!(Source::default().url(false).is_err()); }
    #[test] fn no_credentials_or_query_keys_in_source() {
        assert!(endpoint("https://example.com/weather").is_ok());
        for value in ["http://example.com", "https://key:secret@example.com", "https://example.com/?apikey=secret"] { assert!(endpoint(value).is_err()); }
    }
    fn city() -> City { City { name: "北京".into(), latitude: 39.9, longitude: 116.4, ..City::default() } }
    #[test] fn fixture_weather_requires_valid_fresh_model_and_never_fabricates_rain() {
        let fixture = serde_json::json!({"current":{"time":10000,"temperature_2m":22.4,"weather_code":1},"daily":{"precipitation_probability_max":[75]}});
        let source = Source { provider: "open-meteo".into(), ..Source::default() };
        let weather = parse_weather(&fixture,&source,&city(),10060).unwrap();
        assert!(weather.text.contains("多云")); assert!(weather.text.contains("75%")); assert!(weather.text.contains("天气模型")); assert!(weather.text.contains("带伞"));
        assert!(parse_weather(&fixture,&source,&city(),20000).is_err());
        let missing = serde_json::json!({"current":{"time":10000,"weather_code":1}});
        assert!(parse_weather(&missing,&source,&city(),10060).is_err());
        let no_forecast = serde_json::json!({"current":{"time":10000,"temperature_2m":22.4,"weather_code":0}});
        assert!(!parse_weather(&no_forecast,&source,&city(),10060).unwrap().text.contains("带伞"));
    }
}
