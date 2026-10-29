// @vitest-environment happy-dom
import { mount, flushPromises } from '@vue/test-utils';
import { beforeEach, afterEach, it, expect, vi } from 'vitest';
import PetWindow from './PetWindow.vue';
const host = vi.hoisted(() => ({ invoke: vi.fn(), poke: vi.fn(), hit: vi.fn(() => true) }));
vi.mock('../bridge.ts', () => ({ invoke: host.invoke }));
vi.mock('./PetActor.vue', async () => {
  const { defineComponent, h } = await import('vue');
  return { default: defineComponent({ setup(_, { expose }) { expose({ hit: host.hit, poke: host.poke, look: vi.fn() }); return () => h('div'); } }) };
});
beforeEach(() => {
  vi.useFakeTimers(); host.poke.mockClear(); host.hit.mockReturnValue(true); host.invoke.mockReset();
  host.invoke.mockImplementation(async (command: string) => command === 'get_pet_instance' ? {
    instance: { id: 'a', size: 100, expression: 'idle', links: true }, definition: {}, paused: false, fps: 30, links: true, effect: '',
  } : undefined);
});
afterEach(() => { vi.useRealTimers(); });
it('animates every touch but merges quick clicks into a single dialogue request', async () => {
  const wrapper = mount(PetWindow); await flushPromises();
  for (let i = 0; i < 3; i++) await wrapper.trigger('click', { clientX: 10, clientY: 10 });
  expect(host.poke).toHaveBeenCalledTimes(3);
  expect(host.invoke.mock.calls.filter(c => c[0] === 'companion_dialogue_click')).toHaveLength(0);
  vi.advanceTimersByTime(150); await flushPromises();
  expect(host.invoke.mock.calls.filter(c => c[0] === 'companion_dialogue_click')).toHaveLength(1); wrapper.unmount();
});
it('does not start a dialogue after dragging or clicking transparent space', async () => {
  const wrapper = mount(PetWindow); await flushPromises();
  Object.defineProperty(wrapper.element, 'setPointerCapture', { value: vi.fn() });
  await wrapper.trigger('pointerdown', { button: 0, clientX: 10, clientY: 10, screenX: 0, screenY: 0, pointerId: 1 });
  await wrapper.trigger('pointermove', { clientX: 40, clientY: 10, screenX: 30, screenY: 0 });
  await wrapper.trigger('pointerup', { clientX: 40, clientY: 10, screenX: 30, screenY: 0 });
  await wrapper.trigger('click', { clientX: 40, clientY: 10 });
  host.hit.mockReturnValue(false); await wrapper.trigger('click', { clientX: 80, clientY: 80 });
  vi.advanceTimersByTime(200); await flushPromises();
  expect(host.poke).not.toHaveBeenCalled();
  expect(host.invoke.mock.calls.some(c => c[0] === 'companion_dialogue_click')).toBe(false); wrapper.unmount();
});
