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
    expect(settingsBtn.text()).toContain('设置与关于');

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
});
