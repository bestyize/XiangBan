import { TextureLoader } from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';

export function embeddedModelLoader() {
  const loader = new GLTFLoader();
  // WKWebView permits embedded blob textures as images under the desktop CSP.
  loader.register(parser => {
    parser.textureLoader = new TextureLoader(loader.manager);
    return { name: 'xiaoban-texture-loader' };
  });
  return loader;
}

export function disposeModels(roots) {
  const geometries = new Set(), skeletons = new Set(), materials = new Set(), textures = new Set();
  for (const root of roots) root.traverse(object => {
    if (object.geometry) geometries.add(object.geometry);
    if (object.skeleton) skeletons.add(object.skeleton);
    for (const material of [object.material].flat()) if (material) materials.add(material);
  });
  for (const material of materials) {
    for (const value of Object.values(material)) if (value?.isTexture) textures.add(value);
    material.dispose();
  }
  for (const geometry of geometries) geometry.dispose();
  for (const skeleton of skeletons) skeleton.dispose();
  for (const texture of textures) texture.dispose();
}
