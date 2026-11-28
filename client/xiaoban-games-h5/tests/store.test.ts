import assert from 'node:assert/strict';
import test from 'node:test';
import { GAMES, filterGames, formatScore, getFeaturedGames, getGameById } from '../src/data/games.ts';
import {
  clearAllGameData,
  defaultHallState,
  exportBackupCode,
  importBackupCode,
  loadHallState,
  loadProto1Save,
  markGameHasSave,
  parseHashRoute,
  recordGameDuration,
  recordGameLaunch,
  recordGameScore,
  saveHallState,
  writeProto1Save,
  type StorageAdapter
} from '../src/lib/storage.ts';

function memoryStorage(): StorageAdapter {
  const map = new Map<string, string>();
  return {
    getItem: (k) => (map.has(k) ? map.get(k)! : null),
    setItem: (k, v) => { map.set(k, v); },
    removeItem: (k) => { map.delete(k); },
    keys: () => [...map.keys()]
  };
}

test('catalog exposes 8 official games and accurate portrait filtering', () => {
  assert.equal(GAMES.length, 8);
  assert.equal(getFeaturedGames().length, 3);
  const portrait = filterGames(GAMES, 'portrait').map((g) => g.id);
  assert.deepEqual(portrait, ['water-sort', 'block-puzzle', 'merge-2048']);
  assert.equal(getGameById('ink-blade')?.name, '水墨剑客');
  assert.equal(formatScore(getGameById('dusk-glider')!, 2840), '2,840 m');
});

test('hall state tracks launches, scores, duration and saves', () => {
  const storage = memoryStorage();
  let st = defaultHallState(false);
  st = recordGameLaunch(st, 'block-puzzle', 1000);
  st = recordGameScore(st, 'block-puzzle', 14820, 2000);
  st = recordGameScore(st, 'block-puzzle', 8000, 3000); // should keep highest
  st = recordGameDuration(st, 'block-puzzle', 95);
  st = markGameHasSave(st, 'block-puzzle', true);
  saveHallState(storage, st);

  const reloaded = loadHallState(storage, false);
  assert.equal(reloaded.lastGameId, 'block-puzzle');
  assert.equal(reloaded.records['block-puzzle'].bestScore, 14820);
  assert.equal(reloaded.records['block-puzzle'].playSeconds, 95);
  assert.equal(reloaded.records['block-puzzle'].hasSave, true);
});

test('backup export/import roundtrip preserves hall state, proto1 and proto2 saves', () => {
  const src = memoryStorage();
  const st = markGameHasSave(recordGameScore(defaultHallState(true), 'water-sort', 1200, 5000), 'water-sort', true);
  saveHallState(src, st);
  writeProto1Save(src, 'ink-blade', { chapter: 2, stage: 4 });
  src.setItem('xb_game_store:water-sort:pour-joy-v2', JSON.stringify({ level: 12 }));

  const code = exportBackupCode(src);
  assert.ok(code.startsWith('XB1:'));

  const dst = memoryStorage();
  const res = importBackupCode(dst, code);
  assert.equal(res.ok, true);
  assert.equal(res.count, 3);

  assert.equal(loadHallState(dst).records['water-sort'].bestScore, 1200);
  assert.deepEqual(loadProto1Save(dst, 'ink-blade'), { chapter: 2, stage: 4 });
  assert.equal(dst.getItem('xb_game_store:water-sort:pour-joy-v2'), JSON.stringify({ level: 12 }));

  clearAllGameData(dst);
  assert.equal(dst.keys().length, 0);
});

test('invalid backup code returns structured error', () => {
  const dst = memoryStorage();
  assert.equal(importBackupCode(dst, 'invalid').ok, false);
  assert.equal(importBackupCode(dst, 'XB1:!!!not-base64!!!').ok, false);
});

test('hash route parser handles play deep links', () => {
  assert.deepEqual(parseHashRoute('#/play/ink-blade'), { kind: 'play', gameId: 'ink-blade' });
  assert.deepEqual(parseHashRoute('#/play/water-sort?from=wx'), { kind: 'play', gameId: 'water-sort' });
  assert.deepEqual(parseHashRoute(''), { kind: 'home' });
});
