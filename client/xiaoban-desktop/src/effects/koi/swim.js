import { DynamicDrawUsage, MathUtils, Matrix4, Mesh, Quaternion, Vector3 } from 'three';
import { createSwimRoute } from '../arowana/swim-route.js';

// The RealWorldBuilder aquarium route has two wide passes to show both flanks.
const route = createSwimRoute([
  [-1.50, -.16, .44], [-.70, -.15, .86], [.20, -.12, 1.00], [1.28, -.03, .97],
  [2.13, .14, .57], [2.34, .23, -.16], [2.08, .26, -.95], [1.26, .22, -1.13],
  [.45, .09, -.70], [-.38, -.02, -.24], [-1.27, -.11, .16], [-2.23, -.07, .28],
  [-2.80, .05, -.29], [-2.65, .16, -1.11], [-1.86, .25, -1.51], [-1.06, .22, -1.39],
  [-.80, .11, -.84], [-1.15, -.04, -.17],
], 1.1);
const tau = Math.PI * 2;
const vertical = new Vector3(0, 1, 0);
const forward = new Vector3(1, 0, 0);
const sections = 160;
const shoulder = 1.3;
const step = 4.7 / sections;
const duration = 38;
const beats = Math.round(duration * .83);

export function createKoiSwimmer(model) {
  model.updateWorldMatrix(true, true);
  const inverse = model.matrixWorld.clone().invert();
  const transform = new Matrix4();
  const meshes = [];
  model.traverse(mesh => { if (mesh.isMesh) meshes.push(mesh); });
  const parts = meshes.map(source => {
    // The GLB has a rig for editing. The preview deforms the bind-pose geometry in model space.
    const geometry = source.geometry.clone().applyMatrix4(transform.multiplyMatrices(inverse, source.matrixWorld));
    const mesh = new Mesh(geometry, source.material);
    mesh.name = source.name;
    mesh.frustumCulled = false;
    source.removeFromParent();
    model.add(mesh);
    const position = geometry.attributes.position.setUsage(DynamicDrawUsage);
    const normal = geometry.attributes.normal?.setUsage(DynamicDrawUsage);
    return { position, normal, rest: position.array.slice(), restNormal: normal?.array.slice(),
      paired: /Pectoral|Pelvic/.test(mesh.name), side: /_L(?:_|$)/.test(mesh.name) ? 1 : -1 };
  });

  const tangent = new Vector3(), lateral = new Vector3(), up = new Vector3();
  const basis = new Matrix4(), roll = new Quaternion();
  const start = route.samples[route.startIndex].distance;
  const centerX = new Float64Array(sections + 1), centerZ = new Float64Array(sections + 1);
  const angles = new Float64Array(sections + 1);
  centerX[0] = shoulder;

  function update(time) {
    const progress = MathUtils.euclideanModulo(time / duration, 1);
    const bout = tau * 3 * progress;
    const phase = tau * beats * progress + .35 * Math.sin(bout);
    const turn = route.sample(start + route.length * (progress + .01 * Math.sin(bout)), model.position, tangent);
    const bend = -.28 * Math.tanh(turn * .8);
    const amplitude = .18 + .16 * (.5 + .5 * Math.cos(bout)) ** 2;
    // Integrate a curved spine, preserving length as the tail sweeps from side to side.
    for (let i = 1; i <= sections; i++) {
      const t = i / sections;
      angles[i] = amplitude * t ** 2.2 * Math.sin(phase - 4.1 * t) + bend * t ** 1.8;
      const mid = (angles[i] + angles[i - 1]) * .5;
      centerX[i] = centerX[i - 1] - step * Math.cos(mid);
      centerZ[i] = centerZ[i - 1] - step * Math.sin(mid);
    }

    for (const { position, normal, rest, restNormal, paired, side } of parts) {
      const finAngle = paired ? side * (.035 + .11 * Math.sin(phase + side * .8)) : 0;
      const finSin = Math.sin(finAngle), finCos = Math.cos(finAngle);
      for (let i = 0; i < rest.length; i += 3) {
        const x = rest[i], y = rest[i + 1], z = rest[i + 2];
        const s = MathUtils.clamp((shoulder - x) / step, 0, sections);
        const lo = Math.min(sections - 1, Math.floor(s));
        const f = s - lo;
        const angle = MathUtils.lerp(angles[lo], angles[lo + 1], f);
        const sin = Math.sin(angle), cos = Math.cos(angle);
        const cx = x >= shoulder ? x : MathUtils.lerp(centerX[lo], centerX[lo + 1], f);
        const cz = MathUtils.lerp(centerZ[lo], centerZ[lo + 1], f);
        const span = Math.max(0, Math.abs(z) - .24);
        const localZ = z - Math.sign(z) * span * (1 - finCos);
        position.array[i] = cx - sin * localZ;
        position.array[i + 1] = y + span * finSin;
        position.array[i + 2] = cz + cos * localZ;
        if (normal) {
          const nx = restNormal[i], ny = restNormal[i + 1], nz = restNormal[i + 2];
          const fy = ny * finCos + side * nz * finSin;
          const fz = nz * finCos - side * ny * finSin;
          normal.array[i] = cos * nx - sin * fz;
          normal.array[i + 1] = fy;
          normal.array[i + 2] = sin * nx + cos * fz;
        }
      }
      position.needsUpdate = true;
      if (normal) normal.needsUpdate = true;
    }

    lateral.crossVectors(tangent, vertical).normalize();
    up.crossVectors(lateral, tangent).normalize();
    model.quaternion.setFromRotationMatrix(basis.makeBasis(tangent, up, lateral));
    model.quaternion.multiply(roll.setFromAxisAngle(forward, -.065 * Math.tanh(turn)));
  }

  return { object: model, update };
}
