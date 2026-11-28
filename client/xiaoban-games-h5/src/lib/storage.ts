export interface GameRecord {
  bestScore: number;
  lastPlayedAt: number;
  playCount: number;
  playSeconds: number;
  hasSave: boolean;
}

export interface HallPrefs {
  theme: 'light' | 'dark';
  accentId: string;
  layout: 'grid' | 'list';
  muted: boolean;
  rotateGuide: boolean;
}

export interface HallStateData {
  version: number;
  lastGameId: string | null;
  favorites: string[];
  records: Record<string, GameRecord>;
  prefs: HallPrefs;
}

export interface StorageAdapter {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
  removeItem(key: string): void;
  keys(): string[];
}

export const HALL_STATE_KEY = 'xb_hall_state_v1';
export const PROTO1_SAVE_PREFIX = 'xb_proto1_save:';
export const PROTO2_STORE_PREFIX = 'xb_game_store:';

const KNOWN_NATIVE_KEYS = [
  'pour-joy-v2',
  'xiaban.ink-blade.save',
  'xiaban.block-puzzle.save',
  'xiaban.merge-2048.save',
  'xiaban.dusk-glider.save',
  'xiaban.snake-arena.save',
  'xiaban.pro-cube.save',
  'xiaban.jade-gomoku.save'
];

export function createBrowserStorage(): StorageAdapter {
  return {
    getItem: (k) => {
      try { return localStorage.getItem(k); } catch { return null; }
    },
    setItem: (k, v) => {
      try { localStorage.setItem(k, v); } catch { /* quota */ }
    },
    removeItem: (k) => {
      try { localStorage.removeItem(k); } catch { /* ignore */ }
    },
    keys: () => {
      const out: string[] = [];
      try {
        for (let i = 0; i < localStorage.length; i++) {
          const k = localStorage.key(i);
          if (k) out.push(k);
        }
      } catch { /* ignore */ }
      return out;
    }
  };
}

export function defaultHallState(prefersDark = false): HallStateData {
  return {
    version: 1,
    lastGameId: null,
    favorites: ['ink-blade', 'water-sort'],
    records: {},
    prefs: {
      theme: prefersDark ? 'dark' : 'light',
      accentId: 'red',
      layout: 'grid',
      muted: false,
      rotateGuide: true
    }
  };
}

export function loadHallState(storage: StorageAdapter, prefersDark = false): HallStateData {
  const base = defaultHallState(prefersDark);
  const raw = storage.getItem(HALL_STATE_KEY);
  if (!raw) return base;
  try {
    const parsed = JSON.parse(raw) as Partial<HallStateData>;
    return {
      version: 1,
      lastGameId: typeof parsed.lastGameId === 'string' ? parsed.lastGameId : null,
      favorites: Array.isArray(parsed.favorites)
        ? parsed.favorites.filter((x): x is string => typeof x === 'string')
        : base.favorites,
      records: parsed.records && typeof parsed.records === 'object' ? parsed.records : {},
      prefs: {
        theme: parsed.prefs?.theme === 'dark' ? 'dark' : 'light',
        accentId: typeof parsed.prefs?.accentId === 'string' ? parsed.prefs.accentId : base.prefs.accentId,
        layout: parsed.prefs?.layout === 'list' ? 'list' : 'grid',
        muted: Boolean(parsed.prefs?.muted),
        rotateGuide: parsed.prefs?.rotateGuide !== false
      }
    };
  } catch {
    return base;
  }
}

export function saveHallState(storage: StorageAdapter, state: HallStateData): void {
  storage.setItem(HALL_STATE_KEY, JSON.stringify(state));
}

export function recordGameLaunch(state: HallStateData, gameId: string, now = Date.now()): HallStateData {
  const prev = state.records[gameId] || {
    bestScore: 0,
    lastPlayedAt: 0,
    playCount: 0,
    playSeconds: 0,
    hasSave: false
  };
  return {
    ...state,
    lastGameId: gameId,
    records: {
      ...state.records,
      [gameId]: {
        ...prev,
        lastPlayedAt: now,
        playCount: prev.playCount + 1
      }
    }
  };
}

export function recordGameScore(
  state: HallStateData,
  gameId: string,
  score: number,
  now = Date.now()
): HallStateData {
  const safeScore = Number.isFinite(score) ? Math.max(0, Math.floor(score)) : 0;
  const prev = state.records[gameId] || {
    bestScore: 0,
    lastPlayedAt: now,
    playCount: 1,
    playSeconds: 0,
    hasSave: false
  };
  return {
    ...state,
    lastGameId: gameId,
    records: {
      ...state.records,
      [gameId]: {
        ...prev,
        bestScore: Math.max(prev.bestScore, safeScore),
        lastPlayedAt: now
      }
    }
  };
}

export function recordGameDuration(state: HallStateData, gameId: string, seconds: number): HallStateData {
  const add = Math.max(0, Math.floor(seconds));
  if (!add) return state;
  const prev = state.records[gameId] || {
    bestScore: 0,
    lastPlayedAt: Date.now(),
    playCount: 1,
    playSeconds: 0,
    hasSave: false
  };
  return {
    ...state,
    records: {
      ...state.records,
      [gameId]: {
        ...prev,
        playSeconds: prev.playSeconds + add
      }
    }
  };
}

export function markGameHasSave(state: HallStateData, gameId: string, hasSave = true): HallStateData {
  const prev = state.records[gameId] || {
    bestScore: 0,
    lastPlayedAt: Date.now(),
    playCount: 1,
    playSeconds: 0,
    hasSave: false
  };
  return {
    ...state,
    records: {
      ...state.records,
      [gameId]: {
        ...prev,
        hasSave
      }
    }
  };
}

export function loadProto1Save(storage: StorageAdapter, gameId: string): unknown {
  const raw = storage.getItem(PROTO1_SAVE_PREFIX + gameId);
  if (!raw) return null;
  try { return JSON.parse(raw); } catch { return null; }
}

export function writeProto1Save(storage: StorageAdapter, gameId: string, save: unknown): void {
  if (save === null || save === undefined) {
    storage.removeItem(PROTO1_SAVE_PREFIX + gameId);
    return;
  }
  const encoded = JSON.stringify(save);
  if (encoded.length <= 65536) {
    storage.setItem(PROTO1_SAVE_PREFIX + gameId, encoded);
  }
}

function isManagedGameKey(key: string): boolean {
  return (
    key === HALL_STATE_KEY ||
    key.startsWith(PROTO1_SAVE_PREFIX) ||
    key.startsWith(PROTO2_STORE_PREFIX) ||
    KNOWN_NATIVE_KEYS.includes(key)
  );
}

export function estimateStorageBytes(storage: StorageAdapter): number {
  let bytes = 0;
  for (const k of storage.keys()) {
    if (!isManagedGameKey(k)) continue;
    const v = storage.getItem(k);
    if (v) bytes += (k.length + v.length) * 2;
  }
  return bytes;
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

export function formatRelativeTime(ts: number, now = Date.now()): string {
  if (!ts) return '';
  const diffSec = Math.max(0, Math.floor((now - ts) / 1000));
  if (diffSec < 60) return '刚刚';
  if (diffSec < 3600) return `${Math.floor(diffSec / 60)} 分钟前`;
  if (diffSec < 86400) return `${Math.floor(diffSec / 3600)} 小时前`;
  const days = Math.floor(diffSec / 86400);
  if (days === 1) return '昨天';
  if (days < 30) return `${days} 天前`;
  const d = new Date(ts);
  return `${d.getMonth() + 1}月${d.getDate()}日`;
}

export function formatPlayTime(totalSeconds: number): string {
  const mins = Math.floor(totalSeconds / 60);
  if (mins < 1) return totalSeconds > 0 ? '<1m' : '0m';
  if (mins < 60) return `${mins}m`;
  const h = Math.floor(mins / 60);
  const m = mins % 60;
  return m ? `${h}h${m}m` : `${h}h`;
}

export function exportBackupCode(storage: StorageAdapter): string {
  const entries: Record<string, string> = {};
  for (const k of storage.keys()) {
    if (!isManagedGameKey(k)) continue;
    const v = storage.getItem(k);
    if (v !== null) entries[k] = v;
  }
  const payload = JSON.stringify({
    app: 'xiaoban-games-h5',
    v: 1,
    exportedAt: Date.now(),
    entries
  });
  const bytes = new TextEncoder().encode(payload);
  let bin = '';
  for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i]);
  return 'XB1:' + btoa(bin);
}

export function importBackupCode(storage: StorageAdapter, rawCode: string): { ok: boolean; count: number; error?: string } {
  const trimmed = rawCode.trim();
  if (!trimmed.startsWith('XB1:')) {
    return { ok: false, count: 0, error: '存档码格式不正确，需以 XB1: 开头' };
  }
  try {
    const bin = atob(trimmed.slice(4));
    const bytes = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
    const json = new TextDecoder().decode(bytes);
    const parsed = JSON.parse(json) as { app?: string; entries?: Record<string, unknown> };
    if (parsed.app !== 'xiaoban-games-h5' || !parsed.entries || typeof parsed.entries !== 'object') {
      return { ok: false, count: 0, error: '存档码内容无效或已损坏' };
    }
    let count = 0;
    for (const [k, v] of Object.entries(parsed.entries)) {
      if (isManagedGameKey(k) && typeof v === 'string') {
        storage.setItem(k, v);
        count++;
      }
    }
    return { ok: true, count };
  } catch {
    return { ok: false, count: 0, error: '无法解析存档码，请检查是否完整复制' };
  }
}

export function clearAllGameData(storage: StorageAdapter): void {
  for (const k of storage.keys()) {
    if (isManagedGameKey(k)) storage.removeItem(k);
  }
}

export function parseHashRoute(hash: string): { kind: 'home' } | { kind: 'play'; gameId: string } {
  const cleaned = hash.replace(/^#/, '');
  const m = cleaned.match(/^\/play\/([a-z0-9_-]+)/i);
  if (m) return { kind: 'play', gameId: m[1] };
  return { kind: 'home' };
}
