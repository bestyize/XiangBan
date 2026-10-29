/* 暮色滑翔 · the paper bird, its wind ribbon, contact shadow and particle effects. */
(() => {
  'use strict'

  const TRAIL = 72
  const damp = (current, target, rate, dt) => current + (target - current) * (1 - Math.exp(-rate * dt))

  function tri(positions, colors, color, ...points) {
    for (const p of points) { positions.push(p[0], p[1], p[2]); colors.push(color.r, color.g, color.b) }
  }

  function geometryFrom(THREE, build) {
    const positions = [], colors = []
    build((color, ...points) => tri(positions, colors, color, ...points))
    const g = new THREE.BufferGeometry()
    g.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3))
    g.setAttribute('color', new THREE.Float32BufferAttribute(colors, 3))
    g.computeVertexNormals()
    return g
  }

  // ------------------------------------------------------------ particles

  function createParticles(THREE, scene, count, additive) {
    const positions = new Float32Array(count * 3), colors = new Float32Array(count * 4), sizes = new Float32Array(count)
    const vel = new Float32Array(count * 3), life = new Float32Array(count), maxLife = new Float32Array(count), base = new Float32Array(count)
    const drag = new Float32Array(count), gravity = new Float32Array(count)
    const geometry = new THREE.BufferGeometry()
    geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3).setUsage(THREE.DynamicDrawUsage))
    geometry.setAttribute('pcolor', new THREE.BufferAttribute(colors, 4).setUsage(THREE.DynamicDrawUsage))
    geometry.setAttribute('psize', new THREE.BufferAttribute(sizes, 1).setUsage(THREE.DynamicDrawUsage))
    const material = new THREE.ShaderMaterial({
      uniforms: { scale: { value: 400 } }, transparent: true, depthWrite: false,
      blending: additive ? THREE.AdditiveBlending : THREE.NormalBlending,
      vertexShader: `
        attribute vec4 pcolor; attribute float psize; uniform float scale; varying vec4 vColor;
        void main() {
          vColor = pcolor;
          vec4 mv = modelViewMatrix * vec4(position, 1.0);
          gl_PointSize = psize * scale / -mv.z;
          gl_Position = projectionMatrix * mv;
        }`,
      fragmentShader: `
        varying vec4 vColor;
        void main() {
          float d = length(gl_PointCoord - 0.5);
          float a = smoothstep(0.5, ${additive ? '0.0' : '0.32'}, d) * vColor.a;
          if (a < 0.01) discard;
          gl_FragColor = vec4(vColor.rgb, a);
          #include <colorspace_fragment>
        }`,
    })
    const points = new THREE.Points(geometry, material)
    points.frustumCulled = false
    points.renderOrder = additive ? 5 : 4
    scene.add(points)
    let cursor = 0
    const color = new THREE.Color()

    function emit(x, y, z, vx, vy, vz, size, seconds, hex, opts = {}) {
      const i = cursor; cursor = (cursor + 1) % count
      positions[i * 3] = x; positions[i * 3 + 1] = y; positions[i * 3 + 2] = z
      vel[i * 3] = vx; vel[i * 3 + 1] = vy; vel[i * 3 + 2] = vz
      life[i] = maxLife[i] = seconds
      base[i] = size
      drag[i] = opts.drag ?? 1.5
      gravity[i] = opts.gravity ?? 0
      color.set(hex)
      colors[i * 4] = color.r; colors[i * 4 + 1] = color.g; colors[i * 4 + 2] = color.b; colors[i * 4 + 3] = opts.alpha ?? 1
    }

    function update(dt) {
      for (let i = 0; i < count; i++) {
        if (life[i] <= 0) { sizes[i] = 0; continue }
        life[i] -= dt
        const k = Math.exp(-drag[i] * dt)
        vel[i * 3] *= k; vel[i * 3 + 1] = vel[i * 3 + 1] * k - gravity[i] * dt; vel[i * 3 + 2] *= k
        positions[i * 3] += vel[i * 3] * dt; positions[i * 3 + 1] += vel[i * 3 + 1] * dt; positions[i * 3 + 2] += vel[i * 3 + 2] * dt
        if (gravity[i] > 0 && positions[i * 3 + 1] < 0) { life[i] = 0; sizes[i] = 0; continue }
        const t = Math.max(0, life[i] / maxLife[i])
        sizes[i] = base[i] * (0.35 + 0.65 * Math.sqrt(t))
        colors[i * 4 + 3] = Math.min(colors[i * 4 + 3], t * 1.6)
      }
      geometry.attributes.position.needsUpdate = true
      geometry.attributes.pcolor.needsUpdate = true
      geometry.attributes.psize.needsUpdate = true
    }

    function clear() { life.fill(0); sizes.fill(0); geometry.attributes.psize.needsUpdate = true }
    return { emit, update, clear, material }
  }

  // ------------------------------------------------------------ bird

  function create(THREE, scene, options = {}) {
    const paper = new THREE.Color('#fff6e8'), paperShade = new THREE.Color('#f3dcc2'), coral = new THREE.Color('#ff8a63'), coralDeep = new THREE.Color('#e4604a'), beak = new THREE.Color('#ffb547'), eye = new THREE.Color('#2a2430')
    const material = new THREE.MeshLambertMaterial({ vertexColors: true, flatShading: true, side: THREE.DoubleSide, emissive: new THREE.Color('#ffd39a'), emissiveIntensity: 0.08 })

    // Body: a folded-paper diamond, nose toward +x, belly at y = 0.
    const body = geometryFrom(THREE, (t) => {
      const nose = [1.25, 0.52, 0], tail = [-1.15, 0.62, 0], top = [0.05, 0.98, 0], belly = [0.1, 0.06, 0]
      const near = [0.15, 0.52, 0.42], far = [0.15, 0.52, -0.42]
      t(paper, nose, top, near); t(paperShade, nose, far, top)
      t(paper, top, tail, near); t(paperShade, top, far, tail)
      t(coral, nose, near, belly); t(coralDeep, nose, belly, far)
      t(coral, near, tail, belly); t(coralDeep, far, belly, tail)
      // Head crest and beak.
      t(paper, [0.78, 0.7, 0.12], [1.05, 1.02, 0], [0.9, 0.62, 0])
      t(paperShade, [0.78, 0.7, -0.12], [0.9, 0.62, 0], [1.05, 1.02, 0])
      t(beak, [1.22, 0.58, 0.07], [1.62, 0.5, 0], [1.22, 0.46, 0])
      t(beak, [1.22, 0.58, -0.07], [1.22, 0.46, 0], [1.62, 0.5, 0])
      t(eye, [0.98, 0.66, 0.205], [1.06, 0.7, 0.18], [1.02, 0.6, 0.19])
      t(eye, [0.98, 0.66, -0.205], [1.02, 0.6, -0.19], [1.06, 0.7, -0.18])
      // Forked tail.
      t(coral, [-0.95, 0.62, 0], [-1.85, 1.05, 0.22], [-1.55, 0.6, 0])
      t(coralDeep, [-0.95, 0.62, 0], [-1.55, 0.6, 0], [-1.85, 1.05, -0.22])
      t(paper, [-0.95, 0.62, 0], [-1.9, 0.42, 0.18], [-1.55, 0.6, 0])
      t(paperShade, [-0.95, 0.62, 0], [-1.55, 0.6, 0], [-1.9, 0.42, -0.18])
    })
    // A wing hinged on the x axis, spreading along +z. Two creased panels catch the light differently.
    const wing = geometryFrom(THREE, (t) => {
      t(paper, [0.55, 0, 0], [-0.55, 0, 0], [0.05, 0.1, 1.05])
      t(paperShade, [-0.55, 0, 0], [-0.85, 0.05, 1.2], [0.05, 0.1, 1.05])
      t(paper, [0.05, 0.1, 1.05], [-0.85, 0.05, 1.2], [-0.35, 0.18, 2.05])
      t(coral, [-0.55, 0, 0], [-0.95, 0.02, 0.45], [-0.85, 0.05, 1.2])
    })

    const root = new THREE.Group()
    const pitchNode = new THREE.Group()
    const squash = new THREE.Group()
    root.add(pitchNode); pitchNode.add(squash)
    const bodyMesh = new THREE.Mesh(body, material)
    squash.add(bodyMesh)
    const wings = [1, -1].map((side) => {
      const hinge = new THREE.Group()
      hinge.position.set(0.15, 0.7, side * 0.3)
      const mesh = new THREE.Mesh(wing, material)
      if (side < 0) mesh.scale.z = -1
      hinge.add(mesh)
      squash.add(hinge)
      return { hinge, mesh, side }
    })
    root.scale.setScalar(2)
    for (const m of [bodyMesh, ...wings.map((w) => w.mesh)]) m.castShadow = true
    scene.add(root)

    // Contact shadow keeps the landing readable even when the sun's shadow falls far away.
    const shadowCanvas = document.createElement('canvas')
    shadowCanvas.width = shadowCanvas.height = 64
    const sctx = shadowCanvas.getContext('2d')
    const gradient = sctx.createRadialGradient(32, 32, 0, 32, 32, 32)
    gradient.addColorStop(0, 'rgba(30,20,40,0.6)'); gradient.addColorStop(0.55, 'rgba(30,20,40,0.25)'); gradient.addColorStop(1, 'rgba(30,20,40,0)')
    sctx.fillStyle = gradient; sctx.fillRect(0, 0, 64, 64)
    const blob = new THREE.Mesh(new THREE.PlaneGeometry(1, 1).rotateX(-Math.PI / 2), new THREE.MeshBasicMaterial({ map: new THREE.CanvasTexture(shadowCanvas), transparent: true, depthWrite: false, fog: true }))
    blob.renderOrder = 2
    scene.add(blob)

    // Wind ribbon: a strip following the tail, wider and golden in fever.
    const trailPos = new Float32Array(TRAIL * 2 * 3), trailCol = new Float32Array(TRAIL * 2 * 4)
    const trailIndex = []
    for (let i = 0; i < TRAIL - 1; i++) { const a = i * 2; trailIndex.push(a, a + 1, a + 2, a + 1, a + 3, a + 2) }
    const trailGeometry = new THREE.BufferGeometry()
    trailGeometry.setAttribute('position', new THREE.BufferAttribute(trailPos, 3).setUsage(THREE.DynamicDrawUsage))
    trailGeometry.setAttribute('color', new THREE.BufferAttribute(trailCol, 4).setUsage(THREE.DynamicDrawUsage))
    trailGeometry.setIndex(trailIndex)
    const trail = new THREE.Mesh(trailGeometry, new THREE.MeshBasicMaterial({ vertexColors: true, transparent: true, depthWrite: false, side: THREE.DoubleSide, blending: THREE.AdditiveBlending, fog: false }))
    trail.frustumCulled = false
    trail.renderOrder = 3
    scene.add(trail)
    const history = Array.from({ length: TRAIL }, () => ({ x: 0, y: 0, a: 0, w: 0 }))
    let head = 0, historyTime = 0

    const soft = createParticles(THREE, scene, options.quality > 0 ? 360 : 180, false)
    const glow = createParticles(THREE, scene, options.quality > 0 ? 300 : 140, true)

    const pose = { pitch: 0, fold: 0.6, sweep: 0, flap: 0, flapT: 0, squash: 1, squashV: 0, spin: 0, yaw: 0.32, feverGlow: 0, ground: 0 }
    const trailColor = new THREE.Color(), gold = new THREE.Color('#ffd27a'), white = new THREE.Color('#fff3e6')

    function reset(state) {
      for (const h of history) { h.x = state.bird.x; h.y = state.bird.y; h.a = 0; h.w = 0 }
      soft.clear(); glow.clear()
      pose.spin = 0; pose.squash = 1; pose.squashV = 0
    }

    function flapOnce() { pose.flapT = 0.32 }

    function burst(type, x, y, info = {}) {
      const r = Math.random
      if (type === 'perfect') {
        const chain = info.chain || 1
        for (let i = 0; i < 16 + chain * 3; i++) {
          const a = r() * Math.PI * 2, s = 3 + r() * 6
          glow.emit(x, y + 0.6, (r() - 0.5) * 1.4, Math.cos(a) * s, Math.abs(Math.sin(a)) * s * 0.9 + 1.5, (r() - 0.5) * 3, 0.5 + r() * 0.5, 0.55 + r() * 0.4, i % 3 ? '#ffe2a1' : '#ffffff', { drag: 2.2 })
        }
        pose.spin = chain >= 3 ? 1 : 0
      } else if (type === 'land' || type === 'thud') {
        const n = type === 'thud' ? 18 : 9
        for (let i = 0; i < n; i++) soft.emit(x - r() * 1.2, y + 0.2, (r() - 0.5) * 1.6, -2 - r() * 4, 1 + r() * 2.5, (r() - 0.5) * 2, 0.7 + r() * 0.7, 0.6 + r() * 0.5, info.dust || '#e8d6b0', { drag: 2.5, alpha: 0.7 })
        pose.squashV -= type === 'thud' ? 4.2 : 2
      } else if (type === 'splash') {
        for (let i = 0; i < 34; i++) {
          const a = Math.PI * (0.15 + r() * 0.7), s = 4 + r() * 8
          soft.emit(x + (r() - 0.5), 0.1, (r() - 0.5) * 2, Math.cos(a) * s * 0.8, Math.sin(a) * s, (r() - 0.5) * 3, 0.35 + r() * 0.45, 1.2, i % 4 ? '#e9f6ff' : '#9fd6ea', { gravity: 22, drag: 0.4 })
        }
        pose.squashV -= 3
      } else if (type === 'skip') {
        for (let i = 0; i < 14; i++) soft.emit(x, 0.1, (r() - 0.5) * 1.2, -1 + r() * 4, 2 + r() * 4, (r() - 0.5) * 2, 0.3 + r() * 0.3, 0.9, '#eaf7ff', { gravity: 20, drag: 0.4 })
      } else if (type === 'cloud') {
        for (let i = 0; i < 26; i++) {
          const a = r() * Math.PI * 2
          glow.emit(x + Math.cos(a) * 2.5, y + Math.sin(a) * 2.5, (r() - 0.5) * 2, Math.cos(a) * 3, Math.sin(a) * 3, 0, 0.6 + r() * 0.6, 0.9 + r() * 0.5, '#fff4dc', { drag: 1.2 })
        }
      } else if (type === 'fever') {
        for (let i = 0; i < 40; i++) {
          const a = r() * Math.PI * 2, s = 5 + r() * 8
          glow.emit(x, y + 0.7, 0, Math.cos(a) * s, Math.sin(a) * s, (r() - 0.5) * 4, 0.7 + r() * 0.6, 0.9, i % 2 ? '#ffc861' : '#ff9f6b', { drag: 1.8 })
        }
      } else if (type === 'launch') {
        for (let i = 0; i < 6; i++) soft.emit(x - r(), y + 0.1, (r() - 0.5) * 1.2, -3 - r() * 3, 0.5 + r(), (r() - 0.5) * 1.5, 0.45 + r() * 0.4, 0.45, info.dust || '#f1e2c2', { drag: 3, alpha: 0.55 })
      } else if (type === 'island') {
        for (let i = 0; i < 30; i++) glow.emit(x + (r() - 0.5) * 8, y + r() * 6, (r() - 0.5) * 6, (r() - 0.5) * 2, 2 + r() * 3, (r() - 0.5) * 2, 0.5 + r() * 0.5, 1.4 + r() * 0.6, i % 2 ? '#ffe7b0' : '#ffffff', { drag: 0.8 })
      }
    }

    function update(state, dt, time, env, heightAt, viewHeight, cameraZ) {
      const b = state.bird
      const air = !b.ground
      const surface = heightAt(b.x, 0)
      const slope = Math.atan2(heightAt(b.x + 0.6, 0) - heightAt(b.x - 0.6, 0), 1.2)
      const target = air ? Math.atan2(b.vy, b.vx) : slope
      pose.pitch = damp(pose.pitch, Math.max(-1.25, Math.min(1.1, target)), air ? 7 : 16, dt)
      pose.ground = damp(pose.ground, air ? 0 : 1, 12, dt)
      // Wings: spread to glide, tucked back to dive, half-folded while sliding.
      let fold, sweep
      if (b.hold) { fold = air ? 1.2 : 1.05; sweep = 0.75 }
      else if (air) { fold = -0.08 + Math.max(0, -b.vy) * 0.008; sweep = 0.05 }
      else { fold = 0.65; sweep = 0.35 }
      if (state.night && !air) { fold = 1.15; sweep = 0.6 }
      pose.fold = damp(pose.fold, fold, 10, dt)
      pose.sweep = damp(pose.sweep, sweep, 9, dt)
      pose.flapT = Math.max(0, pose.flapT - dt)
      const lazy = air && !b.hold && b.vy > -2 && b.speed < 22 ? Math.sin(time * 9) * 0.22 : 0
      const flap = pose.flapT > 0 ? Math.sin((0.32 - pose.flapT) / 0.32 * Math.PI * 2) * 0.9 : lazy
      for (const w of wings) {
        w.hinge.rotation.x = -w.side * (pose.fold - flap)
        w.hinge.rotation.y = w.side * pose.sweep
      }
      // Springy squash on impacts.
      pose.squashV += (1 - pose.squash) * 160 * dt
      pose.squashV *= Math.exp(-11 * dt)
      pose.squash += pose.squashV * dt
      squash.scale.set(1 / Math.sqrt(Math.max(0.6, pose.squash)), pose.squash, 1)
      pose.spin = Math.max(0, pose.spin - dt * 1.6)
      // A chained perfect earns a full loop flourish.
      pitchNode.rotation.set(0, 0, pose.pitch + (pose.spin > 0 ? (1 - pose.spin) * Math.PI * 2 : 0))
      const lift = air ? 0 : 0.05
      root.position.set(b.x, b.y + lift, 0)
      root.rotation.set(0, pose.yaw + (air ? Math.sin(time * 1.3) * 0.05 : 0), air ? Math.sin(time * 2.1) * 0.06 : 0)
      pose.feverGlow = damp(pose.feverGlow, state.fever ? 1 : 0, 4, dt)
      material.emissiveIntensity = 0.08 + env.night * 0.18 + pose.feverGlow * (0.35 + Math.sin(time * 10) * 0.12)

      const above = Math.max(0, b.y - surface)
      const size = 2.6 + Math.min(4, above * 0.12)
      blob.position.set(b.x + 0.1, surface + 0.08, 0)
      blob.rotation.set(0, 0, slope)
      blob.scale.set(size, 1, size * 0.6)
      blob.material.opacity = Math.max(0, 0.75 - above / 30) * (1 - env.night * 0.5)
      blob.visible = surface > -0.2 || above < 30

      // Ribbon history sampled at a fixed rate so its length reads as speed.
      historyTime += dt
      while (historyTime > 1 / 50) {
        historyTime -= 1 / 50
        head = (head + 1) % TRAIL
        const h = history[head]
        const c = Math.cos(pose.pitch), s = Math.sin(pose.pitch)
        h.x = b.x - c * 1.6; h.y = b.y + 0.75 - s * 1.6
        h.a = (air ? 0.55 : 0.18) + pose.feverGlow * 0.4
        h.w = (air ? 0.55 : 0.3) + pose.feverGlow * 0.5 + Math.min(0.4, b.speed / 120)
      }
      trailColor.copy(white).lerp(gold, pose.feverGlow * 0.9)
      for (let i = 0; i < TRAIL; i++) {
        const h = history[(head - i + TRAIL) % TRAIL]
        const n = history[(head - i - 1 + TRAIL * 2) % TRAIL]
        let dx = h.x - n.x, dy = h.y - n.y
        const len = Math.hypot(dx, dy) || 1
        dx /= len; dy /= len
        const fade = 1 - i / (TRAIL - 1)
        const w = h.w * fade * 0.5
        const o = i * 6
        trailPos[o] = h.x - dy * w; trailPos[o + 1] = h.y + dx * w; trailPos[o + 2] = 0.05
        trailPos[o + 3] = h.x + dy * w; trailPos[o + 4] = h.y - dx * w; trailPos[o + 5] = 0.05
        const a = h.a * fade * fade * (0.5 + env.night * 0.5)
        const c = i * 8
        for (const off of [0, 4]) { trailCol[c + off] = trailColor.r; trailCol[c + off + 1] = trailColor.g; trailCol[c + off + 2] = trailColor.b; trailCol[c + off + 3] = a }
      }
      // The newest point sticks to the bird so the ribbon never detaches.
      trailPos[0] = trailPos[3] = root.position.x - Math.cos(pose.pitch) * 1.4
      trailGeometry.attributes.position.needsUpdate = true
      trailGeometry.attributes.color.needsUpdate = true

      if (state.fever && Math.random() < dt * 40) glow.emit(b.x - 1 + (Math.random() - 0.5), b.y + 0.7 + (Math.random() - 0.5), (Math.random() - 0.5), -b.vx * 0.2, -b.vy * 0.2 + 0.5, 0, 0.35 + Math.random() * 0.3, 0.6, Math.random() < 0.5 ? '#ffd27a' : '#ffffff', { drag: 2 })
      if (!air && !b.hold && b.speed > 14 && surface > 0.2 && Math.random() < dt * b.speed * 0.4) soft.emit(b.x - 1.2, surface + 0.15, (Math.random() - 0.5) * 0.8, -b.vx * 0.15, 0.6 + Math.random(), (Math.random() - 0.5), 0.35 + Math.random() * 0.3, 0.5, '#f2e6c9', { drag: 3, alpha: 0.5 })
      const scale = viewHeight / (2 * Math.tan(cameraZ / 2))
      soft.material.uniforms.scale.value = scale
      glow.material.uniforms.scale.value = scale
      soft.update(dt); glow.update(dt)
    }

    return { root, reset, burst, update, flap: flapOnce, glow, soft }
  }

  globalThis.XiabanGliderBird = { create }
})()
