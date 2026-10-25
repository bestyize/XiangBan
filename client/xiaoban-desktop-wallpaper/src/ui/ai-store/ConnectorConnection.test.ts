// @vitest-environment happy-dom
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { defineComponent, ref } from 'vue';
import { invoke } from '../../bridge.ts';
import ConnectorConnection from './ConnectorConnection.vue';
import type { InstalledConnector } from './connector-plugins.ts';

vi.mock('../../bridge.ts', () => ({ invoke: vi.fn() }));
const item = (patch: Partial<InstalledConnector> = {}): InstalledConnector => ({ id: 'docs:main', plugin: 'docs', name: '文档', description: '', version: '1', source: 'market', iconUrl: null,
  transport: 'streamable-http', auth: { type: 'auto', configured: [] }, status: 'disconnected', enabled: false, tools: [], skills: [], examples: [], ...patch });
const flow = { status: 'authorization_required', flowId: 'f1', authorizationUrl: 'https://auth.example/login' };
let wrapper: VueWrapper;
let current: InstalledConnector;
beforeEach(() => {
  vi.useFakeTimers(); current = item();
  vi.mocked(invoke).mockReset().mockImplementation(async command => {
    if (command === 'ai_connector_installed') return [current];
    if (command === 'ai_connector_configure') return current = { ...current, status: 'connected', enabled: true };
    if (command === 'ai_store_connector_auth_status') { current = { ...current, status: 'connected', enabled: true }; return { status: 'connected' }; }
    return null;
  });
});
afterEach(() => { wrapper?.unmount(); vi.useRealTimers(); });
async function setup(patch: Partial<InstalledConnector> = {}, autoConnect = false) {
  current = item(patch);
  const initial = current;
  wrapper = mount(defineComponent({ components: { ConnectorConnection }, setup: () => ({ current: ref(initial), autoConnect }), template: '<ConnectorConnection :connector="current" :auto-connect="autoConnect" @updated="current = $event" />' }));
  await flushPromises();
}
async function click(text: string) { await wrapper.findAll('button').find(b => b.text() === text)!.trigger('click'); await flushPromises(); }
describe('connection and authorization in place', () => {
  it('connects remote services only after an explicit action and has no empty config form', async () => {
    await setup(); expect(invoke).not.toHaveBeenCalledWith('ai_connector_configure', expect.anything());
    expect(wrapper.find('form').exists()).toBe(false); expect(wrapper.find('input').exists()).toBe(false);
    await click('连接');
    expect(invoke).toHaveBeenCalledWith('ai_connector_configure', { id: current.id, values: {}, confirmed: true });
    expect(wrapper.text()).toContain('已连接，可在任务中使用');
  });
  it('auto-connects remote services from install-and-connect, including updating an active connection', async () => {
    await setup({ status: 'connected' }, true);
    expect(invoke).toHaveBeenCalledWith('ai_connector_configure', expect.objectContaining({ confirmed: true }));
  });
  it('never starts a local command automatically', async () => {
    await setup({ transport: 'stdio', command: 'npx', args: ['docs-server'] }, true);
    expect(invoke).not.toHaveBeenCalledWith('ai_connector_configure', expect.anything());
    expect(wrapper.text()).toContain('npx docs-server'); await click('运行并连接');
    expect(invoke).toHaveBeenCalledWith('ai_connector_configure', expect.anything());
  });
  it('collects only required missing credentials and clears the form after submission', async () => {
    await setup({ auth: { type: 'token', configured: ['HOST'], schema: { title: '令牌', fields: [{ key: 'TOKEN', label: '密钥', type: 'password', required: true }, { key: 'HOST', label: '地址', type: 'text', required: true }] } } }, true);
    expect(invoke).not.toHaveBeenCalledWith('ai_connector_configure', expect.anything());
    expect(wrapper.find('input[type=checkbox]').exists()).toBe(false);
    await wrapper.get('[aria-label="密钥"]').setValue('secret'); await wrapper.get('form').trigger('submit'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_connector_configure', { id: current.id, values: { TOKEN: 'secret' }, confirmed: true });
    expect((wrapper.get('[aria-label="密钥"]').element as HTMLInputElement).value).toBe('');
  });
  it('opens discovered OAuth and finishes the same connection after browser authorization', async () => {
    vi.mocked(invoke).mockResolvedValueOnce(flow);
    await setup({}, true);
    expect(invoke).toHaveBeenCalledWith('open_download_url', { url: flow.authorizationUrl });
    expect(wrapper.text()).toContain('请在浏览器完成授权');
    await vi.advanceTimersByTimeAsync(1000); await flushPromises();
    expect(wrapper.text()).toContain('已连接，可在任务中使用');
    expect(wrapper.findComponent(ConnectorConnection).emitted('connected')).toHaveLength(1);
  });
  it('keeps browser-open failure actionable without abandoning authorization', async () => {
    vi.mocked(invoke).mockResolvedValueOnce(flow).mockRejectedValueOnce('浏览器不可用');
    await setup({}, true); expect(wrapper.text()).toContain('无法打开授权页面');
    await click('打开授权页面'); await vi.advanceTimersByTimeAsync(1000);
    expect(wrapper.text()).toContain('已连接，可在任务中使用');
  });
  it('stops polling after cancellation and does not report connected', async () => {
    vi.mocked(invoke).mockResolvedValueOnce(flow); await setup({}, true);
    await click('取消授权'); await vi.advanceTimersByTimeAsync(3000);
    expect(invoke).toHaveBeenCalledWith('ai_store_connector_auth_cancel', { flowId: 'f1' });
    expect(invoke).not.toHaveBeenCalledWith('ai_store_connector_auth_status', expect.anything());
    expect(wrapper.findComponent(ConnectorConnection).emitted('connected')).toBeUndefined();
  });
  it('cancels a late OAuth response after navigating away', async () => {
    let resolve!: (value: unknown) => void;
    vi.mocked(invoke).mockImplementationOnce(() => new Promise(r => { resolve = r; }));
    await setup({}, true); wrapper.unmount(); resolve(flow); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_store_connector_auth_cancel', { flowId: 'f1' });
    expect(invoke).not.toHaveBeenCalledWith('open_download_url', expect.anything());
  });
  it('does not confuse a failed reconnect with successful authorization', async () => {
    await setup({ status: 'connected' });
    vi.mocked(invoke).mockRejectedValueOnce('网络不可用'); await click('重新连接');
    expect(wrapper.text()).toContain('网络不可用');
    expect(wrapper.findComponent(ConnectorConnection).emitted('connected')).toBeUndefined();
  });
});
