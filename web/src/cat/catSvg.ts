import { drawEyes, mouths, type Eyes, type Mouth } from './face.ts'
import { C, stroke } from './palette.ts'
import { props as propArt, type Prop } from './props.ts'

export interface CatSpec {
  eyes: Eyes
  mouth: Mouth
  blush?: boolean
  props?: Prop[]
}

const outline = `stroke="${C.ink}" stroke-width="6" stroke-linejoin="round"`

const body = [
  `<path d="M172 224 C214 218 226 180 204 160" ${stroke(17)}/>`,
  `<path d="M172 224 C214 218 226 180 204 160" ${stroke(9, C.fur)}/>`,
  `<path d="M62 242 C62 198 88 182 120 182 C152 182 178 198 178 242 Z" fill="${C.fur}" ${outline}/>`,
  `<path d="M50 104 L56 34 L106 72 Z" fill="${C.fur}" ${outline}/>`,
  `<path d="M190 104 L184 34 L134 72 Z" fill="${C.fur}" ${outline}/>`,
  `<path d="M64 88 L67 54 L92 72 Z" fill="${C.ear}"/>`,
  `<path d="M176 88 L173 54 L148 72 Z" fill="${C.ear}"/>`,
  `<path d="M38 136 C38 84 74 62 120 62 C166 62 202 84 202 136 C202 184 166 206 120 206 C74 206 38 184 38 136 Z" fill="${C.fur}" ${outline}/>`,
  `<path d="M108 67 L111 85 M120 65 L120 85 M132 67 L129 85" ${stroke(5, C.furDark)}/>`,
  `<ellipse cx="96" cy="226" rx="20" ry="13" fill="${C.fur}" stroke="${C.ink}" stroke-width="5"/>`,
  `<ellipse cx="144" cy="226" rx="20" ry="13" fill="${C.fur}" stroke="${C.ink}" stroke-width="5"/>`,
].join('')

const nose = `<path d="M113 144 L127 144 L120 152 Z" fill="${C.nose}" stroke="${C.ink}" stroke-width="2.5" stroke-linejoin="round"/>`
const whiskers = `<path d="M44 140 L76 145 M46 156 L76 154 M196 140 L164 145 M194 156 L164 154" ${stroke(3)} opacity="0.55"/>`
const blush = `<ellipse cx="72" cy="158" rx="13" ry="8" fill="${C.blush}" opacity="0.55"/><ellipse cx="168" cy="158" rx="13" ry="8" fill="${C.blush}" opacity="0.55"/>`

/** Pure string output so the same art renders inline and onto the share-card canvas. */
export function catSvg(spec: CatSpec): string {
  const parts = [
    body,
    spec.blush ? blush : '',
    drawEyes(spec.eyes, [86, 130], [154, 130]),
    nose,
    mouths[spec.mouth],
    whiskers,
    ...(spec.props ?? []).map((p) => propArt[p]),
  ]
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 240 240" width="100%" height="100%">${parts.join('')}</svg>`
}

export function catDataUrl(spec: CatSpec): string {
  return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(catSvg(spec))}`
}
