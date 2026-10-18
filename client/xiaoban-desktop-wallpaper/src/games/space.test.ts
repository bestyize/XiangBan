// @vitest-environment happy-dom
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { mountGameSpace } from '../games.ts';
import { invoke } from '../bridge.ts';

vi.mock('../bridge.ts', () => ({ desktop: true, invoke: vi.fn() }));
vi.mock('./catalog.ts', async original => ({ ...await original<typeof import('./catalog.ts')>(), entryUrl: () => 'about:blank' }));
const game = { id: 'test-game', name: '测试游戏', description: 'test', entry: 'index.html', runtimeProtocol: 1, save: true };
const listeners: [string, EventListenerOrEventListenerObject][] = [];
let spy: ReturnType<typeof vi.spyOn>;

beforeEach(() => {
  document.body.innerHTML = '<div id="app"></div>';
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async command => {
    if (command === 'get_game_catalog') return [game] as never;
    return null as never;
  });
  const original = window.addEventListener.bind(window);
  spy = vi.spyOn(window, 'addEventListener').mockImplementation((type, callback, options) => {
    if (callback) listeners.push([type, callback]);
    original(type, callback, options);
  });
});

afterEach(() => {
  for (const [type, callback] of listeners.splice(0)) window.removeEventListener(type, callback);
  spy.mockRestore();
  document.body.innerHTML = '';
});

async function launch() {
  await mountGameSpace();
  document.querySelector<HTMLButtonElement>('.game-card')!.click();
  await vi.waitFor(() => expect(document.querySelector('iframe')).not.toBeNull());
  const frame = document.querySelector('iframe')!;
  const source = frame.contentWindow!;
  frame.dispatchEvent(new Event('load'));
  const nonce = new URL(frame.src).searchParams.get('hostNonce');
  const send = (type: string, extra = {}) => window.dispatchEvent(new MessageEvent('message', {
    source, data: { channel: 'xiaban.game-pack', protocol: 1, nonce, type, ...extra },
  }));
  send('game.ready');
  return { frame, send };
}

it('leaves navigation to the game and does not intercept Escape during play', async () => {
  const { frame, send } = await launch();
  expect(document.querySelector('#games-back')).toBeNull();
  expect(document.querySelector('#games-close')).toBeNull();
  const escape = new KeyboardEvent('keydown', { key: 'Escape', cancelable: true });
  window.dispatchEvent(escape);
  expect(escape.defaultPrevented).toBe(false);
  expect(document.querySelector('iframe')).toBe(frame);
  send('game.exit');
  await vi.waitFor(() => expect(document.querySelector('iframe')).toBeNull());
  expect(document.querySelector<HTMLElement>('#games-hub')!.hidden).toBe(false);
  expect(invoke).not.toHaveBeenCalledWith('close_game_space', expect.anything());
});

it('flushes a pending save before the native window close request', async () => {
  const { send } = await launch();
  let saved!: () => void;
  vi.mocked(invoke).mockImplementation(async command => {
    if (command === 'save_game_save') await new Promise<void>(resolve => { saved = resolve; });
    return null as never;
  });
  send('game.save', { data: { level: 3 } });
  await vi.waitFor(() => expect(saved).toBeTypeOf('function'));
  window.dispatchEvent(new Event('game-space-close'));
  expect(invoke).not.toHaveBeenCalledWith('close_game_space', expect.anything());
  saved();
  await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith('close_game_space', expect.anything()));
  expect(document.querySelector('iframe')).toBeNull();
});

it('keeps the game open and offers recovery when saving fails', async () => {
  const { frame, send } = await launch();
  vi.mocked(invoke).mockImplementation(async command => {
    if (command === 'save_game_save') throw new Error('disk full');
    return null as never;
  });
  send('game.save', { data: { level: 4 } });
  window.dispatchEvent(new Event('game-space-close'));
  await vi.waitFor(() => expect(document.querySelector<HTMLElement>('#games-failure')!.hidden).toBe(false));
  expect(document.querySelector('iframe')).toBe(frame);
  expect(document.querySelector('#games-error')!.textContent).toContain('disk full');
  expect(invoke).not.toHaveBeenCalledWith('close_game_space', expect.anything());
  window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
});
