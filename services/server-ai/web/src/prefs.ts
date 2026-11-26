import { reactive, watch } from 'vue';
import { ACCENT_IDS, getAccentPreset, type AccentId } from './accents';

export type ThemeMode = 'light' | 'dark';
export type UnitMode = 'both' | 'usd';

const THEME_KEY = 'ai_admin_theme';
const ACCENT_KEY = 'ai_admin_accent';
const UNIT_KEY = 'ai_admin_unit';

function readStorage<T extends string>(key: string, valid: readonly T[], fallback: T): T {
  try {
    const raw = localStorage.getItem(key) as T | null;
    return raw && valid.includes(raw) ? raw : fallback;
  } catch {
    return fallback;
  }
}

export const prefs = reactive({
  theme: readStorage<ThemeMode>(THEME_KEY, ['light', 'dark'], 'light'),
  accent: readStorage<AccentId>(ACCENT_KEY, ACCENT_IDS, 'red'),
  unitMode: readStorage<UnitMode>(UNIT_KEY, ['both', 'usd'], 'both'),
});

export function applyPrefs() {
  const root = document.documentElement;
  root.dataset.theme = prefs.theme;
  root.dataset.accent = prefs.accent;
  const preset = getAccentPreset(prefs.accent);
  const pal = prefs.theme === 'dark' ? preset.dark : preset.light;
  root.style.setProperty('--brand', pal.btn);
  root.style.setProperty('--brand-hi', pal.hi);
  root.style.setProperty('--brand-soft', pal.soft);
  root.style.setProperty('--brand-ink', pal.ink);
  root.style.setProperty('--brand-on', '#ffffff');
  if (document.body) {
    document.body.dataset.unit = prefs.unitMode;
  }
}

watch(
  () => [prefs.theme, prefs.accent] as const,
  ([theme, accent]) => {
    applyPrefs();
    try {
      localStorage.setItem(THEME_KEY, theme);
      localStorage.setItem(ACCENT_KEY, accent);
    } catch {
      /* ignore */
    }
  },
  { immediate: true },
);

watch(
  () => prefs.unitMode,
  (val) => {
    applyPrefs();
    try {
      localStorage.setItem(UNIT_KEY, val);
    } catch {
      /* ignore */
    }
  },
  { immediate: true },
);

export function toggleTheme() {
  prefs.theme = prefs.theme === 'light' ? 'dark' : 'light';
}

export function setAccent(id: AccentId) {
  prefs.accent = id;
}

export function toggleUnitMode() {
  prefs.unitMode = prefs.unitMode === 'both' ? 'usd' : 'both';
}
