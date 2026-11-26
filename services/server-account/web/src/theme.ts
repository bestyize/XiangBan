import { computed, ref } from 'vue';

export type ThemeMode = 'light' | 'dark';

const STORAGE_KEY = 'account_admin_theme';

function readInitial(): ThemeMode {
  const saved = localStorage.getItem(STORAGE_KEY);
  if (saved === 'light' || saved === 'dark') return saved;
  return window.matchMedia?.('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

const themeRef = ref<ThemeMode>(readInitial());

function applyTheme(mode: ThemeMode) {
  document.documentElement.setAttribute('data-theme', mode);
}

applyTheme(themeRef.value);

export function useTheme() {
  const theme = computed(() => themeRef.value);
  const isDark = computed(() => themeRef.value === 'dark');

  function setTheme(mode: ThemeMode) {
    themeRef.value = mode;
    localStorage.setItem(STORAGE_KEY, mode);
    applyTheme(mode);
  }

  function toggleTheme() {
    setTheme(themeRef.value === 'dark' ? 'light' : 'dark');
  }

  return { theme, isDark, setTheme, toggleTheme };
}
