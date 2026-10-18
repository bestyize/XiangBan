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
