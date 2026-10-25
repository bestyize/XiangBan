// @vitest-environment happy-dom
import { mount, flushPromises } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { reactive } from 'vue';
import { NModal } from 'naive-ui';
import RegistryCatalog from './RegistryCatalog.vue';
import AiStorePage from '../pages/AiStorePage.vue';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import type { InstalledConnector, MarketConnector } from './connector-plugins.ts';

vi.mock('../../bridge.ts', () => ({ invoke: vi.fn(), listen: vi.fn().mockResolvedValue(() => {}) }));
vi.mock('naive-ui', async () => {
  const { defineComponent } = await import('vue');
  return {
    NModal: defineComponent({ props: ['show', 'title'], template: '<section v-if="show" class="test-modal"><h2>{{ title }}</h2><slot /></section>' }),
    NAlert: defineComponent({ template: '<div role="alert"><slot /></div>' }),
    NSkeleton: defineComponent({ template: '<div />' }),
  };
});
const connector: MarketConnector = {
  displayName: { zh: '文档连接器', en: 'Docs Connector' }, displayDescription: { zh: '连接团队文档', en: 'Connect docs' },
  examples: { zh: ['查找团队周报'], en: ['Find reports'] }, authTypes: ['token', 'oauth', 'env', 'none'],
  tokenFields: [{ server: 'docs', key: 'TOKEN', label: '访问令牌', type: 'password', required: true }, { server: 'docs', key: 'ORG', label: '组织名称', type: 'text', required: false }],
  transports: ['streamable-http', 'stdio'], skills: ['docs-search'], skillCount: 1, disabledToolCount: 3,
};
const entry = { slug: 'docs-plugin', name: 'Docs', description: 'Description', namespace: 'acme', version: '1.2.0', connector };
const installedItem: InstalledConnector = {
  id: 'docs-plugin:docs', plugin: entry.slug, name: '文档连接器', description: '连接团队文档', version: '1.2.0', source: 'market', iconUrl: null,
  auth: { type: 'token', configured: [], schema: { title: '文档令牌', fields: [{ key: 'TOKEN', label: '访问令牌', type: 'password', required: true }] } },
  status: 'needs_auth', enabled: true, tools: [], skills: ['docs-search'], examples: ['查找团队周报'],
};
const sha = 'a'.repeat(64);
const review = {
  sha256: sha,
  manifest: { version: '1.2.0', connectorCount: 2, skills: ['docs-search'], warnings: ['需要本机 Node 环境'],
    servers: [
      { name: 'docs', transport: 'streamable-http', url: 'https://example.com/mcp', envKeys: [], headerKeys: ['Authorization'] },
      { name: 'local-docs', transport: 'stdio', command: 'npx -y docs-mcp', envKeys: ['DOCS_TOKEN'], headerKeys: [] },
    ], files: [{ path: 'mcp.json', size: 12, sha256: sha, executableScript: false }] },
  warnings: ['包含本地命令'], scripts: [{ path: 'scripts/check.js', sha256: sha, preview: 'console.log("check")', previewTruncated: true }],
};
let listed: typeof entry[];
let local: InstalledConnector[];
const wrappers: ReturnType<typeof mount>[] = [];
function setup(page = false) {
  const store = reactive({ desktop: true, chat: { select: vi.fn().mockResolvedValue(undefined) }, creationDraft: null, navigate: vi.fn(), notify: vi.fn() }) as unknown as ControlStore;
  const wrapper = page ? mount(AiStorePage, { props: { store } }) : mount(RegistryCatalog, { props: { kind: 'connectors', query: '', category: '全部', source: 'market', store } });
  wrappers.push(wrapper); return { wrapper, store };
}
async function settle() { await vi.advanceTimersByTimeAsync(300); await flushPromises(); }
async function open(wrapper: ReturnType<typeof mount>) { await wrapper.findAll('.registry .small-action').at(-1)!.trigger('click'); await flushPromises(); }
async function inspect(wrapper: ReturnType<typeof mount>) { await open(wrapper); await wrapper.get('.market-connector-detail .store-primary').trigger('click'); await flushPromises(); }
beforeEach(() => {
  vi.useFakeTimers(); localStorage.clear(); listed = [structuredClone(entry)]; local = [];
  vi.mocked(invoke).mockReset().mockImplementation(async (c, a) => {
    switch (c) {
      case 'ai_market_browse': return { results: a?.kind === 'connectors' ? listed : [], page: 1, total: listed.length, hasMore: false };
      case 'ai_market_detail': return { item: listed[0], versions: [] };
      case 'ai_connector_installed': return local;
      case 'ai_store_connector_status': case 'ai_store_installed': return [];
      case 'ai_market_inspect': return structuredClone(review);
      case 'ai_market_install': local = [installedItem]; return;
      case 'ai_market_updates': return { results: [] };
      case 'ai_market_take_link': return null;
      case 'ai_market_categories': return { results: [] };
      case 'get_ai_config': return { openComputerUse: false };
      default: throw new Error('Unexpected ' + c);
    }
  });
});
afterEach(() => { wrappers.splice(0).forEach(w => w.unmount()); vi.useRealTimers(); });

describe('市场连接器插件', () => {
  it('shows localized cards and metadata without rendering token values or a config form', async () => {
    // Extra fields from the service must never be rendered as form values.
    Object.assign(listed[0].connector.tokenFields[0], { value: 'never-show-secret', placeholder: 'never-show-placeholder' });
    const { wrapper } = setup(); await settle();
    const card = wrapper.get('.registry .connector-card');
    expect(card.text()).toContain('文档连接器'); expect(card.text()).toContain('令牌认证');
    expect(card.text()).toContain('网页登录授权'); expect(card.text()).toContain('环境变量'); expect(card.text()).toContain('无需认证');
    expect(card.text()).toContain('1 个随包技能');
    await open(wrapper);
    const modal = wrapper.get('.test-modal');
    for (const text of ['连接团队文档', '查找团队周报', '访问令牌 · 必填', '组织名称 · 选填', 'streamable-http、stdio', '被策略禁用的工具：3 个', 'docs-search']) expect(modal.text()).toContain(text);
    expect(modal.text()).not.toContain('never-show'); expect(modal.find('input').exists()).toBe(false);
    expect(modal.find('textarea').exists()).toBe(false); expect(invoke).not.toHaveBeenCalledWith('ai_market_inspect', expect.anything());
  });
  it('uses English when Chinese presentation is missing and handles no-token plugins', async () => {
    Object.assign(listed[0].connector, { displayName: { en: 'English Docs' }, displayDescription: { en: 'English description' }, examples: { en: ['Find docs'] }, tokenFields: [] });
    const { wrapper } = setup(); await settle(); await open(wrapper);
    expect(wrapper.get('.test-modal').text()).toContain('English Docs'); expect(wrapper.text()).toContain('English description');
    expect(wrapper.text()).toContain('Find docs'); expect(wrapper.text()).toContain('无需填写令牌字段');
  });
  it('reviews servers, commands, key names, warnings, files and scripts, then installs only after consent', async () => {
    const { wrapper, store } = setup(); await settle(); await inspect(wrapper);
    expect(invoke).toHaveBeenCalledWith('ai_market_inspect', { kind: 'connectors', namespace: 'acme', slug: entry.slug, version: '1.2.0' });
    const modal = wrapper.get('.test-modal');
    for (const text of ['MCP 服务 · 2 个', 'https://example.com/mcp', '将运行的命令：npx -y docs-mcp', '环境变量名：DOCS_TOKEN', '请求头名：Authorization', '需要本机 Node 环境', '包含本地命令', 'mcp.json', 'scripts/check.js', '已截断', 'console.log("check")', sha]) expect(modal.text()).toContain(text);
    expect(modal.get('.store-primary').attributes('disabled')).toBeDefined();
    await modal.get('.store-primary').trigger('click'); expect(invoke).not.toHaveBeenCalledWith('ai_market_install', expect.anything());
    await modal.get('.review-confirm input').setValue(true); await modal.get('.store-primary').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_market_install', { kind: 'connectors', namespace: 'acme', slug: entry.slug, version: '1.2.0', sha256: sha });
    expect(wrapper.text()).toContain('已安装，下一步到连接器中心填写令牌并连接');
    expect(store.notify).toHaveBeenCalledWith('已安装，下一步到连接器中心填写令牌并连接');
    expect(wrapper.get('.registry .small-action').text()).toBe('已安装');
    expect(invoke).not.toHaveBeenCalledWith('ai_connector_configure', expect.anything());
    expect(invoke).not.toHaveBeenCalledWith('ai_store_connector_connect', expect.anything());
    await modal.get('.store-secondary').trigger('click'); expect(wrapper.emitted('center')).toEqual([[entry.slug]]);
  });
  it('matches installed services by plugin slug and shows updates only with matching market update information', async () => {
    local = [{ ...installedItem, version: '1.0.0' }, { ...installedItem, id: 'docs-plugin:second' }];
    const { wrapper } = setup(); await settle();
    expect(wrapper.get('.small-action').text()).toBe('已安装'); await open(wrapper);
    expect(wrapper.find('.market-connector-detail .store-primary').exists()).toBe(false);
    const updates = { [entry.slug]: { kind: 'connectors', namespace: 'acme', slug: entry.slug, installed: '1.0.0', latest: '1.2.0', hasUpdate: true } };
    await wrapper.setProps({ updates });
    expect(wrapper.get('.small-action').text()).toBe('更新');
    await wrapper.get('.market-connector-detail .store-primary').trigger('click'); await flushPromises();
    await wrapper.get('.review-confirm input').setValue(true); await wrapper.get('.market-connector-detail .store-primary').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_market_install', expect.objectContaining({ kind: 'connectors', version: '1.2.0', sha256: sha }));
  });
  it('ignores updates from another namespace or kind', async () => {
    local = [installedItem]; const { wrapper } = setup(); await settle();
    const info = { kind: 'skills', namespace: 'acme', slug: entry.slug, installed: '1.0.0', latest: '2.0.0', hasUpdate: true };
    await wrapper.setProps({ updates: { [entry.slug]: info } }); expect(wrapper.get('.small-action').text()).toBe('已安装');
    await wrapper.setProps({ updates: { [entry.slug]: { ...info, kind: 'connectors', namespace: 'other' } } }); expect(wrapper.get('.small-action').text()).toBe('已安装');
  });
  it.each(['listing', 'review'])('installs SSE from the %s only after review and consent without connecting', async source => {
    if (source === 'listing') listed[0].connector.transports = ['sse'];
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async (c, a) => c === 'ai_market_inspect' ? { ...review, manifest: { ...review.manifest, servers: [{ ...review.manifest.servers[0], transport: 'sse' }, review.manifest.servers[1]] } } : original(c, a));
    const { wrapper } = setup(); await settle(); await inspect(wrapper);
    expect(invoke).toHaveBeenCalledWith('ai_market_inspect', { kind: 'connectors', namespace: 'acme', slug: entry.slug, version: '1.2.0' });
    expect(wrapper.text()).toContain('docs · sse');
    expect(wrapper.text()).toContain(sha);
    expect(wrapper.get('.review-confirm input').attributes('disabled')).toBeUndefined();
    expect(wrapper.get('.market-connector-detail .store-primary').attributes('disabled')).toBeDefined();
    await wrapper.get('.market-connector-detail .store-primary').trigger('click');
    expect(invoke).not.toHaveBeenCalledWith('ai_market_install', expect.anything());
    await wrapper.get('.review-confirm input').setValue(true);
    await wrapper.get('.market-connector-detail .store-primary').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_market_install', { kind: 'connectors', namespace: 'acme', slug: entry.slug, version: '1.2.0', sha256: sha });
    expect(wrapper.text()).toContain('已安装，下一步到连接器中心填写令牌并连接');
    expect(invoke).not.toHaveBeenCalledWith('ai_connector_configure', expect.anything());
    expect(invoke).not.toHaveBeenCalledWith('ai_store_connector_connect', expect.anything());
  });
  it.each(['ai_market_inspect', 'ai_market_install'])('allows retry after %s fails without reporting success', async command => {
    const original = vi.mocked(invoke).getMockImplementation()!; let fail = true;
    vi.mocked(invoke).mockImplementation(async (c, a) => { if (c === command && fail) throw new Error('操作失败'); return original(c, a); });
    const { wrapper, store } = setup(); await settle(); await inspect(wrapper);
    if (command === 'ai_market_install') { await wrapper.get('.review-confirm input').setValue(true); await wrapper.get('.market-connector-detail .store-primary').trigger('click'); await flushPromises(); }
    expect(wrapper.text()).toContain('操作失败'); expect(store.notify).not.toHaveBeenCalled();
    fail = false; await wrapper.get('.market-connector-detail .store-primary').trigger('click'); await flushPromises();
    if (command === 'ai_market_inspect') expect(wrapper.find('.review-confirm').exists()).toBe(true);
    else expect(store.notify).toHaveBeenCalled();
  });
  it.each([{ sha256: '' }, { manifest: { ...review.manifest, version: '9.9.9' } }])('rejects an incomplete digest or a different reviewed version', async invalid => {
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async (c, a) => c === 'ai_market_inspect' ? { ...review, ...invalid } : original(c, a));
    const { wrapper } = setup(); await settle(); await inspect(wrapper);
    expect(wrapper.text()).toContain('审查版本或摘要不完整'); expect(wrapper.find('.review-confirm').exists()).toBe(false);
    expect(invoke).not.toHaveBeenCalledWith('ai_market_install', expect.anything());
  });
  it('recovers installed-state failures without sending an install request', async () => {
    const original = vi.mocked(invoke).getMockImplementation()!; let fail = true;
    vi.mocked(invoke).mockImplementation(async (c, a) => { if (c === 'ai_connector_installed' && fail) throw new Error('状态不可用'); return original(c, a); });
    const { wrapper } = setup(); await settle(); expect(wrapper.text()).toContain('无法读取已安装连接器');
    expect(wrapper.get('.small-action').attributes('disabled')).toBeDefined();
    fail = false; await wrapper.get('[role="alert"] button').trigger('click'); await flushPromises();
    expect(wrapper.get('.small-action').attributes('disabled')).toBeUndefined();
  });
  it('waits for installed state before offering installation', async () => {
    const original = vi.mocked(invoke).getMockImplementation()!;
    let resolveLocal!: (value: InstalledConnector[]) => void;
    vi.mocked(invoke).mockImplementation((c, a) => c === 'ai_connector_installed' ? new Promise(resolve => { resolveLocal = resolve; }) : original(c, a));
    const { wrapper } = setup(); await settle(); expect(wrapper.get('.small-action').attributes('disabled')).toBeDefined();
    resolveLocal([installedItem]); await flushPromises(); expect(wrapper.get('.small-action').text()).toBe('已安装');
    expect(wrapper.get('.small-action').attributes('disabled')).toBeUndefined();
  });
  it('discards a review that completes after closing the detail', async () => {
    const original = vi.mocked(invoke).getMockImplementation()!;
    let resolveReview!: (value: typeof review) => void;
    vi.mocked(invoke).mockImplementation((c, a) => c === 'ai_market_inspect' ? new Promise(resolve => { resolveReview = resolve; }) : original(c, a));
    const { wrapper } = setup(); await settle(); await inspect(wrapper);
    wrapper.findComponent(NModal).vm.$emit('update:show', false); await flushPromises();
    resolveReview(review); await flushPromises(); await open(wrapper);
    expect(wrapper.find('.review-confirm').exists()).toBe(false); expect(wrapper.text()).not.toContain('console.log');
    expect(invoke).not.toHaveBeenCalledWith('ai_market_install', expect.anything());
  });
  it('revokes the reviewed consent when an update target changes', async () => {
    local = [{ ...installedItem, version: '1.0.0' }]; const { wrapper } = setup(); await settle();
    const info = { kind: 'connectors', namespace: 'acme', slug: entry.slug, installed: '1.0.0', latest: '1.2.0', hasUpdate: true };
    await wrapper.setProps({ updates: { [entry.slug]: info } }); await inspect(wrapper); await wrapper.get('.review-confirm input').setValue(true);
    await wrapper.setProps({ updates: { [entry.slug]: { ...info, latest: '1.3.0' } } });
    expect(wrapper.find('.review-confirm').exists()).toBe(false);
    await wrapper.get('.market-connector-detail .store-primary').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenLastCalledWith('ai_market_inspect', expect.objectContaining({ version: '1.3.0' }));
    expect(invoke).not.toHaveBeenCalledWith('ai_market_install', expect.anything());
  });
  it('does not install when no version is available', async () => {
    listed[0].version = ''; const { wrapper } = setup(); await settle(); await open(wrapper);
    expect(wrapper.text()).toContain('暂未提供可安装的版本'); expect(wrapper.get('.market-connector-detail .store-primary').attributes('disabled')).toBeDefined();
    expect(invoke).not.toHaveBeenCalledWith('ai_market_inspect', expect.anything());
  });
  it('keeps old market config templates on their existing connect path', async () => {
    const original = vi.mocked(invoke).getMockImplementation()!;
    listed = [{ ...entry, connector: undefined } as unknown as typeof entry];
    vi.mocked(invoke).mockImplementation(async (c, a) => {
      if (c === 'ai_market_detail') return { item: { metadata: { configs: [{ name: 'legacy', config: { url: 'https://legacy.example/mcp' } }] } } };
      if (c === 'ai_store_connector_connect') return { status: 'connected' };
      return original(c, a);
    });
    const { wrapper } = setup(); await settle(); await open(wrapper);
    expect(wrapper.get('input[aria-label="服务地址"]').element).toHaveProperty('value', 'https://legacy.example/mcp');
    await wrapper.get('.review-confirm input').setValue(true); await wrapper.get('.store-primary').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_store_connector_connect', expect.objectContaining({ id: entry.slug, confirmed: true }));
    expect(invoke).not.toHaveBeenCalledWith('ai_market_inspect', expect.anything());
  });
  it.each(['streamable-http', 'sse'])('opens the newly installed %s plugin token form in the connector center and clears search', async transport => {
    listed[0].connector.transports = [transport];
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async (c, a) => c === 'ai_market_inspect' ? { ...review, manifest: { ...review.manifest, servers: [{ ...review.manifest.servers[0], transport }, review.manifest.servers[1]] } } : original(c, a));
    const { wrapper } = setup(true); await settle(); await wrapper.get('#store-tab-connectors').trigger('click'); await settle();
    await wrapper.get('input[aria-label="搜索专家、技能或连接器"]').setValue('Docs'); await settle();
    await inspect(wrapper); await wrapper.get('.review-confirm input').setValue(true); await wrapper.get('.market-connector-detail .store-primary').trigger('click'); await flushPromises();
    await wrapper.get('.market-connector-detail .store-secondary').trigger('click'); await flushPromises();
    expect(wrapper.find('.market-connector-detail').exists()).toBe(false);
    expect(wrapper.get('#store-tab-connectors').attributes('aria-selected')).toBe('true');
    expect(wrapper.get('input[aria-label="搜索专家、技能或连接器"]').element).toHaveProperty('value', '');
    expect(wrapper.get('.connector-center .test-modal').text()).toContain('文档令牌');
    expect(wrapper.get('.connector-center input[type="password"]').element).toHaveProperty('value', '');
    expect(invoke).not.toHaveBeenCalledWith('ai_connector_configure', expect.anything());
  });
  it('queues opening the plugin while the connector center is still loading after install', async () => {
    const original = vi.mocked(invoke).getMockImplementation()!;
    let delay = false; const resolvers: ((value: InstalledConnector[]) => void)[] = [];
    vi.mocked(invoke).mockImplementation(async (c, a) => {
      if (c === 'ai_connector_installed' && delay) return new Promise(resolve => resolvers.push(resolve));
      const result = await original(c, a); if (c === 'ai_market_install') delay = true; return result;
    });
    const { wrapper } = setup(true); await settle(); await wrapper.get('#store-tab-connectors').trigger('click'); await settle();
    await inspect(wrapper); await wrapper.get('.review-confirm input').setValue(true); await wrapper.get('.market-connector-detail .store-primary').trigger('click'); await flushPromises();
    await wrapper.get('.market-connector-detail .store-secondary').trigger('click'); await flushPromises();
    expect(wrapper.find('.connector-center input[type="password"]').exists()).toBe(false);
    delay = false; resolvers.forEach(resolve => resolve([installedItem])); await flushPromises();
    expect(wrapper.get('.connector-center .test-modal').text()).toContain('文档令牌');
  });
});
