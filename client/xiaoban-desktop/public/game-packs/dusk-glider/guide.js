/* 暮色滑翔 · guides that make the invisible rules visible: the hold-zone ribbon on the track and the landing marker. */
(() => {
  'use strict'

  const SEGMENTS = 110
  const STEP = 1.6
  const BACK = 12
  const AHEAD = SEGMENTS * STEP - BACK
  const CHEVRON = 4 // metres per chevron along the track

  const KINDS = {
    perfect: { color: '#ffcf4a', label: '完美', glow: 1 },
    good: { color: '#ffffff', label: '普通', glow: 0.55 },
    skip: { color: '#8fdcff', label: '打水漂', glow: 0.7 },
    thud: { color: '#ff6b5a', label: '太陡 会磕', glow: 0.8 },
    water: { color: '#5fb8ff', label: '落水', glow: 0.7 },
  }

  const clamp01 = (v) => v < 0 ? 0 : v > 1 ? 1 : v

  function chevronTexture(THREE) {
    const canvas = document.createElement('canvas')
    canvas.width = 64; canvas.height = 32
    const c = canvas.getContext('2d')
    c.strokeStyle = '#fff'; c.lineWidth = 7; c.lineCap = 'round'; c.lineJoin = 'round'
    c.beginPath(); c.moveTo(20, 6); c.lineTo(42, 16); c.lineTo(20, 26); c.stroke()
    const texture = new THREE.CanvasTexture(canvas)
    texture.wrapS = THREE.RepeatWrapping
    texture.colorSpace = THREE.SRGBColorSpace
    return texture
  }

  function beamTexture(THREE) {
    const canvas = document.createElement('canvas')
    canvas.width = 4; canvas.height = 64
    const c = canvas.getContext('2d')
    const g = c.createLinearGradient(0, 0, 0, 64)
    g.addColorStop(0, 'rgba(255,255,255,0)'); g.addColorStop(1, 'rgba(255,255,255,1)')
    c.fillStyle = g; c.fillRect(0, 0, 4, 64)
    return new THREE.CanvasTexture(canvas)
  }

  function labelTexture(THREE, text, color) {
    const canvas = document.createElement('canvas')
    canvas.width = 256; canvas.height = 96
    const c = canvas.getContext('2d')
    c.font = '700 54px "PingFang SC","HarmonyOS Sans SC","Microsoft YaHei",system-ui,sans-serif'
    c.textAlign = 'center'; c.textBaseline = 'middle'
    c.lineJoin = 'round'; c.lineWidth = 12; c.strokeStyle = 'rgba(42,40,80,0.78)'
    c.strokeText(text, 128, 50)
    c.fillStyle = color
    c.fillText(text, 128, 50)
    const texture = new THREE.CanvasTexture(canvas)
    texture.colorSpace = THREE.SRGBColorSpace
    return texture
  }

  function create(THREE, scene, rules) {
    // ---------------------------------------------- hold-zone ribbon
    // Gold chevrons where a good player keeps the finger down (downhill), a faint sky-blue trail where it is released.
    const count = (SEGMENTS + 1) * 2
    const positions = new Float32Array(count * 3), colors = new Float32Array(count * 4), uvs = new Float32Array(count * 2)
    const index = []
    for (let i = 0; i < SEGMENTS; i++) { const a = i * 2; index.push(a, a + 1, a + 2, a + 1, a + 3, a + 2) }
    const ribbonGeometry = new THREE.BufferGeometry()
    ribbonGeometry.setAttribute('position', new THREE.BufferAttribute(positions, 3).setUsage(THREE.DynamicDrawUsage))
    ribbonGeometry.setAttribute('color', new THREE.BufferAttribute(colors, 4).setUsage(THREE.DynamicDrawUsage))
    ribbonGeometry.setAttribute('uv', new THREE.BufferAttribute(uvs, 2).setUsage(THREE.DynamicDrawUsage))
    ribbonGeometry.setIndex(index)
    const chevrons = chevronTexture(THREE)
    const ribbon = new THREE.Mesh(ribbonGeometry, new THREE.MeshBasicMaterial({
      map: chevrons, vertexColors: true, transparent: true, depthWrite: false, side: THREE.DoubleSide, fog: false,
      polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2,
    }))
    ribbon.frustumCulled = false
    ribbon.renderOrder = 2
    scene.add(ribbon)
    const gold = new THREE.Color('#ffc93c'), sky = new THREE.Color('#a6dcff')

    function updateRibbon(state, time, night) {
      const bx = state.bird.x
      const x0 = Math.floor((bx - BACK) / STEP) * STEP
      for (let i = 0; i <= SEGMENTS; i++) {
        const x = x0 + i * STEP
        const s = rules.sample(state, x)
        const hold = s.d < rules.HOLD_SLOPE
        const color = hold ? gold : sky
        let alpha = s.water ? 0 : hold ? 0.6 + 0.35 * clamp01((-s.d - 0.03) / 0.2) : 0.22
        alpha *= clamp01((x - (bx - BACK)) / 8) * clamp01((bx + AHEAD - x) / 40) * (1 - night * 0.25)
        for (let side = 0; side < 2; side++) {
          const v = (i * 2 + side)
          positions[v * 3] = x; positions[v * 3 + 1] = s.h + 0.12; positions[v * 3 + 2] = side ? 1.1 : -1.1
          colors[v * 4] = color.r; colors[v * 4 + 1] = color.g; colors[v * 4 + 2] = color.b; colors[v * 4 + 3] = alpha
          uvs[v * 2] = x / CHEVRON; uvs[v * 2 + 1] = side
        }
      }
      chevrons.offset.x = -(time * 0.7) % 1
      ribbonGeometry.attributes.position.needsUpdate = true
      ribbonGeometry.attributes.color.needsUpdate = true
      ribbonGeometry.attributes.uv.needsUpdate = true
    }

    // ---------------------------------------------- landing marker
    // A ring on the ground where the bird would touch down with the current input, coloured by how it would land.
    const flat = (geometry) => geometry.rotateX(-Math.PI / 2)
    const paint = () => new THREE.MeshBasicMaterial({ color: 0xffffff, transparent: true, depthWrite: false, side: THREE.DoubleSide, fog: false, polygonOffset: true, polygonOffsetFactor: -3, polygonOffsetUnits: -3 })
    const ringMaterial = paint(), discMaterial = paint(), pulseMaterial = paint()
    const tilt = new THREE.Group() // lies along the slope at the touchdown point
    const ring = new THREE.Mesh(flat(new THREE.RingGeometry(0.95, 1.3, 40)), ringMaterial)
    const disc = new THREE.Mesh(flat(new THREE.CircleGeometry(0.95, 32)), discMaterial)
    const pulse = new THREE.Mesh(flat(new THREE.RingGeometry(0.95, 1.1, 40)), pulseMaterial)
    for (const mesh of [disc, ring, pulse]) { mesh.renderOrder = 3; tilt.add(mesh) }

    const upright = new THREE.Group() // the beam and label always stand straight up
    const beamMaterial = new THREE.MeshBasicMaterial({ map: beamTexture(THREE), transparent: true, depthWrite: false, blending: THREE.AdditiveBlending, side: THREE.DoubleSide, fog: false })
    const beam = new THREE.Mesh(new THREE.CylinderGeometry(0.16, 0.42, 8, 14, 1, true).translate(0, 4, 0), beamMaterial)
    beam.renderOrder = 3
    upright.add(beam)
    const labels = {}
    for (const [kind, spec] of Object.entries(KINDS)) {
      const sprite = new THREE.Sprite(new THREE.SpriteMaterial({ map: labelTexture(THREE, spec.label, spec.color), transparent: true, depthTest: false, depthWrite: false, fog: false }))
      sprite.scale.set(5.2, 1.95, 1)
      sprite.position.y = 9.4
      sprite.renderOrder = 6
      sprite.visible = false
      upright.add(sprite)
      labels[kind] = sprite
    }
    const marker = new THREE.Group()
    marker.add(tilt, upright)
    marker.visible = false
    scene.add(marker)
    const paintColor = new THREE.Color()

    function updateMarker(state, tip, time, dist) {
      const b = state.bird
      const landing = tip.phase === 'air' ? (b.hold ? tip.dive : tip.glide) : null
      const spec = landing && KINDS[landing.kind]
      if (!spec || landing.t < 0.12) { marker.visible = false; return }
      const s = rules.sample(state, landing.x)
      marker.visible = true
      marker.position.set(landing.x, s.h + 0.2, 0)
      tilt.rotation.z = Math.atan(s.d)
      // Grow with the camera's distance so the marker keeps its size on screen when the view pulls back.
      const grow = Math.max(1, Math.min(2.6, dist / 60))
      tilt.scale.setScalar(grow * (1 + Math.sin(time * 7) * 0.05))
      upright.scale.setScalar(grow)
      paintColor.set(spec.color)
      ringMaterial.color.copy(paintColor); ringMaterial.opacity = 0.95
      discMaterial.color.copy(paintColor); discMaterial.opacity = 0.28
      beamMaterial.color.copy(paintColor); beamMaterial.opacity = 0.3 + 0.4 * spec.glow
      const k = (time * 1.4) % 1
      pulse.scale.setScalar(1 + k * 1.6)
      pulseMaterial.color.copy(paintColor); pulseMaterial.opacity = (1 - k) * 0.7 * spec.glow
      for (const [kind, sprite] of Object.entries(labels)) sprite.visible = kind === landing.kind
    }

    let visible = true
    function setVisible(on) {
      visible = on
      ribbon.visible = on
      if (!on) marker.visible = false
    }

    function reset() { marker.visible = false }

    /** `tip` is `rules.advice(state)`; `dist` is the camera's distance to the bird. */
    function update(state, tip, time, dist, night) {
      if (!visible) return
      updateRibbon(state, time, night)
      updateMarker(state, tip, time, dist)
    }

    return { update, reset, setVisible }
  }

  globalThis.XiabanGliderGuide = { create, KINDS }
})()
