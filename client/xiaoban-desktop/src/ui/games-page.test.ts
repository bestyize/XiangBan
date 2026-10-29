// @vitest-environment happy-dom
import { mount, flushPromises } from '@vue/test-utils';
import { it, expect, vi } from 'vitest';
import { NPopconfirm } from 'naive-ui';
import GamesPage from './pages/GamesPage.vue';
import type { ControlStore } from './control-types';
import { fallbackCoverUrl } from '../games/catalog.ts';

const builtin = { id: 'merge-2048', name: '2048', description: '合成数字', entry: 'index.html', cover: 'cover.svg', runtimeProtocol: 1, source: 'builtin' as const, base: '/game-packs/merge-2048/' };
const mine = { id: 'star-catcher', name: '接星星', description: '接住落下的星星', entry: 'index.html', runtimeProtocol: 1, save: true, source: 'user' as const, base: 'game-pack://localhost/star-catcher/' };
const stubs = { NAlert: { template: '<div class="alert"><slot /></div>' }, NButton: { template: '<button><slot /></button>' }, NSkeleton: true, NTag: true, NPopconfirm: { template: '<div><slot name="trigger" /><slot /></div>', emits: ['positive-click'] } };

function setup(catalog: { games: unknown[]; errors?: string[] }, desktop = true) {
  const openGames = vi.fn().mockResolvedValue(undefined);
  const store = {
    desktop, settings: { gameDesktopDoubleClick: false }, game: { desktopDoubleClickSupported: true }, openGames, notify: vi.fn(),
    listGames: vi.fn(async () => ({ games: catalog.games, errors: catalog.errors ?? [], directory: null })),
    deleteUserGame: vi.fn().mockResolvedValue(undefined), openUserGamesFolder: vi.fn().mockResolvedValue(undefined),
  } as unknown as ControlStore;
  return { store, openGames, wrapper: mount(GamesPage, { props: { store }, global: { stubs } }) };
}

async function customTab(wrapper:ReturnType<typeof mount>){await wrapper.findAll('.xb-tabs-lg [role=tab]').find(t=>t.text()==='我的游戏')!.trigger('click');await flushPromises();}

it('opens the selected local game on a single click and supports keyboard entry', async () => {
  const { wrapper, openGames } = setup({ games: [builtin] });
  try {
    await flushPromises();
    expect(wrapper.findAll('.xb-tabs-lg [role=tab]').map(t=>t.text())).toEqual(['官方游戏','我的游戏']);expect(wrapper.find('.resource-library-header').exists()).toBe(false);
    expect(wrapper.get('.game-card img').attributes('src')).toBe('/game-packs/merge-2048/cover.svg');
    await wrapper.get('.game-card').trigger('click'); await flushPromises();
    expect(openGames).toHaveBeenCalledWith('merge-2048');
    expect(wrapper.find('.shortcut-card').exists()).toBe(false);expect(wrapper.find('h2').exists()).toBe(false);expect(wrapper.find('.section-heading').exists()).toBe(false);expect(wrapper.find('.games-body').text()).not.toContain('离线可玩');
    await wrapper.get('.game-card').trigger('keydown', { key: 'Enter' }); await flushPromises();
    expect(openGames).toHaveBeenCalledTimes(2);
  } finally { wrapper.unmount(); }
});

it('shows the user games in their own section with the default cover, and starts them by id', async () => {
  const { wrapper, openGames } = setup({ games: [builtin, mine], errors: ['「broken」：缺少 manifest.json'] });
  try {
    await flushPromises();
    await customTab(wrapper);
    const section = wrapper.get('[data-testid="user-games"]');
    expect(section.findAll('.game-card')).toHaveLength(1);
    expect(section.get('img').attributes('src')).toBe(fallbackCoverUrl(mine));
    expect(section.text()).toContain('接星星');
    expect(section.get('[data-testid="user-game-errors"]').text()).toContain('broken');
    expect(wrapper.findAll('.game-card')).toHaveLength(1);
    await section.get('.game-card').trigger('click'); await flushPromises();
    expect(openGames).toHaveBeenCalledWith('star-catcher');
  } finally { wrapper.unmount(); }
});

it('falls back when a custom cover fails to load without retrying the broken asset', async () => {
  const { wrapper } = setup({ games: [{ ...mine, cover: 'cover.svg' }] });
  try {
    await flushPromises();
    await customTab(wrapper);
    const image = wrapper.get('img');
    expect(image.attributes('src')).toBe(`${mine.base}cover.svg`);
    await image.trigger('error');
    expect(image.attributes('src')).toBe(fallbackCoverUrl(mine));
    await image.trigger('error');
    expect(image.attributes('src')).toBe(fallbackCoverUrl(mine));
  } finally { wrapper.unmount(); }
});

it('explains how to add games when there are none yet', async () => {
  const { wrapper } = setup({ games: [builtin] });
  try {
    await flushPromises();
    await customTab(wrapper);
    const section = wrapper.get('[data-testid="user-games"]');
    expect(section.text()).toContain('还没有自定义游戏');
  } finally { wrapper.unmount(); }
});

it('deletes a user game only after the confirmation and then reloads the list', async () => {
  const { wrapper, store } = setup({ games: [builtin, mine] });
  try {
    await flushPromises();
    await customTab(wrapper);
    await wrapper.get('[data-testid="delete-game-star-catcher"]').trigger('click');
    expect(store.deleteUserGame).not.toHaveBeenCalled();
    expect(wrapper.findAll('.game-card')).toHaveLength(1);
    wrapper.findComponent(NPopconfirm).vm.$emit('positive-click');
    await flushPromises();
    expect(store.deleteUserGame).toHaveBeenCalledWith('star-catcher');
    expect(store.listGames).toHaveBeenCalledTimes(2);
    expect(store.notify).toHaveBeenCalledWith('已删除游戏「接星星」', 'success');
  } finally { wrapper.unmount(); }
});

it('keeps the browser preview to the shipped games', async () => {
  const { wrapper } = setup({ games: [builtin] }, false);
  try {
    await flushPromises();
    expect(wrapper.find('[data-testid="user-games"]').exists()).toBe(false);
    expect(wrapper.findAll('.game-card')).toHaveLength(1);
  } finally { wrapper.unmount(); }
});
