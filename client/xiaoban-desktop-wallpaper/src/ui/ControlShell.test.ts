// @vitest-environment happy-dom
import { mount, flushPromises } from '@vue/test-utils';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import ControlShell from './ControlShell.vue';
import { invoke } from '../bridge.ts';

vi.mock('naive-ui', async (importOriginal) => {
  const actual = await importOriginal<typeof import('naive-ui')>();
  return {
    ...actual,
    useMessage: () => ({ success: vi.fn(), error: vi.fn(), info: vi.fn() }),
  };
});

vi.mock('../bridge.ts', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../bridge.ts')>();
  return {
    ...actual,
    desktop: true,
    invoke: vi.fn().mockImplementation((command: string) => {
      if (command === 'get_settings') return Promise.resolve(actual.defaults);
      if (command === 'get_wallpaper_library') return Promise.resolve({ items: [], source: '', activeId: '' });
      if (command === 'get_app_info') return Promise.resolve({ name: '小伴桌面', version: '0.1.0', platform: 'macos' });
      if (command === 'get_startup_status') return Promise.resolve({ available: true, error: null });
      if (command === 'get_game_status') return Promise.resolve({ desktopDoubleClick: false });
      if (command === 'get_website_status') return Promise.resolve({ announcements: [], update: null, dismissed: {}, errors: [] });
      if (command === 'open_official_site') return Promise.resolve();
      return Promise.resolve();
    }),
    listen: vi.fn().mockResolvedValue(() => {}),
  };
});

describe('ControlShell sidebar', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (window as unknown as { __TAURI__?: unknown }).__TAURI__ = {};
  });

  it('renders official website button above settings-and-about button and triggers invoke', async () => {
    const wrapper = mount(ControlShell);
    await flushPromises();

    const bottom = wrapper.find('.sidebar-bottom');
    expect(bottom.exists()).toBe(true);

    const buttons = bottom.findAll('button');
    const websiteBtn = bottom.find('[data-testid="nav-website"]');
    const settingsBtn = bottom.find('[data-testid="nav-settings"]');

    expect(websiteBtn.exists()).toBe(true);
    expect(settingsBtn.exists()).toBe(true);
    expect(websiteBtn.text()).toContain('小伴官网');
    expect(settingsBtn.text()).toContain('设置');

    // Official website button is placed before the settings button
    const elements: Element[] = buttons.map((b) => b.element);
    const websiteIndex = elements.indexOf(websiteBtn.element);
    const settingsIndex = elements.indexOf(settingsBtn.element);
    expect(websiteIndex).toBeGreaterThanOrEqual(0);
    expect(settingsIndex).toBeGreaterThanOrEqual(0);
    expect(websiteIndex).toBeLessThan(settingsIndex);

    // Clicking triggers open_official_site
    await websiteBtn.trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('open_official_site');

    wrapper.unmount();
  });

  it('marks a new version with a 新版本 badge on the settings button and the about tab, and shows only a dot for unread notices', async () => {
    const base = vi.mocked(invoke).getMockImplementation()!;
    const website = (update: unknown, announcements: unknown[] = []) => vi.mocked(invoke).mockImplementation(((command: string, ...rest: unknown[]) =>
      command === 'get_website_status' ? Promise.resolve({ announcements, update, dismissed: {}, errors: [] }) : base(command, ...(rest as []))) as typeof invoke);

    website({ available: true, version: '0.2.0', notes: '', publishedAt: '', downloadUrl: 'https://example.com' });
    const wrapper = mount(ControlShell);
    await flushPromises();
    const settingsBtn = wrapper.find('[data-testid="nav-settings"]');
    expect(settingsBtn.find('[data-testid="nav-settings-new"]').text()).toBe('新版本');
    expect(settingsBtn.find('.notice-dot').exists()).toBe(false);
    await settingsBtn.trigger('click');
    await import('./pages/SettingsPage.vue');
    await vi.waitFor(async () => { await flushPromises(); expect(wrapper.find('[data-testid="settings-page"]').exists()).toBe(true); });
    const page = wrapper.find('[data-testid="settings-page"]');
    expect(page.find('[data-testid="about-tab-new"]').text()).toBe('新版本');
    expect(page.find('.page-header').exists()).toBe(false);
    expect(page.text()).not.toContain('管理运行偏好');
    wrapper.unmount();

    website(null, [{ id: 'n1', revision: '1', title: '公告', body: '' }]);
    const quiet = mount(ControlShell);
    await flushPromises();
    expect(quiet.find('[data-testid="nav-settings-new"]').exists()).toBe(false);
    expect(quiet.find('[data-testid="nav-settings"] .notice-dot').exists()).toBe(true);
    quiet.unmount();
  });
});


it('opens configuration for the originating widget after the layout loads and handles repeat requests', async () => {
  const base = vi.mocked(invoke).getMockImplementation()!;
  let resolveLayout!: (layout: unknown) => void;
  const pending = new Promise<unknown>(resolve => { resolveLayout = resolve; });
  vi.mocked(invoke).mockImplementation(((command: string, ...rest: unknown[]) => {
    if (command === 'get_desktop_widgets') return pending;
    if (command === 'list_custom_widgets') return Promise.resolve([]);
    return base(command, ...(rest as []));
  }) as typeof invoke);
  const wrapper = mount(ControlShell);
  try {
    await flushPromises();
    await wrapper.get('[data-testid="nav-settings"]').trigger('click');
    await import('./pages/SettingsPage.vue'); await flushPromises();
    window.dispatchEvent(new CustomEvent('widgets-manage', { detail: { id: 'ef12' } }));
    await import('./pages/WidgetsPage.vue');
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith('get_desktop_widgets'));
    resolveLayout({version:1,widgets:[
      {id:'abcd',kind:'calendar',theme:'glass',scale:1,x:.2,y:.2,enabled:true,city:null,monitor:'primary'},
      {id:'ef12',kind:'weather',theme:'glass',scale:1,x:.2,y:.2,enabled:true,city:null,monitor:'second'},
    ],monitors:[
      {id:'primary',name:'主屏幕',width:1920,height:1080,primary:true},
      {id:'second',name:'副屏幕',width:1600,height:900,primary:false},
    ]});
    await vi.waitFor(async () => {
      await flushPromises();
      expect(wrapper.find('[data-testid="widgets-page"]').exists()).toBe(true);
      expect(wrapper.find('[data-testid="settings-page"]').exists()).toBe(false);
      expect(wrapper.get('[data-testid="layout-ef12"]').classes()).toContain('selected');
    });
    expect(wrapper.get('[role="tab"][aria-selected="true"]').text()).toContain('我的桌面');
    expect(wrapper.find('[data-testid="layout-abcd"]').exists()).toBe(false);
    // A later click still opens the same widget after switching back to the library.
    const library = wrapper.findAll('[role="tab"]').find(tab => tab.text() === '组件库')!;
    await library.trigger('click');
    window.dispatchEvent(new CustomEvent('widgets-manage', { detail: { id: 'ef12' } }));
    await flushPromises();
    expect(wrapper.get('[data-testid="layout-ef12"]').classes()).toContain('selected');
    // Older hosts sent an event without an id; they should open configuration too.
    window.dispatchEvent(new Event('widgets-manage')); await flushPromises();
    expect(wrapper.get('[data-testid="layout-abcd"]').classes()).toContain('selected');
  } finally { wrapper.unmount(); vi.mocked(invoke).mockImplementation(base); }
});
