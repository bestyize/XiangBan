// @vitest-environment happy-dom
import { mount, flushPromises } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import InteractivePage from './InteractivePage.vue';
import { defaults } from '../../bridge.ts';
import { openRitual } from '../../rituals/journal.ts';
import type { ControlStore } from '../control-types.ts';
vi.mock('../../rituals/journal.ts', () => ({ openRitual: vi.fn().mockResolvedValue(undefined) }));
vi.mock('../../rituals/PondScene.vue', () => ({ default: { template: '<div />' } }));
vi.mock('../../rituals/FortuneVessel.vue', () => ({ default: { template: '<div />' } }));
function setup() {
  const store = {
    desktop: true, settings: JSON.parse(JSON.stringify(defaults)), game: { active: false },
    library: { items: [{ definition: { id: 'builtin-mint-orbits', kind: 'canvas', title: '浮光', description: '流动风景', controls: [{ key: 'speed', label: '速度', type: 'range', default: .6, min: .1, max: 1.5, step: .1 }] } }] },
    notify: vi.fn(), applyWallpaper: vi.fn().mockResolvedValue(undefined),
  } as unknown as ControlStore;
  const view = mount(InteractivePage, { props: { store } }); return { view, store };
}
describe('interactive wallpaper entry', () => {
  it('opens independent scenes and never applies a wallpaper as a side effect', async () => {
    const { view, store } = setup();
    await view.get('[data-testid="enter-fortune"]').trigger('click'); await flushPromises();
    expect(openRitual).toHaveBeenLastCalledWith('fortune'); expect(store.applyWallpaper).not.toHaveBeenCalled();
    await view.get('[data-testid="enter-pond"]').trigger('click'); await flushPromises();
    expect(openRitual).toHaveBeenLastCalledWith('pond'); view.unmount();
  });
  it('only applies the pond through its explicit wallpaper button', async () => {
    const { view, store } = setup();
    await view.get('[data-testid="apply-pond"]').trigger('click'); await flushPromises();
    expect(store.applyWallpaper).toHaveBeenCalledWith('builtin-wish-pond', expect.objectContaining({values:{}}), false); view.unmount();
  });
  it('keeps the migrated procedural wallpaper controls usable', async () => {
    const { view, store } = setup();
    await view.get('input[type="range"]').setValue('1.2');
    await view.get('.landscape-actions button:last-child').trigger('click'); await flushPromises();
    expect(store.applyWallpaper).toHaveBeenCalledWith('builtin-mint-orbits', expect.objectContaining({values:{speed:1.2}}), false); view.unmount();
  });
});
