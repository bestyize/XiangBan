import {
  BackSide, BufferAttribute, BufferGeometry, CatmullRomCurve3, Color, DoubleSide, FrontSide, Group, Mesh, MeshPhysicalMaterial, SphereGeometry, TubeGeometry, Vector3,
} from 'three'
import { createLeafGeometry, defaultLeaf, shapeLeaf, type LeafShape } from './leaf.ts'
import { makeLeafTextures } from './leaf-texture.ts'
import { soilHeight } from './pot.ts'
import { mixPose, poseFor, type Pose } from './poses.ts'

/** A tube whose radius runs from `from` at the first point to `to` at the last, with a colour that follows the same ramp. */
function taperedTube(points: Vector3[], from: number, to: number, colours: [string, string]): BufferGeometry {
  const radial = 16
  const tubular = 28
  const curve = new CatmullRomCurve3(points)
  const geometry = new TubeGeometry(curve, tubular, 1, radial, false)
  const position = geometry.getAttribute('position') as BufferAttribute
  const colors = new Float32Array(position.count * 3)
  const a = new Color(colours[0])
  const b = new Color(colours[1])
  const colour = new Color()
  const centre = new Vector3()
  for (let i = 0; i <= tubular; i++) {
    const t = i / tubular
    curve.getPointAt(t, centre)
    const radius = from + (to - from) * t
    colour.copy(a).lerp(b, t)
    for (let j = 0; j <= radial; j++) {
      const k = i * (radial + 1) + j
      position.setXYZ(
        k, centre.x + (position.getX(k) - centre.x) * radius, centre.y + (position.getY(k) - centre.y) * radius, centre.z + (position.getZ(k) - centre.z) * radius,
      )
      colors.set([colour.r, colour.g, colour.b], k * 3)
    }
  }
  geometry.setAttribute('color', new BufferAttribute(colors, 3))
  geometry.computeVertexNormals()
  return geometry
}

const leafShapes: Record<'left' | 'right', LeafShape> = {
  left: { ...defaultLeaf, length: 0.84, width: 0.36, pitch: 0.82, bend: 0.62, fold: 0.18, twist: 0.1, wave: 0.06 },
  right: { ...defaultLeaf, length: 0.8, width: 0.35, pitch: 0.5, bend: 0.62, fold: 0.26, twist: -0.1, wave: 0.07 },
}

interface Leaf {
  side: -1 | 1
  group: Group
  geometry: BufferGeometry
  shape: LeafShape
  phase: number
}

export interface Plant {
  group: Group
  update(dt: number, time: number, pose: Pose, lookX: number): void
  /** Where the leaves meet the stem, in pot space. */
  joint: Vector3
}

export function createPlant(): Plant {
  const textures = makeLeafTextures()
  const common = { roughness: 0.46, clearcoat: 0.28, clearcoatRoughness: 0.5, envMapIntensity: 0.75, bumpMap: textures.bump, bumpScale: 1.6 }
  const top = new MeshPhysicalMaterial({
    ...common, map: textures.top, side: FrontSide, sheen: 0.25, sheenColor: new Color('#c8f06a'), sheenRoughness: 0.6,
    emissive: new Color('#1f5a08'), emissiveIntensity: 0.1, shadowSide: DoubleSide,
  })
  const under = new MeshPhysicalMaterial({
    ...common, map: textures.under, side: BackSide, sheen: 0.2, sheenColor: new Color('#efffa0'), sheenRoughness: 0.7,
    emissive: new Color('#4a7a1a'), emissiveIntensity: 0.08, shadowSide: DoubleSide,
  })
  const stemMaterial = new MeshPhysicalMaterial({ vertexColors: true, roughness: 0.42, clearcoat: 0.3, clearcoatRoughness: 0.35, sheen: 0.15, sheenColor: new Color('#c8f06a') })

  const root = new Group()
  const pivot = new Group()
  pivot.position.set(0, soilHeight(0, 0) - 0.01, 0)
  root.add(pivot)
  const stemPoints = [new Vector3(-0.012, 0, 0.01), new Vector3(-0.034, 0.12, 0.012), new Vector3(-0.005, 0.25, 0.004), new Vector3(0.03, 0.37, 0)]
  const stem = new Mesh(taperedTube(stemPoints, 0.056, 0.034, ['#3f8a17', '#6fb52a']), stemMaterial)
  stem.castShadow = true
  pivot.add(stem)
  const jointGroup = new Group()
  jointGroup.position.copy(stemPoints[3])
  pivot.add(jointGroup)
  const knuckle = new Mesh(new SphereGeometry(0.036, 20, 14), new MeshPhysicalMaterial({ color: '#6fb52a', roughness: 0.42, clearcoat: 0.3, clearcoatRoughness: 0.35 }))
  knuckle.castShadow = true
  jointGroup.add(knuckle)

  const leaves: Leaf[] = []
  for (const [side, shape, phase] of [[-1, leafShapes.left, 0], [1, leafShapes.right, 2.1]] as const) {
    const group = new Group()
    group.scale.x = side
    group.rotation.order = 'ZYX'
    const length = 0.15
    const heading = shape.pitch * 0.85
    const end = new Vector3(Math.cos(heading) * length, Math.sin(heading) * length, 0)
    const petiole = new Mesh(
      taperedTube([new Vector3(0, 0, 0), new Vector3(end.x * 0.28, end.y * 0.52, 0), end], 0.026, 0.018, ['#5aa421', '#6fb52a']),
      stemMaterial,
    )
    petiole.castShadow = true
    const blade = new Group()
    blade.position.copy(end)
    const geometry = createLeafGeometry(shape)
    const upper = new Mesh(geometry, top)
    const lower = new Mesh(geometry, under)
    for (const mesh of [upper, lower]) {
      mesh.castShadow = true
      mesh.receiveShadow = true
      blade.add(mesh)
    }
    group.add(petiole, blade)
    jointGroup.add(group)
    leaves.push({ side, group, geometry, shape, phase })
  }

  let current = poseFor('idle')
  return {
    group: root,
    joint: new Vector3(stemPoints[3].x, pivot.position.y + stemPoints[3].y, stemPoints[3].z),
    update(dt, time, pose, lookX) {
      current = mixPose(current, pose, 1 - Math.exp(-dt * 5))
      pivot.rotation.z = -(current.stemLean + lookX * 0.035) + Math.sin(time * 0.9) * 0.012 * current.sway
      for (const leaf of leaves) {
        const right = leaf.side === 1
        const sway = (Math.sin(time * 1.25 + leaf.phase) * 0.05 + Math.sin(time * 2.7 + leaf.phase * 2) * 0.02) * current.sway
        const lift = (right ? current.rightLift : current.leftLift) + sway
        leaf.group.rotation.z = leaf.side * lift
        leaf.group.rotation.x = 0.62
        leaf.group.rotation.y = right ? 0.1 : -0.04
        shapeLeaf(leaf.geometry, {
          ...leaf.shape,
          bend: leaf.shape.bend + (right ? current.rightBend : current.leftBend) + sway * 0.7,
          curlA: right ? current.curl : 0,
        })
      }
    },
  }
}
