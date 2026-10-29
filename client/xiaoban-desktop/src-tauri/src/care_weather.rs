//! Weather transport for care reminders; text reminders never leave the device.
//! Open-Meteo is the default source. Its free endpoint is for personal/non-commercial use only,
//! and only the coordinates of the chosen city are sent to it.
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
        Self { provider: "open-meteo".into(), forecast_url: String::new(), geocoding_url: String::new() }
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
    let client = crate::network::blocking_builder(reqwest::blocking::Client::builder().timeout(Duration::from_secs(12))
        .redirect(reqwest::redirect::Policy::none()).user_agent("XiaoBanDesktop/0.1 weather"))?
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
/// CAMS model-based US AQI. Kept separate from the selected weather provider.
pub fn air_quality(city:&City,now:i64)->Result<Value,String>{
 city.validate()?;let mut url=endpoint("https://air-quality-api.open-meteo.com/v1/air-quality")?;
 url.query_pairs_mut().append_pair("latitude",&city.latitude.to_string()).append_pair("longitude",&city.longitude.to_string()).append_pair("current","us_aqi,pm2_5,pm10").append_pair("timeformat","unixtime");
 air_quality_values(&get(url)?,now)
}
fn air_quality_values(value:&Value,now:i64)->Result<Value,String>{
 let current=value.get("current").ok_or("空气质量服务缺少当前数据")?;
 let number=|key:&str|max_number(current.get(key));
 let aqi=number("us_aqi").ok_or("当前地点暂无空气质量数据")?;
 let model_at=current.get("time").and_then(Value::as_i64).ok_or("空气质量缺少数据时间")?;
 if now.saturating_sub(model_at)>8*3600||model_at.saturating_sub(now)>3600{return Err("空气质量数据已过期".into());}
 Ok(serde_json::json!({"aqi":aqi,"pm25":number("pm2_5"),"pm10":number("pm10"),"modelAt":model_at,"updatedAt":now,"standard":"US AQI","source":"Open-Meteo / CAMS 空气质量预报"}))
}
fn max_number(value:Option<&Value>)->Option<f64>{value.and_then(Value::as_f64).filter(|v|v.is_finite()&&*v>=0.)}
/// City-level location from the public IP address, used only to pre-fill the city field.
/// The IP address necessarily reaches the locator; nothing else is sent. Tried in order.
const LOCATORS: [&str; 2] = [
    "https://ipwho.is/?lang=zh-CN&fields=success,message,city,region,country,latitude,longitude",
    "https://api.ip.sb/geoip",
];
pub fn locate_by_ip() -> Result<City, String> {
    for locator in LOCATORS {
        let Ok(url) = url::Url::parse(locator) else { continue };
        if let Some(city) = get(url).ok().and_then(|value| city_from_ip(&value)) { return Ok(city); }
    }
    Err("暂时无法根据网络位置判断城市，请手动搜索".into())
}
fn city_from_ip(value: &Value) -> Option<City> {
    if value.get("success").and_then(Value::as_bool) == Some(false) { return None; }
    let text = |key: &str| value.get(key).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty());
    let name = text("city").or_else(|| text("region"))?;
    // "上海" / "上海市" would read as a repeat, so the region is kept only when it adds information.
    let region = text("region").filter(|r| !r.starts_with(name) && !name.starts_with(r)).unwrap_or("");
    let city = City { name: name.into(), country: text("country").unwrap_or("").into(), admin1: region.into(),
        latitude: value.get("latitude")?.as_f64()?, longitude: value.get("longitude")?.as_f64()? };
    city.validate().ok().map(|_| city)
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

/// Three-day forecast for a visible weather widget. Reuses the configured, validated weather source.
pub fn forecast(source:&Source,city:&City)->Result<Value,String>{
    city.validate()?;
    let mut url=source.url(false)?;
    url.query_pairs_mut().append_pair("latitude",&city.latitude.to_string())
        .append_pair("longitude",&city.longitude.to_string())
        .append_pair("daily","weather_code,temperature_2m_max,temperature_2m_min")
        .append_pair("forecast_days","3").append_pair("timezone","auto");
    let value=get(url)?;
    forecast_values(&value)
}
fn forecast_values(value:&Value)->Result<Value,String>{
    let dates=value.pointer("/daily/time").and_then(Value::as_array).ok_or("天气预报缺少日期")?;
    let highs=value.pointer("/daily/temperature_2m_max").and_then(Value::as_array).ok_or("天气预报缺少最高温度")?;
    let lows=value.pointer("/daily/temperature_2m_min").and_then(Value::as_array).ok_or("天气预报缺少最低温度")?;
    let codes=value.pointer("/daily/weather_code").and_then(Value::as_array).ok_or("天气预报缺少天气状况")?;
    let mut days=Vec::new();
    for i in 0..dates.len().min(3){
        let date=dates.get(i).and_then(Value::as_str).ok_or("预报日期无效")?;
        let hi=highs.get(i).and_then(Value::as_f64).ok_or("最高温度无效")?;
        let lo=lows.get(i).and_then(Value::as_f64).ok_or("最低温度无效")?;
        let code=codes.get(i).and_then(Value::as_u64).ok_or("天气代码无效")?;
        if chrono::NaiveDate::parse_from_str(date,"%Y-%m-%d").is_err() || !hi.is_finite() || !lo.is_finite() || !(-100.0..=70.0).contains(&hi) || !(-100.0..=70.0).contains(&lo) || hi<lo || code>99{return Err("天气预报内容无效".into());}
        days.push(serde_json::json!({"date":date,"high":hi.round(),"low":lo.round(),"code":code}));
    }
    if days.is_empty(){return Err("暂无可用三日预报".into());}
    Ok(serde_json::json!({"days":days}))
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
    #[test] fn air_quality_rejects_missing_and_stale_values(){
        let valid=serde_json::json!({"current":{"time":1000,"us_aqi":42.,"pm2_5":9.,"pm10":20.}});
        assert_eq!(air_quality_values(&valid,1100).unwrap()["standard"],"US AQI");
        assert!(air_quality_values(&serde_json::json!({"current":{"time":1000,"us_aqi":null}}),1100).is_err());
        assert!(air_quality_values(&valid,100000).is_err());
    }
    use super::*;
    #[test] fn reads_three_day_forecast_without_fabricating_missing_values(){
        let raw=serde_json::json!({"daily":{"time":["2026-10-19","2026-10-20","2026-10-21"],"temperature_2m_max":[26.2,25.9,20.0],"temperature_2m_min":[21.1,19.5,17.2],"weather_code":[1,2,3]}});
        assert_eq!(forecast_values(&raw).unwrap()["days"].as_array().unwrap().len(),3);
        assert!(forecast_values(&serde_json::json!({"daily":{"time":["2026-10-19"]}})).is_err());
    }
    #[test] fn weather_source_defaults_to_open_meteo() {
        let source = Source::default();
        assert_eq!(source.provider, "open-meteo"); assert!(source.validate().is_ok());
        assert_eq!(source.url(false).unwrap().host_str(), Some("api.open-meteo.com"));
        assert_eq!(source.url(true).unwrap().host_str(), Some("geocoding-api.open-meteo.com"));
        let off = Source { provider: "disabled".into(), ..Source::default() };
        assert!(off.url(false).is_err());
    }
    #[test] fn ip_location_becomes_a_valid_city_and_rejects_unusable_answers() {
        let shanghai = serde_json::json!({"success":true,"country":"中国","region":"上海市","city":"上海","latitude":31.22,"longitude":121.45});
        let city = city_from_ip(&shanghai).unwrap();
        assert_eq!((city.name.as_str(), city.admin1.as_str(), city.country.as_str()), ("上海", "", "中国"));
        let hangzhou = serde_json::json!({"region":"浙江省","city":"杭州市","country":"中国","latitude":30.27,"longitude":120.15});
        assert_eq!(city_from_ip(&hangzhou).unwrap().admin1, "浙江省");
        let region_only = serde_json::json!({"region":"Zhejiang","latitude":30.0,"longitude":120.0});
        assert_eq!(city_from_ip(&region_only).unwrap().name, "Zhejiang");
        for bad in [serde_json::json!({"success":false,"message":"limit"}), serde_json::json!({"city":"X"}),
            serde_json::json!({"city":"X","latitude":123.0,"longitude":10.0}), serde_json::json!({"city":"  ","latitude":1.0,"longitude":1.0})] {
            assert!(city_from_ip(&bad).is_none(), "{bad}");
        }
    }
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
