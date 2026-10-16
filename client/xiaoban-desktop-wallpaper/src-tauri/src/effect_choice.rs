//! Which particle effect covers the desktop, and the strength, speed and size the user gave it.
//! The choice belongs to the desktop, not to a wallpaper.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// An effect pack id: small letters, digits, `-` and `_`.
pub fn valid_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    (1..=48).contains(&bytes.len())
        && (bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
        && bytes.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, b'-' | b'_'))
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectParams {
    pub density: f64,
    pub speed: f64,
    pub size: f64,
}
impl Default for EffectParams {
    fn default() -> Self { Self { density: 1.0, speed: 1.0, size: 1.0 } }
}
impl EffectParams {
    fn number(value: &Value, key: &str, fallback: f64, min: f64, max: f64) -> f64 {
        value[key].as_f64().filter(|n| n.is_finite()).map_or(fallback, |n| n.clamp(min, max))
    }
    pub fn from_value(value: &Value) -> Self {
        Self {
            density: Self::number(value, "density", 1.0, 0.2, 2.5),
            speed: Self::number(value, "speed", 1.0, 0.25, 3.0),
            size: Self::number(value, "size", 1.0, 0.5, 2.0),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectsChoice {
    /// The pack in use; empty means no effect.
    pub selected: String,
    /// Each pack keeps its own parameters, so switching back finds them as they were left.
    pub params: BTreeMap<String, EffectParams>,
}

/// The six scenes that earlier versions kept as named fields, in the order the old code gave them priority.
const LEGACY: [(&str, &str); 6] = [("rain", "rain"), ("snow", "snow"), ("fireflies", "fireflies"), ("petals", "petals"), ("leaves", "maple"), ("meteors", "meteors")];

impl EffectsChoice {
    /// Patch only the requested parameters; invalid values must not silently clamp a chat edit.
    pub fn adjust(&mut self, id: &str, patch: &Value) -> Result<EffectParams, String> {
        if !valid_id(id) { return Err("没有有效的特效 id".into()); }
        let object = patch.as_object().ok_or("参数必须是对象")?;
        if object.is_empty() { return Err("至少提供 density、speed、size 中的一项".into()); }
        let mut value = serde_json::to_value(self.params_of(id)).map_err(|e| e.to_string())?;
        for (key, number) in object {
            let (min, max) = match key.as_str() {
                "density" => (0.2, 2.5), "speed" => (0.25, 3.0), "size" => (0.5, 2.0),
                _ => return Err(format!("不支持的特效参数 {key}")),
            };
            let n = number.as_f64().filter(|n| n.is_finite() && (min..=max).contains(n))
                .ok_or_else(|| format!("{key} 必须在 {min} 到 {max} 之间"))?;
            value[key] = serde_json::json!(n);
        }
        let next = EffectParams::from_value(&value);
        self.params.insert(id.to_string(), next.clone());
        Ok(next)
    }

    /// Reads a choice leniently: anything missing or malformed falls back to a safe default.
    /// Also accepts the older shape `{rain:{enabled,intensity,speed}, ...}`, from this setting and from the
    /// per-wallpaper copy before it, and turns it into the current one.
    pub fn from_value(value: &Value) -> Self {
        let Some(object) = value.as_object() else { return Self::default() };
        if object.contains_key("selected") || object.contains_key("params") {
            let selected = object.get("selected").and_then(Value::as_str).filter(|id| valid_id(id)).unwrap_or("").to_string();
            let params = object.get("params").and_then(Value::as_object).map(|map| {
                map.iter().filter(|(id, _)| valid_id(id)).take(64).map(|(id, item)| (id.clone(), EffectParams::from_value(item))).collect()
            }).unwrap_or_default();
            return Self { selected, params };
        }
        let mut choice = Self::default();
        for (old, new) in LEGACY {
            let entry = &value[old];
            if !entry.is_object() { continue; }
            // The old strength 0.5 was the default; the new density 1 is, so the scale doubles.
            let intensity = entry["intensity"].as_f64().filter(|n| n.is_finite()).unwrap_or(0.5);
            let speed = entry["speed"].as_f64().filter(|n| n.is_finite()).unwrap_or(1.0);
            choice.params.insert(new.to_string(), EffectParams::from_value(&serde_json::json!({"density": intensity * 2.0, "speed": speed})));
            if choice.selected.is_empty() && entry["enabled"].as_bool() == Some(true) { choice.selected = new.to_string(); }
        }
        choice
    }
    pub fn params_of(&self, id: &str) -> EffectParams { self.params.get(id).cloned().unwrap_or_default() }
}
impl<'de> Deserialize<'de> for EffectsChoice {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from_value(&Value::deserialize(deserializer)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn ids_follow_the_pack_naming_rule() {
        for good in ["rain", "maple", "my-glitter_2", "0day"] { assert!(valid_id(good), "{good}"); }
        for bad in ["", "Rain", "-x", "_x", "a b", "雨", "a/b", &"a".repeat(49)] { assert!(!valid_id(bad), "{bad}"); }
    }
    #[test]
    fn chat_adjustment_preserves_selection_other_parameters_and_rejects_invalid_patches_atomically() {
        let mut choice = EffectsChoice::from_value(&json!({"selected":"snow","params":{
            "snow":{"density":1.2,"speed":0.7,"size":1.4},"rain":{"speed":2}
        }}));
        let result = choice.adjust("snow",&json!({"density":2})).unwrap();
        assert_eq!(result, EffectParams { density:2.0, speed:0.7, size:1.4 });
        assert_eq!(choice.selected,"snow");
        assert_eq!(choice.params_of("rain").speed,2.0);
        let before = choice.clone();
        for invalid in [json!({}),json!({"density":0}),json!({"size":3}),json!({"density":2.5,"speed":"fast"}),json!({"unknown":1})] {
            assert!(choice.adjust("snow",&invalid).is_err());
            assert_eq!(choice,before);
        }
        choice.adjust("rain",&json!({"speed":1.5})).unwrap();
        assert_eq!(choice.selected,"snow");
    }
    #[test]
    fn the_current_shape_is_read_and_clamped() {
        let choice = EffectsChoice::from_value(&json!({"selected":"petals","params":{
            "petals":{"density":9,"speed":0,"size":"big"}, "Bad Id":{"density":2}, "stars":{"size":1.5}}}));
        assert_eq!(choice.selected, "petals");
        assert_eq!(choice.params["petals"], EffectParams { density: 2.5, speed: 0.25, size: 1.0 });
        assert_eq!(choice.params["stars"], EffectParams { density: 1.0, speed: 1.0, size: 1.5 });
        assert!(!choice.params.contains_key("Bad Id"));
        assert_eq!(EffectsChoice::from_value(&json!({"selected":"No Good"})).selected, "");
    }
    #[test]
    fn anything_unreadable_means_no_effect() {
        for value in [json!(null), json!("snow"), json!(7), json!([1, 2]), json!({})] {
            assert_eq!(EffectsChoice::from_value(&value), EffectsChoice::default(), "{value}");
        }
    }
    #[test]
    fn older_named_scenes_are_converted_and_leaves_become_maple() {
        let choice = EffectsChoice::from_value(&json!({
            "rain":{"enabled":false,"intensity":0.2,"speed":2},
            "leaves":{"enabled":true,"intensity":0.5,"speed":1.5},
            "meteors":{"enabled":true,"intensity":1,"speed":1}}));
        assert_eq!(choice.selected, "maple", "the first chosen scene in the old order wins");
        assert_eq!(choice.params["maple"], EffectParams { density: 1.0, speed: 1.5, size: 1.0 });
        assert_eq!(choice.params["rain"], EffectParams { density: 0.4, speed: 2.0, size: 1.0 });
        assert!(!choice.params.contains_key("leaves"));
        assert_eq!(EffectsChoice::from_value(&json!({"rain":{"enabled":false}})).selected, "");
    }
    #[test]
    fn it_survives_being_saved_and_read_back() {
        let choice = EffectsChoice::from_value(&json!({"selected":"stars","params":{"stars":{"density":1.4,"speed":0.5,"size":2}}}));
        let again: EffectsChoice = serde_json::from_value(serde_json::to_value(&choice).unwrap()).unwrap();
        assert_eq!(again, choice);
        assert_eq!(choice.params_of("stars").density, 1.4);
        assert_eq!(choice.params_of("unknown"), EffectParams::default());
    }
}
