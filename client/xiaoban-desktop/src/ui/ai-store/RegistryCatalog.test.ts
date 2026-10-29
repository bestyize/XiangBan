// @vitest-environment happy-dom
import { mount, flushPromises } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { reactive } from 'vue';
import RegistryCatalog from './RegistryCatalog.vue';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
vi.mock('../../bridge.ts', () => ({ invoke: vi.fn() }));
vi.mock('naive-ui', async () => {
  const { defineComponent } = await import('vue');
  return {
    NModal: defineComponent({ props: ['show'], template: '<section v-if="show" class="test-modal"><slot /></section>' }),
    NAlert: defineComponent({ template: '<div role="alert"><slot /></div>' }),
    NSkeleton: defineComponent({ template: '<div />' }),
  };
});
const entry = { slug: 'real-pack', name: '真实工作流', description: '来自目录', skillCount: 1 };
const review = { ...entry, entrySkill: 'expert-real-pack', sha256: 'a'.repeat(64), content: '工作流说明',
  dependencies: [{ slug: 'child-skill', namespace: 'publisher', version: '1.0.0', sha256: 'b'.repeat(64) }],
  skills: [{ slug: 'child-skill', content: '真实技能内容', manifest: { version: '1.0.0', fileCount: 1, files: [{ path: 'SKILL.md', sha256: 'c'.repeat(64) }] }, scripts: [] }] };
const wrappers: ReturnType<typeof mount>[] = [];
function setup(kind: 'experts' | 'connectors' = 'experts') {
  const store = reactive({ desktop: true, chat: { select: vi.fn().mockResolvedValue(undefined) }, creationDraft: null, navigate: vi.fn(), notify: vi.fn() }) as unknown as ControlStore;
  const wrapper = mount(RegistryCatalog, { props: { kind, query: '', category: '全部', store } });
  wrappers.push(wrapper); return { wrapper, store };
}
async function settle() { await vi.advanceTimersByTimeAsync(300); await flushPromises(); }
beforeEach(() => {
  vi.useFakeTimers();
  vi.mocked(invoke).mockReset().mockImplementation(async (command) => {
    if (command === 'ai_store_catalog') return { results: [entry], page: 1, total: 1, hasMore: false };
    if (command === 'ai_store_experts_installed' || command === 'ai_store_connector_status') return [];
    if (command === 'ai_store_expert_review') return review;
    if (command === 'ai_store_expert_install') return undefined;
    if (command === 'ai_store_connector_detail') return { readme: '真实服务说明', configs: [{ name: 'test', config: { url: 'https://example.com/mcp' } }] };
    throw new Error('Unexpected ' + command);
  });
});
afterEach(() => { wrappers.splice(0).forEach(w => w.unmount()); vi.useRealTimers(); });
describe('SkillHub registry integration', () => {
  it('opens browser authorization and waits for verified connection status',async()=>{
    const original=vi.mocked(invoke).getMockImplementation()!;let checks=0,connected=false;
    vi.mocked(invoke).mockImplementation(async(c,a)=>{
      if(c==='ai_store_connector_connect')return {status:'authorization_required',flowId:'flow-one',authorizationUrl:'https://login.example.com/authorize'};
      if(c==='open_download_url')return undefined;
      if(c==='ai_store_connector_auth_status'){checks++;if(checks===1)return {status:'authorizing'};connected=true;return {status:'connected'};}
      if(c==='ai_store_connector_status')return connected?[{id:entry.slug,name:entry.name,connected:true,toolCount:2}]:[];
      return original(c,a);
    });
    const {wrapper,store}=setup('connectors');await settle();await wrapper.get('.connector-card .small-action').trigger('click');await flushPromises();await wrapper.get('.review-confirm input').setValue(true);await wrapper.get('.store-primary').trigger('click');await flushPromises();
    expect(invoke).toHaveBeenCalledWith('open_download_url',{url:'https://login.example.com/authorize'});expect(wrapper.text()).toContain('等待网页授权');expect(wrapper.get('.connection-status').text()).toBe('未配置');expect(store.notify).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(1000);await flushPromises();expect(wrapper.get('.connection-status').text()).toBe('未配置');
    await vi.advanceTimersByTimeAsync(1000);await flushPromises();expect(wrapper.get('.connection-status').text()).toContain('已连接');expect(store.notify).toHaveBeenCalledWith('授权完成，连接成功');
  });
  it('cancels browser authorization without leaving an active poll or a connected card',async()=>{
    const original=vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async(c,a)=>{
      if(c==='ai_store_connector_connect')return {status:'authorization_required',flowId:'cancel-me',authorizationUrl:'https://login.example.com/authorize'};
      if(c==='open_download_url'||c==='ai_store_connector_auth_cancel')return undefined;
      if(c==='ai_store_connector_auth_status')return {status:'cancelled'};
      return original(c,a);
    });
    const {wrapper}=setup('connectors');await settle();await wrapper.get('.connector-card .small-action').trigger('click');await flushPromises();await wrapper.get('.review-confirm input').setValue(true);await wrapper.get('.store-primary').trigger('click');await flushPromises();
    await wrapper.get('.authorization-wait .store-secondary').trigger('click');await flushPromises();expect(invoke).toHaveBeenCalledWith('ai_store_connector_auth_cancel',{flowId:'cancel-me'});
    await vi.advanceTimersByTimeAsync(2000);await flushPromises();expect(vi.mocked(invoke).mock.calls.filter(([c])=>c==='ai_store_connector_auth_status')).toHaveLength(1);expect(wrapper.get('.connection-status').text()).toBe('未配置');
    expect(wrapper.find('[role="alert"]').exists()).toBe(false);expect((wrapper.get('input[aria-label="服务地址"]').element as HTMLInputElement).value).toBe('https://example.com/mcp');expect((wrapper.get('.review-confirm input').element as HTMLInputElement).checked).toBe(false);
  });
  it('keeps a completed connection when cancellation arrives before the next poll',async()=>{
    const original=vi.mocked(invoke).getMockImplementation()!;
    let connected=false;
    vi.mocked(invoke).mockImplementation(async(c,a)=>{
      if(c==='ai_store_connector_connect')return {status:'authorization_required',flowId:'completed',authorizationUrl:'https://login.example.com/authorize'};
      if(c==='open_download_url')return undefined;
      if(c==='ai_store_connector_auth_cancel'){connected=true;return undefined;}
      if(c==='ai_store_connector_auth_status')return {status:'connected'};
      if(c==='ai_store_connector_status')return connected?[{id:entry.slug,name:entry.name,connected:true,toolCount:2}]:[];
      return original(c,a);
    });
    const {wrapper,store}=setup('connectors');await settle();await wrapper.get('.connector-card .small-action').trigger('click');await flushPromises();await wrapper.get('.review-confirm input').setValue(true);await wrapper.get('.store-primary').trigger('click');await flushPromises();
    await wrapper.get('.authorization-wait .store-secondary').trigger('click');await flushPromises();
    expect(wrapper.get('.connection-status').text()).toContain('已连接');
    expect(store.notify).toHaveBeenCalledWith('授权完成，连接成功');
    expect(wrapper.text()).not.toContain('授权已取消');
  });
  it('reviews the whole expert pack before sending the pinned install request', async () => {
    const { wrapper } = setup(); await settle();
    await wrapper.get('.expert-card .small-action').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_store_expert_review', { slug: 'real-pack' });
    expect(wrapper.get('.test-modal').text()).toContain('@publisher/child-skill');
    expect(wrapper.get('.test-modal .store-primary').attributes('disabled')).toBeUndefined();
    expect(wrapper.find('.review-confirm').exists()).toBe(false);
    await wrapper.get('.test-modal .store-primary').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_store_expert_install', { slug: 'real-pack', sha256: review.sha256 });
  });
  it('shows catalog failure without replacing it with sample experts', async () => {
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation((c, a) => c === 'ai_store_catalog' ? Promise.reject(new Error('offline')) : original(c, a));
    const { wrapper } = setup(); await settle();
    expect(wrapper.findAll('.expert-card')).toHaveLength(0);
    expect(wrapper.get('[role="alert"]').text()).toContain('offline');
    vi.mocked(invoke).mockImplementation(original);
    await wrapper.get('.text-action').trigger('click'); await flushPromises();
    expect(wrapper.findAll('.expert-card')).toHaveLength(1);
  });
  it('discards late catalog responses after changing the query', async () => {
    let resolveOld!: (v: unknown) => void;
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation((c, a) => c === 'ai_store_catalog' && a?.keyword === 'old'
      ? new Promise(resolve => { resolveOld = resolve; }) : original(c, a));
    const { wrapper } = setup(); await settle();
    await wrapper.setProps({ query: 'old' }); await settle();
    await wrapper.setProps({ query: 'new' }); await settle();
    resolveOld({ results: [{ ...entry, name: '过期结果' }], page: 1, total: 1, hasMore: false }); await flushPromises();
    expect(wrapper.text()).not.toContain('过期结果');
  });
  it('does not mark failed MCP connections as connected', async () => {
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation((c, a) => c === 'ai_store_connector_connect' ? Promise.reject(new Error('握手失败')) : original(c, a));
    const { wrapper } = setup('connectors'); await settle();
    await wrapper.get('.connector-card .small-action').trigger('click'); await flushPromises();
    await wrapper.get('.review-confirm input').setValue(true);
    await wrapper.get('.store-primary').trigger('click'); await flushPromises();
    expect(wrapper.get('[role="alert"]').text()).toContain('握手失败');
    expect(wrapper.get('.connection-status').text()).toBe('未配置');
  });
  it('invalidates consent after editing config and rejects malformed JSON locally', async () => {
    const { wrapper } = setup('connectors'); await settle();
    await wrapper.get('.connector-card .small-action').trigger('click'); await flushPromises();
    await wrapper.get('.review-confirm input').setValue(true);
    await wrapper.get('textarea').setValue('{');
    expect(wrapper.get('.store-primary').attributes('disabled')).toBeDefined();
    await wrapper.get('.review-confirm input').setValue(true);
    await wrapper.get('.store-primary').trigger('click'); await flushPromises();
    expect(invoke).not.toHaveBeenCalledWith('ai_store_connector_connect', expect.anything());
    expect(wrapper.find('[role="alert"]').exists()).toBe(true);
  });
  it('can disconnect a saved connection when the remote catalog detail is offline', async () => {
    const original = vi.mocked(invoke).getMockImplementation()!;
    let connected = true;
    vi.mocked(invoke).mockImplementation(async (c, a) => {
      if (c === 'ai_store_connector_status') return [{ id: entry.slug, name: entry.name, connected, toolCount: 2 }];
      if (c === 'ai_store_connector_detail') throw new Error('catalog offline');
      if (c === 'ai_store_connector_disconnect') { connected = false; return undefined; }
      return original(c, a);
    });
    const { wrapper } = setup('connectors'); await settle();
    await wrapper.get('.connector-card .small-action').trigger('click'); await flushPromises();
    expect(wrapper.get('[role="alert"]').text()).toContain('catalog offline');
    await wrapper.get('.connection-summary button').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_store_connector_disconnect', { id: entry.slug });
    expect(wrapper.get('.connection-status').text()).toBe('未连接');
  });
});
