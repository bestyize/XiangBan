/**
 * Whisper-style log-mel features for Smart Turn v3: 8 s of 16 kHz audio → 80 mel bins × 800 frames.
 * Mirrors pipecat's numpy port of `transformers.WhisperFeatureExtractor(chunk_length=8)` with
 * `do_normalize=True` (BSD-2 / Apache-2.0); tested against that reference.
 */
export const SAMPLE_RATE = 16000;
export const SAMPLES = SAMPLE_RATE * 8;
export const N_MELS = 80;
export const FRAMES = 800;
const N_FFT = 400;
const HOP = 160;
const BINS = N_FFT / 2 + 1;

function hzToMel(hz: number) { return hz >= 1000 ? 15 + Math.log(hz / 1000) * (27 / Math.log(6.4)) : 3 * hz / 200; }
function melToHz(mel: number) { return mel >= 15 ? 1000 * Math.exp((Math.log(6.4) / 27) * (mel - 15)) : 200 * mel / 3; }

/** Slaney-normalised triangular filters, `[mel][bin]`. */
function melFilters(): Float64Array[] {
  const low = hzToMel(0), high = hzToMel(SAMPLE_RATE / 2);
  const edges = Array.from({ length: N_MELS + 2 }, (_, i) => melToHz(low + (high - low) * i / (N_MELS + 1)));
  const freqs = Array.from({ length: BINS }, (_, i) => (SAMPLE_RATE / 2) * i / (BINS - 1));
  return Array.from({ length: N_MELS }, (_, m) => {
    const row = new Float64Array(BINS), enorm = 2 / (edges[m + 2] - edges[m]);
    for (let b = 0; b < BINS; b++) {
      const down = (freqs[b] - edges[m]) / (edges[m + 1] - edges[m]);
      const up = (edges[m + 2] - freqs[b]) / (edges[m + 2] - edges[m + 1]);
      row[b] = Math.max(0, Math.min(down, up)) * enorm;
    }
    return row;
  });
}

/** In-place iterative radix-2 FFT. */
function fft(re: Float64Array, im: Float64Array, inverse = false) {
  const n = re.length;
  for (let i = 1, j = 0; i < n; i++) {
    let bit = n >> 1;
    for (; j & bit; bit >>= 1) j ^= bit;
    j ^= bit;
    if (i < j) { [re[i], re[j]] = [re[j], re[i]]; [im[i], im[j]] = [im[j], im[i]]; }
  }
  for (let size = 2; size <= n; size <<= 1) {
    const angle = (inverse ? 2 : -2) * Math.PI / size, wr = Math.cos(angle), wi = Math.sin(angle);
    for (let start = 0; start < n; start += size) {
      let cr = 1, ci = 0;
      for (let k = 0; k < size / 2; k++) {
        const a = start + k, b = a + size / 2;
        const tr = re[b] * cr - im[b] * ci, ti = re[b] * ci + im[b] * cr;
        re[b] = re[a] - tr; im[b] = im[a] - ti; re[a] += tr; im[a] += ti;
        const next = cr * wr - ci * wi; ci = cr * wi + ci * wr; cr = next;
      }
    }
  }
}

/** Bluestein's algorithm turns the 400-point DFT into 1024-point FFTs. */
function makeDft400() {
  const m = 1024;
  const chirpRe = new Float64Array(N_FFT), chirpIm = new Float64Array(N_FFT);
  for (let k = 0; k < N_FFT; k++) { const a = Math.PI * ((k * k) % (2 * N_FFT)) / N_FFT; chirpRe[k] = Math.cos(a); chirpIm[k] = -Math.sin(a); }
  const bRe = new Float64Array(m), bIm = new Float64Array(m);
  bRe[0] = chirpRe[0]; bIm[0] = -chirpIm[0];
  for (let k = 1; k < N_FFT; k++) { bRe[k] = bRe[m - k] = chirpRe[k]; bIm[k] = bIm[m - k] = -chirpIm[k]; }
  fft(bRe, bIm);
  const re = new Float64Array(m), im = new Float64Array(m);
  /** Power spectrum |X[k]|² for k = 0..200 of a real frame. */
  return (frame: Float64Array, out: Float64Array) => {
    re.fill(0); im.fill(0);
    for (let k = 0; k < N_FFT; k++) { re[k] = frame[k] * chirpRe[k]; im[k] = frame[k] * chirpIm[k]; }
    fft(re, im);
    for (let k = 0; k < m; k++) { const r = re[k] * bRe[k] - im[k] * bIm[k]; im[k] = re[k] * bIm[k] + im[k] * bRe[k]; re[k] = r; }
    fft(re, im, true);
    for (let k = 0; k < BINS; k++) {
      const r = re[k] / m, i = im[k] / m;
      const xr = r * chirpRe[k] - i * chirpIm[k], xi = r * chirpIm[k] + i * chirpRe[k];
      out[k] = xr * xr + xi * xi;
    }
  };
}

let tables: { filters: Float64Array[]; window: Float64Array; dft: ReturnType<typeof makeDft400> } | undefined;
function setup() {
  tables ??= {
    filters: melFilters(),
    window: Float64Array.from({ length: N_FFT }, (_, i) => 0.5 - 0.5 * Math.cos(2 * Math.PI * i / N_FFT)),
    dft: makeDft400(),
  };
  return tables;
}

/** Keeps the last 8 s, padding silence in front of shorter audio (as Smart Turn expects). */
export function lastEightSeconds(audio: Float32Array): Float32Array {
  if (audio.length >= SAMPLES) return audio.subarray(audio.length - SAMPLES);
  const out = new Float32Array(SAMPLES);
  out.set(audio, SAMPLES - audio.length);
  return out;
}

/** Log-mel features of exactly 8 s of audio, row-major `[80][800]`. */
export function logMelFeatures(audio: Float32Array): Float32Array {
  const { filters, window, dft } = setup();
  const n = SAMPLES;
  const x = new Float64Array(n);
  x.set(audio.subarray(0, n));
  let mean = 0; for (let i = 0; i < n; i++) mean += x[i]; mean /= n;
  let variance = 0; for (let i = 0; i < n; i++) variance += (x[i] - mean) ** 2; variance /= n;
  const scale = 1 / Math.sqrt(variance + 1e-7);
  for (let i = 0; i < n; i++) x[i] = (Math.fround(x[i]) - mean) * scale;
  // Reflect-pad by half a window on both sides, as the centred STFT does.
  const pad = N_FFT / 2, padded = new Float64Array(n + 2 * pad);
  padded.set(x, pad);
  for (let i = 0; i < pad; i++) { padded[pad - 1 - i] = x[i + 1]; padded[pad + n + i] = x[n - 2 - i]; }
  const features = new Float32Array(N_MELS * FRAMES);
  const frame = new Float64Array(N_FFT), power = new Float64Array(BINS), cached = new Float64Array(BINS);
  let cachedValue = NaN, max = -Infinity;
  const logMel = new Float64Array(N_MELS * FRAMES);
  for (let f = 0; f < FRAMES; f++) {
    const start = f * HOP;
    // Leading silence pads most short utterances: identical constant frames share one spectrum.
    let constant = true; const first = padded[start];
    for (let i = 1; i < N_FFT && constant; i++) constant = padded[start + i] === first;
    if (constant && first === cachedValue) power.set(cached);
    else {
      for (let i = 0; i < N_FFT; i++) frame[i] = padded[start + i] * window[i];
      dft(frame, power);
      if (constant) { cached.set(power); cachedValue = first; }
    }
    for (let m = 0; m < N_MELS; m++) {
      const row = filters[m]; let sum = 0;
      for (let b = 0; b < BINS; b++) sum += row[b] * power[b];
      const value = Math.log10(Math.max(1e-10, sum));
      logMel[m * FRAMES + f] = value;
      if (value > max) max = value;
    }
  }
  for (let i = 0; i < logMel.length; i++) features[i] = (Math.max(logMel[i], max - 8) + 4) / 4;
  return features;
}
