import { COLOR, SYMBOL_ANCHOR, type Palette } from './geometry'
import type { SymbolName } from './params'

const f = (n: number) => n.toFixed(1)
const line = (color: string, width: number) =>
  `fill="none" stroke="${color}" stroke-width="${width}" stroke-linecap="round" stroke-linejoin="round"`

function star(x: number, y: number, r: number) {
  const k = r * 0.22
  return `M${f(x)} ${f(y - r)}Q${f(x + k)} ${f(y - k)} ${f(x + r)} ${f(y)}Q${f(x + k)} ${f(y + k)} ${f(x)} ${f(y + r)}Q${f(x - k)} ${f(y + k)} ${f(x - r)} ${f(y)}Q${f(x - k)} ${f(y - k)} ${f(x)} ${f(y - r)}Z`
}

export function heartPath(x: number, y: number, s: number, sy = 1) {
  const Y = (v: number) => f(y + v * sy)
  return `M${f(x)} ${Y(s * 0.85)}C${f(x - s * 1.55)} ${Y(-s * 0.15)} ${f(x - s * 0.7)} ${Y(-s * 1.25)} ${f(x)} ${Y(-s * 0.42)}C${f(x + s * 0.7)} ${Y(-s * 1.25)} ${f(x + s * 1.55)} ${Y(-s * 0.15)} ${f(x)} ${Y(s * 0.85)}Z`
}

const wave = (t: number, hz: number, phase = 0) => Math.sin((t * hz + phase) * Math.PI * 2)
const pulse = (t: number, hz: number, phase = 0) => 0.5 + 0.5 * wave(t, hz, phase)

type Draw = (t: number, x: number, y: number, c: Palette) => string

const draws: Record<SymbolName, Draw> = {
  sparkle: (t, x, y, c) => {
    const s = 1 + 0.18 * wave(t, 1.3)
    return `<path d="${star(x, y, 9 * s)}" fill="${c.yellow}"/><path d="${star(x - 13, y + 9, 3.6 * (2 - s))}" fill="${c.yellow}" opacity="0.8"/>`
  },
  sparks: (t, x, y, c) => {
    const o = 2 * pulse(t, 2.4)
    return `<path d="M${f(x - 6)} ${f(y - 2 - o)}L${f(x - 3)} ${f(y - 11 - o)}M${f(x + 2)} ${f(y + 1 - o)}L${f(x + 9)} ${f(y - 6 - o)}" ${line(c.yellow, 3.2)}/>` +
      `<path d="${star(x + 9, y - 13, 3.4 + o * 0.6)}" fill="${c.yellow}"/>`
  },
  question: (t, x, y, c) => {
    const a = 9 * wave(t, 0.7)
    return `<g transform="rotate(${f(a)} ${f(x)} ${f(y + 10)})"><path d="M${f(x - 6)} ${f(y - 6)}Q${f(x - 6)} ${f(y - 13)} ${f(x)} ${f(y - 13)}Q${f(x + 6.5)} ${f(y - 13)} ${f(x + 6.5)} ${f(y - 7)}Q${f(x + 6.5)} ${f(y - 2)} ${f(x + 1)} ${f(y)}Q${f(x)} ${f(y + 1)} ${f(x)} ${f(y + 4)}" ${line(c.yellow, 3.6)}/>` +
      `<circle cx="${f(x)}" cy="${f(y + 10)}" r="2.3" fill="${c.yellow}"/></g>`
  },
  exclaim: (t, x, y, c) => {
    const j = 1.2 * wave(t, 6) * pulse(t, 0.8)
    return `<path d="M${f(x + 1 + j)} ${f(y - 14)}L${f(x - 0.5 + j)} ${f(y + 3)}" ${line(c.yellow, 4.4)}/><circle cx="${f(x - 1 + j)}" cy="${f(y + 10)}" r="2.6" fill="${c.yellow}"/>`
  },
  dots: (t, x, y, c) =>
    [0, 1, 2].map((i) => {
      const k = pulse(t, 1.4, -i * 0.18)
      return `<circle cx="${f(x - 10 + i * 9)}" cy="${f(y - 2 * k)}" r="2.6" fill="${c.line}" opacity="${f(0.35 + 0.65 * k)}"/>`
    }).join(''),
  bubble: (t, x, y, c) => {
    const r = 6 + 3.2 * pulse(t, 0.34, -0.25)
    return `<circle cx="${f(x - 6)}" cy="${f(y + 10 - r * 0.4)}" r="${f(r)}" fill="${c.gray}" opacity="0.32" stroke="${c.gray}" stroke-width="1.6"/>` +
      `<circle cx="${f(x - 6 + r * 0.38)}" cy="${f(y + 10 - r * 0.8)}" r="${f(r * 0.24)}" fill="#ffffff" opacity="0.85"/>`
  },
  zzz: (t, x, y, c) =>
    [0, 1, 2].map((i) => {
      const k = (t * 0.45 + i / 3) % 1
      const s = 3 + i * 1.6
      const cx = x - 14 + i * 9 + k * 4
      const cy = y + 10 - i * 8 - k * 6
      const o = Math.min(1, k * 4, (1 - k) * 3)
      return `<path d="M${f(cx - s)} ${f(cy - s)}H${f(cx + s)}L${f(cx - s)} ${f(cy + s)}H${f(cx + s)}" ${line(c.line, 2.4)} opacity="${f(o)}"/>`
    }).join(''),
  anger: (t, x, y, c) => {
    const s = 1 + 0.16 * Math.max(0, wave(t, 1.6))
    const arm = (dx: number, dy: number) =>
      `M${f(x + dx * 2.5 * s)} ${f(y + dy * 8 * s)}Q${f(x + dx * 2.5 * s)} ${f(y + dy * 2.5 * s)} ${f(x + dx * 8 * s)} ${f(y + dy * 2.5 * s)}`
    return `<path d="${arm(1, 1)}${arm(-1, 1)}${arm(1, -1)}${arm(-1, -1)}" ${line(c.red, 3.2)}/>`
  },
  sound: (t, x, y, c) =>
    [0, 1].map((i) => {
      const k = pulse(t, 2.2, -i * 0.22)
      const r = 9 + i * 7
      const cx = x - 12
      const cy = y + 33
      return `<path d="M${f(cx + r * 0.6)} ${f(cy - r * 0.8)}Q${f(cx + r * 1.1)} ${f(cy)} ${f(cx + r * 0.6)} ${f(cy + r * 0.8)}" ${line(c.yellow, 3.2)} opacity="${f(0.35 + 0.65 * k)}"/>`
    }).join(''),
  wave: (t, x, y, c) =>
    [0, 1].map((i) => {
      const k = pulse(t, 1.6, -i * 0.25)
      const r = 6 + i * 6
      return `<path d="M${f(x - 4 + r * 0.2)} ${f(y - r * 0.9)}Q${f(x - 4 + r)} ${f(y - r * 0.5)} ${f(x - 4 + r)} ${f(y + r * 0.4)}" ${line(c.yellow, 3)} opacity="${f(0.35 + 0.65 * k)}"/>`
    }).join(''),
  listen: (t, x, y, c) =>
    [-1, 0, 1].map((i) => {
      const k = pulse(t, 1.8, i * 0.15)
      const len = 5 + 4 * k
      const cx = x + 2
      const cy = y + 22 + i * 7
      const dy = i * 2.4
      return `<path d="M${f(cx)} ${f(cy)}L${f(cx + len)} ${f(cy + dy)}" ${line(c.yellow, 2.8)}/>`
    }).join(''),
  ban: (t, x, y, c) => {
    const o = 0.55 + 0.45 * pulse(t, 0.8)
    return `<g opacity="${f(o)}"><circle cx="${f(x)}" cy="${f(y)}" r="8" ${line(c.orange, 2.8)}/><path d="M${f(x - 5.6)} ${f(y + 5.6)}L${f(x + 5.6)} ${f(y - 5.6)}" ${line(c.orange, 2.8)}/></g>`
  },
  bolt: (t, x, y, c) => {
    const o = 0.6 + 0.4 * pulse(t, 1.2)
    return `<path d="M${f(x + 3)} ${f(y - 12)}L${f(x - 6)} ${f(y + 1)}H${f(x)}L${f(x - 3)} ${f(y + 12)}L${f(x + 6)} ${f(y - 2)}H${f(x)}Z" fill="${c.green}" opacity="${f(o)}"/>`
  },
  sweat: (t, x, y, c) => {
    const k = (t * 0.5) % 1
    const dy = k * 8
    const o = Math.min(1, (1 - k) * 3)
    return `<path d="M${f(x)} ${f(y - 9 + dy)}Q${f(x + 7)} ${f(y + 1 + dy)} ${f(x)} ${f(y + 5 + dy)}Q${f(x - 7)} ${f(y + 1 + dy)} ${f(x)} ${f(y - 9 + dy)}Z" fill="${c.tear}" opacity="${f(o)}"/>`
  },
  heart: (t, x, y, c) => {
    const k = (t * 0.5) % 1
    const o = Math.min(1, k * 5, (1 - k) * 2.5)
    return `<path d="${heartPath(x - 2 + 3 * wave(t, 1), y + 6 - k * 14, 6 + k * 2)}" fill="${c.pink}" opacity="${f(o)}"/>`
  },
  note: (t, x, y, c) => {
    const b = 2 * wave(t, 1.4)
    return `<g transform="translate(0 ${f(b)})"><path d="M${f(x - 4)} ${f(y + 8)}V${f(y - 10)}L${f(x + 8)} ${f(y - 13)}V${f(y + 5)}" ${line(c.yellow, 2.8)}/>` +
      `<ellipse cx="${f(x - 7)}" cy="${f(y + 8)}" rx="3.6" ry="2.8" fill="${c.yellow}"/><ellipse cx="${f(x + 5)}" cy="${f(y + 5)}" rx="3.6" ry="2.8" fill="${c.yellow}"/></g>`
  },
  moon: (t, x, y, c) =>
    `<path d="M${f(x - 1)} ${f(y - 11)}A11 11 0 1 0 ${f(x + 10)} ${f(y + 3)}A8.5 8.5 0 1 1 ${f(x - 1)} ${f(y - 11)}Z" fill="${c.yellow}" opacity="${f(0.75 + 0.25 * pulse(t, 0.5))}"/>`,
  clock: (t, x, y, c) => {
    const a = t * Math.PI * 2 * 0.25
    return `<circle cx="${f(x)}" cy="${f(y)}" r="10" ${line(c.yellow, 2.6)}/>` +
      `<path d="M${f(x)} ${f(y)}L${f(x + Math.sin(a) * 6.5)} ${f(y - Math.cos(a) * 6.5)}M${f(x)} ${f(y)}L${f(x + 4)} ${f(y + 2)}" ${line(c.yellow, 2.4)}/>`
  },
  calendar: (_t, x, y, c) =>
    `<rect x="${f(x - 10)}" y="${f(y - 9)}" width="20" height="19" rx="4" ${line(c.line, 2.4)}/><path d="M${f(x - 10)} ${f(y - 3)}H${f(x + 10)}M${f(x - 5)} ${f(y - 12)}V${f(y - 6)}M${f(x + 5)} ${f(y - 12)}V${f(y - 6)}" ${line(c.line, 2.4)}/>` +
    `<path d="M${f(x - 4.5)} ${f(y + 3)}L${f(x - 1)} ${f(y + 6.5)}L${f(x + 5)} ${f(y)}" ${line(c.red, 2.4)}/>`,
  flame: (t, x, y, c) => {
    const s = 1 + 0.1 * wave(t, 3.1) + 0.05 * wave(t, 7.3)
    const top = y - 13 * s
    return `<path d="M${f(x)} ${f(top)}C${f(x + 10)} ${f(y - 3)} ${f(x + 9)} ${f(y + 10)} ${f(x)} ${f(y + 10)}C${f(x - 9)} ${f(y + 10)} ${f(x - 9)} ${f(y)} ${f(x - 3)} ${f(y - 5)}C${f(x - 2)} ${f(y - 1)} ${f(x + 1)} ${f(y - 2)} ${f(x)} ${f(top)}Z" fill="${c.orange}"/>` +
      `<path d="M${f(x)} ${f(y - 1)}C${f(x + 4)} ${f(y + 3)} ${f(x + 4)} ${f(y + 8)} ${f(x)} ${f(y + 8)}C${f(x - 4)} ${f(y + 8)} ${f(x - 4)} ${f(y + 3)} ${f(x)} ${f(y - 1)}Z" fill="${c.yellow}"/>`
  },
  crown: (t, x, y, c) =>
    `<path d="M${f(x - 11)} ${f(y + 7)}L${f(x - 12)} ${f(y - 7)}L${f(x - 5)} ${f(y - 1)}L${f(x)} ${f(y - 11)}L${f(x + 5)} ${f(y - 1)}L${f(x + 12)} ${f(y - 7)}L${f(x + 11)} ${f(y + 7)}Z" fill="${c.yellow}"/>` +
    `<path d="${star(x + 12, y - 12, 3 * pulse(t, 0.9))}" fill="${c.yellow}"/>`,
  coffee: (t, x, y, c) =>
    `<path d="M${f(x - 9)} ${f(y - 2)}H${f(x + 6)}V${f(y + 6)}Q${f(x + 6)} ${f(y + 11)} ${f(x + 1)} ${f(y + 11)}H${f(x - 4)}Q${f(x - 9)} ${f(y + 11)} ${f(x - 9)} ${f(y + 6)}Z" ${line(c.line, 2.4)}/>` +
    `<path d="M${f(x + 6)} ${f(y + 1)}Q${f(x + 11)} ${f(y + 1)} ${f(x + 11)} ${f(y + 4)}Q${f(x + 11)} ${f(y + 7)} ${f(x + 6)} ${f(y + 7)}" ${line(c.line, 2.2)}/>` +
    [0, 1].map((i) => `<path d="M${f(x - 5 + i * 6)} ${f(y - 6)}q-2.5 -3.5 0 -7" ${line(c.line, 2)} opacity="${f(0.3 + 0.6 * pulse(t, 0.9, i * 0.5))}"/>`).join(''),
  rain: (t, x, y, c) =>
    [[-120, -8], [-74, 2], [-30, -12], [4, 0]].map(([dx, dy], i) => {
      const k = (t * 0.9 + i * 0.27) % 1
      const cx = x + dx
      const cy = y + dy + k * 14
      const o = Math.min(1, k * 4, (1 - k) * 3)
      return `<path d="M${f(cx)} ${f(cy - 5)}Q${f(cx + 3.5)} ${f(cy + 1)} ${f(cx)} ${f(cy + 3)}Q${f(cx - 3.5)} ${f(cy + 1)} ${f(cx)} ${f(cy - 5)}Z" fill="${c.tear}" opacity="${f(o)}"/>`
    }).join(''),
  envelope: (t, x, y, c) =>
    `<rect x="${f(x - 8)}" y="${f(y - 11)}" width="16" height="22" rx="3" fill="${c.red}"/><path d="M${f(x - 8)} ${f(y - 5)}Q${f(x)} ${f(y + 1)} ${f(x + 8)} ${f(y - 5)}" ${line('#ffd0d6', 1.8)}/>` +
    `<circle cx="${f(x)}" cy="${f(y)}" r="${f(3 + 0.6 * pulse(t, 1))}" fill="${c.yellow}"/>`,
  mooncake: (_t, x, y, _c) =>
    `<circle cx="${f(x)}" cy="${f(y)}" r="10.5" fill="#e3a54e"/><circle cx="${f(x)}" cy="${f(y)}" r="6" fill="none" stroke="#9c5d22" stroke-width="1.8"/>` +
    `<path d="M${f(x - 3)} ${f(y)}H${f(x + 3)}M${f(x)} ${f(y - 3)}V${f(y + 3)}" stroke="#9c5d22" stroke-width="1.8" stroke-linecap="round"/>`,
  suitcase: (_t, x, y, c) =>
    `<rect x="${f(x - 11)}" y="${f(y - 5)}" width="22" height="16" rx="3.5" ${line(c.yellow, 2.4)}/><path d="M${f(x - 4)} ${f(y - 5)}V${f(y - 9)}H${f(x + 4)}V${f(y - 5)}M${f(x - 11)} ${f(y + 2)}H${f(x + 11)}" ${line(c.yellow, 2.4)}/>`,
  flower: (t, x, y, c) => {
    const a = 20 * wave(t, 0.4)
    const petals = [0, 72, 144, 216, 288].map((deg) => {
      const r = ((deg + a) * Math.PI) / 180
      return `<circle cx="${f(x + Math.cos(r) * 6)}" cy="${f(y + Math.sin(r) * 6)}" r="4.6" fill="${c.pink}"/>`
    }).join('')
    return petals + `<circle cx="${f(x)}" cy="${f(y)}" r="3.6" fill="${c.yellow}"/>`
  },
  shield: (t, x, y, c) =>
    `<path d="M${f(x)} ${f(y - 12)}L${f(x + 10)} ${f(y - 8)}V${f(y)}Q${f(x + 10)} ${f(y + 9)} ${f(x)} ${f(y + 13)}Q${f(x - 10)} ${f(y + 9)} ${f(x - 10)} ${f(y)}V${f(y - 8)}Z" fill="${c.green}" opacity="${f(0.8 + 0.2 * pulse(t, 0.6))}"/>` +
    `<path d="M${f(x - 4.5)} ${f(y)}L${f(x - 1)} ${f(y + 4)}L${f(x + 5)} ${f(y - 4)}" ${line(c.screen, 2.6)}/>`,
  // Text is drawn by renderSymbol, which knows the badge content.
  badge: () => '',
}

function badge(text: string, x: number, y: number, c: Palette) {
  const size = text.length >= 3 ? 11 : 13
  const w = Math.max(20, text.length * size * 0.66 + 10)
  return `<rect x="${f(x - w / 2)}" y="${f(y - 10)}" width="${f(w)}" height="20" rx="10" fill="${c.yellow}"/>` +
    `<text x="${f(x)}" y="${f(y + size * 0.36)}" text-anchor="middle" font-family="-apple-system,'PingFang SC',sans-serif" font-weight="800" font-size="${size}" fill="${c.screen}">${text.replace(/[^0-9a-zA-Z+]/g, '')}</text>`
}

/** Elastic pop-in curve: 0 → 1 with a small overshoot. */
export function popScale(p: number) {
  if (p <= 0) return 0
  if (p >= 1) return 1
  const c = 1.9
  const q = p - 1
  return 1 + (c + 1) * q * q * q + c * q * q
}

export function renderSymbol(name: SymbolName, appear: number, t: number, text = '', c: Palette = COLOR) {
  if (appear <= 0.01) return ''
  const { x, y } = SYMBOL_ANCHOR
  const s = popScale(appear)
  const body = name === 'badge' ? badge(text, x, y, c) : draws[name](t, x, y, c)
  return `<g opacity="${f(Math.min(1, appear * 1.5))}" transform="translate(${f(x)} ${f(y)}) scale(${s.toFixed(3)}) translate(${f(-x)} ${f(-y)})">${body}</g>`
}

export const SYMBOL_NAMES = Object.keys(draws) as SymbolName[]
