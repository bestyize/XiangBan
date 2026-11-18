import {
  Bone, Float32BufferAttribute, Matrix4, Quaternion, Skeleton, Uint16BufferAttribute, Vector3, Vector4,
} from 'three';

const AXIAL_SURFACE = /^(Body|Dorsal_fin|Anal_fin|Caudal_fin)/;
const FORWARD = new Vector3(1, 0, 0);

function bindSurface(mesh, rootInverse, head, dense, headX, spacing, gills) {
  const original = mesh.skeleton;
  const headBind = original.boneInverses[original.bones.indexOf(head)];
  const restSkin = new Matrix4().multiplyMatrices(head.matrixWorld, headBind);
  const toBody = new Matrix4().multiplyMatrices(rootInverse, restSkin).multiply(mesh.bindMatrix);
  const fins = gills.filter(bone => original.bones.includes(bone));
  const finIndices = fins.map(bone => original.bones.indexOf(bone));
  const inverses = dense.map(bone => bone.matrixWorld.clone().invert().multiply(restSkin));
  inverses.push(...finIndices.map(index => original.boneInverses[index]));

  const geometry = mesh.geometry.clone();
  const vertices = geometry.getAttribute('position');
  const oldJoints = geometry.getAttribute('skinIndex');
  const oldWeights = geometry.getAttribute('skinWeight');
  const indices = new Uint16Array(vertices.count * 4);
  const weights = new Float32Array(vertices.count * 4);
  const position = new Vector3();
  const last = dense.length - 1;
  const station = i => Math.min(last, Math.max(0, i));
  for (let i = 0; i < vertices.count; i++) {
    position.fromBufferAttribute(vertices, i).applyMatrix4(toBody);
    const s = Math.min(last, Math.max(0, (headX - position.x) / spacing));
    let gill = -1;
    let gillWeight = 0;
    for (let slot = 0; fins.length && slot < 4; slot++) {
      const index = finIndices.indexOf(oldJoints.getComponent(i, slot));
      const weight = oldWeights.getComponent(i, slot);
      if (index >= 0 && weight > gillWeight) {
        gill = index;
        gillWeight = weight;
      }
    }
    const offset = 4 * i;
    if (gill >= 0) {
      const middle = Math.floor(s + .5);
      const t = s - middle + .5;
      const bodyWeight = 1 - gillWeight;
      indices.set([station(middle - 1), middle, station(middle + 1), dense.length + gill], offset);
      weights.set([.5 * (1 - t) ** 2 * bodyWeight, (.5 + t - t * t) * bodyWeight,
        .5 * t * t * bodyWeight, gillWeight], offset);
    } else {
      const middle = Math.floor(s);
      const t = s - middle;
      const t2 = t * t, t3 = t2 * t;
      indices.set([station(middle - 1), middle, station(middle + 1), station(middle + 2)], offset);
      weights.set([(1 - t) ** 3 / 6, (3 * t3 - 6 * t2 + 4) / 6,
        (-3 * t3 + 3 * t2 + 3 * t + 1) / 6, t3 / 6], offset);
    }
  }
  geometry.setAttribute('skinIndex', new Uint16BufferAttribute(indices, 4));
  geometry.setAttribute('skinWeight', new Float32BufferAttribute(weights, 4));
  mesh.geometry = geometry;
  mesh.skeleton = new Skeleton([...dense, ...fins], inverses);
}

// Integrate the tangent of one length-preserving curve from the rigid skull.
// The densely spaced bones sample that curve for the existing GPU skinning.
export function createContinuousBody(object, spine, fins, neckX) {
  object.updateWorldMatrix(true, true);
  const rootInverse = object.matrixWorld.clone().invert();
  const matrix = new Matrix4();
  const scale = new Vector3();
  const mounts = [...spine, ...fins].map(bone => {
    const position = new Vector3();
    const rotation = new Quaternion();
    matrix.multiplyMatrices(rootInverse, bone.matrixWorld).decompose(position, rotation, scale);
    return { bone, position, rotation };
  });
  const headX = mounts[0].position.x;
  const frames = spine.map(() => new Quaternion());
  const knots = frames.map((frame, i) => ({ s: headX - mounts[i].position.x, frame }));
  knots.splice(1, 0, { s: headX - neckX, frame: frames[0] });
  for (const knot of knots) {
    knot.orientation = new Vector4();
    knot.slope = new Vector4(0, 0, 0, 0);
  }

  const length = knots.at(-1).s + .54;
  const intervals = Math.ceil(length / .062);
  const spacing = length / intervals;
  const dense = Array.from({ length: intervals + 1 }, (_, i) => {
    const bone = new Bone();
    bone.position.set(headX - spacing * i, 0, 0);
    object.add(bone);
    return bone;
  });
  object.updateWorldMatrix(true, true);
  const gills = fins.filter(bone => bone.name.startsWith('Gill'));
  object.traverse(mesh => {
    if (mesh.isSkinnedMesh && AXIAL_SURFACE.test(mesh.name)) {
      bindSurface(mesh, rootInverse, spine[0], dense, headX, spacing, gills);
    }
  });

  const pivot = new Vector3();
  const tangent = new Vector3();
  const step = new Vector3();
  const middleFrame = new Quaternion();
  const sampleFrame = new Quaternion();
  const interpolated = new Vector4();
  const rootRotation = new Quaternion();
  const inverseParent = new Quaternion();
  const world = new Vector3();
  const center = new Vector3();

  function orientationAt(s, out) {
    if (s <= knots[1].s) return out.copy(frames[0]);
    const last = knots.at(-1);
    if (s >= last.s) return out.copy(last.frame);
    let i = 1;
    while (knots[i + 1].s < s) i++;
    const a = knots[i], b = knots[i + 1];
    const h = b.s - a.s;
    const t = (s - a.s) / h;
    const u = 1 - t;
    interpolated.copy(a.orientation).multiplyScalar(u * u * (1 + 2 * t))
      .addScaledVector(b.orientation, t * t * (3 - 2 * t))
      .addScaledVector(a.slope, h * u * u * t)
      .addScaledVector(b.slope, -h * u * t * t);
    return out.set(interpolated.x, interpolated.y, interpolated.z, interpolated.w).normalize();
  }

  function advance(from, to, initial, final, out) {
    orientationAt((from + to) * .5, middleFrame);
    step.copy(FORWARD).applyQuaternion(middleFrame).multiplyScalar(4);
    step.add(tangent.copy(FORWARD).applyQuaternion(initial));
    step.add(tangent.copy(FORWARD).applyQuaternion(final));
    out.addScaledVector(step, (from - to) / 6);
  }

  function positionAt(s, frame, out) {
    const i = Math.min(intervals, Math.max(0, Math.floor(s / spacing)));
    const station = dense[i];
    out.copy(station.position);
    advance(i * spacing, s, station.quaternion, frame, out);
  }

  function pose() {
    for (const [i, knot] of knots.entries()) {
      const { x, y, z, w } = knot.frame;
      knot.orientation.set(x, y, z, w);
      if (i && knot.orientation.dot(knots[i - 1].orientation) < 0) knot.orientation.negate();
    }
    for (let i = 2; i < knots.length - 1; i++) {
      const a = knots[i - 1], b = knots[i + 1];
      knots[i].slope.copy(b.orientation).sub(a.orientation).divideScalar(b.s - a.s);
    }

    dense[0].quaternion.copy(frames[0]);
    for (let i = 1; i <= intervals; i++) {
      const prior = dense[i - 1], bone = dense[i];
      orientationAt(i * spacing, bone.quaternion);
      bone.position.copy(prior.position);
      advance((i - 1) * spacing, i * spacing, prior.quaternion, bone.quaternion, bone.position);
    }

    // Fin roots and the rigid facial parts follow the same surface frame.
    object.updateWorldMatrix(true, false);
    object.getWorldQuaternion(rootRotation);
    for (const { bone, position, rotation } of mounts) {
      const s = headX - position.x;
      orientationAt(s, sampleFrame);
      positionAt(s, sampleFrame, pivot);
      world.set(0, position.y, position.z).applyQuaternion(sampleFrame).add(pivot).applyMatrix4(object.matrixWorld);
      bone.position.copy(bone.parent.worldToLocal(world));
      bone.parent.getWorldQuaternion(inverseParent).invert();
      bone.quaternion.copy(inverseParent).multiply(rootRotation).multiply(sampleFrame).multiply(rotation);
    }

    orientationAt(headX, sampleFrame);
    positionAt(headX, sampleFrame, center);
  }

  return { frames, pose, center };
}

export function turnAcrossBody(sample, distance, reach, point, before, after) {
  sample(distance - reach, point, before);
  sample(distance + reach, point, after);
  return Math.atan2(before.z * after.x - before.x * after.z, before.x * after.x + before.z * after.z)
    / (2 * reach);
}
