// @vitest-environment happy-dom
import { mount, flushPromises } from '@vue/test-utils';
import { it, expect, vi } from 'vitest';
import { invoke } from '../../bridge.ts';
import AppsPage from './AppsPage.vue';
import type { ControlStore } from '../control-types';

vi.mock('../../bridge.ts', () => ({ invoke: vi.fn() }));

const mbti = {
  id: 'mbti',
  name: 'MBTI 性格测评',
  description: '48 道原创情境题',
  entry: 'index.html',
  cover: 'cover.svg',
  runtimeProtocol: 2,
  orientation: 'portrait' as const,
  source: 'builtin' as const,
  base: '/game-packs/mbti/'
};

const game2048 = {
  id: 'merge-2048',
  name: '2048',
  description: '合成数字',
  entry: 'index.html',
  cover: 'cover.svg',
  runtimeProtocol: 1,
  orientation: 'portrait' as const,
  source: 'builtin' as const,
  base: '/game-packs/merge-2048/'
};

const userApp = {
  id: 'pomodoro-timer',
  name: '番茄时钟',
  description: '自创极简番茄钟',
  entry: 'pages/timer.html',
  runtimeProtocol: 2,
  kind: 'app' as const,
  category: 'tool',
  orientation: 'any',
  source: 'user' as const,
  base: 'game-pack://localhost/pomodoro-timer/'
};

const stubs = {
  NAlert: { template: '<div class="alert"><slot /></div>' },
  NButton: { template: '<button><slot /></button>' },
  NSkeleton: true,
  NPopconfirm: { template: '<div><slot name="trigger" /><slot /></div>', emits: ['positive-click'] }
};

function setupApps(appsList = [mbti, game2048, userApp]) {
  const openApp = vi.fn().mockResolvedValue(undefined);
  const openGames = vi.fn().mockResolvedValue(undefined);
  const store = {
    desktop: true,
    page: 'apps',
    settings: { gameDesktopDoubleClick: false },
    game: { desktopDoubleClickSupported: true },
    openApp,
    openGames,
    notify: vi.fn(),
    startCreation: vi.fn(),
    listGames: vi.fn(async () => ({ games: appsList, errors: [], directory: null })),
    deleteUserGame: vi.fn().mockResolvedValue(undefined),
  } as unknown as ControlStore;

  const wrapper = mount(AppsPage, { props: { store }, global: { stubs } });
  return { store, openApp, openGames, wrapper };
}

it('renders App Plaza tabs, category filters and search bar when store.page is apps', async () => {
  const { wrapper, store } = setupApps();
  try {
    await flushPromises();
    expect(wrapper.find('[data-testid="apps-page"]').exists()).toBe(true);
    expect(wrapper.findAll('.xb-tabs-lg [role=tab]').map(t => t.text())).toEqual(['精选应用', '我的作品']);
    expect(wrapper.findAll('.filter-chip').map(c => c.text())).toEqual(['全部', '精选游戏', '实用工具', '心理测评']);
    expect(wrapper.find('[data-testid="apps-search-input"]').exists()).toBe(true);
    await wrapper.get('[data-testid="apps-create"]').trigger('click');
    expect(store.startCreation).toHaveBeenLastCalledWith('app');
    await wrapper.get('[data-testid="apps-filter-game"]').trigger('click');
    await wrapper.get('[data-testid="apps-create"]').trigger('click');
    expect(store.startCreation).toHaveBeenLastCalledWith('game');
  } finally {
    wrapper.unmount();
  }
});

it('switches to My Works after importing a V2 small app from the header', async () => {
  const { wrapper, store } = setupApps();
  try {
    vi.mocked(invoke).mockResolvedValueOnce({ kind: 'app', id: userApp.id, name: userApp.name });
    await flushPromises();
    await wrapper.get('[data-testid="import-resource"]').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('import_resource_pack');
    expect(store.listGames).toHaveBeenCalledTimes(2);
    expect(wrapper.get('[data-testid="user-apps"]').text()).toContain(userApp.name);
  } finally {
    wrapper.unmount();
  }
});

it('launches protocol 2 apps via store.openApp (V2 container) without affecting background', async () => {
  const { wrapper, openApp, openGames } = setupApps();
  try {
    await flushPromises();
    const mbtiCard = wrapper.get('[data-testid="app-mbti"]');
    expect(mbtiCard.text()).toContain('MBTI 性格测评');
    expect(mbtiCard.text()).toContain('测评 · 协议2');

    await mbtiCard.trigger('click');
    await flushPromises();

    expect(openApp).toHaveBeenCalledWith('mbti', {
      title: 'MBTI 性格测评',
      orientation: 'portrait',
      entry: 'index.html',
    });
    expect(openGames).not.toHaveBeenCalled();
  } finally {
    wrapper.unmount();
  }
});

it('launches apps in App Plaza via store.openApp (V2 container)', async () => {
  const { wrapper, openApp, openGames } = setupApps();
  try {
    await flushPromises();
    const gameCard = wrapper.get('[data-testid="app-merge-2048"]');
    expect(gameCard.text()).toContain('2048');

    await gameCard.trigger('click');
    await flushPromises();

    expect(openApp).toHaveBeenCalledWith('merge-2048', {
      title: '2048',
      orientation: 'portrait',
      entry: 'index.html',
    });
    expect(openGames).not.toHaveBeenCalled();
  } finally {
    wrapper.unmount();
  }
});

it('filters apps by category filter chips and search keyword', async () => {
  const { wrapper } = setupApps();
  try {
    await flushPromises();
    expect(wrapper.findAll('.app-card')).toHaveLength(2); // mbti and merge-2048 in plaza

    // Filter by assessment
    await wrapper.get('[data-testid="apps-filter-assessment"]').trigger('click');
    await flushPromises();
    expect(wrapper.findAll('.app-card')).toHaveLength(1);
    expect(wrapper.get('.app-card').text()).toContain('MBTI');

    // Filter by game
    await wrapper.get('[data-testid="apps-filter-game"]').trigger('click');
    await flushPromises();
    expect(wrapper.findAll('.app-card')).toHaveLength(1);
    expect(wrapper.get('.app-card').text()).toContain('2048');

    // Reset filter
    await wrapper.get('[data-testid="apps-filter-all"]').trigger('click');
    await flushPromises();

    // Search query
    await wrapper.get('[data-testid="apps-search-input"]').setValue('MBTI');
    await flushPromises();
    expect(wrapper.findAll('.app-card')).toHaveLength(1);
    expect(wrapper.get('.app-card').text()).toContain('MBTI');
  } finally {
    wrapper.unmount();
  }
});

it('switches to user apps tab and displays user creations', async () => {
  const { wrapper, openApp } = setupApps();
  try {
    await flushPromises();
    await wrapper.get('[data-testid="apps-tab-custom"]').trigger('click');
    await flushPromises();

    expect(wrapper.find('.apps-filter-bar').exists()).toBe(false); // chips hidden on user apps
    const section = wrapper.get('[data-testid="user-apps"]');
    expect(section.findAll('.app-card')).toHaveLength(1);
    expect(section.text()).toContain('番茄时钟');
    expect(section.text()).toContain('自创工具');

    await section.get('.app-card').trigger('click');
    await flushPromises();
    expect(openApp).toHaveBeenCalledWith('pomodoro-timer', {
      title: '番茄时钟',
      orientation: 'any',
      entry: 'pages/timer.html',
    });
  } finally {
    wrapper.unmount();
  }
});
