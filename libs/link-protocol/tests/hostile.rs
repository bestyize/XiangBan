use base64::{engine::general_purpose::URL_SAFE_NO_PAD as B64, Engine};
use std::collections::HashSet;
use xiaoban_link_protocol::{envelope::{open, pad, seal_with, unpad, Envelope}, keys::DeviceKey, pairing::QrPayload, replay::ReplayGuard, Error};

// Reproducible input stream without adding a production dependency.
fn random(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}
fn bytes(state: &mut u64, n: usize) -> Vec<u8> {
    (0..n).map(|_| random(state) as u8).collect()
}

#[test]
fn random_parsers_padding_keys_and_replay() {
    let mut rng = 0x1975_1031_aa55_9876;
    let key = DeviceKey::from_bytes(&[1; 32]).unwrap();
    let mut guard = ReplayGuard::new();
    let mut accepted = HashSet::new();
    let mut highest = None;
    for i in 0..2000 {
        let n = (random(&mut rng) % 1024) as usize;
        let input = bytes(&mut rng, n);
        let padded = pad(&input);
        assert_eq!(padded.len() % 256, 0);
        assert!(padded.len() > input.len());
        assert_eq!(unpad(padded).unwrap(), input);
        let _ = unpad(input.clone());
        let mut point = bytes(&mut rng, 65);
        if i % 2 == 0 { point[0] = 4; }
        assert_eq!(key.agree(&point), Err(Error::BadKey));
        let s = String::from_utf8_lossy(&input);
        assert!(QrPayload::decode(&s).is_err());
        assert!(QrPayload::decode(&format!("xbl1:{}", B64.encode(&input))).is_err());
        let counter = random(&mut rng) % 32;
        let seq = match i % 5 { 0 => 0, 1 => u64::MAX, _ => random(&mut rng) % 32 };
        let mut ep = [0u8; 16];
        ep[..8].copy_from_slice(&counter.to_be_bytes());
        ep[8..].copy_from_slice(&(random(&mut rng) % 4).to_be_bytes());
        let env = Envelope { v: random(&mut rng) as u8, from: s.to_string(), to: String::new(), ep: B64.encode(ep), seq, ct: B64.encode(&input) };
        let _ = open(&[0; 32], &env.from, &env.to, &env);
        let mut malformed = env.clone();
        malformed.v = 1;
        match i % 3 {
            0 => malformed.ep = s.to_string(),
            1 => malformed.ct = s.to_string(),
            _ => {}
        }
        assert!(open(&[0; 32], &malformed.from, &malformed.to, &malformed).is_err());
        let result = guard.check(&env);
        if highest.is_some_and(|h| counter < h) || accepted.contains(&(counter, seq)) {
            assert_eq!(result, Err(Error::Replay));
        }
        if result.is_ok() {
            assert!(accepted.insert((counter, seq)));
            highest = Some(counter);
        }
    }
    for json in [r#"{}"#, r#"{"r":1,"d":[],"c":null,"k":"","s":""}"#, r#"{"r":"","d":"","c":"","k":"AA","s":"AA"}"#] {
        assert_eq!(QrPayload::decode(&format!("xbl1:{}", B64.encode(json))), Err(Error::BadEncoding));
    }
}

#[test]
fn thousand_random_envelopes_reject_every_field_and_ciphertext_bit_change() {
    let mut rng = 0x7345_baca_2345_1234;
    let prk = [42; 32];
    for _ in 0..1000 {
        let n = (random(&mut rng) % 512) as usize;
        let plain = bytes(&mut rng, n);
        let ep: [u8; 16] = bytes(&mut rng, 16).try_into().unwrap();
        let env = seal_with(&prk, "from", "to", &ep, random(&mut rng), &plain).unwrap();
        assert_eq!(open(&prk, "from", "to", &env).unwrap(), plain);
        let mut changes = Vec::new();
        let mut e = env.clone(); e.v ^= 1; changes.push(e);
        let mut e = env.clone(); e.from.push('x'); changes.push(e);
        let mut e = env.clone(); e.to.push('x'); changes.push(e);
        let mut e = env.clone(); e.seq ^= 1 << (random(&mut rng) % 64); changes.push(e);
        let mut e = env.clone(); let mut raw = B64.decode(&e.ep).unwrap(); let pos = random(&mut rng) as usize % raw.len(); raw[pos] ^= 1 << (random(&mut rng) % 8); e.ep = B64.encode(raw); changes.push(e);
        let mut e = env.clone(); let mut raw = B64.decode(&e.ct).unwrap(); let pos = random(&mut rng) as usize % raw.len(); raw[pos] ^= 1 << (random(&mut rng) % 8); e.ct = B64.encode(raw); changes.push(e);
        for changed in changes { assert!(open(&prk, "from", "to", &changed).is_err()); }
    }
}
