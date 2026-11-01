/**
 * The robot's face is a set of continuous parameters, so any two expressions can be blended
 * and animated between. Units are viewBox units of the 320×280 robot (see geometry.ts).
 */

/** One glowing eye: an elliptical ring that can thicken into a solid lens, be cut by lids and hollowed by a pupil. */
export interface Eye {
  /** Outer half-width / half-height. h near 0 collapses the eye into a line. */
  w: number
  h: number
  /** Ring thickness as a fraction of the eye's smaller radius; 1 fills it in. */
  thick: number
  /** Direction (degrees, 0 = right, -90 = up) of the middle of the visible part of the ring. */
  arcMid: number
  /** Visible fraction of the ring: 1 is a full O, 0.5 a ∩ / ∪ arc. */
  span: number
  /** Upper lid cut, relative to h: -1 is the top edge, 0 the middle. */
  lidTop: number
  /** Upper lid slant: positive lowers the inner corner (angry), negative raises it (sad). */
  lidTilt: number
  /** Lower lid cut, relative to h: 1 is the bottom edge. */
  lidBottom: number
  /** Dark hole punched into the eye, as a fraction of its smaller radius; offsets are fractions of w / h. */
  pupil: number
  pupilX: number
  pupilY: number
  /** 0 solid line, 1 broken into dashes (waiting spinner). */
  dash: number
  x: number
  y: number
}

export interface Mouth {
  /** Half-width. */
  w: number
  /** Positive bends the line into a smile, negative into a ^ frown. */
  curve: number
  /** Depth of the ω bumps. */
  cat: number
  /** How far the lower lip drops; above ~2 it reads as an open mouth. */
  open: number
  /** Pink fill of the open mouth, 0..1. */
  tongue: number
  x: number
  y: number
}

export interface Face {
  eyeL: Eye
  eyeR: Eye
  mouth: Mouth
  blush: number
  /** The /// marks on the cheeks. */
  blushLines: number
  tears: number
  /** Where the face looks, shifting features across the screen. */
  gazeX: number
  gazeY: number
  /** Head tilt in degrees, positive leans right. */
  tilt: number
  /** Vertical offset of the whole robot, negative is up. */
  lift: number
  /** Ear angles in degrees, positive droops outward, negative perks up. */
  earL: number
  earR: number
  /** Ear light, 0..1. */
  earGlow: number
  /** Ear light pulse amplitude (charging). */
  glowPulse: number
  /** Screen brightness, 0 off .. 1 normal. */
  screen: number
  /** Dash rotation speed in turns per second. */
  spin: number
  /** Rhythmic hopping (laughing, excited): height and frequency. */
  bounce: number
  bounceHz: number
  /** Built-in talking, 0..1, on top of the speaking input. */
  talk: number
}

export type EyeGlyph = 'lens' | 'chevron' | 'heart'

export type SymbolName =
  | 'sparkle' | 'sparks' | 'question' | 'exclaim' | 'dots' | 'bubble' | 'zzz' | 'anger'
  | 'sound' | 'wave' | 'listen' | 'ban' | 'bolt' | 'sweat' | 'heart'
  | 'note' | 'moon' | 'clock' | 'calendar' | 'flame' | 'crown' | 'coffee' | 'rain'
  | 'envelope' | 'mooncake' | 'suitcase' | 'flower' | 'shield' | 'badge'

/** Discrete parts of an expression that switch rather than blend. */
export interface Look {
  glyphL: EyeGlyph
  glyphR: EyeGlyph
  symbol: SymbolName | null
  /** Text for the 'badge' symbol, e.g. a streak count. */
  badge: string
  blink: boolean
  /** Small idle glances around. */
  saccade: boolean
}

export type DeepPartial<T> = { [K in keyof T]?: T[K] extends object ? DeepPartial<T[K]> : T[K] }

export const BASE_EYE: Eye = {
  w: 23, h: 23, thick: 0.36, arcMid: -90, span: 1,
  lidTop: -1.4, lidTilt: 0, lidBottom: 1.4,
  pupil: 0, pupilX: 0, pupilY: 0, dash: 0, x: 0, y: 0,
}

export const BASE_MOUTH: Mouth = { w: 10, curve: 0, cat: 4.5, open: 0, tongue: 0, x: 0, y: 0 }

export const BASE_FACE: Face = {
  eyeL: BASE_EYE,
  eyeR: BASE_EYE,
  mouth: BASE_MOUTH,
  blush: 0, blushLines: 0, tears: 0,
  gazeX: 0, gazeY: 0, tilt: 0, lift: 0,
  earL: 0, earR: 0, earGlow: 1, glowPulse: 0,
  screen: 1, spin: 0, bounce: 0, bounceHz: 0, talk: 0,
}

export const BASE_LOOK: Look = { glyphL: 'lens', glyphR: 'lens', symbol: null, badge: '', blink: true, saccade: true }

type Tree = { [key: string]: number | Tree }

/** Deep-merges numeric overrides onto a face, returning a new object. */
export function withFace(base: Face, patch: DeepPartial<Face> | undefined): Face {
  return patch ? (merge(base as unknown as Tree, patch as Tree) as unknown as Face) : base
}

function merge(base: Tree, patch: Tree): Tree {
  const out: Tree = { ...base }
  for (const key of Object.keys(patch)) {
    const value = patch[key]
    if (value === undefined) continue
    const current = base[key]
    out[key] = typeof value === 'object' && typeof current === 'object' ? merge(current, value) : value
  }
  return out
}

export function cloneFace(face: Face): Face {
  return { ...face, eyeL: { ...face.eyeL }, eyeR: { ...face.eyeR }, mouth: { ...face.mouth } }
}

export type { Tree as FaceTree }
