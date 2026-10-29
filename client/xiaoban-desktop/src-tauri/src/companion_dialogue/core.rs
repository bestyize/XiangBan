//! Local conversation policy. No windows, models, tools or operating-system access.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Turn { pub role: String, pub text: String, #[serde(default)] pub interaction: u64 }
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Choice { pub id: String, pub label: String }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reply {
    pub id: u64, pub pet_name: String, pub text: String, pub choices: Vec<Choice>, pub model: bool,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Record {
    pub count: u64, pub last_at: i64, pub history: Vec<Turn>, pub recent: Vec<String>,
    pub current: Option<Reply>, pub personality: String, pub session: Option<(u64, String)>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Data { pub next_id: u64, pub records: BTreeMap<String, Record> }
impl Default for Data { fn default() -> Self { Self { next_id: 1, records: BTreeMap::new() } } }

#[derive(Deserialize)]
pub struct Corpus { pub scenes: BTreeMap<String, Vec<String>> }
impl Corpus {
    pub fn bundled() -> Self { serde_json::from_str(include_str!("../../../src/companion-dialogue/corpus.json")).expect("checked companion corpus") }
}

fn options(scene: &str) -> Vec<Choice> {
    let values: &[(&str, &str)] = match scene {
        "tired" | "worry" => &[("quiet", "陪我待会儿"), ("encourage", "给我打打气")],
        "busy" | "plan" => &[("plan", "先做一件小事"), ("rest", "先歇一小会儿")],
        "happy" | "play" => &[("happy", "有开心的事"), ("curious", "聊点有趣的")],
        "night" | "sleep" => &[("sleep", "准备休息了"), ("quiet", "再陪我一会儿")],
        "quiet" => &[("quiet", "就这样待着"), ("curious", "换个轻松话题")],
        "curious" | "bored" => &[("play", "来点小乐趣"), ("plan", "找件小事做")],
        _ => &[("happy", "今天还不错"), ("tired", "有点累了")],
    };
    values.iter().map(|(id, label)| Choice { id: (*id).into(), label: (*label).into() }).collect()
}

impl Data {
    pub fn valid(&self) -> bool {
        self.next_id > 0 && self.records.len() <= 32 && self.records.iter().all(|(key, r)|
            key.len() <= 128 && r.history.len() <= 16 && r.recent.len() <= 40 &&
            r.history.iter().all(|t| matches!(t.role.as_str(), "user" | "assistant") && t.text.chars().count() <= 160) &&
            r.recent.iter().all(|id| id.len() <= 64) && r.personality.len() <= 32 &&
            r.current.as_ref().is_none_or(|v| v.id < self.next_id && v.pet_name.chars().count() <= 32 && v.text.chars().count() <= 160 &&
                v.choices.len() <= 2 && v.choices.iter().all(|c| c.id.len() <= 32 && c.label.chars().count() <= 32)))
    }
    pub fn interact(&mut self, corpus: &Corpus, key: &str, name: &str, personality: &str,
        now: i64, hour: u32, weekend: bool, choice: Option<&str>, expected: Option<u64>) -> Result<Reply, String> {
        if self.next_id == u64::MAX { return Err("互动记录已满".into()); }
        if !self.records.contains_key(key) && self.records.len() >= 32 {
            let oldest = self.records.iter().min_by_key(|(_, r)| r.last_at).map(|(k, _)| k.clone());
            if let Some(oldest) = oldest { self.records.remove(&oldest); }
        }
        let r = self.records.entry(key.into()).or_default();
        let user = if let Some(selected) = choice {
            let current = r.current.as_ref().filter(|v| Some(v.id) == expected).ok_or("这句话已经过去了，再点一下伙伴吧")?;
            current.choices.iter().find(|c| c.id == selected).map(|c| c.label.clone()).ok_or("没有这个选项")?
        } else { "摸摸头，和我说句话吧".into() };
        let gap = now.saturating_sub(r.last_at);
        let scene = if let Some(selected) = choice { selected }
            else if r.count == 0 { "first" }
            else if gap >= 6 * 3600 { "return" }
            else if gap < 4 { "repeat" }
            else if r.count % 3 == 0 { match hour { 5..=10 => "morning", 11..=16 => "afternoon", 17..=22 => "evening", _ => "night" } }
            else if weekend && r.count % 5 == 0 { "weekend" }
            else { ["touch", "curious", "encourage", "rest"][r.count as usize % 4] };
        let bank = corpus.scenes.get(scene).ok_or("伙伴语料暂不可用")?;
        if bank.is_empty() { return Err("伙伴语料暂不可用".into()); }
        let personality = if choice.is_some() && !r.personality.is_empty() { r.personality.clone() } else { personality.to_string() };
        let tone = match personality.as_str() { "playful" => 1, "quiet" => 2, "proud" => 3, _ => 0 };
        let start = (r.count as usize / 4 * 4 + tone) % bank.len();
        let index = (0..bank.len()).map(|offset| (start + offset * 4) % bank.len())
            .find(|i| !r.recent.contains(&format!("{scene}:{i}")))
            .or_else(|| (0..bank.len()).find(|i| !r.recent.contains(&format!("{scene}:{i}"))))
            .unwrap_or(start);
        let text = bank[index].replace("{name}", name);
        let reply = Reply { id: self.next_id, pet_name: name.chars().take(32).collect(), text,
            choices: options(scene), model: false };
        self.next_id += 1;
        r.count = r.count.saturating_add(1); r.last_at = now; r.personality = personality;
        r.recent.push(format!("{scene}:{index}"));
        if r.recent.len() > 40 { r.recent.remove(0); }
        r.history.push(Turn { role: "user".into(), text: user, interaction: reply.id });
        r.history.push(Turn { role: "assistant".into(), text: reply.text.clone(), interaction: reply.id });
        if r.history.len() > 16 { r.history.drain(..r.history.len() - 16); }
        r.current = Some(reply.clone());
        Ok(reply)
    }
    pub fn replace(&mut self, key: &str, id: u64, text: &str) -> Option<Reply> {
        let text = text.trim();
        if text.is_empty() || text.contains("<think>") || text.contains("```") || text.chars().count() > 120 { return None; }
        let record = self.records.get_mut(key)?;
        if record.history.iter().rev().skip(1).any(|t| t.role == "assistant" && t.text == text) { return None; }
        let reply = record.current.as_mut().filter(|r| r.id == id)?;
        reply.text = text.into(); reply.model = true;
        if let Some(last) = record.history.last_mut() { last.text = text.into(); }
        Some(reply.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn corpus_is_large_unique_and_short() {
        let corpus = Corpus::bundled();
        let lines: Vec<_> = corpus.scenes.values().flatten().collect();
        assert!(lines.len() >= 320);
        assert!(lines.iter().all(|s| !s.trim().is_empty() && s.chars().count() <= 100));
        assert_eq!(lines.len(), lines.iter().collect::<std::collections::BTreeSet<_>>().len());
    }
    #[test] fn choices_remember_context_and_stale_replies_cannot_replace_newer_ones() {
        let bank = Corpus::bundled(); let mut data = Data::default();
        let a = data.interact(&bank,"pet:a","小花","warm",100,9,false,None,None).unwrap();
        let b = data.interact(&bank,"pet:a","小花","warm",101,9,false,Some("tired"),Some(a.id)).unwrap();
        assert!(data.replace("pet:a",a.id,"迟到的回答").is_none());
        assert!(data.replace("pet:a",b.id,"先把肩膀放松一点，我陪你歇一会儿。").is_some());
        assert!(data.interact(&bank,"pet:a","小花","warm",102,9,false,Some("invalid"),Some(b.id)).is_err());
        assert_eq!(data.records["pet:a"].history[2].text,"有点累了");
        assert!(data.valid());
    }
    #[test] fn repeated_touches_rotate_and_history_is_bounded() {
        let bank = Corpus::bundled(); let mut data = Data::default(); let mut seen = std::collections::BTreeSet::new();
        for at in 100..114 {
            let reply = data.interact(&bank,"p","小伴","warm",at,9,false,None,None).unwrap();
            assert!(seen.insert(reply.text));
        }
        for at in 114..300 { data.interact(&bank,"p","小伴","warm",at,9,false,None,None).unwrap(); }
        assert_eq!(data.records["p"].history.len(),16); assert!(data.valid());
        let restored: Data = serde_json::from_str(&serde_json::to_string(&data).unwrap()).unwrap();
        assert_eq!(restored.records["p"].count,200);
    }
    #[test] fn companion_instances_keep_separate_histories_and_renames_keep_identity() {
        let bank = Corpus::bundled(); let mut data = Data::default();
        let a = data.interact(&bank,"pet:a","小花","warm",100,9,false,None,None).unwrap();
        data.interact(&bank,"pet:a","小花","warm",110,9,false,Some("tired"),Some(a.id)).unwrap();
        data.interact(&bank,"pet:b","小花","playful",120,9,false,None,None).unwrap();
        assert_eq!(data.records["pet:b"].history.len(),2);
        assert_eq!(data.records["pet:a"].history.len(),4);
        let renamed = data.interact(&bank,"pet:a","小葵","warm",130,9,false,None,None).unwrap();
        assert_eq!(renamed.pet_name,"小葵"); assert_eq!(data.records["pet:a"].history.len(),6);
        assert_eq!(data.records.len(),2);
    }
}
