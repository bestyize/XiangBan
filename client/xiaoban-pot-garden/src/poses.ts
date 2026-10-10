export type Expression = 'idle' | 'happy' | 'excited' | 'sleepy' | 'sad' | 'surprised' | 'curious' | 'love'

export const EXPRESSIONS: Expression[] = ['idle', 'happy', 'excited', 'sleepy', 'sad', 'surprised', 'curious', 'love']

export const LABELS: Record<Expression, string> = {
  idle: '平静', happy: '开心', excited: '兴奋', sleepy: '困倦', sad: '难过', surprised: '惊讶', curious: '好奇', love: '喜欢',
}

/** How the plant and pot hold themselves for an expression. Leaf values are added to the leaf's own resting shape. */
export interface Pose {
  /** Radians the left and right leaves are raised (positive) or lowered. */
  leftLift: number
  rightLift: number
  /** Extra droop of the blade tips. */
  leftBend: number
  rightBend: number
  /** How far the right leaf's far edge rolls over, 0 to 1.4. */
  curl: number
  /** Lean of the stem about the soil, radians; positive leans right. */
  stemLean: number
  /** Roll of the whole pot, radians. */
  tilt: number
  /** Vertical squash of the pot (negative flattens it). */
  squash: number
  /** Height of the hops, and how many per second. */
  hop: number
  hopRate: number
  /** Multiplier on the leaves' idle sway. */
  sway: number
  /** Show the little burst lines beside the plant. */
  sparkles: boolean
}

const base: Pose = {
  leftLift: 0, rightLift: 0, leftBend: 0, rightBend: 0, curl: 1.15, stemLean: 0, tilt: 0, squash: 0, hop: 0, hopRate: 0, sway: 1, sparkles: false,
}

const poses: Record<Expression, Partial<Pose>> = {
  idle: {},
  happy: { leftLift: 0.14, rightLift: 0.12, leftBend: -0.12, rightBend: -0.1, hop: 0.035, hopRate: 1.6, sway: 1.5, sparkles: true },
  excited: { leftLift: 0.34, rightLift: 0.32, leftBend: -0.25, rightBend: -0.2, curl: 0.8, hop: 0.075, hopRate: 3.1, sway: 2.4, sparkles: true },
  sleepy: { leftLift: -0.42, rightLift: -0.4, leftBend: 0.35, rightBend: 0.3, curl: 0.7, stemLean: 0.05, tilt: 0.04, squash: -0.015, sway: 0.35 },
  sad: { leftLift: -0.5, rightLift: -0.46, leftBend: 0.55, rightBend: 0.5, curl: 0.5, stemLean: -0.04, squash: -0.02, sway: 0.4 },
  surprised: { leftLift: 0.46, rightLift: 0.44, leftBend: -0.3, rightBend: -0.26, curl: 0.6, hop: 0.05, hopRate: 0.9, sway: 0.5 },
  curious: { leftLift: 0.32, rightLift: -0.12, leftBend: -0.1, rightBend: 0.15, curl: 1.3, stemLean: 0.07, tilt: -0.07, sway: 1 },
  love: { leftLift: 0.2, rightLift: 0.2, leftBend: -0.1, rightBend: -0.1, curl: 1.0, hop: 0.03, hopRate: 1.1, sway: 1.7, sparkles: true },
}

export const poseFor = (expression: Expression): Pose => ({ ...base, ...poses[expression] })

export function mixPose(from: Pose, to: Pose, t: number): Pose {
  const out = { ...to }
  for (const key of Object.keys(from) as (keyof Pose)[]) {
    if (typeof from[key] === 'number') (out[key] as number) = (from[key] as number) + ((to[key] as number) - (from[key] as number)) * t
  }
  return out
}

export const isExpression = (value: unknown): value is Expression => typeof value === 'string' && (EXPRESSIONS as string[]).includes(value)
