// @vitest-environment happy-dom
import { flushPromises, mount } from '@vue/test-utils';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { invoke } from '../../bridge.ts';
import SkinsPage from './SkinsPage.vue';
import type { ControlStore } from '../control-types.ts';
import { activeSkin, activeSkinId } from '../skins/store.ts';
vi.mock('../../bridge.ts', async getOriginal => ({ ...await getOriginal<typeof import('../../bridge.ts')>(), invoke: vi.fn() }));
const entry = { id: 'my-skin', name: '自制皮肤', version: '1', author: '我', preview: 'data:image/png;base64,AA==', scene: { layers: [], interactions: [] } };
beforeEach(() => { activeSkin.value = null; activeSkinId.value = null; vi.mocked(invoke).mockReset().mockImplementation(async (command, args) => command === 'skin_import' ? entry : command === 'skin_export' ? true : { skins: [entry], activeId: command === 'skin_apply' ? (args as { id: string | null }).id : null }); });
afterEach(() => document.body.replaceChildren());
it('applies and restores the default from the cards', async () => {
 const store = { desktop: true, notify: vi.fn() } as unknown as ControlStore;
 const wrapper = mount(SkinsPage, { props: { store }, attachTo: document.body }); await flushPromises();
 expect(wrapper.get('[data-testid="skin-my-skin"]').text()).toContain('自制皮肤'); expect(invoke).not.toHaveBeenCalledWith('skin_apply', expect.anything());
 await wrapper.get('[data-testid="skin-apply-my-skin"]').trigger('click'); await flushPromises();
 expect(invoke).toHaveBeenCalledWith('skin_apply', { id: 'my-skin' }); expect(activeSkin.value?.name).toBe('自制皮肤');
 await wrapper.get('[data-testid=skin-apply-default]').trigger('click'); await flushPromises();
 expect(invoke).toHaveBeenCalledWith('skin_apply', { id: null }); expect(activeSkin.value).toBeNull(); wrapper.unmount();
});
it('imports without applying and requires confirmation before deleting', async () => {
 const store = { desktop: true, notify: vi.fn() } as unknown as ControlStore;
 const wrapper = mount(SkinsPage, { props: { store }, attachTo: document.body }); await flushPromises();
 await wrapper.get('[data-testid=skin-import]').trigger('click'); await flushPromises();
 expect(invoke).toHaveBeenCalledWith('skin_import');
 expect(invoke).not.toHaveBeenCalledWith('skin_apply', expect.anything());
 const remove = wrapper.findAll('button').find(button => button.text() === '删除'); expect(remove).toBeDefined();
 await remove!.trigger('click'); expect(invoke).not.toHaveBeenCalledWith('skin_delete', expect.anything());
 const confirm = wrapper.get('[role=alert]'); await confirm.findAll('button').find(button => button.text() === '删除')!.trigger('click'); await flushPromises();
 expect(invoke).toHaveBeenCalledWith('skin_delete', { id: 'my-skin' }); wrapper.unmount();
});

it('hands skin creation to a new chat', async () => {
 const store = { desktop: true, notify: vi.fn(), startCreation: vi.fn() } as unknown as ControlStore;
 const wrapper = mount(SkinsPage, { props: { store }, attachTo: document.body }); await flushPromises();
 await wrapper.get('[data-testid=skin-create]').trigger('click');
 expect(store.startCreation).toHaveBeenCalledWith('skin'); wrapper.unmount();
});

it('includes both new built-in skins and applies them without delete or export actions', async () => {
 const store = { desktop: true, notify: vi.fn() } as unknown as ControlStore;
 const wrapper = mount(SkinsPage, { props: { store }, attachTo: document.body }); await flushPromises();
 for (const [id, name] of [['builtin:jiangnan', '江南烟雨'], ['builtin:seaside-train', '海边列车']]) {
   const card = wrapper.get(`[data-testid="skin-${id}"]`);
   expect(card.text()).toContain(name);
   expect(card.text()).not.toContain('删除'); expect(card.text()).not.toContain('导出');
   await wrapper.get(`[data-testid="skin-apply-${id}"]`).trigger('click'); await flushPromises();
   expect(invoke).toHaveBeenCalledWith('skin_apply', { id }); expect(activeSkin.value?.id).toBe(id);
 }
 wrapper.unmount();
});
