/**
 * Official pot plants share the same body, screen and animated parts.
 * Plant art can change without changing expressions, hit testing or dragging.
 */
import { COLOR, type Palette } from './geometry'
import whiteFlower from './assets/white-flower/white_flower.webp?inline'

export type SkinId = 'pot' | 'pot-white-flower'

/** Art that needs gradient ids gets them from the instance id, so two robots on one page never share a definition. */
export type Markup = string | ((id: string) => string)

/** How the face sits on the shell: light on a dark screen by default, or ink painted straight onto the surface. */
export interface FaceStyle {
  palette: Palette
  /** neon: light bleeds around the lines. ink: the lines sit on the surface with only a soft edge. */
  glow: 'neon' | 'ink'
  /** Glass sheen over the screen. */
  gloss: boolean
}
export const NEON: FaceStyle = { palette: COLOR, glow: 'neon', gloss: true }

/** A shell that is not the robot's head: its own body outline and plant, around the same screen. */
export interface Art {
  /** Replaces the feet; the side pods are not drawn. Empty for none. */
  feet: string
  /** Outline of the body, used for the fill, the outline stroke and the clip. */
  head: string
  /** Shapes over the body fill and under the screen frame, clipped to the body: rim, soil, shading. */
  trim: Markup
  /** Shapes over the body and outside its clip, under the screen frame: the stem that rises out of the soil. */
  front: Markup
  /** Behind everything: a glow around the shell. Its data-part is "aura" so clicks go through it. */
  back?: Markup
  /** Extra gradients and filters the other parts refer to. */
  defs?: Markup
  /** Value of the body's fill; defaults to the shell gradient. */
  fill?: Markup
  /** Replaces the bezel and the dark screen, for shells whose face is painted straight onto the body. */
  panel?: Markup
}

export interface Skin {
  id: SkinId
  label: string
  face?: FaceStyle
  art?: Art
  /** Shrinks the whole robot about its feet, so long ears still fit in the same 320×280 view. */
  scale: number
  /** Left ear, in the 320×280 view; the right ear is its mirror. Both rotate around the shared ear pivot. */
  earOuter: string
  earInner: string
  earHighlight: string
  /** Colour under the lit part of the ear, and the four stops of the light itself, bright to deep. */
  earBase: string
  light: [string, string, string, string]
  /** Centre and radius of the light gradient, in the view. */
  lightAt: [number, number, number]
  halo: string
  /** Glow behind the ear: centre and radii, in the ear's own coordinates. */
  haloAt: [number, number, number, number]
  /** Point the ears turn around; the right ear mirrors it about the middle. */
  earPivot: [number, number]
  /** Head shell gradient, light to dark. */
  shell: [string, string, string, string]
  shade: string
  /** Rim around the head and ears. */
  stroke: string
  earStroke: string
  earShell: [string, string]
  bezel: [string, string, string]
  pod: [string, string, string]
  podCore: [string, string, string]
  podStroke: string
}

const pot: Skin = {
    id: 'pot', label: '盆栽', scale: 0.76,
    art: {
      feet: '',
      // A round-bellied porcelain pot: a small mouth, a full belly and a soft base, with no flared rim or straight sides.
      head: 'M70 56A90 10 0 0 1 250 56C276 62 296 104 296 156C296 214 268 252 222 252H98C52 252 24 214 24 156C24 104 44 62 70 56Z',
      trim:
        '<ellipse cx="160" cy="72" rx="94" ry="9" fill="#6b4a30" opacity="0.1"/>' +
        '<ellipse cx="160" cy="56" rx="90" ry="10" fill="#ece3d8"/>' +
        '<ellipse cx="160" cy="57" rx="81" ry="7.4" fill="#4b3022"/>' +
        '<ellipse cx="122" cy="56" rx="5" ry="1.6" fill="#6b4730"/><ellipse cx="150" cy="60" rx="4" ry="1.4" fill="#6b4730"/>' +
        '<ellipse cx="192" cy="58" rx="5" ry="1.6" fill="#6b4730"/><ellipse cx="214" cy="55" rx="4" ry="1.4" fill="#6b4730"/>' +
        [[104, 57], [134, 61], [190, 61], [220, 57], [176, 54]].map(([x, y]) => `<circle cx="${x}" cy="${y}" r="3.2" fill="#c98b4a"/><circle cx="${x - 1}" cy="${y - 1}" r="1.1" fill="#e8b27a"/>`).join('') +
        '<ellipse cx="110" cy="50" rx="30" ry="1.8" fill="#fff" opacity="0.85"/>' +
        '<path d="M40 114C34 130 34 148 40 164" fill="none" stroke="#fff" stroke-width="5" stroke-linecap="round" opacity="0.6"/>',
      front:
        '<ellipse cx="160" cy="57" rx="9" ry="2.6" fill="#3b2418" opacity="0.6"/>' +
        '<path d="M160 57C161 41 159 20 160 4" fill="none" stroke="#5aa92f" stroke-width="9" stroke-linecap="round"/>' +
        '<path d="M157.5 55C158.5 39 157 20 158 6" fill="none" stroke="#a6dd6b" stroke-width="2.4" stroke-linecap="round" opacity="0.8"/>',
    },
    earOuter: 'M160 6C150 -34 112 -66 64 -62C70 -18 112 24 160 6Z',
    earInner: 'M156 2C148 -26 118 -52 78 -52C84 -20 116 12 156 2Z',
    earHighlight: 'M152 0C130 -18 104 -34 82 -48',
    earBase: '#3f8a22', light: ['#f6ffd8', '#d4f58a', '#8fd94a', '#4fae2e'], lightAt: [112, -30, 70], halo: '#b6f06a', haloAt: [108, -36, 64, 48], earPivot: [160, 4],
    shell: ['#ffffff', '#fbf7f2', '#ece3d9', '#d3c6b8'], shade: '#6b4a30',
    stroke: '#d0c3b4', earStroke: '#4a8f26', earShell: ['#b9e66c', '#6cb62f'],
    bezel: ['#f6f0e8', '#e4d9cb', '#c9bba9'],
    pod: ['#d9a982', '#f8e4d3', '#dcb08c'], podCore: ['#c58f68', '#eed2ba', '#bd8660'], podStroke: '#b97f58',
}

export const SKINS: Record<SkinId, Skin> = {
  pot: { ...pot, label: '绿叶' },
  'pot-white-flower': {
    ...pot, id: 'pot-white-flower', label: '白色小雏菊',
    art: {
      ...pot.art!,
      front: id => `<defs><clipPath id="${id}plantClip"><rect x="64" y="-82" width="192" height="140"/></clipPath></defs>` +
        `<image data-part="plant" href="${whiteFlower}" x="85" y="-82" width="150" height="150" clip-path="url(#${id}plantClip)"/>` +
        '<path data-part="plant-hit" d="M160 -70C179 -74 198 -58 194 -29C216 -30 235 -10 225 13L230 48L200 57H120L92 46L100 10C87 -10 104 -40 125 -37C128 -58 139 -73 160 -70Z" fill="#000" opacity="0"/>',
    },
    earOuter: '', earInner: '', earHighlight: '',
    haloAt: [160, 4, 0, 0],
  },
}

export const faceStyleOf = (skin: Skin): FaceStyle => skin.face ?? NEON

export const SKIN_IDS = Object.keys(SKINS) as SkinId[]
export const DEFAULT_SKIN: SkinId = 'pot'
export const isSkinId = (value: unknown): value is SkinId => typeof value === 'string' && Object.hasOwn(SKINS, value)
export const skinOf = (value: unknown): Skin => SKINS[isSkinId(value) ? value : DEFAULT_SKIN]
