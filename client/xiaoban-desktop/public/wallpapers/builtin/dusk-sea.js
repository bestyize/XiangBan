// 暮海：落日、海面波光与薄云。自包含 Canvas 2D 场景，静态层只在尺寸或参数变化时重画。
const TAU = Math.PI * 2;
const hash = n => { const v = Math.sin(n * 127.1 + 311.7) * 43758.5453; return v - Math.floor(v); };
const mix = (a, b, t) => a + (b - a) * t;
const smooth = t => t * t * (3 - 2 * t);
const noise = x => { const i = Math.floor(x), f = x - i; return mix(hash(i), hash(i + 1), smooth(f)); };
const fbm = x => (noise(x) * .5 + noise(x * 2.07 + 19.7) * .25 + noise(x * 4.3 + 41.1) * .125) / .875;
const layer = (w, h) => { const c = document.createElement('canvas'); c.width = Math.max(1, Math.round(w)); c.height = Math.max(1, Math.round(h)); return c; };
const rgb = hex => [1, 3, 5].map(i => parseInt(hex.slice(i, i + 2), 16));
const lerpColor = (a, b, t) => { const x = rgb(a), y = rgb(b); return `rgb(${x.map((v, i) => Math.round(mix(v, y[i], t))).join(',')})`; };
const lerpRgb = (a, b, t) => a.map((v, i) => Math.round(mix(v, b[i], t))).join(',');
function grainTile(size, amount) {
  const c = layer(size, size), g = c.getContext('2d'), img = g.createImageData(size, size), d = img.data;
  for (let i = 0; i < d.length; i += 4) { const v = Math.random() * 2 - 1; d[i] = d[i + 1] = d[i + 2] = v > 0 ? 255 : 0; d[i + 3] = Math.abs(v) * amount; }
  g.putImageData(img, 0, 0); return c;
}
// warmth 0 = 粉紫黄昏，1 = 金橘暮色
const DUSK = {
  sky: [['#5b6bc0', '#1b2a5c'], ['#b58bd0', '#5d4b91'], ['#f3a3c4', '#d9728f'], ['#ffcfc0', '#f7a466'], ['#fff0d6', '#ffd9a0']],
  sea: [['#f6b4b5', '#f2a67b'], ['#9a79b8', '#7c4f78'], ['#343a78', '#1a2447']],
  glow: [[255, 205, 200], [255, 196, 120]], cloud: [[255, 222, 232], [255, 205, 170]], sun: ['#fff4ea', '#fff1c9'], isle: ['#4a3a72', '#2a2540']
};

export default function create({ canvas, settings = {} }) {
  const ctx = canvas.getContext('2d');
  if (!ctx) throw new Error('无法绘制暮海');
  let values = settings, bg = null, sig = '', pattern = null;
  const warmth = () => Math.min(1, Math.max(0, Number(values.warmth ?? .7)));
  const palette = () => {
    const k = warmth();
    return {
      sky: DUSK.sky.map(([a, b]) => lerpColor(a, b, k)), sea: DUSK.sea.map(([a, b]) => lerpColor(a, b, k)),
      glow: lerpRgb(DUSK.glow[0], DUSK.glow[1], k), cloud: lerpRgb(DUSK.cloud[0], DUSK.cloud[1], k),
      sun: lerpColor(DUSK.sun[0], DUSK.sun[1], k), isle: lerpColor(DUSK.isle[0], DUSK.isle[1], k)
    };
  };
  const geometry = (w, h) => ({ hor: h * .6, sx: w * .64, sr: Math.min(w, h) * .058 });

  function island(g, x0, len, hor, height, color, seed) {
    g.beginPath(); g.moveTo(x0, hor + 1);
    for (let i = 0; i <= 60; i++) { const u = i / 60; g.lineTo(x0 + len * u, hor - height * Math.pow(Math.sin(Math.PI * u), 1.3) * (.55 + .45 * fbm(u * 5 + seed))); }
    g.lineTo(x0 + len, hor + 1); g.closePath(); g.fillStyle = color; g.fill();
  }
  function build(w, h) {
    const p = palette(), { hor, sx, sr } = geometry(w, h), sy = hor - sr * .35, c = layer(w, h), g = c.getContext('2d');
    const sky = g.createLinearGradient(0, 0, 0, hor); p.sky.forEach((col, i) => sky.addColorStop(i / (p.sky.length - 1), col));
    g.fillStyle = sky; g.fillRect(0, 0, w, hor + 1);
    const halo = g.createRadialGradient(sx, sy, 0, sx, sy, Math.max(w, h) * .55);
    halo.addColorStop(0, `rgba(${p.glow},.6)`); halo.addColorStop(.16, `rgba(${p.glow},.26)`); halo.addColorStop(1, `rgba(${p.glow},0)`);
    g.fillStyle = halo; g.fillRect(0, 0, w, hor + 1);
    const sea = g.createLinearGradient(0, hor, 0, h); p.sea.forEach((col, i) => sea.addColorStop(i / (p.sea.length - 1), col));
    g.fillStyle = sea; g.fillRect(0, hor, w, h - hor);
    g.save(); g.beginPath(); g.rect(0, 0, w, hor); g.clip();
    const disc = g.createRadialGradient(sx, sy, 0, sx, sy, sr * 1.25);
    disc.addColorStop(0, p.sun); disc.addColorStop(.78, p.sun); disc.addColorStop(1, `rgba(${p.glow},0)`);
    g.fillStyle = disc; g.beginPath(); g.arc(sx, sy, sr * 1.25, 0, TAU); g.fill(); g.restore();
    const bloom = g.createRadialGradient(sx, sy, sr * .8, sx, sy, sr * 5);
    bloom.addColorStop(0, `rgba(${p.glow},.5)`); bloom.addColorStop(1, `rgba(${p.glow},0)`);
    g.save(); g.beginPath(); g.rect(0, 0, w, hor + sr * .6); g.clip(); g.fillStyle = bloom; g.fillRect(sx - sr * 5, sy - sr * 5, sr * 10, sr * 10); g.restore();
    const column = g.createLinearGradient(0, hor, 0, h);
    column.addColorStop(0, `rgba(${p.glow},.24)`); column.addColorStop(1, `rgba(${p.glow},0)`);
    g.fillStyle = column; g.beginPath(); g.moveTo(sx - sr * .9, hor); g.lineTo(sx + sr * .9, hor); g.lineTo(sx + sr * 5.5, h); g.lineTo(sx - sr * 5.5, h); g.closePath(); g.fill();
    const haze = g.createLinearGradient(0, hor - h * .05, 0, hor + h * .035);
    haze.addColorStop(0, `rgba(${p.glow},0)`); haze.addColorStop(.55, `rgba(${p.glow},.4)`); haze.addColorStop(1, `rgba(${p.glow},0)`);
    g.fillStyle = haze; g.fillRect(0, hor - h * .05, w, h * .085);
    island(g, w * .83, w * .15, hor, h * .034, p.isle, 3); island(g, w * .04, w * .22, hor, h * .02, p.isle, 9);
    g.fillStyle = `rgba(${p.glow},.3)`; g.fillRect(0, hor - h * .012, w, h * .014);
    const vignette = g.createRadialGradient(w / 2, h * .55, Math.min(w, h) * .4, w / 2, h * .55, Math.max(w, h) * .78);
    vignette.addColorStop(0, 'rgba(12,14,40,0)'); vignette.addColorStop(1, 'rgba(12,14,40,.42)');
    g.fillStyle = vignette; g.fillRect(0, 0, w, h);
    return c;
  }
  function clouds(w, h, t, p) {
    const hor = h * .6;
    for (let i = 0; i < 7; i++) {
      const x = ((hash(i) * 1.4 + t * (.006 + hash(i + 3) * .012)) % 1.4 - .2) * w, y = hor * (.12 + hash(i + 11) * .62);
      const rx = w * (.1 + hash(i + 21) * .14), ry = rx * (.05 + hash(i + 31) * .05);
      for (let j = -1; j <= 1; j++) {
        ctx.save(); ctx.translate(x + j * rx * .55, y + Math.abs(j) * ry * .25); ctx.scale(rx * (j ? .6 : 1), ry * (j ? .8 : 1));
        const g = ctx.createRadialGradient(0, 0, 0, 0, 0, 1);
        g.addColorStop(0, `rgba(${p.cloud},${.3 + hash(i + 41 + j) * .26})`); g.addColorStop(1, `rgba(${p.cloud},0)`);
        ctx.fillStyle = g; ctx.beginPath(); ctx.arc(0, 0, 1, 0, TAU); ctx.fill(); ctx.restore();
      }
    }
  }
  function sea(w, h, t, p) {
    const { hor, sx, sr } = geometry(w, h), rows = 48;
    ctx.lineWidth = Math.max(1, w / 1800);
    for (let i = 0; i < 7; i++) {
      const u = .12 + hash(i + 50) * .8, cx = ((hash(i + 60) * 1.5 + t * (.004 + hash(i + 70) * .006)) % 1.5 - .25) * w, cy = hor + Math.pow(u, 1.6) * (h - hor), rx = w * (.16 + hash(i + 80) * .18), ry = (h - hor) * (.025 + u * .05);
      ctx.save(); ctx.translate(cx, cy); ctx.scale(rx, ry);
      const swell = ctx.createRadialGradient(0, 0, 0, 0, 0, 1); swell.addColorStop(0, 'rgba(24,18,60,.14)'); swell.addColorStop(1, 'rgba(24,18,60,0)');
      ctx.fillStyle = swell; ctx.beginPath(); ctx.arc(0, 0, 1, 0, TAU); ctx.fill(); ctx.restore();
    }
    for (let i = 1; i <= 36; i++) {
      const u = i / 36, y = hor + Math.pow(u, 1.9) * (h - hor), amp = (h - hor) * (.0015 + u * .006), k = TAU * (3 + (1 - u) * 5) / w;
      ctx.strokeStyle = `rgba(255,255,255,${.025 + u * .05})`; ctx.beginPath();
      for (let x = 0; x <= w + 24; x += 24) { const yy = y + Math.sin(x * k + t * (.5 + u * .8) + i * 1.9) * amp; x ? ctx.lineTo(x, yy) : ctx.moveTo(x, yy); }
      ctx.stroke();
    }
    for (let j = 0; j < rows; j++) {
      const u = j / rows, y = hor + Math.pow(u, 1.65) * (h - hor) * .92 + 2, wob = .55 + .45 * Math.sin(t * 1.7 + j * 1.3 + hash(j) * 6);
      const len = sr * (1.1 + u * 5.2) * wob * (.7 + .5 * hash(j + 7)), off = Math.sin(t * .9 + j * .7) * sr * (.2 + u * .8);
      const a = .55 * (1 - u * .75) * (.55 + .45 * Math.sin(t * 2.1 + j * 2.7));
      ctx.fillStyle = `rgba(${p.glow},${Math.max(0, a)})`; ctx.fillRect(sx + off - len / 2, y, len, Math.max(1, (h - hor) * (.0035 + u * .008)));
    }
    ctx.fillStyle = '#fff';
    for (let i = 0; i < 130; i++) {
      const u = Math.pow(hash(i * 5 + 1), 1.4), y = hor + u * (h - hor) * .9 + 3, x = sx + (hash(i * 3 + 2) - .5) * sr * (3 + u * 14);
      const tw = Math.max(0, Math.sin(t * (1.5 + hash(i) * 2) + i * 4.1)); ctx.globalAlpha = tw * tw * .6 * (1 - u * .6);
      ctx.fillRect(x, y, Math.max(1.5, sr * (.12 + u * .3)), Math.max(1, (h - hor) * .0025));
    }
    ctx.globalAlpha = 1;
  }

  return {
    update(next) { values = next; },
    render({ time }) {
      const w = canvas.width, h = canvas.height, t = time * Number(values.speed ?? .6), key = `${w}x${h}|${warmth().toFixed(2)}`;
      if (!bg || key !== sig) { bg = build(w, h); sig = key; pattern = ctx.createPattern(grainTile(192, 20), 'repeat'); }
      const p = palette();
      ctx.drawImage(bg, 0, 0); clouds(w, h, t, p); sea(w, h, t, p);
      ctx.fillStyle = pattern; ctx.fillRect(0, 0, w, h);
    },
    resize() { bg = null; },
    destroy() { bg = null; pattern = null; }
  };
}
