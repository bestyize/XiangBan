(() => {
  // Rain Window presets, physics and shaders from live-wallpapers-redesign.html.
  // ==================== Rain Window Presets ====================
  const PRESETS = [
    {
      id: 'dusk',
      name: '暮雨',
      desc: '暮色深蓝与暖金窗灯散景',
      swatch: 'linear-gradient(135deg, #27406f 0%, #f19a52 100%)',
      skyTop: [0.047, 0.086, 0.169],
      skyMid: [0.118, 0.176, 0.290],
      skyBot: [0.235, 0.227, 0.329],
      cloudCol: [0.36, 0.42, 0.56],
      bokehA: [1.0, 0.72, 0.42],
      bokehB: [1.0, 0.49, 0.37],
      bokehC: [0.48, 0.66, 1.0],
      mistTint: [0.22, 0.28, 0.38],
      rainAmt: 0.78,
      mistDensity: 0.74,
      snowMode: 0.0
    },
    {
      id: 'morning',
      name: '晴岚',
      desc: '清透晨曦雾蓝与柔白微光',
      swatch: 'linear-gradient(135deg, #6491b4 0%, #e9d2c4 100%)',
      skyTop: [0.290, 0.451, 0.592],
      skyMid: [0.545, 0.694, 0.788],
      skyBot: [0.875, 0.808, 0.761],
      cloudCol: [0.96, 0.94, 0.92],
      bokehA: [1.0, 0.93, 0.82],
      bokehB: [1.0, 0.81, 0.66],
      bokehC: [0.76, 0.90, 1.0],
      mistTint: [0.76, 0.82, 0.86],
      rainAmt: 0.46,
      mistDensity: 0.58,
      snowMode: 0.0
    },
    {
      id: 'moss',
      name: '梅雨',
      desc: '江南烟青与温润凝露水痕',
      swatch: 'linear-gradient(135deg, #243b3c 0%, #7d9b8e 100%)',
      skyTop: [0.118, 0.192, 0.196],
      skyMid: [0.235, 0.345, 0.337],
      skyBot: [0.435, 0.525, 0.490],
      cloudCol: [0.64, 0.73, 0.69],
      bokehA: [0.95, 0.92, 0.80],
      bokehB: [0.68, 0.86, 0.79],
      bokehC: [0.85, 0.90, 0.88],
      mistTint: [0.42, 0.51, 0.49],
      rainAmt: 0.90,
      mistDensity: 0.82,
      snowMode: 0.0
    },
    {
      id: 'neon',
      name: '霓虹夜',
      desc: '午夜墨蓝与电影感珊瑚光斑',
      swatch: 'linear-gradient(135deg, #0d1633 0%, #ff4d6d 100%)',
      skyTop: [0.027, 0.043, 0.094],
      skyMid: [0.067, 0.102, 0.200],
      skyBot: [0.110, 0.114, 0.216],
      cloudCol: [0.24, 0.26, 0.45],
      bokehA: [1.0, 0.30, 0.43],
      bokehB: [1.0, 0.62, 0.27],
      bokehC: [0.22, 0.74, 0.97],
      mistTint: [0.16, 0.20, 0.32],
      rainAmt: 0.95,
      mistDensity: 0.72,
      snowMode: 0.0
    },
    {
      id: 'snow',
      name: '微雪',
      desc: '冬夜暮蓝与窗外轻盈飘雪',
      swatch: 'linear-gradient(135deg, #1b283f 0%, #ffd28a 100%)',
      skyTop: [0.090, 0.129, 0.200],
      skyMid: [0.161, 0.224, 0.314],
      skyBot: [0.290, 0.353, 0.439],
      cloudCol: [0.68, 0.74, 0.84],
      bokehA: [1.0, 0.84, 0.56],
      bokehB: [1.0, 0.92, 0.75],
      bokehC: [0.70, 0.84, 1.0],
      mistTint: [0.48, 0.56, 0.66],
      rainAmt: 0.54,
      mistDensity: 0.78,
      snowMode: 1.0
    }
  ];

  const PRESET_MAP = new Map(PRESETS.map(p => [p.id, p]));
  const getPreset = id => PRESET_MAP.get(id) || PRESETS[0];
  const clamp = value => Math.min(1, Math.max(0, value));

  // ==================== Physics & Mist Simulation State ====================
  function createSimState(presetId, seedOffset = 0, preWiped = false) {
    const p = getPreset(presetId);
    const mistCanvas = document.createElement('canvas');
    mistCanvas.width = 160;
    mistCanvas.height = 284;
    const mistCtx = mistCanvas.getContext('2d');
    const fillAlpha = p.mistDensity;
    mistCtx.fillStyle = `rgba(255,255,255,${fillAlpha})`;
    mistCtx.fillRect(0, 0, mistCanvas.width, mistCanvas.height);

    // Carve an inviting initial streak or soft wiped heart-area if requested
    if (preWiped) {
      mistCtx.save();
      mistCtx.globalCompositeOperation = 'destination-out';
      const g = mistCtx.createRadialGradient(84, 128, 6, 84, 128, 52);
      g.addColorStop(0, 'rgba(0,0,0,0.88)');
      g.addColorStop(0.55, 'rgba(0,0,0,0.45)');
      g.addColorStop(1, 'rgba(0,0,0,0)');
      mistCtx.fillStyle = g;
      mistCtx.beginPath();
      mistCtx.arc(84, 128, 52, 0, Math.PI * 2);
      mistCtx.fill();
      mistCtx.restore();
    }

    let s = (1337 + seedOffset * 97) >>> 0;
    const rng = () => {
      s = (Math.imul(s, 1664525) + 1013904223) >>> 0;
      return s / 4294967296;
    };

    const drops = Array.from({ length: 24 }, (_, i) => ({
      x: 0.06 + rng() * 0.88,
      y: rng() * 0.96,
      r: 0.011 + rng() * 0.014,
      vy: 0.018 + rng() * 0.048,
      wobble: rng() * Math.PI * 2,
      stall: rng() * 2.0,
      active: i < 18
    }));

    return {
      presetId: p.id,
      current: clonePresetParams(p),
      target: clonePresetParams(p),
      mistCanvas,
      mistCtx,
      mistDirty: true,
      drops,
      ripples: [],
      elapsed: seedOffset * 3.7,
      recondenseAcc: 0,
      still: false,
      rng
    };
  }

  function clonePresetParams(p) {
    return {
      skyTop: [...p.skyTop],
      skyMid: [...p.skyMid],
      skyBot: [...p.skyBot],
      cloudCol: [...p.cloudCol],
      bokehA: [...p.bokehA],
      bokehB: [...p.bokehB],
      bokehC: [...p.bokehC],
      mistTint: [...p.mistTint],
      rainAmt: p.rainAmt,
      mistDensity: p.mistDensity,
      snowMode: p.snowMode
    };
  }

  function setSimPreset(sim, presetId) {
    const p = PRESET_MAP.get(presetId);
    if (!p || sim.presetId === p.id) return;
    sim.presetId = presetId;
    sim.target = clonePresetParams(p);
  }

  function resetSimMist(sim) {
    const { mistCanvas, mistCtx } = sim;
    mistCtx.globalCompositeOperation = 'source-over';
    mistCtx.clearRect(0, 0, mistCanvas.width, mistCanvas.height);
    mistCtx.fillStyle = `rgba(255,255,255,${sim.target.mistDensity})`;
    mistCtx.fillRect(0, 0, mistCanvas.width, mistCanvas.height);
    sim.mistDirty = true;
  }

  function wipeSimMist(sim, nx, ny, prevNx = nx, prevNy = ny) {
    if (![nx, ny, prevNx, prevNy].every(Number.isFinite)) return;
    nx = clamp(nx);
    ny = clamp(ny);
    prevNx = clamp(prevNx);
    prevNy = clamp(prevNy);
    const { mistCanvas, mistCtx } = sim;
    const w = mistCanvas.width;
    const h = mistCanvas.height;
    const x0 = prevNx * w;
    const y0 = prevNy * h;
    const x1 = nx * w;
    const y1 = ny * h;
    const dist = Math.hypot(x1 - x0, y1 - y0);
    const steps = Math.max(1, Math.ceil(dist / 3));
    const brushR = 16;

    mistCtx.save();
    mistCtx.globalCompositeOperation = 'destination-out';
    for (let i = 0; i <= steps; i++) {
      const t = i / steps;
      const cx = x0 + (x1 - x0) * t;
      const cy = y0 + (y1 - y0) * t;
      const g = mistCtx.createRadialGradient(cx, cy, 1, cx, cy, brushR);
      g.addColorStop(0, 'rgba(0,0,0,0.82)');
      g.addColorStop(0.55, 'rgba(0,0,0,0.38)');
      g.addColorStop(1, 'rgba(0,0,0,0)');
      mistCtx.fillStyle = g;
      mistCtx.beginPath();
      mistCtx.arc(cx, cy, brushR, 0, Math.PI * 2);
      mistCtx.fill();
    }
    mistCtx.restore();
    sim.mistDirty = true;

    // Disturb nearby raindrops so they accelerate when touched
    for (const d of sim.drops) {
      if (!d.active) continue;
      if (Math.hypot(d.x - nx, d.y - ny) < 0.11) {
        d.vy = Math.max(d.vy, 0.08 + sim.rng() * 0.06);
        d.stall = 0;
      }
    }
  }

  function tapSimDrop(sim, nx, ny) {
    if (!Number.isFinite(nx) || !Number.isFinite(ny)) return;
    nx = clamp(nx);
    ny = clamp(ny);
    // Spawn or recycle a tactile large drop at tap position
    let slot = sim.drops.find(d => !d.active);
    if (!slot) {
      slot = sim.drops[Math.floor(sim.rng() * 6) + 18];
    }
    slot.x = Math.min(0.95, Math.max(0.05, nx));
    slot.y = Math.min(0.95, Math.max(0.05, ny));
    slot.r = 0.021 + sim.rng() * 0.009;
    slot.vy = 0.025;
    slot.stall = 0.18;
    slot.active = true;

    // Carve small clear droplet spot in mist
    wipeSimMist(sim, nx, ny, nx, ny);
    sim.ripples.push({ x: nx, y: ny, age: 0 });
    if (sim.ripples.length > 4) sim.ripples.shift();
  }

  function stepSim(sim, dt) {
    sim.elapsed += dt;
    const c = sim.current;
    const t = sim.target;
    const k = Math.min(1, dt * 4.5);
    for (const key of ['skyTop', 'skyMid', 'skyBot', 'cloudCol', 'bokehA', 'bokehB', 'bokehC', 'mistTint']) {
      for (let i = 0; i < 3; i++) c[key][i] += (t[key][i] - c[key][i]) * k;
    }
    c.rainAmt += (t.rainAmt - c.rainAmt) * k;
    c.mistDensity += (t.mistDensity - c.mistDensity) * k;
    c.snowMode += (t.snowMode - c.snowMode) * k;

    const { mistCanvas, mistCtx } = sim;
    const mw = mistCanvas.width;
    const mh = mistCanvas.height;

    // Very subtle re-condensation so wiped areas softly heal over ~45 seconds
    sim.recondenseAcc += dt;
    if (sim.recondenseAcc > 0.25) {
      const addAlpha = Math.min(0.02, sim.recondenseAcc * 0.011 * c.mistDensity);
      mistCtx.save();
      mistCtx.globalCompositeOperation = 'source-over';
      mistCtx.fillStyle = `rgba(255,255,255,${addAlpha.toFixed(4)})`;
      mistCtx.fillRect(0, 0, mw, mh);
      mistCtx.restore();
      sim.recondenseAcc = 0;
      sim.mistDirty = true;
    }

    // Update physical drops with stick-slip & mist trail carving
    mistCtx.save();
    mistCtx.globalCompositeOperation = 'destination-out';
    const speedScale = 0.55 + c.rainAmt * 0.75;
    for (const d of sim.drops) {
      if (!d.active) continue;
      if (d.stall > 0) {
        d.stall -= dt;
        continue;
      }
      // Occasional surface-tension pause or sudden release
      if (sim.rng() < dt * 0.35) {
        d.stall = 0.25 + sim.rng() * 0.75;
        d.vy = 0.014 + sim.rng() * 0.03;
      } else {
        d.vy = Math.min(0.16, d.vy + dt * (0.028 * (d.r / 0.016)) * speedScale);
      }
      d.x += Math.sin(sim.elapsed * 2.1 + d.wobble) * dt * 0.008;
      d.y += d.vy * dt * speedScale;

      // Carve slim clear wake behind moving drop
      const px = d.x * mw;
      const py = d.y * mh;
      const pr = Math.max(1.4, d.r * mw * 0.55);
      const wg = mistCtx.createRadialGradient(px, py, 0, px, py, pr * 1.45);
      wg.addColorStop(0, 'rgba(0,0,0,0.65)');
      wg.addColorStop(1, 'rgba(0,0,0,0)');
      mistCtx.fillStyle = wg;
      mistCtx.beginPath();
      mistCtx.arc(px, py, pr * 1.45, 0, Math.PI * 2);
      mistCtx.fill();
      sim.mistDirty = true;

      if (d.y > 1.08) {
        d.y = -0.04 - sim.rng() * 0.12;
        d.x = 0.06 + sim.rng() * 0.88;
        d.r = 0.011 + sim.rng() * 0.014;
        d.vy = 0.02 + sim.rng() * 0.035;
        d.stall = sim.rng() * 1.4;
      }
    }
    mistCtx.restore();

    for (let i = sim.ripples.length - 1; i >= 0; i--) {
      sim.ripples[i].age += dt;
      if (sim.ripples[i].age > 0.9) sim.ripples.splice(i, 1);
    }
  }

  const VS = `
    attribute vec2 aPos;
    varying mediump vec2 vUv;
    void main() {
      vUv = aPos * 0.5 + 0.5;
      gl_Position = vec4(aPos, 0.0, 1.0);
    }
  `;

  const FS_SCENE = `
    #ifdef GL_FRAGMENT_PRECISION_HIGH
    precision highp float;
    #else
    precision mediump float;
    #endif
    varying mediump vec2 vUv;
    uniform float uTime;
    uniform float uAspect;
    uniform vec3 uSkyTop;
    uniform vec3 uSkyMid;
    uniform vec3 uSkyBot;
    uniform vec3 uCloudCol;
    uniform vec3 uBokehA;
    uniform vec3 uBokehB;
    uniform vec3 uBokehC;
    uniform float uRainAmt;
    uniform float uSnowMode;

    float inverseSmoothstep(float high, float low, float x) {
      return 1.0 - smoothstep(low, high, x);
    }

    float hash(vec2 p) {
      p = fract(p * vec2(123.34, 456.21));
      p += dot(p, p + 45.32);
      return fract(p.x * p.y);
    }

    float noise(vec2 p) {
      vec2 i = floor(p);
      vec2 f = fract(p);
      vec2 u = f * f * (3.0 - 2.0 * f);
      float a = hash(i);
      float b = hash(i + vec2(1.0, 0.0));
      float c = hash(i + vec2(0.0, 1.0));
      float d = hash(i + vec2(1.0, 1.0));
      return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
    }

    float fbm(vec2 p) {
      float v = 0.0;
      float a = 0.5;
      for (int i = 0; i < 4; i++) {
        v += a * noise(p);
        p = p * 2.03 + vec2(17.1, 9.2);
        a *= 0.5;
      }
      return v;
    }

    void main() {
      vec2 uv = vUv;
      float y = uv.y;

      // Multi-stop vertical sky gradient
      vec3 sky = mix(uSkyBot, uSkyMid, smoothstep(0.0, 0.48, y));
      sky = mix(sky, uSkyTop, smoothstep(0.45, 1.0, y));

      // Soft drifting clouds in upper sky
      vec2 cloudUv = vec2(uv.x * uAspect * 1.4 + uTime * 0.015, y * 2.2 - uTime * 0.004);
      float c1 = fbm(cloudUv);
      float c2 = fbm(cloudUv * 1.8 - vec2(uTime * 0.022, 0.0));
      float cloudMask = smoothstep(0.42, 0.85, c1 * 0.65 + c2 * 0.35) * smoothstep(0.18, 0.86, y);
      sky = mix(sky, uCloudCol, cloudMask * 0.42);

      // Soft atmospheric horizon glow
      float horizonDist = (y - 0.24) * 4.2;
      float horizonGlow = exp(-horizonDist * horizonDist);
      sky += mix(uSkyBot, uBokehA, 0.35) * horizonGlow * 0.22;

      // Distant mountain / skyline silhouette near lower quarter
      float ridge = 0.14 + 0.055 * sin(uv.x * 5.2 + 0.6) + 0.028 * cos(uv.x * 13.0) + 0.014 * fbm(vec2(uv.x * 8.0, 2.0));
      float cityStep = 0.11 + 0.04 * step(0.55, hash(vec2(floor(uv.x * 26.0), 3.0)));
      float silhouetteH = mix(ridge, max(ridge * 0.9, cityStep), 0.35);
      float hillMask = 1.0 - smoothstep(silhouetteH - 0.018, silhouetteH + 0.018, y);
      vec3 hillCol = uSkyTop * 0.55 + uSkyBot * 0.12;
      sky = mix(sky, hillCol, hillMask * 0.72);

      // 24 out-of-focus cinematic bokeh lights in middle/lower field
      vec2 p = vec2((uv.x - 0.5) * uAspect, uv.y - 0.5);
      for (int i = 0; i < 24; i++) {
        float fi = float(i);
        float h1 = hash(vec2(fi, 1.7));
        float h2 = hash(vec2(fi, 5.3));
        float h3 = hash(vec2(fi, 9.1));
        vec2 center = vec2((h1 - 0.5) * uAspect * 0.92, -0.44 + h2 * 0.62);
        center.x += sin(uTime * 0.2 + fi) * 0.012;
        float rad = 0.028 + h3 * 0.068;
        float d = length(p - center);
        float disc = inverseSmoothstep(rad, rad * 0.66, d);
        float ring = inverseSmoothstep(rad * 0.98, rad * 0.80, d) - inverseSmoothstep(rad * 0.76, rad * 0.55, d);
        float flicker = 0.78 + 0.22 * sin(uTime * (0.9 + h3 * 1.4) + fi * 2.1);
        vec3 bCol = (mod(fi, 3.0) < 1.0) ? uBokehA : ((mod(fi, 3.0) < 2.0) ? uBokehB : uBokehC);
        float intensity = (disc * 0.42 + ring * 0.22) * flicker * (0.35 + 0.65 * (1.0 - h3));
        sky += bCol * intensity;
      }

      // Exterior rain streaks or snow flakes
      if (uSnowMode < 0.5) {
        vec2 rUv = vec2(uv.x * uAspect * 14.0 + uv.y * 1.8, uv.y * 4.5 + uTime * (1.8 + uRainAmt * 1.4));
        vec2 rid = floor(rUv);
        vec2 rf = fract(rUv);
        float rh = hash(rid);
        float streak = inverseSmoothstep(0.08, 0.0, abs(rf.x - 0.5 - (rh - 0.5) * 0.4))
                     * smoothstep(0.0, 0.25, rf.y) * inverseSmoothstep(1.0, 0.35, rf.y)
                     * step(0.68 - uRainAmt * 0.25, rh);
        sky += vec3(0.75, 0.84, 0.96) * streak * 0.10 * uRainAmt;
      } else {
        for (int j = 0; j < 2; j++) {
          float scale = (j == 0) ? 9.0 : 15.0;
          vec2 sUv = vec2(uv.x * uAspect * scale + sin(uTime * 0.6 + uv.y * 4.0) * 0.35, uv.y * scale + uTime * (0.38 + float(j) * 0.2));
          vec2 sid = floor(sUv);
          vec2 sf = fract(sUv) - 0.5;
          float sh = hash(sid + float(j) * 19.0);
          vec2 offset = vec2(hash(sid + 3.1) - 0.5, hash(sid + 7.7) - 0.5) * 0.5;
          float flake = inverseSmoothstep(0.16, 0.04, length(sf - offset)) * step(0.58, sh);
          sky += vec3(0.92, 0.95, 1.0) * flake * (0.28 - float(j) * 0.1);
        }
      }

      gl_FragColor = vec4(clamp(sky, 0.0, 1.0), 1.0);
    }
  `;

  const FS_BLUR = `
    #ifdef GL_FRAGMENT_PRECISION_HIGH
    precision highp float;
    #else
    precision mediump float;
    #endif
    varying mediump vec2 vUv;
    uniform sampler2D uTex;
    uniform vec2 uDir;
    void main() {
      vec4 sum = texture2D(uTex, vUv) * 0.204164;
      sum += texture2D(uTex, vUv + uDir * 1.407333) * 0.304005;
      sum += texture2D(uTex, vUv - uDir * 1.407333) * 0.304005;
      sum += texture2D(uTex, vUv + uDir * 3.294215) * 0.093913;
      sum += texture2D(uTex, vUv - uDir * 3.294215) * 0.093913;
      gl_FragColor = sum;
    }
  `;

  const FS_COMPOSITE = `
    #ifdef GL_FRAGMENT_PRECISION_HIGH
    precision highp float;
    #else
    precision mediump float;
    #endif
    varying mediump vec2 vUv;
    uniform sampler2D uSharpTex;
    uniform sampler2D uBlurTex;
    uniform sampler2D uMistTex;
    uniform float uAspect;
    uniform vec3 uMistTint;
    uniform vec4 uDrops[24];
    uniform vec3 uRipples[4];

    float inverseSmoothstep(float high, float low, float x) {
      return 1.0 - smoothstep(low, high, x);
    }

    float hash21(vec2 p) {
      p = fract(p * vec2(234.34, 435.345));
      p += dot(p, p + 34.23);
      return fract(p.x * p.y);
    }

    // Returns vec3(nx, ny, mask) for procedural static micro-droplets layer
    vec3 microLayer(vec2 uv, float scale, float seed) {
      vec2 p = vec2(uv.x * uAspect, uv.y) * scale;
      vec2 id = floor(p);
      vec2 f = fract(p) - 0.5;
      float rnd = hash21(id + seed);
      if (rnd < 0.32) return vec3(0.0);
      vec2 off = vec2(hash21(id + seed + 11.3) - 0.5, hash21(id + seed + 29.7) - 0.5) * 0.52;
      vec2 d = f - off;
      float r = 0.14 + 0.16 * fract(rnd * 17.3);
      float len = length(d);
      if (len > r) return vec3(0.0);
      float mask = inverseSmoothstep(r, r * 0.72, len);
      vec2 n = d / max(r, 0.001);
      return vec3(n, mask);
    }

    void main() {
      vec2 uv = vUv;
      float mist = texture2D(uMistTex, vec2(uv.x, 1.0 - uv.y)).a;

      // Combine static micro droplets (2 scales) + interactive physics drops
      vec3 m1 = microLayer(uv, 26.0, 3.0);
      vec3 m2 = microLayer(uv, 46.0, 41.0);
      vec2 normal = m1.xy * m1.z * 0.65 + m2.xy * m2.z * 0.45;
      float dropMask = max(m1.z * 0.72, m2.z * 0.55);
      float highlight = 0.0;
      float caustic = 0.0;

      vec2 pAspect = vec2((uv.x - 0.5) * uAspect, uv.y - 0.5);
      for (int i = 0; i < 24; i++) {
        vec4 dr = uDrops[i];
        if (dr.z <= 0.001) continue;
        // dr.xy in screen top-left [0..1], convert y to GL bottom-up
        vec2 dc = vec2((dr.x - 0.5) * uAspect, (1.0 - dr.y) - 0.5);
        vec2 delta = pAspect - dc;
        // Teardrop elongation along Y
        vec2 scaled = vec2(delta.x, delta.y * 0.82);
        float dist = length(scaled);
        if (dist < dr.z) {
          float m = inverseSmoothstep(dr.z, dr.z * 0.78, dist);
          vec2 n = scaled / max(dr.z, 0.0001);
          normal = mix(normal, n * 1.45, m);
          dropMask = max(dropMask, m);
          // Specular highlight top-left & caustic crescent bottom
          float spec = inverseSmoothstep(0.28, 0.0, length(n - vec2(-0.36, 0.42)));
          float rimGlow = smoothstep(0.45, 0.92, -n.y) * smoothstep(0.35, 0.85, length(n));
          highlight += spec * m * 0.92;
          caustic += rimGlow * m * 0.48;
        }
      }

      // Tactile tap ripple distortion
      vec2 rippleOffset = vec2(0.0);
      for (int r = 0; r < 4; r++) {
        vec3 rp = uRipples[r];
        if (rp.z <= 0.0 || rp.z >= 0.9) continue;
        vec2 rc = vec2((rp.x - 0.5) * uAspect, (1.0 - rp.y) - 0.5);
        float d = length(pAspect - rc);
        float ringR = rp.z * 0.24;
        float waveDist = (d - ringR) * 28.0;
        float wave = exp(-waveDist * waveDist) * (1.0 - rp.z);
        rippleOffset += normalize(pAspect - rc + 0.0001) * wave * 0.018;
      }

      // Convex lens refraction: inverted/magnified sample from sharp scene
      vec2 refractedUv = clamp(uv - normal * 0.048 * dropMask + rippleOffset, 0.002, 0.998);
      vec3 sharp = texture2D(uSharpTex, uv + rippleOffset).rgb;
      vec3 refracted = texture2D(uSharpTex, refractedUv).rgb;
      vec3 blurred = texture2D(uBlurTex, clamp(uv + rippleOffset * 0.4, 0.002, 0.998)).rgb;

      // Frosted mist layer mixes blurred scene + cool/warm condensation veil
      vec3 mistyGlass = mix(blurred, blurred * 0.78 + uMistTint * 0.28, 0.62);
      vec3 baseGlass = mix(sharp, mistyGlass, clamp(mist, 0.0, 0.94));

      // Drops cut cleanly through mist and refract the crisp scene behind
      vec3 dropColor = refracted * (0.94 + caustic * 0.45) + vec3(0.95, 0.98, 1.0) * highlight;
      // Subtle edge shadow on large drops for 3D meniscus volume
      float rimShadow = smoothstep(0.55, 0.98, length(normal)) * dropMask * 0.14;
      dropColor *= (1.0 - rimShadow);

      vec3 col = mix(baseGlass, dropColor, clamp(dropMask, 0.0, 1.0));
      // Add small micro-speckle highlights on micro droplets when light hits
      float microSpec = inverseSmoothstep(0.32, 0.0, length(m1.xy - vec2(-0.35, 0.40))) * m1.z * (1.0 - mist * 0.55);
      col += vec3(0.92, 0.96, 1.0) * microSpec * 0.42;

      // Soft filmic vignette
      float vig = inverseSmoothstep(1.25, 0.35, length((uv - 0.5) * vec2(1.0, 1.15)));
      col *= (0.86 + 0.14 * vig);

      gl_FragColor = vec4(clamp(col, 0.0, 1.0), 1.0);
    }
  `;

  function createPipeline(canvas, gl) {
    const releases = [];
    function track(resource, release) {
      if (!resource) throw new Error('WebGL allocation failed');
      releases.push(() => release.call(gl, resource));
      return resource;
    }
    function destroy() {
      for (const release of releases.reverse()) release();
      releases.length = 0;
    }
    function compile(type, source) {
      const shader = track(gl.createShader(type), gl.deleteShader);
      gl.shaderSource(shader, source);
      gl.compileShader(shader);
      if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(shader));
      return shader;
    }
    function link(vertex, fragment, names) {
      const program = track(gl.createProgram(), gl.deleteProgram);
      const shader = compile(gl.FRAGMENT_SHADER, fragment);
      gl.attachShader(program, vertex);
      gl.attachShader(program, shader);
      gl.linkProgram(program);
      if (!gl.getProgramParameter(program, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(program));
      return {
        program,
        position: gl.getAttribLocation(program, 'aPos'),
        uniforms: Object.fromEntries(names.split(' ').map(name => [name, gl.getUniformLocation(program, name)])),
      };
    }
    function makeTexture(width, height) {
      const texture = track(gl.createTexture(), gl.deleteTexture);
      gl.bindTexture(gl.TEXTURE_2D, texture);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, width, height, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
      return texture;
    }
    function makeFbo(width, height) {
      const texture = makeTexture(width, height);
      const framebuffer = track(gl.createFramebuffer(), gl.deleteFramebuffer);
      gl.bindFramebuffer(gl.FRAMEBUFFER, framebuffer);
      gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);
      if (gl.checkFramebufferStatus(gl.FRAMEBUFFER) !== gl.FRAMEBUFFER_COMPLETE) {
        throw new Error('WebGL framebuffer incomplete');
      }
      return { framebuffer, texture, width, height };
    }

    let scene, blur, composite, quad, sharp, blurA, blurB, mist;
    try {
      const vertex = compile(gl.VERTEX_SHADER, VS);
      scene = link(vertex, FS_SCENE, 'uTime uAspect uSkyTop uSkyMid uSkyBot uCloudCol uBokehA uBokehB uBokehC uRainAmt uSnowMode');
      blur = link(vertex, FS_BLUR, 'uTex uDir');
      composite = link(vertex, FS_COMPOSITE, 'uSharpTex uBlurTex uMistTex uAspect uMistTint uDrops[0] uRipples[0]');
      quad = track(gl.createBuffer(), gl.deleteBuffer);
      gl.bindBuffer(gl.ARRAY_BUFFER, quad);
      gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]), gl.STATIC_DRAW);

      sharp = makeFbo(320, 568);
      blurA = makeFbo(128, 224);
      blurB = makeFbo(128, 224);
      mist = makeTexture(160, 284);
    } catch (error) {
      destroy();
      throw error;
    }

    function bindQuad({ program, position }) {
      gl.useProgram(program);
      gl.bindBuffer(gl.ARRAY_BUFFER, quad);
      gl.enableVertexAttribArray(position);
      gl.vertexAttribPointer(position, 2, gl.FLOAT, false, 0, 0);
    }

    const drops = new Float32Array(24 * 4);
    const ripples = new Float32Array(4 * 3);
    let lastMist;
    return {
      destroy,
      render(sim, width, height) {
        if (canvas.width !== width) canvas.width = width;
        if (canvas.height !== height) canvas.height = height;
        const aspect = width / height;
        const color = sim.current;

        if (sim.mistDirty || lastMist !== sim) {
          gl.activeTexture(gl.TEXTURE2);
          gl.bindTexture(gl.TEXTURE_2D, mist);
          gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE, sim.mistCanvas);
          lastMist = sim;
          sim.mistDirty = false;
        }

        // Pass 1: sky, bokeh, distant silhouette and exterior precipitation.
        gl.bindFramebuffer(gl.FRAMEBUFFER, sharp.framebuffer);
        gl.viewport(0, 0, sharp.width, sharp.height);
        bindQuad(scene);
        const u = scene.uniforms;
        gl.uniform1f(u.uTime, sim.elapsed);
        gl.uniform1f(u.uAspect, aspect);
        gl.uniform3fv(u.uSkyTop, color.skyTop);
        gl.uniform3fv(u.uSkyMid, color.skyMid);
        gl.uniform3fv(u.uSkyBot, color.skyBot);
        gl.uniform3fv(u.uCloudCol, color.cloudCol);
        gl.uniform3fv(u.uBokehA, color.bokehA);
        gl.uniform3fv(u.uBokehB, color.bokehB);
        gl.uniform3fv(u.uBokehC, color.bokehC);
        gl.uniform1f(u.uRainAmt, color.rainAmt);
        gl.uniform1f(u.uSnowMode, color.snowMode);
        gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);

        // Passes 2 and 3: separable 9-tap Gaussian blur with bilinear sampling.
        bindQuad(blur);
        gl.activeTexture(gl.TEXTURE0);
        gl.uniform1i(blur.uniforms.uTex, 0);
        gl.bindFramebuffer(gl.FRAMEBUFFER, blurA.framebuffer);
        gl.viewport(0, 0, blurA.width, blurA.height);
        gl.bindTexture(gl.TEXTURE_2D, sharp.texture);
        gl.uniform2f(blur.uniforms.uDir, 1.8 / blurA.width, 0);
        gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);

        gl.bindFramebuffer(gl.FRAMEBUFFER, blurB.framebuffer);
        gl.viewport(0, 0, blurB.width, blurB.height);
        gl.bindTexture(gl.TEXTURE_2D, blurA.texture);
        gl.uniform2f(blur.uniforms.uDir, 0, 1.8 / blurB.height);
        gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);

        // Pass 4: refracted droplets, persistent mist and highlights.
        gl.bindFramebuffer(gl.FRAMEBUFFER, null);
        gl.viewport(0, 0, width, height);
        bindQuad(composite);
        const c = composite.uniforms;
        gl.bindTexture(gl.TEXTURE_2D, sharp.texture);
        gl.uniform1i(c.uSharpTex, 0);
        gl.activeTexture(gl.TEXTURE1);
        gl.bindTexture(gl.TEXTURE_2D, blurB.texture);
        gl.uniform1i(c.uBlurTex, 1);
        gl.activeTexture(gl.TEXTURE2);
        gl.bindTexture(gl.TEXTURE_2D, mist);
        gl.uniform1i(c.uMistTex, 2);
        gl.uniform1f(c.uAspect, aspect);
        gl.uniform3fv(c.uMistTint, color.mistTint);

        drops.fill(0);
        sim.drops.forEach((drop, i) => {
          if (!drop.active) return;
          drops[i * 4] = drop.x;
          drops[i * 4 + 1] = drop.y;
          drops[i * 4 + 2] = drop.r;
        });
        gl.uniform4fv(c['uDrops[0]'], drops);
        ripples.fill(0);
        sim.ripples.forEach((ripple, i) => {
          ripples[i * 3] = ripple.x;
          ripples[i * 3 + 1] = ripple.y;
          ripples[i * 3 + 2] = ripple.age;
        });
        gl.uniform3fv(c['uRipples[0]'], ripples);
        gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
      },
    };
  }

  function createRainWindowFallback(canvas) {
    const ctx = canvas.getContext('2d', { alpha: false });
    const rgb = arr => `rgb(${arr.map(v => Math.round(v * 255)).join(',')})`;
    return {
      render(sim, width, height) {
        if (canvas.width !== width || canvas.height !== height) {
          canvas.width = width;
          canvas.height = height;
        }
        const c = sim.current;
        const g = ctx.createLinearGradient(0, 0, 0, height);
        g.addColorStop(0, rgb(c.skyTop));
        g.addColorStop(0.55, rgb(c.skyMid));
        g.addColorStop(1, rgb(c.skyBot));
        ctx.fillStyle = g;
        ctx.fillRect(0, 0, width, height);

        // Simple soft bokeh fallback
        for (let i = 0; i < 16; i++) {
          const x = ((i * 73) % 100) / 100 * width;
          const y = (0.45 + ((i * 37) % 50) / 100) * height;
          const r = 18 + (i % 4) * 12;
          const bCol = i % 3 === 0 ? c.bokehA : i % 3 === 1 ? c.bokehB : c.bokehC;
          const bg = ctx.createRadialGradient(x, y, 2, x, y, r);
          bg.addColorStop(0, `rgba(${bCol.map(v => Math.round(v * 255)).join(',')},0.34)`);
          bg.addColorStop(1, 'rgba(0,0,0,0)');
          ctx.fillStyle = bg;
          ctx.beginPath();
          ctx.arc(x, y, r, 0, Math.PI * 2);
          ctx.fill();
        }
        ctx.globalAlpha = 0.25;
        ctx.drawImage(sim.mistCanvas, 0, 0, width, height);
        ctx.globalAlpha = 1;
        for (const d of sim.drops) {
          if (!d.active) continue;
          const x = d.x * width;
          const y = d.y * height;
          const r = d.r * Math.min(width, height);
          ctx.fillStyle = 'rgba(255,255,255,0.22)';
          ctx.beginPath();
          ctx.arc(x, y, r, 0, Math.PI * 2);
          ctx.fill();
          ctx.fillStyle = 'rgba(255,255,255,0.75)';
          ctx.beginPath();
          ctx.arc(x - r * 0.3, y - r * 0.3, r * 0.25, 0, Math.PI * 2);
          ctx.fill();
        }
      }
    };
  }

  function createRenderer(invalidate) {
    const canvas = document.createElement('canvas');
    const gl = canvas.getContext('webgl', { alpha: false, antialias: false, depth: false, stencil: false });
    let pipeline;
    let fallbackCanvas, fallback;

    function restore(event) {
      try {
        pipeline = createPipeline(canvas, gl);
      } catch (error) {
        if (!gl.isContextLost()) console.warn('Rain Window Canvas 2D fallback', error);
      }
      if (event) invalidate();
    }
    function lose(event) {
      event.preventDefault();
      pipeline = undefined;
      invalidate();
    }
    if (gl) {
      canvas.addEventListener('webglcontextlost', lose);
      canvas.addEventListener('webglcontextrestored', restore);
      restore();
    }

    return {
      draw(sim, width, height, dt, still) {
        if (still) sim.current = sim.target;
        else {
          if (sim.still) sim.current = clonePresetParams(sim.current);
          if (dt > 0) stepSim(sim, dt);
        }
        sim.still = still;
        if (pipeline && !gl.isContextLost()) {
          pipeline.render(sim, width, height);
          return canvas;
        }
        if (!fallback) {
          // A canvas that has acquired WebGL cannot acquire a 2D context.
          fallbackCanvas = document.createElement('canvas');
          fallback = createRainWindowFallback(fallbackCanvas);
        }
        fallback.render(sim, width, height);
        return fallbackCanvas;
      },
      destroy() {
        canvas.removeEventListener('webglcontextlost', lose);
        canvas.removeEventListener('webglcontextrestored', restore);
        pipeline?.destroy();
        if (gl && !gl.isContextLost()) gl.getExtension('WEBGL_lose_context')?.loseContext();
        canvas.width = canvas.height = 1;
        if (fallbackCanvas) fallbackCanvas.width = fallbackCanvas.height = 1;
      },
    };
  }

  // The gallery paints two independent simulations through one renderer. Only
  // the visible page advances; its last frame stays in place during navigation.
  function createPlayer(paint, fps) {
    const motion = matchMedia('(prefers-reduced-motion: reduce)');
    let frame = 0;
    let active = false;
    let destroyed = false;
    let renderer;
    let lastDraw = -Infinity;
    let interval = 1000 / (fps === 15 ? 15 : 30);

    function invalidate() {
      if (active && !frame) frame = requestAnimationFrame(tick);
    }
    function tick(now) {
      frame = 0;
      if (now - lastDraw + 0.5 >= interval) {
        const dt = Number.isFinite(lastDraw) ? Math.min(0.08, (now - lastDraw) / 1000) : 0;
        lastDraw = now;
        renderer ??= createRenderer(invalidate);
        paint(renderer, dt, motion.matches);
        if (motion.matches) return;
      }
      invalidate();
    }
    function setActive(value) {
      if (destroyed || active === value) return;
      active = value;
      if (active) {
        lastDraw = -Infinity;
        invalidate();
      } else {
        cancelAnimationFrame(frame);
        frame = 0;
      }
    }

    motion.addEventListener('change', invalidate);
    return {
      setActive,
      invalidate,
      setFps(value) { interval = 1000 / (value === 15 ? 15 : 30); },
      destroy() {
        setActive(false);
        destroyed = true;
        motion.removeEventListener('change', invalidate);
        renderer?.destroy();
        renderer = undefined;
      },
    };
  }

  function bindPointer(element, { enabled = () => true, tap, wipe }) {
    let drag;
    const listeners = new AbortController();
    const options = { signal: listeners.signal };
    function point(event) {
      const { left, top, width, height } = element.getBoundingClientRect();
      return [clamp((event.clientX - left) / Math.max(1, width)), clamp((event.clientY - top) / Math.max(1, height))];
    }
    element.addEventListener('pointerdown', event => {
      if (drag || event.button !== 0 || !enabled()) return;
      const [x, y] = point(event);
      drag = { id: event.pointerId, x, y, distance: 0 };
      element.setPointerCapture(event.pointerId);
    }, options);
    element.addEventListener('pointermove', event => {
      if (drag?.id !== event.pointerId || !enabled()) return;
      const [x, y] = point(event);
      const distance = Math.hypot(x - drag.x, y - drag.y);
      if (!distance) return;
      wipe(x, y, drag.x, drag.y);
      drag.x = x;
      drag.y = y;
      drag.distance += distance;
    }, options);
    element.addEventListener('pointerup', event => {
      if (drag?.id !== event.pointerId) return;
      const [x, y] = point(event);
      const distance = Math.hypot(x - drag.x, y - drag.y);
      if (enabled()) {
        if (drag.distance + distance < 0.025) tap(x, y);
        else if (distance) wipe(x, y, drag.x, drag.y);
      }
      drag = undefined;
    }, options);
    function cancel(event) {
      if (drag?.id === event.pointerId) drag = undefined;
    }
    element.addEventListener('pointercancel', cancel, options);
    element.addEventListener('lostpointercapture', cancel, options);
    return () => listeners.abort();
  }

  window.RainWindow = {
    presets: PRESETS,
    getPreset,
    createSim: createSimState,
    setPreset: setSimPreset,
    resetMist: resetSimMist,
    tap: tapSimDrop,
    wipe: wipeSimMist,
    createPlayer,
    bindPointer,
  };
})();
