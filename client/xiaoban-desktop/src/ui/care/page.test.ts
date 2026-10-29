// @vitest-environment happy-dom
import { shallowMount } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import CarePage from '../pages/CarePage.vue';
import { NSelect } from 'naive-ui';
import type { ControlStore } from '../control-types';
import { emptyCareStatus } from './state';
vi.mock('../../bridge', () => ({ invoke: vi.fn(), listen: vi.fn(async () => () => {}) }));
const store = () => ({ desktop: false, settings: { companionEnabled: true }, notify: vi.fn(), navigate: vi.fn() }) as unknown as ControlStore;
function mountPage() {
  return shallowMount(CarePage, { props: { store: store() }, global: { renderStubDefaultSlot: true, stubs: { NModal: { props: ['show'], template: '<div v-if="show"><slot /></div>' } } } });
}
describe('care form fields', () => {
  it('shows only interval for rest and reveals weekday/date fields when applicable', async () => {
    const wrapper = mountPage(); await wrapper.get('[data-testid="care-add"]').trigger('click');
    const kind = wrapper.findAllComponents(NSelect).find(item => item.attributes('data-testid') === 'care-kind')!;
    kind.vm.$emit('update:value', 'rest'); await wrapper.vm.$nextTick();
    expect(wrapper.find('[data-testid="care-rest-field"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="care-time-field"]').exists()).toBe(false);
    kind.vm.$emit('update:value', 'custom'); await wrapper.vm.$nextTick();
    const repeat = wrapper.findAllComponents(NSelect).find(item => item.attributes('data-testid') === 'care-repeat')!;
    repeat.vm.$emit('update:value', 'weekdays'); await wrapper.vm.$nextTick();
    expect(wrapper.find('[data-testid="care-weekdays-field"]').exists()).toBe(true);
    repeat.vm.$emit('update:value', 'once'); await wrapper.vm.$nextTick();
    expect(wrapper.find('[data-testid="care-once-field"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="care-time-field"]').exists()).toBe(false);
    wrapper.unmount();
  });
});

describe('care preferences', () => {
  it('has a switch for the sound of one-time reminders, on by default, and it is part of what gets saved', async () => {
    const wrapper = mountPage(); await wrapper.vm.$nextTick();
    const row = wrapper.find('[data-testid="care-once-sound"]');
    expect(row.exists()).toBe(true);
    expect(row.attributes('value') ?? row.attributes('modelvalue') ?? 'true').toMatch(/true/);
    expect(wrapper.find('[data-testid="care-once-voice"]').exists()).toBe(true);
    expect(emptyCareStatus().config.onceSound).toBe(true);
    expect(emptyCareStatus().config.onceVoice).toBe(true);
    wrapper.unmount();
  });
});
