import {
  BEZEL, BODY_PIVOT, CHEEK_OFFSET, COLOR, EYE_OFFSET, FACE_CENTER, HEAD, MOUTH_OFFSET, SCREEN, VIEW, type Palette,
} from './geometry.ts'
import type { Eye, EyeGlyph, Face, Mouth, SymbolName } from './params.ts'
import { heartPath, renderSymbol } from './symbols.ts'
import { faceStyleOf, skinOf, type Markup, type Skin, type SkinId } from './skins.ts'

/** Everything needed to draw one moment of the robot. */
export interface Frame {
  face: Face
  glyphL: EyeGlyph
  glyphR: EyeGlyph
  /** Vertical squash of each eye (blinks, glyph swaps), 1 = normal. */
  squashL: number
  squashR: number
  symbol: SymbolName | null
  /** Pop-in progress of the symbol, 0..1. */
  symbolP: number
  badge: string
  /** Seconds, drives looping symbol and tear animations. */
  t: number
  /** Dash rotation of the waiting spinner, in turns. */
  dash: number
  /** Breathing phase, -1..1. */
  breath: number
}

const f = (n: number) => (Number.isFinite(n) ? n.toFixed(1) : '0')
const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v))
const LINE_MIN = 6

function mixHex(a: string, b: string, k: number) {
  const pa = parseInt(a.slice(1), 16)
  const pb = parseInt(b.slice(1), 16)
  const ch = (shift: number) => Math.round(((pa >> shift) & 255) + ((((pb >> shift) & 255) - ((pa >> shift) & 255)) * k))
  return `#${((ch(16) << 16) | (ch(8) << 8) | ch(0)).toString(16).padStart(6, '0')}`
}

function ellipsePerimeter(a: number, b: number) {
  return Math.PI * (3 * (a + b) - Math.sqrt(Math.max(0, (3 * a + b) * (a + 3 * b))))
}

interface EyeCtx { id: string; side: -1 | 1; squash: number; glyph: EyeGlyph; color: string; pink: string; dash: number; gx: number; gy: number }

function eyeMarkup(e: Eye, c: EyeCtx) {
  const cx = FACE_CENTER.x + c.side * EYE_OFFSET.x + e.x + c.gx
  const cy = FACE_CENTER.y + EYE_OFFSET.y + e.y + c.gy
  const inner = c.side === -1 ? 1 : -1
  const W = Math.max(e.w, 1)
  const H0 = Math.max(e.h, 0)
  const T = clamp(e.thick, 0, 1) * Math.min(W, Math.max(H0, LINE_MIN))

  if (c.glyph === 'chevron') {
    const hh = Math.max(H0, 18) * 0.7 * c.squash
    const ww = W * 0.62
    return `<path d="M${f(cx - ww * inner)} ${f(cy - hh)}L${f(cx + ww * inner)} ${f(cy)}L${f(cx - ww * inner)} ${f(cy + hh)}" fill="none" stroke="${c.color}" stroke-width="${f(Math.max(T, LINE_MIN))}" stroke-linecap="round" stroke-linejoin="round"/>`
  }
  if (c.glyph === 'heart') {
    return `<path d="${heartPath(cx, cy + 2, W * 0.82, c.squash)}" fill="${c.pink}"/>`
  }

  const sw = Math.max(Math.min(T, 2 * Math.max(H0 * c.squash, 0)), LINE_MIN)
  const H = Math.max(H0 * c.squash, sw / 2)
  const rx = Math.max(W - sw / 2, 0.01)
  const ry = Math.max(H - sw / 2, 0)

  let d: string
  const span = clamp(e.span, 0, 1)
  if (span > 0.995) {
    d = `M${f(cx - rx)} ${f(cy)}A${f(rx)} ${f(ry)} 0 1 0 ${f(cx + rx)} ${f(cy)}A${f(rx)} ${f(ry)} 0 1 0 ${f(cx - rx)} ${f(cy)}`
  } else {
    const a0 = ((e.arcMid - span * 180) * Math.PI) / 180
    const a1 = ((e.arcMid + span * 180) * Math.PI) / 180
    d = `M${f(cx + rx * Math.cos(a0))} ${f(cy + ry * Math.sin(a0))}A${f(rx)} ${f(Math.max(ry, 0.01))} 0 ${span > 0.5 ? 1 : 0} 1 ${f(cx + rx * Math.cos(a1))} ${f(cy + ry * Math.sin(a1))}`
  }

  let dashAttr = ''
  if (e.dash > 0.01) {
    const P = ellipsePerimeter(rx, ry)
    const seg = P / 8
    const on = seg * (1 - 0.55 * clamp(e.dash, 0, 1))
    dashAttr = ` stroke-dasharray="${f(on)} ${f(seg - on)}" stroke-dashoffset="${f(-c.dash * P)}"`
  }

  const defs: string[] = []
  const wrap: string[] = []
  const lidOpen = e.lidTop - Math.abs(e.lidTilt) < -1.05 && e.lidBottom > 1.05
  if (!lidOpen) {
    const x0 = cx - W - 8
    const x1 = cx + W + 8
    const top = (x: number) => cy + (e.lidTop + e.lidTilt * ((x - cx) / W) * inner) * H
    const bottom = cy + e.lidBottom * H
    defs.push(`<clipPath id="${c.id}c"><path d="M${f(x0)} ${f(top(x0))}L${f(x1)} ${f(top(x1))}L${f(x1)} ${f(bottom)}L${f(x0)} ${f(bottom)}Z"/></clipPath>`)
    wrap.push(`clip-path="url(#${c.id}c)"`)
  }
  if (e.pupil > 0.02) {
    const r = e.pupil * Math.min(W, Math.max(H0, 1))
    defs.push(`<mask id="${c.id}m" maskUnits="userSpaceOnUse" x="${f(cx - W - 4)}" y="${f(cy - H0 - 4)}" width="${f(2 * W + 8)}" height="${f(2 * H0 + 8)}">` +
      `<rect x="${f(cx - W - 4)}" y="${f(cy - H0 - 4)}" width="${f(2 * W + 8)}" height="${f(2 * H0 + 8)}" fill="#fff"/>` +
      `<ellipse cx="${f(cx + e.pupilX * W)}" cy="${f(cy + e.pupilY * H)}" rx="${f(r)}" ry="${f(r * Math.min(1, c.squash * 1.2))}" fill="#000"/></mask>`)
    wrap.push(`mask="url(#${c.id}m)"`)
  }
  const stroke = `<path d="${d}" fill="none" stroke="${c.color}" stroke-width="${f(sw)}" stroke-linecap="round"${dashAttr}/>`
  return (defs.length ? `<defs>${defs.join('')}</defs>` : '') + (wrap.length ? `<g ${wrap.join(' ')}>${stroke}</g>` : stroke)
}

function mouthMarkup(m: Mouth, color: string, p: Palette, gx: number, gy: number) {
  const mx = FACE_CENTER.x + MOUTH_OFFSET.x + m.x + gx * 0.8
  const my = FACE_CENTER.y + MOUTH_OFFSET.y + m.y + gy * 0.8
  const w = Math.max(m.w, 1)
  const n = 18
  const upper: [number, number][] = []
  const lower: [number, number][] = []
  for (let i = 0; i <= n; i++) {
    const t = -1 + (2 * i) / n
    const base = 1 - t * t
    const y = my + m.curve * base + m.cat * Math.sin(Math.PI * Math.abs(t))
    upper.push([mx + t * w, y])
    lower.push([mx + t * w, y + Math.max(m.open, 0) * Math.pow(Math.max(base, 0), 0.7)])
  }
  const pts = (list: [number, number][]) => list.map(([x, y], i) => `${i ? 'L' : 'M'}${f(x)} ${f(y)}`).join('')
  if (m.open < 1.2) {
    return `<path d="${pts(upper)}" fill="none" stroke="${color}" stroke-width="3.8" stroke-linecap="round" stroke-linejoin="round"/>`
  }
  const shape = pts(upper) + lower.reverse().map(([x, y]) => `L${f(x)} ${f(y)}`).join('') + 'Z'
  const fill = m.tongue > 0.02 ? p.pink : p.mouth
  return `<path d="${shape}" fill="${fill}" fill-opacity="${f(Math.max(m.tongue, 0.9))}" stroke="${color}" stroke-width="3.2" stroke-linejoin="round"/>`
}

function cheeks(face: Face, p: Palette) {
  const out: string[] = []
  for (const side of [-1, 1] as const) {
    const x = FACE_CENTER.x + side * CHEEK_OFFSET.x + face.gazeX * 0.7
    const y = FACE_CENTER.y + CHEEK_OFFSET.y + face.gazeY * 0.7
    if (face.blush > 0.02) {
      out.push(`<ellipse cx="${f(x)}" cy="${f(y)}" rx="10" ry="5.5" fill="${p.blush}" opacity="${f(clamp(face.blush, 0, 1) * 0.8)}"/>`)
    }
    if (face.blushLines > 0.02) {
      const marks = [-5, 0, 5].map((dx) => `M${f(x + dx - 1.8)} ${f(y + 3.6)}L${f(x + dx + 1.8)} ${f(y - 3.6)}`).join('')
      out.push(`<path d="${marks}" fill="none" stroke="${p.hatch}" stroke-width="2.2" stroke-linecap="round" opacity="${f(clamp(face.blushLines, 0, 1))}"/>`)
    }
  }
  return out.join('')
}

function tears(face: Face, t: number, p: Palette) {
  if (face.tears < 0.02) return ''
  const k = clamp(face.tears, 0, 1)
  const bottom = SCREEN.y + SCREEN.h + 6
  return ([[-1, face.eyeL], [1, face.eyeR]] as const).map(([side, e]) => {
    const x = FACE_CENTER.x + side * EYE_OFFSET.x + e.x + face.gazeX + side * -e.w * 0.25
    const y0 = FACE_CENTER.y + EYE_OFFSET.y + e.y + face.gazeY + 2
    const yEnd = y0 + (bottom - y0) * k
    const d = `M${f(x)} ${f(y0)}C${f(x - side)} ${f(y0 + 18)} ${f(x + side * 2)} ${f(yEnd - 14)} ${f(x + side * 3)} ${f(yEnd)}`
    return `<path d="${d}" fill="none" stroke="${p.tear}" stroke-width="${f(7 + 4 * k)}" stroke-linecap="round" opacity="0.9"/>` +
      `<path d="${d}" fill="none" stroke="#d8f3ff" stroke-width="2.6" stroke-linecap="round" stroke-dasharray="5 13" stroke-dashoffset="${f(-t * 36)}" opacity="0.75"/>`
  }).join('')
}

/** The dynamic contents of the screen. ids must be unique per robot instance on the page. */
export function faceMarkup(frame: Frame, id: string, p: Palette = COLOR) {
  const { face } = frame
  const lit = clamp((face.screen - 0.3) / 0.7, 0, 1)
  const color = mixHex(p.lineDim, p.line, lit)
  const ctx = { color, pink: p.pink, dash: frame.dash, gx: face.gazeX, gy: face.gazeY }
  return [
    cheeks(face, p),
    tears(face, frame.t, p),
    eyeMarkup(face.eyeL, { ...ctx, id: `${id}l`, side: -1, squash: frame.squashL, glyph: frame.glyphL }),
    eyeMarkup(face.eyeR, { ...ctx, id: `${id}r`, side: 1, squash: frame.squashR, glyph: frame.glyphR }),
    mouthMarkup(face.mouth, color, p, face.gazeX, face.gazeY),
    frame.symbol ? renderSymbol(frame.symbol, frame.symbolP, frame.t, frame.badge, p) : '',
  ].join('')
}

export function faceOpacity(frame: Frame) {
  return clamp(0.3 + 0.7 * frame.face.screen, 0, 1)
}

export function bodyTransform(frame: Frame) {
  const s = 1 + 0.006 * frame.breath
  const { x, y } = BODY_PIVOT
  return `translate(0 ${f(frame.face.lift)}) rotate(${f(frame.face.tilt)} ${x} ${y}) translate(${x} ${y}) scale(${(2 - s).toFixed(4)} ${s.toFixed(4)}) translate(${-x} ${-y})`
}

export function earTransform(frame: Frame, side: -1 | 1, pivot: readonly [number, number] = [104, 98]) {
  const angle = side === -1 ? frame.face.earL : frame.face.earR
  const rotate = `rotate(${f(-angle)} ${pivot[0]} ${pivot[1]})`
  return side === -1 ? rotate : `translate(${VIEW.w} 0) scale(-1 1) ${rotate}`
}

export function earLight(frame: Frame) {
  const pulse = frame.face.glowPulse * (0.5 + 0.5 * Math.sin(frame.t * Math.PI * 2 * 1.1))
  return clamp(frame.face.earGlow * (1 + 0.08 * frame.breath) - pulse * 0.45, 0, 1)
}

export function shadowAttrs(frame: Frame) {
  const up = clamp(-frame.face.lift, 0, 30)
  return { rx: f(104 * (1 - up * 0.008)), opacity: (0.3 * (1 - up * 0.015)).toFixed(3) }
}

function ear(id: string, side: -1 | 1, frame: Frame, skin: Skin) {
  const light = earLight(frame)
  return `<g data-part="ear${side === -1 ? 'L' : 'R'}" transform="${earTransform(frame, side, skin.earPivot)}">` +
    `<ellipse cx="${skin.haloAt[0]}" cy="${skin.haloAt[1]}" rx="${skin.haloAt[2]}" ry="${skin.haloAt[3]}" fill="url(#${id}halo)" data-part="halo" opacity="${f(light)}"/>` +
    `<path d="${skin.earOuter}" fill="url(#${id}earShell)" stroke="${skin.earStroke}" stroke-width="1.2"/>` +
    `<path d="${skin.earInner}" fill="${skin.earBase}"/>` +
    `<path d="${skin.earInner}" fill="url(#${id}earLight)" data-part="earLight" opacity="${f(0.25 + 0.75 * light)}"/>` +
    `<path d="${skin.earHighlight}" fill="none" stroke="#fffbe8" stroke-width="3" stroke-linecap="round" opacity="0.55"/>` +
    `</g>`
}

const draw = (value: Markup | undefined, id: string) => (typeof value === 'function' ? value(id) : value ?? '')

function defs(id: string, skin: Skin) {
  const s = SCREEN
  const style = faceStyleOf(skin)
  return `<defs>` +
    `<radialGradient id="${id}shell" cx="0.36" cy="0.2" r="0.95"><stop offset="0" stop-color="${skin.shell[0]}"/><stop offset="0.45" stop-color="${skin.shell[1]}"/><stop offset="0.82" stop-color="${skin.shell[2]}"/><stop offset="1" stop-color="${skin.shell[3]}"/></radialGradient>` +
    `<linearGradient id="${id}shade" x1="0" y1="0" x2="0" y2="1"><stop offset="0.5" stop-color="${skin.shade}" stop-opacity="0"/><stop offset="1" stop-color="${skin.shade}" stop-opacity="0.24"/></linearGradient>` +
    `<linearGradient id="${id}bezel" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="${skin.bezel[0]}"/><stop offset="0.5" stop-color="${skin.bezel[1]}"/><stop offset="1" stop-color="${skin.bezel[2]}"/></linearGradient>` +
    `<radialGradient id="${id}screen" cx="0.5" cy="0.18" r="0.95"><stop offset="0" stop-color="#1d222c"/><stop offset="0.55" stop-color="#090a0e"/><stop offset="1" stop-color="#020203"/></radialGradient>` +
    `<linearGradient id="${id}gloss" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fff" stop-opacity="0.09"/><stop offset="0.38" stop-color="#fff" stop-opacity="0"/></linearGradient>` +
    `<linearGradient id="${id}pod" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="${skin.pod[0]}"/><stop offset="0.45" stop-color="${skin.pod[1]}"/><stop offset="1" stop-color="${skin.pod[2]}"/></linearGradient>` +
    `<linearGradient id="${id}podCore" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="${skin.podCore[0]}"/><stop offset="0.5" stop-color="${skin.podCore[1]}"/><stop offset="1" stop-color="${skin.podCore[2]}"/></linearGradient>` +
    `<linearGradient id="${id}earShell" x1="0.3" y1="0" x2="0.6" y2="1"><stop offset="0" stop-color="${skin.earShell[0]}"/><stop offset="1" stop-color="${skin.earShell[1]}"/></linearGradient>` +
    `<radialGradient id="${id}earLight" gradientUnits="userSpaceOnUse" cx="${skin.lightAt[0]}" cy="${skin.lightAt[1]}" r="${skin.lightAt[2]}"><stop offset="0" stop-color="${skin.light[0]}"/><stop offset="0.3" stop-color="${skin.light[1]}"/><stop offset="0.62" stop-color="${skin.light[2]}"/><stop offset="1" stop-color="${skin.light[3]}"/></radialGradient>` +
    `<radialGradient id="${id}halo"><stop offset="0" stop-color="${skin.halo}" stop-opacity="0.5"/><stop offset="1" stop-color="${skin.halo}" stop-opacity="0"/></radialGradient>` +
    `<radialGradient id="${id}ground"><stop offset="0" stop-color="#000" stop-opacity="1"/><stop offset="1" stop-color="#000" stop-opacity="0"/></radialGradient>` +
    `<clipPath id="${id}headClip">${skin.art ? `<path d="${skin.art.head}"/>` : `<rect x="${HEAD.x}" y="${HEAD.y}" width="${HEAD.w}" height="${HEAD.h}" rx="${HEAD.r}"/>`}</clipPath>` +
    `<clipPath id="${id}screenClip"><rect x="${s.x}" y="${s.y}" width="${s.w}" height="${s.h}" rx="${s.r}"/></clipPath>` +
    (style.glow === 'ink'
      ? `<filter id="${id}glow" filterUnits="userSpaceOnUse" x="${s.x}" y="${s.y}" width="${s.w}" height="${s.h}" color-interpolation-filters="sRGB">` +
        `<feGaussianBlur in="SourceGraphic" stdDeviation="0.9" result="soft"/><feMerge><feMergeNode in="soft"/><feMergeNode in="SourceGraphic"/></feMerge></filter>`
      : `<filter id="${id}glow" filterUnits="userSpaceOnUse" x="${s.x}" y="${s.y}" width="${s.w}" height="${s.h}" color-interpolation-filters="sRGB">` +
    `<feGaussianBlur in="SourceGraphic" stdDeviation="5.5" result="blur"/>` +
    `<feColorMatrix in="blur" type="matrix" values="0.55 0 0 0 0  0 0.85 0 0 0.04  0 0 1 0 0.12  0 0 0 1.35 0" result="wide"/>` +
    `<feGaussianBlur in="SourceGraphic" stdDeviation="1.5" result="near"/>` +
    `<feMerge><feMergeNode in="wide"/><feMergeNode in="near"/><feMergeNode in="SourceGraphic"/></feMerge></filter>`) +
    draw(skin.art?.defs, id) +
    `</defs>`
}

function shell(id: string, skin: Skin) {
  const h = HEAD
  const b = BEZEL
  const s = SCREEN
  const pod = (x: number) =>
    `<rect x="${x}" y="120" width="36" height="88" rx="18" fill="url(#${id}pod)" stroke="${skin.podStroke}" stroke-width="1.4"/>` +
    `<rect x="${x + 6}" y="129" width="24" height="70" rx="12" fill="url(#${id}podCore)" opacity="0.9"/>`
  const foot = (x: number) => `<rect x="${x}" y="226" width="40" height="24" rx="10" fill="#3a3029"/><rect x="${x + 6}" y="240" width="28" height="4" rx="2" fill="#5a4a3e"/>`
  const bezelAndScreen =
    `<rect x="${b.x}" y="${b.y}" width="${b.w}" height="${b.h}" rx="${b.r}" fill="url(#${id}bezel)"/>` +
    `<rect x="${s.x}" y="${s.y}" width="${s.w}" height="${s.h}" rx="${s.r}" fill="url(#${id}screen)" stroke="#000" stroke-opacity="0.7" stroke-width="1.6"/>`
  if (skin.art) {
    const art = skin.art
    return art.feet +
      `<path d="${art.head}" fill="${draw(art.fill, id) || `url(#${id}shell)`}" stroke="${skin.stroke}" stroke-width="1.5" stroke-linejoin="round"/>` +
      `<path d="${art.head}" fill="url(#${id}shade)"/>` +
      `<g clip-path="url(#${id}headClip)">${draw(art.trim, id)}</g>` + draw(art.front, id) + (art.panel === undefined ? bezelAndScreen : draw(art.panel, id))
  }
  return foot(98) + foot(182) + pod(19) + pod(265) +
    `<rect x="${h.x}" y="${h.y}" width="${h.w}" height="${h.h}" rx="${h.r}" fill="url(#${id}shell)" stroke="${skin.stroke}" stroke-width="1.5"/>` +
    `<g clip-path="url(#${id}headClip)"><rect x="${h.x}" y="${h.y}" width="${h.w}" height="${h.h}" fill="url(#${id}shade)"/>` +
    `<ellipse cx="118" cy="84" rx="70" ry="9" fill="#fff" opacity="0.55"/></g>` +
    `<rect x="${b.x}" y="${b.y}" width="${b.w}" height="${b.h}" rx="${b.r}" fill="url(#${id}bezel)"/>` +
    `<rect x="${s.x}" y="${s.y}" width="${s.w}" height="${s.h}" rx="${s.r}" fill="url(#${id}screen)" stroke="#000" stroke-opacity="0.7" stroke-width="1.6"/>`
}

function gloss(id: string, skin: Skin) {
  if (!faceStyleOf(skin).gloss) return ''
  const s = SCREEN
  return `<rect x="${s.x}" y="${s.y}" width="${s.w}" height="${s.h}" rx="${s.r}" fill="url(#${id}gloss)" pointer-events="none"/>` +
    `<path d="M76 128Q78 104 106 100L146 99Q108 106 88 124Z" fill="#fff" opacity="0.07" pointer-events="none"/>`
}

/**
 * Full SVG markup. Pure string so the same art renders inline, as an <img> data URL and onto the share-card canvas.
 * Parts that animate carry data-part attributes; the live component updates them in place.
 */
const scaleOpen = (skin: Skin) =>
  skin.scale === 1 ? '' : `<g transform="translate(${BODY_PIVOT.x} ${BODY_PIVOT.y}) scale(${skin.scale}) translate(${-BODY_PIVOT.x} ${-BODY_PIVOT.y})">`

export function robotMarkup(frame: Frame, id: string, label?: string, skinId?: SkinId | string, skinOverride?: Skin) {
  const skin = skinOverride ?? skinOf(skinId)
  const shadow = shadowAttrs(frame)
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${VIEW.w} ${VIEW.h}" width="100%" height="100%"${label ? ` role="img" aria-label="${label.replace(/[<>&"]/g, '')}"` : ''}>` +
    defs(id, skin) +
    scaleOpen(skin) +
    `<ellipse data-part="shadow" cx="160" cy="250" rx="${shadow.rx}" ry="11" fill="url(#${id}ground)" opacity="${shadow.opacity}"/>` +
    `<g data-part="body" transform="${bodyTransform(frame)}">` +
    draw(skin.art?.back, id) +
    ear(id, -1, frame, skin) + ear(id, 1, frame, skin) +
    shell(id, skin) +
    `<g clip-path="url(#${id}screenClip)"><g data-part="face" filter="url(#${id}glow)" opacity="${f(faceOpacity(frame))}">${faceMarkup(frame, id, faceStyleOf(skin).palette)}</g></g>` +
    gloss(id, skin) +
    `</g>` + (skin.scale === 1 ? '' : '</g>') + `</svg>`
}

/** As an image source. An explicit pixel size keeps canvas drawing sharp, since an <img> SVG has no intrinsic size otherwise. */
export function robotDataUrl(frame: Frame, width = 640, skinId?: SkinId | string) {
  const svg = robotMarkup(frame, 'rb', undefined, skinId).replace('width="100%" height="100%"', `width="${width}" height="${Math.round((width * VIEW.h) / VIEW.w)}"`)
  return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`
}
