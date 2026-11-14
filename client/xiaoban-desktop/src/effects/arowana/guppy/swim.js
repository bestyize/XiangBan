import { MathUtils, Matrix4, Quaternion, Vector3 } from 'three';
import { createCaudalFin } from './caudal-fin.js';
import { createCruise, createSwimDrive } from './swim-drive.js';

const TAU = 2 * Math.PI;
const UP = new Vector3(0, 1, 0);
const FORWARD = new Vector3(1, 0, 0);
const { clamp, euclideanModulo: wrap, lerp } = MathUtils;
const SPINE = ['Head', 'Shoulder', 'Spine_1', 'Spine_2', 'Spine_3', 'Spine_4', 'Peduncle', 'TailBase'];

function bindTransform(node, inverse) {
  const matrix = new Matrix4().multiplyMatrices(inverse, node.matrixWorld);
  const position = new Vector3();
  const rotation = new Quaternion();
  matrix.decompose(position, rotation, new Vector3());
  return { position, rotation };
}

function requireBone(object, name) {
  const bone = object.getObjectByName(name);
  if (!bone?.isBone) throw new Error(`Blue guppy skeleton is missing ${name}`);
  return bone;
}

function yawBetween(a, b) {
  return Math.atan2(a.z * b.x - a.x * b.z, a.x * b.x + a.z * b.z);
}

// Group tracks record positions at regular times. The route stores arc
// lengths at uniform parameter intervals, so seeking uses those same times.
function recordedPace(route, duration) {
  return (time, out) => {
    const frame = wrap(time, duration) * (route.samples.length - 1) / duration;
    const i = Math.min(route.samples.length - 2, Math.floor(frame));
    const a = route.samples[i].distance;
    const b = route.samples[i + 1].distance;
    out.distance = lerp(a, b, frame - i);
    out.speed = (b - a) * (route.samples.length - 1) / duration;
  };
}

export function createSwimmer(object, route, { phase = 0, drive: groupDrive } = {}) {
  const scale = Math.abs(object.scale.x);
  const duration = groupDrive?.duration ?? route.length / (.70 * scale);
  const drive = groupDrive ?? createSwimDrive(duration, phase);
  const pace = groupDrive ? recordedPace(route, duration)
    : createCruise(drive, route.length, route.samples[route.startIndex].distance);
  const finBeats = Math.round(duration * 3.1);
  const phaseOffset = TAU * phase;
  const basis = new Matrix4();
  const anchor = new Vector3();
  const forward = new Vector3();
  const up = new Vector3();
  const lateral = new Vector3();
  const headTangent = new Vector3();
  const turnEntry = new Vector3();
  const turnExit = new Vector3();
  const trailTangent = new Vector3();
  const scratch = new Vector3();
  const rotation = new Quaternion();
  const inverseRoot = new Quaternion();
  const inverseParent = new Quaternion();
  const motion = {
    head: new Vector3(), center: new Vector3(), heading: new Vector3(),
    turn: 0, speed: 0, phase: 0, effort: 0, thrust: 0,
  };

  object.updateWorldMatrix(true, true);
  const inverseBind = object.matrixWorld.clone().invert();
  const joints = SPINE.map(name => {
    const bone = requireBone(object, name);
    return {
      bone, ...bindTransform(bone, inverseBind),
      parentBind: bindTransform(bone.parent, inverseBind).rotation,
      inModel: new Quaternion(), pose: new Quaternion(),
    };
  });
  const stations = [0];
  for (let i = 1; i < joints.length; i++) {
    const joint = joints[i];
    const prior = joints[i - 1];
    joint.parentOffset = prior.rotation.clone().invert().multiply(joint.parentBind);
    stations.push(stations[i - 1] + joint.position.distanceTo(prior.position));
  }
  // Extend the trunk's tangent into the tail root before water bends the fan.
  const trunkLength = stations.at(-1);
  stations.push(trunkLength + .16);
  const length = stations.at(-1);
  const offsets = new Float64Array(stations.length);

  const pectorals = [[1, 'Pectoral_L', .45], [-1, 'Pectoral_R', 2.32]]
    .map(([side, name, offset]) => {
      const bone = requireBone(object, name);
      const inverse = bindTransform(bone, inverseBind).rotation.invert();
      return {
        bone, side, offset, rest: bone.quaternion.clone(),
        yaw: UP.clone().applyQuaternion(inverse),
        roll: FORWARD.clone().applyQuaternion(inverse),
      };
    });
  const membranes = [
    ['Dorsal_sail', .12, .42, .55, .43, .73],
    ['Anal_fin', .085, .34, .48, .34, 1.02],
    ['Pelvic_fin_L', .098, .23, .38, .29, .88],
    ['Pelvic_fin_R', .126, .23, .37, .29, 1.19],
    ['Pectoral_membrane_L', .068, .55, .28, .13, .45],
    ['Pectoral_membrane_R', .079, .53, .28, .13, 2.32],
  ].map(([name, lag, response, ripple, drag, offset]) => {
    const mesh = object.getObjectByName(name);
    const indices = mesh?.morphTargetDictionary;
    if (!indices || [indices.Flex, indices.WaveA, indices.WaveB].some(index => index === undefined)) {
      throw new Error(`Blue guppy mesh is missing fin deformation on ${name}`);
    }
    return {
      weights: mesh.morphTargetInfluences, indices, lag, response, ripple, drag, offset,
      pectoral: name.startsWith('Pectoral'),
    };
  });

  const head = {};
  const delayed = {};
  const travel = {};
  function poseBody(time) {
    drive.sample(time, head);
    pace(time, travel);
    const s = travel.distance;
    route.sample(s, anchor, headTangent);
    route.sample(s + .25 * scale, scratch, turnEntry);
    route.sample(s - .50 * scale, scratch, turnExit);
    const turn = Math.tanh(1.9 * yawBetween(turnExit, turnEntry));
    headTangent.lerp(turnEntry, .45).normalize();
    route.sample(s - 1.09 * scale, scratch, trailTangent);
    const arc = 1.22 * Math.tanh((yawBetween(headTangent, trailTangent) - .32 * turn) / 1.13);
    const angle = TAU * wrap(time, duration) / duration;
    const bank = -.097 * turn + .015 * Math.sin(2 * angle + .2);

    // Offset one continuous midline with a wave whose reach grows toward
    // the tail. The pulse itself also arrives later along the body.
    for (let i = 0; i < stations.length; i++) {
      const u = stations[i] / length;
      drive.sample(time - .085 * u, delayed);
      const effort = lerp(head.effort, delayed.effort, .68);
      const envelope = (.019 * u + .157 * u ** 2.08) *
        (.17 + 1.03 * effort + .13 * Math.abs(turn));
      const wave = head.phase - 3.78 * u;
      offsets[i] = envelope * (Math.sin(wave) + .084 * Math.sin(2 * wave - .5));
    }

    for (let i = 0; i < joints.length; i++) {
      const u = (stations[i] + stations[i + 1]) / (2 * length);
      // Mix the local tangent history with a broad bend over the whole
      // trunk so a tight patch of route cannot kink a single joint.
      route.sample(s - stations[i] * scale, scratch, forward);
      const trail = .99 * Math.tanh(yawBetween(headTangent, forward) / .99);
      const along = stations[i] / trunkLength;
      const steering = i === 0 ? .018 * turn : .29 * trail + .71 * arc * along * (.64 + .36 * along);
      const wave = Math.atan2(offsets[i + 1] - offsets[i], stations[i + 1] - stations[i]);
      const yaw = steering + wave * (1 - .28 * Math.abs(turn));
      forward.copy(headTangent).applyAxisAngle(UP, yaw);
      lateral.crossVectors(forward, UP).normalize();
      up.crossVectors(lateral, forward);
      const roll = bank * (1 - .45 * u) + .040 * u * head.effort * Math.sin(head.phase - 3.2 * u);
      joints[i].pose.setFromRotationMatrix(basis.makeBasis(forward, up, lateral))
        .multiply(rotation.setFromAxisAngle(FORWARD, roll));
    }

    object.quaternion.copy(joints[0].pose);
    object.position.copy(joints[0].position).multiply(object.scale)
      .applyQuaternion(object.quaternion).negate().add(anchor);
    inverseRoot.copy(object.quaternion).invert();

    // Put the midline's frames into the rig's local coordinates. Only
    // rotations change, so every segment retains its original length.
    for (const [i, joint] of joints.entries()) {
      joint.inModel.copy(inverseRoot).multiply(joint.pose).multiply(joint.rotation);
      inverseParent.copy(i ? joints[i - 1].inModel : joint.parentBind);
      if (i) inverseParent.multiply(joint.parentOffset);
      joint.bone.quaternion.copy(inverseParent.invert()).multiply(joint.inModel);
    }

    motion.head.copy(anchor);
    motion.center.copy(object.position);
    motion.heading.copy(FORWARD).applyQuaternion(object.quaternion);
    motion.speed = travel.speed;
    motion.turn = turn;
    motion.phase = head.phase;
    motion.effort = head.effort;
    motion.thrust = head.thrust;
    return head.effort;
  }

  const tailMesh = object.getObjectByName('Caudal_fan');
  const caudalFin = createCaudalFin(tailMesh, joints.at(-1).bone, duration, time => {
    const effort = poseBody(time);
    object.updateMatrixWorld(true);
    return effort;
  });

  function pectoralPhase(time) {
    const angle = TAU * wrap(time, duration) / duration;
    return finBeats * angle + .46 * Math.sin(3 * angle + .38) + phaseOffset;
  }

  function update(time) {
    const effort = poseBody(time);
    caudalFin(time);
    const { turn } = motion;
    for (const fin of pectorals) {
      const stroke = pectoralPhase(time) + fin.offset;
      const trim = (.35 + .56 * (1 - effort)) * (1 + .28 * fin.side * turn);
      const pull = .5 + .5 * Math.sin(stroke);
      fin.bone.quaternion.copy(fin.rest)
        .multiply(rotation.setFromAxisAngle(fin.yaw, -fin.side * (.23 * trim * pull ** 1.5 + .07 * turn)))
        .multiply(rotation.setFromAxisAngle(fin.roll, fin.side * trim * (.12 + .26 * Math.sin(stroke - .70))));
    }

    for (const fin of membranes) {
      const finTime = time - fin.lag;
      drive.sample(finTime, delayed);
      const beat = fin.pectoral ? pectoralPhase(finTime) + fin.offset : delayed.phase - fin.offset;
      const load = fin.pectoral ? .31 + .58 * (1 - delayed.effort) : .24 + .59 * delayed.effort;
      const bend = fin.response * load * Math.sin(beat - .86) - fin.drag * turn * .52;
      const wave = fin.ripple * load * (1 + .15 * Math.abs(turn));
      const { Flex, WaveA, WaveB } = fin.indices;
      fin.weights[Flex] = clamp(bend, -.94, .94);
      fin.weights[WaveA] = wave * Math.sin(beat);
      fin.weights[WaveB] = -wave * Math.cos(beat);
    }
    return motion;
  }

  update(0);
  return { update, duration, getMotion: () => motion };
}
