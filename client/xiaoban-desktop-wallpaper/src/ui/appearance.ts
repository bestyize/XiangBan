import { computed, ref } from 'vue';
export type AppearanceMode = 'system' | 'light' | 'dark';
export const appearanceModes: { value: AppearanceMode; label: string }[] = [
  { value: 'system', label: '跟随系统' }, { value: 'light', label: '浅色' }, { value: 'dark', label: '深色' },
];
const KEY = 'xiaoban.control.appearance';
function read(): AppearanceMode {
  try { const value = localStorage.getItem(KEY); return appearanceModes.some(mode => mode.value === value) ? value as AppearanceMode : 'system'; } catch { return 'system'; }
}
const query = typeof window !== 'undefined' && typeof window.matchMedia === 'function' ? window.matchMedia('(prefers-color-scheme: dark)') : null;
export const appearanceMode = ref<AppearanceMode>(read());
const systemDark = ref(!!query?.matches);
query?.addEventListener?.('change', event => { systemDark.value = event.matches; });
const valid = (value: unknown): value is AppearanceMode => appearanceModes.some(mode => mode.value === value);
/** The choice as the host has it. Windows cannot share page storage, so this is how the others learn what the console chose. */
export function followHost(value: unknown) {
  if (valid(value) && value !== appearanceMode.value) appearanceMode.value = value;
}
export const isDark = computed(() => appearanceMode.value === 'dark' || (appearanceMode.value === 'system' && systemDark.value));
/**
 * What the console does at start with what the host has. A host that was never given a choice takes the one this window
 * already had (the user's pick from before the host kept one) unless it was the system setting; otherwise the host rules.
 */
export function reconcile(host: { appearance: string; appearanceKept?: boolean }, own: AppearanceMode): { send?: AppearanceMode; adopt?: AppearanceMode } {
  if (!host.appearanceKept && own !== 'system') return { send: own };
  if (valid(host.appearance) && host.appearance !== own) return { adopt: host.appearance };
  return {};
}
/** Where a choice is also sent, so the host can keep it and tell the other windows. Set by the console. */
let publish: ((mode: AppearanceMode) => void) | undefined;
export function publishAppearanceTo(callback: (mode: AppearanceMode) => void) { publish = callback; }
export function setAppearance(mode: AppearanceMode) {
  appearanceMode.value = mode;
  try { localStorage.setItem(KEY, mode); } catch { /* The choice still applies for this session. */ }
  publish?.(mode);
}

/* ── Theme colour (方案四) ─────────────────────────────────────────────── */

export type AccentId = 'red' | 'green' | 'blue' | 'purple' | 'amber' | 'graphite';
/**
 * The six roles of a theme colour.
 * btn: primary fill · btnHi: gradient highlight · soft: selected/light fill · ink: text on soft / links
 * tint: sidebar & window background · dot: status markers.
 */
export interface AccentPalette { btn: string; btnHi: string; soft: string; ink: string; tint: string; dot: string }
export interface AccentOption { id: AccentId; name: string; light: AccentPalette; dark: AccentPalette }

export const accents: readonly AccentOption[] = [
  { id: 'red', name: '小伴红',
    light: { btn: '#D93A44', btnHi: '#E8565F', soft: '#FCEBEC', ink: '#B42A33', tint: '#F5F1F1', dot: '#E5484D' },
    dark: { btn: '#E5484D', btnHi: '#F06A6F', soft: '#3A2225', ink: '#FF8C91', tint: '#1E1C1D', dot: '#FF6369' } },
  { id: 'green', name: '薄荷绿',
    light: { btn: '#2E8B5E', btnHi: '#3FA172', soft: '#E3F2E9', ink: '#1E6A45', tint: '#EEF4F0', dot: '#34A36D' },
    dark: { btn: '#34A36D', btnHi: '#48B981', soft: '#1D3328', ink: '#72D6A2', tint: '#1B1E1C', dot: '#3FB87A' } },
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
const ACCENT_KEY = 'xiaoban.control.accent';
const validAccent = (value: unknown): value is AccentId => accents.some(item => item.id === value);
function readAccent(): AccentId {
  try { const value = localStorage.getItem(ACCENT_KEY); return validAccent(value) ? value : DEFAULT_ACCENT; } catch { return DEFAULT_ACCENT; }
}
/** The chosen theme colour. Kept in this window's storage; every console page reads colours through CSS variables. */
export const accent = ref<AccentId>(readAccent());
/** Where a theme colour is also sent, so the host can tell windows that cannot read this storage. Set by the console. */
let publishAccent: ((id: AccentId) => void) | undefined;
export function publishAccentTo(callback: (id: AccentId) => void) { publishAccent = callback; }
export function setAccent(id: AccentId) {
  if (!validAccent(id)) return;
  accent.value = id;
  try { localStorage.setItem(ACCENT_KEY, id); } catch { /* still applies for this session */ }
  publishAccent?.(id);
}
/** The theme colour as the host reports it, for windows that do not share the console's storage. */
export function followAccent(value: unknown) {
  if (validAccent(value) && value !== accent.value) accent.value = value;
}
/** The palette of a theme colour in light or dark. Unknown ids fall back to the default. */
export function accentPalette(id: AccentId = accent.value, dark = false): AccentPalette {
  const option = accents.find(item => item.id === id) ?? accents[0];
  return dark ? option.dark : option.light;
}
// Another window (or a second console) changing the colour.
if (typeof window !== 'undefined') window.addEventListener?.('storage', event => { if (event.key === ACCENT_KEY && validAccent(event.newValue)) accent.value = event.newValue; });
