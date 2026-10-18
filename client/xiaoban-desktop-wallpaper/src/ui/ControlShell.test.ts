// @vitest-environment happy-dom
import { mount, flushPromises } from '@vue/test-utils';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import ControlShell from './ControlShell.vue';
import { invoke, listen } from '../bridge.ts';
import { NModal } from 'naive-ui';
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

  it('uses the brand icon to open login without a separate account card', async () => {
    const wrapper = mount(ControlShell, { attachTo: document.body });
    await flushPromises();
    const entry = wrapper.get('[data-testid="account-entry"]');
    expect(entry.attributes('aria-label')).toBe('登录 / 注册小伴账号');
    expect(wrapper.find('.sidebar-account-box').exists()).toBe(false);
    expect(wrapper.find('[data-testid="sidebar-login-btn"]').exists()).toBe(false);
    await entry.trigger('click');
    await flushPromises();
    expect(wrapper.findComponent(LoginModal).props('store').showLoginModal).toBe(true);
    await wrapper.get('[aria-label="展开导航"]').trigger('click');
    expect(wrapper.get('.brand-copy').text()).toContain('小伴桌面');
    expect(wrapper.text()).not.toContain('你的桌面生活空间');
    wrapper.unmount();
  });

  it('shows account details and signs out through the modal', async () => {
    const wrapper = mount(ControlShell, { attachTo: document.body });
    await flushPromises();
    const store = wrapper.findComponent(LoginModal).props('store');
    store.account = { accessToken: 'secret-access', refreshToken: 'secret-refresh', expiresAt: 0,
      user: { uid: 'test-account-id', username: '用户名'.repeat(20), avatarUrl: null } };
    await flushPromises();
    expect(wrapper.get('[data-testid="account-entry"]').text()).toBe('用');
    await wrapper.get('[data-testid="account-entry"]').trigger('click');
    await flushPromises();
    expect(document.querySelector('[data-testid="account-username"]')?.textContent).toBe(store.account.user.username);
    expect(document.querySelector('[data-testid="account-uid"]')?.textContent).toBe('test-account-id');
    expect(document.body.textContent).not.toContain('secret-access');
    expect(document.body.textContent).not.toContain('secret-refresh');
    const logout = vi.spyOn(store, 'logoutAccount').mockImplementation(async () => { store.account = null; });
    (document.querySelector('[data-testid="account-logout"]') as HTMLButtonElement).click();
    await flushPromises();
    expect(logout).toHaveBeenCalledTimes(1);
    expect(wrapper.findComponent(NModal).props('show')).toBe(false);
    expect(wrapper.get('[data-testid="account-entry"]').attributes('aria-label')).toBe('登录 / 注册小伴账号');
    wrapper.unmount();
  });

  it('keeps details open on logout failure, prevents duplicate requests, and closes on external sign-out', async () => {
    const wrapper = mount(ControlShell, { attachTo: document.body });
    await flushPromises();
    const store = wrapper.findComponent(LoginModal).props('store');
    store.account = { accessToken: 'a', refreshToken: 'r', expiresAt: 0,
      user: { uid: 'u', username: 'Alice', avatarUrl: 'https://example.com/avatar.png' } };
    await flushPromises();
    expect(wrapper.get('.brand-avatar').attributes('src')).toBe('https://example.com/avatar.png');
    await wrapper.get('[data-testid="account-entry"]').trigger('click');
    await flushPromises();
    let reject!: (reason: Error) => void;
    const logout = vi.spyOn(store, 'logoutAccount').mockImplementation(() => new Promise<void>((_, fail) => { reject = fail; }));
    const button = document.querySelector('[data-testid="account-logout"]') as HTMLButtonElement;
    button.click(); button.click();
    await flushPromises();
    expect(logout).toHaveBeenCalledTimes(1);
    expect(button.disabled).toBe(true);
    reject(new Error('network error'));
    await flushPromises();
    expect(wrapper.findComponent(NModal).props('show')).toBe(true);
    expect(button.disabled).toBe(false);
    const callback = vi.mocked(listen).mock.calls.find(([name]) => name === 'account-changed')?.[1];
    expect(callback).toBeDefined();
    callback!(null as never);
    await flushPromises();
    expect(wrapper.findComponent(NModal).props('show')).toBe(false);
    wrapper.unmount();
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
