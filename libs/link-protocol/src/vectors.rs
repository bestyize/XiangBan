//! 测试向量：固定输入算出固定输出，两端（Rust、Kotlin）都必须得到相同结果。
//! 用 `cargo run --example gen_vectors` 重新生成 `vectors/v1.json`；`tests/vectors.rs` 校验文件与实现一致。
use crate::{envelope, keys::DeviceKey, pairing};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as B64, Engine};
use serde_json::{json, Value};

fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{x:02x}")).collect() }

pub fn generate() -> Value {
    let desktop = DeviceKey::from_bytes(&[0x11; 32]).unwrap();
    let phone = DeviceKey::from_bytes(&[0x22; 32]).unwrap();
    let secret = [0x33u8; 32];
    let (desktop_id, phone_id) = ("dsk_test0001", "phn_test0001");

    let qr = pairing::QrPayload {
        r: "wss://link.example.test".into(),
        d: desktop_id.into(),
        c: "PAIRCODE0001".into(),
        k: B64.encode(desktop.public_bytes()),
        s: B64.encode(secret),
    };
    let proof = pairing::make_proof(&secret, phone_id, desktop_id, &phone.public_bytes());
    let prk_d = pairing::derive_pairing_key(&desktop, &phone.public_bytes(), &secret).unwrap();
    let prk_p = pairing::derive_pairing_key(&phone, &desktop.public_bytes(), &secret).unwrap();
    assert_eq!(prk_d, prk_p);

    // ep：启动计数 7，随机部分固定为 0x44 重复 8 次。
    let mut ep = [0x44u8; 16];
    ep[..8].copy_from_slice(&7u64.to_be_bytes());
    let plains = [
        r#"{"t":"hello","id":"m1","v":1}"#,
        r#"{"t":"message.send","id":"m2","session":"s1","text":"你好，小伴"}"#,
    ];
    let mut d2p = Vec::new();
    for (i, p) in plains.iter().enumerate() {
        let e = envelope::seal_with(&prk_d, desktop_id, phone_id, &ep, i as u64, p.as_bytes()).unwrap();
        d2p.push(json!({ "seq": i, "plaintext": p, "envelope": e }));
    }
    let p2d = {
        let e = envelope::seal_with(&prk_p, phone_id, desktop_id, &ep, 0, plains[1].as_bytes()).unwrap();
        json!({ "seq": 0, "plaintext": plains[1], "envelope": e })
    };
    json!({
        "version": 1,
        "note": "测试向量。私钥为 32 字节重复值，仅用于测试，切勿用于真实设备。",
        "desktop": { "id": desktop_id, "private_hex": hex(&[0x11; 32]), "public_hex": hex(&desktop.public_bytes()) },
        "phone": { "id": phone_id, "private_hex": hex(&[0x22; 32]), "public_hex": hex(&phone.public_bytes()) },
        "pair_secret_hex": hex(&secret),
        "qr": { "payload": qr, "text": qr.encode() },
        "proof_hex": hex(&proof),
        "ecdh_shared_hex": hex(&desktop.agree(&phone.public_bytes()).unwrap()),
        "pairing_key_hex": hex(&prk_d),
        "ep_hex": hex(&ep),
        "desktop_to_phone": d2p,
        "phone_to_desktop": p2d,
        "padding": { "input_hex": hex(b"abc"), "padded_len": envelope::pad(b"abc").len(), "padded_hex": hex(&envelope::pad(b"abc")) },
    })
}
