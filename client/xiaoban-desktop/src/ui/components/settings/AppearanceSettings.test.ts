// @vitest-environment happy-dom
import { flushPromises, mount } from '@vue/test-utils';
import { reactive } from 'vue';
import { NSwitch } from 'naive-ui';
import { describe, expect, it, vi } from 'vitest';
import { defaults } from '../../../bridge.ts';
import type { ControlStore } from '../../control-types.ts';
import AppearanceSettings from './AppearanceSettings.vue';

describe('chat pet appearance setting', () => {
  it('saves the switch and surfaces failures without changing the saved choice', async () => {
    const store = reactive({
      settings: { ...defaults }, notify: vi.fn(),
      setChatComposerPet: vi.fn(async (enabled: boolean) => { store.settings.chatComposerPetOptIn = enabled; }),
    });
    const wrapper = mount(AppearanceSettings, { props: { store: store as unknown as ControlStore } });
    const toggle = wrapper.getComponent(NSwitch);
    expect(toggle.props('value')).toBe(false);
    await toggle.vm.$emit('update:value', true);
    await flushPromises();
    expect(store.setChatComposerPet).toHaveBeenCalledWith(true);
    expect(toggle.props('value')).toBe(true);
    store.setChatComposerPet.mockRejectedValueOnce(new Error('保存失败'));
    await toggle.vm.$emit('update:value', false);
    await flushPromises();
    expect(store.notify).toHaveBeenCalledWith(expect.objectContaining({ message: '保存失败' }), 'error');
    expect(toggle.props('value')).toBe(true);
    expect(toggle.props('disabled')).toBe(false);
    wrapper.unmount();
  });
  it('opens the application skin tab in a browser preview', async () => {
    const store = reactive({ settings: { ...defaults }, notify: vi.fn(), closeSettings: vi.fn(), navigate: vi.fn(), studioTab: 'wallpapers' });
    const wrapper = mount(AppearanceSettings, { props: { store: store as unknown as ControlStore } });
    await wrapper.get('[data-testid=settings-open-skins]').trigger('click');
    expect(store.studioTab).toBe('skins'); expect(store.closeSettings).toHaveBeenCalledOnce(); expect(store.navigate).toHaveBeenCalledWith('studio');
    wrapper.unmount();
  });
});
