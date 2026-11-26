// @vitest-environment happy-dom
import { flushPromises, mount } from '@vue/test-utils';
import { beforeEach, expect, it, vi } from 'vitest';
import { defaults, invoke } from '../../bridge.ts';
import { OFFICIAL_THEMES, type PetCatalog } from '../../pets/model.ts';
import PetConfigWindow from './PetConfigWindow.vue';

vi.mock('../../bridge.ts', async original => ({
  ...await original<typeof import('../../bridge.ts')>(), desktop: true,
  invoke: vi.fn(), listen: vi.fn(async () => () => {}),
}));
vi.mock('../control-store.ts', () => ({ createControlStore: () => ({ dispose() {} }) }));

const catalog: PetCatalog = {
  themes: [{ ...OFFICIAL_THEMES[0], source: 'builtin' }], errors: [],
  layout: { version: 1, instances: [{ id: 'test-pet', themeId: OFFICIAL_THEMES[0].id, petId: 'daisy',
    name: '小雏菊', size: 210, x: 0.5, y: 0.5, layer: 'topmost', enabled: true, links: true, expression: 'idle' }] },
};

beforeEach(() => {
  history.replaceState(null, '', '/?scene=configuration&kind=pet&provider=test-pet');
  vi.mocked(invoke).mockReset().mockImplementation(async (name: string) => {
    if (name === 'get_settings') return defaults;
    if (name === 'get_app_info') return { platform: 'macos' };
    if (name === 'get_pet_catalog') return catalog;
  });
});

it('keeps the detached editor open until the size is saved, including on a failed first attempt', async () => {
  const wrapper = mount(PetConfigWindow, { global: { stubs: { PetActor: true, WindowChrome: true, WindowResizeEdges: true } } });
  try {
    await flushPromises();
    await wrapper.get('input[type=range]').setValue('300');
    vi.mocked(invoke).mockRejectedValueOnce('保存失败');
    const done = wrapper.get('[data-testid="pet-dialog-done"]');
    await done.trigger('click');
    await flushPromises();
    expect(wrapper.get('.pet-config-toast').text()).toBe('保存失败');
    expect(wrapper.get('[data-testid="pet-dialog-size"]').text()).toContain('300');
    expect(invoke).not.toHaveBeenCalledWith('configuration_window_close');
    let finish!: (value: PetCatalog) => void;
    vi.mocked(invoke).mockImplementationOnce(() => new Promise(resolve => { finish = resolve; }));
    await done.trigger('click');
    expect(invoke).toHaveBeenCalledWith('update_pet', { id: 'test-pet', patch: { size: 300 } });
    expect(invoke).not.toHaveBeenCalledWith('configuration_window_close');
    finish(catalog);
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('configuration_window_close');
  } finally { wrapper.unmount(); }
});
