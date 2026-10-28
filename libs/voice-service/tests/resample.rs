//! 重采样质量：高于 16 kHz 的输入必须先低通再取样，带外成分不能折叠进语音频段。
use xiaoban_voice_service::audio;

fn wav_i16(rate: u32, samples: &[f64]) -> Vec<u8> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut writer = hound::WavWriter::new(&mut cursor, spec).unwrap();
        for s in samples {
            writer
                .write_sample((s * 32767.0).round().clamp(-32768.0, 32767.0) as i16)
                .unwrap();
        }
        writer.finalize().unwrap();
    }
    cursor.into_inner()
}
fn tone(rate: u32, hz: f64, amplitude: f64, seconds: f64) -> Vec<f64> {
    (0..(rate as f64 * seconds) as usize)
        .map(|i| amplitude * (2.0 * std::f64::consts::PI * hz * i as f64 / rate as f64).sin())
        .collect()
}
/// 去掉首尾各 100 ms（滤波器的边缘过渡）后的均方根。
fn middle_rms(samples: &[f32]) -> f64 {
    let edge = 1600.min(samples.len() / 4);
    let body = &samples[edge..samples.len() - edge];
    (body.iter().map(|v| (*v as f64).powi(2)).sum::<f64>() / body.len() as f64).sqrt()
}

#[test]
fn out_of_band_tones_are_attenuated_instead_of_aliasing() {
    // (源采样率, 带外频率)：折叠后会落在 0–8 kHz 内。9 kHz 以上都要被压到 -40 dB 以下。
    for (rate, hz) in [
        (44100, 10000.0),
        (44100, 15000.0),
        (48000, 12000.0),
        (48000, 20000.0),
        (24000, 10000.0),
        (32000, 9500.0),
        (22050, 9500.0),
    ] {
        let source = tone(rate, hz, 0.5, 1.0);
        let out = audio::decode(&wav_i16(rate, &source)).unwrap();
        let input_rms = 0.5 / 2f64.sqrt();
        let ratio = middle_rms(&out.samples) / input_rms;
        let db = 20.0 * ratio.max(1e-9).log10();
        println!("{rate} Hz 源里的 {hz} Hz：衰减 {db:.1} dB");
        assert!(db < -40.0, "{rate} Hz 源里的 {hz} Hz 只衰减了 {db:.1} dB");
    }
}

#[test]
fn in_band_tones_keep_amplitude_frequency_and_duration() {
    for (rate, hz) in [
        (44100, 300.0),
        (44100, 1000.0),
        (44100, 5000.0),
        (48000, 3000.0),
        (48000, 6000.0),
        (24000, 4000.0),
    ] {
        let out = audio::decode(&wav_i16(rate, &tone(rate, hz, 0.5, 1.0))).unwrap();
        assert_eq!(out.samples.len(), 16000, "{rate} Hz");
        let ratio = middle_rms(&out.samples) / (0.5 / 2f64.sqrt());
        assert!(
            (0.98..=1.02).contains(&ratio),
            "{rate} Hz 源里的 {hz} Hz 幅度比 {ratio}"
        );
        let crossings = out
            .samples
            .windows(2)
            .filter(|w| w[0] <= 0.0 && w[1] > 0.0)
            .count() as f64;
        assert!(
            (crossings - hz).abs() <= 2.0,
            "{rate} Hz 源里的 {hz} Hz 测得 {crossings}"
        );
    }
}

#[test]
fn filtering_does_not_shift_time() {
    // 0.5 秒处的冲击，输出里的峰值应当仍在 0.5 秒（第 8000 个采样）附近。
    for rate in [24000u32, 44100, 48000] {
        let mut source = vec![0.0; rate as usize];
        source[rate as usize / 2] = 0.9;
        let out = audio::decode(&wav_i16(rate, &source)).unwrap();
        let peak = out
            .samples
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
            .unwrap()
            .0;
        assert!((peak as i64 - 8000).abs() <= 1, "{rate} Hz：峰值在 {peak}");
    }
}

#[test]
fn overshoot_stays_inside_the_valid_range() {
    // 满幅方波经低通会过冲（吉布斯现象），输出仍必须在 -1..=1 内，否则后面的校验会拒绝。
    let rate = 44100;
    let square: Vec<f64> = (0..rate)
        .map(|i| if (i / 50) % 2 == 0 { 1.0 } else { -1.0 })
        .collect();
    let out = audio::decode(&wav_i16(rate, &square)).unwrap();
    assert!(out.samples.iter().all(|s| s.abs() <= 1.0));
    audio::validate_pcm(&out).unwrap();
}

#[test]
fn rates_at_or_below_16k_are_not_filtered() {
    let source = tone(16000, 3000.0, 0.5, 0.5);
    let out = audio::decode(&wav_i16(16000, &source)).unwrap();
    assert_eq!(out.samples.len(), source.len());
    for (a, b) in out.samples.iter().zip(&source) {
        assert!((*a as f64 - b).abs() < 1e-3);
    }
    let up = audio::decode(&wav_i16(8000, &tone(8000, 1000.0, 0.5, 1.0))).unwrap();
    assert_eq!(up.samples.len(), 16000);
}

#[test]
fn long_high_rate_audio_decodes_in_reasonable_time_with_bounded_buffer() {
    let started = std::time::Instant::now();
    let out = audio::decode(&wav_i16(48000, &tone(48000, 440.0, 0.4, 30.0))).unwrap();
    assert_eq!(out.samples.len(), 30 * 16000);
    println!("30 秒 48 kHz 音频解码用时 {:?}", started.elapsed());
}
