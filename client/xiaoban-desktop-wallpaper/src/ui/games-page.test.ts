// @vitest-environment happy-dom
import { mount, flushPromises } from '@vue/test-utils';
import { it, expect, vi } from 'vitest';
import GamesPage from './pages/GamesPage.vue';
import type { ControlStore } from './control-types';
it('opens the selected local game on a single click and supports keyboard entry', async () => {
  const openGames = vi.fn().mockResolvedValue(undefined);
  const store = { desktop:true, settings:{gameDesktopDoubleClick:false}, game:{desktopDoubleClickSupported:true}, openGames, notify:vi.fn() } as unknown as ControlStore;
  vi.stubGlobal('fetch', vi.fn(async (url:string) => ({ok:true,json:async () => url.endsWith('catalog.json') ? {games:['merge-2048']} : {id:'merge-2048',name:'2048',description:'合成数字',cover:'cover.svg'}})));
  const wrapper=mount(GamesPage,{props:{store},global:{stubs:{NAlert:true,NButton:true,NSkeleton:true,NSwitch:true,NTag:true}}});
  try {
    await flushPromises();
    expect(wrapper.find('h1').exists()).toBe(false);
    await wrapper.get('.game-card').trigger('click'); await flushPromises();
    expect(openGames).toHaveBeenCalledWith('merge-2048');
    expect(wrapper.find('.shortcut-card').exists()).toBe(false);
    await wrapper.get('.game-card').trigger('keydown',{key:'Enter'}); await flushPromises();
    expect(openGames).toHaveBeenCalledTimes(2);
  } finally { wrapper.unmount(); vi.unstubAllGlobals(); }
});
