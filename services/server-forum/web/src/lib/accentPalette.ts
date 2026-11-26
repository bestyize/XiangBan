// Unified 6 official accent palettes (aligned 1:1 with mobile DesignTokens.kt).
export type AccentId = 'red' | 'green' | 'blue' | 'purple' | 'amber' | 'graphite';
/**
 * The six roles of a theme colour.
 * btn: primary fill · btnHi: hover/gradient highlight · soft: subtle accent wash · ink: text/link
 * tint: subtle surface tint · dot: status markers.
 */
export interface AccentPalette { btn: string; btnHi: string; soft: string; ink: string; tint: string; dot: string }
export interface AccentOption { id: AccentId; name: string; light: AccentPalette; dark: AccentPalette }

export const accents: readonly AccentOption[] = [
  { id: 'red', name: '小伴红',
    light: { btn: '#E3203C', btnHi: '#F0506A', soft: '#FFF3F4', ink: '#E3203C', tint: '#F5F1F1', dot: '#E3203C' },
    dark: { btn: '#EE4B5F', btnHi: '#F76B7D', soft: '#3A2225', ink: '#FF8C91', tint: '#1E1C1D', dot: '#FF6369' } },
  { id: 'green', name: '薄荷绿',
    light: { btn: '#2E8B5E', btnHi: '#3FA172', soft: '#E3F2E9', ink: '#1E6A45', tint: '#EFF5F1', dot: '#2E8B5E' },
    dark: { btn: '#34A36D', btnHi: '#48B981', soft: '#1D3328', ink: '#72D6A2', tint: '#1A231E', dot: '#34C27C' } },
  { id: 'blue', name: '海湾蓝',
    light: { btn: '#2F6FDB', btnHi: '#4A84E6', soft: '#E6EEFC', ink: '#1D53AE', tint: '#EFF3F9', dot: '#3B7BE8' },
    dark: { btn: '#3B7BE8', btnHi: '#5A92F0', soft: '#1E2A42', ink: '#8DB6FF', tint: '#1B1D21', dot: '#4C8DF6' } },
  { id: 'purple', name: '暮紫',
    light: { btn: '#7152D8', btnHi: '#8669E3', soft: '#EDE8FC', ink: '#5537B8', tint: '#F3F1F9', dot: '#7C5CE6' },
    dark: { btn: '#8466E6', btnHi: '#9A80EE', soft: '#2A2342', ink: '#BBA8FF', tint: '#1D1C21', dot: '#9273F0' } },
  { id: 'amber', name: '琥珀',
    light: { btn: '#A8640C', btnHi: '#BD7A1C', soft: '#FAEEDB', ink: '#87500A', tint: '#F6F3ED', dot: '#E09A2B' },
    dark: { btn: '#C27A1A', btnHi: '#D69130', soft: '#3A2C17', ink: '#F0B65A', tint: '#1F1D1A', dot: '#F0A93A' } },
  { id: 'graphite', name: '石墨',
    light: { btn: '#2B2B30', btnHi: '#4A4A50', soft: '#E9E9EC', ink: '#2B2B30', tint: '#F2F2F3', dot: '#55555B' },
    dark: { btn: '#636368', btnHi: '#7C7C82', soft: '#3A3A3D', ink: '#E5E5EA', tint: '#1C1C1E', dot: '#9A9AA0' } },
];
export const DEFAULT_ACCENT: AccentId = 'red';

export function validAccent(value: unknown): value is AccentId { return accents.some(item => item.id === value) }
export function paletteFor(value: unknown, dark = false): AccentPalette {
  const option = accents.find(item => item.id === value) ?? accents[0]
  return dark ? option.dark : option.light
}
function luminance(hex: string) {
  const values = [1,3,5].map(i => parseInt(hex.slice(i,i+2),16)/255).map(v => v <= .04045 ? v/12.92 : ((v+.055)/1.055)**2.4)
  return values[0]*.2126 + values[1]*.7152 + values[2]*.0722
}
export function buttonInk(fill: string) { return 1.05/(luminance(fill)+.05) >= 4.5 ? '#ffffff' : '#000000' }
export function accentInk(palette: AccentPalette, dark: boolean): string {
  if (!dark) return palette.ink
  const channels = [1,3,5].map(i => parseInt(palette.ink.slice(i,i+2),16))
  for (let percent=0; percent<=100; percent+=5) {
    const color = '#' + channels.map(c => Math.round(c+(255-c)*percent/100).toString(16).padStart(2,'0')).join('')
    if ((luminance(color)+.05)/(luminance('#1c1c1e')+.05) >= 4.8) return color
  }
  return '#ffffff'
}
