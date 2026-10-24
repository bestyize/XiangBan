//! What a widget shows, as the assistant sets it: 待办 items, 纪念日, 今日日程 and 股票 symbols.
//! `patch` turns the tool's friendly `content` into the typed content the widget layout stores (which
//! validates it again before saving); `summary` lists what a widget holds so an item can be changed by id.
use serde_json::{json, Map, Value};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

const TODO_MAX: usize = 12;
const REMINDER_MAX: usize = 12;
const SYMBOL_MAX: usize = 8;

fn units(text: &str) -> usize { text.encode_utf16().count() }

fn text_of<'a>(obj: &'a Map<String, Value>, key: &str) -> Result<Option<&'a str>, String> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.trim())),
        Some(_) => Err(format!("`{key}` 必须是字符串")),
    }
}

fn bool_of(obj: &Map<String, Value>, key: &str) -> Result<Option<bool>, String> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(b)) => Ok(Some(*b)),
        Some(_) => Err(format!("`{key}` 必须是 true 或 false")),
    }
}

fn date_ok(value: &str) -> bool {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok_and(|d| d.format("%Y-%m-%d").to_string() == value && ("1900-01-01"..="2100-12-31").contains(&value))
}
fn time_ok(value: &str) -> bool {
    chrono::NaiveTime::parse_from_str(value, "%H:%M").is_ok_and(|t| t.format("%H:%M").to_string() == value)
}
fn datetime_ok(value: &str) -> bool {
    chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M").is_ok_and(|d| d.format("%Y-%m-%dT%H:%M").to_string() == value)
}

/// A lowercase id that is not yet used in this widget.
fn fresh_id(taken: &HashSet<String>, prefix: &str, n: usize) -> String {
    let millis = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or_default();
    (0..).map(|salt| format!("{prefix}{millis}{n}{}", if salt == 0 { String::new() } else { salt.to_string() })).find(|id| !taken.contains(id)).unwrap_or_default()
}

fn items_of(old: Option<&Value>, key: &str) -> Vec<Value> {
    old.and_then(|v| v.get(key)).and_then(Value::as_array).cloned().unwrap_or_default()
}

fn ids_of(items: &[Value]) -> HashSet<String> {
    items.iter().filter_map(|i| i["id"].as_str().map(str::to_string)).collect()
}

/// `items` replaces the list, or adds to it with `append`. Leaving `items` out keeps the list.
fn merged(old: Vec<Value>, given: Option<&Value>, append: bool, build: impl Fn(&Value, &HashSet<String>, usize) -> Result<Value, String>) -> Result<Vec<Value>, String> {
    let Some(given) = given.filter(|v| !v.is_null()) else { return Ok(old) };
    let given = given.as_array().ok_or("items 必须是数组")?;
    let mut items = if append { old } else { Vec::new() };
    for (n, raw) in given.iter().enumerate() {
        let mut taken = ids_of(&items);
        taken.extend(given.iter().filter_map(|g| g["id"].as_str().map(str::to_string)));
        items.push(build(raw, &taken, n)?);
    }
    Ok(items)
}

fn one_of(value: Option<&str>, allowed: &[&str], default: &str, label: &str) -> Result<String, String> {
    let value = value.unwrap_or(default);
    if allowed.contains(&value) { Ok(value.to_string()) } else { Err(format!("{label} 只能是 {}", allowed.join("、"))) }
}

fn schedule_ok(notify: bool, repeat: &str, date: &str, time: &str) -> Result<(), String> {
    if (notify || repeat != "none") && !(date_ok(date) && time_ok(time)) {
        return Err("要到点提醒或重复，必须同时给 date（YYYY-MM-DD）和 time（HH:MM）".into());
    }
    Ok(())
}

fn todo_item(raw: &Value, taken: &HashSet<String>, n: usize) -> Result<Value, String> {
    let obj = match raw {
        Value::String(text) => { let mut m = Map::new(); m.insert("text".into(), json!(text)); m }
        Value::Object(m) => m.clone(),
        _ => return Err("待办的每一项要么是文字，要么是带 text 的对象".into()),
    };
    if let Some(bad) = obj.keys().find(|k| !["id", "text", "done", "date", "time", "notify", "repeat"].contains(&k.as_str())) {
        return Err(format!("待办项不支持 `{bad}`，可用：text、done、date、time、notify、repeat、id"));
    }
    let text = text_of(&obj, "text")?.filter(|t| !t.is_empty()).ok_or("每条待办需要 text")?;
    if units(text) > 120 { return Err(format!("待办「{text}」超过 120 字")); }
    let date = text_of(&obj, "date")?.unwrap_or("");
    let time = text_of(&obj, "time")?.unwrap_or("");
    if (!date.is_empty() && !date_ok(date)) || (!time.is_empty() && !time_ok(time)) { return Err("待办的 date 要是 YYYY-MM-DD，time 要是 HH:MM（24 小时制）".into()); }
    let notify = bool_of(&obj, "notify")?.unwrap_or(false);
    let repeat = one_of(text_of(&obj, "repeat")?, &["none", "daily", "weekdays", "weekly"], "none", "repeat")?;
    schedule_ok(notify, &repeat, date, time)?;
    let id = text_of(&obj, "id")?.filter(|id| !id.is_empty()).map(str::to_string).unwrap_or_else(|| fresh_id(taken, "t", n));
    Ok(json!({"id": id, "text": text, "done": bool_of(&obj, "done")?.unwrap_or(false), "time": time, "date": date, "notify": notify, "repeat": repeat, "completedOn": ""}))
}

fn reminder_item(raw: &Value, taken: &HashSet<String>, n: usize) -> Result<Value, String> {
    let obj = raw.as_object().ok_or("日程的每一项要是带 title 和 at 的对象")?;
    if let Some(bad) = obj.keys().find(|k| !["id", "title", "at", "end", "location", "notify", "repeat", "done"].contains(&k.as_str())) {
        return Err(format!("日程项不支持 `{bad}`，可用：title、at、end、location、notify、repeat、done、id"));
    }
    let title = text_of(obj, "title")?.filter(|t| !t.is_empty()).ok_or("每条日程需要 title")?;
    if units(title) > 60 { return Err(format!("日程「{title}」超过 60 字")); }
    let at = text_of(obj, "at")?.filter(|a| datetime_ok(a)).ok_or("每条日程需要 at，格式 YYYY-MM-DDTHH:MM（本地时间）")?;
    let end = text_of(obj, "end")?.unwrap_or("");
    if !end.is_empty() && (!time_ok(end) || end <= &at[11..]) { return Err("end 要是 HH:MM，且晚于开始时间".into()); }
    let location = text_of(obj, "location")?.unwrap_or("");
    if units(location) > 80 { return Err("location 最多 80 字".into()); }
    let notify = bool_of(obj, "notify")?.unwrap_or(true);
    let repeat = one_of(text_of(obj, "repeat")?, &["none", "daily", "weekdays", "weekly"], "none", "repeat")?;
    schedule_ok(notify, &repeat, &at[..10], &at[11..])?;
    let id = text_of(obj, "id")?.filter(|id| !id.is_empty()).map(str::to_string).unwrap_or_else(|| fresh_id(taken, "r", n));
    Ok(json!({"id": id, "title": title, "at": at, "done": bool_of(obj, "done")?.unwrap_or(false), "end": end, "location": location, "notify": notify, "repeat": repeat, "completedOn": ""}))
}

fn notes(widget: &Value, obj: &Map<String, Value>) -> Result<Value, String> {
    let old = widget.get("note");
    let title = text_of(obj, "title")?.map(str::to_string).or_else(|| old.and_then(|n| n["title"].as_str()).map(str::to_string)).unwrap_or_else(|| "今日待办".into());
    if title.is_empty() || units(&title) > 40 { return Err("title 需要 1 到 40 字".into()); }
    let items = merged(items_of(old, "items"), obj.get("items"), bool_of(obj, "append")?.unwrap_or(false), todo_item)?;
    if items.len() > TODO_MAX { return Err(format!("待办最多 {TODO_MAX} 条，现在有 {} 条；换成 items 整体替换，或先去掉已完成的", items.len())); }
    let text = old.and_then(|n| n["text"].as_str()).unwrap_or("");
    Ok(json!({"note": {"title": title, "mode": "todo", "text": text, "items": items}}))
}

fn countdown(widget: &Value, obj: &Map<String, Value>) -> Result<Value, String> {
    let old = widget.get("countdown");
    let pick = |key: &str, default: &str| -> Result<String, String> {
        Ok(text_of(obj, key)?.map(str::to_string).or_else(|| old.and_then(|c| c[key].as_str()).map(str::to_string)).unwrap_or_else(|| default.into()))
    };
    let (title, date) = (pick("title", "值得期待的日子")?, pick("date", "")?);
    let mode = one_of(Some(&pick("mode", "countdown")?), &["countdown", "elapsed"], "countdown", "mode")?;
    let repeat = one_of(Some(&pick("repeat", "none")?), &["none", "yearly"], "none", "repeat")?;
    if title.is_empty() || units(&title) > 40 { return Err("title 需要 1 到 40 字".into()); }
    if !date_ok(&date) { return Err("date 要是 YYYY-MM-DD（1900 到 2100 年）".into()); }
    if mode == "elapsed" && repeat != "none" { return Err("已过去天数（elapsed）不能每年重复".into()); }
    Ok(json!({"countdown": {"title": title, "date": date, "mode": mode, "repeat": repeat}}))
}

/// The living data of 日程 / 股票 / 木鱼, keeping what the caller did not touch.
fn living(widget: &Value) -> Map<String, Value> {
    let old = widget.get("living");
    let mut data = Map::new();
    data.insert("reminders".into(), json!(items_of(old, "reminders")));
    data.insert("count".into(), old.and_then(|l| l["count"].as_u64()).map_or(json!(0), |c| json!(c)));
    data.insert("symbol".into(), old.and_then(|l| l["symbol"].as_str()).map_or(json!("1.000001"), |s| json!(s)));
    data.insert("symbols".into(), json!(items_of(old, "symbols")));
    data.insert("sound".into(), old.and_then(|l| l["sound"].as_bool()).map_or(json!(true), |b| json!(b)));
    data
}

fn reminders(widget: &Value, obj: &Map<String, Value>) -> Result<Value, String> {
    let mut data = living(widget);
    let old = widget.get("living");
    let items = merged(items_of(old, "reminders"), obj.get("items"), bool_of(obj, "append")?.unwrap_or(false), reminder_item)?;
    if items.len() > REMINDER_MAX { return Err(format!("日程最多 {REMINDER_MAX} 条，现在有 {} 条", items.len())); }
    data.insert("reminders".into(), json!(items));
    Ok(json!({"living": data}))
}

fn stocks(widget: &Value, obj: &Map<String, Value>) -> Result<Value, String> {
    let mut data = living(widget);
    let mut symbols: Vec<String> = data["symbols"].as_array().into_iter().flatten().filter_map(|s| s.as_str().map(str::to_string)).collect();
    if let Some(given) = obj.get("symbols").filter(|v| !v.is_null()) {
        symbols = given.as_array().ok_or("symbols 必须是数组")?.iter().map(|s| s.as_str().map(|s| s.trim().to_string()).ok_or("symbols 里只能是字符串")).collect::<Result<_, _>>()?;
        symbols.dedup();
    }
    if symbols.len() > SYMBOL_MAX { return Err(format!("股票最多 {SYMBOL_MAX} 只")); }
    if let Some(bad) = symbols.iter().find(|s| !crate::widgets::valid_stock_symbol(s)) {
        return Err(format!("`{bad}` 不是有效的股票代码：沪市 1.600519、深市 0.000001、港股 116.00700，格式是 市场代码.证券代码"));
    }
    let current = match text_of(obj, "symbol")? {
        Some(symbol) => symbol.to_string(),
        None => data["symbol"].as_str().filter(|s| symbols.is_empty() || symbols.iter().any(|x| x == s)).map(str::to_string).or_else(|| symbols.first().cloned()).unwrap_or_else(|| "1.000001".into()),
    };
    if !crate::widgets::valid_stock_symbol(&current) { return Err(format!("`{current}` 不是有效的股票代码")); }
    if !symbols.is_empty() && !symbols.contains(&current) { return Err("symbol 必须在 symbols 里".into()); }
    data.insert("symbols".into(), json!(symbols));
    data.insert("symbol".into(), json!(current));
    Ok(json!({"living": data}))
}

/// `widget` is the widget as `widget_list` sees it; the result is a patch with one of `note`, `countdown`, `living`.
pub(super) fn patch(widget: &Value, content: &Value) -> Result<Value, String> {
    let obj = content.as_object().ok_or("content 需要是一个对象")?;
    let kind = widget["kind"].as_str().unwrap_or_default();
    let allowed: &[&str] = match kind {
        "notes" => &["title", "items", "append"],
        "countdown" => &["title", "date", "mode", "repeat"],
        "reminders" => &["items", "append"],
        "stocks" => &["symbols", "symbol"],
        _ => return Err("只有待办（notes）、纪念日（countdown）、今日日程（reminders）、股票（stocks）组件有可设置的内容".into()),
    };
    if let Some(bad) = obj.keys().find(|k| !allowed.contains(&k.as_str())) {
        return Err(format!("{kind} 组件的 content 不支持 `{bad}`，可用：{}", allowed.join("、")));
    }
    if obj.is_empty() { return Err(format!("content 里至少给一项：{}", allowed.join("、"))); }
    match kind {
        "notes" => notes(widget, obj),
        "countdown" => countdown(widget, obj),
        "reminders" => reminders(widget, obj),
        _ => stocks(widget, obj),
    }
}

/// One line per thing a widget holds, with the ids an update can refer to.
pub(super) fn summary(widget: &Value) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(note) = widget.get("note") {
        lines.push(format!("  待办「{}」：", note["title"].as_str().unwrap_or("")));
        for item in note["items"].as_array().into_iter().flatten() {
            let when = [item["date"].as_str().unwrap_or(""), item["time"].as_str().unwrap_or("")].join(" ");
            lines.push(format!("  - {} [{}] {}{}", if item["done"].as_bool() == Some(true) { "已完成" } else { "未完成" }, item["id"].as_str().unwrap_or("?"), item["text"].as_str().unwrap_or(""), if when.trim().is_empty() { String::new() } else { format!("（{}）", when.trim()) }));
        }
    }
    if let Some(c) = widget.get("countdown") {
        lines.push(format!("  纪念日「{}」{} {}{}", c["title"].as_str().unwrap_or(""), c["date"].as_str().unwrap_or(""), if c["mode"] == "elapsed" { "已过去" } else { "倒数" }, if c["repeat"] == "yearly" { "，每年重复" } else { "" }));
    }
    if let Some(living) = widget.get("living") {
        match widget["kind"].as_str() {
            Some("reminders") => for r in living["reminders"].as_array().into_iter().flatten() {
                lines.push(format!("  - {} [{}] {} {}", if r["done"].as_bool() == Some(true) { "已完成" } else { "未完成" }, r["id"].as_str().unwrap_or("?"), r["at"].as_str().unwrap_or(""), r["title"].as_str().unwrap_or("")));
            },
            Some("stocks") => {
                let symbols: Vec<&str> = living["symbols"].as_array().into_iter().flatten().filter_map(Value::as_str).collect();
                lines.push(format!("  股票：当前 {}，自选 {}", living["symbol"].as_str().unwrap_or(""), if symbols.is_empty() { "（未设置）".to_string() } else { symbols.join("、") }));
            }
            _ => {}
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notes_widget() -> Value {
        json!({"id": "w1", "kind": "notes", "note": {"title": "今日待办", "mode": "todo", "text": "", "items": [
            {"id": "a1", "text": "旧事", "done": false, "time": "", "date": "", "notify": false, "repeat": "none", "completedOn": ""}]}})
    }

    #[test]
    fn todo_items_are_added_replaced_and_marked_done_by_id() {
        let added = patch(&notes_widget(), &json!({"items": ["买花", {"text": "开会", "date": "2026-10-30", "time": "09:00", "notify": true}], "append": true})).unwrap();
        let items = added["note"]["items"].as_array().unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0]["id"], "a1");
        assert_eq!(items[1]["text"], "买花");
        assert_eq!(items[2]["notify"], true);
        assert_ne!(items[1]["id"], items[2]["id"]);
        // The ids are ones the layout accepts: lowercase letters and digits, at most 32.
        assert!(items.iter().all(|i| { let id = i["id"].as_str().unwrap(); id.len() <= 32 && id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit()) }));

        let replaced = patch(&notes_widget(), &json!({"items": [{"id": "a1", "text": "旧事", "done": true}]})).unwrap();
        assert_eq!(replaced["note"]["items"][0]["done"], true);
        assert_eq!(patch(&notes_widget(), &json!({"title": "周末"})).unwrap()["note"]["items"].as_array().unwrap().len(), 1, "a title change keeps the list");
    }

    #[test]
    fn todo_rules_are_explained_before_the_layout_refuses_them() {
        let w = notes_widget();
        assert!(patch(&w, &json!({"items": [{"text": "x", "notify": true}]})).unwrap_err().contains("date"));
        assert!(patch(&w, &json!({"items": [{"text": "x", "date": "10月1日"}]})).unwrap_err().contains("YYYY-MM-DD"));
        assert!(patch(&w, &json!({"items": [{"text": "x", "repeat": "monthly"}]})).unwrap_err().contains("repeat"));
        assert!(patch(&w, &json!({"items": [{"title": "x"}]})).unwrap_err().contains("不支持"));
        let many: Vec<Value> = (0..13).map(|i| json!(format!("事{i}"))).collect();
        assert!(patch(&w, &json!({"items": many})).unwrap_err().contains("最多 12"));
        assert!(patch(&w, &json!({"items": ["x".repeat(121)]})).unwrap_err().contains("120"));
        assert!(patch(&w, &json!({})).unwrap_err().contains("至少给一项"));
        assert!(patch(&w, &json!({"date": "2026-10-01"})).unwrap_err().contains("不支持"));
    }

    #[test]
    fn countdown_keeps_what_was_not_given_and_checks_the_date() {
        let w = json!({"id": "w2", "kind": "countdown", "countdown": {"title": "国庆", "date": "2026-10-01", "mode": "countdown", "repeat": "none"}});
        let moved = patch(&w, &json!({"date": "2027-10-01", "repeat": "yearly"})).unwrap();
        assert_eq!(moved["countdown"], json!({"title": "国庆", "date": "2027-10-01", "mode": "countdown", "repeat": "yearly"}));
        assert!(patch(&w, &json!({"date": "明天"})).is_err());
        assert!(patch(&w, &json!({"mode": "elapsed", "repeat": "yearly"})).unwrap_err().contains("elapsed"));
        assert!(patch(&w, &json!({"mode": "later"})).unwrap_err().contains("mode"));
    }

    #[test]
    fn schedule_items_need_a_start_and_keep_the_rest_of_the_living_data() {
        let w = json!({"id": "w3", "kind": "reminders", "living": {"reminders": [], "count": 5, "symbol": "1.000001", "symbols": [], "sound": false}});
        let out = patch(&w, &json!({"items": [{"title": "开会", "at": "2026-10-30T15:00", "end": "16:00", "location": "三楼"}]})).unwrap();
        assert_eq!(out["living"]["reminders"][0]["title"], "开会");
        assert_eq!(out["living"]["count"], 5);
        assert_eq!(out["living"]["sound"], false);
        assert!(patch(&w, &json!({"items": [{"title": "开会"}]})).unwrap_err().contains("at"));
        assert!(patch(&w, &json!({"items": [{"title": "开会", "at": "2026-10-30T15:00", "end": "14:00"}]})).unwrap_err().contains("end"));
    }

    #[test]
    fn stock_symbols_are_checked_and_the_current_one_follows_the_list() {
        let w = json!({"id": "w4", "kind": "stocks", "living": {"reminders": [], "count": 0, "symbol": "116.00700", "symbols": [], "sound": true}});
        let out = patch(&w, &json!({"symbols": ["1.600519", "0.000001"]})).unwrap();
        assert_eq!(out["living"]["symbols"], json!(["1.600519", "0.000001"]));
        assert_eq!(out["living"]["symbol"], "1.600519", "the old current symbol is not in the new list");
        assert_eq!(patch(&w, &json!({"symbols": ["1.600519", "0.000001"], "symbol": "0.000001"})).unwrap()["living"]["symbol"], "0.000001");
        assert!(patch(&w, &json!({"symbols": ["600519"]})).unwrap_err().contains("市场代码"));
        assert!(patch(&w, &json!({"symbols": ["1.600519"], "symbol": "0.000001"})).unwrap_err().contains("symbols"));
    }

    #[test]
    fn other_widgets_have_no_content_and_the_list_shows_ids() {
        assert!(patch(&json!({"id": "w5", "kind": "clock"}), &json!({"title": "x"})).unwrap_err().contains("notes"));
        let lines = summary(&notes_widget());
        assert!(lines.iter().any(|l| l.contains("[a1]") && l.contains("未完成") && l.contains("旧事")));
    }
}
