// 风起花野：蓝天、远山、海岸与盛开的草坡。云、草、花粉和蝴蝶都在微风里缓慢运动。
const TAU = Math.PI * 2;
const clamp = (v, min = 0, max = 1) => Math.min(max, Math.max(min, v));
const mix = (a, b, t) => a + (b - a) * t;
const smooth = (t) => t * t * (3 - 2 * t);
const hash = (n) => {
  const value = Math.sin(n * 127.1 + 311.7) * 43758.5453;
  return value - Math.floor(value);
};
const layer = (w, h) => {
  const canvas = document.createElement("canvas");
  canvas.width = Math.max(1, Math.round(w));
  canvas.height = Math.max(1, Math.round(h));
  return canvas;
};
const color = (hex) => [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
const rgb = (a) => a.map((v) => Math.round(v)).join(",");

const FLOWERS = [
  ["255,255,244", "#f8cf4a"],
  ["255,245,252", "#e99ac2"],
  ["227,243,255", "#81b8ec"],
  ["255,252,221", "#f5b63b"],
  ["238,235,255", "#9c91db"],
];

function grainTile(size, amount) {
  const canvas = layer(size, size);
  const context = canvas.getContext("2d");
  const image = context.createImageData(size, size);
  for (let i = 0; i < image.data.length; i += 4) {
    const value = Math.random() * 2 - 1;
    image.data[i] = image.data[i + 1] = image.data[i + 2] = value > 0 ? 255 : 0;
    image.data[i + 3] = Math.abs(value) * amount;
  }
  context.putImageData(image, 0, 0);
  return canvas;
}

function path(g, points, close = true) {
  g.beginPath();
  g.moveTo(points[0][0], points[0][1]);
  for (let i = 1; i < points.length; i++) g.lineTo(points[i][0], points[i][1]);
  if (close) g.closePath();
}

function drawCloud(g, cloud, w, h, alpha) {
  const x = cloud.x * w;
  const y = cloud.y * h;
  const s = cloud.size * Math.min(w, h);
  g.save();
  g.globalAlpha = alpha;
  g.translate(x, y);
  g.scale(1.16, 0.72);
  for (let i = 0; i < 5; i++) {
    const px = (hash(cloud.seed * 7 + i * 3.1) - 0.5) * s * 1.55;
    const py = (hash(cloud.seed * 11 + i * 4.7) - 0.58) * s * 0.46;
    const radius = s * (0.42 + hash(cloud.seed * 17 + i * 2.3) * 0.34);
    const puff = g.createRadialGradient(px, py - radius * 0.12, 0, px, py, radius * 1.4);
    puff.addColorStop(0, "rgba(255,255,255,.94)");
    puff.addColorStop(0.56, "rgba(249,253,255,.74)");
    puff.addColorStop(1, "rgba(225,240,255,0)");
    g.fillStyle = puff;
    g.beginPath();
    g.arc(px, py, radius, 0, TAU);
    g.fill();
  }
  const under = g.createLinearGradient(0, s * 0.18, 0, s * 0.65);
  under.addColorStop(0, "rgba(218,237,252,.24)");
  under.addColorStop(1, "rgba(218,237,252,0)");
  g.fillStyle = under;
  g.beginPath();
  g.ellipse(0, s * 0.24, s * 1.38, s * 0.26, 0, 0, TAU);
  g.fill();
  g.restore();
}

export default async function create({ canvas, settings = {}, assets = {} }) {
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("无法绘制风起花野");
  ctx.imageSmoothingEnabled = true;
  ctx.imageSmoothingQuality = "high";
  let reference = null;
  if (assets.base?.url) {
    const image = new Image();
    image.src = assets.base.url;
    try {
      await image.decode();
      reference = image;
    } catch {
      reference = null;
    }
  }
  let values = settings;
  let background = null;
  let grain = null;
  let signature = "";
  let clouds = [];
  let grass = [];
  let flowers = [];
  let pollen = [];
  let butterflies = [];
  let waves = [];

  const speed = () => clamp(Number(values.speed ?? 0.65), 0.1, 1.5);
  const breeze = () => clamp(Number(values.breeze ?? 0.72), 0.05, 1.4);
  const cloudAmount = () => clamp(Number(values.clouds ?? 0.78));
  const butterflyAmount = () => clamp(Number(values.butterflies ?? 0.72));

  function buildScene(w, h) {
    const c = layer(w, h);
    const g = c.getContext("2d");
    const m = Math.min(w, h);
    if (reference?.naturalWidth) {
      const sourceRatio = reference.naturalWidth / reference.naturalHeight;
      if (sourceRatio > 1.5) {
        g.drawImage(reference, 0, 0, reference.naturalWidth, reference.naturalHeight, 0, 0, w, h);
      } else {
        // Older square reference files use a wide crop that keeps the sky,
        // mountain ridge, ocean and first flower field in one composition.
        const cropHeight = reference.naturalWidth * 0.5625;
        const cropTop = reference.naturalHeight * 0.1;
        g.drawImage(reference, 0, cropTop, reference.naturalWidth, cropHeight, 0, 0, w, h);
      }
      const veil = g.createLinearGradient(0, 0, 0, h);
      veil.addColorStop(0, "rgba(46,137,220,.05)");
      veil.addColorStop(0.52, "rgba(255,255,255,.03)");
      veil.addColorStop(1, "rgba(35,115,67,.08)");
      g.fillStyle = veil;
      g.fillRect(0, 0, w, h);
      return c;
    }
    const skyBottom = h * 0.64;
    const sky = g.createLinearGradient(0, 0, 0, skyBottom);
    sky.addColorStop(0, "#2c83dc");
    sky.addColorStop(0.42, "#65afe8");
    sky.addColorStop(0.78, "#b7daf1");
    sky.addColorStop(1, "#eaf0dc");
    g.fillStyle = sky;
    g.fillRect(0, 0, w, h);

    const sunX = w * 0.92;
    const sunY = h * 0.08;
    const sun = g.createRadialGradient(sunX, sunY, 0, sunX, sunY, m * 0.52);
    sun.addColorStop(0, "rgba(255,255,239,.98)");
    sun.addColorStop(0.12, "rgba(255,248,214,.76)");
    sun.addColorStop(0.52, "rgba(255,240,196,.18)");
    sun.addColorStop(1, "rgba(255,240,196,0)");
    g.fillStyle = sun;
    g.fillRect(0, 0, w, skyBottom);

    const distant = [[0, 0.47], [0.09, 0.39], [0.18, 0.45], [0.27, 0.33], [0.37, 0.4], [0.48, 0.27], [0.58, 0.38], [0.68, 0.22], [0.78, 0.34], [0.9, 0.25], [1, 0.38]];
    path(g, distant.map(([x, y]) => [x * w, y * h]).concat([[w, skyBottom], [0, skyBottom]]));
    g.fillStyle = "rgba(125,181,221,.7)";
    g.fill();

    const peaks = [[0, 0.52], [0.11, 0.42], [0.2, 0.49], [0.3, 0.34], [0.37, 0.48], [0.47, 0.2], [0.53, 0.39], [0.61, 0.28], [0.72, 0.45], [0.83, 0.3], [0.93, 0.44], [1, 0.38]];
    const mountain = g.createLinearGradient(0, h * 0.2, 0, skyBottom);
    mountain.addColorStop(0, "#5d91ca");
    mountain.addColorStop(1, "#9ac2d4");
    path(g, peaks.map(([x, y]) => [x * w, y * h]).concat([[w, skyBottom], [0, skyBottom]]));
    g.fillStyle = mountain;
    g.fill();

    const snow = [[0.47, 0.2], [0.5, 0.31], [0.53, 0.39], [0.49, 0.29], [0.47, 0.34], [0.44, 0.39]];
    path(g, snow.map(([x, y]) => [x * w, y * h]));
    g.fillStyle = "rgba(232,246,255,.68)";
    g.fill();

    const waterTop = h * 0.57;
    const water = g.createLinearGradient(0, waterTop, 0, h * 0.82);
    water.addColorStop(0, "#9ed9e7");
    water.addColorStop(0.42, "#62bad5");
    water.addColorStop(1, "#348fb9");
    g.fillStyle = water;
    g.fillRect(w * 0.48, waterTop, w * 0.52, h * 0.27);

    path(g, [[w * 0.58, h * 0.56], [w * 0.66, h * 0.52], [w * 0.75, h * 0.56], [w * 0.84, h * 0.51], [w, h * 0.55], [w, h * 0.7], [w * 0.53, h * 0.7]]);
    g.fillStyle = "#a4d27f";
    g.fill();
    path(g, [[w * 0.64, h * 0.58], [w * 0.73, h * 0.55], [w * 0.82, h * 0.58], [w * 0.93, h * 0.54], [w, h * 0.57], [w, h * 0.7], [w * 0.62, h * 0.69]]);
    g.fillStyle = "#7bb96f";
    g.fill();

    const farCliff = g.createLinearGradient(0, h * 0.53, 0, h * 0.75);
    farCliff.addColorStop(0, "#a7bfae");
    farCliff.addColorStop(1, "#4f8790");
    path(g, [[0, h * 0.56], [w * 0.15, h * 0.51], [w * 0.27, h * 0.57], [w * 0.39, h * 0.52], [w * 0.51, h * 0.61], [w * 0.6, h * 0.58], [w * 0.68, h * 0.7], [0, h * 0.76]]);
    g.fillStyle = farCliff;
    g.fill();

    const meadow = g.createLinearGradient(0, h * 0.51, 0, h);
    meadow.addColorStop(0, "#a5d976");
    meadow.addColorStop(0.38, "#73c65f");
    meadow.addColorStop(1, "#2e8a4d");
    path(g, [[0, h * 0.53], [w * 0.16, h * 0.5], [w * 0.3, h * 0.61], [w * 0.45, h * 0.57], [w * 0.62, h * 0.68], [w * 0.78, h * 0.61], [w, h * 0.68], [w, h], [0, h]]);
    g.fillStyle = meadow;
    g.fill();

    const foregroundGlow = g.createRadialGradient(w * 0.48, h * 0.9, 0, w * 0.48, h * 0.9, w * 0.55);
    foregroundGlow.addColorStop(0, "rgba(208,246,124,.45)");
    foregroundGlow.addColorStop(1, "rgba(208,246,124,0)");
    g.fillStyle = foregroundGlow;
    g.fillRect(0, h * 0.54, w, h * 0.46);

    // 左侧的悬崖和树，把画面拉出远近关系。
    path(g, [[0, h * 0.31], [w * 0.08, h * 0.27], [w * 0.2, h * 0.32], [w * 0.22, h * 0.47], [w * 0.16, h * 0.52], [0, h * 0.46]]);
    g.fillStyle = "#91b96d";
    g.fill();
    path(g, [[0, h * 0.45], [w * 0.18, h * 0.42], [w * 0.22, h * 0.47], [w * 0.17, h * 0.55], [0, h * 0.53]]);
    g.fillStyle = "#b6aa80";
    g.fill();
    for (let i = 0; i < 34; i++) {
      const x = w * (0.015 + hash(i + 50) * 0.2);
      const y = h * (0.27 + hash(i + 51) * 0.18);
      const r = m * (0.008 + hash(i + 52) * 0.016);
      const leaf = g.createRadialGradient(x, y, 0, x, y, r);
      leaf.addColorStop(0, "rgba(46,133,67,.9)");
      leaf.addColorStop(1, "rgba(90,164,73,0)");
      g.fillStyle = leaf;
      g.fillRect(x - r, y - r, r * 2, r * 2);
    }

    // 远处海岸线的白色浪花。
    g.lineCap = "round";
    g.strokeStyle = "rgba(236,255,250,.62)";
    g.lineWidth = Math.max(1, m * 0.0018);
    for (let i = 0; i < 42; i++) {
      const y = h * (0.59 + hash(i + 200) * 0.17);
      const x = w * (0.59 + hash(i + 201) * 0.39);
      const len = w * (0.008 + hash(i + 202) * 0.032);
      g.beginPath();
      g.moveTo(x, y);
      g.quadraticCurveTo(x + len * 0.45, y - m * 0.002, x + len, y);
      g.stroke();
    }

    return c;
  }

  function rebuild(w, h) {
    background = buildScene(w, h);
    grain = ctx.createPattern(grainTile(192, 14), "repeat");
    clouds = Array.from({ length: 13 }, (_, i) => ({
      x: hash(i + 410) * 1.28 - 0.12,
      y: 0.1 + hash(i + 411) * 0.35,
      size: 0.018 + hash(i + 412) * 0.042,
      speed: 0.012 + hash(i + 413) * 0.03,
      depth: 0.55 + hash(i + 414) * 0.45,
      seed: i + 1,
    }));
    grass = Array.from({ length: 260 }, (_, i) => ({
      x: hash(i + 600),
      y: 0.66 + hash(i + 601) * 0.35,
      height: 0.018 + hash(i + 602) * 0.075,
      width: 0.0008 + hash(i + 603) * 0.0018,
      phase: hash(i + 604) * TAU,
      hue: hash(i + 605),
      layer: hash(i + 606),
    }));
    flowers = Array.from({ length: 210 }, (_, i) => ({
      x: hash(i + 800),
      y: 0.66 + Math.pow(hash(i + 801), 0.82) * 0.34,
      size: 0.002 + hash(i + 802) * 0.006,
      phase: hash(i + 803) * TAU,
      type: Math.floor(hash(i + 804) * FLOWERS.length),
      depth: hash(i + 805),
    }));
    pollen = Array.from({ length: 58 }, (_, i) => ({
      x: hash(i + 900),
      y: 0.48 + hash(i + 901) * 0.47,
      size: 0.0008 + hash(i + 902) * 0.0022,
      speed: 0.012 + hash(i + 903) * 0.035,
      phase: hash(i + 904) * TAU,
    }));
    butterflies = Array.from({ length: 7 }, (_, i) => ({
      x: 0.12 + hash(i + 1000) * 0.78,
      y: 0.26 + hash(i + 1001) * 0.48,
      size: 0.006 + hash(i + 1002) * 0.008,
      speed: 0.15 + hash(i + 1003) * 0.23,
      phase: hash(i + 1004) * TAU,
      color: ["#fff1a8", "#ffb3d1", "#c6ebff", "#ffd48e"][i % 4],
    }));
    waves = Array.from({ length: 17 }, (_, i) => ({
      y: 0.58 + hash(i + 1100) * 0.2,
      x: 0.56 + hash(i + 1101) * 0.45,
      length: 0.025 + hash(i + 1102) * 0.1,
      phase: hash(i + 1103) * TAU,
      speed: 0.12 + hash(i + 1104) * 0.2,
    }));
  }

  function drawClouds(w, h, t) {
    const amount = cloudAmount();
    for (const cloud of clouds) {
      const x = ((cloud.x + t * cloud.speed * speed() * 0.018) % 1.3 + 1.3) % 1.3 - 0.12;
      drawCloud(ctx, { ...cloud, x }, w, h, (0.16 + cloud.depth * 0.26) * (0.42 + amount * 0.5));
    }
  }

  function drawWaves(w, h, t) {
    ctx.save();
    ctx.lineCap = "round";
    ctx.lineWidth = Math.max(1, Math.min(w, h) * 0.0015);
    for (const wave of waves) {
      const y = wave.y * h;
      const x = wave.x * w + Math.sin(t * wave.speed + wave.phase) * w * 0.012;
      const length = wave.length * w;
      ctx.strokeStyle = `rgba(233,255,250,${0.16 + (1 - wave.y) * 0.28})`;
      ctx.beginPath();
      ctx.moveTo(x, y);
      ctx.bezierCurveTo(x + length * 0.28, y - h * 0.005, x + length * 0.68, y + h * 0.004, x + length, y);
      ctx.stroke();
    }
    ctx.restore();
  }

  function drawGrass(w, h, t) {
    const wind = breeze();
    const m = Math.min(w, h);
    ctx.save();
    ctx.lineCap = "round";
    for (const blade of grass) {
      const x = blade.x * w;
      const base = blade.y * h;
      const height = blade.height * h;
      const wave = Math.sin(t * (0.55 + blade.layer * 0.6) + blade.phase + blade.x * 6) * wind;
      const lean = wave * height * (0.18 + blade.layer * 0.45);
      const y = base - height;
      ctx.strokeStyle = blade.hue > 0.72 ? "rgba(53,142,71,.72)" : blade.hue > 0.32 ? "rgba(82,171,70,.76)" : "rgba(128,190,65,.76)";
      ctx.lineWidth = Math.max(1, m * blade.width * (0.65 + blade.layer * 0.7));
      ctx.beginPath();
      ctx.moveTo(x, base);
      ctx.quadraticCurveTo(x + lean * 0.45, y + height * 0.35, x + lean, y);
      ctx.stroke();
    }
    ctx.restore();
  }

  function drawFlowers(w, h, t) {
    const m = Math.min(w, h);
    for (const flower of flowers) {
      const [petal, center] = FLOWERS[flower.type];
      const x = flower.x * w + Math.sin(t * 0.35 + flower.phase) * m * 0.0022;
      const y = flower.y * h;
      const size = flower.size * m * (0.75 + flower.depth * 0.75);
      ctx.save();
      ctx.translate(x, y);
      ctx.rotate(Math.sin(t * 0.5 + flower.phase) * breeze() * 0.18);
      ctx.globalAlpha = 0.28 + flower.depth * 0.72;
      ctx.fillStyle = `rgba(${petal},.88)`;
      for (let k = 0; k < 5; k++) {
        ctx.save();
        ctx.rotate((k / 5) * TAU);
        ctx.beginPath();
        ctx.ellipse(0, -size * 0.64, size * 0.43, size * 0.72, 0, 0, TAU);
        ctx.fill();
        ctx.restore();
      }
      ctx.fillStyle = center;
      ctx.beginPath();
      ctx.arc(0, 0, size * 0.26, 0, TAU);
      ctx.fill();
      ctx.restore();
    }
  }

  function drawPollen(w, h, t) {
    const m = Math.min(w, h);
    ctx.save();
    ctx.globalCompositeOperation = "screen";
    for (const mote of pollen) {
      const x = ((mote.x + t * mote.speed * speed() * 0.012) % 1.12 + 1.12) % 1.12 - 0.06;
      const y = mote.y + Math.sin(t * 0.22 + mote.phase) * 0.012;
      const alpha = 0.12 + 0.22 * (0.5 + 0.5 * Math.sin(t * 0.7 + mote.phase));
      ctx.fillStyle = `rgba(255,247,177,${alpha})`;
      ctx.beginPath();
      ctx.arc(x * w, y * h, mote.size * m, 0, TAU);
      ctx.fill();
    }
    ctx.restore();
  }

  function drawButterflies(w, h, t) {
    const amount = butterflyAmount();
    const m = Math.min(w, h);
    ctx.save();
    ctx.globalCompositeOperation = "screen";
    for (let i = 0; i < butterflies.length; i++) {
      const butterfly = butterflies[i];
      const visible = clamp(amount * 1.3 - i * 0.12);
      if (visible <= 0.02) continue;
      const phase = t * butterfly.speed * speed() + butterfly.phase;
      const x = butterfly.x * w + Math.sin(phase * 0.83) * w * 0.035;
      const y = butterfly.y * h + Math.sin(phase * 1.37) * h * 0.035;
      const flap = 0.56 + Math.abs(Math.sin(phase * 4.2)) * 0.44;
      const size = butterfly.size * m;
      ctx.save();
      ctx.translate(x, y);
      ctx.rotate(Math.sin(phase * 0.7) * 0.45);
      ctx.globalAlpha = visible * (0.68 + 0.2 * Math.sin(phase * 0.5));
      ctx.fillStyle = butterfly.color;
      ctx.beginPath();
      ctx.ellipse(-size * 0.6, -size * 0.14, size * 0.62, size * flap, -0.35, 0, TAU);
      ctx.ellipse(size * 0.6, -size * 0.14, size * 0.62, size * flap, 0.35, 0, TAU);
      ctx.fill();
      ctx.fillStyle = "rgba(97,75,76,.7)";
      ctx.beginPath();
      ctx.ellipse(0, 0, size * 0.12, size * 0.72, 0, 0, TAU);
      ctx.fill();
      ctx.strokeStyle = "rgba(97,75,76,.54)";
      ctx.lineWidth = Math.max(1, m * 0.001);
      ctx.beginPath();
      ctx.moveTo(-size * 0.03, -size * 0.4);
      ctx.quadraticCurveTo(-size * 0.35, -size * 0.84, -size * 0.5, -size * 0.65);
      ctx.moveTo(size * 0.03, -size * 0.4);
      ctx.quadraticCurveTo(size * 0.35, -size * 0.84, size * 0.5, -size * 0.65);
      ctx.stroke();
      ctx.restore();
    }
    ctx.restore();
  }

  return {
    update(next) {
      values = next ?? {};
    },
    render({ time }) {
      const w = canvas.width;
      const h = canvas.height;
      const key = `${w}x${h}`;
      if (!background || key !== signature) {
        rebuild(w, h);
        signature = key;
      }
      const t = time * speed();
      ctx.drawImage(background, 0, 0);
      drawClouds(w, h, t);
      drawWaves(w, h, t);
      drawPollen(w, h, t);
      drawGrass(w, h, t);
      drawFlowers(w, h, t);
      drawButterflies(w, h, t);
      ctx.fillStyle = grain;
      ctx.globalAlpha = 0.38;
      ctx.fillRect(0, 0, w, h);
      ctx.globalAlpha = 1;
    },
    resize() {
      background = null;
      signature = "";
    },
    destroy() {
      background = null;
      grain = null;
      clouds = [];
      grass = [];
      flowers = [];
      pollen = [];
      butterflies = [];
      waves = [];
    },
  };
}
