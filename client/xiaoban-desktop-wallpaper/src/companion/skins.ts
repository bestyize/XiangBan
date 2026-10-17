/**
 * Official shells for the robot. A shell changes the ears and the colours only: the screen, the symbol anchor
 * and the part names stay the same, so expressions, ear motion, hit testing and dragging work in every shell.
 */
export type SkinId = 'cat' | 'bunny' | 'pot'

/** A shell that is not the robot's head: its own body outline and plant, around the same screen. */
export interface Art {
  /** Replaces the feet; the side pods are not drawn. Empty for none. */
  feet: string
  /** Outline of the body, used for the fill, the outline stroke and the clip. */
  head: string
  /** Shapes over the body fill and under the screen frame, clipped to the body: rim, soil, shading. */
  trim: string
  /** Shapes over the body and outside its clip, under the screen frame: the stem that rises out of the soil. */
  front: string
}

export interface Skin {
  id: SkinId
  label: string
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

export const SKINS: Record<SkinId, Skin> = {
  cat: {
    id: 'cat', label: '猫耳', scale: 1,
    earOuter: 'M48 124C43 94 52 52 69 22Q75 12 84 17C106 32 128 56 145 88L122 114Z',
    earInner: 'M58 114C55 90 62 58 74 33Q78 26 84 30C102 44 119 64 133 92L112 110Z',
    earHighlight: 'M66 100C64 80 69 58 78 40',
    earBase: '#a8641f', light: ['#fffbe0', '#ffe08a', '#ffb43f', '#f28a1c'], lightAt: [98, 96, 78], halo: '#ffbe55', haloAt: [92, 62, 56, 52], earPivot: [104, 98],
    shell: ['#fffdf8', '#f6ebdb', '#e4cfae', '#c9aa80'], shade: '#6e4c27',
    stroke: '#d2b085', earStroke: '#d8bb94', earShell: ['#fbf3e6', '#e2c9a4'],
    bezel: ['#f4e2c3', '#d9b682', '#b98c55'],
    pod: ['#d9bf98', '#fbf4e8', '#dcc29c'], podCore: ['#c9a87b', '#efe0c8', '#c49f70'], podStroke: '#c9a273',
  },
  bunny: {
    id: 'bunny', label: '兔耳', scale: 0.82,
    earOuter: 'M66 118C46 84 44 20 64 -26Q76 -46 90 -26C108 18 116 84 108 118Z',
    earInner: 'M76 106C62 78 60 28 74 -10Q82 -26 90 -10C102 26 104 78 98 106Z',
    earHighlight: 'M72 96C66 70 66 30 76 0',
    earBase: '#a8641f', light: ['#fffbe0', '#ffe08a', '#ffb43f', '#f28a1c'], lightAt: [98, 96, 78], halo: '#ffbe55', haloAt: [92, 62, 56, 52], earPivot: [104, 98],
    shell: ['#ffffff', '#fdf1f5', '#f1d6df', '#dcb4c2'], shade: '#7a4a5a',
    stroke: '#e0bccb', earStroke: '#e3c4d0', earShell: ['#ffffff', '#f0d4de'],
    bezel: ['#fbe6ee', '#ebc3d1', '#cf98ab'],
    pod: ['#e6c6d2', '#fff6f9', '#ebcdd8'], podCore: ['#dbb3c2', '#fbe9f0', '#d6aabb'], podStroke: '#d9b0bf',
  },
  pot: {
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
  },
}

export const SKIN_IDS = Object.keys(SKINS) as SkinId[]
export const DEFAULT_SKIN: SkinId = 'cat'
export const isSkinId = (value: unknown): value is SkinId => typeof value === 'string' && Object.hasOwn(SKINS, value)
export const skinOf = (value: unknown): Skin => SKINS[isSkinId(value) ? value : DEFAULT_SKIN]
