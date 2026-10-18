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
// Another window (the control console) changed the choice: follow it, so the floating assistant matches.
if (typeof window !== 'undefined') window.addEventListener('storage', event => { if (event.key === KEY || event.key === null) appearanceMode.value = read(); });
export const isDark = computed(() => appearanceMode.value === 'dark' || (appearanceMode.value === 'system' && systemDark.value));
export function setAppearance(mode: AppearanceMode) {
  appearanceMode.value = mode;
  try { localStorage.setItem(KEY, mode); } catch { /* The choice still applies for this session. */ }
}
