//! Whisper-style log-mel features for Smart Turn v3: 8 s of 16 kHz audio → 80 mel bins × 800 frames.
//! Mirrors pipecat's numpy port of `transformers.WhisperFeatureExtractor(chunk_length=8)` with
//! `do_normalize=True`; the tests compare against values from that reference.
use rustfft::{num_complex::Complex, FftPlanner};
use std::sync::OnceLock;

pub const SAMPLE_RATE: usize = 16000;
pub const SAMPLES: usize = SAMPLE_RATE * 8;
pub const N_MELS: usize = 80;
pub const FRAMES: usize = 800;
const N_FFT: usize = 400;
const HOP: usize = 160;
const BINS: usize = N_FFT / 2 + 1;

fn hz_to_mel(hz: f64) -> f64 { if hz >= 1000.0 { 15.0 + (hz / 1000.0).ln() * (27.0 / 6.4f64.ln()) } else { 3.0 * hz / 200.0 } }
fn mel_to_hz(mel: f64) -> f64 { if mel >= 15.0 { 1000.0 * ((6.4f64.ln() / 27.0) * (mel - 15.0)).exp() } else { 200.0 * mel / 3.0 } }

/// Each mel filter is non-zero over a short run of bins; only that run is kept.
struct Tables { filters: Vec<(usize, Vec<f64>)>, window: [f64; N_FFT] }

fn tables() -> &'static Tables {
    static TABLES: OnceLock<Tables> = OnceLock::new();
    TABLES.get_or_init(|| {
        let (low, high) = (hz_to_mel(0.0), hz_to_mel(SAMPLE_RATE as f64 / 2.0));
        let edges: Vec<f64> = (0..N_MELS + 2).map(|i| mel_to_hz(low + (high - low) * i as f64 / (N_MELS + 1) as f64)).collect();
        let filters = (0..N_MELS).map(|m| {
            let mut row = [0.0; BINS];
            let enorm = 2.0 / (edges[m + 2] - edges[m]);
            for (b, value) in row.iter_mut().enumerate() {
                let freq = (SAMPLE_RATE as f64 / 2.0) * b as f64 / (BINS - 1) as f64;
                let down = (freq - edges[m]) / (edges[m + 1] - edges[m]);
                let up = (edges[m + 2] - freq) / (edges[m + 2] - edges[m + 1]);
                *value = down.min(up).max(0.0) * enorm;
            }
            let start = row.iter().position(|v| *v != 0.0).unwrap_or(0);
            let end = row.iter().rposition(|v| *v != 0.0).map_or(start, |e| e + 1);
            (start, row[start..end].to_vec())
        }).collect();
        let mut window = [0.0; N_FFT];
        for (i, w) in window.iter_mut().enumerate() { *w = 0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / N_FFT as f64).cos(); }
        Tables { filters, window }
    })
}

/// Keeps the last 8 s, padding silence in front of shorter audio (as Smart Turn expects).
pub fn last_eight_seconds(audio: &[f32]) -> Vec<f32> {
    if audio.len() >= SAMPLES { return audio[audio.len() - SAMPLES..].to_vec(); }
    let mut out = vec![0.0; SAMPLES - audio.len()];
    out.extend_from_slice(audio);
    out
}

/// Log-mel features of exactly 8 s of audio, row-major `[80][800]`.
pub fn log_mel(audio: &[f32]) -> Vec<f32> {
    let Tables { filters, window } = tables();
    let n = SAMPLES;
    let mut x = vec![0.0f64; n];
    for (target, source) in x.iter_mut().zip(audio) { *target = *source as f64; }
    let mean = x.iter().sum::<f64>() / n as f64;
    let variance = x.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
    let scale = 1.0 / (variance + 1e-7).sqrt();
    for v in &mut x { *v = (*v - mean) * scale; }
    // Reflect-pad by half a window on both sides, as the centred STFT does.
    let pad = N_FFT / 2;
    let mut padded = vec![0.0f64; n + 2 * pad];
    padded[pad..pad + n].copy_from_slice(&x);
    for i in 0..pad { padded[pad - 1 - i] = x[i + 1]; padded[pad + n + i] = x[n - 2 - i]; }
    let fft = FftPlanner::<f64>::new().plan_fft_forward(N_FFT);
    let mut buffer = vec![Complex::new(0.0, 0.0); N_FFT];
    let mut scratch = vec![Complex::new(0.0, 0.0); fft.get_inplace_scratch_len()];
    let mut power = [0.0f64; BINS];
    let (mut cached, mut cached_value) = ([0.0f64; BINS], f64::NAN);
    let mut values = vec![0.0f64; N_MELS * FRAMES];
    let mut max = f64::NEG_INFINITY;
    for f in 0..FRAMES {
        let frame = &padded[f * HOP..f * HOP + N_FFT];
        // Leading silence pads most short utterances: identical constant frames share one spectrum.
        let constant = frame.iter().all(|v| *v == frame[0]);
        if constant && frame[0] == cached_value { power = cached; } else {
            for (slot, (sample, w)) in buffer.iter_mut().zip(frame.iter().zip(window)) { *slot = Complex::new(sample * w, 0.0); }
            fft.process_with_scratch(&mut buffer, &mut scratch);
            for (p, c) in power.iter_mut().zip(&buffer) { *p = c.norm_sqr(); }
            if constant { cached = power; cached_value = frame[0]; }
        }
        for (m, (start, row)) in filters.iter().enumerate() {
            let sum: f64 = row.iter().zip(&power[*start..]).map(|(a, b)| a * b).sum();
            let value = sum.max(1e-10).log10();
            values[m * FRAMES + f] = value;
            max = max.max(value);
        }
    }
    values.into_iter().map(|v| ((v.max(max - 8.0) + 4.0) / 4.0) as f32).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two tones over the last 3 s of an 8 s window, the rest silence; reference values from
    /// pipecat's `compute_whisper_log_mel_features` on the same signal.
    fn signal() -> Vec<f32> {
        let mut audio = vec![0.0f32; SAMPLES];
        for i in 0..SAMPLE_RATE * 3 {
            let t = i as f64 / SAMPLE_RATE as f64;
            audio[SAMPLES - SAMPLE_RATE * 3 + i] = (0.3 * (2.0 * std::f64::consts::PI * 220.0 * t).sin() + 0.1 * (2.0 * std::f64::consts::PI * 1330.0 * t).sin()) as f32;
        }
        audio
    }

    #[test]
    fn matches_the_reference_features() {
        let features = log_mel(&signal());
        assert_eq!(features.len(), N_MELS * FRAMES);
        for (mel, frame, expected) in REFERENCE {
            let got = features[mel * FRAMES + frame];
            assert!((got - expected).abs() < 1e-4, "mel {mel} frame {frame}: {got} vs {expected}");
        }
    }

    #[test]
    fn pads_short_audio_in_front_and_keeps_the_end_of_long_audio() {
        let short = last_eight_seconds(&[1.0; 10]);
        assert_eq!(short.len(), SAMPLES);
        assert!(short[..SAMPLES - 10].iter().all(|v| *v == 0.0) && short[SAMPLES - 10..].iter().all(|v| *v == 1.0));
        let mut long = vec![0.0; SAMPLES + 5];
        long[SAMPLES + 4] = 2.0;
        assert_eq!(last_eight_seconds(&long)[SAMPLES - 1], 2.0);
    }

    // (mel, frame, value) picked across silence, onset and the tones.
    const REFERENCE: &[(usize, usize, f32)] = &include!("features_reference.in");
}
