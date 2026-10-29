// 山水：宣纸、淡墨远山、流动的薄雾、一点红日、飞雁与小舟。自包含 Canvas 2D 场景。
const TAU = Math.PI * 2;
const hash = n => { const v = Math.sin(n * 127.1 + 311.7) * 43758.5453; return v - Math.floor(v); };
const mix = (a, b, t) => a + (b - a) * t;
const smooth = t => t * t * (3 - 2 * t);
const noise = x => { const i = Math.floor(x), f = x - i; return mix(hash(i), hash(i + 1), smooth(f)); };
const fbm = x => (noise(x) * .5 + noise(x * 2.07 + 19.7) * .25 + noise(x * 4.3 + 41.1) * .125) / .875;
const layer = (w, h) => { const c = document.createElement('canvas'); c.width = Math.max(1, Math.round(w)); c.height = Math.max(1, Math.round(h)); return c; };
function grainTile(size, amount) {
  const c = layer(size, size), g = c.getContext('2d'), img = g.createImageData(size, size), d = img.data;
  for (let i = 0; i < d.length; i += 4) { const v = Math.random() * 2 - 1; d[i] = d[i + 1] = d[i + 2] = v > 0 ? 255 : 0; d[i + 3] = Math.abs(v) * amount; }
  g.putImageData(img, 0, 0); return c;
}
const INK = '30,34,40', PAPER = '241,234,218';

export default function create({ canvas, settings = {} }) {
  const ctx = canvas.getContext('2d');
  if (!ctx) throw new Error('无法绘制山水');
  let values = settings, bg = null, mist = null, sig = '', pattern = null;
  const showSun = () => values.sun !== false;

  function paper(g, w, h) {
    const base = g.createRadialGradient(w * .5, h * .42, 0, w * .5, h * .42, Math.max(w, h) * .8);
    base.addColorStop(0, '#f6f0e0'); base.addColorStop(1, '#e4d9bf'); g.fillStyle = base; g.fillRect(0, 0, w, h);
    for (let i = 0; i < 46; i++) {
      const x = hash(i * 3) * w, y = hash(i * 3 + 1) * h, r = Math.min(w, h) * (.12 + hash(i * 3 + 2) * .3), b = g.createRadialGradient(x, y, 0, x, y, r);
      b.addColorStop(0, i % 3 ? 'rgba(170,140,90,.05)' : 'rgba(120,120,110,.04)'); b.addColorStop(1, 'rgba(170,140,90,0)'); g.fillStyle = b; g.fillRect(x - r, y - r, r * 2, r * 2);
    }
    g.lineCap = 'round';
    for (let i = 0; i < 2600; i++) {
      const x = hash(i * 7 + 5) * w, y = hash(i * 7 + 6) * h, a = hash(i * 7 + 7) * TAU, l = 5 + hash(i * 7 + 8) * 22;
      g.strokeStyle = i % 4 ? 'rgba(150,125,85,.07)' : 'rgba(255,252,240,.18)'; g.lineWidth = .6 + hash(i + 9) * .7;
      g.beginPath(); g.moveTo(x, y); g.quadraticCurveTo(x + Math.cos(a + .6) * l * .6, y + Math.sin(a + .6) * l * .6, x + Math.cos(a) * l, y + Math.sin(a) * l); g.stroke();
    }
  }
  function mountain(g, w, h, i, n) {
    const f = i / (n - 1), base = h * (.43 + f * .28), amp = h * (.15 - f * .02), freq = 2.1 + f * 1.1, alpha = [.2, .3, .46, .64, .86][i], pts = [];
    for (let x = -6; x <= w + 10; x += Math.max(4, w / 400)) {
      const u = x / w * freq, soft = fbm(u + i * 9.3), peak = Math.pow(1 - Math.abs(2 * noise(u * 1.4 + i * 4.1) - 1), 1.5);
      pts.push([x, base - amp * mix(soft, peak, .62 - f * .25) * 1.7]);
    }
    const top = base - amp * 1.7, wash = g.createLinearGradient(0, top, 0, base + amp * 1.1);
    wash.addColorStop(0, `rgba(${INK},${alpha})`); wash.addColorStop(.45, `rgba(${INK},${alpha * .42})`); wash.addColorStop(1, `rgba(${INK},0)`);
    const body = g.createLinearGradient(0, top, 0, base + amp * 1.1);
    body.addColorStop(0, `rgba(${PAPER},.92)`); body.addColorStop(.55, `rgba(${PAPER},.7)`); body.addColorStop(1, `rgba(${PAPER},0)`);
    for (const fill of [body, wash]) {
      g.fillStyle = fill; g.beginPath(); g.moveTo(-6, h);
      for (const [x, y] of pts) g.lineTo(x, y);
      g.lineTo(w + 10, h); g.closePath(); g.fill();
    }
    g.lineCap = 'round'; g.lineJoin = 'round';
    for (let pass = 0; pass < 2; pass++) {
      g.strokeStyle = `rgba(${INK},${alpha * (pass ? .35 : .6)})`; g.lineWidth = Math.max(1, w / 1200) * (1.2 + f * 1.6) * (pass ? 1.8 : 1);
      g.beginPath(); pts.forEach(([x, y], k) => { const jy = y + (noise(x * .08 + pass * 31) - .5) * h * .004; k ? g.lineTo(x, jy) : g.moveTo(x, jy); }); g.stroke();
    }
    for (let k = 0; k < 230; k++) {
      const idx = Math.floor(hash(k + i * 500) * (pts.length - 2)), [x, y] = pts[idx], slope = pts[idx + 1][1] - y, depth = Math.pow(hash(k + 77 + i * 31), 1.6) * amp * .5, sx = x + (hash(k + 5) - .5) * 14, sy = y + depth + 3;
      g.strokeStyle = `rgba(${INK},${alpha * (.1 + (1 - depth / (amp * .5)) * .3)})`; g.lineWidth = Math.max(.7, w / 1800) * (.7 + hash(k + 3));
      const len = (6 + hash(k + 9) * 16) * (w / 1600), dir = Math.sign(slope || 1); g.beginPath(); g.moveTo(sx, sy); g.lineTo(sx + len * .9 * dir, sy + len * (.35 + hash(k + 12) * .3)); g.stroke();
    }
  }
  function build(w, h) {
    const c = layer(w, h), g = c.getContext('2d'), s = Math.min(w, h);
    paper(g, w, h);
    if (showSun()) {
      const sx = w * .72, sy = h * .24, r = s * .075, halo = g.createRadialGradient(sx, sy, r * .8, sx, sy, r * 3);
      halo.addColorStop(0, 'rgba(190,60,48,.16)'); halo.addColorStop(1, 'rgba(190,60,48,0)'); g.fillStyle = halo; g.fillRect(sx - r * 3, sy - r * 3, r * 6, r * 6);
      const disc = g.createRadialGradient(sx - r * .2, sy - r * .25, r * .1, sx, sy, r); disc.addColorStop(0, '#d2503f'); disc.addColorStop(.85, '#bd3a2e'); disc.addColorStop(1, 'rgba(189,58,46,.88)');
      g.fillStyle = disc; g.beginPath(); g.arc(sx, sy, r, 0, TAU); g.fill();
    }
    for (let i = 0; i < 5; i++) mountain(g, w, h, i, 5);
    const water = g.createLinearGradient(0, h * .8, 0, h); water.addColorStop(0, `rgba(${PAPER},0)`); water.addColorStop(.35, `rgba(${PAPER},.86)`); water.addColorStop(1, `rgba(${PAPER},.94)`);
    g.fillStyle = water; g.fillRect(0, h * .78, w, h * .22);
    g.lineCap = 'round';
    for (let i = 0; i < 70; i++) {
      const y = h * (.82 + hash(i) * .17), x = hash(i + 100) * w, l = w * (.03 + hash(i + 200) * .09);
      g.strokeStyle = `rgba(${INK},${.04 + hash(i + 300) * .07})`; g.lineWidth = Math.max(1, w / 1500) * (.6 + hash(i + 400)); g.beginPath(); g.moveTo(x, y); g.lineTo(x + l, y); g.stroke();
    }
    const seal = s * .045, qx = w * .9, qy = h * .9; g.save(); g.translate(qx, qy); g.rotate(-.04);
    g.fillStyle = 'rgba(176,48,40,.9)'; g.fillRect(-seal / 2, -seal / 2, seal, seal);
    g.fillStyle = `rgba(${PAPER},.95)`; g.font = `700 ${seal * .42}px "Songti SC","STSong","Noto Serif CJK SC","SimSun",serif`; g.textAlign = 'center'; g.textBaseline = 'middle';
    g.fillText('小', 0, -seal * .22); g.fillText('伴', 0, seal * .22); g.restore();
    const vignette = g.createRadialGradient(w / 2, h / 2, s * .5, w / 2, h / 2, Math.max(w, h) * .8);
    vignette.addColorStop(0, 'rgba(90,70,40,0)'); vignette.addColorStop(1, 'rgba(90,70,40,.2)'); g.fillStyle = vignette; g.fillRect(0, 0, w, h);
    return c;
  }
  function mistSprite() {
    const c = layer(1400, 200), g = c.getContext('2d');
    for (let i = 0; i < 16; i++) {
      const x = hash(i * 3 + 1) * 1400, y = 70 + hash(i * 3 + 2) * 70, rx = 150 + hash(i * 3 + 3) * 200, ry = 24 + hash(i + 40) * 30;
      for (const off of [-1400, 0, 1400]) {
        g.save(); g.translate(x + off, y); g.scale(rx, ry); const r = g.createRadialGradient(0, 0, 0, 0, 0, 1);
        r.addColorStop(0, `rgba(${PAPER},.7)`); r.addColorStop(.6, `rgba(${PAPER},.3)`); r.addColorStop(1, `rgba(${PAPER},0)`); g.fillStyle = r; g.beginPath(); g.arc(0, 0, 1, 0, TAU); g.fill(); g.restore();
      }
    }
    return c;
  }
  function life(w, h, t) {
    const s = w / 1600;
    ctx.strokeStyle = `rgba(${INK},.7)`; ctx.lineCap = 'round'; ctx.lineWidth = Math.max(1.2, 1.6 * s);
    for (let i = 0; i < 7; i++) {
      const lead = Math.abs(i - 3), x = ((.1 + t * .008) % 1.2 - .1) * w - lead * 26 * s, y = h * (.2 + .02 * Math.sin(t * .4)) + lead * 14 * s, size = (10 - lead * .6) * s, flap = Math.sin(t * 2.6 + i * .7) * size * .5;
      ctx.beginPath(); ctx.moveTo(x - size, y - flap); ctx.quadraticCurveTo(x - size * .4, y - size * .55, x, y); ctx.quadraticCurveTo(x + size * .4, y - size * .55, x + size, y - flap); ctx.stroke();
    }
    const bx = w * .42 + Math.sin(t * .15) * w * .01, by = h * .9 + Math.sin(t * .7) * h * .002, bs = w / 1600;
    ctx.fillStyle = `rgba(${INK},.88)`; ctx.beginPath(); ctx.moveTo(bx - 46 * bs, by); ctx.quadraticCurveTo(bx, by + 12 * bs, bx + 46 * bs, by - 2 * bs); ctx.quadraticCurveTo(bx, by + 3 * bs, bx - 46 * bs, by); ctx.fill();
    ctx.beginPath(); ctx.moveTo(bx - 4 * bs, by - 1 * bs); ctx.lineTo(bx - 6 * bs, by - 16 * bs); ctx.lineTo(bx + 4 * bs, by - 16 * bs); ctx.lineTo(bx + 4 * bs, by - 1 * bs); ctx.fill();
    ctx.beginPath(); ctx.arc(bx - 1 * bs, by - 19 * bs, 3.5 * bs, 0, TAU); ctx.fill();
    ctx.strokeStyle = `rgba(${INK},.55)`; ctx.lineWidth = Math.max(1, 1.2 * bs); ctx.beginPath(); ctx.moveTo(bx + 5 * bs, by - 14 * bs); ctx.lineTo(bx + 60 * bs, by - 30 * bs); ctx.stroke();
    for (let k = 0; k < 3; k++) { const ph = (t * .25 + k / 3) % 1; ctx.strokeStyle = `rgba(${INK},${.18 * (1 - ph)})`; ctx.lineWidth = Math.max(1, bs); ctx.beginPath(); ctx.ellipse(bx, by + 5 * bs, (50 + ph * 70) * bs, (4 + ph * 8) * bs, 0, 0, TAU); ctx.stroke(); }
  }
  return {
    update(next) { values = next; },
    render({ time }) {
      const w = canvas.width, h = canvas.height, t = time * Number(values.speed ?? .6), key = `${w}x${h}|${showSun()}`;
      if (!bg || key !== sig) { bg = build(w, h); mist = mistSprite(); sig = key; pattern = ctx.createPattern(grainTile(192, 14), 'repeat'); }
      ctx.drawImage(bg, 0, 0);
      for (const [i, y, a, spd] of [[0, .46, .55, .006], [1, .6, .6, .01], [2, .73, .5, .015]]) {
        const sw = w * 1.1, off = -((t * spd * w + i * 300) % sw); ctx.globalAlpha = a;
        for (let k = 0; k < 3; k++) ctx.drawImage(mist, off + k * sw, h * y - h * .09, sw, h * .18);
      }
      ctx.globalAlpha = 1; life(w, h, t);
      ctx.fillStyle = pattern; ctx.fillRect(0, 0, w, h);
    },
    resize() { bg = null; },
    destroy() { bg = null; mist = null; pattern = null; }
  };
}
