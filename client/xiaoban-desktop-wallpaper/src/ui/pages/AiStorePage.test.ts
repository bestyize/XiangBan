// @vitest-environment happy-dom
import { mount, flushPromises } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { reactive } from 'vue';
import AiStorePage from './AiStorePage.vue';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import type { Skill } from '../ai-store/catalog.ts';

vi.mock('../../bridge.ts', () => ({ invoke: vi.fn() }));
vi.mock('naive-ui', async () => {
  const { defineComponent } = await import('vue');
  return {
    NModal: defineComponent({ props: ['show'], template: '<section v-if="show" class="test-modal"><slot /></section>' }),
    NAlert: defineComponent({ template: '<div role="alert"><slot /></div>' }),
    NSkeleton: defineComponent({ template: '<div />' }),
  };
});
const remoteSkill = { slug: 'demo-skill', name: '网页总结', description: '整理网页内容', version: '1.2.3' };
const manifestHash = 'a'.repeat(64);
const review = { slug: remoteSkill.slug, sha256: manifestHash, content: '---\nname: demo-skill\n---\n请阅读网页',
  manifest: { version: '1.2.3', fileCount: 2, totalBytes: 128, files: [
    { path: 'SKILL.md', size: 100, sha256: 'b'.repeat(64), executableScript: false },
    { path: 'scripts/run.py', size: 28, sha256: 'c'.repeat(64), executableScript: true },
  ] }, scripts: [{ path: 'scripts/run.py', sha256: 'c'.repeat(64), preview: 'print("hello")', previewTruncated: false }] };
let local: Skill[];
let enabled: boolean;
const wrappers: ReturnType<typeof mount>[] = [];
function setup() {
  const store = reactive({ desktop: true, chat: { select: vi.fn().mockResolvedValue(undefined) },
    creationDraft: null, navigate: vi.fn(), notify: vi.fn() }) as unknown as ControlStore;
  const wrapper = mount(AiStorePage, { props: { store } });
  wrappers.push(wrapper);
  return { wrapper, store };
}
async function settle() { await vi.advanceTimersByTimeAsync(300); await flushPromises(); }
async function skills(wrapper: ReturnType<typeof mount>) { await wrapper.get('#store-tab-skills').trigger('click'); await settle(); }

beforeEach(() => {
  vi.useFakeTimers(); local = []; enabled = false;
  vi.mocked(invoke).mockReset().mockImplementation(async (command, args) => {
    if (command === 'ai_store_installed') return local;
    if (command === 'get_ai_config') return { openComputerUse: enabled };
    if (command === 'ai_store_browse') return { results: [remoteSkill], page: 1, total: 1, hasMore: false };
    if (command === 'ai_store_inspect') return review;
    if (command === 'ai_store_install') { local = [{ ...remoteSkill, removable: true, source: 'SkillHub' }]; return; }
    if (command === 'ai_store_uninstall') { local = []; return; }
    if (command === 'save_ai_preferences') { enabled = !!args?.openComputerUse; return { openComputerUse: enabled }; }
    throw new Error(`Unexpected command ${command}`);
  });
});
afterEach(() => { wrappers.splice(0).forEach(w => w.unmount()); vi.useRealTimers(); });

describe('AI 商店', () => {
  it('shows the compact expert grid with search and category filters', async () => {
    const { wrapper } = setup(); await flushPromises();
    expect(wrapper.findAll('.expert-card')).toHaveLength(6);
    expect(wrapper.find('h1').exists()).toBe(false);
    expect(wrapper.find('.hero').exists()).toBe(false);
    expect(wrapper.find('aside').exists()).toBe(false);
    await wrapper.get('input[aria-label="搜索专家、技能或连接器"]').setValue('编程');
    expect(wrapper.findAll('.expert-card')).toHaveLength(1);
    expect(wrapper.get('.expert-card').text()).toContain('编程开发专家');
    await wrapper.get('input').setValue('');
    await wrapper.findAll('.store-filters button').find(b => b.text() === '创作')!.trigger('click');
    expect(wrapper.findAll('.expert-card')).toHaveLength(2);
  });
  it('starts a new expert conversation with a draft for the user to complete', async () => {
    const { wrapper, store } = setup(); await flushPromises();
    await wrapper.findAll('.expert-card')[0].get('.store-primary').trigger('click'); await flushPromises();
    expect(store.chat.select).toHaveBeenCalledWith(null);
    expect(store.creationDraft).toContain('请作为深度研究专家帮助我');
    expect(store.navigate).toHaveBeenCalledWith('chat');
  });
  it('requires review before installing the exact pinned package and refreshes real state', async () => {
    const { wrapper } = setup(); await skills(wrapper);
    expect(invoke).toHaveBeenCalledWith('ai_store_browse', { keyword: '', page: 1 });
    await wrapper.get('.skill-card .small-action').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_store_inspect', { slug: 'demo-skill', version: '1.2.3' });
    expect(wrapper.get('.test-modal').text()).toContain('scripts/run.py');
    expect(wrapper.get('.script-preview pre').text()).toBe('print("hello")');
    expect(wrapper.get('.test-modal .store-primary').attributes('disabled')).toBeDefined();
    await wrapper.get('.review-confirm input').setValue(true);
    await wrapper.get('.test-modal .store-primary').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_store_install', { slug: 'demo-skill', version: '1.2.3', sha256: manifestHash });
    expect(wrapper.get('.skill-card .small-action').text()).toContain('已安装');
  });
  it('keeps installation unavailable when inspection fails and supports retry', async () => {
    const { wrapper } = setup(); await skills(wrapper);
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementationOnce(async () => { throw new Error('offline'); });
    await wrapper.get('.skill-card .small-action').trigger('click'); await flushPromises();
    expect(wrapper.get('.test-modal [role="alert"]').text()).toContain('offline');
    expect(wrapper.get('.test-modal .store-primary').attributes('disabled')).toBeDefined();
    vi.mocked(invoke).mockImplementation(original);
    await wrapper.get('.test-modal .text-action').trigger('click'); await flushPromises();
    expect(wrapper.find('.review-confirm').exists()).toBe(true);
  });
  it('does not replace newer search results with a late response', async () => {
    const { wrapper } = setup(); await skills(wrapper);
    let resolveOld!: (v: unknown) => void;
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation((command, args) => command === 'ai_store_browse' && args?.keyword === 'old'
      ? new Promise(resolve => { resolveOld = resolve; }) : original(command, args));
    await wrapper.get('input').setValue('old'); await settle();
    await wrapper.get('input').setValue('new'); await settle();
    resolveOld({ results: [{ ...remoteSkill, name: '过期结果' }], page: 1, total: 1, hasMore: false }); await flushPromises();
    expect(wrapper.text()).not.toContain('过期结果');
    expect(wrapper.get('.skill-card').text()).toContain('网页总结');
  });
  it('requires a second click to uninstall and preserves bundled skills', async () => {
    local = [{ ...remoteSkill, removable: true, source: 'SkillHub' }, { slug: 'builtin', name: '内置技能', description: '', version: '', removable: false }];
    const { wrapper } = setup(); await skills(wrapper);
    await wrapper.get('.installed-filter').trigger('click'); await flushPromises();
    await wrapper.findAll('.skill-card')[1].get('.small-action').trigger('click');
    expect(wrapper.find('.test-modal .store-secondary').exists()).toBe(false);
    await wrapper.findAll('.skill-card')[0].get('.small-action').trigger('click');
    await wrapper.get('.test-modal .store-secondary').trigger('click');
    expect(invoke).not.toHaveBeenCalledWith('ai_store_uninstall', expect.anything());
    await wrapper.get('.test-modal .store-secondary').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_store_uninstall', { slug: 'demo-skill' });
  });
  it('distinguishes planned connectors from the working computer-control preference', async () => {
    const { wrapper } = setup(); await flushPromises();
    await wrapper.get('#store-tab-connectors').trigger('click');
    expect(wrapper.findAll('.connector-card')).toHaveLength(9);
    await wrapper.findAll('.connector-card')[1].get('.small-action').trigger('click');
    expect(wrapper.get('.test-modal').text()).toContain('此连接器尚未接入');
    expect(wrapper.get('.test-modal .store-primary').attributes('disabled')).toBeDefined();
    await wrapper.get('.test-modal .store-secondary').trigger('click');
    await wrapper.get('.connector-card .small-action').trigger('click');
    await wrapper.get('.test-modal .store-primary').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('save_ai_preferences', { openComputerUse: true });
    expect(wrapper.get('.connection-status').text()).toBe('已启用');
  });
  it('supports keyboard tab navigation', async () => {
    const { wrapper } = setup(); await flushPromises();
    await wrapper.get('[role="tablist"]').trigger('keydown', { key: 'End' });
    expect(wrapper.get('#store-tab-connectors').attributes('aria-selected')).toBe('true');
    expect(wrapper.get('#store-tab-connectors').attributes('tabindex')).toBe('0');
    expect(wrapper.get('[role="tabpanel"]').attributes('aria-labelledby')).toBe('store-tab-connectors');
  });
});
