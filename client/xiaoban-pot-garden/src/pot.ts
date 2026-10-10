import {
  BufferAttribute, BufferGeometry, Color, DynamicDrawUsage, Group, IcosahedronGeometry, InstancedMesh, Matrix4, Mesh,
  MeshPhysicalMaterial, MeshStandardMaterial, Object3D, SphereGeometry,
} from 'three'

/** Rim top is at about 1.1 and the base at 0. The screen sits on the front of the wall. */
export const POT = {
  /** Superellipse exponent of every horizontal section: 2 is a circle, larger is squarer. */
  n: 2.45,
  /** Front-to-back size relative to side-to-side. */
  depth: 0.93,
  soilY: 1.03,
  innerRadius: 0.535,
  screenY: 0.46,
  screenHalfW: 0.46,
  screenHalfH: 0.27,
} as const

export const rand = (seed: number) => () => {
  seed = (seed + 0x6d2b79f5) | 0
  let t = Math.imul(seed ^ (seed >>> 15), 1 | seed)
  t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t
  return ((t ^ (t >>> 14)) >>> 0) / 4294967296
}

const ease = (x: number) => (x <= 0 ? 0 : x >= 1 ? 1 : x * x * (3 - 2 * x))
const signedPow = (value: number, exponent: number) => Math.sign(value) * Math.abs(value) ** exponent
const section = (angle: number) => [signedPow(Math.cos(angle), 2 / POT.n), signedPow(Math.sin(angle), 2 / POT.n)] as const

/** Half-width of the straight wall under the rim. */
export const wallRadius = (y: number) => 0.55 + (y - 0.15) * ((0.64 - 0.55) / (0.74 - 0.15))

/** Where the front of the wall is at (x, y), so inlays can follow its curve. */
export function surfaceZ(x: number, y: number): number {
  const a = wallRadius(y)
  const ratio = Math.min(Math.abs(x) / a, 0.999)
  return POT.depth * a * (1 - ratio ** POT.n) ** (1 / POT.n)
}

function chaikin(points: [number, number][], rounds: number): [number, number][] {
  let current = points
  for (let round = 0; round < rounds; round++) {
    const next: [number, number][] = [current[0]]
    for (let i = 0; i < current.length - 1; i++) {
      const [x0, y0] = current[i]
      const [x1, y1] = current[i + 1]
      next.push([0.75 * x0 + 0.25 * x1, 0.75 * y0 + 0.25 * y1], [0.25 * x0 + 0.75 * x1, 0.25 * y0 + 0.75 * y1])
    }
    next.push(current[current.length - 1])
    current = next
  }
  return current
}

/** (radius, height) from the middle of the base, up the wall, over the rim and down inside to the soil. */
const outlinePoints: [number, number][] = [
  [0.02, 0], [0.3, 0], [0.42, 0.012], [0.49, 0.07], [0.53, 0.12], [0.55, 0.15],
  [0.64, 0.74],
  [0.652, 0.762], [0.668, 0.79], [0.69, 0.84], [0.694, 0.9], [0.694, 1.0],
  [0.684, 1.055], [0.645, 1.095], [0.6, 1.103], [0.562, 1.088], [0.545, 1.05],
  [0.54, 1.0], [0.536, 0.96],
]

const RING = 168

export function buildBodyGeometry(): BufferGeometry {
  const profile = chaikin(outlinePoints, 3)
  const positions = new Float32Array(profile.length * RING * 3)
  const colors = new Float32Array(profile.length * RING * 3)
  const apex = profile.reduce((best, point, i) => (point[1] > profile[best][1] ? i : best), 0)
  let k = 0
  let c3 = 0
  for (const [pi, [r, y]] of profile.entries()) {
    const shade = 1 - ambient(pi < apex, y)
    for (let j = 0; j < RING; j++, c3 += 3) colors.set([shade, shade * 0.985, shade * 0.96], c3)
    for (let j = 0; j < RING; j++) {
      const [c, s] = section((j / RING) * Math.PI * 2)
      positions[k++] = r * c
      positions[k++] = y
      positions[k++] = r * s * POT.depth
    }
  }
  const index: number[] = []
  for (let i = 0; i < profile.length - 1; i++) {
    for (let j = 0; j < RING; j++) {
      const a = i * RING + j
      const b = i * RING + ((j + 1) % RING)
      const c = (i + 1) * RING + j
      const d = (i + 1) * RING + ((j + 1) % RING)
      index.push(a, c, b, b, c, d)
    }
  }
  const geometry = new BufferGeometry()
  geometry.setAttribute('position', new BufferAttribute(positions, 3))
  geometry.setAttribute('color', new BufferAttribute(colors, 3))
  geometry.setIndex(index)
  geometry.computeVertexNormals()
  return geometry
}

/** Cheap ambient occlusion baked into the glaze: the wall under the lip, the foot on the ground and the inside of the bowl. */
function ambient(outside: boolean, y: number): number {
  if (!outside) return 0.32 * ease((1.06 - y) / 0.12)
  return 0.2 * ease((y - 0.52) / 0.24) * (y < 0.8 ? 1 : 0) + 0.22 * ease((0.11 - y) / 0.11)
}

interface PatchOptions {
  cy: number
  halfW: number
  halfH: number
  /** Corner sharpness of the outline: 2 is an ellipse, larger is squarer. */
  exponent: number
  /** Height above the wall at a normalised distance d from the edge (0) to the middle (1). */
  lift: (d: number) => number
  segments?: number
}

/** A rounded-rectangle sheet laid on the wall: the bezel and the glass are both made this way so they follow its curve. */
export function buildPatch(o: PatchOptions): BufferGeometry {
  const S = o.segments ?? 64
  const positions = new Float32Array((S + 1) * (S + 1) * 3)
  const uvs = new Float32Array((S + 1) * (S + 1) * 2)
  let p = 0
  let q = 0
  for (let i = 0; i <= S; i++) {
    for (let j = 0; j <= S; j++) {
      const s = -1 + (2 * i) / S
      const t = -1 + (2 * j) / S
      const m = Math.max(Math.abs(s), Math.abs(t))
      const scale = m === 0 ? 0 : m / (Math.abs(s) ** o.exponent + Math.abs(t) ** o.exponent) ** (1 / o.exponent)
      const px = s * scale
      const py = t * scale
      const x = px * o.halfW
      const y = o.cy + py * o.halfH
      positions[p++] = x
      positions[p++] = y
      positions[p++] = surfaceZ(x, y) + o.lift(1 - m)
      uvs[q++] = 0.5 + px / 2
      uvs[q++] = 0.5 + py / 2
    }
  }
  const index: number[] = []
  const at = (i: number, j: number) => i * (S + 1) + j
  for (let i = 0; i < S; i++) {
    for (let j = 0; j < S; j++) {
      const a = at(i, j)
      const b = at(i + 1, j)
      const c = at(i, j + 1)
      const d = at(i + 1, j + 1)
      index.push(a, b, c, b, d, c)
    }
  }
  const geometry = new BufferGeometry()
  geometry.setAttribute('position', new BufferAttribute(positions, 3))
  geometry.setAttribute('uv', new BufferAttribute(uvs, 2))
  geometry.setAttribute('color', new BufferAttribute(new Float32Array(positions.length).fill(1), 3))
  geometry.setIndex(index)
  geometry.computeVertexNormals()
  return geometry
}

function valueNoise(x: number, z: number, seed: number): number {
  const hash = (ix: number, iz: number) => {
    let h = Math.imul(ix, 374761393) ^ Math.imul(iz, 668265263) ^ Math.imul(seed, 2147483647)
    h = Math.imul(h ^ (h >>> 13), 1274126177)
    return ((h ^ (h >>> 16)) >>> 0) / 4294967296
  }
  const ix = Math.floor(x)
  const iz = Math.floor(z)
  const fx = ease(x - ix)
  const fz = ease(z - iz)
  const top = hash(ix, iz) * (1 - fx) + hash(ix + 1, iz) * fx
  const bottom = hash(ix, iz + 1) * (1 - fx) + hash(ix + 1, iz + 1) * fx
  return top * (1 - fz) + bottom * fz
}

/** Height of the soil surface at (x, z), with the edge rising a little against the wall. */
export function soilHeight(x: number, z: number): number {
  const rho = Math.min(1, soilRho(x, z))
  const lumps = valueNoise(x * 8, z * 8, 3) * 0.6 + valueNoise(x * 19, z * 19, 7) * 0.4
  return POT.soilY + (lumps - 0.5) * 0.05 + 0.02 * ease((rho - 0.72) / 0.28)
}

const soilRho = (x: number, z: number) => (Math.abs(x / 0.55) ** POT.n + Math.abs(z / (0.55 * POT.depth)) ** POT.n) ** (1 / POT.n)

function buildSoilGeometry(): BufferGeometry {
  const rings = 30
  const segments = 128
  const positions = new Float32Array(((rings + 1) * segments + 1) * 3)
  const colors = new Float32Array(positions.length)
  const dark = new Color('#2a190f')
  const mid = new Color('#4a2d1b')
  const colour = new Color()
  const write = (index: number, x: number, z: number) => {
    const y = soilHeight(x, z)
    positions.set([x, y, z], index * 3)
    const rho = Math.min(1, soilRho(x, z))
    const patch = valueNoise(x * 14, z * 14, 11)
    colour.copy(dark).lerp(mid, patch)
    colour.multiplyScalar(1 - 0.5 * ease((rho - 0.62) / 0.38))
    colors.set([colour.r, colour.g, colour.b], index * 3)
  }
  write(0, 0, 0)
  for (let ring = 1; ring <= rings; ring++) {
    for (let j = 0; j < segments; j++) {
      const [c, s] = section((j / segments) * Math.PI * 2)
      const rho = ring / rings
      write(1 + (ring - 1) * segments + j, 0.55 * rho * c, 0.55 * POT.depth * rho * s)
    }
  }
  const index: number[] = []
  for (let j = 0; j < segments; j++) index.push(0, 1 + ((j + 1) % segments), 1 + j)
  for (let ring = 1; ring < rings; ring++) {
    for (let j = 0; j < segments; j++) {
      const a = 1 + (ring - 1) * segments + j
      const b = 1 + (ring - 1) * segments + ((j + 1) % segments)
      const c = 1 + ring * segments + j
      const d = 1 + ring * segments + ((j + 1) % segments)
      index.push(a, b, c, b, d, c)
    }
  }
  const geometry = new BufferGeometry()
  geometry.setAttribute('position', new BufferAttribute(positions, 3))
  geometry.setAttribute('color', new BufferAttribute(colors, 3))
  geometry.setIndex(index)
  geometry.computeVertexNormals()
  return geometry
}

const chunkColours = ['#1a0f08', '#22140a', '#2c1a0d', '#382311', '#442c17', '#52341c', '#603d20']

function buildChunks(count: number): InstancedMesh {
  const random = rand(42)
  const base = new IcosahedronGeometry(1, 1)
  const squash = new Matrix4().makeScale(1, 0.62, 1)
  base.applyMatrix4(squash)
  const material = new MeshStandardMaterial({ roughness: 0.92, metalness: 0 })
  const mesh = new InstancedMesh(base, material, count)
  mesh.instanceMatrix.setUsage(DynamicDrawUsage)
  const object = new Object3D()
  const colour = new Color()
  const rho = 0.93
  for (let i = 0; i < count; i++) {
    let x = 0
    let z = 0
    do {
      x = (random() * 2 - 1) * 0.53
      z = (random() * 2 - 1) * 0.53 * POT.depth
    } while (soilRho(x, z) > rho)
    const size = 0.01 + random() ** 2.4 * 0.03
    object.position.set(x, soilHeight(x, z) + size * 0.2, z)
    object.rotation.set(random() * 3, random() * 6.28, random() * 3)
    object.scale.set(size * (0.8 + random() * 0.6), size * (0.7 + random() * 0.5), size * (0.8 + random() * 0.6))
    object.updateMatrix()
    mesh.setMatrixAt(i, object.matrix)
    colour.set(chunkColours[Math.floor(random() * chunkColours.length)]).multiplyScalar(0.85 + random() * 0.3)
    mesh.setColorAt(i, colour)
  }
  mesh.castShadow = true
  mesh.receiveShadow = true
  return mesh
}

function buildPebbles(): Group {
  const random = rand(7)
  const group = new Group()
  const geometry = new SphereGeometry(1, 28, 20)
  const spots: [number, number, number][] = [
    [-0.4, 0.04, 0.032], [-0.2, 0.2, 0.026], [0.3, 0.23, 0.03], [0.45, -0.02, 0.034], [0.08, -0.22, 0.027], [-0.32, -0.16, 0.025], [0.22, -0.36, 0.024],
  ]
  for (const [x, z, r] of spots) {
    const material = new MeshPhysicalMaterial({
      color: new Color('#b0652c').offsetHSL((random() - 0.5) * 0.02, 0, (random() - 0.5) * 0.06),
      roughness: 0.5, clearcoat: 0.2, clearcoatRoughness: 0.5,
    })
    const pebble = new Mesh(geometry, material)
    pebble.scale.set(r, r * 0.9, r)
    pebble.position.set(x, soilHeight(x, z) + r * 0.45, z)
    pebble.castShadow = true
    pebble.receiveShadow = true
    group.add(pebble)
  }
  return group
}

export interface PotParts {
  group: Group
  /** The glass material: the face is its emissive map. */
  glass: MeshPhysicalMaterial
  ceramic: MeshPhysicalMaterial
}

export function createPot(): PotParts {
  const group = new Group()
  const ceramic = new MeshPhysicalMaterial({
    vertexColors: true, color: '#f7f2eb', roughness: 0.3, metalness: 0, clearcoat: 1, clearcoatRoughness: 0.14,
    sheen: 0.5, sheenColor: new Color('#ffe9d2'), sheenRoughness: 0.55,
  })
  const body = new Mesh(buildBodyGeometry(), ceramic)
  body.castShadow = true
  body.receiveShadow = true
  group.add(body)

  const soil = new Mesh(buildSoilGeometry(), new MeshStandardMaterial({ vertexColors: true, roughness: 0.95, metalness: 0 }))
  soil.receiveShadow = true
  group.add(soil, buildChunks(900), buildPebbles())

  const bezelMaterial = ceramic.clone()
  bezelMaterial.polygonOffset = true
  bezelMaterial.polygonOffsetFactor = -2
  bezelMaterial.polygonOffsetUnits = -2
  const bezel = new Mesh(
    buildPatch({ cy: POT.screenY, halfW: POT.screenHalfW + 0.055, halfH: POT.screenHalfH + 0.055, exponent: 5, lift: d => 0.024 * ease(d / 0.32) }),
    bezelMaterial,
  )
  bezel.castShadow = false
  bezel.receiveShadow = false
  const glass = new MeshPhysicalMaterial({
    color: '#050506', roughness: 0.07, metalness: 0, clearcoat: 1, clearcoatRoughness: 0.03,
    emissive: new Color('#ffffff'), emissiveIntensity: 1.3, envMapIntensity: 0.28,
  })
  const screen = new Mesh(
    buildPatch({ cy: POT.screenY, halfW: POT.screenHalfW, halfH: POT.screenHalfH, exponent: 5, lift: d => 0.027 + 0.014 * ease(d / 0.1) }),
    glass,
  )
  screen.receiveShadow = false
  group.add(bezel, screen)
  return { group, glass, ceramic }
}
