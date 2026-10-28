// @vitest-environment happy-dom
import { mount, flushPromises } from '@vue/test-utils';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import ControlShell from './ControlShell.vue';
import { invoke, listen } from '../bridge.ts';
import LoginModal from './components/LoginModal.vue';

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
      if (command === 'ai_store_installed') return Promise.resolve([]);
      if (command === 'get_ai_config') return Promise.resolve({ openComputerUse: false });
      if (command === 'ai_store_browse') return Promise.resolve({ results: [], page: 1, hasMore: false, total: 0 });
      if (command === 'ai_store_catalog') return Promise.resolve({ results: [], page: 1, hasMore: false, total: 0 });
      if (command === 'ai_store_skill_categories') return Promise.resolve({ results: [] });
      if (command === 'ai_market_browse') return Promise.resolve({ results: [], page: 1, hasMore: false, total: 0 });
      if (command === 'ai_market_categories') return Promise.resolve({ results: [] });
      if (command === 'ai_market_updates') return Promise.resolve({ results: [] });
      if (command === 'ai_market_take_link') return Promise.resolve(null);
      if (command === 'ai_store_experts_installed' || command === 'ai_store_connector_status') return Promise.resolve([]);
      if (command === 'assistant_status') return Promise.resolve({enabled:true,shortcut:'default',shortcutLabel:'⌥ Space',shortcuts:[],ball:true,open:'type',read:'never',hintSeen:true,registered:true,error:''});
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
  afterEach(() => { document.body.innerHTML = ''; });
  const storeOf = (wrapper: ReturnType<typeof mount>) => wrapper.findComponent(LoginModal).props('store');

  it('lists 小伴 AI (new task, AI plugins, library) above 装扮, 游玩 and 提醒, and keeps the AI store tab', async () => {
    const wrapper = mount(ControlShell, { attachTo: document.body });
    try {
      await flushPromises();
      expect(wrapper.findAll('#control-navigation .nav-item[data-testid^="nav-"]').map(b => b.attributes('data-testid')))
        .toEqual(['nav-ai-group', 'nav-new-task', 'nav-ai-store', 'nav-library', 'nav-studio', 'nav-games', 'nav-care']);
      expect(wrapper.find('.control-statusbar').exists()).toBe(false);
      expect(wrapper.find('.sidebar-desktop-state').exists()).toBe(false);
      expect(wrapper.find('[data-testid="assistant-launcher"]').exists()).toBe(false);
      await wrapper.get('[data-testid="nav-ai-store"]').trigger('click');
      await vi.waitFor(async () => { await flushPromises(); expect(wrapper.find('[data-testid="ai-store-page"]').exists()).toBe(true); }, { timeout: 15000 });
      expect(wrapper.get('.control-workspace').classes()).toContain('fixed-page-workspace');
      await wrapper.get('#store-tab-connectors').trigger('click');
      await wrapper.get('[data-testid="nav-studio"]').trigger('click');
      await wrapper.get('[data-testid="nav-ai-store"]').trigger('click'); await flushPromises();
      expect(wrapper.get('#store-tab-connectors').attributes('aria-selected')).toBe('true');
    } finally { wrapper.unmount(); }
  });

  it('reuses a page when returning through the sidebar', async () => {
    const wrapper = mount(ControlShell, { attachTo: document.body });
    try {
      await flushPromises();
      await wrapper.get('[data-testid="nav-games"]').trigger('click');
      await vi.waitFor(async () => { await flushPromises(); expect(wrapper.find('[data-testid="games-page"]').exists()).toBe(true); });
      const first = wrapper.get('[data-testid="games-page"]').element;
      await wrapper.get('[data-testid="nav-library"]').trigger('click');
      await vi.waitFor(async () => { await flushPromises(); expect(wrapper.find('[data-testid="library-page"]').exists()).toBe(true); });
      await wrapper.get('[data-testid="nav-games"]').trigger('click');
      await vi.waitFor(async () => { await flushPromises(); expect(wrapper.find('[data-testid="games-page"]').exists()).toBe(true); });
      expect(wrapper.get('[data-testid="games-page"]').element).toBe(first);
    } finally { wrapper.unmount(); }
  });

  it('opens 装扮 on the tab an old page id names', async () => {
    const wrapper = mount(ControlShell, { attachTo: document.body, global: { stubs: {
      WallpaperPage: { name: 'WallpaperPage', template: '<div data-testid="wallpapers-stub" />' },
      CompanionPage: { name: 'CompanionPage', props: ['settingsRequest'], template: '<div data-testid="companion-stub" />' },
    } } });
    try {
      await flushPromises();
      const store = storeOf(wrapper);
      store.navigate('companion');
      await vi.waitFor(async () => { await flushPromises(); expect(wrapper.find('[data-testid="companion-stub"]').exists()).toBe(true); });
      expect(wrapper.get('[data-testid="nav-studio"]').attributes('aria-current')).toBe('page');
      expect(wrapper.get('[data-testid="studio-tab-pets"]').attributes('aria-selected')).toBe('true');
      await wrapper.get('[data-testid="studio-tab-wallpapers"]').trigger('click');
      await vi.waitFor(async () => { await flushPromises(); expect(wrapper.find('[data-testid="wallpapers-stub"]').exists()).toBe(true); });
      expect(store.studioTab).toBe('wallpapers');
    } finally { wrapper.unmount(); }
  });

  it('groups conversations by working folder under 空间 and opens one in the chat page', async () => {
    const wrapper = mount(ControlShell, { attachTo: document.body, global: { stubs: { ChatPage: { name: 'ChatPage', template: '<div data-testid="chat-stub" />' } } } });
    try {
      await flushPromises();
      const store = storeOf(wrapper);
      store.chat.sessions = [
        { id: 'a', title: '没有文件夹', createdAt: 1, updatedAt: 1 },
        { id: 'b', title: '改首页', createdAt: 1, updatedAt: 1, workspace: '/Users/me/site' },
        { id: 'c', title: '改导航', createdAt: 1, updatedAt: 1, workspace: 'C:\\work\\site2\\' },
      ];
      store.chat.views.b = { approvals: [{}], questions: [], items: [] } as never;
      await flushPromises();
      expect(wrapper.get('[data-testid="space-recent"]').text()).toContain('最近任务');
      expect(wrapper.find('[data-testid="session-a"]').exists()).toBe(true);
      expect(wrapper.find('[data-testid="session-b"]').exists()).toBe(false);
      const site = wrapper.get('[data-testid="space-site"]');
      expect(site.find('.xb-dot').exists()).toBe(true);
      expect(wrapper.find('[data-testid="space-site2"]').exists()).toBe(true);
      await site.trigger('click');
      expect(wrapper.get('[data-testid="session-b"]').find('.xb-dot').exists()).toBe(true);
      const select = vi.spyOn(store.chat, 'select').mockResolvedValue();
      await wrapper.get('[data-testid="session-b"]').trigger('click');
      await flushPromises();
      expect(store.page).toBe('chat');
      expect(select).toHaveBeenCalledWith('b');
      // The 小伴 AI group shows a dot while folded when something waits for the user.
      Object.defineProperty(store.chat, 'attention', { get: () => 1, configurable: true });
      await wrapper.get('[data-testid="nav-ai-group"]').trigger('click');
      expect(wrapper.find('[data-testid="chat-attention"]').exists()).toBe(true);
    } finally { wrapper.unmount(); }
  });

  it('renames, moves and deletes a conversation from its menu in the space tree', async () => {
    const wrapper = mount(ControlShell, { attachTo: document.body, global: { stubs: { ChatPage: { name: 'ChatPage', template: '<div data-testid="chat-stub" />' }, NewTaskPage: { name: 'NewTaskPage', template: '<div />' } } } });
    try {
      await flushPromises();
      const store = storeOf(wrapper);
      store.chat.sessions = [
        { id: 'a', title: '周计划', createdAt: 1, updatedAt: 1 },
        { id: 'b', title: '改首页', createdAt: 1, updatedAt: 1, workspace: '/Users/me/site' },
      ];
      store.chat.groups = [{ id: 'g1', name: '旧分组' }] as never;
      await flushPromises();
      expect(wrapper.text()).not.toContain('旧分组'); // groups are kept but no longer shown
      // Right click opens the same menu as the hover ⋯ button.
      await wrapper.get('[data-testid="session-a"]').element.parentElement!.dispatchEvent(new MouseEvent('contextmenu', { bubbles: true, clientX: 40, clientY: 300 }));
      await flushPromises();
      expect(wrapper.find('[data-testid="session-menu"]').exists()).toBe(true);
      await wrapper.get('[data-testid="session-menu-rename"]').trigger('click');
      await flushPromises();
      expect(wrapper.find('[data-testid="session-menu"]').exists()).toBe(false);
      const rename = vi.spyOn(store.chat, 'rename').mockResolvedValue(true);
      const input = document.querySelector('[data-testid="chat-name-input"] input') as HTMLInputElement;
      input.value = '下周计划'; input.dispatchEvent(new Event('input'));
      (document.querySelector('[data-testid="chat-name-ok"]') as HTMLButtonElement).click();
      await flushPromises();
      expect(rename).toHaveBeenCalledWith('a', '下周计划');

      const move = vi.spyOn(store.chat, 'moveToWorkspace').mockImplementation(async (id: string, path: string) => { store.chat.sessions = store.chat.sessions.map(s => s.id === id ? { ...s, workspace: path } : s); return true; });
      await wrapper.get('[data-testid="session-more-a"]').trigger('click');
      await wrapper.get('[data-testid="session-move-site"]').trigger('click');
      await flushPromises();
      expect(move).toHaveBeenCalledWith('a', '/Users/me/site');
      expect(wrapper.find('[data-testid="session-a"]').exists()).toBe(true); // its folder is unfolded
      vi.spyOn(store.chat, 'pickWorkspace').mockResolvedValue('/Users/me/新项目');
      await wrapper.get('[data-testid="session-more-b"]').trigger('click');
      await wrapper.get('[data-testid="session-move-pick"]').trigger('click');
      await flushPromises();
      expect(move).toHaveBeenLastCalledWith('b', '/Users/me/新项目');

      const remove = vi.spyOn(store.chat, 'remove').mockImplementation(async (id: string) => { store.chat.sessions = store.chat.sessions.filter(s => s.id !== id); });
      await wrapper.get('[data-testid="session-more-a"]').trigger('click');
      await wrapper.get('[data-testid="session-menu-delete"]').trigger('click');
      expect(remove).not.toHaveBeenCalled(); // asks first
      expect(wrapper.get('[data-testid="session-delete-confirm"]').text()).toContain('周计划');
      await wrapper.get('[data-testid="session-delete-yes"]').trigger('click');
      await flushPromises();
      expect(remove).toHaveBeenCalledWith('a');
      expect(store.chat.groups).toHaveLength(1);
    } finally { wrapper.unmount(); }
  });

  it('starts a new task in a picked folder from the 空间 plus button', async () => {
    const wrapper = mount(ControlShell, { attachTo: document.body, global: { stubs: { NewTaskPage: { name: 'NewTaskPage', template: '<div data-testid="new-task-stub" />' } } } });
    try {
      await flushPromises();
      const store = storeOf(wrapper);
      store.navigate('games');
      vi.spyOn(store.chat, 'pickWorkspace').mockResolvedValue('/Users/me/新项目');
      const switchWorkspace = vi.spyOn(store.chat, 'switchWorkspace').mockImplementation(async (path: string) => { store.chat.pendingWorkspace = path; return 'done'; });
      await wrapper.get('[data-testid="nav-new-workspace"]').trigger('click');
      await flushPromises();
      expect(switchWorkspace).toHaveBeenCalledWith('/Users/me/新项目');
      expect(store.page).toBe('new-task');
      expect(wrapper.find('[data-testid="space-新项目"]').exists()).toBe(true);
    } finally { wrapper.unmount(); }
  });

  it('opens settings as a dialog from the account menu, and old settings links open the right category', async () => {
    const wrapper = mount(ControlShell, { attachTo: document.body });
    try {
      await flushPromises();
      const store = storeOf(wrapper);
      store.navigate('games');
      await wrapper.get('[data-testid="account-entry"]').trigger('click');
      await wrapper.get('[data-testid="nav-settings"]').trigger('click');
      await vi.waitFor(async () => { await flushPromises(); expect(document.querySelector('[data-testid="settings-page"]')).not.toBeNull(); });
      expect(store.page).toBe('games');
      expect(wrapper.find('[data-testid="account-menu"]').exists()).toBe(false);
      const labels = [...document.querySelectorAll('.xb-settings-tab')].map(tab => tab.textContent?.trim());
      expect(labels).toEqual(['通用', '外观', 'AI 模型', '超级小伴', '声音', '连接与网络', '安全与权限', '关于与更新']);
      // Native settings links switch the category while the dialog is open.
      const handler = vi.mocked(listen).mock.calls.find(([name]) => name === 'assistant-open-settings')![1];
      handler({ tab: 'assistant' });
      await vi.waitFor(async () => { await flushPromises(); expect(document.querySelector('[data-testid="assistant-page"]')).not.toBeNull(); });
      expect(document.querySelector('[data-testid="assistant-page"]')!.classList).toContain('is-embedded');
      expect(store.settingsTab).toBe('assistant');
      (document.querySelector('[data-testid="settings-close"]') as HTMLButtonElement).click();
      await flushPromises();
      expect(store.settingsOpen).toBe(false);
      store.navigate('settings');
      expect(store.settingsOpen).toBe(true);
      expect(store.page).toBe('games');
    } finally { wrapper.unmount(); }
  });

  it('opens the page the assistant asks for: a sidebar page, a 装扮 tab, or a settings category', async () => {
    const wrapper = mount(ControlShell, { attachTo: document.body });
    try {
      await flushPromises();
      const store = storeOf(wrapper);
      const open = vi.mocked(listen).mock.calls.find(([name]) => name === 'control-navigate')![1];
      open({ page: 'studio', tab: 'pets' });
      expect(store.page).toBe('studio');
      expect(store.studioTab).toBe('pets');
      open({ page: 'care', tab: null });
      expect(store.page).toBe('care');
      open({ page: 'settings', tab: 'network' });
      expect(store.settingsOpen).toBe(true);
      expect(store.settingsTab).toBe('network');
      open({ page: 'games' });
      expect(store.settingsOpen).toBe(false);
      expect(store.page).toBe('games');
    } finally { wrapper.unmount(); }
  });

  it('switches the theme colour from the account menu', async () => {
    const wrapper = mount(ControlShell, { attachTo: document.body });
    try {
      await flushPromises();
      await wrapper.get('[data-testid="account-entry"]').trigger('click');
      await wrapper.get('[data-testid="accent-blue"]').trigger('click');
      const { accent } = await import('./appearance.ts');
      expect(accent.value).toBe('blue');
      expect(wrapper.get('[data-testid="accent-blue"]').attributes('aria-checked')).toBe('true');
      expect(wrapper.find('[data-testid="account-menu"]').exists()).toBe(true);
      accent.value = 'red';
    } finally { wrapper.unmount(); }
  });

  it('opens the floating assistant from the account menu', async () => {
    const wrapper = mount(ControlShell, { attachTo: document.body });
    try {
      await flushPromises();
      expect(wrapper.find('[data-testid="nav-assistant"]').exists()).toBe(false);
      await wrapper.get('[data-testid="account-entry"]').trigger('click');
      await wrapper.get('[data-testid="open-assistant"]').trigger('click');
      await flushPromises();
      expect(invoke).toHaveBeenCalledWith('assistant_try');
    } finally { wrapper.unmount(); }
  });

  it('offers login at the top of the account menu when signed out, and folds the sidebar', async () => {
    const wrapper = mount(ControlShell, { attachTo: document.body });
    try {
      await flushPromises();
      const entry = wrapper.get('[data-testid="account-entry"]');
      expect(entry.attributes('aria-label')).toBe('账号菜单（未登录）');
      expect(entry.text()).toBe('未登录');
      await entry.trigger('click');
      expect(wrapper.find('[data-testid="account-logout"]').exists()).toBe(false);
      await wrapper.get('[data-testid="account-login"]').trigger('click');
      await flushPromises();
      expect(storeOf(wrapper).showLoginModal).toBe(true);
      expect(wrapper.find('[data-testid="account-menu"]').exists()).toBe(false);
      expect(wrapper.classes()).not.toContain('is-collapsed');
      expect(wrapper.get('[aria-label="收起侧栏"]').attributes('aria-expanded')).toBe('true');
      await wrapper.get('[aria-label="收起侧栏"]').trigger('click');
      expect(wrapper.classes()).toContain('is-collapsed');
      await wrapper.get('[aria-label="展开侧栏"]').trigger('click');
      expect(wrapper.classes()).not.toContain('is-collapsed');
      expect(wrapper.get('.brand-copy').text()).toContain('小伴桌面');
    } finally { wrapper.unmount(); }
  });

  it('shows account details in the menu and signs out', async () => {
    const wrapper = mount(ControlShell, { attachTo: document.body });
    try {
      await flushPromises();
      const store = storeOf(wrapper);
      store.account = { accessToken: 'secret-access', refreshToken: 'secret-refresh', expiresAt: 0,
        user: { uid: 'test-account-id', username: '用户名'.repeat(20), avatarUrl: null } };
      await flushPromises();
      expect(wrapper.get('[data-testid="account-entry"]').text()).toBe(store.account.user.username);
      await wrapper.get('[data-testid="account-entry"]').trigger('click');
      expect(wrapper.get('[data-testid="account-username"]').text()).toBe(store.account.user.username);
      expect(wrapper.get('[data-testid="account-uid"]').text()).toBe('test-account-id');
      expect(document.body.textContent).not.toContain('secret-access');
      expect(document.body.textContent).not.toContain('secret-refresh');
      const logout = vi.spyOn(store, 'logoutAccount').mockImplementation(async () => { store.account = null; });
      await wrapper.get('[data-testid="account-logout"]').trigger('click');
      await flushPromises();
      expect(logout).toHaveBeenCalledTimes(1);
      expect(wrapper.find('[data-testid="account-menu"]').exists()).toBe(false);
      expect(wrapper.get('[data-testid="account-entry"]').attributes('aria-label')).toBe('账号菜单（未登录）');
    } finally { wrapper.unmount(); }
  });

  it('keeps the menu open on logout failure and prevents duplicate requests', async () => {
    const wrapper = mount(ControlShell, { attachTo: document.body });
    try {
      await flushPromises();
      const store = storeOf(wrapper);
      store.account = { accessToken: 'a', refreshToken: 'r', expiresAt: 0,
        user: { uid: 'u', username: 'Alice', avatarUrl: 'https://example.com/avatar.png' } };
      await flushPromises();
      expect(wrapper.get('.brand-avatar').attributes('src')).toBe('https://example.com/avatar.png');
      await wrapper.get('[data-testid="account-entry"]').trigger('click');
      let reject!: (reason: Error) => void;
      const logout = vi.spyOn(store, 'logoutAccount').mockImplementation(() => new Promise<void>((_, fail) => { reject = fail; }));
      const button = wrapper.get('[data-testid="account-logout"]').element as HTMLButtonElement;
      button.click(); button.click();
      await flushPromises();
      expect(logout).toHaveBeenCalledTimes(1);
      expect(button.disabled).toBe(true);
      reject(new Error('network error'));
      await flushPromises();
      expect(wrapper.find('[data-testid="account-menu"]').exists()).toBe(true);
      expect((wrapper.get('[data-testid="account-logout"]').element as HTMLButtonElement).disabled).toBe(false);
      const callback = vi.mocked(listen).mock.calls.find(([name]) => name === 'account-changed')?.[1];
      expect(callback).toBeDefined();
      callback!(null as never);
      await flushPromises();
      expect(wrapper.find('[data-testid="account-logout"]').exists()).toBe(false);
      expect(wrapper.find('[data-testid="account-login"]').exists()).toBe(true);
    } finally { wrapper.unmount(); }
  });

  it('routes widget settings requests to the studio page and consumes them after opening', async () => {
    const wrapper = mount(ControlShell, { global: { stubs: { WidgetsPage: { name: 'WidgetsPage', props: ['settingsRequest'], emits: ['settingsOpened'], template: '<div data-testid="widget-page-stub" />' } } } });
    try {
      await flushPromises();
      window.dispatchEvent(new CustomEvent('widgets-manage', { detail: { id: 'notetwo' } }));
      await vi.waitFor(async () => { await flushPromises(); expect(wrapper.find('[data-testid="widget-page-stub"]').exists()).toBe(true); });
      const page = wrapper.findComponent({ name: 'WidgetsPage' });
      expect(page.props('settingsRequest')).toEqual({ id: 'notetwo' });
      page.vm.$emit('settingsOpened'); await flushPromises();
      expect(page.props('settingsRequest')).toBeNull();
      window.dispatchEvent(new CustomEvent('widgets-manage', { detail: { id: 'notetwo' } })); await flushPromises();
      expect(page.props('settingsRequest')).toEqual({ id: 'notetwo' });
    } finally { wrapper.unmount(); }
  });

  it('opens the official website from the account menu', async () => {
    const wrapper = mount(ControlShell);
    try {
      await flushPromises();
      await wrapper.get('[data-testid="account-entry"]').trigger('click');
      const items = wrapper.findAll('[data-testid="account-menu"] [role="menuitem"]').map(item => item.attributes('data-testid'));
      expect(items.indexOf('nav-settings')).toBeLessThan(items.indexOf('nav-website'));
      expect(wrapper.get('[data-testid="nav-website"]').text()).toContain('小伴官网');
      await wrapper.get('[data-testid="nav-website"]').trigger('click');
      await flushPromises();
      expect(invoke).toHaveBeenCalledWith('open_official_site');
    } finally { wrapper.unmount(); }
  });

  it('marks a new version in the account menu and the about category, and shows only a dot for unread notices', async () => {
    const base = vi.mocked(invoke).getMockImplementation()!;
    const website = (update: unknown, announcements: unknown[] = []) => vi.mocked(invoke).mockImplementation(((command: string, ...rest: unknown[]) =>
      command === 'get_website_status' ? Promise.resolve({ announcements, update, dismissed: {}, errors: [] }) : base(command, ...(rest as []))) as typeof invoke);

    website({ available: true, version: '0.2.0', notes: '', publishedAt: '', downloadUrl: 'https://example.com' });
    const wrapper = mount(ControlShell, { attachTo: document.body });
    await flushPromises();
    await wrapper.get('[data-testid="account-entry"]').trigger('click');
    expect(wrapper.get('[data-testid="nav-settings-new"]').text()).toBe('新版本');
    expect(wrapper.find('[data-testid="nav-notice-dot"]').exists()).toBe(false);
    expect(wrapper.find('[data-testid="nav-notices"] .xb-dot').exists()).toBe(true);
    await wrapper.get('[data-testid="nav-check-update"]').trigger('click');
    await vi.waitFor(async () => { await flushPromises(); expect(document.querySelector('[data-testid="settings-about"]')).not.toBeNull(); });
    expect(document.querySelector('[data-testid="about-tab-new"]')?.textContent).toBe('新');
    wrapper.unmount();
    document.body.innerHTML = '';

    website(null, [{ id: 'n1', revision: '1', title: '公告', body: '' }]);
    const quiet = mount(ControlShell);
    await flushPromises();
    await quiet.get('[data-testid="account-entry"]').trigger('click');
    expect(quiet.find('[data-testid="nav-settings-new"]').exists()).toBe(false);
    expect(quiet.find('[data-testid="nav-notice-dot"]').exists()).toBe(true);
    quiet.unmount();
  });
});
