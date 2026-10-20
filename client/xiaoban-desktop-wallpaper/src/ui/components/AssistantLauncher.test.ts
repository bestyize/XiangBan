// @vitest-environment happy-dom
import { mount } from '@vue/test-utils';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import AssistantLauncher from './AssistantLauncher.vue';

let resized: () => void;
const disconnect = vi.fn();
beforeEach(() => {
  vi.stubGlobal('ResizeObserver', class {
    constructor(callback: () => void) { resized = callback; }
    observe() {} disconnect = disconnect;
  });
});
afterEach(() => { document.body.replaceChildren(); vi.unstubAllGlobals(); vi.clearAllMocks(); });

function setup() {
  const parent = document.createElement('div');
  document.body.append(parent);
  let width = 800, height = 600;
  parent.getBoundingClientRect = () => ({ left: 170, top: 20, width, height } as DOMRect);
  const wrapper = mount(AssistantLauncher, { attachTo: parent });
  const button = wrapper.element as HTMLButtonElement;
  button.parentElement!.getBoundingClientRect = parent.getBoundingClientRect;
  button.getBoundingClientRect = () => ({ left: 170 + (parseFloat(button.style.left) || 722), top: 20 + (parseFloat(button.style.top) || 498), width: 54, height: 54 } as DOMRect);
  button.setPointerCapture = vi.fn();
  button.hasPointerCapture = vi.fn(() => true);
  button.releasePointerCapture = vi.fn();
  const pointer = (type: string, x: number, y: number, id = 1) => wrapper.trigger(type, { button: 0, pointerId: id, clientX: x, clientY: y });
  return { wrapper, button, pointer, resize: async (w: number, h: number) => { width = w; height = h; resized(); await wrapper.vm.$nextTick(); } };
}

it('uses a conversation icon and opens on a click or slight pointer movement', async () => {
  const { wrapper, pointer } = setup();
  expect(wrapper.find('svg.lucide-message-circle-more').exists()).toBe(true);
  await pointer('pointerdown', 900, 530);
  await pointer('pointermove', 902, 532);
  await pointer('pointerup', 902, 532);
  await wrapper.trigger('click');
  expect(wrapper.emitted('open')).toHaveLength(1);
  expect(wrapper.attributes('style')).toBeUndefined();
  wrapper.unmount();
});

it('drags within the app without opening, preserves placement across pages and still allows a later click', async () => {
  const { wrapper, button, pointer } = setup();
  await pointer('pointerdown', 900, 530);
  await pointer('pointermove', 600, 330);
  expect(button.style.left).toBe('422px');
  expect(button.style.top).toBe('298px');
  expect(wrapper.classes()).toContain('dragging');
  await pointer('pointerup', 600, 330);
  await wrapper.trigger('click');
  expect(wrapper.emitted('open')).toBeUndefined();
  await wrapper.setProps({ nearComposer: true });
  expect(button.style.top).toBe('298px');
  await pointer('pointerdown', 600, 330);
  await pointer('pointerup', 600, 330);
  await wrapper.trigger('click');
  expect(wrapper.emitted('open')).toHaveLength(1);
  wrapper.unmount();
  expect(disconnect).toHaveBeenCalledOnce();
});

it('clamps dragging and window resizing to keep the entire button visible', async () => {
  const { wrapper, button, pointer, resize } = setup();
  await pointer('pointerdown', 900, 530);
  await pointer('pointermove', -1000, -1000);
  expect(button.style.left).toBe('12px');
  expect(button.style.top).toBe('12px');
  await pointer('pointermove', 3000, 3000);
  await pointer('pointerup', 3000, 3000);
  expect(button.style.left).toBe('734px');
  expect(button.style.top).toBe('534px');
  await resize(400, 300);
  expect(button.style.left).toBe('334px');
  expect(button.style.top).toBe('234px');
  wrapper.unmount();
});

it('ignores other pointers and cancels a gesture without opening the dialog', async () => {
  const { wrapper, button, pointer } = setup();
  await pointer('pointerdown', 900, 530);
  await pointer('pointermove', 100, 100, 2);
  expect(button.style.left).toBe('');
  await pointer('pointercancel', 900, 530);
  await wrapper.trigger('click');
  expect(wrapper.emitted('open')).toBeUndefined();
  await wrapper.trigger('keydown', { key: 'Enter' });
  await wrapper.trigger('click');
  expect(wrapper.emitted('open')).toHaveLength(1);
  wrapper.unmount();
});

it('keeps a disabled or busy launcher inactive', async () => {
  const { wrapper, pointer } = setup();
  for (const props of [{ disabled: true, busy: false }, { disabled: false, busy: true }]) {
    await wrapper.setProps(props);
    await pointer('pointerdown', 900, 530);
    await pointer('pointermove', 600, 330);
    await wrapper.trigger('click');
    expect(wrapper.emitted('open')).toBeUndefined();
    expect(wrapper.attributes('style')).toBeUndefined();
  }
  wrapper.unmount();
});
