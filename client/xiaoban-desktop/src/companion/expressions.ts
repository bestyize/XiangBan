import {
  BASE_FACE, BASE_LOOK, withFace,
  type DeepPartial, type Eye, type Face, type Look, type SymbolName,
} from './params.ts'
import type { Frame } from './render.ts'

export interface Expression {
  label: string
  en: string
  face: DeepPartial<Face>
  look?: Partial<Look>
}

const both = (eye: Partial<Eye>): DeepPartial<Face> => ({ eyeL: eye, eyeR: eye })
/** ∩ arcs: the top half of a flattened ring. */
const smileEyes: Partial<Eye> = { span: 0.5, arcMid: -90, w: 22, h: 16, thick: 0.5, y: 5 }
const solid: Partial<Eye> = { thick: 1 }
const cat = (w: number, depth: number) => ({ w, cat: depth, curve: 0, open: 0, tongue: 0 })
const frown = (w: number, depth: number) => ({ w, curve: -depth, cat: 0, open: 0 })
const openMouth = (w: number, open: number) => ({ w, cat: 2.6, curve: 0, open, tongue: 1 })

/** The 25 base expressions, ordered as in the design sheet. */
export const EXPRESSIONS = {
  idle: { label: '平静', en: 'Idle', face: {} },
  happy: { label: '开心', en: 'Happy', face: { ...both(smileEyes), blush: 0.9 }, look: { blink: false } },
  excited: {
    label: '兴奋', en: 'Excited',
    face: { ...both({ w: 22, h: 21, thick: 0.38 }), mouth: openMouth(8, 9), blush: 0.9, bounce: 2.5, bounceHz: 2.2, earL: -6, earR: -6 },
    look: { glyphL: 'chevron', glyphR: 'chevron', symbol: 'sparks', blink: false, saccade: false },
  },
  laughing: {
    label: '大笑', en: 'Laughing',
    face: { ...both({ w: 23, h: 22, thick: 0.38 }), mouth: openMouth(10, 13), blush: 1, bounce: 4.5, bounceHz: 3.2, earL: -8, earR: -8 },
    look: { glyphL: 'chevron', glyphR: 'chevron', symbol: 'sparks', blink: false, saccade: false },
  },
  proud: {
    label: '得意', en: 'Proud',
    face: { eyeR: { w: 22, h: 21, thick: 0.38 }, mouth: cat(8, 3.4), tilt: -4, earL: -4 },
    look: { glyphR: 'chevron', symbol: 'sparkle', blink: false },
  },
  curious: {
    label: '好奇', en: 'Curious',
    face: { ...both({ ...solid, pupil: 0.56, pupilX: 0.44, pupilY: -0.16 }), mouth: { w: 3.6, curve: 0, cat: 0, open: 3.4, tongue: 0, x: 2 }, tilt: 6, gazeX: 3, earR: -8 },
    look: { symbol: 'question' },
  },
  thinking: {
    label: '思考', en: 'Thinking',
    face: { ...both({ ...solid, w: 22, h: 19, lidTop: -0.12, y: 4, pupil: 0.34, pupilY: 0.42, pupilX: 0.2 }), mouth: { w: 7, curve: 0, cat: 0, x: 3 }, gazeX: 4, gazeY: -3, tilt: -3 },
    look: { symbol: 'dots', saccade: false },
  },
  confused: {
    label: '困惑', en: 'Confused',
    face: {
      eyeL: { ...solid, pupil: 0.45, pupilX: 0.3, pupilY: 0.15, lidTop: -0.35, lidTilt: -0.3 },
      eyeR: { ...solid, pupil: 0.5, pupilX: -0.1, pupilY: 0.15, lidTop: -0.55, lidTilt: 0.25 },
      mouth: frown(5, 3), tilt: -7, earL: 6,
    },
    look: { symbol: 'question' },
  },
  surprised: {
    label: '惊讶', en: 'Surprised',
    face: { ...both({ w: 26, h: 27, thick: 0.3 }), mouth: { w: 3.4, curve: 0, cat: 0, open: 4, tongue: 0 }, lift: -3, earL: -10, earR: -10 },
    look: { symbol: 'exclaim', blink: false, saccade: false },
  },
  shy: {
    label: '害羞', en: 'Shy',
    face: { ...both({ ...smileEyes, w: 20, h: 14 }), mouth: cat(7, 3.4), blush: 0.85, blushLines: 1, gazeY: 3, tilt: 5, earL: 6, earR: 6 },
    look: { blink: false },
  },
  love: {
    label: '喜爱', en: 'Love',
    face: { ...both({ w: 23 }), mouth: cat(10, 4.5), blush: 0.7, blushLines: 0.9, bounce: 1.4, bounceHz: 1.3 },
    look: { glyphL: 'heart', glyphR: 'heart', blink: false },
  },
  sleepy: {
    label: '困倦', en: 'Sleepy',
    face: { ...both({ ...solid, w: 25, h: 15, lidTop: 0.05, y: 2 }), mouth: cat(5, 2.6), earL: 10, earR: 10, earGlow: 0.7, tilt: 3 },
    look: { symbol: 'bubble', blink: false, saccade: false },
  },
  sleeping: {
    label: '睡眠', en: 'Sleeping',
    face: { ...both({ span: 0.5, arcMid: 90, w: 21, h: 14, thick: 0.5, y: -2 }), mouth: cat(8, 3.6), earL: 14, earR: 14, earGlow: 0.45, lift: 2 },
    look: { symbol: 'zzz', blink: false, saccade: false },
  },
  sad: {
    label: '难过', en: 'Sad',
    face: {
      eyeL: { ...solid, pupil: 0.5, pupilX: 0.32, pupilY: 0.2, lidTop: -0.45, lidTilt: -0.5 },
      eyeR: { ...solid, pupil: 0.5, pupilX: -0.32, pupilY: 0.2, lidTop: -0.45, lidTilt: -0.5 },
      mouth: frown(6, 3.4), gazeY: 3, earL: 12, earR: 12, earGlow: 0.75,
    },
  },
  worried: {
    label: '担心', en: 'Worried',
    face: {
      eyeL: { ...solid, pupil: 0.46, pupilX: 0.26, pupilY: 0.24, lidTop: -0.3, lidTilt: -0.7 },
      eyeR: { ...solid, pupil: 0.46, pupilX: -0.26, pupilY: 0.24, lidTop: -0.3, lidTilt: -0.7 },
      mouth: frown(6, 3), tilt: -3, earL: 8, earR: 8,
    },
  },
  angry: {
    label: '生气', en: 'Angry',
    face: { ...both({ ...solid, w: 25, h: 21, lidTop: -0.2, lidTilt: 0.8 }), mouth: frown(7, 3.4), earL: -12, earR: -12 },
    look: { symbol: 'anger', saccade: false },
  },
  annoyed: {
    label: '不耐烦', en: 'Annoyed',
    face: { ...both({ ...solid, w: 24, h: 18, lidTop: -0.05, lidTilt: 0.12, pupil: 0.44, pupilX: 0.34, pupilY: 0.38 }), mouth: frown(6, 2.6), gazeX: 3 },
    look: { symbol: 'anger', saccade: false },
  },
  crying: {
    label: '哭泣', en: 'Crying',
    face: { ...both({ span: 0.42, arcMid: -90, w: 20, h: 13, thick: 0.55, y: 2 }), mouth: frown(6, 3), tears: 1, earL: 14, earR: 14, earGlow: 0.7 },
    look: { blink: false, saccade: false },
  },
  embarrassed: {
    label: '尴尬', en: 'Embarrassed',
    face: { ...both({ w: 21, h: 21, thick: 0.34 }), mouth: cat(9, 3.2), blush: 0.8, blushLines: 1, gazeX: -3, tilt: -3 },
    look: { symbol: 'sweat' },
  },
  listening: {
    label: '倾听', en: 'Listening',
    face: { ...both({ ...solid, w: 22, h: 23, pupil: 0.3, pupilX: 0.36, pupilY: -0.38 }), mouth: cat(5, 2.4), tilt: 8, earR: -12, earL: 4 },
    look: { symbol: 'listen', saccade: false },
  },
  speaking: {
    label: '说话', en: 'Speaking',
    face: { ...both(smileEyes), mouth: openMouth(8, 6), talk: 0.7 },
    look: { symbol: 'sound', blink: false },
  },
  greeting: {
    label: '打招呼', en: 'Greeting',
    face: { ...both(smileEyes), mouth: openMouth(8, 8), bounce: 1.6, bounceHz: 1.6, tilt: -5, earL: -10 },
    look: { symbol: 'wave', blink: false },
  },
  waiting: {
    label: '等待', en: 'Waiting',
    face: { ...both({ dash: 1, thick: 0.3 }), mouth: cat(7, 3), spin: 0.35 },
    look: { blink: false, saccade: false },
  },
  offline: {
    label: '离线', en: 'Offline',
    face: { ...both({ w: 20, h: 0, thick: 0.36 }), mouth: cat(6, 2.6), screen: 0.5, earGlow: 0.22, earL: 10, earR: 10 },
    look: { symbol: 'ban', blink: false, saccade: false },
  },
  charging: {
    label: '充电', en: 'Charging',
    face: { ...both(smileEyes), mouth: cat(7, 3), glowPulse: 0.6, earGlow: 0.95 },
    look: { symbol: 'bolt', blink: false, saccade: false },
  },
} satisfies Record<string, Expression>

export type ExpressionName = keyof typeof EXPRESSIONS
export const EXPRESSION_NAMES = Object.keys(EXPRESSIONS) as ExpressionName[]

/** An expression plus optional per-use tweaks: a different screen symbol, a badge number, face overrides. */
export interface RobotSpec {
  expression: ExpressionName
  symbol?: SymbolName | null
  badge?: string
  face?: DeepPartial<Face>
}

export type RobotInput = ExpressionName | RobotSpec

export function toSpec(input: RobotInput): RobotSpec {
  return typeof input === 'string' ? { expression: input } : input
}

export interface Target { face: Face; look: Look }

export function resolve(input: RobotInput): Target {
  const spec = toSpec(input)
  const expr: Expression = EXPRESSIONS[spec.expression] ?? EXPRESSIONS.idle
  const face = withFace(withFace(BASE_FACE, expr.face), spec.face)
  const look: Look = { ...BASE_LOOK, ...expr.look }
  if (spec.symbol !== undefined) look.symbol = spec.symbol
  if (spec.badge) {
    look.symbol = 'badge'
    look.badge = spec.badge
  }
  return { face, look }
}

/** A still frame of an expression, for thumbnails, data URLs and reduced motion. */
export function stillFrame(input: RobotInput, t = 0.6): Frame {
  const { face, look } = resolve(input)
  return {
    face, glyphL: look.glyphL, glyphR: look.glyphR, squashL: 1, squashR: 1,
    symbol: look.symbol, symbolP: 1, badge: look.badge, t, dash: 0, breath: 0,
  }
}
