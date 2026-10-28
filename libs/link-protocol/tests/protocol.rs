use base64::{engine::general_purpose::URL_SAFE_NO_PAD as B64, Engine};
use xiaoban_link_protocol::{envelope::*, keys::DeviceKey, pairing::*, replay::ReplayGuard, Error};

const D: &str = "dsk_a";
const P: &str = "phn_a";

fn paired() -> ([u8; 32], DeviceKey, DeviceKey, [u8; 32]) {
    let desktop = DeviceKey::generate();
    let phone = DeviceKey::generate();
    let mut secret = [0u8; 32];
    rand_core::RngCore::fill_bytes(&mut rand_core::OsRng, &mut secret);
    let prk = derive_pairing_key(&desktop, &phone.public_bytes(), &secret).unwrap();
    (prk, desktop, phone, secret)
}

#[test]
fn both_sides_derive_the_same_key_and_talk() {
    let (prk, desktop, phone, secret) = paired();
    assert_eq!(prk, derive_pairing_key(&phone, &desktop.public_bytes(), &secret).unwrap());
    let mut tx = Sender::new(prk, D, P, 1);
    let env = tx.seal("你好，小伴".as_bytes()).unwrap();
    assert_eq!(open(&prk, D, P, &env).unwrap(), "你好，小伴".as_bytes());
    // 方向不同，密钥不同：手机发给电脑的信封不能当作电脑发给手机的收下。
    let mut back = Sender::new(prk, P, D, 1);
    let env2 = back.seal(b"hi").unwrap();
    assert_eq!(open(&prk, D, P, &env2), Err(Error::Decrypt));
    assert_eq!(open(&prk, P, D, &env2).unwrap(), b"hi");
}

#[test]
fn wrong_secret_gives_a_different_key() {
    let (prk, desktop, phone, _) = paired();
    let other = derive_pairing_key(&desktop, &phone.public_bytes(), &[9u8; 32]).unwrap();
    assert_ne!(prk, other);
    let env = Sender::new(prk, D, P, 1).seal(b"x").unwrap();
    assert_eq!(open(&other, D, P, &env), Err(Error::Decrypt));
}

#[test]
fn tampering_with_any_visible_field_fails() {
    let (prk, ..) = paired();
    let mut tx = Sender::new(prk, D, P, 1);
    let env = tx.seal(b"secret").unwrap();
    for mutate in [
        |e: &mut Envelope| e.seq += 1,
        |e: &mut Envelope| e.from = "dsk_evil".into(),
        |e: &mut Envelope| e.to = "phn_evil".into(),
        |e: &mut Envelope| {
            let mut ep = B64.decode(&e.ep).unwrap();
            ep[15] ^= 1;
            e.ep = B64.encode(ep);
        },
        |e: &mut Envelope| {
            let mut ct = B64.decode(&e.ct).unwrap();
            ct[0] ^= 1;
            e.ct = B64.encode(ct);
        },
    ] {
        let mut e = env.clone();
        mutate(&mut e);
        assert!(open(&prk, D, P, &e).is_err());
    }
    let mut v = env.clone();
    v.v = 2;
    assert_eq!(open(&prk, D, P, &v), Err(Error::UnsupportedVersion(2)));
}

#[test]
fn padding_hides_length_and_roundtrips() {
    let (prk, ..) = paired();
    let mut tx = Sender::new(prk, D, P, 1);
    let a = tx.seal(b"a").unwrap();
    let b = tx.seal(&vec![b'b'; 200]).unwrap();
    assert_eq!(a.ct.len(), b.ct.len(), "短消息和 200 字节的消息密文一样长");
    let big = tx.seal(&vec![b'c'; 300]).unwrap();
    assert!(big.ct.len() > a.ct.len());
    assert_eq!(open(&prk, D, P, &big).unwrap(), vec![b'c'; 300]);
    assert_eq!(unpad(pad(b"")).unwrap(), b"");
    assert_eq!(unpad(vec![0, 0]), Err(Error::BadPadding));
}

#[test]
fn oversized_messages_are_refused() {
    let (prk, ..) = paired();
    assert_eq!(Sender::new(prk, D, P, 1).seal(&vec![0u8; 300 * 1024]), Err(Error::TooLarge));
}

#[test]
fn qr_roundtrip_and_validation() {
    let (_, desktop, _, secret) = paired();
    let qr = QrPayload { r: "wss://x".into(), d: D.into(), c: "C".into(), k: B64.encode(desktop.public_bytes()), s: B64.encode(secret) };
    let text = qr.encode();
    assert!(text.starts_with("xbl1:"));
    let back = QrPayload::decode(&text).unwrap();
    assert_eq!(back, qr);
    assert_eq!(back.secret().unwrap(), secret);
    assert_eq!(QrPayload::decode("xbl2:abc"), Err(Error::BadEncoding));
    let short = QrPayload { s: B64.encode([1u8; 8]), ..qr.clone() };
    assert_eq!(QrPayload::decode(&short.encode()), Err(Error::BadEncoding));
}

#[test]
fn pairing_proof_binds_identity_and_key() {
    let (_, desktop, phone, secret) = paired();
    let proof = make_proof(&secret, P, D, &phone.public_bytes());
    assert!(verify_proof(&secret, P, D, &phone.public_bytes(), &proof).is_ok());
    // 中转不知道一次性密钥，换成自己的公钥或换身份都通不过。
    let relay = DeviceKey::generate();
    assert_eq!(verify_proof(&secret, P, D, &relay.public_bytes(), &proof), Err(Error::BadProof));
    assert_eq!(verify_proof(&secret, "phn_other", D, &phone.public_bytes(), &proof), Err(Error::BadProof));
    assert_eq!(verify_proof(&[0u8; 32], P, D, &phone.public_bytes(), &proof), Err(Error::BadProof));
    let _ = desktop;
}

#[test]
fn bad_public_keys_are_rejected() {
    let (_, desktop, ..) = paired();
    assert_eq!(desktop.agree(&[0u8; 65]), Err(Error::BadKey));
    assert_eq!(desktop.agree(&[4u8; 10]), Err(Error::BadKey));
    assert!(DeviceKey::from_bytes(&[0u8; 32]).is_err());
}

#[test]
fn replay_guard_rules() {
    let (prk, ..) = paired();
    let mut tx = Sender::new(prk, D, P, 5);
    let e0 = tx.seal(b"0").unwrap();
    let e1 = tx.seal(b"1").unwrap();
    let e2 = tx.seal(b"2").unwrap();
    let mut g = ReplayGuard::new();
    assert!(g.check(&e0).is_ok());
    assert!(g.check(&e2).is_ok(), "允许跳号");
    assert_eq!(g.check(&e1), Err(Error::Replay), "回头的序号");
    assert_eq!(g.check(&e2), Err(Error::Replay), "同一条重放");

    // 对方重启：启动计数加一，序号从 0 重新开始。
    let mut tx2 = Sender::new(prk, D, P, 6);
    let n0 = tx2.seal(b"n0").unwrap();
    assert!(g.check(&n0).is_ok());
    // 旧会话的信封再也进不来。
    assert_eq!(g.check(&e2), Err(Error::Replay));

    // 状态持久化后恢复，仍然拒绝旧信封。
    let saved = g.state().cloned();
    let mut g2 = ReplayGuard::restore(saved);
    assert_eq!(g2.check(&e0), Err(Error::Replay));
    assert_eq!(g2.check(&n0), Err(Error::Replay));

    // 同一启动计数、不同随机部分：冲突，拒绝。
    let clash = Sender::new(prk, D, P, 6).seal(b"clash").unwrap();
    assert_eq!(g2.check(&clash), Err(Error::Replay));
}

#[test]
fn restart_without_incrementing_counter_is_caught_by_nonce_safety() {
    // 两次启动用了相同计数（落盘失败）：随机部分不同，密钥不同，不会发生随机数重用；
    // 但接收端会因“计数相同、ep 不同”拒绝，提示对方需要重新配对或修复计数。
    let (prk, ..) = paired();
    let a = Sender::new(prk, D, P, 3).seal(b"x").unwrap();
    let b = Sender::new(prk, D, P, 3).seal(b"x").unwrap();
    assert_ne!(a.ep, b.ep);
    assert_ne!(a.ct, b.ct);
    let mut g = ReplayGuard::new();
    assert!(g.check(&a).is_ok());
    assert_eq!(g.check(&b), Err(Error::Replay));
}
