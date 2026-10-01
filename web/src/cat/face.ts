import { C, heartPath, starPath, stroke } from './palette.ts'

export type Eyes =
  | 'open' | 'happy' | 'closed' | 'sleepy' | 'smug' | 'worried' | 'sparkle'
  | 'shocked' | 'teary' | 'heart' | 'wink' | 'determined' | 'squint' | 'dizzy'

export type Mouth = 'cat' | 'smile' | 'open' | 'o' | 'flat' | 'wavy' | 'frown' | 'tongue' | 'smirk'

/** side: -1 for the cat's left eye (viewer's left), 1 for the right one. */
type EyeDraw = (x: number, y: number, side: -1 | 1) => string

const pupil = (x: number, y: number) =>
  `<ellipse cx="${x}" cy="${y}" rx="10" ry="13" fill="${C.ink}"/><circle cx="${x + 3.5}" cy="${y - 5}" r="3.6" fill="${C.white}"/>`

const brow = (x: number, y: number, side: -1 | 1, innerUp: boolean) => {
  const inward = -side
  const [outerY, innerY] = innerUp ? [y - 18, y - 25] : [y - 25, y - 17]
  return `<path d="M${x - 11 * inward} ${outerY} L${x + 9 * inward} ${innerY}" ${stroke(5)}/>`
}

const eyes: Record<Exclude<Eyes, 'wink'>, EyeDraw> = {
  open: (x, y) => pupil(x, y),
  happy: (x, y) => `<path d="M${x - 12} ${y + 4} Q${x} ${y - 12} ${x + 12} ${y + 4}" ${stroke(5.5)}/>`,
  closed: (x, y) => `<path d="M${x - 12} ${y - 3} Q${x} ${y + 10} ${x + 12} ${y - 3}" ${stroke(5.5)}/>`,
  sleepy: (x, y) => `<path d="M${x - 12} ${y + 2} Q${x} ${y + 6} ${x + 12} ${y + 2}" ${stroke(5.5)}/>`,
  smug: (x, y) =>
    `<path d="M${x + 12} ${y - 3} L${x - 12} ${y - 3} A12 10 0 0 0 ${x + 12} ${y - 3} Z" fill="${C.ink}"/>` +
    `<circle cx="${x + 4}" cy="${y + 1}" r="2.6" fill="${C.white}"/>`,
  worried: (x, y, side) => pupil(x, y + 2) + brow(x, y, side, true),
  determined: (x, y, side) => pupil(x, y + 2) + brow(x, y, side, false),
  sparkle: (x, y) => `<path d="${starPath(x, y, 15)}" fill="${C.lamp}" stroke="${C.ink}" stroke-width="3" stroke-linejoin="round"/>`,
  shocked: (x, y) =>
    `<circle cx="${x}" cy="${y}" r="12" fill="${C.white}" stroke="${C.ink}" stroke-width="4"/><circle cx="${x}" cy="${y}" r="4" fill="${C.ink}"/>`,
  teary: (x, y) =>
    pupil(x, y) +
    `<path d="M${x + 6} ${y + 12} Q${x + 13} ${y + 23} ${x + 6} ${y + 27} Q${x - 1} ${y + 23} ${x + 6} ${y + 12} Z" fill="${C.tear}" stroke="${C.ink}" stroke-width="2"/>`,
  heart: (x, y) => `<path d="${heartPath(x, y, 11)}" fill="${C.blush}" stroke="${C.ink}" stroke-width="3"/>`,
  squint: (x, y, side) => {
    const d = -side * 9
    return `<path d="M${x - d} ${y - 9} L${x + d} ${y} L${x - d} ${y + 9}" ${stroke(5.5)}/>`
  },
  dizzy: (x, y) =>
    `<circle cx="${x}" cy="${y}" r="11" ${stroke(4)}/><circle cx="${x + 2}" cy="${y - 1}" r="5" ${stroke(3.5)}/>`,
}

export function drawEyes(kind: Eyes, left: [number, number], right: [number, number]) {
  if (kind === 'wink') return eyes.open(...left, -1) + eyes.happy(...right, 1)
  return eyes[kind](...left, -1) + eyes[kind](...right, 1)
}

const catMouth = `<path d="M106 156 Q113 165 120 156 Q127 165 134 156" ${stroke(4.5)}/>`

export const mouths: Record<Mouth, string> = {
  cat: catMouth,
  smile: `<path d="M104 155 Q120 172 136 155" ${stroke(4.5)}/>`,
  open:
    `<path d="M104 154 Q120 184 136 154 Z" fill="${C.mouth}" stroke="${C.ink}" stroke-width="4" stroke-linejoin="round"/>` +
    `<ellipse cx="120" cy="165" rx="7" ry="4" fill="${C.nose}"/>`,
  o: `<ellipse cx="120" cy="163" rx="7" ry="9" fill="${C.mouth}" stroke="${C.ink}" stroke-width="3.5"/>`,
  flat: `<path d="M108 160 L132 160" ${stroke(4.5)}/>`,
  wavy: `<path d="M102 161 Q108 155 114 161 Q120 167 126 161 Q132 155 138 161" ${stroke(4)}/>`,
  frown: `<path d="M106 166 Q120 153 134 166" ${stroke(4.5)}/>`,
  tongue: catMouth + `<path d="M115 160 Q120 173 125 160 Z" fill="${C.nose}" stroke="${C.ink}" stroke-width="2.5" stroke-linejoin="round"/>`,
  smirk: `<path d="M106 161 Q124 166 136 151" ${stroke(4.5)}/>`,
}
