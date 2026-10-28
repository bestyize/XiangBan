mod common;
use common::{fixtures::fixture, wav};
use std::{sync::mpsc, time::Duration};
use xiaoban_voice_service::audio;

fn random(state: &mut u64) -> u64 {
    *state ^= *state << 13; *state ^= *state >> 7; *state ^= *state << 17; *state
}
fn bounded_decode(input: Vec<u8>) {
    let (tx, rx) = mpsc::sync_channel(1);
    let saved = input.clone();
    std::thread::spawn(move || { let _ = tx.send(std::panic::catch_unwind(|| audio::decode(&input))); });
    if rx.recv_timeout(Duration::from_secs(2)).expect("decode exceeded 2 seconds").is_err() {
        std::fs::write("target/hostile-audio-panic.bin", saved).unwrap();
        panic!("decoder panicked; input saved in target/hostile-audio-panic.bin");
    }
}

#[test]
fn random_and_mutated_compressed_inputs_return_without_panicking() {
    let mut rng = 0x37_aa45_1234_5678;
    let prefixes: &[&[u8]] = &[b"", b"RIFF\0\0\0\0WAVE", b"fLaC", b"OggS", b"ID3", b"\xff\xfb"];
    for prefix in prefixes {
        for _ in 0..60 {
            let mut input = prefix.to_vec();
            let n = (random(&mut rng) % 512) as usize;
            input.extend((0..n).map(|_| random(&mut rng) as u8));
            bounded_decode(input);
        }
    }
    for format in ["mp3", "flac", "ogg"] {
        let original = fixture(format);
        for i in 0..60 {
            let mut data = original.clone();
            match i % 4 {
                0 => { let p = random(&mut rng) as usize % data.len(); data[p] ^= 1 << (random(&mut rng) % 8); },
                1 => { let p = random(&mut rng) as usize % data.len(); data.truncate(p); },
                2 => data.extend_from_slice(&original[..original.len().min(256)]),
                _ => data.extend_from_slice(&original),
            }
            bounded_decode(data);
        }
    }
}

#[test]
fn corrupt_mp3_bit_reservoir_is_an_error() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let data = STANDARD.decode(include_str!("fixtures/mp3-corrupt.b64").trim()).unwrap();
    assert!(audio::decode(&data).is_err());
}

#[test]
fn ten_minute_boundary_and_one_frame_over() {
    let exact = wav(&vec![0.0; 600], 1, 1, 16, false);
    assert_eq!(audio::decode(&exact).unwrap().samples.len(), audio::MAX_SAMPLES);
    let excess = wav(&vec![0.0; 601], 1, 1, 16, false);
    assert!(matches!(audio::decode(&excess), Err(xiaoban_voice_service::VoiceError::TooLong)));
}

#[test]
fn false_wav_headers_and_invalid_ogg_lacing_are_bounded() {
    let original = wav(&[0.0; 16], 16000, 1, 16, false);
    for rate in [0u32, 1, 384000, u32::MAX] {
        for channels in [0u16, 1, 9, u16::MAX] {
            for bits in [0u16, 7, 16, u16::MAX] {
                let mut data = original.clone();
                data[22..24].copy_from_slice(&channels.to_le_bytes());
                data[24..28].copy_from_slice(&rate.to_le_bytes());
                data[34..36].copy_from_slice(&bits.to_le_bytes());
                data[40..44].copy_from_slice(&u32::MAX.to_le_bytes());
                bounded_decode(data);
            }
        }
    }
    let mut ogg = fixture("ogg"); ogg[26] = 255; ogg.truncate(30); bounded_decode(ogg);
    let mut flac = fixture("flac");
    flac[21] |= 0x0f;
    flac[22..26].fill(0xff);
    assert!(matches!(audio::decode(&flac), Err(xiaoban_voice_service::VoiceError::TooLong)));
}

#[test]
fn short_legal_wavs_preserve_length_bounds_and_source_points() {
    let mut rng = 0x93a1_7135_6655_1321;
    let rates = [8000u32, 16000, 22050, 44100, 48000];
    for i in 0..100 {
        let rate = rates[i % rates.len()];
        let frames = 1 + (random(&mut rng) % 200) as usize;
        let channels = (1 + random(&mut rng) % 3) as u16;
        let samples: Vec<f32> = (0..frames * channels as usize).map(|_| (random(&mut rng) % 12001) as f32 / 12000.0 - 0.5).collect();
        let out = audio::decode(&wav(&samples, rate, channels, 16, false)).unwrap();
        assert_eq!(out.samples.len(), ((frames as u64 * 16000 + u64::from(rate) / 2) / u64::from(rate)) as usize);
        assert!(out.samples.iter().all(|v| v.is_finite() && (-1.0..=1.0).contains(v)));
        if rate == 8000 || rate == 16000 {
            let step = 16000 / rate as usize;
            for frame in 0..frames {
                let average = samples[frame * channels as usize..(frame + 1) * channels as usize].iter().sum::<f32>() / f32::from(channels);
                assert!((out.samples[frame * step] - average).abs() < 0.0001);
            }
        }
    }
}

#[test]
fn short_wav_length_range_and_unfiltered_16khz() {
    for rate in [8000u32, 16000, 22050, 44100, 48000, 96000] {
        for frames in [1usize, 2, 37, 171] {
            let input: Vec<_> = (0..frames).map(|i| ((i % 13) as f32 - 6.0) / 16.0).collect();
            let out = audio::decode(&wav(&input, rate, 1, 16, false)).unwrap();
            assert_eq!(out.samples.len(), (frames as u64 * 16000 + rate as u64 / 2) as usize / rate as usize);
            assert!(out.samples.iter().all(|s| s.is_finite() && (-1.0..=1.0).contains(s)));
            if rate == 16000 { for (a, b) in input.iter().zip(out.samples) { assert!((a-b).abs() < 0.0001); } }
        }
    }
}

#[cfg(windows)]
#[test]
#[ignore = "manual release benchmark; set VOICE_BENCH_FILE to a generated FLAC"]
fn release_flac_stress_measurement() {
    use std::{ffi::c_void, mem::size_of, time::Instant};
    #[repr(C)]
    struct Counters { size: u32, faults: u32, peak_working_set: usize, working_set: usize, peak_paged: usize, paged: usize, peak_nonpaged: usize, nonpaged: usize, pagefile: usize, peak_pagefile: usize }
    #[link(name = "kernel32")]
    unsafe extern "system" { fn GetCurrentProcess() -> *mut c_void; }
    #[link(name = "psapi")]
    unsafe extern "system" { fn GetProcessMemoryInfo(process: *mut c_void, counters: *mut Counters, size: u32) -> i32; }
    let path = std::env::var("VOICE_BENCH_FILE").unwrap();
    let input = std::fs::read(&path).unwrap();
    let start = Instant::now();
    let result = audio::decode(&input);
    let elapsed = start.elapsed();
    let mut memory = Counters { size: size_of::<Counters>() as u32, faults: 0, peak_working_set: 0, working_set: 0, peak_paged: 0, paged: 0, peak_nonpaged: 0, nonpaged: 0, pagefile: 0, peak_pagefile: 0 };
    assert_ne!(unsafe { GetProcessMemoryInfo(GetCurrentProcess(), &mut memory, size_of::<Counters>() as u32) }, 0);
    println!("{path}: {elapsed:?}, peak RSS {:.2} MiB, result {:?}", memory.peak_working_set as f64 / 1048576.0, result.as_ref().map(|p| p.samples.len()));
}
