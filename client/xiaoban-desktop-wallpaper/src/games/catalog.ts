import type { GameManifest } from './protocol.ts';
import { generatedCover } from './cover.ts';

export interface GameCatalog { games: GameManifest[]; errors: string[]; directory: string | null }

/** Shown when a game ships no cover image. */
export const DEFAULT_COVER = `data:image/svg+xml,${encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 320 180"><defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#cfe3df"/><stop offset="1" stop-color="#8fb0ad"/></linearGradient></defs><rect width="320" height="180" fill="url(#g)"/><g fill="none" stroke="#fff" stroke-width="7" stroke-linecap="round" stroke-linejoin="round" opacity=".9"><rect x="112" y="64" width="96" height="56" rx="26"/><path d="M138 92h20M148 82v20"/></g><circle cx="182" cy="88" r="4.5" fill="#fff"/><circle cx="194" cy="98" r="4.5" fill="#fff"/></svg>')}`;

export const builtinBase = (id: string) => `/game-packs/${id}/`;
// Shipped games live under the app's own origin; user games are served by the native game-pack protocol.
const BASE = /^(?:\/game-packs\/|game-pack:\/\/localhost\/|http:\/\/game-pack\.localhost\/)([a-z0-9][a-z0-9-]*)\/$/;

/** True when `base` is a folder URL this app produces for exactly this game. */
export function validBase(id: string, base: string): boolean {
  return BASE.exec(base)?.[1] === id;
}
export const gameBase = (game: Pick<GameManifest, 'id' | 'base'>) => game.base ?? builtinBase(game.id);
const encodePath = (path: string) => path.split('/').map(encodeURIComponent).join('/');
export const entryUrl = (game: GameManifest) => `${gameBase(game)}${encodePath(game.entry)}`;
export const fallbackCoverUrl = (game: GameManifest) => game.source === 'user' ? generatedCover(game) : DEFAULT_COVER;
export const coverUrl = (game: GameManifest) => (game.cover ? `${gameBase(game)}${encodePath(game.cover)}` : fallbackCoverUrl(game));

/** Keeps the entries that look like games; one bad entry never hides the rest. */
export function readGames(raw: unknown): GameManifest[] {
  if (!Array.isArray(raw)) return [];
  const seen = new Set<string>();
  const games: GameManifest[] = [];
  for (const item of raw) {
    if (!item || typeof item !== 'object') continue;
    const g = item as Record<string, unknown>;
    if (typeof g.id !== 'string' || !/^[a-z0-9][a-z0-9-]*$/.test(g.id) || seen.has(g.id)) continue;
    if (typeof g.name !== 'string' || !g.name || typeof g.description !== 'string' || g.entry !== 'index.html') continue;
    if (g.base !== undefined && (typeof g.base !== 'string' || !validBase(g.id, g.base))) continue;
    if (g.cover !== undefined && (typeof g.cover !== 'string' || !g.cover)) continue;
    seen.add(g.id);
    games.push({ ...(g as unknown as GameManifest), source: g.source === 'user' ? 'user' : 'builtin' });
  }
  return games;
}

/** Desktop answers from `list_games`; the browser preview reads the shipped games from the web server. */
export function readCatalog(raw: unknown): GameCatalog {
  const value = (raw && typeof raw === 'object' ? raw : {}) as Record<string, unknown>;
  const errors = Array.isArray(value.errors) ? value.errors.filter((e): e is string => typeof e === 'string') : [];
  return { games: readGames(value.games), errors, directory: typeof value.directory === 'string' ? value.directory : null };
}

export async function loadBuiltinCatalog(): Promise<GameCatalog> {
  const response = await fetch('/game-packs/catalog.json');
  if (!response.ok) throw new Error('游戏目录暂时无法读取。');
  const catalog = (await response.json()) as { games?: unknown };
  if (!Array.isArray(catalog.games)) throw new Error('游戏目录格式暂时无法读取。');
  const ids = catalog.games.filter((id): id is string => typeof id === 'string' && /^[a-z0-9-]+$/.test(id));
  const results = await Promise.all(ids.map(async (id) => {
    try {
      const reply = await fetch(`${builtinBase(id)}manifest.json`);
      return reply.ok ? ((await reply.json()) as Record<string, unknown>) : null;
    } catch { return null; }
  }));
  const games = readGames(results.filter((item) => item !== null));
  return { games, errors: games.length < catalog.games.length ? ['部分游戏资料暂时无法显示。'] : [], directory: null };
}
