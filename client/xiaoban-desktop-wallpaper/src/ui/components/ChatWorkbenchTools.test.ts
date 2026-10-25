// @vitest-environment happy-dom
import { mount, flushPromises } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import ChatWorkbenchTools from './ChatWorkbenchTools.vue';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';

vi.mock('../../bridge.ts', async (importOriginal) => ({
  ...await importOriginal<typeof import('../../bridge.ts')>(),
  invoke: vi.fn(),
}));

function setup() {
  const store = { desktop: true, notify: vi.fn(), navigate: vi.fn() } as unknown as ControlStore;
  const wrapper = mount(ChatWorkbenchTools, {
    props: { store },
    attachTo: document.body,
  });
  return { wrapper };
}
beforeEach(() => {
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation((async (command: string) => {
    if (command === 'ai_store_experts_installed' || command === 'ai_store_installed' || command === 'chat_invocable_skills' || command === 'ai_store_connector_status') return [];
    return { results: [] };
  }) as typeof invoke);
});
afterEach(() => { vi.useRealTimers(); });

describe('add-capability menu hover interaction', () => {
  it('opens on hover and stays open while crossing the gap into the menu', async () => {
    vi.useFakeTimers();
    const { wrapper } = setup();
    const trigger = wrapper.get('[data-testid="chat-add-menu"]');
    await trigger.trigger('mouseenter');
    await flushPromises();
    expect(trigger.attributes('aria-expanded')).toBe('true');
    expect(wrapper.get('.wb-main').isVisible()).toBe(true);

    // Clicking the already hover-open trigger must not immediately dismiss the menu.
    await trigger.trigger('click');
    expect(wrapper.get('.wb-main').isVisible()).toBe(true);

    await wrapper.get('.wb-controls').trigger('mouseleave');
    await wrapper.get('.wb-main').trigger('mouseenter');
    vi.advanceTimersByTime(220);
    await flushPromises();
    expect(wrapper.get('.wb-main').isVisible()).toBe(true);
    wrapper.unmount();
  });

  it('closes after the pointer leaves the whole menu region', async () => {
    vi.useFakeTimers();
    const { wrapper } = setup();
    await wrapper.get('[data-testid="chat-add-menu"]').trigger('mouseenter');
    await flushPromises();
    await wrapper.get('.wb-controls').trigger('mouseleave');
    vi.advanceTimersByTime(220);
    await flushPromises();
    expect(wrapper.get('[data-testid="chat-add-menu"]').attributes('aria-expanded')).toBe('false');
    expect(wrapper.get('.wb-main').isVisible()).toBe(false);
    wrapper.unmount();
  });

  it('still opens when focused for keyboard access and closes on Escape', async () => {
    const { wrapper } = setup();
    await wrapper.get('[data-testid="chat-add-menu"]').trigger('focus');
    await flushPromises();
    expect(wrapper.get('.wb-main').isVisible()).toBe(true);
    await wrapper.get('[data-testid="chat-add-menu"]').trigger('keydown', { key: 'Escape' });
    expect(wrapper.get('.wb-main').isVisible()).toBe(false);
    wrapper.unmount();
  });
});
