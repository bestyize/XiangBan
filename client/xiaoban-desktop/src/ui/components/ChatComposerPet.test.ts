// @vitest-environment happy-dom
import { mount, flushPromises } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import ChatComposerPet from './ChatComposerPet.vue';

const actorPoke = vi.hoisted(() => vi.fn());
vi.mock('../../pets/model.ts', () => ({
  OFFICIAL_THEMES: [{ pets: [{ id: 'sunflower', name: '向日葵', parts: [], kind: 'flower' }] }],
}));
vi.mock('../../pets/render.ts', () => ({ petImage: () => 'data:image/svg+xml,test-pet' }));
vi.mock('../../pets/PetActor.vue', async () => {
  const { defineComponent, h } = await import('vue');
  return { default: defineComponent({
    name: 'StubPetActor',
    props: ['definition', 'size', 'paused', 'interactive'],
    setup(_props, { expose }) {
      expose({ poke: actorPoke });
      return () => h('div', { class: 'pet-art', tabindex: 0 }, 'animated pet');
    },
  }) };
});

beforeEach(() => { actorPoke.mockClear(); });
afterEach(() => { vi.useRealTimers(); document.body.replaceChildren(); });

describe('small pet above the chat composer', () => {
  it('renders an accessible clickable control without changing the original pet asset', async () => {
    const wrapper = mount(ChatComposerPet, { props: { size: 'sm' }, attachTo: document.body });
    await flushPromises();
    const pet = wrapper.get('[data-testid="composer-pet"]');
    expect(pet.element.tagName).toBe('BUTTON');
    expect(pet.attributes('aria-label')).toBe('摸摸小伴');
    expect(wrapper.get('.composer-pet-picture').attributes('src')).toBe('data:image/svg+xml,test-pet');
    expect(wrapper.classes()).not.toContain('is-lg');
    expect(wrapper.find('.composer-pet-live').exists()).toBe(false);
    wrapper.unmount();
  });

  it('reacts with the existing pet actor only after a click, then returns to the light still image', async () => {
    const wrapper = mount(ChatComposerPet, { props: { size: 'lg' }, attachTo: document.body });
    await flushPromises();
    vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'] });
    expect(wrapper.get('.composer-pet').classes()).toContain('is-lg');
    expect(wrapper.find('.composer-pet-live').exists()).toBe(false);
    expect(actorPoke).not.toHaveBeenCalled();

    await wrapper.get('[data-testid="composer-pet"]').trigger('click');
    await flushPromises();
    expect(wrapper.get('[data-testid="composer-pet-reply"]').text()).toContain('收到这一点温柔');
    expect(wrapper.find('.composer-pet-live').exists()).toBe(true);
    expect(wrapper.findComponent({ name: 'StubPetActor' }).props('size')).toBe(58);
    expect(actorPoke).toHaveBeenCalledTimes(1);
    expect(wrapper.get('.pet-art').attributes('tabindex')).toBe('-1');

    vi.advanceTimersByTime(2400);
    await flushPromises();
    expect(wrapper.find('.composer-pet-live').exists()).toBe(false);
    expect(wrapper.find('[data-testid="composer-pet-reply"]').exists()).toBe(true);
    vi.advanceTimersByTime(9600); await flushPromises();
    expect(wrapper.find('[data-testid="composer-pet-reply"]').exists()).toBe(false);
    expect(wrapper.get('.composer-pet-picture').classes()).not.toContain('is-hidden');
    wrapper.unmount();
  });

  it('tapping again resets the reply timer rather than creating duplicate actors', async () => {
    const wrapper = mount(ChatComposerPet, { props: { size: 'sm' }, attachTo: document.body });
    await flushPromises();
    vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'] });
    await wrapper.get('[data-testid="composer-pet"]').trigger('click');
    await flushPromises();
    vi.advanceTimersByTime(1800);
    await wrapper.get('[data-testid="composer-pet"]').trigger('click');
    await flushPromises();
    expect(actorPoke).toHaveBeenCalledTimes(2);
    vi.advanceTimersByTime(1000);
    await flushPromises();
    expect(wrapper.find('[data-testid="composer-pet-reply"]').exists()).toBe(true);
    vi.advanceTimersByTime(1400);
    await flushPromises();
    expect(wrapper.find('[data-testid="composer-pet-reply"]').exists()).toBe(true);
    vi.advanceTimersByTime(9600); await flushPromises();
    expect(wrapper.find('[data-testid="composer-pet-reply"]').exists()).toBe(false);
    wrapper.unmount();
  });
});

