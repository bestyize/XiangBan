// @vitest-environment happy-dom
import { describe, expect, it, vi } from 'vitest';
import { defineComponent } from 'vue';
import { flushPromises, mount } from '@vue/test-utils';
import MediaPreview from './MediaPreview.vue';

describe('local HTML wallpaper card preview', () => {
  it('runs the sandbox only while hovered or keyboard focused', async () => {
    vi.stubGlobal('IntersectionObserver', class { observe() {} disconnect() {} });
    const view = mount(defineComponent({
      components: { MediaPreview },
      template: '<button class="wallpaper-card"><MediaPreview kind="web" source="wallpaper-asset://localhost/hash" poster="" title="桌面小星球" /></button>',
    }));
    try {
      await flushPromises();
      const card = view.get('button');
      const preview = view.get('.media-preview');
      expect(preview.find('iframe').exists()).toBe(false);
      await card.trigger('mouseenter');
      await flushPromises();
      expect(preview.get('iframe').attributes('sandbox')).toBe('allow-scripts');
      await card.trigger('mouseleave');
      await flushPromises();
      expect(preview.find('iframe').exists()).toBe(false);
      await card.trigger('focus');
      await flushPromises();
      expect(preview.find('iframe').exists()).toBe(true);
      await card.trigger('blur');
      await flushPromises();
      expect(preview.find('iframe').exists()).toBe(false);
    } finally {
      view.unmount();
      vi.unstubAllGlobals();
    }
  });
});
