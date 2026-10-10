import { CanvasTexture, CapsuleGeometry, Group, Mesh, MeshBasicMaterial, SRGBColorSpace } from 'three'
import { drawFace, FACE_HEIGHT, FACE_WIDTH } from './face.ts'
import { createPlant, type Plant } from './plant.ts'
import { createPot, type PotParts } from './pot.ts'
import { mixPose, poseFor, type Expression, type Pose } from './poses.ts'

/** The little burst lines beside the plant: where they sit (pot units), the angle of the line, and how long it is. */
const dashes = [
  { x: -0.76, y: 1.54, degrees: -49, length: 0.18 },
  { x: -0.9, y: 1.34, degrees: -24, length: 0.16 },
  { x: 0.84, y: 1.37, degrees: 47, length: 0.18 },
  { x: 0.93, y: 1.16, degrees: 18, length: 0.16 },
]

function createSparkles(): { group: Group; dashes: Mesh[] } {
  const group = new Group()
  const material = new MeshBasicMaterial({ color: '#f5a93f', toneMapped: false })
  const meshes = dashes.map(({ x, y, degrees, length }) => {
    const mesh = new Mesh(new CapsuleGeometry(0.018, length, 6, 12), material)
    mesh.position.set(x, y, 0.3)
    mesh.rotation.z = (degrees * Math.PI) / 180 - Math.PI / 2
    group.add(mesh)
    return mesh
  })
  return { group, dashes: meshes }
}

export class Character {
  readonly group = new Group()
  private readonly pot: PotParts
  private readonly plant: Plant
  private readonly sparkles = createSparkles()
  private readonly canvas = document.createElement('canvas')
  private readonly texture: CanvasTexture
  private base: Expression = 'idle'
  private override: { expression: Expression; left: number; shown: boolean } | null = null
  private lastTime: number | null = null
  private pose: Pose = poseFor('idle')
  private blink = 0
  private nextBlink = 2.5
  private gaze = { x: 0, y: 0, targetX: 0, targetY: 0 }
  private drawn = ''
  private sparkle = 0

  constructor() {
    this.pot = createPot()
    this.plant = createPlant()
    this.canvas.width = FACE_WIDTH
    this.canvas.height = FACE_HEIGHT
    this.texture = new CanvasTexture(this.canvas)
    this.texture.colorSpace = SRGBColorSpace
    this.texture.anisotropy = 8
    this.pot.glass.emissiveMap = this.texture
    this.group.add(this.pot.group, this.plant.group, this.sparkles.group)
    this.paint(0)
  }

  get expression(): Expression {
    return this.override ? this.override.expression : this.base
  }

  setExpression(expression: Expression): void {
    this.base = expression
    this.override = null
  }

  /** A touch: a short burst of delight, then back to what it was doing. */
  poke(): void {
    this.override = { expression: this.base === 'happy' ? 'excited' : 'happy', left: 2.4, shown: false }
  }

  look(x: number, y: number): void {
    this.gaze.targetX = Math.max(-1, Math.min(1, x))
    this.gaze.targetY = Math.max(-1, Math.min(1, y))
  }

  private paint(time: number): void {
    const expression = this.expression
    const animated = expression === 'sleepy' || expression === 'sad' || expression === 'love' ? Math.floor(time * 12) : 0
    const key = [expression, Math.round(this.blink * 5), Math.round(this.gaze.x * 8), Math.round(this.gaze.y * 8), animated].join('|')
    if (key === this.drawn) return
    this.drawn = key
    drawFace(this.canvas, { expression, blink: this.blink, gazeX: this.gaze.x, gazeY: this.gaze.y, time })
    this.texture.needsUpdate = true
  }

  update(dt: number, time: number, calm = false): void {
    const elapsed = this.lastTime === null ? 0 : Math.max(0, time - this.lastTime)
    this.lastTime = time
    if (this.override) {
      // A touch is always shown for at least one frame, however slowly frames arrive.
      if (this.override.shown) this.override.left -= elapsed
      this.override.shown = true
      if (this.override.left <= 0) this.override = null
    }
    const target = poseFor(this.expression)
    if (calm) Object.assign(target, { hop: 0, sway: target.sway * 0.3 })
    this.pose = mixPose(this.pose, target, 1 - Math.exp(-dt * 6))

    this.gaze.x += (this.gaze.targetX - this.gaze.x) * (1 - Math.exp(-dt * 8))
    this.gaze.y += (this.gaze.targetY - this.gaze.y) * (1 - Math.exp(-dt * 8))
    if (time >= this.nextBlink) {
      this.blink = 1
      this.nextBlink = time + 2.4 + ((time * 7.31) % 1) * 3
    } else this.blink = Math.max(0, this.blink - dt * 9)
    this.paint(time)

    const bounce = this.pose.hop * Math.abs(Math.sin(time * Math.PI * this.pose.hopRate))
    this.group.position.y = bounce
    this.group.scale.y = 1 + this.pose.squash + Math.sin(time * 1.8) * 0.005
    this.group.rotation.z = this.pose.tilt - this.gaze.x * 0.015
    this.plant.update(dt, time, this.pose, this.gaze.x)

    this.sparkle += ((this.pose.sparkles ? 1 : 0) - this.sparkle) * (1 - Math.exp(-dt * (this.pose.sparkles ? 9 : 14)))
    this.sparkles.dashes.forEach((dash, i) => {
      const pulse = 1 + 0.12 * Math.sin(time * 7 + i * 1.7)
      dash.scale.setScalar(Math.max(0.0001, this.sparkle * pulse))
    })
    this.sparkles.group.position.y = -bounce
  }
}
