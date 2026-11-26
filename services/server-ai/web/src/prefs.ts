import { reactive, watch } from 'vue';

export type ThemeMode = 'light' | 'dark';
export type UnitMode = 'both' | 'usd';

const THEME_KEY = 'ai_admin_theme';
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
  unitMode: readStorage<UnitMode>(UNIT_KEY, ['both', 'usd'], 'both'),
});

export function applyPrefs() {
  document.documentElement.dataset.theme = prefs.theme;
  document.body.dataset.unit = prefs.unitMode;
}

watch(
  () => prefs.theme,
  (val) => {
    applyPrefs();
    try {
      localStorage.setItem(THEME_KEY, val);
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

export function toggleUnitMode() {
  prefs.unitMode = prefs.unitMode === 'both' ? 'usd' : 'both';
}
