export const C = {
  ink: '#1e1a2e',
  fur: '#f7a65a',
  furDark: '#d9792f',
  ear: '#ffb3a7',
  nose: '#ff8e8a',
  blush: '#ff7e79',
  mouth: '#7a2e3a',
  white: '#ffffff',
  tear: '#8fd3ff',
  lamp: '#ffb938',
  red: '#e8453c',
  sky: '#5aa9ff',
  leaf: '#6cc58a',
} as const

export const stroke = (width: number, color: string = C.ink) =>
  `fill="none" stroke="${color}" stroke-width="${width}" stroke-linecap="round" stroke-linejoin="round"`

export function heartPath(x: number, y: number, s: number) {
  return `M${x} ${y + s * 0.9} C${x - s * 1.6} ${y - s * 0.2} ${x - s * 0.6} ${y - s * 1.3} ${x} ${y - s * 0.4} C${x + s * 0.6} ${y - s * 1.3} ${x + s * 1.6} ${y - s * 0.2} ${x} ${y + s * 0.9} Z`
}

export function starPath(x: number, y: number, r: number) {
  const k = r * 0.18
  return `M${x} ${y - r} Q${x + k} ${y - k} ${x + r} ${y} Q${x + k} ${y + k} ${x} ${y + r} Q${x - k} ${y + k} ${x - r} ${y} Q${x - k} ${y - k} ${x} ${y - r} Z`
}
