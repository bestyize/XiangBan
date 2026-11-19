import {
  Color, DirectionalLight, Group, MathUtils, NeutralToneMapping, PerspectiveCamera, Scene, Spherical,
  SRGBColorSpace, TextureLoader, WebGLRenderer,
} from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { clone } from 'three/addons/utils/SkeletonUtils.js';
import { cruiseRoute, vermilionCruiseDuration } from './cruise-route.js';
import { createTankEnvironment } from './environment.js';
import { createSwimmer } from './swim.js';
import { createGuppySchool } from './guppy/school.js';
import { soloGuppyRoute } from './guppy/solo-route.js';
import { createSwimmer as createGuppySwimmer } from './guppy/swim.js';
import { loadModelBytes } from './model-source.js';
import { createKoiSwimmer } from '../koi/swim.js';

const bytesBySpecies = new Map();
function modelBytes(species) {
  if (!bytesBySpecies.has(species)) {
    const bytes = loadModelBytes(species).catch(error => { bytesBySpecies.delete(species); throw error; });
    bytesBySpecies.set(species, bytes);
  }
  return bytesBySpecies.get(species);
}

function disposeModels(roots) {
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

export function createAquariumRenderer(canvas, id) {
  const loader = new GLTFLoader();
  // Embedded GLB textures use blob URLs; route them through the image policy.
  loader.register(parser => {
    parser.textureLoader = new TextureLoader(loader.manager);
    return { name: 'xiaoban-texture-loader' };
  });
  const roots = new Set();
  const loaded = new Map();
  const prepared = new Map();
  const scene = new Scene();
  const renderer = new WebGLRenderer({ canvas, alpha: true, antialias: true });
  const koi = id === 'koi';
  let destroyed = false;
  let request;
  let mode = null;
  let visible = [];
  let aquascape = false;

  function destroy() {
    if (destroyed) return;
    destroyed = true;
    disposeModels(roots);
    scene.environment?.dispose();
    renderer.dispose();
    renderer.forceContextLoss();
  }

  function rig(species) {
    if (!loaded.has(species)) {
      loaded.set(species, modelBytes(species).then(bytes => {
        if (destroyed) throw new DOMException('Renderer closed', 'AbortError');
        return loader.parseAsync(bytes, '');
      }).then(({ scene: object }) => {
        if (destroyed) {
          disposeModels([object]);
          throw new DOMException('Renderer closed', 'AbortError');
        }
        roots.add(object);
        return object;
      }).catch(error => { loaded.delete(species); throw error; }));
    }
    return loaded.get(species);
  }

  function prepare(kind) {
    if (!prepared.has(kind)) prepared.set(kind, rig(kind === 'school' ? 'guppy' : kind).then(model => {
      if (destroyed) throw new DOMException('Renderer closed', 'AbortError');
      if (kind === 'koi') {
        model.scale.setScalar(.66);
        const swimmer = createKoiSwimmer(model);
        model.visible = false;
        scene.add(model);
        return swimmer;
      }
      const createGuppy = (route, { scale = 1, ...motion } = {}) => {
        // Each fish owns its bones and deforming tail, while textures are shared.
        const object = clone(model);
        roots.add(object);
        object.scale.multiplyScalar(scale);
        return { object, ...createGuppySwimmer(object, route, motion) };
      };
      if (kind === 'arowana') {
        model.scale.setScalar(0.9);
        const { map, normalMap } = model.getObjectByName('Body').material;
        const anisotropy = Math.min(8, renderer.capabilities.getMaxAnisotropy());
        map.anisotropy = anisotropy;
        normalMap.anisotropy = anisotropy;
      }
      const swimmer = kind === 'arowana' ? { object: model, ...createSwimmer(model, cruiseRoute, { duration: vermilionCruiseDuration }) }
        : kind === 'guppy' ? createGuppy(soloGuppyRoute) : createGuppySchool(createGuppy, new Group());
      swimmer.object.traverse(object => {
        if (object.isSkinnedMesh) object.frustumCulled = false;
      });
      swimmer.object.visible = false;
      scene.add(swimmer.object);
      return swimmer;
    }).catch(error => { prepared.delete(kind); throw error; }));
    return prepared.get(kind);
  }

  try {
    renderer.outputColorSpace = SRGBColorSpace;
    renderer.toneMapping = NeutralToneMapping;
    renderer.toneMappingExposure = 0.99;
    renderer.setClearAlpha(0);
    scene.environment = createTankEnvironment(renderer);
    const mainLevel = koi ? 1.6 : 1.87, fillLevel = koi ? .55 : .64;
    const rimLevel = koi ? .85 : 1.18, environmentLevel = koi ? .72 : .85;
    const key = new DirectionalLight(koi ? 0xfff7ee : 0xf5f9ff, mainLevel);
    key.position.set(koi ? -2.5 : -2.2, koi ? 5.8 : 4.8, 3.6);
    const rim = new DirectionalLight(koi ? 0xf3ecdf : 0xd5e6ff, rimLevel);
    rim.position.set(2.1, 3.5, -4.3);
    const fill = new DirectionalLight(koi ? 0xf2f7ff : 0xe9f6ff, fillLevel);
    fill.position.set(koi ? 2.8 : 2, koi ? -1.35 : -0.85, koi ? 4.8 : 4.7);
    scene.add(key, rim, fill);
    const aim = new Spherical().setFromVector3(key.position);
    const originalColor = key.color.clone();
    const cool = new Color(0x9dc5ff), warm = new Color(0xffc18c);

    const viewHeight = 4.5;
    const camera = new PerspectiveCamera(25, 16 / 9, 0.12, 80);
    let width = 0, height = 0, ratio = 0, size = 1;
    function frameCamera() {
      const fitHeight = Math.max(viewHeight, viewHeight * (16 / 9) / camera.aspect);
      const baseDistance = mode === 'guppy' ? 12 : 11;
      const distance = baseDistance * (1 / size + 0.2) / 1.2;
      const y = mode === 'mixed' ? -0.31 : mode === 'arowana' ? 0.18 : 0.05;
      camera.fov = MathUtils.radToDeg(2 * Math.atan(fitHeight / (2 * baseDistance)));
      camera.position.set(0, y + (koi ? distance * .16 : 0), distance);
      camera.lookAt(0, y, 0);
      camera.updateProjectionMatrix();
    }

    function layoutSchool() {
      const mixed = mode === 'mixed';
      if (!mixed && mode !== 'school') return;
      const { object } = visible.at(-1);
      const clearance = aquascape ? (mixed ? 0.28 : 0.36) : 0;
      object.scale.setScalar(mixed ? 0.49 : 0.69);
      object.position.set(0, (mixed ? -1.22 : -0.18) + clearance, mixed ? -0.34 : -0.35);
    }

    return {
      get mode() { return mode; },
      get fishCount() { return visible.reduce((count, fish) => count + (fish.count ?? 1), 0); },
      async setMode(next) {
        const token = request = {};
        if (mode === next) return;
        const mixed = next === 'mixed';
        const residents = await Promise.all(mixed ? [prepare('arowana'), prepare('school')] : [prepare(next)]);
        if (destroyed || request !== token) return;
        for (const fish of visible) fish.object.visible = false;
        for (const fish of residents) fish.object.visible = true;
        visible = residents;
        mode = next;
        layoutSchool();
        frameCamera();
      },
      setAquascape(enabled) {
        if (enabled === aquascape) return;
        aquascape = enabled;
        layoutSchool();
      },
      setLighting(light) {
        key.intensity = mainLevel * light.main / 100;
        fill.intensity = fillLevel * light.fill / 100;
        rim.intensity = rimLevel * light.rim / 100;
        scene.environmentIntensity = environmentLevel * light.environment / 100;
        key.position.setFromSphericalCoords(aim.radius,
          aim.phi - MathUtils.degToRad(light.elevation - 49),
          aim.theta + MathUtils.degToRad(light.azimuth + 31));
        key.color.copy(originalColor).lerp(light.warmth < 0 ? cool : warm, Math.abs(light.warmth) / 100);
      },
      resize(w, h, dpr, nextSize) {
        const nextRatio = Math.min(dpr, 1.5, 1920 / w, 1080 / h);
        if (nextRatio !== ratio) renderer.setPixelRatio(nextRatio);
        if (w !== width || h !== height) renderer.setSize(w, h, false);
        if (w !== width || h !== height || size !== nextSize) {
          camera.aspect = w / h;
          size = nextSize;
          frameCamera();
        }
        width = w; height = h; ratio = nextRatio;
      },
      render(time) {
        for (const fish of visible) fish.update(time);
        renderer.render(scene, camera);
      },
      destroy,
    };
  } catch (error) {
    destroy();
    throw error;
  }
}
