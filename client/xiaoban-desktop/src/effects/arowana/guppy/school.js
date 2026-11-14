import { MathUtils, Vector3 } from 'three';
import { createSwimRoute } from '../swim-route.js';
import { applyGuppyColorway } from './colorways.js';
import { createSwimDrive, driveAcceleration } from './swim-drive.js';

const { smoothstep } = MathUtils;
const TAU = 2 * Math.PI;
const DURATION = 43;
const SAMPLES = DURATION * 8;
const SUBSTEPS = 12;
const STEP = DURATION / (SAMPLES * SUBSTEPS);
const HEAD_STATION = .41;
const CLEARANCE_POINTS = [
  [-1.84, -.76, .15], [-1.84, 0, .23], [-1.84, .76, .15],
  [-1.27, -.61, .14], [-1.28, .02, .23], [-1.27, .61, .14],
  [-.55, 0, .20], [.02, -.03, .25], [.53, 0, .22],
];

const shoalPath = createSwimRoute([
  [-3.82, -.18, 1.27], [-2.67, -.14, 2.17], [-1.11, -.25, 2.40], [.68, -.12, 2.31],
  [2.33, .04, 1.92], [3.49, .20, 1.00], [3.95, .22, -.39], [3.39, .13, -1.82],
  [2.19, .02, -2.50], [.50, .06, -2.25], [-.94, .17, -1.88], [-2.45, .32, -2.01],
  [-3.83, .13, -1.65], [-4.37, -.04, -.27],
], 2.5);

const RESIDENTS = [
  { colorway: 'blue',     scale: .61, along:  2.15, side:  .12, height: -.20, phase: .01 },
  { colorway: 'red',      scale: .58, along:   .79, side: -.87, height:  .88, phase: .58 },
  { colorway: 'leopard',  scale: .53, along:   .60, side:  .99, height: -.52, phase: .87 },
  { colorway: 'platinum', scale: .55, along:  -.88, side: -.46, height: -1.03, phase: .28 },
  { colorway: 'red',      scale: .51, along: -1.94, side: -1.06, height:  .18, phase: .67 },
  { colorway: 'blue',     scale: .56, along: -2.15, side:  .89, height:  .66, phase: .18 },
];

function guideProgress(time) {
  const phase = TAU * time / DURATION;
  return phase / TAU + .006 * Math.sin(3 * phase - .65) + .003 * Math.sin(5 * phase + .4);
}

function yieldToNeighbour(state, difference, force) {
  state.separation.addScaledVector(difference, force)
    .addScaledVector(state.heading, -.94 * force * difference.dot(state.heading));
}

function schoolTracks(drives) {
  const states = RESIDENTS.map((member, i) => ({
    position: new Vector3(), velocity: new Vector3(), target: new Vector3(),
    targetVelocity: new Vector3(), force: new Vector3(), separation: new Vector3(), track: [],
    heading: new Vector3(), guide: new Vector3(), stations: CLEARANCE_POINTS.map(() => new Vector3()),
    drive: drives[i], stroke: {}, thrustGain: (1.95 + .20 * Math.sin(member.phase * TAU)) * member.scale,
  }));
  const position = new Vector3();
  const tangent = new Vector3();
  const lateral = new Vector3();
  const delta = new Vector3();
  const difference = new Vector3();
  const average = new Vector3(), offset = new Vector3();
  const start = shoalPath.samples[shoalPath.startIndex].distance;

  function targets(time) {
    const progress = guideProgress(time);
    const angle = TAU * time / DURATION;
    for (const [i, member] of RESIDENTS.entries()) {
      const angleOffset = member.phase * TAU;
      const follow = .30 * Math.sin(2 * angle + angleOffset) + .19 * Math.sin(3 * angle - angleOffset);
      const distance = start + shoalPath.length * progress + member.along + HEAD_STATION * member.scale + follow;
      shoalPath.sample(distance, position, tangent);
      lateral.set(-tangent.z, 0, tangent.x).normalize();
      const width = 1 + .23 * Math.sin(2 * angle - 1.33);
      const side = member.side * width + .15 * Math.sin(angle + angleOffset);
      const height = member.height + .16 * Math.sin(2 * angle - angleOffset);
      states[i].guide.copy(tangent);
      states[i].target.copy(position).addScaledVector(lateral, side);
      states[i].target.y += height;
    }
  }

  targets(0);
  for (const state of states) state.position.copy(state.target);

  const n = SAMPLES * SUBSTEPS;
  // Settle the preceding laps so the tracks begin with the shoal in motion.
  // The periodic spline through the resulting samples can be scrubbed.
  for (let frame = -2 * n; frame < n; frame++) {
    for (const state of states) state.targetVelocity.copy(state.target);
    targets(frame * STEP);
    average.set(0, 0, 0);
    for (const state of states) average.add(state.position);
    average.divideScalar(states.length);
    for (const [i, state] of states.entries()) {
      const member = RESIDENTS[i];
      state.targetVelocity.subVectors(state.target, state.targetVelocity).multiplyScalar(1 / STEP);
      // The guide supplies a steering target; a swimmer still accelerates
      // in response to its own half-strokes against drag.
      const stiffness = 3.14 + .13 * (i % 3);
      state.force.subVectors(state.target, state.position).multiplyScalar(stiffness);
      delta.subVectors(state.targetVelocity, state.velocity);
      const paceError = delta.dot(state.guide);
      state.force.addScaledVector(delta, 3.38).addScaledVector(state.guide, -3.38 * paceError);
      state.force.addScaledVector(offset.subVectors(average, state.position), .058);
      state.separation.set(0, 0, 0);
      state.heading.copy(state.targetVelocity).normalize();
      state.drive.sample((frame + .5) * STEP, state.stroke);
      state.force.addScaledVector(state.guide,
        driveAcceleration(0, paceError, state.stroke.thrust, state.thrustGain, 2.34));
      CLEARANCE_POINTS.forEach(([along, rise], j) => {
        const point = state.stations[j];
        point.copy(state.position).addScaledVector(state.heading, (along - HEAD_STATION) * member.scale);
        point.y += rise * member.scale;
      });
    }

    for (let i = 0; i < states.length - 1; i++) {
      const a = states[i];
      const aSize = RESIDENTS[i].scale;
      for (let j = i + 1; j < states.length; j++) {
        const b = states[j];
        const bSize = RESIDENTS[j].scale;
        // Measure clearance around the upper and lower edges of the fan too.
        let pressure = 0;
        let gap = 1;
        for (const [p, near] of a.stations.entries()) {
          for (const [q, far] of b.stations.entries()) {
            offset.subVectors(near, far);
            const distance = offset.length();
            const clearance = .35 + CLEARANCE_POINTS[p][2] * aSize + CLEARANCE_POINTS[q][2] * bSize;
            const overlap = 1 - distance / clearance;
            if (distance < .00001 || overlap <= pressure) continue;
            pressure = overlap;
            gap = distance;
            difference.copy(offset);
          }
        }
        if (pressure > 0) {
          // Give way across the direction of swimming so nobody has to stop
          // or reverse direction when two tails get close.
          const strength = pressure * pressure * 28 / gap;
          yieldToNeighbour(a, difference, strength);
          yieldToNeighbour(b, difference, -strength);
        }
        const alignment = .13 * (1 - smoothstep(a.position.distanceTo(b.position), 1.4, 2.9));
        delta.subVectors(b.velocity, a.velocity).multiplyScalar(alignment);
        a.force.add(delta);
        b.force.sub(delta);
      }
    }

    for (const state of states) {
      if (frame >= 0 && frame % SUBSTEPS === 0) {
        state.track.push(state.position.toArray());
      }
      state.force.add(state.separation).clampLength(0, 4.9);
      state.velocity.addScaledVector(state.force, STEP).clampLength(0, 1.72);
      state.position.addScaledVector(state.velocity, STEP);
    }
  }
  return states.map(state => state.track);
}

export function createGuppySchool(createFish, group) {
  const drives = RESIDENTS.map(member => createSwimDrive(DURATION, member.phase));
  const tracks = schoolTracks(drives);
  const fish = RESIDENTS.map((member, i) => {
    const route = createSwimRoute(tracks[i], 0, { samplesPerWaypoint: 6 });
    const swimmer = createFish(route, { scale: member.scale, phase: member.phase, drive: drives[i] });
    applyGuppyColorway(swimmer.object, member.colorway, 3.7 * (i + 1));
    group.add(swimmer.object);
    return swimmer;
  });

  function update(time) {
    for (const swimmer of fish) swimmer.update(time);
  }
  return { object: group, count: fish.length, duration: DURATION, update };
}
