// @vitest-environment happy-dom
import { flushPromises, mount } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import { reactive } from 'vue';
import CompanionPage from './CompanionPage.vue';
import { defaults } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';

function setup(companionLinks = true) {
  const store = reactive({
    desktop: false,
    settings: { ...JSON.parse(JSON.stringify(defaults)), companionLinks },
    notify: vi.fn(), navigate: vi.fn(),
    updateAppearance: vi.fn(async () => {}), setMode: vi.fn(async () => {}), setCompanion: vi.fn(async () => {}),
    setCompanionLinks: vi.fn(async (enabled: boolean) => { store.settings.companionLinks = enabled; }),
  }) as unknown as ControlStore;
  const wrapper = mount(CompanionPage, { props: { store }, global: { stubs: { PetPreview: true } } });
  return { wrapper, store };
}

describe('companion page links switch', () => {
  it('shows the switch on by default with its explanation', () => {
    const { wrapper } = setup();
    const toggle = wrapper.get('[data-testid="companion-links"]');
    expect(toggle.attributes('aria-checked')).toBe('true');
    expect(wrapper.text()).toContain('小伴联动：让小伴回应特效和你的状态');
  });

  it('turns links off and on through the store', async () => {
    const { wrapper, store } = setup();
    await wrapper.get('[data-testid="companion-links"]').trigger('click');
    await flushPromises();
    expect(store.setCompanionLinks).toHaveBeenLastCalledWith(false);
    expect(wrapper.get('[data-testid="companion-links"]').attributes('aria-checked')).toBe('false');
    await wrapper.get('[data-testid="companion-links"]').trigger('click');
    await flushPromises();
    expect(store.setCompanionLinks).toHaveBeenLastCalledWith(true);
  });

  it('reflects a saved off value and reports a failed save', async () => {
    const { wrapper, store } = setup(false);
    expect(wrapper.get('[data-testid="companion-links"]').attributes('aria-checked')).toBe('false');
    vi.mocked(store.setCompanionLinks).mockRejectedValueOnce(new Error('保存失败'));
    await wrapper.get('[data-testid="companion-links"]').trigger('click');
    await flushPromises();
    expect(store.notify).toHaveBeenCalledWith(expect.any(Error), 'error');
    expect(wrapper.get('[data-testid="companion-links"]').attributes('aria-checked')).toBe('false');
  });
});
