import { Float32BufferAttribute, MathUtils, Matrix3, Matrix4, Vector3 } from 'three';

const UP = new Vector3(0, 1, 0);
const { smoothstep } = MathUtils;

function toModelSpace(mesh, rootInverse) {
  const { bones, boneInverses } = mesh.skeleton;
  return new Matrix4().multiplyMatrices(rootInverse, bones[0].matrixWorld)
    .multiply(boneInverses[0]).multiply(mesh.bindMatrix);
}

function addMorph(geometry, name, toModel, deform) {
  const { position, normal } = geometry.attributes;
  const fromModel = toModel.clone().invert();
  const normalToModel = new Matrix3().getNormalMatrix(toModel);
  const normalFromModel = new Matrix3().getNormalMatrix(fromModel);
  const point = new Vector3(), direction = new Vector3(), rest = new Vector3();
  const positions = new Float32BufferAttribute(new Float32Array(position.count * 3), 3);
  const normals = new Float32BufferAttribute(new Float32Array(normal.count * 3), 3);
  positions.name = normals.name = name;
  for (let i = 0; i < position.count; i++) {
    point.fromBufferAttribute(position, i).applyMatrix4(toModel);
    direction.fromBufferAttribute(normal, i).applyMatrix3(normalToModel);
    deform(point, direction, i);
    point.applyMatrix4(fromModel).sub(rest.fromBufferAttribute(position, i));
    direction.applyMatrix3(normalFromModel).sub(rest.fromBufferAttribute(normal, i));
    positions.setXYZ(i, point.x, point.y, point.z);
    normals.setXYZ(i, direction.x, direction.y, direction.z);
  }
  geometry.morphTargetsRelative = true;
  (geometry.morphAttributes.position ??= []).push(positions);
  (geometry.morphAttributes.normal ??= []).push(normals);
}

export function bindGillBreathing(object) {
  const mesh = object.getObjectByName('Body');
  const toModel = toModelSpace(mesh, object.matrixWorld.clone().invert());
  const hinge = new Vector3(1.19, 0, 0);
  addMorph(mesh.geometry, 'GillFlare', toModel, (point, normal) => {
    const rear = smoothstep(point.x, .75, .88);
    const front = 1 - smoothstep(point.x, 1.09, 1.19);
    const height = 1 - smoothstep(Math.abs(point.y + .025), .13, .31);
    const side = smoothstep(Math.abs(point.z), .045, .115) * Math.sign(point.z);
    const angle = .35 * rear * front * height * side;
    point.sub(hinge).applyAxisAngle(UP, angle).add(hinge);
    normal.applyAxisAngle(UP, angle);
  });
  mesh.updateMorphTargets();
  return amount => { mesh.morphTargetInfluences[mesh.morphTargetDictionary.GillFlare] = amount; };
}
