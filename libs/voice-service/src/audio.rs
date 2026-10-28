//! In-memory decoding, bounded streaming resampling, segmentation, and WAV output.
use crate::{Pcm, VoiceError};
use std::{io::Cursor, ops::Range};

/// Recognition sample rate.
pub const SAMPLE_RATE: usize = 16_000;
/// Maximum decoded duration (ten minutes).
pub const MAX_SAMPLES: usize = 600 * SAMPLE_RATE;

fn invalid() -> VoiceError {
    VoiceError::InvalidInput("malformed audio".into())
}
fn unsupported() -> VoiceError {
    VoiceError::Unsupported("WAV, MP3, FLAC, OGG Vorbis".into())
}

// Keeps only one previous source sample and the bounded 16 kHz output, even for
// compressed input. Linear interpolation preserves duration; no anti-alias filter.
struct Resampler {
    rate: u32,
    frames: usize,
    previous: f32,
    output: Vec<f32>,
}
impl Resampler {
    fn new(rate: u32, channels: usize) -> Result<Self, VoiceError> {
        if !(1..=384_000).contains(&rate) || !(1..=8).contains(&channels) {
            return Err(unsupported());
        }
        Ok(Self {
            rate,
            frames: 0,
            previous: 0.0,
            output: Vec::new(),
        })
    }
    fn push(&mut self, sample: f32) -> Result<(), VoiceError> {
        if !sample.is_finite() {
            return Err(invalid());
        }
        if self.frames >= self.rate as usize * 600 {
            return Err(VoiceError::TooLong);
        }
        let sample = sample.clamp(-1.0, 1.0);
        let position = self.frames as f64;
        loop {
            let next = self.output.len() as f64 * self.rate as f64 / SAMPLE_RATE as f64;
            if next > position {
                break;
            }
            let fraction = (next - (position - 1.0)).clamp(0.0, 1.0) as f32;
            self.output
                .push(self.previous + (sample - self.previous) * fraction);
        }
        self.previous = sample;
        self.frames += 1;
        Ok(())
    }
    fn finish(mut self) -> Pcm {
        let length =
            (self.frames as u64 * SAMPLE_RATE as u64 + self.rate as u64 / 2) / self.rate as u64;
        self.output.resize(length as usize, self.previous);
        Pcm {
            samples: self.output,
        }
    }
}

/// Decode WAV PCM16/24/32 or float32, MP3, FLAC, or OGG Vorbis into 16 kHz mono.
/// Format detection uses file bytes, never the client-supplied filename/MIME type.
/// Stops while decoding at ten minutes. Channels are averaged before resampling.
pub fn decode(bytes: &[u8]) -> Result<Pcm, VoiceError> {
    if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WAVE") {
        decode_wav(bytes)
    } else if bytes.starts_with(b"fLaC") {
        decode_flac(bytes)
    } else if bytes.starts_with(b"OggS") {
        // An OGG container alone is insufficient (Opus must remain unsupported).
        let reader = ogg_identification(bytes)?;
        decode_ogg(reader)
    } else if bytes.starts_with(b"ID3")
        || bytes
            .get(..2)
            .is_some_and(|b| b[0] == 0xff && b[1] & 0xe0 == 0xe0)
    {
        decode_mp3(bytes)
    } else {
        Err(unsupported())
    }
}
fn decode_wav(bytes: &[u8]) -> Result<Pcm, VoiceError> {
    let mut reader = hound::WavReader::new(Cursor::new(bytes)).map_err(|_| invalid())?;
    let spec = reader.spec();
    if !matches!(
        (spec.sample_format, spec.bits_per_sample),
        (hound::SampleFormat::Int, 16 | 24 | 32) | (hound::SampleFormat::Float, 32)
    ) {
        return Err(unsupported());
    }
    let channels = spec.channels as usize;
    let mut out = Resampler::new(spec.sample_rate, channels)?;
    if reader.duration() as u64 > spec.sample_rate as u64 * 600 {
        return Err(VoiceError::TooLong);
    }
    let samples: Box<dyn Iterator<Item = Result<f32, VoiceError>> + '_> = match spec.sample_format {
        hound::SampleFormat::Float => {
            Box::new(reader.samples::<f32>().map(|s| s.map_err(|_| invalid())))
        }
        hound::SampleFormat::Int => {
            let scale = 2_f32.powi(spec.bits_per_sample as i32 - 1);
            Box::new(
                reader
                    .samples::<i32>()
                    .map(move |s| s.map(|v| v as f32 / scale).map_err(|_| invalid())),
            )
        }
    };
    push_interleaved(&mut out, samples, channels)?;
    Ok(out.finish())
}
fn push_interleaved(
    out: &mut Resampler,
    samples: impl Iterator<Item = Result<f32, VoiceError>>,
    channels: usize,
) -> Result<(), VoiceError> {
    let mut sum = 0.0;
    let mut count = 0;
    for sample in samples {
        let sample = sample?;
        if !sample.is_finite() {
            return Err(invalid());
        }
        sum += sample / channels as f32;
        count += 1;
        if count == channels {
            out.push(sum)?;
            sum = 0.0;
            count = 0;
        }
    }
    if count != 0 {
        return Err(invalid());
    }
    Ok(())
}
fn decode_flac(bytes: &[u8]) -> Result<Pcm, VoiceError> {
    let mut reader = claxon::FlacReader::new(Cursor::new(bytes)).map_err(|_| invalid())?;
    let info = reader.streaminfo();
    let mut out = Resampler::new(info.sample_rate, info.channels as usize)?;
    if info
        .samples
        .is_some_and(|n| n > info.sample_rate as u64 * 600)
    {
        return Err(VoiceError::TooLong);
    }
    let scale = 2_f32.powi(info.bits_per_sample as i32 - 1);
    push_interleaved(
        &mut out,
        reader
            .samples()
            .map(|s| s.map(|v| v as f32 / scale).map_err(|_| invalid())),
        info.channels as usize,
    )?;
    Ok(out.finish())
}
fn ogg_identification(
    bytes: &[u8],
) -> Result<lewton::inside_ogg::OggStreamReader<Cursor<&[u8]>>, VoiceError> {
    // The first page contains the codec identification packet after the lacing table.
    let segments = *bytes.get(26).ok_or_else(invalid)? as usize;
    let begin = 27 + segments;
    if bytes.get(begin..begin + 7) != Some(b"\x01vorbis") {
        return Err(unsupported());
    }
    lewton::inside_ogg::OggStreamReader::new(Cursor::new(bytes)).map_err(|_| invalid())
}
fn decode_ogg(
    mut reader: lewton::inside_ogg::OggStreamReader<Cursor<&[u8]>>,
) -> Result<Pcm, VoiceError> {
    let channels = reader.ident_hdr.audio_channels as usize;
    let rate = reader.ident_hdr.audio_sample_rate;
    let mut out = Resampler::new(rate, channels)?;
    while let Some(packet) = reader.read_dec_packet_itl().map_err(|_| invalid())? {
        if reader.ident_hdr.audio_channels as usize != channels
            || reader.ident_hdr.audio_sample_rate != rate
        {
            return Err(unsupported());
        }
        push_interleaved(
            &mut out,
            packet.into_iter().map(|v| Ok(v as f32 / 32768.0)),
            channels,
        )?;
    }
    Ok(out.finish())
}
fn decode_mp3(bytes: &[u8]) -> Result<Pcm, VoiceError> {
    let mut decoder = puremp3::Mp3Decoder::new(Cursor::new(bytes));
    let mut out: Option<Resampler> = None;
    loop {
        let frame = match decoder.next_frame() {
            Ok(frame) => frame,
            Err(puremp3::Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break
            }
            Err(_) => return Err(invalid()),
        };
        let rate = frame.header.sample_rate.hz();
        let resampler = match out.as_mut() {
            Some(out) => out,
            None => out.insert(Resampler::new(rate, 2)?),
        };
        if resampler.rate != rate {
            return Err(unsupported());
        }
        for i in 0..frame.num_samples {
            resampler.push((frame.samples[0][i] + frame.samples[1][i]) * 0.5)?;
        }
    }
    out.map(Resampler::finish).ok_or_else(invalid)
}

/// Validate backend PCM, allowing empty captured speech but never over ten minutes.
pub fn validate_pcm(pcm: &Pcm) -> Result<(), VoiceError> {
    if pcm.samples.len() > MAX_SAMPLES {
        return Err(VoiceError::TooLong);
    }
    if pcm
        .samples
        .iter()
        .any(|s| !s.is_finite() || !(-1.0..=1.0).contains(s))
    {
        return Err(invalid());
    }
    Ok(())
}

/// Partition without overlap/loss at locally lowest mean-square energy.
/// The nominal boundary is `max_samples - radius`, so its ±1.5s search window
/// stays inside the backend's hard maximum. Ties prefer the nominal boundary.
pub fn segments(samples: &[f32], max_samples: usize) -> Result<Vec<Range<usize>>, VoiceError> {
    if max_samples == 0 {
        return Err(VoiceError::Internal("zero backend limit".into()));
    }
    let mut result = Vec::new();
    let mut start = 0;
    while samples.len() - start > max_samples {
        let radius = (SAMPLE_RATE * 3 / 2).min(max_samples / 4);
        let target = start + max_samples - radius;
        let low = (target - radius).max(start + 1);
        let high = start + max_samples;
        let half = (SAMPLE_RATE / 100).min(radius).max(1);
        let energy = |p: usize| -> f64 {
            let s = samples[p] as f64;
            s * s
        };
        let mut left = low.saturating_sub(half).max(start);
        let mut right = (low + half).min(samples.len());
        let mut sum: f64 = (left..right).map(energy).sum();
        let mut best = target;
        let mut best_energy = f64::INFINITY;
        for cut in low..=high {
            let next_left = cut.saturating_sub(half).max(start);
            let next_right = (cut + half).min(samples.len());
            while left < next_left {
                sum -= energy(left);
                left += 1;
            }
            while right < next_right {
                sum += energy(right);
                right += 1;
            }
            let value = (sum / (right - left) as f64).max(0.0);
            if value < best_energy - 1e-12
                || ((value - best_energy).abs() <= 1e-12
                    && cut.abs_diff(target) < best.abs_diff(target))
            {
                best = cut;
                best_energy = value;
            }
        }
        result.push(start..best);
        start = best;
    }
    if start < samples.len() {
        result.push(start..samples.len());
    }
    Ok(result)
}

/// Encode PCM into mono 16 kHz PCM16 WAV, entirely in memory.
pub fn wav(pcm: &Pcm) -> Result<Vec<u8>, VoiceError> {
    validate_pcm(pcm)?;
    let mut cursor = Cursor::new(Vec::new());
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    {
        let mut writer = hound::WavWriter::new(&mut cursor, spec).map_err(|_| invalid())?;
        for s in &pcm.samples {
            writer
                .write_sample((s * 32768.0).round().clamp(-32768.0, 32767.0) as i16)
                .map_err(|_| invalid())?;
        }
        writer.finalize().map_err(|_| invalid())?;
    }
    Ok(cursor.into_inner())
}
