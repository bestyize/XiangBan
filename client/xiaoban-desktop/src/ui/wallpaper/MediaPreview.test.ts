// @vitest-environment happy-dom
import { flushPromises, mount } from '@vue/test-utils';
import { afterEach, expect, it, vi } from 'vitest';
import { invoke } from '../../bridge.ts';
import MediaPreview from './MediaPreview.vue';

const spies = vi.hoisted(() => ({ cleanup: vi.fn() }));
vi.mock('../../wallpaper-media.ts', async original => ({
  ...await original<typeof import('../../wallpaper-media.ts')>(),
  bindMedia: () => spies.cleanup,
}));
vi.mock('../../bridge.ts', () => ({ invoke: vi.fn() }));
afterEach(() => vi.unstubAllGlobals());

it('shows the media without desktop particle effects', async () => {
  const wrapper = mount(MediaPreview, { props: { kind: 'image', source: '', poster: '/test.png', title: 'test' } });
  await flushPromises();
  expect(wrapper.find('.media-preview').exists()).toBe(true);
  expect(wrapper.find('canvas').exists()).toBe(false);
  wrapper.unmount();
  expect(spies.cleanup).toHaveBeenCalled();
});

it('captures a visible local web wallpaper and uses the result as its idle poster', async () => {
  let showCover!: () => void;
  vi.stubGlobal('IntersectionObserver', class {
    constructor(callback: IntersectionObserverCallback) {
      showCover = () => callback([{ isIntersecting: true } as IntersectionObserverEntry], this as unknown as IntersectionObserver);
    }
    observe() {}
    disconnect() {}
  });
  const poster = 'http://wallpaper-asset.localhost/cover/test';
  vi.mocked(invoke).mockResolvedValue(poster);
  const wrapper = mount(MediaPreview, {
    props: { kind: 'web', source: 'http://wallpaper-asset.localhost/test', poster: '', title: '星空', coverId: 'local-test' },
  });
  await flushPromises();
  expect(wrapper.find('iframe').exists()).toBe(false);
  expect(invoke).not.toHaveBeenCalled();

  showCover();
  await flushPromises();
  expect(invoke).toHaveBeenCalledWith('wallpaper_ensure_cover', { id: 'local-test' });
  expect(wrapper.get('img').attributes('src')).toBe(poster);
  wrapper.unmount();
});
