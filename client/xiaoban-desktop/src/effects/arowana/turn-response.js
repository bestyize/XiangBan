import { MathUtils, Vector3 } from 'three';
import { turnAcrossBody } from './continuous-body.js';

const STEP = .09;
const STEPS = 22;
const MAX_BEND = .46;
const { lerp, smoothstep } = MathUtils;
export const TURN_RESPONSE_HISTORY = STEPS * STEP;

export function createTurnResponse(stations, bodyScale, ahead) {
  // The front settles promptly. The rear retains the turn and has a small
  // opposite bend as angular momentum is shed after a strong manoeuvre.
  const weights = stations.map(along => {
    const lag = lerp(.075, .205, along);
    const damping = lerp(1, .47, along ** 1.35);
    const frequency = Math.sqrt(1 - damping * damping);
    const samples = Array.from({ length: STEPS }, (_, i) => {
      const elapsed = (i + .5) * STEP;
      const t = elapsed / lag;
      const wave = frequency ? Math.sin(frequency * t) / frequency : t;
      const fade = 1 - smoothstep(elapsed, TURN_RESPONSE_HISTORY - .27, TURN_RESPONSE_HISTORY);
      return Math.exp(-damping * t) * wave * fade;
    });
    const sum = samples.reduce((total, weight) => total + weight, 0);
    return samples.map(weight => weight / sum);
  });
  const loads = new Float64Array(STEPS);
  const angles = new Float64Array(stations.length);
  const pace = {};
  const point = new Vector3();
  const earlier = new Vector3();
  const later = new Vector3();

  function update(time, sample, samplePace) {
    for (let i = 0; i < loads.length; i++) {
      samplePace(time - (i + .5) * STEP, pace);
      const bend = turnAcrossBody(sample, pace.distance + ahead * bodyScale, .31 * bodyScale,
        point, earlier, later);
      const tight = smoothstep(Math.abs(bend) * bodyScale, .35, 1.12);
      const speed = smoothstep(pace.speed / bodyScale, .24, .90);
      loads[i] = Math.sign(bend) * tight * lerp(.76, 1, speed);
    }

    for (let joint = 0; joint < stations.length; joint++) {
      let load = 0;
      for (let i = 0; i < loads.length; i++) load += loads[i] * weights[joint][i];
      angles[joint] = -MAX_BEND * stations[joint] ** 1.35 * load;
    }
  }

  return {
    angles, update,
    get tailTurn() { return MathUtils.clamp(-angles.at(-1) / MAX_BEND, -1, 1); },
  };
}
