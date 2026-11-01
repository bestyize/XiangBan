// @vitest-environment happy-dom
import { mount } from '@vue/test-utils';
import { afterEach, describe, expect, it, vi } from 'vitest';
import SkinScene from './SkinScene.vue';
const img = 'data:image/png;base64,aGVsbG8=';
const scene = { layers: [{ id: 'river', image: img, anchor: 'center', depth: 0 }, { id: 'lamp', image: img, anchor: 'bottom-right', depth: 2, interactive: true }, { id: 'bad', image: 'https://example.com/tracker.png', anchor: 'center', depth: 1 }], interactions: [{ target: 'lamp', trigger: 'click', action: 'glow', label: '点亮花灯' }] };
afterEach(() => { vi.restoreAllMocks(); document.body.replaceChildren(); });
describe('application skin scene', () => {
  it('only renders inline raster images and exposes a labelled keyboard-operable target', async () => {
    vi.spyOn(document, 'hasFocus').mockReturnValue(true);
    const wrapper = mount(SkinScene, { props: { scene }, attachTo: document.body });
    expect(wrapper.findAll('img')).toHaveLength(2);
    const lamp = wrapper.get('button'); expect(lamp.attributes('aria-label')).toBe('点亮花灯');
    await lamp.trigger('click'); await wrapper.vm.$nextTick(); expect(lamp.classes()).toContain('is-active');
    wrapper.unmount();
  });
  it('does not register interaction when disabled and pauses all movement', async () => {
    const wrapper = mount(SkinScene, { props: { scene, interactive: false, paused: true } });
    expect(wrapper.find('button').exists()).toBe(false);
    expect(wrapper.classes()).toContain('is-paused');
    wrapper.unmount();
  });
});
