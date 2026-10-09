import { describe, expect, it } from 'vitest';
import { GameSession, metrics, validSave, sameJson } from './protocol';
const source = {} as Window;
const packet = (type: string, payload = {}) => ({ source, data: { channel: 'xiaban.game-pack', protocol: 1, nonce: 'secret', type, ...payload } });
describe('desktop game boundary', () => {
  it('compares native JSON saves independently of object key order while retaining array order', () => {
    expect(sameJson({ desktopSmoke: true, chapters: { bamboo: 2, snow: 1 } }, { chapters: { snow: 1, bamboo: 2 }, desktopSmoke: true })).toBe(true);
    expect(sameJson({ levels: [1, 2] }, { levels: [2, 1] })).toBe(false);
    expect(sameJson({ chapters: { bamboo: 2 } }, { chapters: { bamboo: 3 } })).toBe(false);
    expect(sameJson({ a: null }, { a: false })).toBe(false);
  });
  it('requires the current frame, nonce, protocol and game binding', () => {
    const s = new GameSession('merge-2048', 'secret', source);
    expect(s.accept({ ...packet('game.ready'), source: {} as Window })).toBeNull();
    expect(s.accept(packet('game.ready', { nonce: 'old' }))).toBeNull();
    expect(s.accept(packet('game.ready', { protocol: 2 }))).toBeNull();
    expect(s.accept(packet('game.ready', { gameId: 'snake-arena' }))).toBeNull();
    expect(s.accept(packet('game.ready'))).not.toBeNull();
  });
  it('accepts final results only for a started current round and validates share metrics', () => {
    const s = new GameSession('merge-2048', 'secret', source);
    expect(s.accept(packet('game.result', { score: 8 }))).toBeNull();
    s.accept(packet('game.ready')); s.accept(packet('game.start'));
    s.accept(packet('game.progress', { score: 12, maxTile: 8 }));
    expect(s.accept(packet('game.result', { score: 8 }))).toBeNull();
    expect(s.accept(packet('game.result', { score: 12, maxTile: 8 }))).not.toBeNull();
    expect(s.accept(packet('game.share', { result: { gameId: 'merge-2048', score: 24, maxTile: 8 } }))).toBeNull();
    expect(s.accept(packet('game.share', { result: { gameId: 'merge-2048', score: 12, maxTile: 8 } }))).not.toBeNull();
    expect(s.accept(packet('game.result', { score: 12 }))).toBeNull();
    s.accept(packet('game.start'));
    expect(s.accept(packet('game.share', { result: { gameId: 'merge-2048', score: 12, maxTile: 8 } }))).toBeNull();
  });
  it('rejects noninteger metrics and enforces save byte limits', () => {
    expect(metrics({ score: -1 })).toBeNull(); expect(metrics({ score: NaN })).toBeNull(); expect(metrics({ score: 3.5 })).toBeNull(); expect(metrics({ score: 2, lines: Infinity })).toBeNull();
    expect(validSave([])).toBe(false); expect(validSave(null)).toBe(false); expect(validSave(new Date())).toBe(false); expect(validSave({ text: '猫'.repeat(12000) })).toBe(false); expect(validSave({ chapter: 2 })).toBe(true);
  });
});
