// 云海：日出时分，七层远近的山脊浮在缓缓流动的云雾里，远处有飞鸟。自包含 Canvas 2D 场景。
const TAU = Math.PI * 2;
const hash = n => { const v = Math.sin(n * 127.1 + 311.7) * 43758.5453; return v - Math.floor(v); };
const mix = (a, b, t) => a + (b - a) * t;
const smooth = t => t * t * (3 - 2 * t);
const clamp = (v, a = 0, b = 1) => Math.min(b, Math.max(a, v));
const noise = x => { const i = Math.floor(x), f = x - i; return mix(hash(i), hash(i + 1), smooth(f)); };
const fbm = x => (noise(x) * .5 + noise(x * 2.07 + 19.7) * .25 + noise(x * 4.3 + 41.1) * .125) / .875;
const layer = (w, h) => { const c = document.createElement('canvas'); c.width = Math.max(1, Math.round(w)); c.height = Math.max(1, Math.round(h)); return c; };
const rgb = hex => [1, 3, 5].map(i => parseInt(hex.slice(i, i + 2), 16));
const lerp3 = (a, b, k) => a.map((v, i) => Math.round(mix(v, b[i], k)));
function grainTile(size, amount) {
  const c = layer(size, size), g = c.getContext('2d'), img = g.createImageData(size, size), d = img.data;
  for (let i = 0; i < d.length; i += 4) { const v = Math.random() * 2 - 1; d[i] = d[i + 1] = d[i + 2] = v > 0 ? 255 : 0; d[i + 3] = Math.abs(v) * amount; }
  g.putImageData(img, 0, 0); return c;
}
// tone 0 = 青岚日出，0.5 = 晨曦金粉，1 = 暮云紫金
const TONES = [
  { sky: ['#4c7fa8', '#8db9c4', '#d6dcc8', '#f9e4bb', '#fff3d6'], glow: '255,232,180', mist: [240, 248, 240], far: '#a9c7c3', near: '#173a46' },
  { sky: ['#6f8fd0', '#b9a6dc', '#f4b9bd', '#ffd9ae', '#fff0d0'], glow: '255,224,168', mist: [255, 235, 225], far: '#d3b2c6', near: '#2b2d57' },
  { sky: ['#2e3c7d', '#6c58a1', '#d3748f', '#f6a86b', '#ffd69a'], glow: '255,190,120', mist: [255, 214, 200], far: '#c27f96', near: '#1a1b45' }
];

export default function create({ canvas, settings = {} }) {
  const ctx = canvas.getContext('2d');
  if (!ctx) throw new Error('无法绘制云海');
  let values = settings, bg = null, mist = null, sig = '', pattern = null, ridges = [];
  const tone = () => clamp(Number(values.tone ?? .5));
  const pick = () => { const k = tone() * 2, i = Math.min(1, Math.floor(k)), a = TONES[i], b = TONES[i + 1], f = k - i; return { glow: f < .5 ? a.glow : b.glow, sky: a.sky.map((c, j) => `rgb(${lerp3(rgb(c), rgb(b.sky[j]), f)})`), mist: lerp3(a.mist, b.mist, f), far: lerp3(rgb(a.far), rgb(b.far), f), near: lerp3(rgb(a.near), rgb(b.near), f), glowRgb: lerp3(a.glow.split(',').map(Number), b.glow.split(',').map(Number), f).join(',') }; };

  function mistSprite(p) {
    const c = layer(1400, 220), g = c.getContext('2d');
    for (let i = 0; i < 22; i++) {
      const x = hash(i * 3 + 1) * 1400, y = 60 + hash(i * 3 + 2) * 100, rx = 130 + hash(i * 3 + 3) * 170, ry = 28 + hash(i + 40) * 36;
      for (const off of [-1400, 0, 1400]) {
        g.save(); g.translate(x + off, y); g.scale(rx, ry);
        const r = g.createRadialGradient(0, 0, 0, 0, 0, 1); r.addColorStop(0, `rgba(${p.mist},.62)`); r.addColorStop(.6, `rgba(${p.mist},.26)`); r.addColorStop(1, `rgba(${p.mist},0)`);
        g.fillStyle = r; g.beginPath(); g.arc(0, 0, 1, 0, TAU); g.fill(); g.restore();
      }
    }
    return c;
  }
  function build(w, h, p) {
    const c = layer(w, h), g = c.getContext('2d'), sx = w * .3, sy = h * .4;
    const sky = g.createLinearGradient(0, 0, 0, h * .78); p.sky.forEach((col, i) => sky.addColorStop(i / (p.sky.length - 1), col));
    g.fillStyle = sky; g.fillRect(0, 0, w, h);
    const halo = g.createRadialGradient(sx, sy, 0, sx, sy, Math.max(w, h) * .6);
    halo.addColorStop(0, `rgba(${p.glowRgb},.85)`); halo.addColorStop(.12, `rgba(${p.glowRgb},.4)`); halo.addColorStop(1, `rgba(${p.glowRgb},0)`);
    g.fillStyle = halo; g.fillRect(0, 0, w, h);
    const disc = g.createRadialGradient(sx, sy, 0, sx, sy, Math.min(w, h) * .05); disc.addColorStop(0, 'rgba(255,252,240,1)'); disc.addColorStop(.8, 'rgba(255,250,235,.95)'); disc.addColorStop(1, `rgba(${p.glowRgb},0)`);
    g.fillStyle = disc; g.beginPath(); g.arc(sx, sy, Math.min(w, h) * .05, 0, TAU); g.fill();
    return c;
  }
  function buildRidges(w, h, p) {
    ridges = [];
    for (let i = 0; i < 7; i++) {
      const f = Math.pow(i / 6, .85), base = h * (.47 + i * .07), amp = h * (.075 + i * .012), freq = 1.4 + i * .55, pts = [];
      const sharp = .55 - f * .3;
      for (let x = 0; x <= w + 10; x += Math.max(5, w / 320)) {
        const u = x / w * freq, soft = Math.pow(fbm(u + i * 13.7), 1.2), peak = Math.pow(1 - Math.abs(2 * noise(u * 1.35 + i * 5.1) - 1), 1.2);
        pts.push([x, base - amp * mix(soft, peak, sharp) * 1.7]);
      }
      ridges.push({ pts, base, amp, color: lerp3(p.far, p.near, f), f });
    }
  }
  function drawRidges(w, h, t, p) {
    for (let i = 0; i < ridges.length; i++) {
      const r = ridges[i], top = r.base - r.amp * 1.7, grad = ctx.createLinearGradient(0, top, 0, h);
      grad.addColorStop(0, `rgb(${r.color})`); grad.addColorStop(1, `rgb(${lerp3(r.color, p.mist, .28 + (1 - r.f) * .3)})`);
      ctx.fillStyle = grad; ctx.beginPath(); ctx.moveTo(0, h);
      for (const [x, y] of r.pts) ctx.lineTo(x, y);
      ctx.lineTo(w, h); ctx.closePath(); ctx.fill();
      if (i < ridges.length - 1) {
        const sw = w * 1.15, off = -((t * (.006 + i * .0035) * w) % sw), y = r.base - r.amp * .35, hh = h * .17, a = .5 + .35 * (1 - r.f);
        ctx.globalAlpha = a;
        for (let k = 0; k < 2; k++) ctx.drawImage(mist, off + k * sw, y, sw, hh);
        if (sw < w + 2 * Math.abs(off)) ctx.drawImage(mist, off + 2 * sw, y, sw, hh);
        ctx.globalAlpha = 1;
      }
    }
  }
  function birds(w, h, t) {
    ctx.strokeStyle = 'rgba(48,46,84,.62)'; ctx.lineCap = 'round'; ctx.lineWidth = Math.max(1, w / 1000);
    for (let i = 0; i < 5; i++) {
      const x = ((hash(i) + t * .012 * (.6 + hash(i + 4))) % 1.2 - .1) * w, y = h * (.2 + hash(i + 8) * .16) + Math.sin(t * .5 + i * 2) * h * .012, s = w * (.008 + hash(i + 12) * .007), flap = Math.sin(t * 3.2 + i * 1.7) * s * .55;
      ctx.beginPath(); ctx.moveTo(x - s, y - flap); ctx.quadraticCurveTo(x - s * .45, y - s * .5 - flap * .4, x, y); ctx.quadraticCurveTo(x + s * .45, y - s * .5 - flap * .4, x + s, y - flap); ctx.stroke();
    }
  }
  function rebuild(w, h) {
    const p = pick(); bg = build(w, h, p); mist = mistSprite(p); buildRidges(w, h, p);
    pattern = ctx.createPattern(grainTile(192, 16), 'repeat');
    const vignette = bg.getContext('2d'), v = vignette.createRadialGradient(w / 2, h * .5, Math.min(w, h) * .45, w / 2, h * .5, Math.max(w, h) * .8);
    v.addColorStop(0, 'rgba(20,20,50,0)'); v.addColorStop(1, 'rgba(20,20,50,.28)'); vignette.fillStyle = v; vignette.fillRect(0, 0, w, h);
  }
  return {
    update(next) { values = next; },
    render({ time }) {
      const w = canvas.width, h = canvas.height, t = time * Number(values.speed ?? .6), key = `${w}x${h}|${tone().toFixed(2)}`;
      if (!bg || key !== sig) { rebuild(w, h); sig = key; }
      const p = pick();
      ctx.drawImage(bg, 0, 0); drawRidges(w, h, t, p); birds(w, h, t);
      ctx.fillStyle = pattern; ctx.fillRect(0, 0, w, h);
    },
    resize() { bg = null; },
    destroy() { bg = null; mist = null; pattern = null; }
  };
}
