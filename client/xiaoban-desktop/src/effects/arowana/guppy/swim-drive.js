import { MathUtils } from 'three';

const { lerp, smoothstep, euclideanModulo: wrap } = MathUtils;
const TAU = 2 * Math.PI;

// These are groups of strokes with room between them to coast. Both the
// animation and the swimmer's forward acceleration use this same signal.
export function createSwimDrive(duration, phase) {
  const count = Math.max(1, Math.round(duration / 2.8));
  const interval = duration / count;
  const seed = TAU * phase;
  const cycle = Array.from({ length: count }, (_, i) => {
    const angle = TAU * i / count;
    const variation = Math.sin(3 * angle + 1.8 * seed + .8);
    const width = interval * (.40 + .073 * Math.sin(5 * angle - seed + 1.4));
    const start = interval * (i + .25 * variation - phase);
    const strength = .76 + .22 * Math.sin(2 * angle + seed + .6);
    return { start, width, strength };
  });
  const frames = Math.ceil(duration * 120);
  const dt = duration / frames;
  const efforts = new Float32Array(frames + 1);
  const beats = new Float64Array(frames + 1);
  let priorRate = 0;
  for (let i = 0; i <= frames; i++) {
    const t = i * dt;
    let effort = 0;
    for (const { start, width, strength } of cycle) {
      const u = wrap(t - start, duration) / width;
      if (u < 1) effort += strength * smoothstep(u, 0, .22) * (1 - smoothstep(u, .58, 1));
    }
    const rate = (.99 + 1.53 * effort) * (1 + .085 * Math.sin(seed + .62));
    if (i) beats[i] = beats[i - 1] + .5 * (priorRate + rate) * dt;
    efforts[i] = effort;
    priorRate = rate;
  }
  const radiansPerBeat = TAU * Math.max(1, Math.round(beats[frames])) / beats[frames];

  function sample(time, out) {
    const frame = wrap(time, duration) / dt;
    const i = Math.min(frames - 1, Math.floor(frame));
    const u = frame - i;
    out.effort = lerp(efforts[i], efforts[i + 1], u);
    out.phase = seed + radiansPerBeat * lerp(beats[i], beats[i + 1], u);
    // A force peak on each half-stroke, just after the tail sweeps sideways.
    out.thrust = out.effort * (.67 + .33 * Math.cos(2 * out.phase - 1.28));
  }

  return { duration, dt, frames, sample };
}

export function createCruise(drive, length, start) {
  const { dt, frames, duration } = drive;
  const stroke = {};
  const distances = new Float64Array(frames + 1);
  let speed = .61;
  // Drag settles over the preceding lap, so the end joins the start at speed.
  for (let i = -frames; i < frames; i++) {
    drive.sample((i + .5) * dt, stroke);
    const next = speed + dt * driveAcceleration(speed, .54, stroke.thrust, 1.13);
    if (i >= 0) distances[i + 1] = distances[i] + .5 * (speed + next) * dt;
    speed = next;
  }
  const factor = length / distances[frames];

  return (time, out) => {
    const frame = wrap(time, duration) / dt;
    const i = Math.min(frames - 1, Math.floor(frame));
    out.distance = start + factor * lerp(distances[i], distances[i + 1], frame - i);
    out.speed = factor * (distances[i + 1] - distances[i]) / dt;
  };
}

// A stroke supplies extra force; releasing it leaves momentum to coast.
export function driveAcceleration(speed, pace, thrust, gain, resistance = 1.14) {
  return resistance * (pace - speed) + gain * thrust;
}
