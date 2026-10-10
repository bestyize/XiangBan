import { BufferAttribute, BufferGeometry } from 'three'

/** A leaf blade is a grid: u runs from the base to the tip along the midrib, v across the blade from -1 to 1. */
export const LENGTH_SEGMENTS = 44
export const WIDTH_SEGMENTS = 22

/** Where across the half-blade (0 midrib, 1 edge) an edge curl begins. */
const CURL_START = 0.45

export interface LeafShape {
  /** Midrib length. */
  length: number
  /** Largest half-width. */
  width: number
  /** Angle (radians above horizontal) the leaf leaves the stem at. */
  pitch: number
  /** How far the tip droops: the midrib turns by this much between base and tip. */
  bend: number
  /** V-shaped cross-section; positive lifts both edges. */
  fold: number
  /** Edge curl on the v < 0 and v > 0 sides; 1 rolls the edge up and over so its underside shows. */
  curlA: number
  curlB: number
  /** Roll of the cross-section around the midrib, base to tip. */
  twist: number
  /** Ripple along the edges, as a fraction of the width. */
  wave: number
}

export const defaultLeaf: LeafShape = { length: 0.62, width: 0.25, pitch: 0.5, bend: 0.9, fold: 0.22, curlA: 0, curlB: 0, twist: 0, wave: 0.05 }

/** Ovate outline with a drawn-out tip: widest about 38% along, closed at both ends. */
export function outline(u: number): number {
  const t = Math.min(1, Math.max(0, u))
  return Math.pow(Math.sin(Math.PI * Math.pow(t, 0.72)), 0.85) * (1 - 0.12 * t)
}

export const vertexCount = (LENGTH_SEGMENTS + 1) * (WIDTH_SEGMENTS + 1)

/** Writes the blade into `out` (three numbers per vertex, rows of constant u). */
export function fillLeaf(shape: LeafShape, out: Float32Array): void {
  const dx: number[] = [0]
  const dy: number[] = [0]
  const ds = shape.length / LENGTH_SEGMENTS
  let x = 0
  let y = 0
  for (let i = 1; i <= LENGTH_SEGMENTS; i++) {
    const a = shape.pitch - shape.bend * ((i - 0.5) / LENGTH_SEGMENTS)
    x += Math.cos(a) * ds
    y += Math.sin(a) * ds
    dx.push(x)
    dy.push(y)
  }
  let k = 0
  for (let i = 0; i <= LENGTH_SEGMENTS; i++) {
    const u = i / LENGTH_SEGMENTS
    const angle = shape.pitch - shape.bend * u
    const nx = -Math.sin(angle)
    const ny = Math.cos(angle)
    const w = shape.width * outline(u)
    const roll = shape.twist * u
    const cr = Math.cos(roll)
    const sr = Math.sin(roll)
    for (let j = 0; j <= WIDTH_SEGMENTS; j++) {
      const v = -1 + (2 * j) / WIDTH_SEGMENTS
      const edge = Math.abs(v)
      const curl = (v < 0 ? shape.curlA : shape.curlB) * Math.sqrt(Math.sin(Math.PI * Math.pow(u, 0.9)))
      let reach = edge * w * Math.cos(shape.fold)
      let h = edge * w * Math.sin(shape.fold)
      if (curl > 0.001 && edge > CURL_START) {
        // The outer part of the blade rolls up and over like a sheet of paper curling at its edge.
        const run = (edge - CURL_START) * w
        const radius = 0.2 * w
        const limit = Math.min(curl, 1.5) * Math.PI * 0.8
        const angle = Math.min(run / radius, limit)
        const extra = Math.max(0, run - radius * limit)
        const f = shape.fold
        reach = CURL_START * w * Math.cos(f) + radius * (Math.sin(f + angle) - Math.sin(f)) + extra * Math.cos(f + angle)
        h = CURL_START * w * Math.sin(f) + radius * (Math.cos(f) - Math.cos(f + angle)) + extra * Math.sin(f + angle)
      }
      h += shape.wave * w * Math.sin(u * 15 + v * 3) * edge * edge
      let z = Math.sign(v) * reach
      const zr = z * cr - h * sr
      h = z * sr + h * cr
      z = zr
      out[k++] = dx[i] + nx * h
      out[k++] = dy[i] + ny * h
      out[k++] = z
    }
  }
}

export function createLeafGeometry(shape: LeafShape = defaultLeaf): BufferGeometry {
  const geometry = new BufferGeometry()
  const positions = new Float32Array(vertexCount * 3)
  const uvs = new Float32Array(vertexCount * 2)
  let k = 0
  for (let i = 0; i <= LENGTH_SEGMENTS; i++) {
    for (let j = 0; j <= WIDTH_SEGMENTS; j++) {
      uvs[k++] = i / LENGTH_SEGMENTS
      uvs[k++] = j / WIDTH_SEGMENTS
    }
  }
  const index: number[] = []
  const at = (i: number, j: number) => i * (WIDTH_SEGMENTS + 1) + j
  for (let i = 0; i < LENGTH_SEGMENTS; i++) {
    for (let j = 0; j < WIDTH_SEGMENTS; j++) {
      const a = at(i, j)
      const b = at(i + 1, j)
      const c = at(i, j + 1)
      const d = at(i + 1, j + 1)
      index.push(a, c, b, b, c, d)
    }
  }
  geometry.setAttribute('position', new BufferAttribute(positions, 3))
  geometry.setAttribute('uv', new BufferAttribute(uvs, 2))
  geometry.setIndex(index)
  shapeLeaf(geometry, shape)
  return geometry
}

/** Re-shapes an existing blade in place, so a leaf can bend every frame without new allocations. */
export function shapeLeaf(geometry: BufferGeometry, shape: LeafShape): void {
  const position = geometry.getAttribute('position') as BufferAttribute
  fillLeaf(shape, position.array as Float32Array)
  position.needsUpdate = true
  geometry.computeVertexNormals()
  geometry.computeBoundingSphere()
}
