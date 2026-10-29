// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { invoke, listen } from '../../bridge.ts';
import { openModelWindow, openSettingsWindow, serveConfigurationWindows } from './windows.ts';
vi.mock('../../bridge.ts', () => ({ invoke: vi.fn(), listen: vi.fn() }));
const ipc = vi.mocked(invoke), subscribe = vi.mocked(listen);
beforeEach(() => { vi.resetAllMocks(); ipc.mockResolvedValue(null); subscribe.mockResolvedValue(() => {}); });
describe('native configuration handoff', () => {
  it('passes only the selected category or provider identity to window creation', async () => {
    await openSettingsWindow('network'); await openModelWindow('custom_test');
    expect(ipc.mock.calls).toEqual([
      ['configuration_window_open', { target: 'settings', tab: 'network', provider: null }],
      ['configuration_window_open', { target: 'model', tab: null, provider: 'custom_test' }],
    ]);
  });
  it('takes the authenticated command from native state instead of trusting an event payload', async () => {
    ipc.mockImplementation(async command => {
      if (command === 'configuration_window_take') return { id: '1', command: 'get_ai_config', args: {} } as never;
      if (command === 'get_ai_config') return { model: 'saved' } as never;
    });
    await serveConfigurationWindows();
    await subscribe.mock.calls[0][1]({ id: '1', command: 'chat_send', args: { text: 'forged' } });
    expect(ipc).not.toHaveBeenCalledWith('chat_send', expect.anything());
    expect(ipc).toHaveBeenCalledWith('configuration_window_reply', { id: '1', ok: true, value: { model: 'saved' } });
  });
  it('ignores expired or replayed request ids', async () => {
    ipc.mockRejectedValue(new Error('expired'));
    await serveConfigurationWindows();
    await subscribe.mock.calls[0][1]({ id: 'stale' });
    expect(ipc.mock.calls).toEqual([['configuration_window_take', { id: 'stale' }]]);
  });
  it('returns binary audio as bytes and preserves structured errors', async () => {
    ipc.mockImplementation(async command => {
      if (command === 'configuration_window_take') return { id: '2', command: 'tts_synthesize', args: {} } as never;
      if (command === 'tts_synthesize') return new Uint8Array([1, 2, 3]).buffer as never;
    });
    await serveConfigurationWindows();
    await subscribe.mock.calls[0][1]({ id: '2' });
    expect(ipc).toHaveBeenCalledWith('configuration_window_reply', { id: '2', ok: true, value: [1, 2, 3] });
    ipc.mockImplementation(async command => {
      if (command === 'configuration_window_take') return { id: '3', command: 'save_ai_provider', args: {} } as never;
      if (command === 'save_ai_provider') throw { code: 'bad_key', message: '配置未保存' };
    });
    await subscribe.mock.calls[0][1]({ id: '3' });
    expect(ipc).toHaveBeenCalledWith('configuration_window_reply', { id: '3', ok: false, value: { code: 'bad_key', message: '配置未保存' } });
  });
});
