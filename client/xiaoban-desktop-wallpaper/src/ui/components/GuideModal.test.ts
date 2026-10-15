// @vitest-environment happy-dom
import { mount, flushPromises } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import GuideModal from './GuideModal.vue';

describe('GuideModal', () => {
  it('renders cards and emits select-prompt when trying a prompt', async () => {
    const wrapper = mount(GuideModal, {
      props: { show: true, desktop: true },
      attachTo: document.body,
    });
    await flushPromises();

    expect(document.body.textContent).toContain('小伴桌面 · AI 玩法指南');
    expect(document.body.textContent).toContain('极简赛博朋克时钟');

    // Click "一键试试" on the first card
    const tryButtons = [...document.body.querySelectorAll('button')].filter(b => b.textContent?.includes('一键试试'));
    expect(tryButtons.length).toBeGreaterThan(0);
    tryButtons[0].click();
    await flushPromises();

    expect(wrapper.emitted('select-prompt')).toBeTruthy();
    expect(wrapper.emitted('select-prompt')![0][0]).toContain('极简赛博朋克风格的时钟挂件');
    expect(wrapper.emitted('update:show')![0][0]).toBe(false);

    wrapper.unmount();
  });
});
