import type { EvaluatedResult, TestId } from './types.ts';

export interface QuizDraft {
  testId: TestId;
  step: number;
  answers: Record<string, string>;
  updatedAt: number;
}

export interface StorageAdapter {
  getResults(): Partial<Record<TestId, EvaluatedResult>>;
  saveResult(result: EvaluatedResult): void;
  getDraft(testId: TestId): QuizDraft | null;
  saveDraft(draft: QuizDraft): void;
  clearDraft(testId: TestId): void;
  getNickname(): string;
  saveNickname(name: string): void;
  getTheme(): 'light' | 'dark';
  saveTheme(theme: 'light' | 'dark'): void;
}

const KEY_RESULTS = 'vendme_results_v1';
const KEY_DRAFT_PREFIX = 'vendme_draft_v1_';
const KEY_NICKNAME = 'vendme_nickname_v1';
const KEY_THEME = 'vendme_theme_v1';

function safeStorage(): Storage | null {
  try {
    if (typeof window !== 'undefined' && window.localStorage) {
      return window.localStorage;
    }
  } catch {
    // 隐私模式或无 DOM 环境自动降级
  }
  return null;
}

const memoryFallback = new Map<string, string>();

function readRaw(key: string): string | null {
  const ls = safeStorage();
  if (ls) {
    try {
      return ls.getItem(key);
    } catch {
      return memoryFallback.get(key) ?? null;
    }
  }
  return memoryFallback.get(key) ?? null;
}

function writeRaw(key: string, value: string): void {
  const ls = safeStorage();
  if (ls) {
    try {
      ls.setItem(key, value);
      return;
    } catch {
      // fallback
    }
  }
  memoryFallback.set(key, value);
}

function removeRaw(key: string): void {
  const ls = safeStorage();
  if (ls) {
    try {
      ls.removeItem(key);
      return;
    } catch {
      // fallback
    }
  }
  memoryFallback.delete(key);
}

export const localStorageAdapter: StorageAdapter = {
  getResults() {
    const raw = readRaw(KEY_RESULTS);
    if (!raw) return {};
    try {
      return JSON.parse(raw) as Partial<Record<TestId, EvaluatedResult>>;
    } catch {
      return {};
    }
  },
  saveResult(result) {
    const all = this.getResults();
    all[result.testId] = result;
    writeRaw(KEY_RESULTS, JSON.stringify(all));
  },
  getDraft(testId) {
    const raw = readRaw(KEY_DRAFT_PREFIX + testId);
    if (!raw) return null;
    try {
      return JSON.parse(raw) as QuizDraft;
    } catch {
      return null;
    }
  },
  saveDraft(draft) {
    writeRaw(KEY_DRAFT_PREFIX + draft.testId, JSON.stringify(draft));
  },
  clearDraft(testId) {
    removeRaw(KEY_DRAFT_PREFIX + testId);
  },
  getNickname() {
    return readRaw(KEY_NICKNAME) || '你';
  },
  saveNickname(name) {
    const clean = name.trim().slice(0, 12);
    writeRaw(KEY_NICKNAME, clean || '你');
  },
  getTheme() {
    return readRaw(KEY_THEME) === 'dark' ? 'dark' : 'light';
  },
  saveTheme(theme) {
    writeRaw(KEY_THEME, theme);
  }
};
