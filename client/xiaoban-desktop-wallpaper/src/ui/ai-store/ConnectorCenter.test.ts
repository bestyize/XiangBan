// @vitest-environment happy-dom
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { reactive } from 'vue';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import ConnectorCenter from './ConnectorCenter.vue';
import ConnectorTokenForm from './ConnectorTokenForm.vue';
import type { InstalledConnector } from './connector-plugins.ts';

vi.mock('../../bridge.ts', () => ({ invoke: vi.fn() }));
vi.mock('naive-ui', async () => {
  const { defineComponent } = await import('vue');
  return { NModal: defineComponent({ props: ['show'], template: '<section v-if="show" role="dialog"><slot /></section>' }) };
});
const schema = { title: '服务令牌', titleEn: 'Service token', description: '在服务商获取令牌', fields: [
  { key: 'TOKEN', label: '令牌', type: 'password' as const, required: true, placeholder: '填入令牌', description: '仅保存在本机' },
  { key: 'HOST', label: '主机', type: 'text' as const, required: true },
] };
const connector = (patch: Partial<InstalledConnector> = {}): InstalledConnector => ({ id: 'repo', name: '代码仓库', version: '1.2', description: '查看代码', source: 'local', iconUrl: null,
  auth: { type: 'token', configured: [], schema }, status: 'disconnected', enabled: false,
  tools: [{ name: 'search', description: '搜索代码' }], examples: ['找出最近的提交'], skills: ['review'], ...patch });
let wrapper: VueWrapper;
let current: InstalledConnector;
let store: ControlStore;
beforeEach(() => {
  current = connector();
  vi.mocked(invoke).mockReset().mockImplementation(async (command, args) => {
    if (command === 'ai_connector_installed') return [current];
    if (command === 'ai_connector_configure') return current = { ...current, status: 'connected', enabled: true, auth: { ...current.auth, configured: ['TOKEN', 'HOST'] } };
    if (command === 'ai_connector_set_enabled') return current = { ...current, enabled: args!.enabled as boolean, status: args!.enabled ? 'connected' : 'disabled' };
    if (command === 'ai_connector_unbind') { current = connector(); return null; }
    if (command === 'ai_connector_reauthorize') return { status: 'authorization_required', flowId: 'flow', authorizationUrl: 'https://auth.example/authorize' };
    if (command === 'ai_store_connector_auth_status') { current = { ...current, status: 'connected' }; return { status: 'connected' }; }
    return null;
  });
  store = reactive({ desktop: true, chat: { select: vi.fn().mockResolvedValue(undefined) }, creationDraft: null, navigate: vi.fn(), notify: vi.fn() }) as unknown as ControlStore;
});
afterEach(() => { wrapper?.unmount(); vi.useRealTimers(); });
async function open(patch: Partial<InstalledConnector> = {}) {
  current = connector(patch); wrapper = mount(ConnectorCenter, { props: { store } }); await flushPromises();
  await click('管理');
}
async function click(text: string) {
  const button = wrapper.findAll('button').find(b => b.text() === text);
  if (!button) throw new Error(`Missing ${text}`);
  await button.trigger('click'); await flushPromises();
}
describe('installed connector center', () => {
  it('shows Chinese status, version, auth, tools, examples and bundled skills', async () => {
    await open();
    expect(wrapper.text()).toContain('未连接'); expect(wrapper.text()).toContain('v1.2');
    expect(wrapper.text()).toContain('令牌认证'); expect(wrapper.text()).toContain('1 个工具');
    expect(wrapper.text()).toContain('搜索代码'); expect(wrapper.text()).toContain('review');
    await click('找出最近的提交');
    expect(store.chat.select).toHaveBeenCalledWith(null); expect(store.creationDraft).toBe('找出最近的提交'); expect(store.navigate).toHaveBeenCalledWith('new-task');
  });
  it('renders the schema and validates required fields before explicit confirmation', async () => {
    await open();
    const form = wrapper.getComponent(ConnectorTokenForm);
    expect(form.text()).toContain('服务令牌'); expect(form.text()).toContain('在服务商获取令牌');
    expect(form.get('[aria-label="令牌"]').attributes('type')).toBe('password');
    expect(form.get('[aria-label="令牌"]').attributes('placeholder')).toBe('填入令牌');
    expect(form.get('button').attributes('disabled')).toBeDefined();
    await form.get('[aria-label="令牌"]').setValue('secret');
    await form.get('input[type=checkbox]').setValue(true); await form.get('form').trigger('submit');
    expect(invoke).not.toHaveBeenCalledWith('ai_connector_configure', expect.anything());
    await form.get('[aria-label="主机"]').setValue('api.example');
    expect((form.get('input[type=checkbox]').element as HTMLInputElement).checked).toBe(false);
    await form.get('input[type=checkbox]').setValue(true); await form.get('form').trigger('submit'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_connector_configure', { id: 'repo', values: { TOKEN: 'secret', HOST: 'api.example' }, confirmed: true });
    expect((form.get('[aria-label="令牌"]').element as HTMLInputElement).value).toBe('');
    expect(form.text()).toContain('已保存'); expect(wrapper.html()).not.toContain('secret');
  });
  it('keeps saved credentials opaque and omits unchanged fields', async () => {
    await open({ auth: { type: 'token', schema, configured: ['TOKEN', 'HOST'] } });
    const form = wrapper.getComponent(ConnectorTokenForm);
    expect(form.findAll('input:not([type=checkbox])').every(i => (i.element as HTMLInputElement).value === '')).toBe(true);
    await form.get('input[type=checkbox]').setValue(true); await form.get('form').trigger('submit'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_connector_configure', { id: 'repo', values: {}, confirmed: true });
  });
  it('clears credentials after a failed submit and keeps an actionable error', async () => {
    await open({ auth: { type: 'token', schema, configured: ['HOST'] } });
    vi.mocked(invoke).mockRejectedValueOnce('服务不可用');
    const form = wrapper.getComponent(ConnectorTokenForm);
    await form.get('[aria-label="令牌"]').setValue('secret'); await form.get('input[type=checkbox]').setValue(true);
    await form.get('form').trigger('submit'); await flushPromises();
    expect(wrapper.text()).toContain('服务不可用'); expect((form.get('[aria-label="令牌"]').element as HTMLInputElement).value).toBe('');
  });
  it('enables, disables, and only unbinds after confirming credential removal', async () => {
    await open(); await click('启用');
    expect(invoke).toHaveBeenCalledWith('ai_connector_set_enabled', { id: 'repo', enabled: true });
    await click('停用'); expect(wrapper.text()).toContain('已停用');
    await click('断开并清除凭据'); expect(invoke).not.toHaveBeenCalledWith('ai_connector_unbind', expect.anything());
    await click('确认断开并清除凭据'); expect(invoke).toHaveBeenCalledWith('ai_connector_unbind', { id: 'repo' });
  });
  it('displays unsupported transports without allowing connection', async () => {
    await open({ unsupported: 'SSE 暂不可用' });
    expect(wrapper.text()).toContain('暂不支持'); expect(wrapper.findComponent(ConnectorTokenForm).exists()).toBe(false);
    expect(wrapper.findAll('button').find(b => b.text() === '启用')!.attributes('disabled')).toBeDefined();
  });
  it('opens OAuth in the browser and polls the existing flow commands', async () => {
    vi.useFakeTimers(); await open({ auth: { type: 'oauth', configured: [] } });
    await click('重新授权'); expect(invoke).toHaveBeenCalledWith('open_download_url', { url: 'https://auth.example/authorize' });
    await vi.advanceTimersByTimeAsync(1000); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_store_connector_auth_status', { flowId: 'flow' }); expect(wrapper.text()).toContain('已连接');
  });
  it('cancels OAuth and stops polling on cancel or unmount', async () => {
    vi.useFakeTimers(); await open({ auth: { type: 'oauth', configured: [] } });
    await click('重新授权'); await click('取消授权');
    expect(invoke).toHaveBeenCalledWith('ai_store_connector_auth_cancel', { flowId: 'flow' });
    await vi.advanceTimersByTimeAsync(2000); expect(invoke).not.toHaveBeenCalledWith('ai_store_connector_auth_status', expect.anything());
    await click('重新授权'); wrapper.unmount(); expect(vi.mocked(invoke).mock.calls.filter(([c]) => c === 'ai_store_connector_auth_cancel')).toHaveLength(2);
  });
  it('shows load errors and can retry', async () => {
    vi.mocked(invoke).mockRejectedValueOnce('读取失败');
    wrapper = mount(ConnectorCenter, { props: { store } }); await flushPromises();
    expect(wrapper.text()).toContain('读取失败'); await click('重试'); expect(wrapper.text()).toContain('代码仓库');
  });
});
