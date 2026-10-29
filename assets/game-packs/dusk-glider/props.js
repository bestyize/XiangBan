/*
 * 暮色滑翔 (dusk-glider) 场景道具：树、房屋、风车、灯塔、干草堆、岩石、花丛、薰衣草田、帆船、热气球、鸟群、萤火虫。
 * 经典脚本（无 import），依赖全局 THREE，导出 globalThis.XiabanGliderProps。
 * 坐标：x 沿赛道向前，y 向上（海平面 0），z 为纵深（赛道 z=0，相机在 +z，岛屿主体在 -z）。
 * 静态道具以顶点色烘焙并合并为少量 Mesh；只有风车扇叶、灯塔光束等动画部件单独成对象。
 */
(function () {
  'use strict';

  var LAND = 0.95; // 道具足迹的最低地面高度（硬性要求 >= 0.8）
  var FRONT_LAND = 1.3, FRONT_DROP = 1.8; // 赛道前方低矮装饰：更靠内陆，且比赛道低得更多，避免遮挡飞鸟视线

  function mulberry32(a) {
    return function () {
      a = (a + 0x6D2B79F5) | 0;
      var t = Math.imul(a ^ (a >>> 15), 1 | a);
      t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
      return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    };
  }
  function clamp(v, a, b) { return v < a ? a : v > b ? b : v; }
  function pick(arr, rnd) { return arr[(rnd() * arr.length) | 0]; }

  // 各群系缺省配置：世界传入的 biome 字段优先，缺失时才回落到这里
  var DEF = {
    wheat: { trees: 'round', td: 0.5, props: ['haystack', 'house', 'fence'] },
    windmill: { trees: 'round', td: 0.45, props: ['windmill', 'house', 'flowers'] },
    orange: { trees: 'orange', td: 0.7, props: ['house', 'lighthouse'] },
    maple: { trees: 'maple', td: 0.8, props: ['cabin', 'lantern'] },
    lavender: { trees: 'birch', td: 0.4, props: ['lavender', 'house'] },
    pine: { trees: 'pine', td: 0.9, props: ['cabin', 'rock'], snow: true },
    palm: { trees: 'palm', td: 0.55, props: ['hut', 'rock', 'umbrella'] },
    generic: { trees: 'round', td: 0.4, props: ['house'] }
  };
  var TREE_TYPES = { round: 1, orange: 1, maple: 1, birch: 1, pine: 1, palm: 1 };
  var ALIAS = { stonelantern: 'lantern', toro: 'lantern', beachumbrella: 'umbrella', flower: 'flowers', hay: 'haystack', boatdock: 'dock', rocks: 'rock', lavenderrows: 'lavender' };

  function propSet(list) {
    var s = {};
    if (!list || !list.length) return s;
    for (var i = 0; i < list.length; i++) {
      var k = String(list[i]).toLowerCase().replace(/[^a-z]/g, '');
      s[ALIAS[k] || k] = true;
    }
    return s;
  }

  // ---------- 顶点色累加器：把带变换的图元烘焙进一个大数组，最后一次性生成 BufferGeometry ----------
  function Acc(lib) {
    this.lib = lib;
    this.p = [];
    this.c = [];
    this.tris = 0;
    this.g0 = 0.84; // 图元底部亮度系数
    this.g1 = 1.08; // 图元顶部亮度系数（低多边形常用的"下暗上亮"）
    this.alt = null; // 若设置，则奇数三角形使用该颜色（伞面、茅草条纹）
    this.hasBase = false;
    this.base = new lib.THREE.Matrix4();
  }
  Acc.prototype.grad = function (a, b) { this.g0 = a; this.g1 = b; return this; };
  Acc.prototype.setBase = function (x, y, z, yaw, sc) {
    var S = this.lib.s;
    S.e.set(0, yaw || 0, 0, 'YZX');
    S.q.setFromEuler(S.e);
    S.v.set(x, y, z);
    S.sc.set(sc || 1, sc || 1, sc || 1);
    this.base.compose(S.v, S.q, S.sc);
    this.hasBase = true;
  };
  Acc.prototype.clearBase = function () { this.hasBase = false; };
  Acc.prototype.emit = function (s, col, m) {
    if (this.hasBase) m.premultiply(this.base);
    var e = m.elements, pos = s.pos, cols = s.cols, n = pos.length, P = this.p, C = this.c;
    var span = (s.maxY - s.minY) || 1, k0 = this.g0, k1 = this.g1, alt = this.alt;
    for (var i = 0; i < n; i += 3) {
      var x = pos[i], y = pos[i + 1], z = pos[i + 2];
      P.push(e[0] * x + e[4] * y + e[8] * z + e[12], e[1] * x + e[5] * y + e[9] * z + e[13], e[2] * x + e[6] * y + e[10] * z + e[14]);
      var k = k0 + (k1 - k0) * ((y - s.minY) / span);
      if (cols) {
        C.push(cols[i] * k, cols[i + 1] * k, cols[i + 2] * k);
      } else {
        var cc = (alt && (((i / 9) | 0) & 1)) ? alt : col;
        C.push(cc.r * k, cc.g * k, cc.b * k);
      }
    }
    this.tris += n / 9;
  };
  // 旋转顺序 YZX：先绕 x，再绕 z（倾斜），最后绕 y（偏航）
  Acc.prototype.part = function (s, col, px, py, pz, sx, sy, sz, ry, rz, rx) {
    var S = this.lib.s;
    S.e.set(rx || 0, ry || 0, rz || 0, 'YZX');
    S.q.setFromEuler(S.e);
    S.v.set(px, py, pz);
    S.sc.set(sx, sy, sz);
    S.m.compose(S.v, S.q, S.sc);
    this.emit(s, col, S.m);
  };
  // 让单位图元（底在 y=0、高 1）从 a 点指向 b 点，水平半径 r（树干段、绳索）
  Acc.prototype.seg = function (s, col, ax, ay, az, bx, by, bz, r) {
    var S = this.lib.s;
    S.v.set(bx - ax, by - ay, bz - az);
    var len = S.v.length();
    if (len < 1e-6) return;
    S.v.multiplyScalar(1 / len);
    S.q.setFromUnitVectors(S.up, S.v);
    S.v.set(ax, ay, az);
    S.sc.set(r, len, r);
    S.m.compose(S.v, S.q, S.sc);
    this.emit(s, col, S.m);
  };
  Acc.prototype.build = function () {
    var THREE = this.lib.THREE, P = this.p, n = P.length;
    if (!n) return null;
    var pos = new Float32Array(P), col = new Float32Array(this.c), nor = new Float32Array(n);
    for (var i = 0; i < n; i += 9) {
      var ux = pos[i + 3] - pos[i], uy = pos[i + 4] - pos[i + 1], uz = pos[i + 5] - pos[i + 2];
      var vx = pos[i + 6] - pos[i], vy = pos[i + 7] - pos[i + 1], vz = pos[i + 8] - pos[i + 2];
      var nx = uy * vz - uz * vy, ny = uz * vx - ux * vz, nz = ux * vy - uy * vx;
      var l = Math.sqrt(nx * nx + ny * ny + nz * nz);
      if (l < 1e-12) { nx = 0; ny = 1; nz = 0; l = 1; }
      nx /= l; ny /= l; nz /= l;
      for (var k = 0; k < 9; k += 3) { nor[i + k] = nx; nor[i + k + 1] = ny; nor[i + k + 2] = nz; }
    }
    var g = new THREE.BufferGeometry();
    g.setAttribute('position', new THREE.BufferAttribute(pos, 3));
    g.setAttribute('normal', new THREE.BufferAttribute(nor, 3));
    g.setAttribute('color', new THREE.BufferAttribute(col, 3));
    g.computeBoundingSphere();
    return g;
  };

  // ---------- 图元源：全部转成非索引 Float32Array，底部统一在 y=0（球/二十面体以原点为中心） ----------
  function toSrc(geo) {
    var g = geo.index ? geo.toNonIndexed() : geo;
    var a = g.attributes.position.array, pos = new Float32Array(a.length), mn = Infinity, mx = -Infinity;
    for (var i = 0; i < a.length; i += 3) {
      pos[i] = a[i]; pos[i + 1] = a[i + 1]; pos[i + 2] = a[i + 2];
      if (a[i + 1] < mn) mn = a[i + 1];
      if (a[i + 1] > mx) mx = a[i + 1];
    }
    if (g !== geo) g.dispose();
    geo.dispose();
    return { pos: pos, minY: mn, maxY: mx, cols: null };
  }
  function rawSrc(list) {
    var pos = new Float32Array(list), mn = Infinity, mx = -Infinity;
    for (var i = 1; i < pos.length; i += 3) { if (pos[i] < mn) mn = pos[i]; if (pos[i] > mx) mx = pos[i]; }
    return { pos: pos, minY: mn, maxY: mx, cols: null };
  }
  function tri(out, a, b, c) { out.push(a[0], a[1], a[2], b[0], b[1], b[2], c[0], c[1], c[2]); }
  function tri2(out, a, b, c) { tri(out, a, b, c); tri(out, c, b, a); } // 双面

  // ---------- 共享库 ----------
  function beamGeometry(THREE, col, len, rad, segs) {
    var g = new THREE.ConeGeometry(rad, len, segs, 1, true);
    g.rotateZ(Math.PI / 2); // 锥尖(+y) -> -x，锥底 -> +x
    g.translate(len / 2, 0, 0); // 锥尖落在原点，向 +x 张开
    var p = g.attributes.position, c = new Float32Array(p.count * 3);
    for (var i = 0; i < p.count; i++) {
      var t = clamp(p.getX(i) / len, 0, 1), f = Math.pow(1 - t, 1.6); // 越远越淡（加法混合下黑=透明）
      c[i * 3] = col.r * f; c[i * 3 + 1] = col.g * f; c[i * 3 + 2] = col.b * f;
    }
    g.setAttribute('color', new THREE.BufferAttribute(c, 3));
    return g;
  }

  function createLibrary(THREE, opts) {
    var qual = opts && opts.quality != null ? clamp(opts.quality | 0, 0, 2) : 1;
    var lib = { THREE: THREE, quality: qual, g: {}, owned: [] };
    var S = lib.s = {
      e: new THREE.Euler(), q: new THREE.Quaternion(), m: new THREE.Matrix4(),
      v: new THREE.Vector3(), sc: new THREE.Vector3(), up: new THREE.Vector3(0, 1, 0),
      hsl: { h: 0, s: 0, l: 0 }, cp: []
    };
    for (var i = 0; i < 8; i++) S.cp.push(new THREE.Color());
    var cache = {};
    lib.C = function (h) { return cache[h] || (cache[h] = new THREE.Color(h)); };
    // 色相/饱和度/明度抖动，结果写入临时色槽 k（0..7），注意调用方不要长期持有
    lib.jit = function (k, base, rnd, dh, ds, dl) {
      base.getHSL(S.hsl);
      return S.cp[k].setHSL((S.hsl.h + (rnd() - 0.5) * 2 * dh + 1) % 1,
        clamp(S.hsl.s + (rnd() - 0.5) * 2 * ds, 0, 1), clamp(S.hsl.l + (rnd() - 0.5) * 2 * dl, 0.02, 0.98));
    };
    lib.mix = function (k, a, b, t) { return S.cp[k].copy(a).lerp(b, t); };

    lib.matLit = new THREE.MeshLambertMaterial({ vertexColors: true, flatShading: true });
    // 发光材质：仅用于窗户/灯笼/灯塔灯室，白天为顶点色，夜里 emissive 点亮
    lib.matGlow = new THREE.MeshLambertMaterial({ vertexColors: true, flatShading: true, emissive: new THREE.Color('#ffcf7a'), emissiveIntensity: 0 });
    lib.matBeam = new THREE.MeshBasicMaterial({ vertexColors: true, transparent: true, opacity: 0, blending: THREE.AdditiveBlending, depthWrite: false, side: THREE.DoubleSide });
    lib.beamGeo = beamGeometry(THREE, lib.C('#ffcf7a'), 34, 3.2, 10);
    lib.night = 0;
    lib.setEnv = function (env) {
      var n = clamp(env && env.night || 0, 0, 1), d = clamp(env && env.dusk || 0, 0, 1);
      lib.night = n;
      lib.matGlow.emissiveIntensity = clamp(n + d * 0.35 * (1 - n), 0, 1) * 1.3;
      lib.matBeam.opacity = 0.2 * n;
    };

    var G = lib.g, T = THREE;
    G.box = toSrc(new T.BoxGeometry(1, 1, 1).translate(0, 0.5, 0));
    G.cyl6 = toSrc(new T.CylinderGeometry(1, 1, 1, 6).translate(0, 0.5, 0));
    G.cyl8 = toSrc(new T.CylinderGeometry(1, 1, 1, 8).translate(0, 0.5, 0));
    G.cyl8o = toSrc(new T.CylinderGeometry(1, 1, 1, 8, 1, true).translate(0, 0.5, 0));
    G.rod = toSrc(new T.CylinderGeometry(1, 1, 1, 3, 1, true).translate(0, 0.5, 0));
    G.trunk = toSrc(new T.CylinderGeometry(0.75, 1, 1, 5, 1, true).translate(0, 0.5, 0));
    G.tower = toSrc(new T.CylinderGeometry(0.72, 1, 1, 10, 1, true).translate(0, 0.5, 0));
    G.tband = toSrc(new T.CylinderGeometry(0.93, 1, 1, 10, 1, true).translate(0, 0.5, 0));
    G.cone3 = toSrc(new T.ConeGeometry(1, 1, 3, 1, true).translate(0, 0.5, 0));
    G.cone7 = toSrc(new T.ConeGeometry(1, 1, 7, 1, true).translate(0, 0.5, 0));
    G.cone8 = toSrc(new T.ConeGeometry(1, 1, 8, 1, true).translate(0, 0.5, 0));
    G.pyr = toSrc(new T.ConeGeometry(1.41421, 1, 4, 1, true).rotateY(Math.PI / 4).translate(0, 0.5, 0)); // 底面半边长 1
    G.ico0 = toSrc(new T.IcosahedronGeometry(1, 0));
    G.ico1 = toSrc(new T.IcosahedronGeometry(1, 1));
    G.oct = toSrc(new T.OctahedronGeometry(1, 0));
    G.dome = toSrc(new T.SphereGeometry(1, 8, 3, 0, Math.PI * 2, 0, Math.PI / 2));

    // 三棱柱屋顶：x 方向长 1、z 方向宽 1、高 1，底在 y=0，脊线沿 x
    var a0 = [-0.5, 0, -0.5], a1 = [-0.5, 0, 0.5], a2 = [-0.5, 1, 0];
    var b0 = [0.5, 0, -0.5], b1 = [0.5, 0, 0.5], b2 = [0.5, 1, 0], pr = [];
    tri(pr, a0, a1, a2); tri(pr, b0, b2, b1);
    tri(pr, a1, b1, b2); tri(pr, a1, b2, a2);
    tri(pr, a0, a2, b2); tri(pr, a0, b2, b0);
    G.prism = rawSrc(pr);

    // 棕榈叶：沿 +x 伸出 1 单位，中段上拱、末端下垂，双面 8 个三角形
    var fr = [], s0 = [0, 0, 0], l1 = [0.4, 0.17, 0.15], r1 = [0.4, 0.17, -0.15];
    var l2 = [0.8, 0.08, 0.13], r2 = [0.8, 0.08, -0.13], tip = [1.15, -0.32, 0];
    tri2(fr, s0, l1, r1); tri2(fr, l1, l2, r2); tri2(fr, l1, r2, r1); tri2(fr, l2, tip, r2);
    G.frond = rawSrc(fr);

    // 小灯球（帆船桅灯、热气球火焰），走发光材质
    var ga = new Acc(lib).grad(1, 1);
    ga.part(G.oct, lib.C('#ffe2a0'), 0, 0, 0, 0.18, 0.22, 0.18);
    lib.glowGeo = ga.build();

    lib.dispose = function () {
      lib.matLit.dispose(); lib.matGlow.dispose(); lib.matBeam.dispose();
      lib.beamGeo.dispose(); lib.glowGeo.dispose();
    };
    return lib;
  }

  // ---------- 树（c 为 chunk 上下文：lib/A/W/rnd/biome/g0/g1/snow/qual）----------
  function treeR(type, H) { // 树冠水平最大半径，用来避开赛道 |z|<4 并留出间距
    switch (type) {
      case 'pine': return 0.26 * H + 0.7;
      case 'birch': return 0.2 * H + 1.2;
      case 'palm': return 0.5 * H + 1.3;
      default: return 1.4 * (0.3 * H + 0.8);
    }
  }
  function trunk(c, x, y, z, h, w, col) {
    c.A.grad(0.78, 1.0);
    c.A.part(c.lib.g.trunk, col, x, y - 0.15, z, w, h + 0.15, w, c.rnd() * 6.28);
  }

  function tRound(c, x, y, z, H, orange) {
    var A = c.A, L = c.lib, G = L.g, rnd = c.rnd, J = L.jit;
    var th = H * 0.46;
    trunk(c, x, y, z, th, 0.07 * H + 0.1, J(0, L.C('#7a5a3e'), rnd, 0.02, 0.05, 0.05));
    var R = 0.3 * H + 0.8, cy = y + th + R * 0.3;
    var leaf = J(1, L.mix(5, L.C(orange ? '#4f9a45' : '#5fa14a'), c.g1, 0.3), rnd, 0.03, 0.08, 0.05);
    A.grad(0.76, 1.12);
    if (orange) {
      A.part(G.ico1, leaf, x, cy, z, R * 1.05, R * 0.95, R * 1.05, rnd() * 6.28, 0, 0);
      var fruit = J(3, L.C('#f08a24'), rnd, 0.02, 0.05, 0.05), nf = c.qual === 0 ? 3 : 5;
      A.grad(1, 1);
      for (var i = 0; i < nf; i++) { // 橙子主要落在朝向相机(+z)的一侧
        var ta = Math.PI * 0.5 + (rnd() - 0.5) * 3.2, ph = 0.25 + rnd() * 1.1;
        A.part(G.oct, fruit, x + Math.sin(ph) * Math.cos(ta) * R * 1.04, cy + Math.cos(ph) * R * 0.94,
          z + Math.sin(ph) * Math.sin(ta) * R * 1.04, 0.26, 0.26, 0.26, rnd() * 3);
      }
      return;
    }
    var a = rnd() * 6.28;
    A.part(G.ico0, leaf, x, cy, z, R, R * 0.92, R, rnd() * 6.28, (rnd() - 0.5) * 0.5);
    A.part(G.ico0, J(2, leaf, rnd, 0.01, 0.03, 0.04), x + Math.cos(a) * R * 0.62, cy - R * 0.22, z + Math.sin(a) * R * 0.62,
      R * 0.72, R * 0.66, R * 0.72, rnd() * 6.28);
    A.part(G.ico0, J(4, leaf, rnd, 0.015, 0.03, 0.05), x + Math.cos(a + 2.4) * R * 0.3, cy + R * 0.62, z + Math.sin(a + 2.4) * R * 0.3,
      R * 0.62, R * 0.58, R * 0.62, rnd() * 6.28);
  }

  function tPine(c, x, y, z, H) {
    var A = c.A, L = c.lib, G = L.g, rnd = c.rnd, J = L.jit;
    var n = 3 + (rnd() < 0.5 ? 1 : 0), R = 0.26 * H + 0.6;
    trunk(c, x, y, z, H * 0.22, 0.05 * H + 0.1, J(0, L.C('#6b4a33'), rnd, 0.02, 0.05, 0.04));
    var base = J(1, L.C('#2f6b47'), rnd, 0.025, 0.08, 0.04), snow = L.C('#f3f7fb');
    for (var i = 0; i < n; i++) {
      var yb = y + H * 0.16 + i * (H * 0.72 / n), hh = Math.min(H * 0.46, y + H - yb), rr = R * (1 - 0.24 * i), ry = rnd() * 6.28;
      A.grad(0.7 + i * 0.05, 1.0 + i * 0.03);
      A.part(G.cone7, base, x, yb, z, rr, hh, rr, ry);
      if (c.snow) { // 雪顶：与锥面同斜率的小锥，叠在每层顶部
        A.grad(0.95, 1.05);
        A.part(G.cone7, snow, x, yb + hh * 0.58, z, rr * 0.44, hh * 0.44, rr * 0.44, ry);
      }
    }
  }

  function tBirch(c, x, y, z, H) {
    var A = c.A, L = c.lib, G = L.g, rnd = c.rnd, J = L.jit;
    var th = H * 0.72;
    trunk(c, x, y, z, th, 0.035 * H + 0.08, J(0, L.C('#ece6d8'), rnd, 0.01, 0.03, 0.03));
    var R = 0.2 * H + 0.6, leaf = J(1, L.mix(5, L.C('#b4d268'), c.g1, 0.2), rnd, 0.03, 0.08, 0.05), a = rnd() * 6.28;
    A.grad(0.8, 1.12);
    A.part(G.ico0, leaf, x, y + th * 0.9, z, R * 0.85, R * 1.25, R * 0.85, rnd() * 6.28);
    A.part(G.ico0, J(2, leaf, rnd, 0.02, 0.04, 0.05), x + Math.cos(a) * R * 0.6, y + th * 0.72, z + Math.sin(a) * R * 0.6, R * 0.65, R * 0.95, R * 0.65, rnd() * 6.28);
    A.part(G.ico0, J(4, leaf, rnd, 0.02, 0.04, 0.05), x - Math.cos(a) * R * 0.5, y + th * 1.02, z - Math.sin(a) * R * 0.5, R * 0.55, R * 0.85, R * 0.55, rnd() * 6.28);
  }

  function tMaple(c, x, y, z, H) {
    var A = c.A, L = c.lib, G = L.g, rnd = c.rnd, J = L.jit;
    var th = H * 0.42;
    trunk(c, x, y, z, th, 0.07 * H + 0.1, J(0, L.C('#5e4332'), rnd, 0.02, 0.05, 0.04));
    var R = 0.3 * H + 0.9, warm = rnd() < 0.5 ? ['#c9452b', '#e07b26', '#e3a92f'] : ['#e07b26', '#c9452b', '#d9962c'];
    A.grad(0.78, 1.12);
    A.part(G.ico0, J(1, L.C(warm[0]), rnd, 0.015, 0.06, 0.05), x, y + th + R * 0.25, z, R, R * 0.6, R, rnd() * 6.28);
    A.part(G.ico0, J(2, L.C(warm[1]), rnd, 0.015, 0.06, 0.05), x + (rnd() - 0.5) * R * 0.3, y + th + R * 0.75, z + (rnd() - 0.5) * R * 0.3, R * 0.78, R * 0.52, R * 0.78, rnd() * 6.28);
    A.part(G.ico0, J(3, L.C(warm[2]), rnd, 0.015, 0.06, 0.05), x + (rnd() - 0.5) * R * 0.2, y + th + R * 1.18, z + (rnd() - 0.5) * R * 0.2, R * 0.5, R * 0.42, R * 0.5, rnd() * 6.28);
  }

  function tPalm(c, x, y, z, H) {
    var A = c.A, L = c.lib, G = L.g, rnd = c.rnd, J = L.jit;
    var dir = rnd() * 6.28, lean = (0.14 + rnd() * 0.1) * H, segs = 5;
    var tc = J(0, L.C('#a98560'), rnd, 0.015, 0.05, 0.04), r0 = 0.18 + 0.03 * H;
    var px = x, py = y - 0.2, pz = z;
    A.grad(0.66, 1.02); // 每段上亮下暗，自然形成竹节般的环纹
    for (var i = 1; i <= segs; i++) {
      var t = i / segs, nx = x + Math.cos(dir) * lean * t * t, ny = y + H * t, nz = z + Math.sin(dir) * lean * t * t;
      A.seg(G.trunk, tc, px, py, pz, nx, ny, nz, r0);
      r0 *= 0.75; px = nx; py = ny; pz = nz;
    }
    var nF = c.qual < 2 ? 6 : 7, len0 = 0.3 * H + 1.1, leaf = J(2, L.mix(5, L.C('#3f9f55'), c.g1, 0.1), rnd, 0.03, 0.08, 0.05);
    A.grad(0.85, 1.12);
    for (var k = 0; k < nF; k++) {
      var len = len0 * (0.85 + rnd() * 0.3);
      A.part(G.frond, (k & 1) ? leaf : J(3, leaf, rnd, 0.01, 0.03, 0.06), px, py + 0.1, pz, len, len, len * 0.9,
        k / nF * 6.283 + (rnd() - 0.5) * 0.4, (rnd() - 0.3) * 0.5);
    }
    A.grad(0.9, 1.0);
    var nut = J(4, L.C('#6b4a2f'), rnd, 0.02, 0.05, 0.04);
    A.part(G.ico0, nut, px + 0.22, py - 0.15, pz, 0.22, 0.22, 0.22, rnd() * 3);
    A.part(G.ico0, nut, px - 0.12, py - 0.2, pz + 0.2, 0.2, 0.2, 0.2, rnd() * 3);
  }

  // ---------- 建筑 ----------
  var WALLS = ['#f0e4cc', '#f6efe0', '#e9d3b0', '#d9e4df', '#f2d7bd', '#efe0c8'];
  var ROOFS = ['#c8553d', '#3f8f8b', '#b5623a', '#d0743c', '#4a6fa5', '#a3443c'];
  var DOORS = ['#6a4a35', '#2f6f6b', '#8a3f32', '#3f5a80'];

  // 窗户：玻璃走发光网格（夜里点亮），窗框走普通网格；坐标是当前 base 的局部坐标，y 为窗底
  function windowAt(c, x, y, z, w, h) {
    var L = c.lib;
    c.W.grad(1, 1);
    c.W.part(L.g.box, L.C('#cfe6ee'), x, y, z, w, h, 0.1);
    if (c.qual > 0) {
      c.A.grad(0.95, 1.0);
      c.A.part(L.g.box, L.C('#fbf5e8'), x, y - 0.06, z - 0.015, w + 0.16, h + 0.12, 0.08);
    }
  }

  function bHouse(c, x, hi, lo, z, yaw) {
    var A = c.A, L = c.lib, G = L.g, rnd = c.rnd, J = L.jit;
    var w = 3.6 + rnd() * 1.6, d = 2.8 + rnd() * 1.0, h = 2.2 + rnd() * 0.6, rh = 1.4 + rnd() * 0.7, fh = hi - lo + 0.5;
    var wall = J(0, L.C(pick(WALLS, rnd)), rnd, 0.01, 0.05, 0.03), roof = J(1, L.C(pick(ROOFS, rnd)), rnd, 0.015, 0.06, 0.05);
    c.setBase(x, hi, z, yaw, 1);
    A.grad(0.82, 1.0);
    A.part(G.box, J(2, L.C('#9a9284'), rnd, 0.01, 0.04, 0.04), 0, -fh, 0, w + 0.3, fh + 0.3, d + 0.3);
    A.grad(0.86, 1.05);
    A.part(G.box, wall, 0, 0.3, 0, w, h, d);
    A.grad(0.9, 1.12);
    A.part(G.prism, roof, 0, 0.25 + h, 0, w + 0.8, rh, d + 0.9);
    A.grad(0.85, 1.0);
    A.part(G.box, J(3, L.C('#9b5a45'), rnd, 0.01, 0.05, 0.04), w * 0.26, 0.3 + h, -d * 0.16, 0.5, rh + 0.45, 0.5);
    A.part(G.box, L.C('#4a4038'), w * 0.26, 0.3 + h + rh + 0.45, -d * 0.16, 0.66, 0.1, 0.66);
    A.part(G.box, J(4, L.C(pick(DOORS, rnd)), rnd, 0.01, 0.05, 0.04), -w * 0.02, 0.3, d / 2, 0.75, 1.4, 0.1);
    windowAt(c, w * 0.3, 0.3 + h * 0.45, d / 2, 0.65, 0.6);
    windowAt(c, -w * 0.32, 0.3 + h * 0.45, d / 2, 0.65, 0.6);
    c.clearBase();
  }

  function bCabin(c, x, hi, lo, z, yaw) {
    var A = c.A, L = c.lib, G = L.g, rnd = c.rnd, J = L.jit;
    var w = 3.8 + rnd() * 1.2, d = 3.0 + rnd() * 0.6, h = 2.0 + rnd() * 0.4, rh = 1.3 + rnd() * 0.5, fh = hi - lo + 0.5;
    var wood = J(0, L.C('#8b5e3c'), rnd, 0.015, 0.06, 0.04), logs = J(1, L.C('#6e4527'), rnd, 0.015, 0.05, 0.03);
    var roof = J(2, L.C(c.snow ? '#5c4a46' : '#6a4a3a'), rnd, 0.015, 0.05, 0.04), stone = J(3, L.C('#8c7f72'), rnd, 0.01, 0.04, 0.04);
    c.setBase(x, hi, z, yaw, 1);
    A.grad(0.82, 1.0);
    A.part(G.box, stone, 0, -fh, 0, w + 0.3, fh + 0.3, d + 0.3);
    A.grad(0.86, 1.05);
    A.part(G.box, wood, 0, 0.3, 0, w, h, d);
    if (c.qual > 0) for (var i = 0; i < 3; i++) A.part(G.box, logs, 0, 0.75 + i * 0.6, 0, w + 0.1, 0.12, d + 0.1);
    A.grad(0.9, 1.1);
    A.part(G.prism, roof, 0, 0.25 + h, 0, w + 1.0, rh, d + 1.0);
    if (c.snow) { // 屋顶积雪：略薄一圈，露出屋檐
      A.grad(1, 1);
      A.part(G.prism, L.C('#f4f7fb'), 0, 0.33 + h, 0, w + 1.05, rh + 0.03, (d + 1.0) * 0.88);
    }
    A.grad(0.85, 1.0);
    A.part(G.box, stone, w * 0.3, 0.3 + h, -d * 0.16, 0.6, rh + 0.7, 0.6);
    A.part(G.box, L.C(c.snow ? '#f4f7fb' : '#5a524a'), w * 0.3, 0.3 + h + rh + 0.7, -d * 0.16, 0.76, 0.12, 0.76);
    if (c.snow && c.qual > 0) { // 烟囱冒烟
      A.grad(0.92, 1.08);
      for (var k = 0; k < 3; k++) A.part(G.ico0, L.C('#e8e8ee'), w * 0.3 + 0.2 * k, 0.3 + h + rh + 1.3 + k * 0.7, -d * 0.16, 0.25 + k * 0.09, 0.22 + k * 0.08, 0.25 + k * 0.09, rnd() * 3);
    }
    A.grad(0.85, 1.0);
    A.part(G.box, J(4, L.C(pick(DOORS, rnd)), rnd, 0.01, 0.05, 0.04), -w * 0.1, 0.3, d / 2, 0.8, 1.4, 0.1);
    A.part(G.box, logs, -w * 0.1, 0.0, d / 2 + 0.5, 1.7, 0.3, 0.9); // 门廊台阶
    windowAt(c, w * 0.26, 0.3 + h * 0.45, d / 2, 0.7, 0.6);
    windowAt(c, -w * 0.36, 0.3 + h * 0.45, d / 2, 0.55, 0.6);
    c.clearBase();
  }

  function bHut(c, x, hi, lo, z, yaw) {
    var A = c.A, L = c.lib, G = L.g, rnd = c.rnd, J = L.jit;
    var r = 1.5 + rnd() * 0.3, h = 1.9 + rnd() * 0.3, fh = hi - lo + 0.5;
    var wall = J(0, L.C(pick(['#d2a872', '#c69a66', '#e0c08c'], rnd)), rnd, 0.015, 0.06, 0.04);
    var straw = J(1, L.C('#c9a24e'), rnd, 0.015, 0.06, 0.04), straw2 = J(2, L.C('#a98335'), rnd, 0.015, 0.06, 0.04);
    c.setBase(x, hi, z, yaw, 1);
    A.grad(0.8, 1.0);
    A.part(G.cyl8, J(3, L.C('#8b6b45'), rnd, 0.01, 0.05, 0.04), 0, -fh, 0, r + 0.25, fh + 0.4, r + 0.25);
    A.grad(0.86, 1.06);
    A.part(G.cyl8, wall, 0, 0.4, 0, r, h, r, Math.PI / 8);
    A.grad(0.9, 1.1);
    A.alt = straw2;
    A.part(G.cone8, straw, 0, 0.3 + h, 0, r + 0.9, 1.9 + rnd() * 0.3, r + 0.9, Math.PI / 8);
    A.alt = null;
    A.part(G.ico0, straw2, 0, 0.3 + h + 2.05, 0, 0.2, 0.26, 0.2);
    A.grad(0.85, 1.0);
    A.part(G.box, J(4, L.C(pick(DOORS, rnd)), rnd, 0.01, 0.05, 0.04), 0, 0.4, r * 0.92 - 0.02, 0.7, 1.3, 0.14);
    windowAt(c, -r * 0.62, 0.4 + h * 0.5, r * 0.74, 0.5, 0.45);
    c.clearBase();
  }

  // 风车：塔身静态合并，带 4 片帆的转子单独成对象（c.addRotor）
  function bWindmill(c, x, hi, lo, z, yaw) {
    var A = c.A, L = c.lib, G = L.g, rnd = c.rnd, J = L.jit, THREE = L.THREE;
    var rb = 2.0 + rnd() * 0.5, hb = 9 + rnd() * 2.2, fh = hi - lo + 0.5;
    var body = J(0, L.C(pick(['#efe6d2', '#f3ead8', '#e4d6bb', '#dfe6e2'], rnd)), rnd, 0.01, 0.04, 0.03);
    var roof = J(1, L.C(pick(ROOFS, rnd)), rnd, 0.015, 0.05, 0.05);
    function rAt(t) { return rb * (1 - 0.28 * t / hb); } // t: 距塔底高度
    c.setBase(x, hi, z, yaw, 1);
    A.grad(0.8, 1.0);
    A.part(G.cyl8, J(2, L.C('#9a9284'), rnd, 0.01, 0.04, 0.04), 0, -fh, 0, rb * 1.18, fh + 0.5, rb * 1.18);
    A.grad(0.82, 1.08);
    A.part(G.tower, body, 0, 0.4, 0, rb, hb, rb, Math.PI / 10); // 转 18° 让一个面正对 +z
    A.grad(0.9, 1.0);
    A.part(G.cyl8o, roof, 0, 0.4 + hb * 0.36, 0, rAt(hb * 0.36) * 1.1, 0.3, rAt(hb * 0.36) * 1.1);
    A.part(G.cyl8, body, 0, 0.4 + hb - 0.1, 0, rAt(hb) * 1.06, 0.3, rAt(hb) * 1.06);
    A.grad(0.88, 1.12);
    A.part(G.cone8, roof, 0, 0.4 + hb + 0.1, 0, rAt(hb) * 1.3, 2.6 + rnd() * 0.8, rAt(hb) * 1.3);
    var fz = Math.cos(Math.PI / 10);
    A.grad(0.85, 1.0);
    A.part(G.box, J(3, L.C(pick(DOORS, rnd)), rnd, 0.01, 0.05, 0.04), 0, 0.4, rb * fz - 0.03, 0.9, 1.6, 0.14);
    windowAt(c, 0, 0.4 + hb * 0.42, rAt(hb * 0.42) * fz - 0.02, 0.6, 0.75);
    windowAt(c, 0, 0.4 + hb * 0.66, rAt(hb * 0.66) * fz - 0.02, 0.5, 0.6);
    var py = 0.4 + hb * 0.84, off = rAt(hb * 0.84) * fz + 0.6;
    A.grad(0.85, 1.1);
    A.part(G.ico0, J(4, L.C('#6d5238'), rnd, 0.01, 0.05, 0.04), 0, py, off - 0.35, 0.7, 0.7, 0.8, rnd() * 3); // 机头
    c.clearBase();

    // 转子（局部坐标：原点在轮毂，扇叶在 xy 平面，朝 +z 一侧）
    var R = new Acc(L).grad(0.9, 1.05), Lb = 4.8 + rnd() * 1.2;
    var arm = J(5, L.C('#7b5a3c'), rnd, 0.01, 0.05, 0.04), cloth = J(6, L.C(pick(['#f4ead2', '#f7f0e0', '#efe2c4'], rnd)), rnd, 0.01, 0.04, 0.03);
    for (var k = 0; k < 4; k++) {
      var ang = k * Math.PI / 2, ca = Math.cos(ang), sa = Math.sin(ang);
      R.part(G.box, arm, 0, 0, 0, 0.24, Lb, 0.2, 0, ang);
      R.part(G.box, cloth, 0.62 * ca - 1.0 * sa, 0.62 * sa + 1.0 * ca, 0.08, 1.0, Lb - 1.2, 0.06, 0, ang);
    }
    R.part(G.ico0, J(7, L.C('#5a422c'), rnd, 0.01, 0.05, 0.04), 0, 0, 0.1, 0.5, 0.5, 0.5);
    c.addRotor(R.build(), x + Math.sin(yaw) * off, hi + py, z + Math.cos(yaw) * off, yaw);
  }

  function bLighthouse(c, x, hi, lo, z) {
    var A = c.A, W = c.W, L = c.lib, G = L.g, rnd = c.rnd, J = L.jit;
    var red = J(0, L.C('#d8483c'), rnd, 0.01, 0.05, 0.03), white = J(1, L.C('#f4efe6'), rnd, 0.01, 0.03, 0.02);
    var dark = L.C('#34343e'), fh = hi - lo + 0.6, fz = Math.cos(Math.PI / 10);
    c.setBase(x, hi, z, 0, 1);
    A.grad(0.8, 1.0);
    A.part(G.cyl8, J(2, c.rockC, rnd, 0.01, 0.04, 0.04), 0, -fh, 0, 2.7, fh + 0.4, 2.7);
    for (var i = 0; i < 4; i++) {
      var ri = 1.9 * Math.pow(0.93, i);
      A.grad(0.84, 1.08);
      A.part(G.tband, i % 2 ? red : white, 0, 0.4 + i * 2.7, 0, ri, 2.7, ri, Math.PI / 10);
    }
    A.grad(0.85, 1.0);
    A.part(G.box, J(3, L.C('#6a4a35'), rnd, 0.01, 0.05, 0.04), 0, 0.4, 1.9 * fz - 0.03, 0.8, 1.5, 0.14);
    windowAt(c, 0, 5.0, 1.64 * fz - 0.04, 0.5, 0.7);
    windowAt(c, 0, 8.2, 1.46 * fz - 0.04, 0.45, 0.65);
    A.part(G.cyl8, dark, 0, 11.2, 0, 2.05, 0.3, 2.05); // 回廊
    A.part(G.cyl8o, dark, 0, 11.5, 0, 1.95, 0.5, 1.95); // 栏杆
    W.grad(1, 1);
    W.part(G.cyl8, L.C('#ffe2a0'), 0, 11.5, 0, 1.0, 1.5, 1.0); // 灯室（夜里发光）
    A.part(G.cyl8, dark, 0, 13.0, 0, 1.2, 0.2, 1.2);
    A.grad(0.88, 1.12);
    A.part(G.cone8, red, 0, 13.2, 0, 1.35, 1.6, 1.35);
    A.part(G.ico0, dark, 0, 14.75, 0, 0.2, 0.2, 0.2);
    c.clearBase();
    c.addBeam(x, hi + 12.25, z);
  }

  function bLantern(c, x, hi, z, s) { // 石灯笼
    var A = c.A, W = c.W, L = c.lib, G = L.g, rnd = c.rnd, J = L.jit;
    var st = J(0, L.C('#a8a496'), rnd, 0.01, 0.04, 0.05);
    c.setBase(x, hi, z, rnd() * 0.6 - 0.3, s);
    A.grad(0.8, 1.05);
    A.part(G.box, st, 0, -0.3, 0, 0.9, 0.3, 0.9);
    A.part(G.cyl6, st, 0, 0, 0, 0.17, 0.9, 0.17);
    A.part(G.box, st, 0, 0.9, 0, 0.62, 0.12, 0.62);
    W.grad(1, 1);
    W.part(G.box, L.C('#ffe9b8'), 0, 1.02, 0, 0.42, 0.5, 0.42);
    A.part(G.pyr, st, 0, 1.52, 0, 0.55, 0.38, 0.55);
    A.part(G.ico0, st, 0, 1.88, 0, 0.09, 0.09, 0.09);
    c.clearBase();
  }

  function bHay(c, x, y, z, s) { // 干草堆：圆柱 + 半球顶 + 束腰
    var A = c.A, L = c.lib, G = L.g, rnd = c.rnd, J = L.jit;
    var r = (0.95 + rnd() * 0.5) * s, h = (1.1 + rnd() * 0.6) * s, col = J(0, L.C('#e2bd5f'), rnd, 0.015, 0.08, 0.05);
    A.grad(0.84, 1.04);
    A.part(G.cyl8, col, x, y - 0.1, z, r, h, r, rnd() * 3);
    A.grad(0.95, 1.12);
    A.part(G.dome, J(1, col, rnd, 0.01, 0.03, 0.04), x, y - 0.1 + h, z, r, r * 0.85, r, rnd() * 3);
    A.grad(0.9, 1.0);
    A.part(G.cyl8o, L.mix(2, col, L.C('#8a6a2a'), 0.55), x, y - 0.1 + h * 0.42, z, r * 1.03, 0.14 * s, r * 1.03);
  }
  function bBale(c, x, y, z, s) { // 卧着的圆草捆
    var A = c.A, L = c.lib, rnd = c.rnd, r = (0.6 + rnd() * 0.15) * s, len = 1.1 * s;
    A.grad(0.85, 1.08);
    A.part(L.g.cyl8, L.jit(0, L.C('#dcb554'), rnd, 0.015, 0.08, 0.05), x + len / 2, y - 0.05 + r, z, r, len, r, 0, Math.PI / 2);
  }

  function bRock(c, x, y, z, s, big) {
    var A = c.A, L = c.lib, G = L.g, rnd = c.rnd;
    A.grad(0.76, 1.12);
    A.part(big ? G.ico1 : G.ico0, L.jit(0, c.rockC, rnd, 0.01, 0.05, 0.06), x, y + s * 0.1, z,
      s * (0.9 + rnd() * 0.4), s * (0.55 + rnd() * 0.25), s * (0.8 + rnd() * 0.3), rnd() * 6.28, (rnd() - 0.5) * 0.3, (rnd() - 0.5) * 0.3);
    if (c.snow) {
      A.grad(0.95, 1.05);
      A.part(G.ico0, L.C('#f3f7fb'), x, y + s * 0.38, z, s * 0.66, s * 0.3, s * 0.58, rnd() * 6.28);
    }
  }

  var UMBRELLAS = [['#e0523f', '#f6efe0'], ['#2f8f9a', '#f6efe0'], ['#f0a53a', '#f6efe0'], ['#d9638f', '#f6efe0']];
  function bUmbrella(c, x, y, z) { // 沙滩伞 + 浴巾
    var A = c.A, L = c.lib, G = L.g, rnd = c.rnd, pair = pick(UMBRELLAS, rnd), tilt = 0.12 + rnd() * 0.08;
    A.grad(0.9, 1.05);
    A.part(G.cyl6, L.C('#efe7d6'), x, y - 0.1, z, 0.05, 2.4, 0.05, 0, tilt);
    A.alt = L.C(pair[1]);
    A.part(G.cone8, L.C(pair[0]), x - Math.sin(tilt) * 2.4, y - 0.1 + Math.cos(tilt) * 2.4 - 0.45, z, 1.35, 0.55, 1.35, rnd() * 3, tilt * 0.6);
    A.alt = null;
    A.part(G.box, L.jit(0, L.C(pair[0]), rnd, 0.01, 0.05, 0.05), x + 0.4, y, z + 0.9, 0.9, 0.06, 1.5, rnd() * 0.5);
  }

  // ---------- 地块道具 ----------
  function buildChunk(lib, o) {
    var THREE = lib.THREE, qual = lib.quality, G = lib.g, L = lib, J = lib.jit;
    var group = new THREE.Group();
    group.name = 'glider-props';
    var geos = [], rotors = [], rotSpd = [], beams = [], beamPh = [];
    var res = {
      group: group,
      triangles: 0,
      update: function (t, dt, env) {
        lib.setEnv(env);
        var w = env && env.wind != null ? clamp(env.wind, 0, 1) : 0.3, i, sp = dt * (0.5 + w * 1.9);
        for (i = 0; i < rotors.length; i++) rotors[i].rotation.z -= sp * rotSpd[i];
        var lit = lib.night > 0.03;
        for (i = 0; i < beams.length; i++) {
          beams[i].visible = lit;
          if (lit) beams[i].rotation.y = t * 0.75 + beamPh[i];
        }
      },
      dispose: function () {
        for (var i = 0; i < geos.length; i++) geos[i].dispose();
        geos.length = 0; rotors.length = 0; beams.length = 0;
        while (group.children.length) group.remove(group.children[0]);
      }
    };
    var isl = o.island;
    if (!isl) return res; // 海峡：没有道具

    var x0 = o.x0, x1 = o.x1, hAt = o.heightAt, biome = o.biome || {};
    var def = DEF[biome.id] || DEF.generic;
    var treeType = TREE_TYPES[biome.trees] ? biome.trees : (TREE_TYPES[def.trees] ? def.trees : 'round');
    var td = isFinite(biome.treeDensity) ? +biome.treeDensity : def.td;
    var has = propSet(biome.props && biome.props.length ? biome.props : def.props);
    var snow = biome.snow != null ? !!biome.snow : !!def.snow;
    var grass = biome.grass && biome.grass.length ? biome.grass : ['#a9c060', '#7fae54'];
    var ia = isFinite(isl.x0) ? isl.x0 : x0, ib = -Infinity;
    if (isFinite(isl.x1)) ib = isl.x1;
    if (isFinite(isl.end)) ib = Math.max(ib, isl.end);
    if (ib === -Infinity) ib = x1;
    var xa = Math.max(x0, ia), xb = Math.min(x1, ib), span = xb - xa;
    if (span < 6) return res;

    var A = new Acc(lib), W = new Acc(lib);
    var rnd = mulberry32(Math.imul(o.seed | 0, 0x9E3779B1) ^ Math.imul(Math.floor(x0) | 0, 0x85EBCA6B) ^ 0x27D4EB2F);
    var c = {
      lib: lib, A: A, W: W, rnd: rnd, biome: biome, snow: snow, qual: qual,
      g0: lib.C(grass[0]), g1: lib.C(grass[grass.length > 1 ? 1 : 0]), rockC: lib.C(biome.rock || '#9c8a6e'),
      setBase: function (x, y, z, yaw, s) { A.setBase(x, y, z, yaw, s); W.setBase(x, y, z, yaw, s); },
      clearBase: function () { A.clearBase(); W.clearBase(); },
      addRotor: function (geo, wx, wy, wz, yaw) {
        if (!geo) return;
        geos.push(geo);
        var pivot = new THREE.Group(), rotor = new THREE.Group(), m = new THREE.Mesh(geo, lib.matLit);
        pivot.position.set(wx, wy, wz);
        pivot.rotation.y = yaw;
        m.castShadow = qual > 0;
        rotor.add(m);
        rotor.rotation.z = rnd() * 6.28;
        pivot.add(rotor);
        group.add(pivot);
        rotors.push(rotor); rotSpd.push(0.8 + rnd() * 0.4);
      },
      addBeam: function (x, y, z) { // 灯塔光束：两条反向的加法混合锥体，夜里绕灯旋转
        var bg = new THREE.Group();
        bg.position.set(x, y, z);
        for (var s = 0; s < 2; s++) {
          for (var k = 0; k < (qual > 0 ? 2 : 1); k++) {
            var m = new THREE.Mesh(lib.beamGeo, lib.matBeam);
            m.rotation.y = s * Math.PI;
            if (k) m.scale.set(1, 0.45, 0.45);
            m.frustumCulled = false;
            m.renderOrder = 5;
            bg.add(m);
          }
        }
        bg.visible = false;
        group.add(bg);
        beams.push(bg); beamPh.push(rnd() * 6.28);
      }
    };

    var occ = [], cap = [1700, 3500, 5600][qual], QF = [0.3, 0.65, 1][qual];
    function room(n) { return A.tris + W.tris + n <= cap; }
    function free(x, z, r) {
      for (var i = 0; i < occ.length; i += 3) {
        var dx = occ[i] - x, dz = occ[i + 1] - z, rr = occ[i + 2] + r;
        if (dx * dx + dz * dz < rr * rr) return false;
      }
      return true;
    }
    // 足迹探测：中心 + 四向(+四角)采样，全部在陆地上且坡度不陡才通过，结果写入 pr
    var pr = { c: 0, lo: 0, hi: 0 };
    function probe(x, z, r, ms, diag) {
      var cc = hAt(x, z), lo = cc, hi = cc, v, k, s;
      var ox = [r, -r, 0, 0, r * 0.72, -r * 0.72, r * 0.72, -r * 0.72];
      var oz = [0, 0, r, -r, r * 0.72, r * 0.72, -r * 0.72, -r * 0.72];
      s = diag ? 8 : 4;
      for (k = 0; k < s; k++) {
        v = hAt(x + ox[k], z + oz[k]);
        if (v < lo) lo = v;
        if (v > hi) hi = v;
      }
      if (!(lo >= LAND) || !(hi - lo <= ms * 2 * r)) return false;
      pr.c = cc; pr.lo = lo; pr.hi = hi;
      return true;
    }
    function frontOK(x, z) { var h = hAt(x, z); return h >= FRONT_LAND && hAt(x, 0) - h >= FRONT_DROP; } // 赛道前方的低矮装饰：地面须明显低于赛道(>=1.5m，留余量取 1.8m)

    // ---- 低矮装饰 ----
    var fcol = L.C(biome.flowers || '#f4e3a1'), cream = L.C('#fff3dc');
    function flowerPatch(px, pz, r) {
      var n = qual === 0 ? 3 : 4 + ((rnd() * 3) | 0), stem = J(1, L.mix(5, c.g1, L.C('#4f9a45'), 0.6), rnd, 0.03, 0.06, 0.05);
      for (var i = 0; i < n; i++) {
        var a = rnd() * 6.28, d = r * Math.sqrt(rnd()), fx = px + Math.cos(a) * d, fz = pz + Math.sin(a) * d, fy = hAt(fx, fz);
        if (!(fy >= (pz > 0 ? FRONT_LAND : LAND))) continue;
        var h = 0.35 + rnd() * 0.3;
        A.grad(0.8, 1.0);
        A.part(G.cone3, stem, fx, fy - 0.05, fz, 0.05, h, 0.05, rnd() * 3);
        A.grad(0.95, 1.1);
        A.part(G.oct, rnd() < 0.22 ? cream : J(0, fcol, rnd, 0.025, 0.08, 0.06), fx, fy + h, fz, 0.17, 0.12, 0.17, rnd() * 3);
      }
    }
    function tuft(px, py, pz) {
      var gc = J(0, L.mix(5, c.g0, c.g1, rnd()), rnd, 0.02, 0.05, 0.04), n = qual > 1 ? 4 : 3;
      A.grad(0.8, 1.18);
      for (var i = 0; i < n; i++) {
        A.part(G.cone3, gc, px + (rnd() - 0.5) * 0.4, py - 0.03, pz + (rnd() - 0.5) * 0.4, 0.1, 0.5 + rnd() * 0.4, 0.1,
          rnd() * 3, (rnd() - 0.5) * 0.5, (rnd() - 0.5) * 0.5);
      }
    }
    var fxs = [], fys = [], fzs = [];
    function fenceRun(sx, sz, n, front) { // 木栅栏：沿 x 方向，立柱高 0.95m，两道横杆随地形起伏
      var cnt = 0, i, k;
      for (i = 0; i < n; i++) {
        var px = sx + i * 1.6, pz = sz + Math.sin(i * 0.7 + sx) * 0.15, ph = hAt(px, pz);
        if (px < xa + 1 || px > xb - 1 || !(ph >= (front ? FRONT_LAND : LAND))) break;
        if (front ? (hAt(px, 0) - ph < FRONT_DROP) : pz > -6.5) break;
        if (cnt > 0 && Math.abs(ph - fys[cnt - 1]) > 0.9) break;
        if (!free(px, pz, 0.5)) break;
        fxs[cnt] = px; fys[cnt] = ph; fzs[cnt] = pz; cnt++;
      }
      if (cnt < 3 || !room(cnt * 60)) return;
      var wood = J(0, L.C('#b48a5c'), rnd, 0.015, 0.06, 0.05);
      A.grad(0.85, 1.05);
      for (i = 0; i < cnt; i++) {
        A.part(G.box, wood, fxs[i], fys[i] - 0.15, fzs[i], 0.14, 1.1, 0.14);
        occ.push(fxs[i], fzs[i], 0.4);
        if (i > 0) {
          for (k = 0; k < 2; k++) {
            var hk = 0.36 + k * 0.34;
            A.seg(G.box, wood, fxs[i - 1], fys[i - 1] + hk, fzs[i - 1], fxs[i], fys[i] + hk, fzs[i], 0.07);
          }
        }
      }
    }
    function lavField(fx, fz, len, rows, front) { // 薰衣草田：紫色小丘连成的条纹
      var n = Math.max(3, Math.floor(len / 1.35)), i, j;
      for (j = 0; j < rows; j++) {
        var rz = fz + (j - (rows - 1) / 2) * 1.25, col = J(0, L.C(j % 2 ? '#8e6fc2' : '#a384d4'), rnd, 0.015, 0.06, 0.04);
        for (i = 0; i < n; i++) {
          var bx = fx + (i - (n - 1) / 2) * 1.35 + (rnd() - 0.5) * 0.25, bz = rz + (rnd() - 0.5) * 0.25, bh = hAt(bx, bz);
          if (!(bh >= (front ? FRONT_LAND : LAND)) || (front ? hAt(bx, 0) - bh < FRONT_DROP : bz > -6.5)) continue;
          if (!room(22)) return;
          A.grad(0.8, 1.15);
          A.part(G.ico0, col, bx, bh + 0.08, bz, 0.62, (0.5 + rnd() * 0.2) * (front ? 0.75 : 1), 0.5, rnd() * 6);
        }
      }
      occ.push(fx, fz, len * 0.5 + 0.5);
    }

    // ---- 灯塔：每座岛一个，放在 ramp 所在的 chunk，z ≈ -8…-12 ----
    var t, t2, i;
    if (has.lighthouse && isFinite(isl.ramp) && isl.ramp >= x0 && isl.ramp < x1 && x1 - x0 > 12) {
      var lr = mulberry32(Math.imul(o.seed | 0, 0x2545F491) ^ Math.imul((isl.index | 0) + 11, 0x9E3779B1));
      for (t = 0; t < 16; t++) {
        var lx = clamp(isl.ramp + (lr() - 0.5) * 10, x0 + 5.6, x1 - 5.6), lz = -(8 + lr() * 4 + (t > 7 ? 6 : 0));
        if (probe(lx, lz, 2.8, 0.5, true)) {
          bLighthouse(c, lx, pr.hi, pr.lo, lz);
          occ.push(lx, lz, 5.5);
          break;
        }
      }
    }

    var bx0 = xa + 5, bx1 = xb - 5;

    // ---- 风车 ----
    if (has.windmill && bx1 - bx0 > 8 && rnd() < (qual === 0 ? 0.5 : 0.68)) {
      for (t = 0; t < 14; t++) {
        var wx = bx0 + 1 + rnd() * (bx1 - bx0 - 2), wz = -(13 + rnd() * 22);
        if (probe(wx, wz, 3.0, 0.3, true) && free(wx, wz, 6.5) && room(400)) {
          bWindmill(c, wx, pr.hi, pr.lo, wz, (rnd() - 0.5) * 0.5);
          occ.push(wx, wz, 6.5);
          break;
        }
      }
    }

    function tryLantern(lx, lz, needFree) {
      if (!probe(lx, lz, 0.6, 0.7) || (needFree && !free(lx, lz, 0.9)) || !room(100)) return;
      bLantern(c, lx, pr.c, lz, 0.9 + rnd() * 0.3);
      occ.push(lx, lz, 0.8);
    }

    // ---- 村落：房屋聚成一簇，周围配干草堆/栅栏/灯笼 ----
    var kinds = [];
    if (has.house) kinds.push('house');
    if (has.cabin) kinds.push('cabin');
    if (has.hut) kinds.push('hut');
    var nVill = 0;
    if (kinds.length && bx1 - bx0 > 8) {
      nVill = rnd() < (qual === 0 ? 0.6 : 0.85) ? 1 : 0;
      if (qual > 0 && span > 40 && rnd() < 0.3) nVill++;
    }
    for (var vi = 0; vi < nVill; vi++) {
      var vx = 0, vz = 0, ok = false;
      for (t = 0; t < 14 && !ok; t++) {
        vx = bx0 + rnd() * (bx1 - bx0);
        vz = -(11 + rnd() * 22);
        ok = probe(vx, vz, 4, 0.3, true) && free(vx, vz, 5);
      }
      if (!ok) continue;
      var nb = qual === 0 ? 1 + ((rnd() * 2) | 0) : 2 + ((rnd() * 3) | 0);
      for (var b = 0; b < nb; b++) {
        for (t2 = 0; t2 < 8; t2++) {
          var ang = rnd() * 6.283, dd = 2 + rnd() * 8, hx = vx + Math.cos(ang) * dd * 1.3, hz = vz + Math.sin(ang) * dd * 0.75;
          if (hx < bx0 || hx > bx1 || hz > -8 || hz < -46) continue;
          if (!probe(hx, hz, 2.7, 0.3, true) || !free(hx, hz, 4.4) || !room(130)) continue;
          var kind = pick(kinds, rnd), yaw = (rnd() - 0.5) * 0.9;
          (kind === 'cabin' ? bCabin : kind === 'hut' ? bHut : bHouse)(c, hx, pr.hi, pr.lo, hz, yaw);
          occ.push(hx, hz, 4.4);
          if (has.lantern && rnd() < 0.7) tryLantern(hx + (rnd() < 0.5 ? -3.7 : 3.7), hz + 1.6, false);
          break;
        }
      }
      if (has.haystack) {
        var nh = qual === 0 ? 1 : 1 + ((rnd() * 3) | 0);
        for (var h = 0; h < nh; h++) {
          for (t2 = 0; t2 < 8; t2++) {
            var sx = vx + (rnd() - 0.5) * 22, sz = vz + (rnd() - 0.5) * 12;
            if (sx < bx0 || sx > bx1 || sz > -7.5 || !probe(sx, sz, 1.5, 0.5) || !free(sx, sz, 2.0) || !room(140)) continue;
            if (rnd() < 0.35) bBale(c, sx, pr.c, sz, 1); else bHay(c, sx, pr.c, sz, 0.9 + rnd() * 0.3);
            occ.push(sx, sz, 2.0);
            break;
          }
        }
      }
      if (has.fence) fenceRun(vx - 6 + rnd() * 4, Math.min(vz + 5 + rnd() * 2, -7), 5 + ((rnd() * 4) | 0), false);
      if (has.lantern && !has.cabin) tryLantern(vx + (rnd() - 0.5) * 8, Math.min(vz + 3, -7), true);
    }

    // ---- 散布的大件：岩石、沙滩伞、独立石灯笼、薰衣草田 ----
    if (has.rock) {
      var nR = qual === 0 ? 1 : 2 + ((rnd() * 3) | 0);
      for (i = 0; i < nR; i++) {
        for (t = 0; t < 10; t++) {
          var rs = 0.9 + rnd() * 1.5, rx = xa + 2 + rnd() * (span - 4), rz = -(7 + Math.pow(rnd(), 1.3) * 38);
          if (!probe(rx, rz, rs * 1.2, 0.9) || !free(rx, rz, rs * 1.3) || !room(120)) continue;
          bRock(c, rx, pr.lo, rz, rs, true);
          occ.push(rx, rz, rs * 1.3);
          if (rnd() < 0.2 && room(60)) bRock(c, rx + rs * 1.1, pr.lo, rz + rs * 0.5, rs * 0.5, false);
          break;
        }
      }
    }
    if (has.umbrella && rnd() < (qual === 0 ? 0.4 : 0.7)) {
      for (t = 0; t < 20; t++) {
        var ux = xa + 2 + rnd() * (span - 4), uz = -(7 + rnd() * 27);
        if (!probe(ux, uz, 1.6, 0.35) || !free(ux, uz, 2.2) || !room(80)) continue;
        if (t < 12 && pr.c > 4) continue; // 优先靠近水边的低地
        bUmbrella(c, ux, pr.c, uz);
        occ.push(ux, uz, 2.2);
        break;
      }
    }
    if (has.lantern && rnd() < 0.6) tryLantern(xa + 2 + rnd() * (span - 4), -(8 + rnd() * 14), true);
    if (has.lavender && span > 14) {
      var nF = qual === 0 ? 1 : (rnd() < 0.5 ? 2 : 1);
      for (i = 0; i < nF; i++) {
        for (t = 0; t < 10; t++) {
          var flen = 7 + rnd() * 7, fx = xa + flen * 0.5 + 1 + rnd() * Math.max(0, span - flen - 2), fz = -(9 + rnd() * 16);
          if (!probe(fx, fz, 6, 0.45) || !free(fx, fz, flen * 0.5 + 0.5)) continue;
          lavField(fx, fz, flen, qual === 0 ? 2 : 3 + ((rnd() * 2) | 0), false);
          break;
        }
      }
    }

    // ---- 树：70% 成林（林心按距离加权），30% 零星散布；离赛道越远越稀 ----
    var nTrees = Math.round(span / 48 * 34 * td * QF), ng = 2 + (rnd() < 0.5 ? 1 : 0), groves = [];
    for (i = 0; i < ng; i++) groves.push({ x: xa + 2 + rnd() * (span - 4), z: -(8 + 46 * Math.pow(rnd(), 1.3)), r: 4 + rnd() * 6 });
    var placed = 0, treeCap = cap * 0.8;
    for (var att = 0; placed < nTrees && att < nTrees * 6; att++) {
      var tx, tz;
      if (rnd() < 0.7) {
        var gv = groves[(rnd() * ng) | 0], ta = rnd() * 6.283, tr = gv.r * Math.sqrt(rnd());
        tx = gv.x + Math.cos(ta) * tr * 1.4; tz = gv.z + Math.sin(ta) * tr * 0.8;
      } else {
        tx = xa + 1.2 + rnd() * (span - 2.4); tz = -(6.5 + 55 * Math.pow(rnd(), 1.5));
      }
      if (tx < xa + 1.2 || tx > xb - 1.2 || tz < -62) continue;
      if (rnd() > 0.22 + 0.78 * Math.max(0, 1 - (-tz - 6) / 56)) continue;
      var r1 = rnd(), far = 1 + 0.25 * (-tz - 6) / 56, H;
      switch (treeType) {
        case 'orange': H = 3 + r1 * 2.5; break;
        case 'maple': H = 4.5 + r1 * 3.5; break;
        case 'birch': H = 4 + r1 * 3; break;
        case 'pine': H = 5 + Math.pow(r1, 1.2) * 4; break;
        case 'palm': H = 5 + r1 * 3; break;
        default: H = 3.5 + Math.pow(r1, 1.3) * 4;
      }
      H *= far;
      if (tz > -Math.max(6, 4.2 + treeR(treeType, H))) continue; // 树冠不得侵入 |z|<4
      if (!probe(tx, tz, 0.8, 0.9) || !free(tx, tz, 0.9 + 0.03 * H)) continue;
      if (A.tris + W.tris + 90 > treeCap) break;
      switch (treeType) {
        case 'orange': tRound(c, tx, pr.lo, tz, H, true); break;
        case 'maple': tMaple(c, tx, pr.lo, tz, H); break;
        case 'birch': tBirch(c, tx, pr.lo, tz, H); break;
        case 'pine': tPine(c, tx, pr.lo, tz, H); break;
        case 'palm': tPalm(c, tx, pr.lo, tz, H); break;
        default: tRound(c, tx, pr.lo, tz, H, false);
      }
      occ.push(tx, tz, 0.9 + 0.03 * H);
      placed++;
    }

    // ---- 赛道前方(z>0)：只放 ≤1.2m 的低矮装饰，且地面须比赛道低 1.5m 以上 ----
    var nFront = Math.round(span / 48 * 26 * QF);
    for (var fi = 0, fa = 0; fi < nFront && fa < nFront * 4; fa++) {
      var qx = xa + 1.5 + rnd() * (span - 3), qz = 5.6 + rnd() * 10.4, kr = rnd();
      if (!frontOK(qx, qz)) continue;
      if (!room(40)) break;
      var qy = hAt(qx, qz);
      if (kr < 0.5 || (kr >= 0.85 && !has.rock)) tuft(qx, qy, qz);
      else if (kr < 0.85) flowerPatch(qx, qz, 1.0);
      else bRock(c, qx, qy, qz, 0.3 + rnd() * 0.4, false);
      fi++;
    }
    if (has.fence && rnd() < 0.7 && span > 14) fenceRun(xa + 2 + rnd() * (span - 14), 5.8 + rnd() * 3, 5 + ((rnd() * 4) | 0), true);
    if (has.lavender && qual > 0 && span > 14) lavField(xa + 6 + rnd() * (span - 12), 7 + rnd() * 5, 6, 2, true);

    // ---- 赛道后方的花丛与草丛 ----
    var nFl = Math.round(span / 48 * (has.flowers ? 16 : 8) * QF);
    for (i = 0; i < nFl; i++) {
      var ppx = xa + 1.5 + rnd() * (span - 3), ppz = -(5.8 + rnd() * 20);
      if (!probe(ppx, ppz, 1.2, 0.6) || !free(ppx, ppz, 1.2) || !room(70)) continue;
      flowerPatch(ppx, ppz, 1.2);
    }
    var nTuft = Math.round(span / 48 * 20 * QF);
    for (i = 0; i < nTuft; i++) {
      var gx = xa + 1.2 + rnd() * (span - 2.4), gz = -(5.2 + rnd() * 25);
      if (!probe(gx, gz, 0.4, 0.9) || !free(gx, gz, 0.5) || !room(20)) continue;
      tuft(gx, pr.c, gz);
    }

    // ---- 合并输出：普通网格 + 发光网格 ----
    var ga = A.build();
    if (ga) {
      geos.push(ga);
      var ma = new THREE.Mesh(ga, lib.matLit);
      ma.castShadow = qual > 0;
      group.add(ma);
    }
    var gw = W.build();
    if (gw) {
      geos.push(gw);
      group.add(new THREE.Mesh(gw, lib.matGlow));
    }
    res.triangles = A.tris + W.tris;
    return res;
  }

  // ---------- 环境物件的几何 ----------
  function fanSrc(a, b, c, bulge) { // 带鼓起的双面三角帆
    var out = [], m = [(a[0] + b[0] + c[0]) / 3, (a[1] + b[1] + c[1]) / 3, bulge];
    tri2(out, a, b, m); tri2(out, b, c, m); tri2(out, c, a, m);
    return rawSrc(out);
  }

  function makeBoatGeo(lib, hullHex, jibHex) {
    var A = new Acc(lib), G = lib.g, C = lib.C;
    var sailMain = fanSrc([0, 0, 0], [0, 1, 0], [-1, 0, 0], 0.16), sailJib = fanSrc([0, 0, 0], [0, 1, 0], [1, 0, 0], 0.12);
    A.grad(0.82, 1.06);
    A.part(G.box, C(hullHex), -0.1, 0, 0, 2.6, 0.55, 1.0);
    A.part(G.pyr, C(hullHex), 1.2, 0.275, 0, 0.275, 1.0, 0.5, 0, -Math.PI / 2); // 船头：尖端朝 +x
    A.grad(0.95, 1.05);
    A.part(G.box, C('#f1e6cf'), -0.1, 0.55, 0, 2.62, 0.08, 1.04);
    A.part(G.box, C('#f1e6cf'), -0.7, 0.63, 0, 0.8, 0.37, 0.6);
    A.part(G.cyl6, C('#6b4a33'), 0.2, 0.55, 0, 0.05, 3.3, 0.05);
    A.grad(1, 1);
    A.part(sailMain, C('#f6efe0'), 0.25, 0.75, 0, 1.4, 2.5, 1);
    A.part(sailJib, C(jibHex), 0.4, 0.8, 0, 1.6, 2.1, 1);
    A.part(sailJib, C('#d8483c'), 0.22, 3.55, 0, 0.5, 0.25, 1); // 桅顶小旗
    return A.build();
  }

  function makeBalloonGeo(lib, pal) { // 旋转体气囊 + 按经线分段的条纹，吊篮与吊绳
    var THREE = lib.THREE, G = lib.g, C = lib.C, segs = 12, i, prof = [[0.3, -1.1], [0.62, -0.9], [0.9, -0.5], [1.0, 0], [0.98, 0.5], [0.78, 0.95], [0.42, 1.22], [0.001, 1.35]], pts = [];
    for (i = 0; i < prof.length; i++) pts.push(new THREE.Vector2(prof[i][0], prof[i][1]));
    var g = new THREE.LatheGeometry(pts, segs).toNonIndexed(), p = g.attributes.position.array;
    var pos = new Float32Array(p), cols = new Float32Array(p.length), score = 0, mn = Infinity, mx = -Infinity;
    for (i = 0; i < pos.length; i += 9) { // 统计朝向：外法线与径向同向才是正面，否则整体翻转绕序
      var ux = pos[i + 3] - pos[i], uy = pos[i + 4] - pos[i + 1], uz = pos[i + 5] - pos[i + 2];
      var vx = pos[i + 6] - pos[i], vy = pos[i + 7] - pos[i + 1], vz = pos[i + 8] - pos[i + 2];
      var nx = uy * vz - uz * vy, nz = ux * vy - uy * vx;
      score += nx * (pos[i] + pos[i + 3] + pos[i + 6]) + nz * (pos[i + 2] + pos[i + 5] + pos[i + 8]);
    }
    if (score < 0) {
      for (i = 0; i < pos.length; i += 9) {
        var t0 = pos[i + 3], t1 = pos[i + 4], t2 = pos[i + 5];
        pos[i + 3] = pos[i + 6]; pos[i + 4] = pos[i + 7]; pos[i + 5] = pos[i + 8];
        pos[i + 6] = t0; pos[i + 7] = t1; pos[i + 8] = t2;
      }
    }
    for (i = 0; i < pos.length; i += 9) {
      var cx = pos[i] + pos[i + 3] + pos[i + 6], cz = pos[i + 2] + pos[i + 5] + pos[i + 8];
      var seg = Math.floor((Math.atan2(cz, cx) / (Math.PI * 2) + 1) * segs) % segs, col = C(pal[seg % pal.length]);
      for (var k = 0; k < 9; k += 3) { cols[i + k] = col.r; cols[i + k + 1] = col.g; cols[i + k + 2] = col.b; }
    }
    for (i = 1; i < pos.length; i += 3) { if (pos[i] < mn) mn = pos[i]; if (pos[i] > mx) mx = pos[i]; }
    g.dispose();
    var A = new Acc(lib);
    A.grad(0.78, 1.1);
    A.part({ pos: pos, minY: mn, maxY: mx, cols: cols }, null, 0, 0, 0, 1, 1, 1);
    A.grad(0.9, 1.0);
    A.part(G.box, C('#8a5a3c'), 0, -1.78, 0, 0.32, 0.24, 0.32);
    var rope = C('#5a4634');
    for (i = 0; i < 4; i++) {
      var sx = i & 1 ? 1 : -1, sz = i & 2 ? 1 : -1;
      A.seg(G.rod, rope, sx * 0.14, -1.54, sz * 0.14, sx * 0.22, -1.08, sz * 0.22, 0.012);
    }
    return A.build();
  }

  // ---------- 环境：帆船、热气球、鸟群、萤火虫（围绕相机循环复用）----------
  function createAmbient(lib, scene, heightAt) {
    var THREE = lib.THREE, qual = lib.quality, hAt = heightAt, C = lib.C, i, k;
    var rnd = mulberry32(0x5EED1E55);
    var root = new THREE.Group();
    root.name = 'glider-ambient';
    scene.add(root);
    var ownGeos = [], ownMats = [], ownTex = [];
    var _m = new THREE.Matrix4(), _q = new THREE.Quaternion(), _p = new THREE.Vector3(), _s = new THREE.Vector3(), _up = new THREE.Vector3(0, 1, 0);

    // 帆船：只出现在赛道后方的海面（海峡与远岸）
    var hulls = ['#8a5a3c', '#f1e6cf', '#3f6f8f', '#a3443c'], jibs = ['#d8483c', '#2f8f9a', '#f0a53a', '#4a6fa5', '#d9638f'];
    var boats = [], nBoats = [2, 3, 5][qual], boatGeos = [];
    for (i = 0; i < Math.min(4, nBoats); i++) { var bg = makeBoatGeo(lib, hulls[i % 4], jibs[i % 5]); boatGeos.push(bg); ownGeos.push(bg); }
    for (i = 0; i < nBoats; i++) {
      var bm = new THREE.Mesh(boatGeos[i % boatGeos.length], lib.matLit), lamp = new THREE.Mesh(lib.glowGeo, lib.matGlow);
      lamp.position.set(-0.7, 1.15, 0); // 舱顶桅灯，夜里发光
      bm.add(lamp);
      bm.visible = false;
      root.add(bm);
      boats.push({ mesh: bm, x: 0, z: 0, vx: 0, ph: rnd() * 6.28, ready: false });
    }
    function spawnBoat(b, cx, first) {
      for (var n = 0; n < 8; n++) {
        // Only behind the track (straits and the far coast): boats between the track and the camera would hide the bird.
        var x = first ? cx - 50 + rnd() * 200 : cx + 70 + rnd() * 90, z = -(30 + rnd() * 90);
        if (!(hAt(x, z) < 0.2)) continue;
        var dir = rnd() < 0.5 ? 1 : -1;
        b.x = x; b.z = z; b.vx = dir * (0.4 + rnd() * 0.6);
        b.mesh.scale.setScalar(2 + -z / 60 + rnd() * 0.8);
        b.mesh.rotation.y = dir > 0 ? 0 : Math.PI;
        b.mesh.visible = b.ready = true;
        return;
      }
      b.ready = b.mesh.visible = false;
    }

    // 热气球：远在岛屿后方，跟随相机 ~90%，视差很小，缓慢漂移
    var balloons = [], nBal = [1, 2, 3][qual];
    var pals = [['#e0523f', '#f6efe0'], ['#2f8f9a', '#f0a53a', '#f6efe0'], ['#d9638f', '#f6efe0', '#6a5acd'], ['#f0a53a', '#c9452b']];
    for (i = 0; i < nBal; i++) {
      var ballg = makeBalloonGeo(lib, pals[i % 4]);
      ownGeos.push(ballg);
      var mesh = new THREE.Mesh(ballg, lib.matLit), flame = new THREE.Mesh(lib.glowGeo, lib.matGlow);
      flame.position.set(0, -1.2, 0);
      flame.scale.setScalar(0.3);
      mesh.add(flame);
      root.add(mesh);
      balloons.push({ mesh: mesh, x: 0, y: 34 + rnd() * 26, z: -(140 + rnd() * 50), s: 4.5 + rnd() * 2, drift: 0.6 + rnd() * 0.8, ph: rnd() * 6.28 });
      mesh.scale.setScalar(balloons[i].s);
    }

    // 鸟群：InstancedMesh，V 字编队，靠缩放 y 扑翅
    var flocks = [], nFlock = [1, 2, 2][qual], nPer = [5, 7, 9][qual], inst = null;
    if (nFlock) {
      var bird = [];
      tri2(bird, [0.3, 0, 0], [-0.15, 0.45, 1.0], [-0.25, 0, 0]);
      tri2(bird, [0.3, 0, 0], [-0.25, 0, 0], [-0.15, 0.45, -1.0]);
      var birdGeo = new THREE.BufferGeometry();
      birdGeo.setAttribute('position', new THREE.BufferAttribute(new Float32Array(bird), 3));
      var birdMat = new THREE.MeshBasicMaterial({ color: '#2b2a3a' });
      ownGeos.push(birdGeo); ownMats.push(birdMat);
      inst = new THREE.InstancedMesh(birdGeo, birdMat, nFlock * nPer);
      inst.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
      inst.frustumCulled = false;
      root.add(inst);
      for (i = 0; i < nFlock; i++) {
        var f = { x: 0, y: 24 + rnd() * 14, z: -(45 + rnd() * 30), s: 1.0 + rnd() * 0.4, drift: 3 + rnd() * 2, ox: [], oy: [], oz: [], ph: [] };
        for (k = 0; k < nPer; k++) {
          var rank = (k + 1) >> 1, side = k & 1 ? 1 : -1;
          f.ox.push(-rank * 1.9 + (rnd() - 0.5) * 0.6); f.oy.push((rnd() - 0.5) * 0.6); f.oz.push(side * rank * 1.5); f.ph.push(rnd() * 6.28);
        }
        flocks.push(f);
      }
    }

    // 萤火虫：相机前后一小片范围，夜里出现，颜色闪烁模拟明灭
    var nFF = [10, 26, 54][qual], ffBase = new Float32Array(nFF * 3), ffPos = new Float32Array(nFF * 3), ffCol = new Float32Array(nFF * 3), ffPh = new Float32Array(nFF * 2);
    var ffGeo = new THREE.BufferGeometry(), posAttr = new THREE.BufferAttribute(ffPos, 3), colAttr = new THREE.BufferAttribute(ffCol, 3);
    posAttr.setUsage(THREE.DynamicDrawUsage); colAttr.setUsage(THREE.DynamicDrawUsage);
    ffGeo.setAttribute('position', posAttr); ffGeo.setAttribute('color', colAttr);
    var ffOpts = { size: 0.4, sizeAttenuation: true, vertexColors: true, transparent: true, opacity: 0, depthWrite: false, blending: THREE.AdditiveBlending };
    if (typeof document !== 'undefined') { // 柔边圆点贴图
      var cv = document.createElement('canvas');
      cv.width = cv.height = 32;
      var cx2 = cv.getContext('2d'), gr = cx2.createRadialGradient(16, 16, 0, 16, 16, 16);
      gr.addColorStop(0, 'rgba(255,255,255,1)'); gr.addColorStop(0.4, 'rgba(255,255,255,0.45)'); gr.addColorStop(1, 'rgba(255,255,255,0)');
      cx2.fillStyle = gr; cx2.fillRect(0, 0, 32, 32);
      var tex = new THREE.CanvasTexture(cv);
      ownTex.push(tex);
      ffOpts.map = tex;
    }
    var ffMat = new THREE.PointsMaterial(ffOpts), pts = new THREE.Points(ffGeo, ffMat), ffc = C('#ffe08a');
    pts.frustumCulled = false; pts.visible = false;
    ownGeos.push(ffGeo); ownMats.push(ffMat);
    if (nFF) root.add(pts);
    function respawnFF(n, cx) {
      var x = cx - 15 + rnd() * 75, z = -12 + rnd() * 22, gh = hAt(x, z);
      if (!(gh >= 0.3)) { z = -6 - rnd() * 8; gh = hAt(x, z); if (!(gh >= 0.3)) gh = 0.5; }
      ffBase[n * 3] = x; ffBase[n * 3 + 1] = gh + 0.6 + rnd() * 3.5; ffBase[n * 3 + 2] = z;
    }

    var inited = false, lastCx = 0;
    return {
      update: function (t, dt, cameraX, env) {
        lib.setEnv(env);
        dt = clamp(dt || 0, 0, 0.1);
        var cx = cameraX || 0, dcx = inited ? cx - lastCx : 0, n = lib.night, j;
        if (!inited) {
          for (j = 0; j < boats.length; j++) spawnBoat(boats[j], cx, true);
          for (j = 0; j < balloons.length; j++) balloons[j].x = cx - 40 + j * 110 + rnd() * 50;
          for (j = 0; j < flocks.length; j++) flocks[j].x = cx + 20 + j * 90;
          for (j = 0; j < nFF; j++) { respawnFF(j, cx); ffPh[j * 2] = rnd() * 6.28; ffPh[j * 2 + 1] = rnd() * 6.28; }
          inited = true;
        }
        lastCx = cx;

        for (j = 0; j < boats.length; j++) {
          var b = boats[j];
          if (!b.ready || b.x < cx - 70 || b.x > cx + 260) { spawnBoat(b, cx, false); if (!b.ready) continue; }
          b.x += b.vx * dt;
          b.mesh.position.set(b.x, Math.sin(t * 1.1 + b.ph) * 0.08, b.z);
          b.mesh.rotation.z = Math.sin(t * 0.9 + b.ph) * 0.05;
        }

        for (j = 0; j < balloons.length; j++) {
          var bl = balloons[j];
          bl.x += dcx * 0.9 + bl.drift * dt;
          if (bl.x - cx < -130 || bl.x - cx > 330) bl.x = cx + 150 + rnd() * 90;
          bl.mesh.position.set(bl.x, bl.y + Math.sin(t * 0.3 + bl.ph) * 1.5, bl.z);
          bl.mesh.rotation.y = t * 0.05 + bl.ph;
        }

        if (inst) {
          inst.visible = n < 0.85; // 入夜后归巢
          if (inst.visible) {
            for (j = 0; j < flocks.length; j++) {
              var fl = flocks[j];
              fl.x += dcx * 0.88 + fl.drift * dt;
              if (fl.x - cx < -120 || fl.x - cx > 300) fl.x = cx + 160 + rnd() * 60;
              for (k = 0; k < nPer; k++) {
                var ph = fl.ph[k], flap = Math.sin(t * 7 + ph * 3) * 0.9 + 0.1;
                _p.set(fl.x + fl.ox[k], fl.y + fl.oy[k] + Math.sin(t * 0.8 + ph) * 0.5, fl.z + fl.oz[k]);
                _q.setFromAxisAngle(_up, Math.sin(t * 0.5 + ph) * 0.08);
                _s.set(fl.s, fl.s * flap, fl.s);
                _m.compose(_p, _q, _s);
                inst.setMatrixAt(j * nPer + k, _m);
              }
            }
            inst.instanceMatrix.needsUpdate = true;
          }
        }

        var show = nFF > 0 && n > 0.12;
        pts.visible = show;
        if (show) {
          ffMat.opacity = clamp((n - 0.1) * 1.6, 0, 1) * 0.95;
          for (j = 0; j < nFF; j++) {
            var p0 = ffPh[j * 2], p1 = ffPh[j * 2 + 1];
            if (ffBase[j * 3] < cx - 18 || ffBase[j * 3] > cx + 90) respawnFF(j, cx);
            ffPos[j * 3] = ffBase[j * 3] + Math.sin(t * 0.5 + p0) * 0.9;
            ffPos[j * 3 + 1] = ffBase[j * 3 + 1] + Math.sin(t * 0.9 + p1) * 0.5;
            ffPos[j * 3 + 2] = ffBase[j * 3 + 2] + Math.cos(t * 0.4 + p0) * 0.9;
            var blink = 0.3 + 0.7 * Math.max(0, Math.sin(t * (1.3 + (p1 % 1)) + p0 * 5));
            ffCol[j * 3] = ffc.r * blink; ffCol[j * 3 + 1] = ffc.g * blink; ffCol[j * 3 + 2] = ffc.b * blink;
          }
          posAttr.needsUpdate = true;
          colAttr.needsUpdate = true;
        }
      },
      dispose: function () {
        scene.remove(root);
        for (var d = 0; d < ownGeos.length; d++) ownGeos[d].dispose();
        for (d = 0; d < ownMats.length; d++) ownMats[d].dispose();
        for (d = 0; d < ownTex.length; d++) ownTex[d].dispose();
        if (inst) inst.dispose();
        while (root.children.length) root.remove(root.children[0]);
      }
    };
  }

  globalThis.XiabanGliderProps = {
    createLibrary: createLibrary,
    buildChunk: buildChunk,
    createAmbient: createAmbient
  };
})();
