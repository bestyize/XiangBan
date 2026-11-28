import type { AccountUser } from './accountApi.ts';
import type { EvaluatedResult, TestId } from './types.ts';

export interface QuizDraft {
  testId: TestId;
  step: number;
  answers: Record<string, string>;
  updatedAt: number;
}

export interface UserPrefs {
  autoAdvance: boolean;
  defaultPosterRatio: '916' | '34';
}

export interface StoredAuthSession {
  accessToken: string;
  refreshToken: string;
  expiresAt: number;
  user: AccountUser;
}

export interface StorageAdapter {
  getResults(uid?: string): Partial<Record<TestId, EvaluatedResult>>;
  saveResult(result: EvaluatedResult, uid?: string): void;
  mergeGuestResultsToAccount(uid: string): number;
  getDraft(testId: TestId): QuizDraft | null;
  listDrafts(): QuizDraft[];
  saveDraft(draft: QuizDraft): void;
  clearDraft(testId: TestId): void;
  getNickname(): string;
  saveNickname(name: string): void;
  getTheme(): 'light' | 'dark';
  saveTheme(theme: 'light' | 'dark'): void;
  getPrefs(): UserPrefs;
  savePrefs(prefs: Partial<UserPrefs>): UserPrefs;
  getCompatCount(): number;
  incrementCompatCount(): number;
  getAuthSession(): StoredAuthSession | null;
  saveAuthSession(session: StoredAuthSession): void;
  clearAuthSession(): void;
}

const KEY_RESULTS = 'vendme_results_v1';
const KEY_RESULTS_UID_PREFIX = 'vendme_results_uid_v1_';
const KEY_DRAFT_PREFIX = 'vendme_draft_v1_';
const KEY_NICKNAME = 'vendme_nickname_v1';
const KEY_THEME = 'vendme_theme_v1';
const KEY_PREFS = 'vendme_prefs_v1';
const KEY_COMPAT_COUNT = 'vendme_compat_count_v1';
const KEY_AUTH_SESSION = 'vendme_auth_session_v1';

const ALL_TEST_IDS: TestId[] = [
  'sbti',
  'pantone',
  'animal',
  'manual',
  'checkup',
  'classic16',
  'ingredients',
  'xianxia',
  'rpg',
  'bigfive',
  'attachment',
  'darktriad'
];

const DEFAULT_PREFS: UserPrefs = {
  autoAdvance: true,
  defaultPosterRatio: '916'
};

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

function resultsKey(uid?: string): string {
  return uid ? `${KEY_RESULTS_UID_PREFIX}${uid}` : KEY_RESULTS;
}

function readResultsByKey(key: string): Partial<Record<TestId, EvaluatedResult>> {
  const raw = readRaw(key);
  if (!raw) return {};
  try {
    return JSON.parse(raw) as Partial<Record<TestId, EvaluatedResult>>;
  } catch {
    return {};
  }
}

export const localStorageAdapter: StorageAdapter = {
  getResults(uid) {
    return readResultsByKey(resultsKey(uid));
  },
  saveResult(result, uid) {
    // 游客态写入全局本地档案；登录态同步写入账号档案与本机档案，保证退出后本机缓存保留
    const guestAll = readResultsByKey(KEY_RESULTS);
    guestAll[result.testId] = result;
    writeRaw(KEY_RESULTS, JSON.stringify(guestAll));
    if (uid) {
      const accKey = resultsKey(uid);
      const accAll = readResultsByKey(accKey);
      accAll[result.testId] = result;
      writeRaw(accKey, JSON.stringify(accAll));
    }
  },
  mergeGuestResultsToAccount(uid) {
    const guestAll = readResultsByKey(KEY_RESULTS);
    const accKey = resultsKey(uid);
    const accAll = readResultsByKey(accKey);
    let mergedCount = 0;

    for (const id of ALL_TEST_IDS) {
      const g = guestAll[id];
      const a = accAll[id];
      if (g && (!a || g.completedAt > a.completedAt)) {
        accAll[id] = g;
        mergedCount += 1;
      }
    }
    writeRaw(accKey, JSON.stringify(accAll));
    return mergedCount;
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
  listDrafts() {
    const drafts: QuizDraft[] = [];
    for (const id of ALL_TEST_IDS) {
      const d = this.getDraft(id);
      if (d && Object.keys(d.answers || {}).length > 0) {
        drafts.push(d);
      }
    }
    return drafts.sort((a, b) => b.updatedAt - a.updatedAt);
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
  },
  getPrefs() {
    const raw = readRaw(KEY_PREFS);
    if (!raw) return { ...DEFAULT_PREFS };
    try {
      const parsed = JSON.parse(raw) as Partial<UserPrefs>;
      return {
        autoAdvance:
          typeof parsed.autoAdvance === 'boolean'
            ? parsed.autoAdvance
            : DEFAULT_PREFS.autoAdvance,
        defaultPosterRatio:
          parsed.defaultPosterRatio === '34' ? '34' : DEFAULT_PREFS.defaultPosterRatio
      };
    } catch {
      return { ...DEFAULT_PREFS };
    }
  },
  savePrefs(partial) {
    const next: UserPrefs = {
      ...this.getPrefs(),
      ...partial
    };
    writeRaw(KEY_PREFS, JSON.stringify(next));
    return next;
  },
  getCompatCount() {
    const n = Number.parseInt(readRaw(KEY_COMPAT_COUNT) || '0', 10);
    return Number.isFinite(n) && n > 0 ? n : 0;
  },
  incrementCompatCount() {
    const next = this.getCompatCount() + 1;
    writeRaw(KEY_COMPAT_COUNT, String(next));
    return next;
  },
  getAuthSession() {
    const raw = readRaw(KEY_AUTH_SESSION);
    if (!raw) return null;
    try {
      const parsed = JSON.parse(raw) as StoredAuthSession;
      if (parsed && parsed.accessToken && parsed.refreshToken && parsed.user?.uid) {
        return parsed;
      }
      return null;
    } catch {
      return null;
    }
  },
  saveAuthSession(session) {
    writeRaw(KEY_AUTH_SESSION, JSON.stringify(session));
  },
  clearAuthSession() {
    removeRaw(KEY_AUTH_SESSION);
  }
};
