/* 暮色滑翔 · world: island heightfield chunks, sea, sky, clouds, distant ranges, sun and time of day. */
(() => {
  'use strict'

  const SEA_FLOOR = -9
  const CHUNK = 48
  const FAR_CHUNK = 320
  const DAY_FULL = 56

  const BIOMES = [
    { id: 'wheat', name: '麦浪坡', grass: ['#d9bf62', '#9fae55'], rock: '#9c8a6e', sand: '#ecd7a2', far: '#8f9a7c', mountains: 0.7, snow: false, trees: 'round', treeDensity: 0.45, flowers: '#f6e3a0', props: ['haystack', 'house', 'fence'] },
    { id: 'windmill', name: '风车原', grass: ['#9fc36a', '#5f9a52'], rock: '#8d8a78', sand: '#ead9a8', far: '#7d977f', mountains: 0.8, snow: false, trees: 'round', treeDensity: 0.5, flowers: '#f2f0e8', props: ['windmill', 'house', 'flowers'] },
    { id: 'orange', name: '橘子湾', grass: ['#b4c766', '#6f9c4e'], rock: '#a3876a', sand: '#f1d9a6', far: '#8c9a78', mountains: 0.9, snow: false, trees: 'orange', treeDensity: 0.6, flowers: '#ffb25a', props: ['house', 'lighthouse'] },
    { id: 'maple', name: '枫叶岭', grass: ['#c99a55', '#8d7444'], rock: '#7f6c5c', sand: '#e7cf9e', far: '#937d74', mountains: 1.15, snow: false, trees: 'maple', treeDensity: 0.65, flowers: '#e8643c', props: ['cabin', 'lantern'] },
    { id: 'lavender', name: '薰衣草谷', grass: ['#a9b97a', '#7c8f5f'], rock: '#8c8494', sand: '#ead9b4', far: '#8a87a0', mountains: 0.85, snow: false, trees: 'birch', treeDensity: 0.4, flowers: '#a68ad8', props: ['lavender', 'house'] },
    { id: 'pine', name: '雪松峰', grass: ['#7e9e74', '#4f7660'], rock: '#7c7f86', sand: '#e2d8c2', far: '#8c98a8', mountains: 1.45, snow: true, trees: 'pine', treeDensity: 0.7, flowers: '#f4f6f8', props: ['cabin', 'rock'] },
    { id: 'palm', name: '椰影礁', grass: ['#a8c86a', '#6aa058'], rock: '#a08f78', sand: '#f6e2b0', far: '#7fa08e', mountains: 0.45, snow: false, trees: 'palm', treeDensity: 0.4, flowers: '#ff8fa3', props: ['hut', 'rock', 'umbrella'] },
  ]

  function biomeFor(index) { return BIOMES[((index % BIOMES.length) + BIOMES.length) % BIOMES.length] }
  function islandName(index) {
    const biome = biomeFor(index)
    return index >= BIOMES.length ? biome.name + ' · 远岸' : biome.name
  }

  // Time-of-day keyframes by remaining light (1 = full afternoon). Colours are sRGB hex; mixing happens in linear space.
  const KEYS = [
    { l: -0.35, top: '#060a22', horizon: '#1d2550', sun: '#8ea6e0', sunI: 0.45, hemiSky: '#3a4a8a', hemiGround: '#1a1c30', hemiI: 0.55, fog: '#1f2650', deep: '#0b1530', alt: -12, cloud: '#56608e' },
    { l: 0, top: '#141d4c', horizon: '#6b5a8c', sun: '#d77a7e', sunI: 0.7, hemiSky: '#6a6aa8', hemiGround: '#2c2838', hemiI: 0.65, fog: '#544e80', deep: '#18234a', alt: -3, cloud: '#9c7f9c' },
    { l: 0.12, top: '#2a336f', horizon: '#e2777a', sun: '#ff6c48', sunI: 1.5, hemiSky: '#8c80b8', hemiGround: '#40303a', hemiI: 0.7, fog: '#b06c7c', deep: '#232f5c', alt: 1.5, cloud: '#f0a08c' },
    { l: 0.32, top: '#4b5aa6', horizon: '#ff9a62', sun: '#ff8a4a', sunI: 2.1, hemiSky: '#a7a6d6', hemiGround: '#5a4434', hemiI: 0.75, fog: '#eb9d7a', deep: '#2e4a78', alt: 6, cloud: '#ffc7a2' },
    { l: 0.62, top: '#6383c6', horizon: '#ffc98c', sun: '#ffcf8a', sunI: 2.5, hemiSky: '#b6c8ee', hemiGround: '#6c5a40', hemiI: 0.8, fog: '#f2c99c', deep: '#2c5c86', alt: 14, cloud: '#ffe6cc' },
    { l: 1, top: '#5b92d6', horizon: '#f4dcb4', sun: '#fff0d4', sunI: 2.7, hemiSky: '#bcd6ff', hemiGround: '#7a6c50', hemiI: 0.85, fog: '#e9d8bd', deep: '#2b6c92', alt: 24, cloud: '#ffffff' },
  ]

  // ------------------------------------------------------------ noise

  function hash(ix, iz, seed) {
    let h = (Math.imul(ix | 0, 374761393) + Math.imul(iz | 0, 668265263) + Math.imul(seed | 0, 1442695041)) | 0
    h = Math.imul(h ^ (h >>> 13), 1274126177)
    return ((h ^ (h >>> 16)) >>> 0) / 4294967296
  }
  function vnoise(x, z, seed) {
    const ix = Math.floor(x), iz = Math.floor(z)
    let fx = x - ix, fz = z - iz
    fx = fx * fx * (3 - 2 * fx); fz = fz * fz * (3 - 2 * fz)
    const a = hash(ix, iz, seed), b = hash(ix + 1, iz, seed), c = hash(ix, iz + 1, seed), d = hash(ix + 1, iz + 1, seed)
    return a + (b - a) * fx + (c - a) * fz + (a - b - c + d) * fx * fz
  }
  function fbm(x, z, seed) {
    return vnoise(x, z, seed) * 0.57 + vnoise(x * 2.03 + 17.1, z * 2.03 - 9.4, seed + 1) * 0.29 + vnoise(x * 4.1 - 3.3, z * 4.1 + 5.7, seed + 2) * 0.14
  }
  const clamp01 = (v) => v < 0 ? 0 : v > 1 ? 1 : v
  const smooth = (a, b, v) => { const t = clamp01((v - a) / (b - a)); return t * t * (3 - 2 * t) }
  const lerp = (a, b, t) => a + (b - a) * t

  // ------------------------------------------------------------ world

  function create(THREE, options) {
    const { scene, rules, props } = options
    let state = options.state
    let quality = options.quality
    const seedOf = () => state.seed % 100000
    const tmpColor = new THREE.Color(), tmpColor2 = new THREE.Color()
    const lib = props ? props.createLibrary(THREE, { quality }) : null

    // ---------------------------------------------- height field

    function heightAt(x, z) {
      const tr = rules.terrain(state, x).h
      const mask = smooth(-7, 1.6, tr)
      const seed = seedOf()
      if (z >= 0) {
        // The track's hills rise out of a low meadow that runs down to a beach in front of the camera.
        const reach = 24 + 6 * vnoise(x * 0.025, 3.7, seed + 7)
        const meadow = lerp(SEA_FLOOR, 2.2 + 1.6 * fbm(x * 0.05, z * 0.08, seed + 11), mask)
        const low = lerp(meadow, SEA_FLOOR, Math.pow(smooth(13, reach, z), 1.3))
        return lerp(tr, low, smooth(1.4, 10, z))
      }
      const t = -z
      const biome = biomeFor(rules.islandAt(state, x).index)
      // Flat across the track line itself (|z| < 1.5), so guides laid on it sit exactly on the ground.
      const follow = Math.exp(-Math.max(0, t - 1.5) / 7)
      const rolling = fbm(x * 0.017, t * 0.021, seed + 3)
      const ridge = 1 - Math.abs(2 * fbm(x * 0.011 + 40, t * 0.013, seed + 5) - 1)
      // Keep the near backdrop lower than the track's peaks; the taller range sits further back so the sky shows.
      const band = smooth(26, 46, t) * (1 - smooth(60, 76, t))
      const hills = 3 + 7 * rolling + band * (5 + 18 * ridge * ridge) * biome.mountains
      let h = tr * follow + lerp(SEA_FLOOR, hills, mask) * (1 - follow)
      const coast = smooth(60, 86, t + 14 * (vnoise(x * 0.02, 9.1, seed + 9) - 0.5))
      return lerp(h, SEA_FLOOR - 5, coast)
    }

    function farHeight(x, z) {
      const t = -z
      const seed = seedOf() + 50
      const presence = smooth(0.32, 0.62, fbm(x * 0.0026, t * 0.004, seed))
      const ridge = 1 - Math.abs(2 * fbm(x * 0.006, t * 0.008, seed + 1) - 1)
      const body = smooth(110, 170, t) * (1 - smooth(330, 420, t))
      return -12 + presence * body * (16 + 42 * ridge * ridge + 10 * fbm(x * 0.02, t * 0.02, seed + 2)) * (0.6 + t / 800)
    }

    // ---------------------------------------------- terrain colours

    const palette = BIOMES.map((b) => ({
      grassLo: new THREE.Color(b.grass[0]), grassHi: new THREE.Color(b.grass[1]), rock: new THREE.Color(b.rock),
      sand: new THREE.Color(b.sand), far: new THREE.Color(b.far), flowers: new THREE.Color(b.flowers),
    }))
    const shallow = new THREE.Color('#5fb4ae'), deepFloor = new THREE.Color('#1f4f6a'), foam = new THREE.Color('#fbf3dc'), snow = new THREE.Color('#f4f6fb')

    function colorAt(out, x, y, z, ny, biomeIndex) {
      const p = palette[biomeIndex]
      const seed = seedOf()
      const n = vnoise(x * 0.11, z * 0.11, seed + 21)
      if (y < -0.35) {
        out.copy(p.sand).lerp(shallow, clamp01(-y / 2.5) * 0.6).lerp(deepFloor, clamp01((-y - 1.5) / 6))
      } else if (y < 0.55) {
        out.copy(foam)
      } else if (y < 1.6 + n * 0.9) {
        out.copy(p.sand)
      } else {
        out.copy(p.grassLo).lerp(p.grassHi, clamp01((y - 2) / 20 + (n - 0.5) * 0.5))
        // Land behind the track recedes into the island's distant tint, which separates it from the track.
        if (z < -6) out.lerp(p.far, clamp01((-z - 6) / 46) * 0.55)
        // The track hills' facing slopes stay grassy so their outline reads as one shape; rock only shows further back.
        if (ny < 0.66 && (z < -3 || z > 12)) out.lerp(p.rock, clamp01((0.66 - ny) / 0.2))
        const snowLine = BIOMES[biomeIndex].snow ? 17 + n * 6 : 33 + n * 6
        if (y > snowLine && ny > 0.6) out.lerp(snow, clamp01((y - snowLine) / 3))
      }
      const jitter = 0.94 + hash(Math.round(x * 5), Math.round(z * 5), seed + 3) * 0.1
      out.r *= jitter; out.g *= jitter; out.b *= jitter
      return out
    }

    // ---------------------------------------------- chunks

    const terrainMaterial = new THREE.MeshLambertMaterial({ vertexColors: true, flatShading: true })
    const farMaterial = new THREE.MeshLambertMaterial({ vertexColors: true, flatShading: true })
    const Z_ROWS = [26, 22, 19, 16, 13.5, 11, 9, 7.2, 5.6, 4.2, 3, 2, 1.2, 0.55, 0, -0.55, -1.2, -2, -3, -4.4, -6, -8, -10.5, -13.5, -17, -21, -25.5, -30.5, -36, -42, -48.5, -55.5, -63, -71, -79, -88, -98]
    const FAR_Z = [-100, -112, -126, -142, -160, -180, -202, -226, -252, -280, -310, -342, -376, -412, -450]
    const chunks = new Map(), farChunks = new Map()

    function buildGrid(x0, width, cells, rows, height, jitterAmp, colorFn) {
      const cols = cells + 1
      const positions = new Float32Array(cols * rows.length * 3)
      const seed = seedOf()
      let i = 0
      for (let r = 0; r < rows.length; r++) {
        for (let c = 0; c < cols; c++) {
          const gx = Math.round(x0 / width * cells) + c
          let x = x0 + c * width / cells
          let z = rows[r]
          // Shared, deterministic jitter keeps the low-poly facets irregular without cracks between chunks.
          const near = Math.abs(z) < 1.3
          if (!near) {
            x += (hash(gx, r, seed + 31) - 0.5) * jitterAmp * width / cells
            const dz = r > 0 && r < rows.length - 1 ? Math.min(Math.abs(rows[r - 1] - z), Math.abs(rows[r + 1] - z)) : 0
            z += (hash(gx, r, seed + 37) - 0.5) * jitterAmp * dz
          }
          positions[i++] = x; positions[i++] = height(x, z); positions[i++] = z
        }
      }
      const index = []
      for (let r = 0; r < rows.length - 1; r++) {
        for (let c = 0; c < cells; c++) {
          const a = r * cols + c, b = a + 1, d = a + cols, e = d + 1
          // Alternate the diagonal for a less regular triangulation.
          if ((c + r) % 2) index.push(a, b, d, b, e, d)
          else index.push(a, e, d, a, b, e)
        }
      }
      const geometry = new THREE.BufferGeometry()
      geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3))
      geometry.setIndex(index)
      geometry.computeVertexNormals()
      const normals = geometry.getAttribute('normal').array
      const colors = new Float32Array(positions.length)
      for (let v = 0; v < positions.length; v += 3) {
        colorFn(tmpColor, positions[v], positions[v + 1], positions[v + 2], normals[v + 1])
        colors[v] = tmpColor.r; colors[v + 1] = tmpColor.g; colors[v + 2] = tmpColor.b
      }
      geometry.setAttribute('color', new THREE.BufferAttribute(colors, 3))
      geometry.computeBoundingSphere()
      return geometry
    }

    function buildChunk(key) {
      const x0 = key * CHUNK
      rules.ensure(state, x0 + CHUNK + 40)
      const cells = quality > 0 ? 30 : 20
      const geometry = buildGrid(x0, CHUNK, cells, Z_ROWS, heightAt, 0.55, (out, x, y, z, ny) => {
        const isle = rules.islandAt(state, x)
        return colorAt(out, x, y, z, ny, ((isle.index % BIOMES.length) + BIOMES.length) % BIOMES.length)
      })
      const mesh = new THREE.Mesh(geometry, terrainMaterial)
      mesh.receiveShadow = quality > 0
      mesh.castShadow = quality > 0
      scene.add(mesh)
      const chunk = { key, mesh, props: null }
      if (lib) {
        const isle = rules.islandAt(state, x0 + CHUNK / 2)
        const inside = x0 + CHUNK > isle.x0 && x0 < isle.x1
        chunk.props = props.buildChunk(lib, { x0, x1: x0 + CHUNK, seed: state.seed, biome: biomeFor(isle.index), heightAt, island: inside ? isle : null })
        scene.add(chunk.props.group)
      }
      return chunk
    }

    function buildFar(key) {
      const x0 = key * FAR_CHUNK
      const geometry = buildGrid(x0, FAR_CHUNK, quality > 0 ? 28 : 18, FAR_Z, farHeight, 0.6, (out, x, y, z, ny) => {
        out.copy(palette[0].far).lerp(palette[5].far, clamp01(-z / 450))
        if (y < 0.5) out.copy(palette[0].sand)
        if (ny < 0.75) out.lerp(palette[5].rock, 0.35)
        if (y > 82 && ny > 0.55) out.lerp(snow, 0.7)
        return out
      })
      const mesh = new THREE.Mesh(geometry, farMaterial)
      scene.add(mesh)
      return { key, mesh }
    }

    function disposeChunk(chunk) {
      scene.remove(chunk.mesh)
      chunk.mesh.geometry.dispose()
      if (chunk.props) { scene.remove(chunk.props.group); chunk.props.dispose() }
    }

    function stream(map, size, from, to, build, budget) {
      const k0 = Math.floor(from / size), k1 = Math.floor(to / size)
      for (const [key, chunk] of map) if (key < k0 - 1 || key > k1 + 1) { disposeChunk(chunk); map.delete(key) }
      // Nearest missing chunk first, a limited number per frame to avoid hitches.
      const missing = []
      for (let k = k0; k <= k1; k++) if (!map.has(k)) missing.push(k)
      const centre = (k0 + k1) / 2
      missing.sort((a, b) => Math.abs(a - centre) - Math.abs(b - centre))
      for (let i = 0; i < Math.min(budget, missing.length); i++) map.set(missing[i], build(missing[i]))
    }

    // ---------------------------------------------- sky

    const skyUniforms = {
      top: { value: new THREE.Color() }, horizon: { value: new THREE.Color() }, sunColor: { value: new THREE.Color() },
      sunDir: { value: new THREE.Vector3(0, 0.3, -1) }, moonDir: { value: new THREE.Vector3(-0.5, 0.4, -0.8).normalize() },
      stars: { value: 0 }, moon: { value: 0 }, time: { value: 0 },
    }
    const sky = new THREE.Mesh(new THREE.SphereGeometry(1400, 32, 16), new THREE.ShaderMaterial({
      uniforms: skyUniforms, side: THREE.BackSide, depthWrite: false, fog: false,
      vertexShader: `
        varying vec3 vDir;
        void main() {
          vDir = normalize((modelMatrix * vec4(position, 1.0)).xyz - cameraPosition);
          gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
          gl_Position.z = gl_Position.w;
        }`,
      fragmentShader: `
        uniform vec3 top, horizon, sunColor, sunDir, moonDir;
        uniform float stars, moon, time;
        varying vec3 vDir;
        float h21(vec2 p) { p = fract(p * vec2(123.34, 456.21)); p += dot(p, p + 45.32); return fract(p.x * p.y); }
        void main() {
          vec3 d = normalize(vDir);
          float up = clamp(d.y, -0.2, 1.0);
          vec3 col = mix(horizon, top, pow(clamp(up * 1.25, 0.0, 1.0), 0.62));
          col = mix(col, horizon * 0.92, smoothstep(0.0, -0.2, d.y));
          float s = max(dot(d, sunDir), 0.0);
          // Warm halo around the sun, flattened along the horizon.
          float band = exp(-abs(d.y - sunDir.y) * 7.0);
          col += sunColor * (pow(s, 8.0) * 0.55 + pow(s, 64.0) * 0.9 + band * pow(s, 2.0) * 0.28);
          col = mix(col, sunColor * 1.7 + vec3(0.25), smoothstep(0.9994, 0.9997, s));
          if (stars > 0.0) {
            vec2 uv = vec2(atan(d.z, d.x) * 160.0, asin(clamp(d.y, -1.0, 1.0)) * 160.0);
            vec2 cell = floor(uv);
            float h = h21(cell);
            vec2 f = fract(uv) - 0.5 - (vec2(h21(cell + 7.1), h21(cell + 3.7)) - 0.5) * 0.6;
            float star = step(0.986, h) * smoothstep(0.16, 0.0, length(f)) * (0.6 + 0.4 * sin(time * (1.5 + h * 3.0) + h * 40.0));
            col += vec3(0.95, 0.92, 1.0) * star * stars * smoothstep(0.02, 0.25, d.y) * 1.6;
          }
          if (moon > 0.0) {
            float m = dot(d, moonDir);
            col += vec3(0.86, 0.9, 1.0) * (smoothstep(0.99955, 0.9997, m) * 1.2 + pow(max(m, 0.0), 90.0) * 0.18) * moon;
          }
          gl_FragColor = vec4(col, 1.0);
          #include <tonemapping_fragment>
          #include <colorspace_fragment>
        }`,
    }))
    sky.frustumCulled = false
    sky.renderOrder = -10
    scene.add(sky)

    // ---------------------------------------------- sea

    const seaUniforms = THREE.UniformsUtils.merge([THREE.UniformsLib.fog, {
      time: { value: 0 }, sunDir: { value: new THREE.Vector3() }, sunColor: { value: new THREE.Color() },
      deep: { value: new THREE.Color() }, top: { value: new THREE.Color() }, horizon: { value: new THREE.Color() }, sunI: { value: 1 },
    }])
    const sea = new THREE.Mesh(new THREE.PlaneGeometry(2600, 1400, 1, 1).rotateX(-Math.PI / 2), new THREE.ShaderMaterial({
      uniforms: seaUniforms, fog: true, transparent: true,
      vertexShader: `
        varying vec3 vWorld;
        #include <fog_pars_vertex>
        void main() {
          vec4 world = modelMatrix * vec4(position, 1.0);
          vWorld = world.xyz;
          vec4 mvPosition = viewMatrix * world;
          gl_Position = projectionMatrix * mvPosition;
          #include <fog_vertex>
        }`,
      fragmentShader: `
        uniform float time, sunI;
        uniform vec3 sunDir, sunColor, deep, top, horizon;
        varying vec3 vWorld;
        #include <fog_pars_fragment>
        void main() {
          vec2 p = vWorld.xz;
          float t = time;
          float dist = length(cameraPosition - vWorld);
          // Fine ripples fade out with distance so far water stays calm instead of shimmering into noise.
          float fine = 1.0 - smoothstep(60.0, 260.0, dist);
          vec3 n = normalize(vec3(
            0.05 * cos(p.x * 0.11 + p.y * 0.05 + t * 0.8) + 0.035 * cos(p.x * 0.27 - p.y * 0.17 + t * 1.3) + fine * 0.025 * cos(dot(p, vec2(0.71, 0.43)) + t * 2.1),
            1.0,
            0.05 * cos(p.y * 0.13 - p.x * 0.04 + t * 0.7) + 0.035 * cos(p.y * 0.31 + p.x * 0.12 + t * 1.2) + fine * 0.025 * cos(dot(p, vec2(-0.38, 0.82)) + t * 1.9)));
          vec3 V = normalize(cameraPosition - vWorld);
          float fres = 0.06 + 0.9 * pow(1.0 - max(dot(n, V), 0.0), 4.0);
          vec3 R = reflect(-V, n);
          vec3 skyCol = mix(horizon, top, clamp(R.y * 1.4, 0.0, 1.0));
          vec3 col = mix(deep, skyCol, fres);
          float s = max(dot(R, sunDir), 0.0);
          col += sunColor * (pow(s, 380.0) * 5.0 + pow(s, 40.0) * 0.45 + pow(s, 8.0) * 0.12) * sunI;
          float alpha = clamp(0.8 + fres, 0.0, 1.0);
          gl_FragColor = vec4(col, alpha);
          #include <tonemapping_fragment>
          #include <colorspace_fragment>
          #include <fog_fragment>
        }`,
    }))
    sea.renderOrder = 1
    scene.add(sea)

    // ---------------------------------------------- clouds

    const cloudMaterial = new THREE.MeshLambertMaterial({ color: 0xffffff, flatShading: true, emissive: new THREE.Color('#ffffff'), emissiveIntensity: 0.18 })
    const cloudGeometries = []
    for (let g = 0; g < 4; g++) {
      const parts = []
      const puffs = 4 + g
      for (let i = 0; i < puffs; i++) {
        const r = 1.6 + hash(g, i, 3) * 2.2 + (i === Math.floor(puffs / 2) ? 1.2 : 0)
        const puff = new THREE.IcosahedronGeometry(r, 1)
        puff.scale(1.15, 0.72, 0.9)
        puff.translate((i - puffs / 2) * 2.3 + hash(g, i, 5) * 1.2, hash(g, i, 7) * 1.4 - (Math.abs(i - puffs / 2) * 0.35), (hash(g, i, 9) - 0.5) * 2.4)
        parts.push(puff)
      }
      const merged = THREE.mergeGeometries(parts)
      for (const part of parts) part.dispose()
      merged.translate(0, 1.2, 0)
      cloudGeometries.push(merged)
    }
    const clouds = []
    const CLOUD_SPAN = 760
    for (let i = 0; i < 26; i++) {
      const mesh = new THREE.Mesh(cloudGeometries[i % cloudGeometries.length], cloudMaterial)
      const near = i < 8
      const scale = near ? 1.3 + hash(i, 1, 2) * 0.9 : 2.6 + hash(i, 1, 2) * 3.2
      mesh.scale.set(scale, scale * (0.8 + hash(i, 3, 4) * 0.4), scale)
      mesh.userData = {
        offset: hash(i, 5, 6) * CLOUD_SPAN, y: near ? 32 + hash(i, 7, 8) * 22 : 55 + hash(i, 7, 8) * 90,
        z: near ? -8 - hash(i, 9, 10) * 26 : -90 - hash(i, 9, 10) * 280, drift: 0.6 + hash(i, 11, 12) * 1.2,
      }
      mesh.castShadow = false
      scene.add(mesh)
      clouds.push(mesh)
    }

    // ---------------------------------------------- lights

    const sun = new THREE.DirectionalLight(0xffffff, 2.5)
    const hemi = new THREE.HemisphereLight(0xbcd6ff, 0x7a6c50, 0.8)
    // The sun sits behind the islands, so a soft sky-tinted fill from the camera side keeps the facing slopes readable.
    const fill = new THREE.DirectionalLight(0xffffff, 0.8)
    fill.position.set(-40, 60, 120)
    scene.add(sun, sun.target, hemi, fill, fill.target)
    const fog = new THREE.Fog(0xe9d8bd, 110, 560)
    scene.fog = fog

    function configureShadows() {
      sun.castShadow = quality > 0
      if (!sun.castShadow) return
      const size = quality > 1 ? 2048 : 1024
      if (sun.shadow.mapSize.x !== size) {
        sun.shadow.mapSize.set(size, size)
        sun.shadow.map?.dispose(); sun.shadow.map = null
      }
      const cam = sun.shadow.camera
      cam.left = -75; cam.right = 75; cam.top = 60; cam.bottom = -60; cam.near = 1; cam.far = 420
      cam.updateProjectionMatrix()
      sun.shadow.bias = -0.0006
      sun.shadow.normalBias = 0.05
    }
    configureShadows()

    // ---------------------------------------------- environment

    const keyColors = KEYS.map((k) => {
      const out = { ...k }
      for (const name of ['top', 'horizon', 'sun', 'hemiSky', 'hemiGround', 'fog', 'deep', 'cloud']) out[name] = new THREE.Color(k[name])
      return out
    })
    const env = {
      light: 1, night: 0, dusk: 0, wind: 0.3, sunColor: new THREE.Color(), skyColor: new THREE.Color(), horizon: new THREE.Color(),
      fogColor: new THREE.Color(), sunDir: new THREE.Vector3(), sunIntensity: 1, cloudColor: new THREE.Color(),
    }
    let lightValue = 1

    function mixKey(name, a, b, t, out) { return out.copy(a[name]).lerp(b[name], t) }

    function applyEnvironment(target, dt) {
      lightValue += (target - lightValue) * (1 - Math.exp(-dt * 1.4))
      const l = lightValue
      let i = 0
      while (i < keyColors.length - 2 && l > keyColors[i + 1].l) i++
      const a = keyColors[i], b = keyColors[i + 1]
      const t = clamp01((l - a.l) / (b.l - a.l))
      env.light = l
      env.night = clamp01((0.1 - l) / 0.38)
      env.dusk = clamp01(1 - Math.abs(l - 0.22) / 0.3)
      mixKey('top', a, b, t, skyUniforms.top.value)
      mixKey('horizon', a, b, t, skyUniforms.horizon.value)
      mixKey('sun', a, b, t, env.sunColor)
      mixKey('fog', a, b, t, env.fogColor)
      mixKey('cloud', a, b, t, env.cloudColor)
      env.skyColor.copy(skyUniforms.top.value)
      env.horizon.copy(skyUniforms.horizon.value)
      const alt = lerp(a.alt, b.alt, t) * Math.PI / 180
      // The sun sets ahead of the bird and a little behind the islands, so the player always chases it.
      const az = 0.62
      env.sunDir.set(Math.cos(alt) * Math.sin(az), Math.sin(alt), -Math.cos(alt) * Math.cos(az)).normalize()
      env.sunIntensity = lerp(a.sunI, b.sunI, t)
      skyUniforms.sunDir.value.copy(env.sunDir)
      skyUniforms.sunColor.value.copy(env.sunColor).multiplyScalar(clamp01(alt * 6 + 0.6))
      skyUniforms.stars.value = env.night
      skyUniforms.moon.value = env.night
      seaUniforms.sunDir.value.copy(env.sunDir)
      seaUniforms.sunColor.value.copy(env.sunColor)
      seaUniforms.sunI.value = clamp01(alt * 8 + 0.5) + env.night * 0.3
      mixKey('deep', a, b, t, seaUniforms.deep.value)
      seaUniforms.top.value.copy(skyUniforms.top.value)
      seaUniforms.horizon.value.copy(skyUniforms.horizon.value)
      seaUniforms.fogColor.value.copy(env.fogColor)
      fog.color.copy(env.fogColor)
      // At night the key light becomes moonlight from the moon's side.
      if (env.night > 0) {
        tmpColor.set('#8ea6e0')
        sun.color.copy(env.sunColor).lerp(tmpColor, env.night)
      } else sun.color.copy(env.sunColor)
      sun.intensity = Math.max(0.35, env.sunIntensity * clamp01(alt * 5 + 0.75))
      mixKey('hemiSky', a, b, t, hemi.color)
      mixKey('hemiGround', a, b, t, hemi.groundColor)
      hemi.intensity = lerp(a.hemiI, b.hemiI, t)
      // Warm bounce from the glowing horizon, strongest around sunset when the sun itself only rims the hills.
      fill.color.copy(hemi.color).lerp(env.horizon, 0.45)
      fill.intensity = 0.75 + 0.35 * clamp01(l) + 0.45 * env.dusk
      cloudMaterial.color.copy(env.cloudColor)
      cloudMaterial.emissive.copy(env.cloudColor).lerp(tmpColor2.copy(env.horizon), 0.5)
      cloudMaterial.emissiveIntensity = 0.14 + env.dusk * 0.18
    }

    // ---------------------------------------------- update

    let ambient = lib ? props.createAmbient(lib, scene, heightAt) : null
    const lightDir = new THREE.Vector3()

    function update(camera, focus, dt, time) {
      const b = state.bird
      const target = state.night ? -0.35 * clamp01(state.nightTime / 6) : Math.min(1, state.daylight / DAY_FULL)
      applyEnvironment(target, dt)
      env.wind = clamp01(0.25 + b.speed / 80)
      const cx = camera.position.x
      stream(chunks, CHUNK, cx - 150, cx + 210, buildChunk, 2)
      stream(farChunks, FAR_CHUNK, cx - 760, cx + 900, buildFar, 1)
      sky.position.copy(camera.position)
      sea.position.set(Math.round(cx / 10) * 10, 0, 0)
      seaUniforms.time.value = time
      skyUniforms.time.value = time
      for (const cloud of clouds) {
        const u = cloud.userData
        const x = cx - CLOUD_SPAN * 0.42 + ((u.offset - time * u.drift - cx * (u.z < -60 ? 0.12 : 0)) % CLOUD_SPAN + CLOUD_SPAN) % CLOUD_SPAN
        cloud.position.set(x, u.y, u.z)
      }
      // Shadow frustum follows the focus point; the light sits along the sun direction (or the moon at night).
      lightDir.copy(env.sunDir)
      if (env.night > 0) lightDir.lerp(skyUniforms.moonDir.value, env.night).normalize()
      if (lightDir.y < 0.12) { lightDir.y = 0.12; lightDir.normalize() }
      fill.target.position.set(focus.x, 0, 0)
      fill.position.set(focus.x - 60, 80, 140)
      sun.target.position.set(focus.x + 20, focus.y * 0.4, -12)
      sun.position.copy(sun.target.position).addScaledVector(lightDir, 220)
      for (const chunk of chunks.values()) chunk.props?.update(time, dt, env)
      ambient?.update(time, dt, cx, env)
    }

    function reset(next) {
      state = next
      for (const chunk of chunks.values()) disposeChunk(chunk)
      for (const chunk of farChunks.values()) disposeChunk(chunk)
      chunks.clear(); farChunks.clear()
      lightValue = Math.min(1, state.daylight / DAY_FULL)
    }

    function setQuality(next) {
      if (next === quality) return
      quality = next
      configureShadows()
      for (const chunk of chunks.values()) { chunk.mesh.castShadow = chunk.mesh.receiveShadow = quality > 0 }
    }

    return { env, heightAt, update, reset, setQuality, sun, get state() { return state } }
  }

  globalThis.XiabanGliderWorld = { BIOMES, biomeFor, islandName, create, SEA_FLOOR }
})()
