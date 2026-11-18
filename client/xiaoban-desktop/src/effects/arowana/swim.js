import { MathUtils, Matrix4, Quaternion, Vector3 } from 'three';
import { createContinuousBody, turnAcrossBody } from './continuous-body.js';
import { createTurnResponse } from './turn-response.js';
import { bindGillBreathing } from './gill-breathing.js';

const UP = new Vector3(0, 1, 0);
const FORWARD = new Vector3(1, 0, 0);
const PITCH_AXIS = new Vector3(0, 0, 1);
const CYCLE = Math.PI * 2;
const CRUISE_SPEED = .62;
const BODY_TRIM = -.064;
const TURN_REACH = .82;
const TURN_LIMIT = .89;
const PITCH_LIMIT = .38;
const spine = ['Head', 'Shoulder', 'SpineA', 'SpineB', 'Peduncle', 'Tail', 'TailTip'];

function incline(direction) {
  return Math.atan2(direction.y, Math.hypot(direction.x, direction.z));
}

function bindBone(object, name, rootRotation) {
  const bone = object.getObjectByName(name);
  if (!bone?.isBone) throw new Error(`Vermilion arowana rig is missing ${name}`);
  // Map the fish's axes into this bone's own rest orientation.
  const toBone = bone.getWorldQuaternion(new Quaternion()).invert().multiply(rootRotation);
  return {
    bone,
    yaw: UP.clone().applyQuaternion(toBone),
    roll: FORWARD.clone().applyQuaternion(toBone),
  };
}

export function createSwimmer(object, route, { duration: requestedDuration } = {}) {
  object.updateWorldMatrix(true, true);
  const rootRotation = object.getWorldQuaternion(new Quaternion());
  const joints = spine.map(name => {
    const bone = object.getObjectByName(name);
    if (!bone?.isBone) throw new Error(`Vermilion arowana rig is missing ${name}`);
    const station = object.worldToLocal(bone.getWorldPosition(new Vector3())).x;
    return { bone, station };
  });
  const firstStation = joints[0].station;
  const lastStation = joints.at(-1).station;
  const stations = joints.map(({ station }) => (firstStation - station) / (firstStation - lastStation));
  const bend = createTurnResponse(stations, object.scale.x, .31);
  for (const [i, joint] of joints.entries()) {
    const along = stations[i];
    joint.waveDelay = 4.8 * along;
    joint.waveAmplitude = .079 * along ** 2.65;
    joint.waveSlope = .079 * 2.65 * along ** 1.65;
  }
  const routeTurns = new Float64Array(joints.length);
  const routePitches = new Float64Array(joints.length);
  const fins = ['Pectoral', 'Pelvic'].flatMap((kind) => ['L', 'R'].map((suffix) => {
    const fin = bindBone(object, `${kind}_${suffix}`, rootRotation);
    const position = object.worldToLocal(fin.bone.getWorldPosition(new Vector3()));
    return { ...fin, side: Math.sign(position.z), span: kind === 'Pectoral' ? 1 : .55 };
  }));
  const body = createContinuousBody(object, joints.map(joint => joint.bone), fins.map(fin => fin.bone), 1.08);
  const poseGills = bindGillBreathing(object);

  const defaultDuration = route.length / CRUISE_SPEED;
  const duration = requestedDuration ?? defaultDuration;
  if (!Number.isFinite(duration) || duration <= 0) throw new RangeError('Invalid Vermilion arowana swim duration');
  const speed = route.length / duration;
  // Stretch the strokes with the lap so a slower fish also beats its fins more slowly.
  const tailStrokes = Math.max(1, Math.round(defaultDuration * .54));
  const finStrokes = Math.max(1, Math.round(defaultDuration * .79));
  const gillStrokes = Math.max(1, Math.round(defaultDuration * .95));
  const startDistance = route.samples[route.startIndex].distance;
  const cadence = 3 * CYCLE;
  const speedVariation = .16;
  const motion = { center: new Vector3(), heading: new Vector3(), turn: 0, speed: 0, phase: 0 };
  const routePoint = new Vector3();
  const localHeading = new Vector3();
  const earlierHeading = new Vector3();
  const torsoCenter = new Vector3();
  const forward = new Vector3();
  const side = new Vector3();
  const up = new Vector3();
  const orientation = new Matrix4();
  const rotation = new Quaternion();

  const reach = object.scale.x * TURN_REACH;
  const cruise = {};

  function sampleCruise(timeSeconds, out) {
    const progress = MathUtils.euclideanModulo(timeSeconds / duration, 1);
    const glide = cadence * progress;
    // Integrate a periodic speed so distance, phase and speed all close at the loop seam.
    const travel = progress + speedVariation * Math.sin(glide) / cadence;
    out.travel = travel;
    out.phase = CYCLE * tailStrokes * travel;
    out.effort = .76 + .24 * Math.cos(glide);
    out.distance = startDistance + travel * route.length;
    out.speed = speed * (1 + speedVariation * Math.cos(glide));
  }

  function update(timeSeconds) {
    sampleCruise(timeSeconds, cruise);
    const { travel, phase, effort, distance } = cruise;
    const sample = route.sample;
    const curvature = sample(distance, motion.center, motion.heading);
    bend.update(timeSeconds, sample, sampleCruise);
    const approachingTurn = turnAcrossBody(sample, distance + .56 * reach, .38 * reach,
      routePoint, earlierHeading, localHeading);
    const steer = MathUtils.lerp(Math.tanh(1.5 * approachingTurn), bend.tailTurn, .22);
    const strokeEffort = (.48 + .43 * effort) * (1 - .16 * Math.abs(steer));
    const heading = motion.heading;
    const slope = incline(heading);
    const pitch = BODY_TRIM + PITCH_LIMIT * Math.tanh(1.55 * slope / PITCH_LIMIT);
    forward.set(heading.x, 0, heading.z).normalize()
      .multiplyScalar(Math.cos(pitch)).addScaledVector(UP, Math.sin(pitch));
    side.crossVectors(forward, UP).normalize();
    up.crossVectors(side, forward);
    object.position.copy(motion.center);
    object.quaternion.setFromRotationMatrix(orientation.makeBasis(forward, up, side))
      .multiply(rotation.setFromAxisAngle(FORWARD, -.115 * steer + .009 * Math.sin(phase)));

    for (const [i, joint] of joints.entries()) {
      sample(distance + joint.station * reach, routePoint, localHeading);
      const yaw = Math.atan2(heading.z * localHeading.x - heading.x * localHeading.z,
        heading.x * localHeading.x + heading.z * localHeading.z);
      routeTurns[i] = TURN_LIMIT * Math.tanh(yaw / TURN_LIMIT);
      routePitches[i] = .095 * Math.tanh(5.5 * (incline(localHeading) - slope));
    }

    // Follow the tangent of one lateral wave. Its displacement grows
    // toward the tail and the head is nearly still during a glide.
    for (const [i, joint] of joints.entries()) {
      const turn = routeTurns[i];
      const wave = phase - joint.waveDelay;
      const heading = turn + bend.angles[i] + Math.atan(strokeEffort * (
        joint.waveSlope * Math.sin(wave) - 4.8 * joint.waveAmplitude * Math.cos(wave)));
      body.frames[i].setFromAxisAngle(UP, heading)
        .multiply(rotation.setFromAxisAngle(PITCH_AXIS, routePitches[i]));
    }

    body.pose();
    object.position.sub(torsoCenter.copy(body.center).multiply(object.scale).applyQuaternion(object.quaternion));
    const breathing = .14 * (.5 + .5 * Math.sin(CYCLE * gillStrokes * travel)) ** 1.7;
    poseGills(breathing);

    for (const fin of fins) {
      const phaseOffset = fin.side > 0 ? .45 : 1.18;
      const stroke = CYCLE * finStrokes * travel + phaseOffset;
      const steering = fin.side * steer;
      const sweep = fin.side * fin.span * (.022 + .081 * effort * Math.sin(stroke) + .074 * steering);
      const feather = fin.side * fin.span * (.012 + .070 * effort * Math.sin(stroke + 1.1) + .042 * Math.max(0, steering));
      fin.bone.quaternion.multiply(rotation.setFromAxisAngle(fin.yaw, sweep))
        .multiply(rotation.setFromAxisAngle(fin.roll, feather));
    }

    motion.turn = curvature;
    motion.speed = cruise.speed;
    motion.phase = phase;
    return motion;
  }

  update(0);
  return { update, getMotion: () => motion, duration };
}
