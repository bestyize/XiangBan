import { AnimationMixer, DirectionalLight, HemisphereLight, OrthographicCamera, Scene, WebGLRenderer } from 'three';
import { disposeModels, embeddedModelLoader } from '../../three/embedded-model.js';
import { clipForMood } from './actions.ts';
import { installEyelids } from './eyelids.js';

export async function loadMingyue(canvas, signal) {
  const response = await fetch('/pets/mingyue/dafeiyu-maid.glb', { signal });
  if (!response.ok) throw new Error('溟月模型读取失败');
  const { scene: model, animations } = await embeddedModelLoader().parseAsync(await response.arrayBuffer(), '');
  let renderer, eyelids;
  try {
    signal.throwIfAborted();
    // Native hit testing samples the last presented frame, including after a pause.
    renderer = new WebGLRenderer({ canvas, alpha: true, antialias: true, preserveDrawingBuffer: true });
    eyelids = installEyelids(model, animations);
  } catch (error) {
    renderer?.dispose();
    renderer?.forceContextLoss();
    disposeModels([model]);
    throw error;
  }
  renderer.setClearColor(0, 0);
  const scene = new Scene();
  const camera = new OrthographicCamera(-1, 1, 1, -1, .01, 30);
  const key = new DirectionalLight(0xfff2e4, 1.35), rim = new DirectionalLight(0xddedff, 1.4);
  key.position.set(-3, 5, 5);
  rim.position.set(3, 3, -2);
  scene.add(model, key, rim, new HemisphereLight(0xfff8f3, 0xc1cce6, 1.85));
  const mixer = new AnimationMixer(model);
  const clips = new Map(animations.map(clip => [clip.name, clip]));
  // Costumes and held props still use the author's discrete visibility keys.
  const parts = [];
  model.traverse(part => parts.push(part));
  const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)');
  const pixel = new Uint8Array(4);
  const gl = renderer.getContext();
  let active, dragging = false, touchLeft = 0, lastPreference, width = 0, height = 0, dpr = 0;

  function frameCamera(name) {
    const asleep = name === 'sleep';
    const x = asleep ? 0 : .13, y = asleep ? .72 : 1, z = asleep ? -.8 : 0;
    const angle = asleep ? 1.85 : .23, halfHeight = asleep ? 1.3 : 1.23;
    const aspect = width / height;
    camera.top = halfHeight / Math.min(aspect, 1);
    camera.bottom = -camera.top;
    camera.right = camera.top * aspect;
    camera.left = -camera.right;
    camera.position.set(x + 7 * Math.sin(angle), y + (asleep ? 4.8 : .75), z + 7 * Math.cos(angle));
    camera.lookAt(x, y, z);
    camera.updateProjectionMatrix();
  }

  function render(seconds, expression, preference = 'auto') {
    const w = canvas.clientWidth, h = canvas.clientHeight, ratio = Math.min(devicePixelRatio || 1, 1.5);
    if (!w || !h || gl.isContextLost()) return;
    const resized = w !== width || h !== height || ratio !== dpr;
    if (resized) {
      width = w; height = h; dpr = ratio;
      renderer.setDrawingBufferSize(w, h, ratio);
    }
    if (preference !== lastPreference) touchLeft = 0;
    lastPreference = preference;
    const name = dragging ? 'walk' : touchLeft > 0 ? 'contact' : preference === 'auto' ? clipForMood(expression) : preference;
    const clip = clips.get(name) ?? clips.get('idle');
    const changed = active?.getClip() !== clip;
    if (changed) {
      active?.stop();
      active = mixer.clipAction(clip).reset().play();
      canvas.dataset.clip = clip.name;
    }
    if (resized || changed) frameCamera(clip.name);
    const dt = Math.min(seconds, .25);
    touchLeft = Math.max(0, touchLeft - dt);
    if (reducedMotion.matches && !resized && !changed && seconds > 0) return;
    mixer.update(reducedMotion.matches ? 0 : dt);
    eyelids.update(active);
    for (const part of parts) part.visible = part.scale.lengthSq() > .000001;
    renderer.render(scene, camera);
  }

  function hit(x, y) {
    const rect = canvas.getBoundingClientRect();
    if (x < rect.left || x >= rect.right || y < rect.top || y >= rect.bottom || gl.isContextLost()) return false;
    const column = Math.floor((x - rect.left) / rect.width * gl.drawingBufferWidth);
    const row = gl.drawingBufferHeight - 1 - Math.floor((y - rect.top) / rect.height * gl.drawingBufferHeight);
    gl.readPixels(column, row, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixel);
    return pixel[3] >= 16;
  }
  return {
    render, hit,
    poke() { touchLeft = 2.5; },
    setDragging(value) { dragging = value; if (value) touchLeft = 0; },
    destroy() {
      mixer.stopAllAction();
      mixer.uncacheRoot(model);
      eyelids.dispose();
      disposeModels([model]);
      renderer.dispose();
      renderer.forceContextLoss();
    },
  };
}
