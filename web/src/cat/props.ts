import { C, heartPath, starPath, stroke } from './palette.ts'

const star = (x: number, y: number, r: number, fill: string = C.lamp) =>
  `<path d="${starPath(x, y, r)}" fill="${fill}" stroke="${C.ink}" stroke-width="2.5" stroke-linejoin="round"/>`

const outlined = `stroke="${C.ink}" stroke-width="4" stroke-linejoin="round"`

/** Accessories drawn on top of the face, in a 240×240 viewBox (head ~38–202 × 62–206, paws at the bottom). */
export const props = {
  zzz: `<path d="M176 44 H194 L176 64 H194" ${stroke(4.5)}/><path d="M202 20 H214 L202 34 H214" ${stroke(3.5)}/>`,
  sweat: `<path d="M188 82 Q200 101 188 108 Q176 101 188 82 Z" fill="${C.tear}" stroke="${C.ink}" stroke-width="3"/>`,
  sparkles: star(36, 58, 13) + star(206, 70, 9) + star(214, 36, 6),
  heart: `<path d="${heartPath(204, 52, 13)}" fill="${C.blush}" stroke="${C.ink}" stroke-width="3"/>`,
  exclaim: `<path d="M208 26 L208 56" ${stroke(8)}/><circle cx="208" cy="72" r="5" fill="${C.ink}"/>`,
  question:
    `<path d="M196 38 Q196 22 210 22 Q224 22 224 36 Q224 46 212 50 L212 58" ${stroke(6)}/>` +
    `<circle cx="212" cy="72" r="4.5" fill="${C.ink}"/>`,
  crown: `<path d="M94 72 L98 38 L111 56 L120 30 L129 56 L142 38 L146 72 Z" fill="${C.lamp}" ${outlined}/>`,
  partyHat:
    `<path d="M104 70 L120 16 L136 70 Z" fill="${C.blush}" ${outlined}/>` +
    `<path d="M110 52 L130 52 M114 38 L126 38" ${stroke(3, C.lamp)}/><circle cx="120" cy="16" r="6" fill="${C.lamp}" stroke="${C.ink}" stroke-width="3"/>`,
  flower:
    [[0, -9], [9, 0], [0, 9], [-9, 0]].map(([dx, dy]) => `<circle cx="${60 + dx}" cy="${62 + dy}" r="7" fill="${C.blush}" stroke="${C.ink}" stroke-width="2.5"/>`).join('') +
    `<circle cx="60" cy="62" r="5" fill="${C.lamp}" stroke="${C.ink}" stroke-width="2.5"/>`,
  note: `<path d="M196 58 V24 L222 18 V50" ${stroke(4)}/><ellipse cx="190" cy="58" rx="8" ry="6" fill="${C.ink}"/><ellipse cx="216" cy="50" rx="8" ry="6" fill="${C.ink}"/>`,
  moon: `<path d="M48 18 A24 24 0 1 0 74 54 A19 19 0 1 1 48 18 Z" fill="${C.lamp}" stroke="${C.ink}" stroke-width="3"/>`,
  flame: `<path d="M206 22 C220 38 222 50 214 62 C208 70 194 70 190 60 C186 50 194 44 196 36 C200 42 204 42 206 22 Z" fill="${C.red}" ${outlined}/><path d="M204 46 C210 52 210 58 204 62 C198 60 198 54 204 46 Z" fill="${C.lamp}"/>`,
  rain: [[30, 40], [52, 18], [196, 30], [218, 58]].map(([x, y]) => `<path d="M${x} ${y} q5 9 0 13 q-5 -4 0 -13 Z" fill="${C.tear}" stroke="${C.ink}" stroke-width="2"/>`).join(''),
  umbrella:
    `<path d="M10 70 Q46 18 82 70 Q73 62 64 70 Q55 62 46 70 Q37 62 28 70 Q19 62 10 70 Z" fill="${C.sky}" ${outlined}/>` +
    `<path d="M46 40 V104 Q46 112 38 112" ${stroke(4)}/>`,
  envelope:
    `<rect x="101" y="182" width="38" height="48" rx="5" fill="${C.red}" ${outlined}/>` +
    `<path d="M101 194 Q120 206 139 194" ${stroke(3, C.ink)}/><circle cx="120" cy="204" r="7" fill="${C.lamp}" stroke="${C.ink}" stroke-width="2.5"/>`,
  mooncake:
    `<circle cx="120" cy="206" r="24" fill="#e0a24f" ${outlined}/>` +
    `<circle cx="120" cy="206" r="13" fill="none" stroke="#a8682a" stroke-width="3"/><path d="M114 206 H126 M120 200 V212" stroke="#a8682a" stroke-width="3" stroke-linecap="round"/>`,
  coffee:
    `<path d="M104 188 H134 V212 Q134 224 122 224 H116 Q104 224 104 212 Z" fill="${C.white}" ${outlined}/>` +
    `<path d="M134 194 Q146 194 146 203 Q146 212 134 212" ${stroke(4)}/><path d="M112 180 q-4 -6 0 -12 M124 180 q-4 -6 0 -12" ${stroke(3)} opacity="0.6"/>`,
  suitcase:
    `<rect x="164" y="186" width="54" height="42" rx="7" fill="${C.sky}" ${outlined}/>` +
    `<path d="M180 186 V176 H202 V186 M164 204 H218" ${stroke(4)}/>`,
  calendar:
    `<rect x="172" y="26" width="52" height="50" rx="7" fill="${C.white}" ${outlined}/>` +
    `<path d="M172 40 H224" ${stroke(4)}/><path d="M184 20 V32 M212 20 V32" ${stroke(4)}/><path d="M186 54 L196 64 L212 46" ${stroke(4, C.red)}/>`,
  clock:
    `<circle cx="200" cy="52" r="22" fill="${C.white}" ${outlined}/>` +
    `<path d="M200 40 V52 L209 58" ${stroke(4)}/><path d="M182 30 L188 36 M218 30 L212 36" ${stroke(4)}/>`,
  shield:
    `<path d="M120 178 L146 186 V202 Q146 222 120 232 Q94 222 94 202 V186 Z" fill="${C.leaf}" ${outlined}/>` +
    `<path d="M108 204 L117 213 L133 195" ${stroke(4.5, C.white)}/>`,
} as const

export type Prop = keyof typeof props

/** A round number badge (milestones), top right. */
export function badge(text: string) {
  const size = text.length >= 3 ? 18 : 24
  return `<circle cx="198" cy="52" r="26" fill="${C.lamp}" stroke="${C.ink}" stroke-width="4"/>` +
    `<text x="198" y="${52 + size * 0.36}" text-anchor="middle" font-family="sans-serif" font-weight="700" font-size="${size}" fill="${C.ink}">${text}</text>`
}
