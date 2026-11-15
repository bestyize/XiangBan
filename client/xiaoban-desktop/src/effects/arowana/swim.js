import { MathUtils, Matrix4, Quaternion, Vector3 } from 'three';

const UP = new Vector3(0, 1, 0);
const FORWARD = new Vector3(1, 0, 0);
const CYCLE = Math.PI * 2;
const CRUISE_SPEED = .62;
const BODY_TRIM = -.064;
const TURN_REACH = .82;
const TURN_LIMIT = .89;
// The swimming wave grows from the head toward the tail.
const spine = [
  { name: 'Head', stroke: .010 },
  { name: 'Shoulder', stroke: .020 },
  { name: 'SpineA', stroke: .042 },
  { name: 'SpineB', stroke: .074 },
  { name: 'Peduncle', stroke: .110 },
  { name: 'Tail', stroke: .144 },
  { name: 'TailTip', stroke: .136 },
];

function bindBone(object, name, rootRotation) {
  const bone = object.getObjectByName(name);
  if (!bone?.isBone) throw new Error(`Vermilion arowana rig is missing ${name}`);
  // Map the fish's axes into this bone's own rest orientation.
  const toBone = bone.getWorldQuaternion(new Quaternion()).invert().multiply(rootRotation);
  return {
    bone,
    rest: bone.quaternion.clone(),
    yaw: UP.clone().applyQuaternion(toBone),
    roll: FORWARD.clone().applyQuaternion(toBone),
  };
}

export function createSwimmer(object, route, { duration: requestedDuration } = {}) {
  object.updateWorldMatrix(true, true);
  const rootRotation = object.getWorldQuaternion(new Quaternion());
  const joints = spine.map(({ name, stroke }) => {
    const joint = bindBone(object, name, rootRotation);
    const station = object.worldToLocal(joint.bone.getWorldPosition(new Vector3())).x;
    return { ...joint, station, stroke };
  });
  const [, , frontTrunk, rearTrunk] = joints;
  const centerBlend = frontTrunk.station / (frontTrunk.station - rearTrunk.station);
  const fins = ['Pectoral', 'Pelvic'].flatMap((kind) => ['L', 'R'].map((suffix) => {
    const fin = bindBone(object, `${kind}_${suffix}`, rootRotation);
    const position = object.worldToLocal(fin.bone.getWorldPosition(new Vector3()));
    return { ...fin, side: Math.sign(position.z), span: kind === 'Pectoral' ? 1 : .55 };
  }));

  const defaultDuration = route.length / CRUISE_SPEED;
  const duration = requestedDuration ?? defaultDuration;
  if (!Number.isFinite(duration) || duration <= 0) throw new RangeError('Invalid Vermilion arowana swim duration');
  const speed = route.length / duration;
  // Stretch the strokes with the lap so a slower fish also beats its fins more slowly.
  const tailStrokes = Math.max(1, Math.round(defaultDuration * .54));
  const finStrokes = Math.max(1, Math.round(defaultDuration * .79));
  const startDistance = route.samples[route.startIndex].distance;
  const cadence = 3 * CYCLE;
  const speedVariation = .16;
  const motion = { center: new Vector3(), heading: new Vector3(), turn: 0, speed: 0, phase: 0 };
  const routePoint = new Vector3();
  const localHeading = new Vector3();
  const torsoCenter = new Vector3();
  const rearPosition = new Vector3();
  const forward = new Vector3();
  const side = new Vector3();
  const up = new Vector3();
  const orientation = new Matrix4();
  const rotation = new Quaternion();

  function update(timeSeconds) {
    const progress = MathUtils.euclideanModulo(timeSeconds / duration, 1);
    const glide = cadence * progress;
    // Integrate a periodic speed so distance, phase and speed all close at the loop seam.
    const travel = progress + speedVariation * Math.sin(glide) / cadence;
    const effort = .76 + .24 * Math.cos(glide);
    const phase = CYCLE * tailStrokes * travel;
    const distance = startDistance + travel * route.length;
    const curvature = route.sample(distance, motion.center, motion.heading);
    const reach = object.scale.x * TURN_REACH;
    const approachingTurn = route.sample(distance + .56 * reach, routePoint, localHeading);
    const steer = Math.tanh(1.5 * approachingTurn);
    const strokeEffort = effort * (1 - .18 * Math.abs(steer));
    const heading = motion.heading;
    const slope = Math.atan2(heading.y, Math.hypot(heading.x, heading.z));
    // The model's torso rises toward the snout. Trim that angle and follow the
    // route's gentle changes in depth with only a small change in body pitch.
    const pitch = BODY_TRIM + .27 * slope;
    forward.set(heading.x, 0, heading.z).normalize()
      .multiplyScalar(Math.cos(pitch)).addScaledVector(UP, Math.sin(pitch));
    side.crossVectors(forward, UP).normalize();
    up.crossVectors(side, forward);
    object.position.copy(motion.center);
    object.quaternion.setFromRotationMatrix(orientation.makeBasis(forward, up, side))
      .multiply(rotation.setFromAxisAngle(FORWARD, -.115 * steer + .009 * Math.sin(phase)));

    let parentTurn = 0;
    for (const [i, joint] of joints.entries()) {
      // Each section follows the heading where it lies along the route. The
      // head enters a bend first, and the tail is still turning on the exit.
      route.sample(distance + joint.station * reach, routePoint, localHeading);
      const yaw = Math.atan2(heading.z * localHeading.x - heading.x * localHeading.z,
        heading.x * localHeading.x + heading.z * localHeading.z);
      const turn = TURN_LIMIT * Math.tanh(yaw / TURN_LIMIT);
      const angle = turn - parentTurn + joint.stroke * strokeEffort * Math.sin(phase - .67 * i);
      joint.bone.quaternion.copy(joint.rest).multiply(rotation.setFromAxisAngle(joint.yaw, angle));
      parentTurn = turn;
    }

    // The rig pivots from the head. Keep the middle of the posed body on the route.
    frontTrunk.bone.getWorldPosition(torsoCenter).lerp(rearTrunk.bone.getWorldPosition(rearPosition), centerBlend);
    object.parent?.worldToLocal(torsoCenter);
    object.position.addScaledVector(torsoCenter.sub(motion.center), -1);

    for (const fin of fins) {
      const phaseOffset = fin.side > 0 ? .45 : 1.18;
      const stroke = CYCLE * finStrokes * travel + phaseOffset;
      const steering = fin.side * steer;
      const sweep = fin.side * fin.span * (.022 + .081 * effort * Math.sin(stroke) + .074 * steering);
      const feather = fin.side * fin.span * (.012 + .070 * effort * Math.sin(stroke + 1.1) + .042 * Math.max(0, steering));
      fin.bone.quaternion.copy(fin.rest)
        .multiply(rotation.setFromAxisAngle(fin.yaw, sweep))
        .multiply(rotation.setFromAxisAngle(fin.roll, feather));
    }

    motion.turn = curvature;
    motion.speed = speed * (1 + speedVariation * Math.cos(glide));
    motion.phase = phase;
    return motion;
  }

  update(0);
  return { update, getMotion: () => motion, duration };
}
