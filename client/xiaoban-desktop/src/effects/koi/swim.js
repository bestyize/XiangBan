import { DynamicDrawUsage, MathUtils, Matrix4, Mesh, Quaternion, Vector3 } from 'three';

import { createSwimRoute } from '../arowana/swim-route.js';

// Wide end turns give the caudal fin room to follow the trunk.
const kohakuCruisePoints = [
  [0, .04, 3.75], [2.50, .09, 3.86], [4.50, .19, 3.70],
  [6.63, .27, 3.02], [8.15, .22, 1.35], [8.44, .08, -.91],
  [7.20, -.11, -2.94], [5.14, -.22, -3.83], [2.55, -.21, -3.79],
  [0, -.13, -3.43], [-2.55, -.015, -3.44], [-5.14, .14, -3.80],
  [-7.20, .24, -2.95], [-8.43, .23, -.91], [-8.25, .14, 1.36],
  [-6.83, .04, 3.14], [-4.71, -.04, 3.88], [-2.46, -.03, 3.67],
];

// Fit the preview's orbit to the fixed desktop camera.
const route = createSwimRoute(kohakuCruisePoints.map(([x, y, z]) => [x / 3, y, z / 3]));
const TAU = Math.PI * 2;
const UP = new Vector3(0, 1, 0);
const FORWARD = new Vector3(1, 0, 0);
const TURN_LIMIT = .82;

/** Absolute-time swimming. Every instance owns its deformed geometry. */
export function createKoiSwimmer(scene) {
  const duration = 38;
  scene.updateWorldMatrix(true, true);
  const inverse = scene.matrixWorld.clone().invert();
  const meshes = [];
  scene.traverse(mesh => { if (mesh.isMesh) meshes.push(mesh); });
  const parts = meshes.map(source => {
    const geometry = source.geometry.clone();
    geometry.applyMatrix4(new Matrix4().multiplyMatrices(inverse, source.matrixWorld));
    // The editable GLB keeps a rig; the real-time procedural controller operates
    // on bind-pose vertices instead, without applying skinning a second time.
    const mesh = new Mesh(geometry, source.material);
    mesh.name = source.name;
    source.removeFromParent();
    scene.add(mesh);
    mesh.frustumCulled = false;
    const pectoral = mesh.name.includes('Pectoral');
    const finRoot = pectoral ? .40 : mesh.name.includes('Pelvic') ? .24 : 0;
    let side = 0;
    if (finRoot) {
      geometry.computeBoundingBox();
      side = Math.sign(geometry.boundingBox.min.z + geometry.boundingBox.max.z);
    }
    return {
      position: geometry.attributes.position.setUsage(DynamicDrawUsage),
      normal: geometry.attributes.normal?.setUsage(DynamicDrawUsage),
      rest: geometry.attributes.position.array.slice(),
      restNormal: geometry.attributes.normal?.array.slice(),
      finRoot, side, pectoral,
    };
  });
  const tangent = new Vector3();
  const forward = new Vector3();
  const lead = new Vector3();
  const sectionTangent = new Vector3();
  const previousTangent = new Vector3();
  const routePoint = new Vector3();
  const bodyOffset = new Vector3();
  const lateral = new Vector3();
  const up = new Vector3();
  const matrix = new Matrix4();
  const roll = new Quaternion();
  const motion = { center: new Vector3(), head: new Vector3(), heading: new Vector3(), turn: 0, speed: 0, phase: 0 };
  const start = route.samples[route.startIndex]?.distance ?? 0;
  // An integral beat count closes both route and tail motion at the loop seam.
  const beats = Math.max(1, Math.round(duration * .86));
  const finBeats = Math.max(1, Math.round(duration * 1.12));
  const bouts = TAU * 4;
  const speedVariation = .26;
  const scale = Math.abs(scene.scale.x);

  // Integrate a centerline so the spine keeps its length as it bends.
  const sections = 160;
  const steeringSections = 8;
  const shoulder = 1.3;
  const spineLength = 4.7;
  const step = spineLength / sections;
  const centerIndex = shoulder / step;
  const centerFloor = Math.floor(centerIndex);
  const centerFraction = centerIndex - centerFloor;
  const centerX = new Float64Array(sections + 1);
  const centerZ = new Float64Array(sections + 1);
  const angles = new Float64Array(sections + 1);
  const steering = new Float64Array(steeringSections + 1);

  function update(seconds) {
    const progress = MathUtils.euclideanModulo(seconds / duration, 1);
    const bout = bouts * progress;
    const routeProgress = progress + speedVariation * Math.sin(bout) / bouts;
    // The push leads the change in speed. Tail strokes quiet down during the glide.
    const effort = MathUtils.smoothstep(.5 + .5 * Math.cos(bout + .9), .12, .95);
    const phase = TAU * beats * routeProgress;
    const finPhase = TAU * finBeats * progress;
    const distance = start + route.length * routeProgress;
    const turn = route.sample(distance, motion.center, tangent);
    route.sample(distance + shoulder * scale, routePoint, lead);
    forward.copy(tangent).lerp(lead, .42).normalize();

    // Follow the route at successive stations from the shoulder back to the
    // tail. The front enters an arc first and the rear releases it last.
    previousTangent.copy(lead);
    let bend = 0;
    for (let i = 1; i <= steeringSections; i++) {
      const station = shoulder - spineLength * i / steeringSections;
      route.sample(distance + station * scale, routePoint, sectionTangent);
      bend += Math.atan2(
        previousTangent.x * sectionTangent.z - previousTangent.z * sectionTangent.x,
        previousTangent.x * sectionTangent.x + previousTangent.z * sectionTangent.z,
      );
      steering[i] = TURN_LIMIT * Math.tanh(bend / TURN_LIMIT);
      previousTangent.copy(sectionTangent);
    }

    const steer = steering[steeringSections] / TURN_LIMIT;
    const amplitude = (.065 + .37 * effort) * (1 - .26 * Math.abs(steer));
    centerX[0] = shoulder;
    centerZ[0] = 0;
    angles[0] = 0;
    for (let i = 1; i <= sections; i++) {
      const t = i / sections;
      const station = t * steeringSections;
      const lo = Math.min(steeringSections - 1, Math.floor(station));
      const steerAngle = MathUtils.lerp(steering[lo], steering[lo + 1], station - lo);
      const tailLag = .42 * MathUtils.smoothstep(t, .72, 1);
      angles[i] = steerAngle + amplitude * t ** 1.85 * Math.sin(phase - 4.25 * t - tailLag);
      const mid = (angles[i] + angles[i - 1]) * .5;
      centerX[i] = centerX[i - 1] - step * Math.cos(mid);
      centerZ[i] = centerZ[i - 1] - step * Math.sin(mid);
    }
    for (const part of parts) {
      const { position, normal, rest, restNormal, finRoot, side, pectoral } = part;
      const insideTurn = Math.max(0, -side * steer);
      const finAngle = finRoot ? (pectoral ? 1 : .55) * (
        .035 + .17 * (1 - .52 * effort) * Math.sin(finPhase + side * .9) + .16 * insideTurn
      ) : 0;
      const finSin = Math.sin(finAngle), finCos = Math.cos(finAngle);
      for (let i = 0; i < rest.length; i += 3) {
        const x = rest[i], y = rest[i + 1], z = rest[i + 2];
        const s = MathUtils.clamp((shoulder - x) / step, 0, sections);
        const lo = Math.min(sections - 1, Math.floor(s));
        const f = s - lo;
        const theta = MathUtils.lerp(angles[lo], angles[lo + 1], f);
        const sn = Math.sin(theta), cs = Math.cos(theta);
        const cx = x >= shoulder ? x : MathUtils.lerp(centerX[lo], centerX[lo + 1], f);
        const cz = MathUtils.lerp(centerZ[lo], centerZ[lo + 1], f);
        // Start the fin stroke outside its attachment to the trunk.
        const finSpan = finRoot ? Math.max(0, side * z - finRoot) : 0;
        const localZ = z + side * finSpan * (finCos - 1);
        position.array[i] = cx - sn * localZ;
        position.array[i + 1] = y + finSpan * finSin;
        position.array[i + 2] = cz + cs * localZ;
        if (normal && restNormal) {
          const nx = restNormal[i], ny = restNormal[i + 1], nz = restNormal[i + 2];
          const fy = finSpan ? ny * finCos + side * nz * finSin : ny;
          const fz = finSpan ? nz * finCos - side * ny * finSin : nz;
          normal.array[i] = cs * nx - sn * fz;
          normal.array[i + 1] = fy;
          normal.array[i + 2] = sn * nx + cs * fz;
        }
      }
      position.needsUpdate = true;
      if (normal) normal.needsUpdate = true;
    }
    lateral.crossVectors(forward, UP).normalize();
    up.crossVectors(lateral, forward).normalize();
    scene.quaternion.setFromRotationMatrix(matrix.makeBasis(forward, up, lateral));
    scene.quaternion.multiply(roll.setFromAxisAngle(FORWARD, -.12 * steer + .012 * effort * Math.sin(phase)));
    bodyOffset.set(
      MathUtils.lerp(centerX[centerFloor], centerX[centerFloor + 1], centerFraction), 0,
      MathUtils.lerp(centerZ[centerFloor], centerZ[centerFloor + 1], centerFraction),
    ).multiply(scene.scale).applyQuaternion(scene.quaternion);
    scene.position.copy(motion.center).sub(bodyOffset);
    motion.head.set(2.3, 0, 0).multiply(scene.scale).applyQuaternion(scene.quaternion).add(scene.position);
    motion.heading.copy(forward);
    motion.turn = turn;
    motion.speed = route.length / duration * (1 + speedVariation * Math.cos(bout));
    motion.phase = phase;
    return motion;
  }
  update(0);
  return { object: scene, update };
}
