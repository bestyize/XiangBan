// 枫林暖阳：低角度的阳光穿过层层枫林，光束里漂着细小的浮尘。自包含 Canvas 2D 场景。
const TAU = Math.PI * 2;
const hash = n => { const v = Math.sin(n * 127.1 + 311.7) * 43758.5453; return v - Math.floor(v); };
const mix = (a, b, t) => a + (b - a) * t;
const smooth = t => t * t * (3 - 2 * t);
const clamp = (v, a = 0, b = 1) => Math.min(b, Math.max(a, v));
const noise = x => { const i = Math.floor(x), f = x - i; return mix(hash(i), hash(i + 1), smooth(f)); };
const layer = (w, h) => { const c = document.createElement('canvas'); c.width = Math.max(1, Math.round(w)); c.height = Math.max(1, Math.round(h)); return c; };
function grainTile(size, amount) {
  const c = layer(size, size), g = c.getContext('2d'), img = g.createImageData(size, size), d = img.data;
  for (let i = 0; i < d.length; i += 4) { const v = Math.random() * 2 - 1; d[i] = d[i + 1] = d[i + 2] = v > 0 ? 255 : 0; d[i + 3] = Math.abs(v) * amount; }
  g.putImageData(img, 0, 0); return c;
}
const LEAVES = ['217,130,43', '232,163,61', '181,71,43', '140,59,36', '240,195,106', '204,102,40'];
const leafColor = (i, shade) => { const c = LEAVES[Math.floor(hash(i * 1.7 + 3) * LEAVES.length)].split(',').map(Number); return c.map(v => Math.round(v * shade)).join(','); };

export default function create({ canvas, settings = {} }) {
  const ctx = canvas.getContext('2d');
  if (!ctx) throw new Error('无法绘制枫林暖阳');
  let values = settings, bg = null, sig = '', pattern = null;
  const sun = (w, h) => ({ x: w * .56, y: h * .34 });

  function cluster(g, x, y, r, seed, shade, alpha) {
    const c = leafColor(seed, shade), blob = g.createRadialGradient(x, y, 0, x, y, r);
    blob.addColorStop(0, `rgba(${c},${alpha})`); blob.addColorStop(.62, `rgba(${c},${alpha * .8})`); blob.addColorStop(1, `rgba(${c},0)`);
    g.fillStyle = blob; g.fillRect(x - r, y - r, r * 2, r * 2);
  }
  function leaves(g, x, y, r, seed, shade, count, size) {
    for (let k = 0; k < count; k++) {
      const a = hash(seed + k * 2.3) * TAU, d = Math.sqrt(hash(seed + k * 3.1 + 7)) * r, lx = x + Math.cos(a) * d, ly = y + Math.sin(a) * d * .85;
      g.fillStyle = `rgba(${leafColor(seed + k, shade * (.82 + hash(k + seed) * .36))},${.55 + hash(k + seed + 4) * .4})`;
      g.save(); g.translate(lx, ly); g.rotate(hash(seed + k + 9) * TAU); g.beginPath(); g.ellipse(0, 0, size * (.7 + hash(k + seed + 2) * .7), size * .42, 0, 0, TAU); g.fill(); g.restore();
    }
  }
  function trunk(g, x, w0, h, lean, seed) {
    g.beginPath(); g.moveTo(x - w0 / 2, h);
    for (let y = h, i = 0; y >= -10; y -= h * .04, i++) g.lineTo(x - (w0 / 2) * (1 - (h - y) / h * .45) + lean * (h - y) / h + (noise(i * .7 + seed) - .5) * w0 * .08, y);
    for (let y = -10, i = 0; y <= h; y += h * .04, i++) g.lineTo(x + (w0 / 2) * (1 - (h - y) / h * .45) + lean * (h - y) / h + (noise(i * .7 + seed + 40) - .5) * w0 * .08, y);
    g.closePath(); const bark = g.createLinearGradient(x - w0, 0, x + w0, 0); bark.addColorStop(0, '#1c100a'); bark.addColorStop(.55, '#3a2316'); bark.addColorStop(1, '#150b07');
    g.fillStyle = bark; g.fill();
  }
  function build(w, h) {
    const c = layer(w, h), g = c.getContext('2d'), s = sun(w, h), m = Math.min(w, h), hor = h * .66;
    const sky = g.createLinearGradient(0, 0, 0, hor); sky.addColorStop(0, '#e9964a'); sky.addColorStop(.5, '#f7c27a'); sky.addColorStop(1, '#ffe7b4'); g.fillStyle = sky; g.fillRect(0, 0, w, h);
    const halo = g.createRadialGradient(s.x, s.y, 0, s.x, s.y, Math.max(w, h) * .6); halo.addColorStop(0, 'rgba(255,248,214,.95)'); halo.addColorStop(.1, 'rgba(255,230,160,.6)'); halo.addColorStop(1, 'rgba(255,200,110,0)'); g.fillStyle = halo; g.fillRect(0, 0, w, h);
    const nearSun = (x, y, r) => Math.hypot(x - s.x, (y - s.y) * .8) < m * .32 + r * .4;
    for (let i = 0; i < 70; i++) { const x = hash(i) * w, y = h * (.34 + hash(i + 1) * .26), r = m * (.07 + hash(i + 2) * .08); if (!nearSun(x, y, r)) cluster(g, x, y, r, i + 11, 1, .42); }
    g.fillStyle = 'rgba(255,225,170,.38)'; g.fillRect(0, 0, w, h);
    for (let i = 0; i < 60; i++) { const x = hash(i + 300) * w; if (Math.abs(x - s.x) < w * .2) continue; g.fillStyle = 'rgba(92,52,30,.28)'; g.fillRect(x, h * (.42 + hash(i + 301) * .1), Math.max(1, w * .0016), h * .3); }
    for (let i = 0; i < 80; i++) { const x = hash(i + 500) * w, y = h * (.3 + hash(i + 501) * .35), r = m * (.09 + hash(i + 502) * .1); if (nearSun(x, y, r)) continue; cluster(g, x, y, r, i + 41, .96, .4); leaves(g, x, y, m * .1, i * 13, .95, 26, m * .007); }
    const glow = g.createRadialGradient(s.x, s.y, 0, s.x, s.y, m * .5); glow.addColorStop(0, 'rgba(255,250,225,1)'); glow.addColorStop(.12, 'rgba(255,238,180,.8)'); glow.addColorStop(.5, 'rgba(255,214,130,.25)'); glow.addColorStop(1, 'rgba(255,214,130,0)'); g.fillStyle = glow; g.fillRect(0, 0, w, h);
    g.fillStyle = 'rgba(255,253,240,.96)'; g.beginPath(); g.arc(s.x, s.y, m * .03, 0, TAU); g.fill();
    const haze = g.createLinearGradient(0, h * .56, 0, h * .72); haze.addColorStop(0, 'rgba(255,224,160,0)'); haze.addColorStop(.6, 'rgba(255,214,140,.5)'); haze.addColorStop(1, 'rgba(255,214,140,0)'); g.fillStyle = haze; g.fillRect(0, h * .56, w, h * .16);
    for (let i = 0; i < 12; i++) { const x = hash(i + 700) * w; if (Math.abs(x - s.x) < w * .1) continue; trunk(g, x, w * (.012 + hash(i + 701) * .01), h * .72, (hash(i + 702) - .5) * w * .02, i); }
    const ground = g.createLinearGradient(0, h * .66, 0, h); ground.addColorStop(0, '#d49a52'); ground.addColorStop(.07, '#a8662c'); ground.addColorStop(.32, '#5d3219'); ground.addColorStop(1, '#241108'); g.fillStyle = ground; g.fillRect(0, h * .66, w, h * .34);
    const patch = g.createRadialGradient(s.x, h * .8, 0, s.x, h * .8, w * .35); patch.addColorStop(0, 'rgba(255,214,130,.32)'); patch.addColorStop(1, 'rgba(255,214,130,0)'); g.save(); g.translate(0, h * .8); g.scale(1, .3); g.translate(0, -h * .8); g.fillStyle = patch; g.fillRect(0, h * .2, w, h * 1.6); g.restore();
    for (let i = 0; i < 1500; i++) {
      const u = Math.pow(hash(i + 900), .8), y = h * .68 + u * h * .32, x = hash(i + 901) * w, sz = m * (.004 + u * .011);
      g.fillStyle = `rgba(${leafColor(i, .55 + u * .45)},${.5 + hash(i + 902) * .45})`; g.save(); g.translate(x, y); g.rotate((hash(i + 903) - .5) * 2.4); g.beginPath(); g.ellipse(0, 0, sz, sz * .38, 0, 0, TAU); g.fill(); g.restore();
    }
    for (const side of [0, 1]) {
      const cx = side ? w * .94 : w * .06; trunk(g, cx, w * .06, h * 1.02, side ? -w * .02 : w * .02, side * 9 + 3);
    }
    const spots = [[.02, .0, .2], [.14, -.02, .17], [.26, .01, .15], [.36, -.02, .11], [.66, -.02, .11], [.78, .01, .15], [.9, -.02, .18], [1.0, .03, .2], [.0, .22, .12], [1.0, .24, .12]];
    spots.forEach(([fx, fy, fr], i) => {
      const x = fx * w, y = fy * h, r = fr * m * 1.25;
      cluster(g, x, y, r, i * 5 + 100, .6, .96); cluster(g, x + r * .15, y + r * .1, r * .8, i * 5 + 101, .78, .9); leaves(g, x, y, r * .95, i * 17 + 5, .85, 520, m * .0085);
    });
    const vignette = g.createRadialGradient(w / 2, h * .5, m * .5, w / 2, h * .5, Math.max(w, h) * .78); vignette.addColorStop(0, 'rgba(40,16,4,0)'); vignette.addColorStop(1, 'rgba(40,16,4,.4)'); g.fillStyle = vignette; g.fillRect(0, 0, w, h);
    return c;
  }
  function rays(w, h, t) {
    const s = sun(w, h), strength = clamp(Number(values.rays ?? .6), 0, 1), len = Math.hypot(w, h) * 1.15;
    ctx.globalCompositeOperation = 'lighter';
    for (let i = 0; i < 9; i++) {
      const a = Math.PI / 2 + (i - 4) * .14 + Math.sin(t * .09 + i * 1.7) * .025, half = .02 + hash(i + 3) * .025, pulse = .5 + .5 * Math.sin(t * .35 + i * 2.1);
      const ox = s.x, oy = s.y;
      const g = ctx.createLinearGradient(ox, oy, ox + Math.cos(a) * len, oy + Math.sin(a) * len);
      g.addColorStop(0, 'rgba(255,236,170,0)'); g.addColorStop(.1, `rgba(255,236,170,${.2 * strength * pulse})`); g.addColorStop(.5, `rgba(255,214,130,${.08 * strength * pulse})`); g.addColorStop(1, 'rgba(255,214,130,0)');
      ctx.fillStyle = g;
      for (const k of [1, 1.9, 3]) {
        ctx.globalAlpha = k === 1 ? 1 : k === 1.9 ? .55 : .3;
        ctx.beginPath(); ctx.moveTo(ox, oy); ctx.lineTo(ox + Math.cos(a - half * k) * len, oy + Math.sin(a - half * k) * len); ctx.lineTo(ox + Math.cos(a + half * k) * len, oy + Math.sin(a + half * k) * len); ctx.closePath(); ctx.fill();
      }
      ctx.globalAlpha = 1;
    }
    const m = Math.min(w, h);
    for (let i = 0; i < 80; i++) {
      const x = ((hash(i) * w + Math.sin(t * .2 + i) * m * .03 + t * m * .004 * (.4 + hash(i + 3))) % w + w) % w, y = ((hash(i + 1) * h * .8 + h * .1 - t * m * .003 * (.3 + hash(i + 4))) % (h * .8) + h * .8) % (h * .8) + h * .1;
      const near = clamp(1 - Math.hypot(x - s.x, (y - s.y) * .7) / (w * .55)), a = (.1 + .5 * Math.max(0, Math.sin(t * (.6 + hash(i + 6)) + i * 3))) * (.3 + near) * strength;
      ctx.fillStyle = `rgba(255,236,190,${a})`; ctx.beginPath(); ctx.arc(x, y, m * (.0012 + hash(i + 7) * .0022), 0, TAU); ctx.fill();
    }
    ctx.globalCompositeOperation = 'source-over';
  }
  return {
    update(next) { values = next; },
    render({ time }) {
      const w = canvas.width, h = canvas.height, t = time * Number(values.speed ?? .6), key = `${w}x${h}`;
      if (!bg || key !== sig) { bg = build(w, h); sig = key; pattern = ctx.createPattern(grainTile(192, 20), 'repeat'); }
      ctx.drawImage(bg, 0, 0); rays(w, h, t);
      ctx.fillStyle = pattern; ctx.fillRect(0, 0, w, h);
    },
    resize() { bg = null; },
    destroy() { bg = null; pattern = null; }
  };
}
