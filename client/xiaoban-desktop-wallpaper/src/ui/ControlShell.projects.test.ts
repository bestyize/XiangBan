// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import { reactive } from 'vue';
import ControlShell from './ControlShell.vue';

const fixture = vi.hoisted(() => ({ store: null as any }));
vi.mock('./control-store.ts', () => ({ createControlStore: () => fixture.store }));
vi.mock('../bridge.ts', () => ({ invoke: vi.fn().mockResolvedValue(null), listen: vi.fn().mockResolvedValue(() => {}) }));
vi.mock('./components/SettingsDialog.vue', () => ({ __esModule: true, default: { template: '<div />' } }));
vi.mock('./components/ChatNameDialog.vue', () => ({ __esModule: true, default: { template: '<div />' } }));
vi.mock('./components/LoginModal.vue', () => ({ __esModule: true, default: { template: '<div />' } }));
vi.mock('./community/CommunityApplyDialog.vue', () => ({ __esModule: true, default: { template: '<div />' } }));
vi.mock('./components/WorkspaceSearchDialog.vue', () => ({ default: { props: ['show', 'spaces'], emits: ['select', 'update:show'], template: '<div v-if="show" data-testid="workspace-search-dialog" />' } }));
vi.mock('naive-ui', () => ({ NAlert: { template: '<div />' }, NButton: { template: '<button />' }, NSkeleton: { template: '<div />' }, useMessage: () => ({ success: vi.fn(), error: vi.fn(), info: vi.fn() }) }));

let wrapper: VueWrapper | undefined;
const projectPaths = () => wrapper!.findAll('[data-workspace-path]').map(row => row.attributes('data-workspace-path'));
const button = (name: string) => wrapper!.get(`[data-testid="${name}"]`);
function geometry() {
  const rect = (top: number, bottom: number) => ({ left: 0, right: 260, top, bottom, height: bottom - top, width: 260, x: 0, y: top, toJSON() {} });
  vi.spyOn(wrapper!.get('.sidebar-scroll').element, 'getBoundingClientRect').mockReturnValue(rect(0, 500));
  wrapper!.findAll('[data-workspace-path]').forEach((row, i) => {
    vi.spyOn(row.element, 'getBoundingClientRect').mockReturnValue(rect(100 + i * 40, 132 + i * 40));
  });
}
async function drag(source: string, y: number, finish = true) {
  await button(`space-${source}`).trigger('pointerdown', { button: 0, isPrimary: true, pointerId: 1, clientX: 60, clientY: 110 + projectPaths().indexOf(`/${source}`) * 40 });
  window.dispatchEvent(new PointerEvent('pointermove', { pointerId: 1, clientX: 60, clientY: y, cancelable: true }));
  if (finish) window.dispatchEvent(new PointerEvent('pointerup', { pointerId: 1, clientX: 60, clientY: y }));
  await flushPromises();
}
beforeEach(() => {
  localStorage.clear();
  fixture.store = reactive({
    info: { platform: 'preview', version: 'test' }, desktop: false, ready: false, initialError: '', account: null,
    website: { announcements: [], dismissed: {}, update: null }, settings: {},
    page: 'chat', notify: vi.fn(), initialize: vi.fn().mockResolvedValue(undefined), dispose: vi.fn(),
    refreshSettings: vi.fn().mockResolvedValue(undefined), openSettings: vi.fn(),
    navigate: vi.fn((page: string) => { fixture.store.page = page; }),
    chat: {
      workspace: '/default', pendingWorkspace: '', currentId: 'a', attention: 0,
      sessions: ['A', 'B', 'C'].map(name => ({ id: name.toLowerCase(), title: name, workspace: `/${name}` })),
      views: { a: { running: true, approvals: [], questions: [] } },
      select: vi.fn(async (id: string | null) => { fixture.store.chat.currentId = id; }),
      switchWorkspace: vi.fn(async (path: string) => { fixture.store.chat.pendingWorkspace = path; return 'done'; }),
    },
  });
});
afterEach(() => { wrapper?.unmount(); wrapper = undefined; vi.restoreAllMocks(); });
function render() { wrapper = mount(ControlShell, { attachTo: document.body }); return wrapper; }

describe('workspace project actions', () => {
  it('limits projects and sessions to six and reveals a hidden search result', async () => {
    fixture.store.chat.sessions = Array.from({ length: 8 }, (_, project) => Array.from({ length: 8 }, (_, row) => ({ id: `p${project}s${row}`, title: `会话 ${project}-${row}`, workspace: `/项目${project}` }))).flat();
    fixture.store.chat.currentId = null;
    fixture.store.chat.views = { p0s7: { approvals: [{}], questions: [] }, p7s7: { approvals: [{}], questions: [] } };
    render();
    expect(projectPaths()).toHaveLength(6);
    expect(button('workspace-projects-more').find('.xb-dot').exists()).toBe(true);
    await button('space-项目0').trigger('click');
    expect(wrapper!.findAll('[data-workspace-path="/项目0"] .nav-session')).toHaveLength(6);
    expect(button('sessions-more-/项目0').find('.xb-dot').exists()).toBe(true);
    await button('sessions-more-/项目0').trigger('click');
    expect(wrapper!.findAll('[data-workspace-path="/项目0"] .nav-session')).toHaveLength(8);
    await button('sessions-more-/项目0').trigger('click');
    expect(wrapper!.findAll('[data-workspace-path="/项目0"] .nav-session')).toHaveLength(6);
    await button('workspace-projects-more').trigger('click'); expect(projectPaths()).toHaveLength(8);
    await button('workspace-projects-more').trigger('click'); expect(projectPaths()).toHaveLength(6);
    await button('nav-workspace-search').trigger('click');
    const dialog = wrapper!.findComponent({ name: 'WorkspaceSearchDialog' });
    // All sessions, including those outside the rendered six rows, remain searchable.
    expect(dialog.props('spaces').flatMap((space: any) => space.sessions)).toHaveLength(64);
    dialog.vm.$emit('select', 'p7s7'); await flushPromises();
    expect(projectPaths()).toHaveLength(8);
    expect(button('session-p7s7').text()).toContain('会话 7-7');
    expect(button('space-项目7').attributes('aria-expanded')).toBe('true');
    expect(fixture.store.chat.select).toHaveBeenCalledWith('p7s7');
  });
  it.each(['contextmenu', 'more'])('starts a fresh conversation in the chosen project from %s', async entry => {
    render();
    if (entry === 'contextmenu') await button('space-B').trigger('contextmenu', { clientX: 30, clientY: 150 });
    else await button('space-more-B').trigger('click');
    await button('workspace-menu-new-session').trigger('click');
    await flushPromises();
    expect(fixture.store.chat.select).toHaveBeenCalledWith(null);
    expect(fixture.store.chat.switchWorkspace).toHaveBeenCalledWith('/B');
    expect(fixture.store.chat.select.mock.invocationCallOrder[0]).toBeLessThan(fixture.store.chat.switchWorkspace.mock.invocationCallOrder[0]);
    expect(fixture.store.chat.pendingWorkspace).toBe('/B');
    expect(fixture.store.page).toBe('new-task');
    expect(fixture.store.chat.views.a.running).toBe(true);
    expect(button('space-B').attributes('aria-expanded')).toBe('true');
    expect(wrapper!.find('[data-testid="workspace-menu"]').exists()).toBe(false);
  });

  it('moves projects in both directions, persists the order and ignores the click after dragging', async () => {
    render(); geometry();
    await drag('C', 102);
    expect(projectPaths()).toEqual(['/C', '/A', '/B']);
    expect(JSON.parse(localStorage.getItem('xiaoban.workspace-order')!)).toEqual(['/C', '/A', '/B']);
    await button('space-C').trigger('click');
    expect(button('space-C').attributes('aria-expanded')).toBe('false');
    wrapper!.unmount(); wrapper = undefined;
    render(); geometry();
    expect(projectPaths()).toEqual(['/C', '/A', '/B']);
    await drag('C', 211);
    expect(projectPaths()).toEqual(['/A', '/B', '/C']);
    expect(wrapper!.findAll('.nav-space-row')[0]!.text()).toContain('最近任务');
  });

  it('cancels on Escape or a drop outside the list and preserves ordinary project clicks', async () => {
    render(); geometry();
    await drag('A', 211, false);
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
    window.dispatchEvent(new PointerEvent('pointerup', { pointerId: 1, clientX: 60, clientY: 211 }));
    await flushPromises();
    expect(projectPaths()).toEqual(['/A', '/B', '/C']);
    await drag('A', 600);
    expect(localStorage.getItem('xiaoban.workspace-order')).toBe(null);
    await button('space-B').trigger('pointerdown', { button: 0, isPrimary: true, pointerId: 1, clientX: 60, clientY: 150 });
    window.dispatchEvent(new PointerEvent('pointerup', { pointerId: 1, clientX: 60, clientY: 150 }));
    await button('space-B').trigger('click');
    expect(button('space-B').attributes('aria-expanded')).toBe('true');
  });

  it('keeps the previous order if saving fails and tolerates damaged stored preferences', async () => {
    localStorage.setItem('xiaoban.workspace-order', '{broken');
    render(); geometry();
    vi.spyOn(localStorage, 'setItem').mockImplementation(() => { throw new Error('full'); });
    await drag('A', 211);
    expect(projectPaths()).toEqual(['/A', '/B', '/C']);
    expect(fixture.store.notify).toHaveBeenCalledWith('项目排序保存失败，请重试', 'error');
  });

  it('uses the brand mark for all account placeholders while retaining a real account avatar', async () => {
    render();
    expect(button('account-entry').find('.account-avatar.is-placeholder .lucide-sparkles').exists()).toBe(true);
    await button('account-entry').trigger('click');
    expect(button('account-login').find('.lucide-sparkles').exists()).toBe(true);
    fixture.store.account = { user: { username: 'read', uid: '1' } };
    await flushPromises();
    expect(wrapper!.get('.account-head .account-avatar').classes()).toContain('is-placeholder');
    fixture.store.account.user.avatarUrl = 'https://example.com/avatar.png';
    await flushPromises();
    expect(button('account-entry').get('img').attributes('src')).toBe('https://example.com/avatar.png');
    expect(wrapper!.get('.account-head img').attributes('src')).toBe('https://example.com/avatar.png');
    expect(wrapper!.findAll('.account-avatar.is-placeholder')).toHaveLength(0);
  });
});
