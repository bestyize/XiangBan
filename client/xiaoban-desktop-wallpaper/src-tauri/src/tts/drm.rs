//! Sec-MS-GEC v2 DRM token generator for Microsoft Edge TTS.
//!
//! Generates a v2 Sec-MS-GEC token combining a CS64 digest and a modified SHA-256 digest
//! in the format "<16 hex CS64>-<64 hex SHA256>".

const WIN_EPOCH: i64 = 11_644_473_600;
const TRUSTED_CLIENT_TOKEN: &str = "6A5AA1D4EAFF4E9FB37E23D68491D6F4";

const SHA256_INITIAL: [u32; 8] = [
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A,
    0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
];
const SHA256_STATE: [u32; 8] = [
    0xFD9FBEA2, 0x91243CE7, 0xBE499094, 0xCD2E1E0A,
    0x6C006F49, 0x04BD35AB, 0xFEB0A7E4, 0x03EFFBD2,
];
const SHA256_SCHEDULE: [u32; 14] = [
    0xEB24BF9E, 0x1558691F, 0xA0C09EE2, 0x0CFA32BA,
    0xA622EE8C, 0xD1DD9491, 0xC0DD25A0, 0xF9E70AB6,
    0xF0C31573, 0x7058CE4E, 0x7188E606, 0x00E5AB33,
    0xD258B060, 0x72766564,
];
const SHA256_ROUND_CONSTANTS: [u32; 64] = [
    0x428A2F98, 0x71374491, 0xB5C0FBCF, 0xE9B5DBA5, 0x3956C25B, 0x59F111F1, 0x923F82A4, 0xAB1C5ED5,
    0xD807AA98, 0x12835B01, 0x243185BE, 0x550C7DC3, 0x72BE5D74, 0x80DEB1FE, 0x9BDC06A7, 0xC19BF174,
    0xE49B69C1, 0xEFBE4786, 0x0FC19DC6, 0x240CA1CC, 0x2DE92C6F, 0x4A7484AA, 0x5CB0A9DC, 0x76F988DA,
    0x983E5152, 0xA831C66D, 0xB00327C8, 0xBF597FC7, 0xC6E00BF3, 0xD5A79147, 0x06CA6351, 0x14292967,
    0x27B70A85, 0x2E1B2138, 0x4D2C6DFC, 0x53380D13, 0x650A7354, 0x766A0ABB, 0x81C2C92E, 0x92722C85,
    0xA2BFE8A1, 0xA81A664B, 0xC24B8B70, 0xC76C51A3, 0xD192E819, 0xD6990624, 0xF40E3585, 0x106AA070,
    0x19A4C116, 0x1E376C08, 0x2748774C, 0x34B0BCB5, 0x391C0CB3, 0x4ED8AA4A, 0x5B9CCA4F, 0x682E6FF3,
    0x748F82EE, 0x78A5636F, 0x84C87814, 0x8CC70208, 0x90BEFFFA, 0xA4506CEB, 0xBEF9A3F7, 0xC67178F2,
];
const SHA256_ADJUSTMENT: u32 = 0xC9DC06A7;
const SHA256_SIGMA0_ADJUSTMENT: u32 = 0x059C0B80;
const SHA256_SIGMA1_ADJUSTMENT: u32 = 0x000B92C0;

const CS64_COEFFICIENTS: [(u32, u32); 14] = [
    (0xE82E91B1, 0xB32700D0), (0x44DE24C6, 0x88DFF51D), (0x3BEB65BC, 0x29514038), (0xC89C5933, 0xB328CBDF),
    (0x8D1EC211, 0xD405B4D5), (0x314EF5D5, 0x26E55F3C), (0xE92491B1, 0x607801C6), (0xF0A5B9D5, 0x82B2C1FB),
    (0x06064CF3, 0xFA51A3DA), (0xDA79FA60, 0x207CC464), (0xBED20560, 0x3C3C74F3), (0x89F9C9F8, 0xD2D14B61),
    (0x35FA38C6, 0xBB04A2C1), (0xCB49E3C6, 0xC5626167),
];
const CS64_FACTOR: u32 = 0x82893933;
const CS64_ADJUSTMENT0: u32 = 0xF2F90B1A;
const CS64_ADJUSTMENT1: u32 = 0x0B1A0000;

#[inline(always)]
fn sigma0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}

#[inline(always)]
fn sigma1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}

fn sha256_rounds(state: [u32; 8], words: &[u32; 64], first_round: usize) -> [u32; 8] {
    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];
    let mut e = state[4];
    let mut f = state[5];
    let mut g = state[6];
    let mut h = state[7];

    for i in first_round..64 {
        let k = if first_round == 14 && i == 14 {
            SHA256_ADJUSTMENT
        } else {
            SHA256_ROUND_CONSTANTS[i]
        };
        let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let ch = g ^ (e & (f ^ g));
        let t1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(k).wrapping_add(words[i]);
        let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let maj = (a & b) | (c & (a | b));
        let t2 = s0.wrapping_add(maj);

        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }
    [a, b, c, d, e, f, g, h]
}

fn expand_schedule(words: &mut [u32; 64], first: usize) {
    for i in first..64 {
        words[i] = sigma1(words[i - 2])
            .wrapping_add(words[i - 7])
            .wrapping_add(sigma0(words[i - 15]))
            .wrapping_add(words[i - 16]);
    }
}

fn sha256_init(data: &[u8]) -> [u32; 8] {
    let mut words = [0u32; 64];
    words[14] = ((data[0] as u32) << 16) | ((data[1] as u32) << 8) | (data[2] as u32);
    words[15] = u32::from_be_bytes(data[3..7].try_into().unwrap());

    for i in 16..30 {
        let mut val = SHA256_SCHEDULE[i - 16];
        if i - 2 >= 14 {
            let mut s1 = sigma1(words[i - 2]);
            if i == 16 {
                s1 ^= SHA256_SIGMA1_ADJUSTMENT;
            }
            val = val.wrapping_add(s1);
        }
        if i - 7 >= 14 {
            val = val.wrapping_add(words[i - 7]);
        }
        if i - 15 >= 14 {
            let mut s0 = sigma0(words[i - 15]);
            if i == 29 {
                s0 ^= SHA256_SIGMA0_ADJUSTMENT;
            }
            val = val.wrapping_add(s0);
        }
        words[i] = val;
    }
    words[30] = sigma1(words[28])
        .wrapping_add(words[23])
        .wrapping_add(sigma0(words[15]))
        .wrapping_add(words[14])
        .wrapping_add(SHA256_ADJUSTMENT)
        .wrapping_sub(SHA256_ROUND_CONSTANTS[14]);

    expand_schedule(&mut words, 31);
    let working = sha256_rounds(SHA256_STATE, &words, 14);
    let mut res = [0u32; 8];
    for (idx, r) in res.iter_mut().enumerate() {
        *r = SHA256_INITIAL[idx].wrapping_add(working[idx]);
    }
    res
}

fn sha256_digest(data: &[u8]) -> [u8; 32] {
    let length = 57 + data.len();
    let zero_count = (55i64 - length as i64).rem_euclid(64) as usize;
    let mut padded = Vec::with_capacity(data.len() - 7 + 1 + zero_count + 8);
    padded.extend_from_slice(&data[7..]);
    padded.push(0x80);
    padded.resize(padded.len() + zero_count, 0x00);
    padded.extend_from_slice(&((length as u64) * 8).to_be_bytes());

    let mut state = sha256_init(&data[..7]);
    for chunk in padded.chunks_exact(64) {
        let mut words = [0u32; 64];
        for (w_idx, w_bytes) in chunk.chunks_exact(4).enumerate() {
            words[w_idx] = u32::from_be_bytes(w_bytes.try_into().unwrap());
        }
        expand_schedule(&mut words, 16);
        let working = sha256_rounds(state, &words, 0);
        for idx in 0..8 {
            state[idx] = state[idx].wrapping_add(working[idx]);
        }
    }

    let mut out = [0u8; 32];
    for (idx, word) in state.iter().enumerate() {
        out[idx * 4..idx * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

fn words_le4_even(data: &[u8]) -> Vec<u32> {
    let mut padded = data.to_vec();
    let rem = padded.len() % 4;
    if rem != 0 {
        padded.resize(padded.len() + (4 - rem), 0);
    }
    let mut words: Vec<u32> = padded
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
        .collect();
    if words.len() % 2 != 0 {
        words.push(0);
    }
    words
}

fn cs64_digest(data: &[u8], digest: &[u8; 32]) -> (u32, u32) {
    let key_a = u32::from_le_bytes(digest[0..4].try_into().unwrap()) | 1;
    let key_b = u32::from_le_bytes(digest[4..8].try_into().unwrap()) | 1;

    let factors = [
        key_a.wrapping_mul(CS64_FACTOR),
        key_b.wrapping_mul(CS64_FACTOR),
    ];

    let mut terms: Vec<u32> = CS64_COEFFICIENTS
        .iter()
        .enumerate()
        .map(|(i, (c, _))| c.wrapping_mul(factors[i % 2]))
        .collect();

    let mut offsets: Vec<u32> = CS64_COEFFICIENTS.iter().map(|(_, o)| *o).collect();

    let mut zero_plus_data = Vec::with_capacity(data.len() + 1);
    zero_plus_data.push(0);
    zero_plus_data.extend_from_slice(data);

    for (i, word) in words_le4_even(&zero_plus_data).into_iter().enumerate() {
        let mut term = word.wrapping_mul(if i % 2 == 0 { key_a } else { key_b });
        let mut offset = word
            .rotate_left(16)
            .wrapping_mul(if i % 2 == 0 { 0xEF0569FB } else { 0xC31713DB });

        if i == 0 {
            term = term.wrapping_add(CS64_ADJUSTMENT0.wrapping_mul(factors[0]));
            offset = offset.wrapping_add(CS64_ADJUSTMENT1);
        }
        terms.push(term);
        offsets.push(offset);
    }

    let mut swap_acc = 0u32;
    let mut swap_last = 0u32;
    let mut state = 0u32;
    let mut acc = 0u32;

    for i in (0..terms.len()).step_by(2) {
        let x = terms[i];
        let y = terms[i + 1];

        let t = (offsets[i].wrapping_add(x)).rotate_left(16);
        let q = t.wrapping_mul(0x76F50F34);
        swap_acc = swap_acc
            .wrapping_add(q.rotate_left(16).wrapping_mul(0xC3EFEA97))
            .wrapping_add(q);

        let t = (offsets[i + 1].wrapping_add(y)).rotate_left(16);
        let q = t.wrapping_mul(0x1881CE3C);
        swap_last = q.rotate_left(16).wrapping_mul(0x35BD1EC9).wrapping_add(q);
        swap_acc = swap_acc.wrapping_add(swap_last);

        state = state.wrapping_mul(key_a).wrapping_add(x);
        state = state.rotate_left(16).wrapping_mul(0xCF98B111);
        state = state.rotate_left(16).wrapping_mul(0x87085B9F);
        state = state.rotate_left(16).wrapping_mul(0x12CEB96D);
        state = state.rotate_left(16).wrapping_mul(0x257E1D83);
        acc = acc.wrapping_add(state);

        let t = state.wrapping_mul(key_b).wrapping_add(y);
        let t = t.rotate_left(16).wrapping_mul(0xA27416F5);
        let t = t.rotate_left(16).wrapping_mul(0xD38396FF);
        state = (t.rotate_left(16).wrapping_mul(0x7C932B89))
            .rotate_left(16)
            .wrapping_mul(0xBFA49F69);
        acc = acc.wrapping_add(state);
    }

    (swap_last ^ state, swap_acc ^ acc)
}

pub fn generate_sec_ms_gec_v2(unix: i64) -> String {
    let windows = unix + WIN_EPOCH;
    let ticks = (windows - windows.rem_euclid(300)) * 10_000_000;
    let str_to_hash = format!("{ticks}{TRUSTED_CLIENT_TOKEN}");
    let digest = sha256_digest(str_to_hash.as_bytes());
    let (w1, w2) = cs64_digest(str_to_hash.as_bytes(), &digest);

    let mut cs64_hex = String::with_capacity(16);
    for b in w1.to_le_bytes().into_iter().chain(w2.to_le_bytes()) {
        use std::fmt::Write;
        let _ = write!(cs64_hex, "{b:02X}");
    }

    let mut sha_hex = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write;
        let _ = write!(sha_hex, "{b:02X}");
    }

    format!("{cs64_hex}-{sha_hex}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gec_v2_vector() {
        let token = generate_sec_ms_gec_v2(1_700_000_000);
        assert_eq!(
            token,
            "2CD23A2CA0B2595A-F3C846C370E05F9CC83ACC5F995C5F5D87A4D0F14CE8936A0F902E64099E38A2"
        );
    }

    #[test]
    fn test_window_consistency() {
        assert_eq!(generate_sec_ms_gec_v2(1_700_000_000), generate_sec_ms_gec_v2(1_700_000_001));
        assert_ne!(generate_sec_ms_gec_v2(1_700_000_000), generate_sec_ms_gec_v2(1_700_000_300));
    }
}
