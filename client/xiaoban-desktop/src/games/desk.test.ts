import { describe, expect, it, vi } from 'vitest';
import { DeskHost, type DeskInit } from './desk.ts';

const source = {} as Window;
const manifest = { id: 'desk-pal', name: '桌面伙伴', description: 'x', entry: 'index.html', runtimeProtocol: 1, layer: 'desktop' as const };
const make = (capabilities: string[], save = false) => {
  const init: DeskInit = { manifest: { ...manifest, save, capabilities }, screen: { width: 1440, height: 900 }, capabilities };
  const posted: Record<string, unknown>[] = [];
  const invoke = vi.fn(async () => undefined);
  const host = new DeskHost(init, 'secret', source, { invoke, post: (m) => posted.push(m) });
  return { host, posted, invoke };
};
const packet = (type: string, payload = {}) => ({ source, data: { channel: 'xiaban.game-pack', protocol: 1, nonce: 'secret', type, ...payload } });

describe('desktop-layer host', () => {
  it('greets the game with its layer, screen and capabilities', () => {
    const { host, posted } = make(['windows']);
    host.start({ level: 2 });
    expect(posted[0]).toMatchObject({ channel: 'xiaban.game-host', protocol: 1, type: 'host.init', nonce: 'secret', save: { level: 2 }, layer: 'desktop', capabilities: ['windows'], screen: { width: 1440, height: 900 } });
  });

  it('sends nothing until the game is ready, and only what the manifest asked for', async () => {
    const { host, posted } = make(['windows']);
    const reading = { windows: [{ x: 1, y: 2, w: 300, h: 200 }], pointer: { x: 5, y: 6, down: false } };
    host.frame(reading);
    expect(posted).toEqual([]);
    await host.handle(packet('game.ready'));
    host.frame(reading);
    expect(posted).toHaveLength(1);
    expect(posted[0]).toMatchObject({ type: 'host.frame', nonce: 'secret', windows: reading.windows });
    expect(posted[0]).not.toHaveProperty('pointer');
    posted.length = 0;
    host.frame({ pointer: reading.pointer });
    expect(posted).toEqual([]);
    host.frame('nope');
    host.frame(null);
    expect(posted).toEqual([]);
  });

  it('forwards the pointer only to a game that asked for it', async () => {
    const { host, posted } = make(['pointer']);
    await host.handle(packet('game.ready'));
    host.frame({ windows: [{ x: 0, y: 0, w: 100, h: 100 }], pointer: { x: 5, y: 6, down: true } });
    expect(posted[0]).toMatchObject({ pointer: { x: 5, y: 6, down: true } });
    expect(posted[0]).not.toHaveProperty('windows');
  });

  it('passes the mouse regions on once the game is ready, and ignores strangers', async () => {
    const { host, invoke } = make([]);
    await host.handle(packet('game.regions', { rects: [{ x: 1, y: 2, w: 3, h: 4 }] }));
    expect(invoke).not.toHaveBeenCalled();
    await host.handle(packet('game.ready'));
    await host.handle({ ...packet('game.regions', { rects: [] }), source: {} as Window });
    await host.handle(packet('game.regions', { rects: [], nonce: 'old' }));
    await host.handle(packet('game.regions', { rects: 'all' }));
    expect(invoke).not.toHaveBeenCalled();
    await host.handle(packet('game.regions', { rects: [{ x: 1, y: 2, w: 3, h: 4 }] }));
    expect(invoke).toHaveBeenCalledExactlyOnceWith('desk_set_regions', { rects: [{ x: 1, y: 2, w: 3, h: 4 }] });
  });

  it('saves only for games that declared a save, and finishes saving before it closes', async () => {
    const plain = make([]);
    await plain.host.handle(packet('game.ready'));
    await plain.host.handle(packet('game.save', { data: { a: 1 } }));
    expect(plain.invoke).not.toHaveBeenCalled();

    const { host, invoke } = make([], true);
    await host.handle(packet('game.ready'));
    await host.handle(packet('game.save', { data: { a: 1 } }));
    expect(invoke).toHaveBeenCalledWith('save_game_save', { gameId: 'desk-pal', data: { a: 1 } });
    await host.handle(packet('game.exit'));
    expect(invoke).toHaveBeenLastCalledWith('close_desktop_game');
  });
});
