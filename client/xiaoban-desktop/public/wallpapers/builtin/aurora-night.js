// 极光之夜：银河、闪烁的星、三层极光幕布与偶尔划过的流星。自包含 Canvas 2D 场景。
const TAU = Math.PI * 2;
const hash = n => { const v = Math.sin(n * 127.1 + 311.7) * 43758.5453; return v - Math.floor(v); };
const mix = (a, b, t) => a + (b - a) * t;
const smooth = t => t * t * (3 - 2 * t);
const clamp = (v, a = 0, b = 1) => Math.min(b, Math.max(a, v));
const noise = x => { const i = Math.floor(x), f = x - i; return mix(hash(i), hash(i + 1), smooth(f)); };
const fbm = x => (noise(x) * .5 + noise(x * 2.07 + 19.7) * .25 + noise(x * 4.3 + 41.1) * .125) / .875;
const layer = (w, h) => { const c = document.createElement('canvas'); c.width = Math.max(1, Math.round(w)); c.height = Math.max(1, Math.round(h)); return c; };
function grainTile(size, amount) {
  const c = layer(size, size), g = c.getContext('2d'), img = g.createImageData(size, size), d = img.data;
  for (let i = 0; i < d.length; i += 4) { const v = Math.random() * 2 - 1; d[i] = d[i + 1] = d[i + 2] = v > 0 ? 255 : 0; d[i + 3] = Math.abs(v) * amount; }
  g.putImageData(img, 0, 0); return c;
}
function hexToHsl(hex) {
  const [r, g, b] = [1, 3, 5].map(i => parseInt(hex.slice(i, i + 2), 16) / 255), max = Math.max(r, g, b), min = Math.min(r, g, b), l = (max + min) / 2, d = max - min;
  let h = 0; if (d) { h = max === r ? ((g - b) / d) % 6 : max === g ? (b - r) / d + 2 : (r - g) / d + 4; h *= 60; if (h < 0) h += 360; }
  return [h, d ? d / (1 - Math.abs(2 * l - 1)) : 0, l];
}
function hslToRgb(h, s, l) {
  const c = (1 - Math.abs(2 * l - 1)) * s, x = c * (1 - Math.abs((h / 60) % 2 - 1)), m = l - c / 2;
  const [r, g, b] = h < 60 ? [c, x, 0] : h < 120 ? [x, c, 0] : h < 180 ? [0, c, x] : h < 240 ? [0, x, c] : h < 300 ? [x, 0, c] : [c, 0, x];
  return [r + m, g + m, b + m].map(v => Math.round(v * 255));
}

export default function create({ canvas, settings = {} }) {
  const ctx = canvas.getContext('2d');
  if (!ctx) throw new Error('无法绘制极光之夜');
  let values = settings, bg = null, sig = '', sprites = [], pattern = null, stars = [];
  const accent = () => /^#[0-9a-f]{6}$/i.test(String(values.accent)) ? String(values.accent) : '#4dffb0';
  const curtainSprite = rgb => {
    const c = layer(2, 256), g = c.getContext('2d'), grad = g.createLinearGradient(0, 0, 0, 256);
    grad.addColorStop(0, `rgba(${rgb},0)`); grad.addColorStop(.3, `rgba(${rgb},.12)`); grad.addColorStop(.58, `rgba(${rgb},.48)`); grad.addColorStop(.75, `rgba(${rgb},.98)`); grad.addColorStop(.87, `rgba(${rgb},.4)`); grad.addColorStop(1, `rgba(${rgb},0)`);
    g.fillStyle = grad; g.fillRect(0, 0, 2, 256); return c;
  };
  function ridge(g, w, h, base, amp, freq, seed, fill) {
    g.beginPath(); g.moveTo(0, h);
    for (let x = 0; x <= w + 8; x += 8) g.lineTo(x, base - amp * Math.pow(fbm(x / w * freq + seed), 1.4) * 1.6);
    g.lineTo(w, h); g.closePath(); g.fillStyle = fill; g.fill();
  }
  function build(w, h) {
    const c = layer(w, h), g = c.getContext('2d'), s = Math.min(w, h);
    const sky = g.createLinearGradient(0, 0, 0, h);
    sky.addColorStop(0, '#030816'); sky.addColorStop(.45, '#0a1634'); sky.addColorStop(.78, '#143055'); sky.addColorStop(1, '#0b2a3a');
    g.fillStyle = sky; g.fillRect(0, 0, w, h);
    const ax = w * .12, ay = h * .78, bx = w * .92, by = h * .06, dx = bx - ax, dy = by - ay, len = Math.hypot(dx, dy), nx = -dy / len, ny = dx / len;
    for (let i = 0; i < 16; i++) {
      const u = i / 15, x = ax + dx * u, y = ay + dy * u, r = s * (.35 + .12 * Math.sin(u * 7));
      const band = g.createRadialGradient(x, y, 0, x, y, r); band.addColorStop(0, 'rgba(160,178,255,.15)'); band.addColorStop(1, 'rgba(150,170,255,0)');
      g.fillStyle = band; g.fillRect(x - r, y - r, r * 2, r * 2);
    }
    const dot = (x, y, r, a, col) => { g.fillStyle = `rgba(${col},${a})`; g.beginPath(); g.arc(x, y, r, 0, TAU); g.fill(); };
    for (let i = 0; i < 3200; i++) {
      const u = hash(i * 3 + 1), off = (hash(i * 5 + 2) + hash(i * 7 + 3) + hash(i * 11 + 4) - 1.5) * s * .34;
      dot(ax + dx * u + nx * off, ay + dy * u + ny * off, .4 + hash(i + 9) * 1.1, (.12 + hash(i + 17) * .5) * (1 - Math.abs(off) / (s * .34)), i % 5 ? '210,222,255' : '255,225,200');
    }
    for (let i = 0; i < 5; i++) {
      const u = .2 + hash(i + 70) * .6, x = ax + dx * u + nx * (hash(i + 80) - .5) * s * .1, y = ay + dy * u + ny * (hash(i + 90) - .5) * s * .1, r = s * (.05 + hash(i + 60) * .06);
      const lane = g.createRadialGradient(x, y, 0, x, y, r); lane.addColorStop(0, 'rgba(3,8,22,.14)'); lane.addColorStop(1, 'rgba(3,8,22,0)'); g.fillStyle = lane; g.fillRect(x - r, y - r, r * 2, r * 2);
    }
    for (let i = 0; i < 520; i++) dot(hash(i * 2) * w, hash(i * 2 + 1) * h * .8, .35 + hash(i + 400) * .9, .15 + hash(i + 700) * .6, '225,235,255');
    const glow = g.createRadialGradient(w / 2, h * .98, 0, w / 2, h * .98, w * .55);
    glow.addColorStop(0, 'rgba(60,170,170,.28)'); glow.addColorStop(1, 'rgba(60,170,170,0)'); g.fillStyle = glow; g.fillRect(0, 0, w, h);
    ridge(g, w, h, h * .78, h * .16, 2.4, 4, '#112848'); ridge(g, w, h, h * .86, h * .13, 2.9, 7, '#091730'); ridge(g, w, h, h * .9, h * .1, 3.1, 11, '#050b17');
    g.fillStyle = '#03070f';
    for (let i = 0; i < 70; i++) {
      const x = hash(i + 200) * w, base = h * .9 - h * .1 * Math.pow(fbm(x / w * 3.1 + 11), 1.4) * 1.6 + h * .02, th = h * (.035 + hash(i + 300) * .06);
      for (let k = 0; k < 4; k++) { const yy = base - th * (k / 4), half = th * .17 * (1 - k * .2); g.beginPath(); g.moveTo(x - half, yy); g.lineTo(x, yy - th * .4); g.lineTo(x + half, yy); g.fill(); }
    }
    g.fillStyle = '#03070f'; g.fillRect(0, h * .965, w, h * .04);
    const vignette = g.createRadialGradient(w / 2, h * .5, s * .45, w / 2, h * .5, Math.max(w, h) * .8);
    vignette.addColorStop(0, 'rgba(0,0,10,0)'); vignette.addColorStop(1, 'rgba(0,0,10,.5)'); g.fillStyle = vignette; g.fillRect(0, 0, w, h);
    return c;
  }
  function curtains(w, h, t) {
    const step = Math.max(2, Math.round(w / 420)), strength = clamp(Number(values.intensity ?? .8), .1, 1.3);
    ctx.globalCompositeOperation = 'lighter';
    for (let k = 0; k < 3; k++) {
      for (let x = 0; x <= w; x += step) {
        const u = x / w, base = h * (.36 + k * .06) + h * .07 * Math.sin(u * TAU * 1.1 + t * .24 + k * 2.1) + h * .07 * (fbm(u * 2.2 + t * .05 + k * 7) - .5);
        const tall = h * (.2 + .13 * fbm(u * 4 + t * .08 + k * 3)), fade = smooth(clamp(Math.min(u, 1 - u) / .16));
        const rays = noise(u * 70 + k * 13 + t * .6) * .6 + noise(u * 23 + k * 5 - t * .35) * .4;
        const a = clamp(.25 + 1.3 * (fbm(u * 6 + t * .25 + k * 5) - .3)) * fade * (.35 + .65 * rays) * strength * 1.15;
        if (a > .01) { ctx.globalAlpha = a; ctx.drawImage(sprites[k], 0, 0, 2, 256, x, base - tall, step, tall); }
      }
    }
    ctx.globalAlpha = 1; ctx.globalCompositeOperation = 'source-over';
  }
  function sparkle(w, h, time) {
    ctx.fillStyle = '#fff';
    for (const s of stars) { const a = .35 + .65 * Math.max(0, Math.sin(time * s.f + s.p)); ctx.globalAlpha = a * s.a; ctx.beginPath(); ctx.arc(s.x * w, s.y * h, s.r * w / 1600 + .6, 0, TAU); ctx.fill(); }
    ctx.globalAlpha = 1;
    const period = 9, id = Math.floor(time / period), p = (time % period) / period / .09;
    if (p < 1) {
      const sx = (.15 + hash(id) * .6) * w, sy = (.04 + hash(id + 9) * .22) * h, ang = .5 + hash(id + 5) * .3, tail = w * .16, hx = sx + Math.cos(ang) * tail * p * 2.2, hy = sy + Math.sin(ang) * tail * p * 2.2;
      const g = ctx.createLinearGradient(hx - Math.cos(ang) * tail, hy - Math.sin(ang) * tail, hx, hy);
      g.addColorStop(0, 'rgba(255,255,255,0)'); g.addColorStop(1, `rgba(255,255,255,${Math.sin(p * Math.PI)})`);
      ctx.strokeStyle = g; ctx.lineWidth = Math.max(1.2, w / 1100); ctx.lineCap = 'round'; ctx.beginPath(); ctx.moveTo(hx - Math.cos(ang) * tail, hy - Math.sin(ang) * tail); ctx.lineTo(hx, hy); ctx.stroke();
    }
  }
  function rebuild(w, h) {
    const [hue, sat] = hexToHsl(accent()), s = Math.max(.6, sat);
    sprites = [0, 55, 120].map(shift => curtainSprite(hslToRgb((hue + shift) % 360, s, .6).join(',')));
    bg = build(w, h); pattern = ctx.createPattern(grainTile(192, 14), 'repeat');
    stars = Array.from({ length: 90 }, (_, i) => ({ x: hash(i * 9 + 1), y: hash(i * 9 + 2) * .7, r: .5 + hash(i * 9 + 3) * 1.1, a: .5 + hash(i * 9 + 4) * .5, f: .8 + hash(i * 9 + 5) * 2.4, p: hash(i * 9 + 6) * TAU }));
  }
  return {
    update(next) { values = next; },
    render({ time }) {
      const w = canvas.width, h = canvas.height, t = time * Number(values.speed ?? .6), key = `${w}x${h}|${accent()}`;
      if (!bg || key !== sig) { rebuild(w, h); sig = key; }
      ctx.drawImage(bg, 0, 0); curtains(w, h, t); sparkle(w, h, time);
      ctx.fillStyle = pattern; ctx.fillRect(0, 0, w, h);
    },
    resize() { bg = null; },
    destroy() { bg = null; pattern = null; }
  };
}
