// @vitest-environment happy-dom
import { mount, flushPromises, type VueWrapper } from '@vue/test-utils';
import { beforeEach, afterEach, describe, expect, it, vi } from 'vitest';
import WorkspaceSearchDialog from './WorkspaceSearchDialog.vue';
vi.mock('naive-ui', () => ({ NModal: { props: ['show'], template: '<div v-if="show"><slot /></div>' } }));
vi.mock('../../bridge.ts', () => ({ invoke: vi.fn() }));
import { invoke } from '../../bridge.ts';

const spaces = [{ key: '/项目', name: '自定义项目', sessions: [{ id: 's1', title: '文档任务', createdAt: 1, updatedAt: 2 }] }];
const hit = { sessionId: 's1', title: '文档任务', updatedAt: 2, excerpt: '核对 OCR 文档结果', matchedField: 'content' };
let wrapper: VueWrapper;
function render() {
  wrapper = mount(WorkspaceSearchDialog, { props: { show: true, desktop: true, spaces }, global: { stubs: { NModal: { props: ['show'], template: '<div v-if="show"><slot /></div>' } } } });
  return wrapper;
}
async function settle() { await vi.advanceTimersByTimeAsync(250); await flushPromises(); }
beforeEach(() => { vi.resetAllMocks(); vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'] }); vi.mocked(invoke).mockResolvedValue({ hits: [hit], skipped: 0, cancelled: false }); });
afterEach(() => { wrapper?.unmount(); vi.useRealTimers(); });

describe('workspace conversation search', () => {
  it('searches saved content with the selected scope, escapes snippets and opens the selected session', async () => {
    render(); await settle();
    await wrapper.get('[data-testid="workspace-search-input"]').setValue('OCR');
    await wrapper.get('[data-testid="workspace-search-content"]').trigger('click');
    await wrapper.get('select').setValue('/项目'); await settle();
    expect(invoke).toHaveBeenLastCalledWith('chat_search_sessions', { request: { query: 'OCR', field: 'content', sessionIds: ['s1'], labels: { s1: '自定义项目' } } });
    expect(wrapper.get('mark').text()).toBe('OCR');
    await wrapper.get('[data-testid="workspace-search-result"]').trigger('click');
    expect(wrapper.emitted('select')).toEqual([['s1']]);
    expect(wrapper.emitted('update:show')).toContainEqual([false]);
    vi.mocked(invoke).mockResolvedValue({ hits: [{ ...hit, title: '<img src=x>', excerpt: '<script>OCR</script>' }], skipped: 0, cancelled: false });
    await wrapper.get('[data-testid="workspace-search-input"]').setValue('script'); await settle();
    expect(wrapper.find('script').exists()).toBe(false); expect(wrapper.find('img').exists()).toBe(false);
    expect(wrapper.text()).toContain('<script>OCR</script>');
  });

  it('discards stale searches, supports keyboard selection and stops work when closed', async () => {
    let oldResolve!: (value: unknown) => void;
    vi.mocked(invoke).mockImplementationOnce(() => new Promise(resolve => { oldResolve = resolve; }) as never);
    render(); await settle();
    await wrapper.get('input').setValue('新关键词'); await settle();
    oldResolve({ hits: [{ ...hit, sessionId: 'old', title: '过期结果' }], skipped: 0, cancelled: false }); await flushPromises();
    expect(wrapper.text()).not.toContain('过期结果');
    await wrapper.get('input').trigger('keydown', { key: 'Enter' });
    expect(wrapper.emitted('select')).toEqual([['s1']]);
    await wrapper.get('input').setValue('不会执行'); await wrapper.setProps({ show: false });
    const count = vi.mocked(invoke).mock.calls.length; await settle();
    expect(invoke).toHaveBeenCalledTimes(count);
  });

  it('reports unreadable histories and query errors instead of presenting them as no matches', async () => {
    vi.mocked(invoke).mockResolvedValue({ hits: [], skipped: 2, cancelled: false });
    render(); await settle(); expect(wrapper.text()).toContain('2 个会话暂时无法读取');
    vi.mocked(invoke).mockRejectedValue(new Error('读取失败'));
    await wrapper.get('input').setValue('文档'); await settle();
    expect(wrapper.get('[role="alert"]').text()).toContain('读取失败');
    expect(wrapper.text()).not.toContain('没有找到相关会话');
  });
});
