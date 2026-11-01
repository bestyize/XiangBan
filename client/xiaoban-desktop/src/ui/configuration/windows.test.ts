// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { emit, invoke, listen } from '../../bridge.ts';
import { openModelWindow, openSettingsWindow, serveConfigurationWindows } from './windows.ts';
vi.mock('../../bridge.ts', () => ({ emit: vi.fn(), invoke: vi.fn(), listen: vi.fn() }));
const ipc = vi.mocked(invoke), subscribe = vi.mocked(listen);
beforeEach(() => { vi.resetAllMocks(); ipc.mockResolvedValue(null); subscribe.mockResolvedValue(() => {}); vi.mocked(emit).mockResolvedValue(undefined); });
describe('native configuration handoff', () => {
  it('broadcasts removal so the main catalog refreshes before the detail window closes', async () => {
    ipc.mockImplementation(async command => command === 'configuration_window_take' ? { id: 'remove', command: 'ai_store_uninstall', args: { slug: 'repo-plugin' } } as never : null);
    await serveConfigurationWindows();
    await subscribe.mock.calls[0][1]({ id: 'remove' });
    expect(ipc).toHaveBeenCalledWith('ai_store_uninstall', { slug: 'repo-plugin' });
    expect(emit).toHaveBeenCalledWith('ai-store-changed');
    expect(ipc).toHaveBeenCalledWith('configuration_window_reply', { id: 'remove', ok: true, value: null });
  });
  it('does not refresh the catalog on every pending authorization poll', async () => {
    let status = 'pending';
    ipc.mockImplementation(async command => command === 'configuration_window_take' ? { id: 'auth', command: 'ai_store_connector_auth_status', args: { flowId: 'flow' } } as never : command === 'ai_store_connector_auth_status' ? { status } as never : null);
    await serveConfigurationWindows();
    await subscribe.mock.calls[0][1]({ id: 'auth' });
    expect(emit).not.toHaveBeenCalled();
    status = 'connected';
    await subscribe.mock.calls[0][1]({ id: 'auth' });
    expect(emit).toHaveBeenCalledWith('ai-store-changed');
  });
  it('handles verified plugin tasks in the console and broadcasts successful configuration changes', async () => {
    const openTask = vi.fn().mockResolvedValue(undefined);
    ipc.mockResolvedValue({ id: 'store', command: 'ai_store_open_task', args: { task: { kind: 'skill', id: 'writer' } } });
    await serveConfigurationWindows(openTask);
    await subscribe.mock.calls[0][1]({ id: 'store' });
    expect(openTask).toHaveBeenCalledWith({ task: { kind: 'skill', id: 'writer' } });
    expect(ipc).not.toHaveBeenCalledWith('ai_store_open_task', expect.anything());
    ipc.mockImplementation(async command => command === 'configuration_window_take' ? { id: 'changed', command: 'ai_connector_configure', args: { id: 'docs' } } as never : null);
    await subscribe.mock.calls[0][1]({ id: 'changed' });
    expect(emit).toHaveBeenCalledWith('ai-store-changed');
  });
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
