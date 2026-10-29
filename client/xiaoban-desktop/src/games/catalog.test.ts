import { afterEach, describe, expect, it, vi } from 'vitest';
import { DEFAULT_COVER, coverUrl, fallbackCoverUrl, entryUrl, loadBuiltinCatalog, readCatalog, readGames, validBase } from './catalog.ts';

const game = (over: Record<string, unknown> = {}) => ({ id: 'my-game', name: '我的游戏', description: '简介', entry: 'index.html', runtimeProtocol: 1, ...over });

describe('game catalog', () => {
  it('does not need exactly seven games and keeps the order it was given', () => {
    expect(readGames([game({ id: 'a1' })]).map(g => g.id)).toEqual(['a1']);
    const many = ['merge-2048', 'ink-blade', 'x-1', 'x-2', 'x-3', 'x-4', 'x-5', 'x-6', 'x-7'].map(id => game({ id }));
    expect(readGames(many).map(g => g.id)).toEqual(many.map(g => g.id));
    expect(readGames([])).toEqual([]);
  });

  it('builds entry and cover addresses from the base, shipped games falling back to /game-packs', () => {
    const shipped = readGames([game({ id: 'ink-blade', cover: 'cover.svg' })])[0];
    expect(entryUrl(shipped)).toBe('/game-packs/ink-blade/index.html');
    expect(coverUrl(shipped)).toBe('/game-packs/ink-blade/cover.svg');
    const unix = readGames([game({ source: 'user', base: 'game-pack://localhost/my-game/', cover: 'art/cover one.png' })])[0];
    expect(entryUrl(unix)).toBe('game-pack://localhost/my-game/index.html');
    expect(coverUrl(unix)).toBe('game-pack://localhost/my-game/art/cover%20one.png');
    const windows = readGames([game({ source: 'user', base: 'http://game-pack.localhost/my-game/' })])[0];
    expect(entryUrl(windows)).toBe('http://game-pack.localhost/my-game/index.html');
    expect(windows.source).toBe('user');
  });

  it('uses personalized artwork for user games and keeps the shipped fallback', () => {
    const [plain] = readGames([game({ source: 'user', base: 'game-pack://localhost/my-game/' })]);
    expect(coverUrl(plain)).toBe(fallbackCoverUrl(plain));
    expect(coverUrl(plain)).not.toBe(DEFAULT_COVER);
    expect(decodeURIComponent(coverUrl(plain))).toContain('我的游戏');
    expect(coverUrl(readGames([game()])[0])).toBe(DEFAULT_COVER);
    expect(DEFAULT_COVER.startsWith('data:image/svg+xml,')).toBe(true);
    expect(decodeURIComponent(DEFAULT_COVER)).toContain('<svg');
  });

  it('drops entries with a base that is not this game’s own folder', () => {
    expect(validBase('my-game', 'game-pack://localhost/my-game/')).toBe(true);
    expect(validBase('my-game', '/game-packs/my-game/')).toBe(true);
    for (const base of ['https://evil.example/my-game/', 'game-pack://localhost/other/', 'game-pack://localhost/my-game/../x/', 'game-pack://evil/my-game/', '/game-packs/my-game', 'javascript:alert(1)//']) {
      expect(validBase('my-game', base), base).toBe(false);
      expect(readGames([game({ base })]), base).toEqual([]);
    }
    expect(readGames([game(), null, 'x', game({ id: 'Bad Id' }), game({ name: '' }), game({ entry: 'main.html' }), game()]).length).toBe(1);
  });

  it('reads the desktop answer with its folder errors', () => {
    const catalog = readCatalog({ games: [game({ source: 'user', base: 'game-pack://localhost/my-game/' })], errors: ['「x」：坏了', 3], directory: '/data/games/packs' });
    expect(catalog.games).toHaveLength(1);
    expect(catalog.errors).toEqual(['「x」：坏了']);
    expect(catalog.directory).toBe('/data/games/packs');
    expect(readCatalog(null)).toEqual({ games: [], errors: [], directory: null });
  });
});

describe('browser preview catalog', () => {
  afterEach(() => vi.unstubAllGlobals());
  it('loads the shipped games and tolerates one failing manifest', async () => {
    vi.stubGlobal('fetch', vi.fn(async (url: string) => {
      if (url.endsWith('catalog.json')) return { ok: true, json: async () => ({ games: ['merge-2048', 'water-sort'] }) };
      if (url.includes('water-sort')) return { ok: false };
      return { ok: true, json: async () => game({ id: 'merge-2048', cover: 'cover.svg' }) };
    }));
    const catalog = await loadBuiltinCatalog();
    expect(catalog.games.map(g => g.id)).toEqual(['merge-2048']);
    expect(catalog.games[0].source).toBe('builtin');
    expect(catalog.errors).toEqual(['部分游戏资料暂时无法显示。']);
  });
});
