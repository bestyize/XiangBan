import { C, heartPath, starPath, stroke } from './palette.ts'

const star = (x: number, y: number, r: number, fill: string = C.lamp) =>
  `<path d="${starPath(x, y, r)}" fill="${fill}" stroke="${C.ink}" stroke-width="2.5" stroke-linejoin="round"/>`

/** Accessories drawn on top of the face, in a 240×240 viewBox. */
export const props = {
  zzz: `<path d="M176 44 H194 L176 64 H194" ${stroke(4.5)}/><path d="M202 20 H214 L202 34 H214" ${stroke(3.5)}/>`,
  sweat: `<path d="M188 82 Q200 101 188 108 Q176 101 188 82 Z" fill="${C.tear}" stroke="${C.ink}" stroke-width="3"/>`,
  sparkles: star(36, 58, 13) + star(206, 70, 9) + star(214, 36, 6),
  heart: `<path d="${heartPath(204, 52, 13)}" fill="${C.blush}" stroke="${C.ink}" stroke-width="3"/>`,
  exclaim: `<path d="M208 26 L208 56" ${stroke(8)}/><circle cx="208" cy="72" r="5" fill="${C.ink}"/>`,
  question:
    `<path d="M196 38 Q196 22 210 22 Q224 22 224 36 Q224 46 212 50 L212 58" ${stroke(6)}/>` +
    `<circle cx="212" cy="72" r="4.5" fill="${C.ink}"/>`,
  crown: `<path d="M94 72 L98 38 L111 56 L120 30 L129 56 L142 38 L146 72 Z" fill="${C.lamp}" stroke="${C.ink}" stroke-width="4" stroke-linejoin="round"/>`,
} as const

export type Prop = keyof typeof props
