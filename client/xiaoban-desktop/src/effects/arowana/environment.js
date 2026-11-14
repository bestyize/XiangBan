import * as THREE from 'three';

export function createTankEnvironment(renderer) {
  const width = 256;
  const height = 128;
  const data = new Uint16Array(width * height * 4);
  const emitters = [
    { vector: new THREE.Vector3(-0.24, 0.92, 0.31).normalize(), color: [4.0, 4.4, 4.9], vertical: 28, horizontal: 3.7 },
    { vector: new THREE.Vector3(0.30, 0.12, 0.94).normalize(), color: [1.26, 1.23, 1.16], vertical: 12, horizontal: 13 },
    { vector: new THREE.Vector3(0.73, 0.32, -0.52).normalize(), color: [1.30, 1.11, 0.86], vertical: 29, horizontal: 20 },
    { vector: new THREE.Vector3(-0.35, 0.65, -0.67).normalize(), color: [0.46, 0.64, 0.92], vertical: 31, horizontal: 20 },
    { vector: new THREE.Vector3(0.30, -0.96, 0.25).normalize(), color: [0.35, 0.43, 0.52], vertical: 10, horizontal: 4 },
  ].map(emitter => ({
    ...emitter,
    right: new THREE.Vector3().crossVectors(new THREE.Vector3(0, 1, 0), emitter.vector).normalize(),
  })).map(emitter => ({
    ...emitter,
    up: new THREE.Vector3().crossVectors(emitter.vector, emitter.right).normalize(),
  }));
  const direction = new THREE.Vector3();
  const radiance = [0, 0, 0];
  for (let y = 0; y < height; y += 1) {
    const latitude = ((y + 0.5) / height - 0.5) * Math.PI;
    for (let x = 0; x < width; x += 1) {
      const longitude = ((x + 0.5) / width - 0.5) * 2 * Math.PI;
      direction.set(Math.cos(longitude) * Math.cos(latitude), Math.sin(latitude), Math.sin(longitude) * Math.cos(latitude));
      const upper = Math.max(0, direction.y);
      const offset = 4 * (y * width + x);
      radiance[0] = 0.075 + 0.19 * upper;
      radiance[1] = 0.103 + 0.20 * upper;
      radiance[2] = 0.136 + 0.21 * upper;
      for (const emitter of emitters) {
        if (emitter.vector.dot(direction) < 0) continue;
        const cross = direction.dot(emitter.right);
        const rise = direction.dot(emitter.up);
        const weight = Math.exp(-cross * cross * emitter.horizontal - rise * rise * emitter.vertical);
        for (let channel = 0; channel < 3; channel += 1) {
          radiance[channel] += weight * emitter.color[channel];
        }
      }
      for (let channel = 0; channel < 3; channel += 1) {
        data[offset + channel] = THREE.DataUtils.toHalfFloat(radiance[channel]);
      }
      data[offset + 3] = 0x3c00;
    }
  }
  const equirect = new THREE.DataTexture(data, width, height, THREE.RGBAFormat, THREE.HalfFloatType);
  equirect.mapping = THREE.EquirectangularReflectionMapping;
  equirect.colorSpace = THREE.LinearSRGBColorSpace;
  equirect.minFilter = THREE.LinearFilter;
  equirect.magFilter = THREE.LinearFilter;
  equirect.needsUpdate = true;
  const generator = new THREE.PMREMGenerator(renderer);
  const environment = generator.fromEquirectangular(equirect).texture;
  generator.dispose();
  equirect.dispose();
  return environment;
}
