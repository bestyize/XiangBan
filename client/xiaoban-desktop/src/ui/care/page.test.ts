// @vitest-environment happy-dom
import { mount } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import CarePage from '../pages/CarePage.vue';
import type { ControlStore } from '../control-types';
import { emptyCareStatus } from './state';

vi.mock('../../bridge', () => ({ invoke: vi.fn(), listen: vi.fn(async () => () => {}) }));
const store = () => ({ desktop: false, settings: { companionEnabled: true }, notify: vi.fn(), navigate: vi.fn(), chat: { select: vi.fn() } }) as unknown as ControlStore;
function mountPage() {
  return mount(CarePage, { props: { store: store() }, global: { stubs: { NAlert: true } } });
}

describe('care form fields', () => {
  it('shows only interval for rest and reveals weekday/interval/date fields when applicable', async () => {
    const wrapper = mountPage();
    await wrapper.get('[data-testid="care-add"]').trigger('click');
    await wrapper.get('[data-testid="care-kind-rest"]').trigger('click');
    expect(wrapper.find('[data-testid="care-rest-field"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="care-time-field"]').exists()).toBe(false);

    await wrapper.get('[data-testid="care-kind-custom"]').trigger('click');
    await wrapper.get('[data-testid="care-repeat-weekdays"]').trigger('click');
    expect(wrapper.find('[data-testid="care-weekdays-field"]').exists()).toBe(true);

    await wrapper.get('[data-testid="care-repeat-interval"]').trigger('click');
    expect(wrapper.find('[data-testid="care-interval-field"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="care-weekdays-field"]').exists()).toBe(true);

    await wrapper.get('[data-testid="care-repeat-once"]').trigger('click');
    expect(wrapper.find('[data-testid="care-once-field"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="care-time-field"]').exists()).toBe(false);

    // Per-rule delivery channels can be toggled independently inside the editor.
    const soundBtn = wrapper.get('[data-testid="care-channel-sound"]');
    expect(soundBtn.attributes('aria-pressed')).toBe('false');
    await soundBtn.trigger('click');
    expect(soundBtn.attributes('aria-pressed')).toBe('true');

    wrapper.unmount();
  });

  it('adds a reminder immediately via the quick-capture bar', async () => {
    const wrapper = mountPage();
    const input = wrapper.get('[data-testid="care-quick-input"]');
    await input.setValue('30分钟后 关火');
    await wrapper.get('[data-testid="care-quick-capture"]').trigger('submit');
    await wrapper.vm.$nextTick();
    expect(wrapper.text()).toContain('关火');
    wrapper.unmount();
  });
});

describe('care preferences', () => {
  it('has switches for one-time sound and voice inside the preferences popover, on by default', async () => {
    const wrapper = mountPage();
    await wrapper.vm.$nextTick();
    await wrapper.get('[data-testid="care-prefs-toggle"]').trigger('click');
    const row = wrapper.find('[data-testid="care-once-sound"]');
    expect(row.exists()).toBe(true);
    expect(wrapper.find('[data-testid="care-once-voice"]').exists()).toBe(true);
    expect(emptyCareStatus().config.onceSound).toBe(true);
    expect(emptyCareStatus().config.onceVoice).toBe(true);
    wrapper.unmount();
  });
});
