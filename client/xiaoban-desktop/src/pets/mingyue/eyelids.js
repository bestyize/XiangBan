// Eye artwork and UV masking from RealWorldBuilder's 潮汐书房 (037ef10, CC BY-NC-SA 4.0).
// See public/pets/mingyue/CREDITS.txt for the source and desktop adaptations.
import * as THREE from 'three';

// v1.0.2 lays both eyes upside down in this part of the 1536 × 768 face texture.
const crop = { x: 488, y: 0, width: 560, height: 335 };
const artScale = 560 / 490;
const phases = [0.22, 0.46, 0.7, 0.88, 1];
const expressions = new Set([
  'WhaleFace.scale',
  'Blink.scale',
  'ClosedEyes.scale',
  'Smile.scale',
  'SmileBlink.scale',
]);

function surface(w = crop.width, h = crop.height) {
  const canvas = document.createElement('canvas');
  canvas.width = w;
  canvas.height = h;
  return canvas.getContext('2d', { willReadFrequently: true });
}
function socket(ctx) {
  ctx.save();
  ctx.translate(157, 145);
  ctx.scale(1.17, 1.07);
  ctx.translate(-157, -145);
  ctx.beginPath();
  ctx.moveTo(91, 142);
  ctx.bezierCurveTo(96, 119, 112, 97, 126, 88);
  ctx.bezierCurveTo(132, 76, 138, 66, 153, 66);
  ctx.bezierCurveTo(177, 66, 195, 72, 202, 86);
  ctx.bezierCurveTo(207, 108, 212, 129, 221, 142);
  ctx.bezierCurveTo(216, 152, 216, 181, 210, 198);
  ctx.bezierCurveTo(199, 216, 179, 218, 158, 216);
  ctx.bezierCurveTo(133, 216, 117, 210, 106, 204);
  ctx.bezierCurveTo(102, 190, 107, 163, 91, 142);
  ctx.closePath();
  ctx.fill();
  ctx.restore();
}

function eyeMask(reference) {
  const w = crop.width,
    h = crop.height,
    ctx = surface();
  ctx.setTransform(artScale, 0, 0, artScale, 0, 0);
  ctx.filter = 'blur(1.6px)';
  socket(ctx);
  ctx.translate(503, 0);
  ctx.scale(-1, 1);
  socket(ctx);
  const pixels = reference.getImageData(0, 0, w, h).data;
  const mask = ctx.getImageData(0, 0, w, h);
  // Keep the original fringe wherever it crosses the outline of an eye.
  // Flood only connected hair colours so the blue irises remain inside the mask.
  const hair = new Uint8Array(w * h),
    queue = new Int32Array(w * h);
  let first = 0,
    last = 0;
  for (let i = 0; i < hair.length; i++) {
    const k = i * 4,
      r = pixels[k],
      g = pixels[k + 1],
      b = pixels[k + 2];
    if (r > 25 && b - r > 12 && b - g > 11 && g - r < 21) {
      hair[i] = 1;
      if (mask.data[k + 3] < 8) {
        hair[i] = 2;
        queue[last++] = i;
      }
    }
  }
  while (first < last) {
    const i = queue[first++];
    for (const j of [i - 1, i + 1, i - w, i + w]) {
      if (hair[j] === 1) {
        hair[j] = 2;
        queue[last++] = j;
      }
    }
  }
  for (let i = 0; i < hair.length; i++) if (hair[i] === 2) mask.data[i * 4 + 3] = 0;
  ctx.putImageData(mask, 0, 0);
  return ctx.canvas;
}

function skinPlate(reference, mask) {
  const { width: w, height: h } = crop;
  const original = reference.getImageData(0, 0, w, h).data;
  const alpha = mask.getContext('2d').getImageData(0, 0, w, h).data;
  const swatches = [];
  const isSkin = (r, g, b) => r > 192 && g > 145 && r > g + 6 && g > b + 2 && r - b < 91;
  // Sample the existing skin around each socket. Interpolate small opaque swatches so
  // the blush and the nose shading meet the lid without a seam or a pale eye-shaped rim.
  for (let y = 45; y < 263; y += 5)
    for (let x = 83; x < w - 65; x += 5) {
      const k = (y * w + x) * 4,
        r = original[k],
        g = original[k + 1],
        b = original[k + 2];
      if (alpha[k + 3] < 5 && isSkin(r, g, b)) swatches.push({ x, y, r, g, b });
    }
  const step = 4,
    low = surface(Math.ceil(w / step), Math.ceil(h / step));
  const grid = low.createImageData(low.canvas.width, low.canvas.height);
  for (let y = 0; y < grid.height; y++)
    for (let x = 0; x < grid.width; x++) {
      let r = 0,
        g = 0,
        b = 0,
        total = 0;
      for (const s of swatches) {
        const dx = x * step + 2 - s.x,
          dy = y * step + 2 - s.y,
          inv = 1 / (dx * dx + dy * dy + 12);
        const weight = inv * inv * inv;
        r += s.r * weight;
        g += s.g * weight;
        b += s.b * weight;
        total += weight;
      }
      const k = (y * grid.width + x) * 4;
      grid.data[k] = r / total;
      grid.data[k + 1] = g / total;
      grid.data[k + 2] = b / total;
      grid.data[k + 3] = 255;
    }
  low.putImageData(grid, 0, 0);
  const plate = surface();
  plate.drawImage(low.canvas, 0, 0, w, h);
  const corrected = plate.getImageData(0, 0, w, h),
    out = corrected.data;
  for (let y = 0; y < h; y++)
    for (let x = 0; x < w; x++) {
      const k = (y * w + x) * 4,
        r = original[k],
        g = original[k + 1],
        b = original[k + 2];
      if (!isSkin(r, g, b)) continue;
      const u = x / artScale,
        v = y / artScale,
        center = u < 251 ? 157 : 346;
      const edge = ((u - center) / 73) ** 2 + ((v - 144) / 76) ** 2;
      const blend = THREE.MathUtils.smoothstep(edge, 0.82, 1.15);
      out[k] += (r - out[k]) * blend;
      out[k + 1] += (g - out[k + 1]) * blend;
      out[k + 2] += (b - out[k + 2]) * blend;
    }
  plate.putImageData(corrected, 0, 0);
  return plate.canvas;
}
function lash(ctx, closed) {
  if (closed <= 0.45) return;
  ctx.save();
  ctx.globalAlpha = THREE.MathUtils.smoothstep(closed, 0.45, 0.97);
  ctx.fillStyle = '#513146';
  ctx.beginPath();
  ctx.moveTo(106, 151);
  ctx.bezierCurveTo(132, 164, 165, 175, 204, 152);
  ctx.bezierCurveTo(178, 177, 144, 178, 115, 162);
  ctx.quadraticCurveTo(108, 160, 105, 157);
  ctx.lineTo(114, 158);
  ctx.closePath();
  ctx.fill();
  ctx.beginPath();
  ctx.moveTo(120, 159);
  ctx.quadraticCurveTo(111, 158, 107, 151);
  ctx.quadraticCurveTo(111, 163, 127, 168);
  ctx.closePath();
  ctx.fill();
  ctx.strokeStyle = '#bd847b';
  ctx.lineWidth = 1.5;
  ctx.lineCap = 'round';
  ctx.globalAlpha *= 0.55;
  ctx.beginPath();
  ctx.moveTo(127, 151);
  ctx.quadraticCurveTo(152, 163, 179, 154);
  ctx.stroke();
  ctx.restore();
}
function eyelidAtlas(image) {
  const { x, y, width: w, height: h } = crop;
  const reference = surface();
  reference.translate(0, h);
  reference.scale(1, -1);
  reference.drawImage(image, x, y, w, h, 0, 0, w, h);
  const mask = eyeMask(reference),
    skin = skinPlate(reference, mask),
    eyes = surface();
  eyes.drawImage(reference.canvas, 0, 0);
  eyes.globalCompositeOperation = 'destination-in';
  eyes.drawImage(mask, 0, 0);
  const atlas = surface(w, h * phases.length);
  phases.forEach((amount, i) => {
    const tile = surface();
    tile.drawImage(skin, 0, 0);
    tile.scale(artScale, artScale);
    // The open eye folds into the lid. At full closure there is no iris underneath the lash.
    if (amount < 1) {
      tile.save();
      tile.translate(0, 159);
      tile.scale(1, 1 - amount);
      tile.translate(0, -159);
      tile.drawImage(eyes.canvas, 0, 0, 490, h / artScale);
      tile.restore();
    }
    lash(tile, amount);
    tile.save();
    tile.translate(503, 0);
    tile.scale(-1, 1);
    lash(tile, amount);
    tile.restore();
    atlas.drawImage(tile.canvas, 0, i * h);
  });
  const texture = new THREE.CanvasTexture(atlas.canvas);
  texture.flipY = false;
  texture.colorSpace = THREE.SRGBColorSpace;
  texture.anisotropy = 4;
  const socketTexture = new THREE.CanvasTexture(mask);
  socketTexture.flipY = false;
  socketTexture.anisotropy = 4;
  return { texture, socketTexture };
}

// Read the authored cues before AnimationMixer binds the clips. Hair, head bones,
// clothing and props keep their original tracks. Only the five duplicate faces lose theirs.
export function prepareFaceCues(clips) {
  const cues = new Map();
  for (const clip of clips) {
    const faceTracks = clip.tracks.filter(track => expressions.has(track.name));
    const enabled = name => faceTracks.some(track => track.name === name && track.values[0] > 0);
    const smiling = enabled('Smile.scale');
    const asleep = enabled('ClosedEyes.scale');
    const blink = faceTracks.find(track => track.name === (smiling ? 'SmileBlink.scale' : 'Blink.scale'));
    const events = [];
    if (blink) {
      let start;
      for (let i = 0; i < blink.times.length; i++) {
        if (blink.values[i * 3] > 0) {
          start ??= blink.times[i];
        } else if (start !== undefined) {
          events.push([start, blink.times[i]]);
          start = undefined;
        }
      }
      if (start !== undefined) events.push([start, clip.duration]);
    }
    cues.set(clip, { smiling, asleep, events });
    clip.tracks = clip.tracks.filter(track => !expressions.has(track.name));
  }
  return cues;
}

export function eyelidFrame({ asleep, events }, time) {
  if (asleep) return phases.length - 1;
  for (const [start, end] of events) {
    if (time < start || time >= end) continue;
    const t = (time - start) / (end - start);
    const closure = THREE.MathUtils.smoothstep(t, 0, 0.34) * (1 - THREE.MathUtils.smoothstep(t, 0.53, 1));
    return Math.round(closure * phases.length) - 1;
  }
  return -1;
}

export function installEyelids(model, clips) {
  const cues = prepareFaceCues(clips);
  const face = model.getObjectByName('WhaleFace');
  const material = face.material;
  const smile = model.getObjectByName('Smile').material;
  const { texture, socketTexture } = eyelidAtlas(material.map.image);
  const frame = { value: -1 };
  material.onBeforeCompile = (shader) => {
    shader.uniforms.eyeAtlas = { value: texture };
    shader.uniforms.eyeSocket = { value: socketTexture };
    shader.uniforms.eyeFrame = frame;
    shader.fragmentShader = shader.fragmentShader
      .replace(
        '#include <common>',
        `#include <common>
        uniform sampler2D eyeAtlas;
        uniform sampler2D eyeSocket;
        uniform float eyeFrame;`
      )
      .replace(
        '#include <map_fragment>',
        `
        vec4 faceColor = texture2D(map, vMapUv);
        vec2 localEyeUv = (vMapUv * vec2(1536.0, 768.0) - vec2(${crop.x}.0, ${crop.y}.0)) / vec2(${crop.width}.0, ${crop.height}.0);
        vec2 socketUv = vec2(localEyeUv.x, 1.0-localEyeUv.y);
        // Metal needs defined texture gradients on both sides of the eye bounds.
        vec2 socketDx = dFdx(socketUv), socketDy = dFdy(socketUv);
        bool eyeColumn = (socketUv.x > .145 && socketUv.x < .49) || (socketUv.x > .53 && socketUv.x < .89);
        if (eyeFrame >= 0.0 && eyeColumn && socketUv.y > .18 && socketUv.y < .80) {
          vec2 tile = vec2(1.0, 1.0 / ${phases.length}.0);
          vec2 atlasUv = (vec2(0.0,eyeFrame) + socketUv) * tile;
          vec4 eyelid = textureGrad(eyeAtlas, atlasUv, socketDx * tile, socketDy * tile);
          float coverage = textureGrad(eyeSocket, socketUv, socketDx, socketDy).a;
          faceColor.rgb = mix(faceColor.rgb,eyelid.rgb,coverage);
        }
        diffuseColor *= faceColor;`
      )
      .replace('#include <emissivemap_fragment>', 'totalEmissiveRadiance *= faceColor.rgb;');
  };
  material.customProgramCacheKey = () => 'whale-eyelids-v1';
  return {
    update(action) {
      const cue = cues.get(action.getClip());
      // The GLB's Smile and SmileBlink images are identical, with eyes already smiling shut.
      face.material = cue.smiling ? smile : material;
      frame.value = eyelidFrame(cue, action.time);
    },
    dispose() {
      face.material = material;
      texture.dispose();
      socketTexture.dispose();
    },
  };
}
