/** Fixed layout of the robot in its 320×280 viewBox. */
export const VIEW = { w: 320, h: 280 } as const

export const HEAD = { x: 38, y: 76, w: 244, h: 164, r: 66 } as const
export const BEZEL = { x: 53, y: 90, w: 214, h: 137, r: 50 } as const
export const SCREEN = { x: 58, y: 95, w: 204, h: 127, r: 45 } as const

export const FACE_CENTER = { x: 160, y: 158 } as const
export const EYE_OFFSET = { x: 50, y: -7 } as const
export const MOUTH_OFFSET = { x: 0, y: 18 } as const
export const CHEEK_OFFSET = { x: 80, y: 20 } as const

/** Top-right corner of the screen where status symbols pop up. */
export const SYMBOL_ANCHOR = { x: 236, y: 117 } as const

/** Pivot the whole body rotates around (between the feet). */
export const BODY_PIVOT = { x: 160, y: 246 } as const

export const COLOR = {
  line: '#eaf9ff',
  lineDim: '#5fb6ff',
  pink: '#ff6f9a',
  blush: '#ff6f91',
  yellow: '#ffd64d',
  red: '#ff5a6e',
  orange: '#ff8a4c',
  green: '#7be07b',
  tear: '#5ec8ff',
  gray: '#b9c3cf',
  screen: '#050608',
  mouth: '#050608',
  hatch: '#ff9ab6',
} as const

/** The colours the face and screen symbols are drawn in. A shell may bring its own (ink on paper instead of light on a dark screen). */
export type Palette = { readonly [K in keyof typeof COLOR]: string }
