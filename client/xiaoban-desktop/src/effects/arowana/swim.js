import * as THREE from 'three';
import { lowerIndex } from './swim-route.js';

const TAU = 2 * Math.PI;
const X = new THREE.Vector3(1, 0, 0);
const Y = new THREE.Vector3(0, 1, 0);
const JOINTS = [
  'Head', 'Spine_1', 'Spine_2', 'Spine_3', 'Spine_4', 'Peduncle', 'Tail', 'Tail_Tip',
];

const wrap = THREE.MathUtils.euclideanModulo;
const lerp = THREE.MathUtils.lerp;
const mix = THREE.MathUtils.smoothstep;

function buildTiming(route) {
  let elapsed = 0;
  let cycles = 0;
  const samples = [];
  for (const [i, sample] of route.samples.entries()) {
    const angle = TAU * sample.distance / route.length;
    const turn = mix(Math.abs(sample.curvature), .20, 1.4);
    const effort = .48 + .29 * Math.sin(4 * angle - .75)
      + .15 * Math.sin(7 * angle + .55) + .08 * Math.cos(2 * angle + .8);
    const speed = .52 + .245 * effort - .105 * turn;
    const cadence = .56 + .54 * effort + .10 * turn;
    if (i) {
      const prior = samples[i - 1];
      const dt = 2 * (sample.distance - prior.distance) / (prior.speed + speed);
      elapsed += dt;
      cycles += .5 * dt * (prior.cadence + cadence);
    }
    samples.push({ ...sample, time: elapsed, cycles, speed, cadence, effort });
  }

  return {
    samples, duration: elapsed, cycles,
    startTime: samples[route.startIndex].time,
  };
}

function bindTransform(node, rootInverse) {
  const matrix = new THREE.Matrix4().multiplyMatrices(rootInverse, node.matrixWorld);
  const position = new THREE.Vector3();
  const rotation = new THREE.Quaternion();
  matrix.decompose(position, rotation, new THREE.Vector3());
  return { position, rotation };
}

function bindFin(scene, name, side, type, rootInverse) {
  const bone = scene.getObjectByName(name);
  if (!bone) return null;
  const inverseParent = bindTransform(bone.parent, rootInverse).rotation.invert();
  return {
    bone, side, type, bind: bone.quaternion.clone(),
    sweep: Y.clone().applyQuaternion(inverseParent),
    feather: X.clone().applyQuaternion(inverseParent),
  };
}

/**
 * Animate a glTF rig with +X toward the nose, +Y up and +Z toward the front
 * of the aquarium. The caller provides the route and drives update in seconds.
 */
export function createSwimmer(scene, route) {
  const { samples, duration, cycles, startTime } = buildTiming(route);
  const wholeCycles = Math.round(cycles);
  scene.updateWorldMatrix(true, true);
  const rootInverse = scene.matrixWorld.clone().invert();
  const joints = JOINTS.map((name) => {
    const bone = scene.getObjectByName(name);
    if (!bone?.isBone) throw new Error(`Arowana skeleton is missing ${name}`);
    return {
      bone,
      ...bindTransform(bone, rootInverse),
      parentBind: bindTransform(bone.parent, rootInverse).rotation,
      rotationInModel: new THREE.Quaternion(),
      pose: new THREE.Quaternion(),
    };
  });
  for (let i = 1; i < joints.length; i++) {
    const joint = joints[i];
    joint.parentOffset = joints[i - 1].rotation.clone().invert().multiply(joint.parentBind);
  }

  const fins = ['Pectoral', 'Pelvic', 'Gill'].flatMap((type) => [
    bindFin(scene, `${type}_L`, 1, type, rootInverse),
    bindFin(scene, `${type}_R`, -1, type, rootInverse),
  ]).filter(Boolean);

  const distances = [0];
  const bodyScale = scene.scale.x;
  for (let i = 1; i < joints.length; i++) {
    distances.push(distances[i - 1] + bodyScale *
      joints[i].position.distanceTo(joints[i - 1].position));
  }
  // The trailing end of the fin continues beyond the final joint pivot.
  distances.push(distances.at(-1) + .23 * bodyScale);
  const length = distances.at(-1);
  const points = distances.map(() => new THREE.Vector3());
  const curveTurns = distances.map(() => 0);
  const headTangent = new THREE.Vector3();
  const tangent = new THREE.Vector3();
  const lateral = new THREE.Vector3();
  const forward = new THREE.Vector3();
  const up = new THREE.Vector3();
  const matrix = new THREE.Matrix4();
  const rootRotationInverse = new THREE.Quaternion();
  const axisRotation = new THREE.Quaternion();
  const parentRotationInverse = new THREE.Quaternion();
  const motion = {
    head: new THREE.Vector3(), center: new THREE.Vector3(), heading: new THREE.Vector3(),
    turn: 0, speed: 0, phase: 0,
  };

  function update(timeSeconds) {
    const t = wrap(timeSeconds + startTime, duration);
    const index = lowerIndex(samples, 'time', t);
    const a = samples[index];
    const b = samples[index + 1];
    const u = (t - a.time) / (b.time - a.time);
    const distance = lerp(a.distance, b.distance, u);
    const effort = lerp(a.effort, b.effort, u);
    const progress = t / duration;
    const phase = TAU * wholeCycles * lerp(a.cycles, b.cycles, u) / cycles;
    const finPhase = phase * Math.round(duration * 1.21) / wholeCycles;
    const breath = TAU * (Math.round(duration * 1.36) * progress +
      .10 * Math.sin(4 * TAU * progress));

    // The head leads along the route. Each following cross-section samples
    // farther back along the same curve before the lateral body wave is added.
    for (let i = 0; i < points.length; i++) {
      const along = distances[i] / length;
      const point = points[i];
      const curve = route.sample(distance - distances[i], point, tangent);
      curveTurns[i] = curve;
      if (i === 0) headTangent.copy(tangent);
      lateral.crossVectors(tangent, Y).normalize();
      const wave = phase - 4.85 * along;
      const weight = (.012 * along + .215 * along ** 2.5) *
        (.58 + .74 * effort) * (1 - .30 * mix(Math.abs(curve), .25, 1.5));
      point.addScaledVector(lateral, bodyScale * weight * Math.sin(wave));
      point.y += bodyScale * .011 * along ** 2 * Math.sin(wave - 1.1);
    }

    for (let i = 0; i < joints.length; i++) {
      const joint = joints[i];
      forward.subVectors(points[i], points[i + 1]).normalize();
      if (i === 0) forward.lerp(headTangent, .42).normalize();
      lateral.crossVectors(forward, Y).normalize();
      up.crossVectors(lateral, forward);
      const roll = -.091 * Math.tanh(1.12 * curveTurns[i])
        + .021 * Math.sin(2 * TAU * progress + .5 - i * .13);
      joint.pose.setFromRotationMatrix(matrix.makeBasis(forward, up, lateral))
        .multiply(axisRotation.setFromAxisAngle(X, roll));
    }

    scene.quaternion.copy(joints[0].pose);
    scene.position.copy(joints[0].position).multiply(scene.scale)
      .applyQuaternion(scene.quaternion).negate().add(points[0]);
    rootRotationInverse.copy(scene.quaternion).invert();

    // Keep the rig's original bone lengths and roll. Convert route poses
    // into each parent's space so bends are independent of export orientation.
    for (const [i, joint] of joints.entries()) {
      joint.rotationInModel.copy(rootRotationInverse).multiply(joint.pose)
        .multiply(joint.rotation);
      parentRotationInverse.copy(i ? joints[i - 1].rotationInModel : joint.parentBind);
      if (i) parentRotationInverse.multiply(joint.parentOffset);
      joint.bone.quaternion.copy(parentRotationInverse.invert()).multiply(joint.rotationInModel);
    }

    for (const fin of fins) {
      const { side, type, bone } = fin;
      let sweep;
      let feather;
      if (type === 'Gill') {
        const opening = (.5 + .5 * Math.sin(breath + .11 * side)) ** 1.7;
        sweep = side * .011 * opening;
        feather = -side * (.004 + .033 * opening);
      } else {
        const stroke = finPhase + (side > 0 ? .33 : 1.75) +
          .30 * Math.sin(3 * TAU * progress + side);
        const power = type === 'Pectoral' ? 1 : .56;
        const steering = Math.tanh(curveTurns[0]) * side;
        sweep = side * power * (.029 + .044 * steering +
          (.073 + .042 * effort) * Math.sin(stroke) + .020 * Math.sin(2 * stroke + .45));
        feather = side * power * (.013 + (.075 + .026 * effort) * Math.sin(stroke - .92));
      }
      bone.quaternion.copy(axisRotation.setFromAxisAngle(fin.sweep, sweep))
        .multiply(axisRotation.setFromAxisAngle(fin.feather, feather)).multiply(fin.bind);
    }

    motion.head.copy(points[0]);
    motion.center.copy(scene.position);
    motion.heading.copy(X).applyQuaternion(scene.quaternion);
    motion.turn = curveTurns[0];
    motion.speed = lerp(a.speed, b.speed, u);
    motion.phase = phase;
    return motion;
  }

  update(0);
  return { update, getMotion: () => motion, duration };
}
