import { BufferAttribute, BufferGeometry, DynamicDrawUsage, MathUtils, Matrix4, Vector3 } from 'three';

const { clamp, lerp, smoothstep, euclideanModulo: wrap } = MathUtils;

function periodicSpring(values, dt, omega, damping) {
  const result = new Float32Array(values.length);
  let displacement = values[0];
  let velocity = 0;
  // Settle on the periodic trajectory before saving samples for seeking.
  const warmup = Math.max(values.length, Math.ceil(1.2 / (values.length * dt)) * values.length);
  for (let i = -warmup; i < values.length; i++) {
    const index = wrap(i, values.length);
    const acceleration = omega * omega * (values[index] - displacement) - 2 * damping * omega * velocity;
    velocity += acceleration * dt;
    displacement += velocity * dt;
    if (i >= 0) result[i] = displacement;
  }
  return result;
}

export function createCaudalFin(mesh, tailBone, duration, poseBody) {
  const source = mesh.geometry;
  const grid = source.getAttribute('_fin_ray');
  if (!grid) throw new Error('Caudal fin is missing its ray grid');
  const base = source.getAttribute('position');
  const baseNormal = source.getAttribute('normal');
  const baseTangent = source.getAttribute('tangent');
  const deformation = new BufferAttribute(base.array.slice(), 3).setUsage(DynamicDrawUsage);
  const normal = baseNormal.clone().setUsage(DynamicDrawUsage);
  const tangent = baseTangent.clone().setUsage(DynamicDrawUsage);
  const geometry = new BufferGeometry();
  geometry.setIndex(source.index);
  for (const [key, attribute] of Object.entries(source.attributes)) geometry.setAttribute(key, attribute);
  geometry.setAttribute('position', deformation);
  geometry.setAttribute('normal', normal);
  geometry.setAttribute('tangent', tangent);
  mesh.geometry = geometry;

  let rows = 0;
  let columns = 0;
  let rootIndex = 0;
  for (let i = 0; i < grid.count; i++) {
    const row = grid.getX(i);
    rows = Math.max(rows, row);
    columns = Math.max(columns, grid.getY(i));
    if (row === 0) rootIndex = i;
  }
  columns++;
  const root = new Vector3().fromBufferAttribute(base, rootIndex);
  const gridIndex = (row, column) => row ? 1 + (row - 1) * columns + column : 0;
  const mapping = Int32Array.from({ length: grid.count }, (_, i) => gridIndex(grid.getX(i), grid.getY(i)));
  const rayPoints = Array.from({ length: rows * columns + 1 }, () => new Vector3());
  const restPoints = rayPoints.map(point => point.clone());
  for (let i = 0; i < mapping.length; i++) restPoints[mapping[i]].fromBufferAttribute(base, i);
  const free = new Float32Array(restPoints.length);
  const bendEnvelope = new Float32Array(restPoints.length);
  const restSegments = restPoints.slice(1).map((point, index) => {
    const i = index + 1;
    const row = Math.floor((i - 1) / columns) + 1;
    const prior = restPoints[row === 1 ? 0 : i - columns];
    const vector = point.clone().sub(prior);
    const x = (point.x + prior.x) * .5;
    free[i] = smoothstep(root.x - x, .045, .36);
    const radius = (row - .5) / rows;
    bendEnvelope[i] = free[i] * (1 - (1 - radius) ** 2);
    return {
      vector,
      droop: free[i] * radius ** 1.10,
      blend: smoothstep(radius, .20, .97),
    };
  });

  const samples = Math.max(2, Math.ceil(duration * 120));
  const dt = duration / samples;
  const boneIndex = mesh.skeleton.bones.indexOf(tailBone);
  const inverseBind = mesh.skeleton.boneInverses[boneIndex];
  const boneTransform = new Matrix4();
  const states = Array.from({ length: samples }, (_, i) => {
    const effort = poseBody(i * dt);
    boneTransform.copy(tailBone.matrixWorld).multiply(inverseBind).multiply(mesh.bindMatrix);
    return {
      effort,
      root: mesh.applyBoneTransform(rootIndex, root.clone()).applyMatrix4(mesh.matrixWorld),
      normal: new Vector3(0, 0, 1).transformDirection(boneTransform),
      chord: new Vector3(-1, 0, 0).transformDirection(boneTransform),
    };
  });
  const scale = mesh.matrixWorld.getMaxScaleOnAxis();
  const mid = new Float32Array(samples);
  const tip = new Float32Array(samples);
  const droop = new Float32Array(samples);
  const start = new Vector3(), end = new Vector3(), velocity = new Vector3();
  function velocityAt(before, after, station) {
    start.copy(before.chord).multiplyScalar(station).add(before.root);
    end.copy(after.chord).multiplyScalar(station).add(after.root);
    return velocity.subVectors(end, start).multiplyScalar(1 / (2 * dt * scale));
  }
  for (let i = 0; i < samples; i++) {
    const before = states[(i + samples - 1) % samples];
    const state = states[i];
    const after = states[(i + 1) % samples];
    // Drag opposes lateral motion through still water. The bounded targets
    // are equilibrium angles for a damped elastic response.
    mid[i] = -.26 * Math.tanh(velocityAt(before, after, .29).dot(state.normal) / 1.30);
    const tipFlow = velocityAt(before, after, .97);
    tip[i] = -.57 * Math.tanh(tipFlow.dot(state.normal) / 1.40);
    const tension = smoothstep(state.effort, .12, .88);
    droop[i] = .036 + .48 * (1 - tension) + .075 * Math.tanh(tipFlow.y / .63);
  }

  const middleBend = periodicSpring(mid, dt, 25, .85);
  const trailingBend = periodicSpring(tip, dt, 15, .81);
  const hanging = periodicSpring(droop, dt, 13, .88);

  const segment = new Vector3();
  const radial = new Vector3();
  const across = new Vector3();
  const refNormal = new Vector3();
  const vertexNormal = new Vector3();
  const vertexTangent = new Vector3();
  const rotationAxis = new Vector3();
  rayPoints[0].copy(root);

  return function updateCaudalFin(time) {
    const frame = wrap(time, duration) / dt;
    const index = Math.floor(frame);
    const here = index % samples;
    const next = (here + 1) % samples;
    const mix = frame - index;
    const midBend = lerp(middleBend[here], middleBend[next], mix);
    const tipBend = lerp(trailingBend[here], trailingBend[next], mix);
    const sag = clamp(lerp(hanging[here], hanging[next], mix), 0, .59);
    for (let column = 0; column < columns; column++) {
      const side = 2 * column / (columns - 1) - 1;
      const flexibility = 1 - .13 * side;
      for (let row = 1; row <= rows; row++) {
        const i = gridIndex(row, column);
        const rest = restSegments[i - 1];
        // Droop follows the chord. Adjacent rays lean downward together.
        const pitch = sag * rest.droop;
        const pitchCos = Math.cos(pitch);
        const pitchSin = Math.sin(pitch);
        const { x, y, z } = rest.vector;
        const chord = x * pitchCos - y * pitchSin;
        const bend = bendEnvelope[i] * flexibility * lerp(midBend, tipBend, rest.blend);
        const cos = Math.cos(bend);
        const sin = Math.sin(bend);
        segment.set(chord * cos + z * sin, x * pitchSin + y * pitchCos, -chord * sin + z * cos);
        rayPoints[i].copy(rayPoints[gridIndex(row - 1, column)]).add(segment);
      }
    }
    for (let i = 0; i < mapping.length; i++) {
      const point = rayPoints[mapping[i]];
      deformation.setXYZ(i, point.x, point.y, point.z);
      const row = grid.getX(i);
      const col = grid.getY(i);
      refNormal.fromBufferAttribute(baseNormal, i);
      if (row) {
        radial.subVectors(rayPoints[gridIndex(Math.min(rows, row + 1), col)], rayPoints[gridIndex(row - 1, col)]);
        across.subVectors(rayPoints[gridIndex(row, Math.min(columns - 1, col + 1))], rayPoints[gridIndex(row, Math.max(0, col - 1))]);
        vertexNormal.crossVectors(radial, across).normalize();
        if (vertexNormal.dot(refNormal) < 0) vertexNormal.negate();
      } else {
        vertexNormal.copy(refNormal);
      }
      normal.setXYZ(i, vertexNormal.x, vertexNormal.y, vertexNormal.z);
      vertexTangent.fromBufferAttribute(baseTangent, i);
      // Rotate the original tangent frame with the bent membrane.
      rotationAxis.crossVectors(refNormal, vertexNormal);
      const sin = rotationAxis.length();
      if (sin > 1e-6) vertexTangent.applyAxisAngle(rotationAxis.multiplyScalar(1 / sin), Math.atan2(sin, refNormal.dot(vertexNormal)));
      tangent.setXYZW(i, vertexTangent.x, vertexTangent.y, vertexTangent.z, baseTangent.getW(i));
    }
    deformation.needsUpdate = true;
    normal.needsUpdate = true;
    tangent.needsUpdate = true;
  };
}
