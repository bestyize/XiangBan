// @vitest-environment happy-dom
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { invoke } from '../../bridge.ts';
const original = window.__TAURI__;
const coreInvoke = vi.fn();
beforeEach(() => { coreInvoke.mockReset().mockResolvedValue({ saved: true }); Object.defineProperty(window, '__TAURI__', { value: { core: { invoke: coreInvoke } }, configurable: true, writable: true }); });
afterEach(() => { window.__TAURI__ = original; history.replaceState(null, '', '/'); });
it('routes provider credentials through authenticated native IPC without putting them in the URL', async () => {
  history.replaceState(null, '', '/?scene=configuration&kind=model&provider=deepseek');
  const args = { providerId: 'deepseek', apiKey: 'private-key' };
  await invoke('save_ai_provider', args);
  expect(coreInvoke).toHaveBeenCalledWith('configuration_window_invoke', { command: 'save_ai_provider', args });
  expect(location.href).not.toContain('private-key');
});
it('uses direct IPC for native window controls and for the actual control window', async () => {
  history.replaceState(null, '', '/?scene=configuration&kind=settings');
  await invoke('configuration_window_close');
  expect(coreInvoke).toHaveBeenLastCalledWith('configuration_window_close', undefined);
  history.replaceState(null, '', '/');
  await invoke('save_ai_provider', { providerId: 'deepseek' });
  expect(coreInvoke).toHaveBeenLastCalledWith('save_ai_provider', { providerId: 'deepseek' });
});
