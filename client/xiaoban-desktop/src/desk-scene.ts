import './games.css';
import { invoke } from './bridge.ts';
import { entryUrl } from './games/catalog.ts';
import { DeskHost, type DeskInit } from './games/desk.ts';

declare global {
  interface Window {
    __XIAOBAN_GAME_TOKEN__?: string;
    __XIAOBAN_DESK__?: DeskInit;
  }
}

/**
 * The desktop layer: a transparent page above the other windows that holds one sandboxed game frame. The window
 * is click-through except where the game asked for the mouse, which the native side arranges.
 */
export async function mountDeskGame() {
  document.body.className = 'desk-scene';
  const init = window.__XIAOBAN_DESK__;
  const token = window.__XIAOBAN_GAME_TOKEN__;
  const call = (command: string, args: Record<string, unknown> = {}) => invoke(command, { ...args, hostToken: token });
  if (!init || !token) {
    await invoke('close_desktop_game').catch(() => undefined);
    return;
  }
  const { manifest } = init;
  let save: Record<string, unknown> | null = null;
  try {
    if (manifest.save) save = (await call('load_game_save', { gameId: manifest.id })) as Record<string, unknown> | null;
  } catch {
    // A save that cannot be read is left alone on disk; the game starts fresh and its first save is a deliberate one.
  }

  const nonce = crypto.randomUUID();
  const frame = document.createElement('iframe');
  frame.title = manifest.name;
  frame.sandbox.add('allow-scripts');
  frame.allow = 'autoplay';
  frame.referrerPolicy = 'no-referrer';
  frame.src = `${entryUrl(manifest)}?hostNonce=${encodeURIComponent(nonce)}&desktop=1&layer=desktop`;
  let host: DeskHost | null = null;
  frame.addEventListener('load', () => {
    if (!frame.contentWindow || host) return;
    host = new DeskHost(init, nonce, frame.contentWindow, {
      invoke: call,
      post: (message) => frame.contentWindow?.postMessage(message, '*'),
    });
    host.start(save);
  });
  window.addEventListener('message', (event) => { void host?.handle(event); });
  window.addEventListener('desk-frame', (event) => host?.frame((event as CustomEvent).detail));
  document.body.append(frame);

  // An invisible layer that never starts would just sit there, so it is closed instead.
  window.setTimeout(() => { if (!host?.session.ready) void invoke('close_desktop_game', { hostToken: token }).catch(() => undefined); }, 15000);
}
