// @vitest-environment happy-dom
// 小伴市场作为默认来源：浏览、审查安装、故障退回 SkillHub、更新、专家团、网页唤起。
import { mount, flushPromises } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { reactive } from 'vue';
import AiStorePage from './AiStorePage.vue';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import type { Skill } from '../ai-store/catalog.ts';

vi.mock('../../bridge.ts', () => ({ invoke: vi.fn(), listen: vi.fn().mockResolvedValue(() => {}) }));
vi.mock('naive-ui', async () => {
  const { defineComponent } = await import('vue');
  return {
    NModal: defineComponent({ props: ['show'], template: '<section v-if="show" class="test-modal"><slot /></section>' }),
    NAlert: defineComponent({ template: '<div role="alert"><slot /></div>' }),
    NSkeleton: defineComponent({ template: '<div />' }),
  };
});

const sha = 'a'.repeat(64);
const item = { slug: 'pdf-kit', name: 'PDF 文档处理', description: '处理 PDF', version: '1.2.0', namespace: 'acme', source: 'official', variant: 'single' };
const review = { slug: 'pdf-kit', namespace: 'acme', sha256: sha, content: '---\nname: pdf-kit\n---\n读取 PDF',
  manifest: { version: '1.2.0', fileCount: 2, totalBytes: 128, files: [
    { path: 'SKILL.md', size: 100, sha256: 'b'.repeat(64), executableScript: false },
    { path: 'scripts/run.py', size: 28, sha256: 'c'.repeat(64), executableScript: true },
  ] }, scripts: [{ path: 'scripts/run.py', sha256: 'c'.repeat(64), preview: 'print("hi")', previewTruncated: false }] };
const team = { slug: 'launch-team', name: '产品发布小组', description: '四位专家协作', version: '1.0.0', namespace: 'acme', source: 'official', variant: 'team', memberCount: 2 };
const groupReview = { name: '产品发布小组', kind: 'experts', sha256: 'd'.repeat(64), members: [
  { ...review, slug: 'req-analyst', name: '需求梳理', state: 'pending', kind: 'experts' },
  { ...review, slug: 'fe-engineer', name: '前端开发工程师', state: 'installed', kind: 'experts' },
] };
const detailOf = (over = {}) => ({ item: { ...item, ...over }, versions: [{ version: '1.2.0', yanked: false, manifestSha256: sha }], members: [] });

let local: Skill[];
let marketUp: boolean;
let pendingLink: unknown;
let updateList: unknown[];
const wrappers: ReturnType<typeof mount>[] = [];
function setup() {
  const store = reactive({ desktop: true, chat: { select: vi.fn().mockResolvedValue(undefined) }, creationDraft: null, navigate: vi.fn(), notify: vi.fn() }) as unknown as ControlStore;
  const wrapper = mount(AiStorePage, { props: { store } });
  wrappers.push(wrapper);
  return { wrapper, store };
}
async function settle() { await vi.advanceTimersByTimeAsync(300); await flushPromises(); }
async function skills(wrapper: ReturnType<typeof mount>) { await wrapper.get('#store-tab-skills').trigger('click'); await settle(); }

beforeEach(() => {
  vi.useFakeTimers(); localStorage.clear();
  local = []; marketUp = true; pendingLink = null; updateList = [];
  vi.mocked(invoke).mockReset().mockImplementation(async (command, args) => {
    switch (command) {
      case 'ai_store_installed': return local;
      case 'get_ai_config': return { openComputerUse: false };
      case 'ai_market_take_link': { const link = pendingLink; pendingLink = null; return link; }
      case 'ai_market_updates': return { results: updateList };
      case 'ai_market_categories': return { results: [{ key: 'doc', name: '文档处理' }] };
      case 'ai_market_browse':
        if (!marketUp) throw new Error('暂时无法连接小伴市场');
        if (args?.kind === 'experts') return { results: [team], page: 1, total: 1, hasMore: false };
        if (args?.kind === 'connectors') return { results: [{ slug: 'web-search', name: '网页搜索', description: '联网检索', namespace: 'acme', category: '搜索与信息' }], page: 1, total: 1, hasMore: false };
        return { results: [item], page: 1, total: 1, hasMore: false };
      case 'ai_market_detail': return args?.kind === 'connectors'
        ? { item: { description: '服务说明', metadata: { configs: [{ name: 'web', config: { url: 'https://example.com/mcp' } }] } }, versions: [] }
        : detailOf();
      case 'ai_market_inspect': return review;
      case 'ai_market_install': { local = [{ ...item, removable: true, source: '小伴市场', market: true, marketKind: 'skill' } as Skill]; return; }
      case 'ai_market_group_inspect': return groupReview;
      case 'ai_market_group_install': return { installed: 1, skipped: 1 };
      case 'ai_market_update': return;
      case 'ai_store_skill_categories': return { results: [{ key: 'office', name: '办公效率' }] };
      case 'ai_store_browse': return { results: [{ slug: 'hub-skill', name: 'Hub 技能', description: '来自 SkillHub', version: '2.0.0' }], page: 1, total: 1, hasMore: false };
      case 'ai_store_experts_installed': case 'ai_store_connector_status': return [];
      case 'ai_store_catalog': return { results: [], page: 1, total: 0, hasMore: false };
      default: throw new Error(`Unexpected command ${String(command)}`);
    }
  });
});
afterEach(() => { wrappers.splice(0).forEach(w => w.unmount()); vi.useRealTimers(); });

describe('小伴市场', () => {
  it('is the default source and installs exactly the digest that was reviewed', async () => {
    const { wrapper, store } = setup(); await skills(wrapper);
    expect(invoke).toHaveBeenCalledWith('ai_market_browse', { kind: 'skills', keyword: '', page: 1, category: '', source: '', sortBy: 'hot', variant: '' });
    expect(invoke).not.toHaveBeenCalledWith('ai_store_browse', expect.anything());
    expect(wrapper.get('.skill-card .xc-tag').text()).toBe('小伴市场 · 官方');
    await wrapper.get('.skill-card .small-action').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_market_inspect', { kind: 'skills', namespace: 'acme', slug: 'pdf-kit', version: '1.2.0' });
    expect(wrapper.get('.version-lock').text()).toContain('aaaaaaaa…aaaaaa');
    expect(wrapper.get('.test-modal .store-primary').attributes('disabled')).toBeDefined();
    await wrapper.get('.review-confirm input').setValue(true);
    await wrapper.get('.test-modal .store-primary').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_market_install', { kind: 'skills', namespace: 'acme', slug: 'pdf-kit', version: '1.2.0', sha256: sha });
    expect(store.notify).toHaveBeenCalledWith('已安装「PDF 文档处理」，可在新任务中使用');
  });

  it('shows market categories by their keys and passes the chosen one to the search', async () => {
    const { wrapper } = setup(); await skills(wrapper);
    await wrapper.findAll('.store-filters button').find(b => b.text() === '文档处理')!.trigger('click'); await settle();
    expect(invoke).toHaveBeenCalledWith('ai_market_browse', expect.objectContaining({ kind: 'skills', category: 'doc' }));
  });

  it('shows SkillHub while the market is unreachable and goes back on retry', async () => {
    marketUp = false;
    const { wrapper } = setup(); await skills(wrapper); await settle();
    expect(wrapper.get('.market-down').text()).toContain('暂时无法连接小伴市场');
    expect(invoke).toHaveBeenCalledWith('ai_store_browse', expect.anything());
    expect(wrapper.get('.skill-card .xc-tag').text()).toBe('SkillHub');
    marketUp = true;
    await wrapper.get('.market-down .text-action').trigger('click'); await settle();
    expect(wrapper.find('.market-down').exists()).toBe(false);
    expect(wrapper.get('.skill-card .xc-tag').text()).toBe('小伴市场 · 官方');
  });

  it('does not treat "no such item" as the market being down', async () => {
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async (c, a) => { if (c === 'ai_market_browse') throw new Error('条目不存在'); return original(c, a); });
    const { wrapper } = setup(); await skills(wrapper);
    expect(wrapper.find('.market-down').exists()).toBe(false);
    expect(wrapper.text()).toContain('暂时无法加载小伴市场：');
    expect(invoke).not.toHaveBeenCalledWith('ai_store_browse', expect.anything());
  });

  it('remembers a choice of SkillHub as the source', async () => {
    const { wrapper } = setup(); await skills(wrapper);
    await wrapper.findAll('.source-switch button')[1].trigger('click'); await settle();
    expect(localStorage.getItem('xb.market.source')).toBe('hub');
    expect(invoke).toHaveBeenCalledWith('ai_store_browse', expect.anything());
  });

  it('lists market experts like skills, and a team is reviewed and installed together', async () => {
    const { wrapper, store } = setup(); await settle();
    expect(invoke).toHaveBeenCalledWith('ai_market_browse', expect.objectContaining({ kind: 'experts' }));
    expect(wrapper.get('.skill-card .xc-tag').text()).toBe('专家团');
    expect(wrapper.get('.skill-card .xc-meta').text()).toBe('2 位专家');
    await wrapper.get('.skill-card .small-action').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_market_group_inspect', { kind: 'experts', namespace: 'acme', slug: 'launch-team' });
    expect(wrapper.get('.test-modal').text()).toContain('已安装 · 跳过');
    const install = wrapper.get('.test-modal .store-primary');
    expect(install.attributes('disabled')).toBeDefined();
    await wrapper.get('.test-modal .review-confirm input').setValue(true);
    await wrapper.get('.test-modal .store-primary').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_market_group_install', { kind: 'experts', namespace: 'acme', slug: 'launch-team', sha256: 'd'.repeat(64) });
    expect(store.notify).toHaveBeenCalledWith('已安装「产品发布小组」的 1 位专家，跳过 1 个已安装');
  });

  it('offers an update with the file differences and sends the reviewed digest', async () => {
    local = [{ slug: 'pdf-kit', name: 'PDF 文档处理', description: '处理 PDF', version: '1.0.0', namespace: 'acme', removable: true, source: '小伴市场',
      market: true, marketKind: 'skill', manifestSha256: 'e'.repeat(64), files: [{ path: 'SKILL.md', sha256: 'old' }, { path: 'references/legacy.md', sha256: 'x' }] }];
    updateList = [{ kind: 'skill', namespace: 'acme', slug: 'pdf-kit', installed: '1.0.0', latest: '1.2.0', hasUpdate: true }];
    const { wrapper, store } = setup(); await skills(wrapper);
    expect(wrapper.find('.update-dot').exists()).toBe(true);
    expect(wrapper.get('.update-hint').text()).toContain('1 个已安装技能有新版本');
    await wrapper.get('.update-hint .text-action').trigger('click'); await settle();
    expect(wrapper.get('.skill-card .xc-meta').text()).toBe('v1.0.0 → v1.2.0');
    await wrapper.get('.skill-card .xc-pill-fill').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_market_inspect', { kind: 'skills', namespace: 'acme', slug: 'pdf-kit', version: '1.2.0' });
    const text = wrapper.get('.update-dialog, .test-modal').text();
    expect(text).toContain('修改'); expect(text).toContain('新增'); expect(text).toContain('删除');
    expect(text).toContain('旧版本保留为备份');
    await wrapper.get('.test-modal .review-confirm input').setValue(true);
    await wrapper.get('.test-modal .store-primary').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_market_update', { kind: 'skills', namespace: 'acme', slug: 'pdf-kit', version: '1.2.0', sha256: sha });
    expect(store.notify).toHaveBeenCalledWith('「PDF 文档处理」已更新到 v1.2.0，旧版本已备份');
  });

  it('can switch update checks off', async () => {
    updateList = [{ kind: 'skill', namespace: 'acme', slug: 'pdf-kit', installed: '1.0.0', latest: '1.2.0', hasUpdate: true }];
    local = [{ slug: 'pdf-kit', name: 'PDF', description: '', version: '1.0.0', namespace: 'acme', removable: true, source: '小伴市场', market: true, marketKind: 'skill' }];
    localStorage.setItem('xb.market.autoUpdates', '0');
    const { wrapper } = setup(); await skills(wrapper);
    expect(invoke).not.toHaveBeenCalledWith('ai_market_updates');
    expect(wrapper.find('.update-hint').exists()).toBe(false);
  });
});

describe('网页唤起', () => {
  const link = (over = {}) => ({ action: 'install', kind: 'skills', namespace: 'acme', slug: 'pdf-kit', version: '1.2.0', sha256: sha, ...over });

  it('asks before anything else and goes on to the normal review when the digest matches', async () => {
    pendingLink = link();
    const { wrapper } = setup(); await settle();
    expect(wrapper.get('.link-dialog, .test-modal').text()).toContain('网页只能请求打开这个确认窗口');
    expect(invoke).toHaveBeenCalledWith('ai_market_detail', { kind: 'skills', namespace: 'acme', slug: 'pdf-kit' });
    expect(invoke).not.toHaveBeenCalledWith('ai_market_inspect', expect.anything());
    expect(invoke).not.toHaveBeenCalledWith('ai_market_install', expect.anything());
    await wrapper.get('.test-modal .store-primary').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_market_inspect', { kind: 'skills', namespace: 'acme', slug: 'pdf-kit', version: '1.2.0' });
    expect(wrapper.get('.test-modal').text()).toContain('确认安装');
    expect(invoke).not.toHaveBeenCalledWith('ai_market_install', expect.anything());
  });

  it('refuses a link whose digest is not the one the market records', async () => {
    pendingLink = link({ sha256: 'f'.repeat(64) });
    const { wrapper } = setup(); await settle();
    expect(wrapper.get('.test-modal').text()).toContain('已拒绝安装');
    expect(wrapper.find('.test-modal .store-primary').exists()).toBe(false);
    expect(invoke).not.toHaveBeenCalledWith('ai_market_inspect', expect.anything());
  });

  it('reviews the newest version when the link names an older one', async () => {
    pendingLink = link({ version: '1.1.0', sha256: 'b'.repeat(64) });
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async (c, a) => c === 'ai_market_detail' ? { ...detailOf(), versions: [
      { version: '1.2.0', yanked: false, manifestSha256: sha }, { version: '1.1.0', yanked: false, manifestSha256: 'b'.repeat(64) }] } : original(c, a));
    const { wrapper } = setup(); await settle();
    expect(wrapper.get('.test-modal').text()).toContain('不是最新版本');
    await wrapper.get('.test-modal .store-primary').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_market_inspect', { kind: 'skills', namespace: 'acme', slug: 'pdf-kit', version: '1.2.0' });
  });

  it('closing the dialog leaves nothing installed', async () => {
    pendingLink = link();
    const { wrapper } = setup(); await settle();
    await wrapper.get('.test-modal .store-secondary').trigger('click'); await flushPromises();
    expect(wrapper.find('.test-modal').exists()).toBe(false);
    expect(invoke).not.toHaveBeenCalledWith('ai_market_install', expect.anything());
  });
});

describe('连接器', () => {
  it('reads the connection template from the market, then uses the usual connection flow', async () => {
    const { wrapper } = setup(); await settle();
    await wrapper.get('#store-tab-connectors').trigger('click'); await settle();
    expect(invoke).toHaveBeenCalledWith('ai_market_browse', expect.objectContaining({ kind: 'connectors' }));
    await wrapper.get('.connector-card:not([style]) .small-action, .store-card.connector-card:last-of-type .small-action').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_market_detail', { kind: 'connectors', namespace: 'acme', slug: 'web-search' });
    expect(invoke).not.toHaveBeenCalledWith('ai_store_connector_detail', expect.anything());
    expect(wrapper.get('.test-modal').text()).toContain('服务说明');
  });
});
