#![allow(dead_code)]
use base64::Engine;
pub fn fixture(name: &str) -> Vec<u8> {
    let text = match name {
        "mp3" => include_str!("fixtures/mp3.b64"),
        "flac" => include_str!("fixtures/flac.b64"),
        "ogg" => include_str!("fixtures/ogg.b64"),
        _ => panic!("unknown fixture"),
    };
    base64::engine::general_purpose::STANDARD
        .decode(text.split_whitespace().collect::<String>())
        .unwrap()
}
