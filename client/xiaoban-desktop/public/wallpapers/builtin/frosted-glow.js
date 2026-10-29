// 糖霜微光：缓慢游走的弥散光斑，叠着几颗磨砂玻璃球和细小颗粒。自包含 Canvas 2D 场景。
const TAU = Math.PI * 2;
const hash = n => { const v = Math.sin(n * 127.1 + 311.7) * 43758.5453; return v - Math.floor(v); };
const mix = (a, b, t) => a + (b - a) * t;
const clamp = (v, a = 0, b = 1) => Math.min(b, Math.max(a, v));
const layer = (w, h) => { const c = document.createElement('canvas'); c.width = Math.max(1, Math.round(w)); c.height = Math.max(1, Math.round(h)); return c; };
const hex = (v, d) => /^#[0-9a-f]{6}$/i.test(String(v)) ? String(v) : d;
const rgb = h => [1, 3, 5].map(i => parseInt(h.slice(i, i + 2), 16));
const blend = (a, b, t) => a.map((v, i) => Math.round(mix(v, b[i], t)));
function grainTile(size, amount) {
  const c = layer(size, size), g = c.getContext('2d'), img = g.createImageData(size, size), d = img.data;
  for (let i = 0; i < d.length; i += 4) { const v = Math.random() * 2 - 1; d[i] = d[i + 1] = d[i + 2] = v > 0 ? 255 : 0; d[i + 3] = Math.abs(v) * amount; }
  g.putImageData(img, 0, 0); return c;
}

export default function create({ canvas, settings = {} }) {
  const ctx = canvas.getContext('2d');
  if (!ctx) throw new Error('无法绘制糖霜微光');
  let values = settings, pattern = null, tileKey = '';
  const dark = () => values.dark === true;
  const palette = () => {
    const a = rgb(hex(values.accent, '#ff9fb4')), b = rgb(hex(values.second, '#a99bff')), white = [255, 255, 255], warm = [255, 214, 170], cool = [150, 225, 255];
    const lift = c => blend(c, white, dark() ? .0 : .15);
    return { base: dark() ? '#1b1740' : '#fff4ee', blobs: [a, b, blend(a, b, .5), blend(a, warm, .55), blend(b, cool, .5), warm].map(lift) };
  };
  const blobs = Array.from({ length: 7 }, (_, i) => ({ cx: .15 + hash(i * 5 + 1) * .7, cy: .15 + hash(i * 5 + 2) * .7, ax: .1 + hash(i * 5 + 3) * .16, ay: .08 + hash(i * 5 + 4) * .14, fx: .05 + hash(i * 5 + 5) * .08, fy: .04 + hash(i * 5 + 6) * .08, ph: hash(i + 90) * TAU, r: .32 + hash(i + 120) * .26 }));
  const orbs = Array.from({ length: 5 }, (_, i) => ({ cx: .12 + hash(i * 7 + 11) * .76, cy: .18 + hash(i * 7 + 12) * .64, ax: .02 + hash(i * 7 + 13) * .04, ay: .03 + hash(i * 7 + 14) * .05, f: .08 + hash(i * 7 + 15) * .1, ph: hash(i + 200) * TAU, r: .045 + hash(i + 210) * .075 }));

  // 弥散光本来就是柔焦的，在 1/4 分辨率的离屏层里绘制再放大，开销只有全分辨率的十六分之一。
  let low = null;
  function glow(w, h, t, p) {
    const lw = Math.max(2, Math.round(w / 4)), lh = Math.max(2, Math.round(h / 4));
    if (!low || low.width !== lw || low.height !== lh) low = layer(lw, lh);
    const c2 = low.getContext('2d'), m = Math.max(lw, lh), strong = dark() ? .8 : .62;
    c2.fillStyle = p.base; c2.fillRect(0, 0, lw, lh);
    blobs.forEach((b, i) => {
      const x = (b.cx + b.ax * Math.sin(t * b.fx * TAU + b.ph)) * lw, y = (b.cy + b.ay * Math.cos(t * b.fy * TAU + b.ph * 1.3)) * lh, r = b.r * m, c = p.blobs[i % p.blobs.length].join(',');
      const g = c2.createRadialGradient(x, y, 0, x, y, r); g.addColorStop(0, `rgba(${c},${strong})`); g.addColorStop(.55, `rgba(${c},${strong * .38})`); g.addColorStop(1, `rgba(${c},0)`);
      c2.fillStyle = g; c2.fillRect(x - r, y - r, r * 2, r * 2);
    });
    const vignette = c2.createRadialGradient(lw / 2, lh / 2, Math.min(lw, lh) * .5, lw / 2, lh / 2, m * .78);
    vignette.addColorStop(0, 'rgba(255,255,255,0)'); vignette.addColorStop(1, dark() ? 'rgba(5,0,30,.35)' : 'rgba(255,240,235,.28)');
    c2.fillStyle = vignette; c2.fillRect(0, 0, lw, lh);
    ctx.imageSmoothingEnabled = true; ctx.imageSmoothingQuality = 'high'; ctx.drawImage(low, 0, 0, w, h);
  }
  function orb(x, y, r, c, t, i) {
    const col = c.join(','), s = dark() ? 1.25 : 1;
    const shadow = ctx.createRadialGradient(x + r * .25, y + r * .45, r * .5, x + r * .25, y + r * .45, r * 1.5);
    shadow.addColorStop(0, `rgba(${dark() ? '0,0,20' : '120,80,110'},${dark() ? .28 : .13})`); shadow.addColorStop(1, 'rgba(0,0,0,0)');
    ctx.fillStyle = shadow; ctx.fillRect(x - r * 1.6, y - r * 1.4, r * 3.2, r * 3.4);
    const body = ctx.createRadialGradient(x - r * .35, y - r * .4, r * .05, x, y, r);
    body.addColorStop(0, `rgba(255,255,255,${.7 * s})`); body.addColorStop(.35, `rgba(255,255,255,${.22 * s})`); body.addColorStop(.8, `rgba(${col},.2)`); body.addColorStop(1, `rgba(255,255,255,${.34 * s})`);
    ctx.fillStyle = body; ctx.beginPath(); ctx.arc(x, y, r, 0, TAU); ctx.fill();
    const rim = ctx.createLinearGradient(x - r, y - r, x + r, y + r); rim.addColorStop(0, 'rgba(255,255,255,.85)'); rim.addColorStop(.5, 'rgba(255,255,255,.05)'); rim.addColorStop(1, 'rgba(255,255,255,.5)');
    ctx.strokeStyle = rim; ctx.lineWidth = Math.max(1, r * .028); ctx.beginPath(); ctx.arc(x, y, r * .985, 0, TAU); ctx.stroke();
    const caustic = ctx.createRadialGradient(x + r * .1, y + r * .72, 0, x + r * .1, y + r * .72, r * .55);
    caustic.addColorStop(0, `rgba(255,255,255,${.38 * s})`); caustic.addColorStop(1, 'rgba(255,255,255,0)');
    ctx.save(); ctx.beginPath(); ctx.arc(x, y, r, 0, TAU); ctx.clip(); ctx.fillStyle = caustic; ctx.fillRect(x - r, y, r * 2, r); ctx.restore();
    ctx.save(); ctx.translate(x - r * .38, y - r * .45); ctx.rotate(-.7); ctx.scale(r * .26, r * .13);
    const spec = ctx.createRadialGradient(0, 0, 0, 0, 0, 1); spec.addColorStop(0, 'rgba(255,255,255,.95)'); spec.addColorStop(1, 'rgba(255,255,255,0)');
    ctx.fillStyle = spec; ctx.beginPath(); ctx.arc(0, 0, 1, 0, TAU); ctx.fill(); ctx.restore();
  }
  function rings(w, h, t) {
    ctx.strokeStyle = dark() ? 'rgba(255,255,255,.07)' : 'rgba(255,255,255,.5)'; ctx.lineWidth = Math.max(1, w / 1500);
    for (let i = 0; i < 3; i++) { ctx.beginPath(); ctx.arc(w * (.5 + .06 * Math.sin(t * .03 + i)), h * .5, Math.min(w, h) * (.3 + i * .16), 0, TAU); ctx.stroke(); }
  }
  return {
    update(next) { values = next; },
    render({ time }) {
      const w = canvas.width, h = canvas.height, t = time * Number(values.speed ?? .6), p = palette(), m = Math.min(w, h);
      const key = `${w}x${h}|${dark()}`;
      if (!pattern || key !== tileKey) { pattern = ctx.createPattern(grainTile(192, dark() ? 20 : 22), 'repeat'); tileKey = key; }
      glow(w, h, t, p); rings(w, h, t);
      orbs.forEach((o, i) => orb((o.cx + o.ax * Math.sin(t * o.f * TAU + o.ph)) * w, (o.cy + o.ay * Math.cos(t * o.f * TAU * .8 + o.ph)) * h, o.r * m, p.blobs[(i + 1) % p.blobs.length], t, i));
      ctx.fillStyle = pattern; ctx.fillRect(0, 0, w, h);
    },
    resize() { pattern = null; low = null; },
    destroy() { pattern = null; low = null; }
  };
}
