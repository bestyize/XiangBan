// @vitest-environment happy-dom
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import { NSlider } from 'naive-ui';
import { afterEach, describe, expect, it, vi } from 'vitest';
import ChatContextRing from './ChatContextRing.vue';

const base = { used: 64_000, limit: 256_000, canCompact: true, compacting: false, busy: false, disabled: false };
let wrapper: VueWrapper | undefined;

const mountRing = (props: Partial<typeof base> = {}) => (wrapper = mount(ChatContextRing, { props: { ...base, ...props }, attachTo: document.body }));
const body = <T extends HTMLElement = HTMLElement>(testid: string) => document.body.querySelector<T>(`[data-testid="${testid}"]`);
const ring = () => wrapper!.find('[data-testid="chat-context"]');
const slider = () => wrapper!.findComponent(NSlider);
async function openPopover() {
  await ring().trigger('click');
  await flushPromises();
}
async function slide(value: number) {
  slider().vm.$emit('update:value', value);
  await flushPromises();
}

afterEach(() => { vi.useRealTimers(); wrapper?.unmount(); wrapper = undefined; document.body.innerHTML = ''; });

describe('the context ring', () => {
  it('is a ring filled to the share of the context in use, and says so to a screen reader', () => {
    mountRing();
    expect(ring().attributes('aria-label')).toBe('上下文已用 25%，点击查看');
    const circumference = 2 * Math.PI * 8;
    expect(wrapper!.find('[data-testid="chat-context-fill"]').attributes('stroke-dasharray')).toBe(`${((circumference * 25) / 100).toFixed(2)} ${circumference.toFixed(2)}`);
  });

  it('turns amber from 70% and red from 90%', async () => {
    mountRing({ used: 120_000 });
    expect(ring().classes()).toContain('ok');
    await wrapper!.setProps({ used: 190_000 });
    expect(ring().classes()).toContain('warn');
    await wrapper!.setProps({ used: 235_000 });
    expect(ring().classes()).toContain('full');
  });

  it('stays a full ring when the context is over the limit', () => {
    mountRing({ used: 300_000 });
    expect(ring().attributes('aria-label')).toContain('100%');
    expect(ring().classes()).toContain('full');
  });

  it('opens a popover with three rows: the numbers with a bar, the limit slider, and the compact button', async () => {
    mountRing();
    expect(body('chat-context-pop')).toBeNull();
    await openPopover();
    const pop = body('chat-context-pop')!;
    expect(pop.children).toHaveLength(3);
    expect(pop.children[0].contains(body('ctx-numbers'))).toBe(true);
    expect(pop.children[1].contains(body('ctx-slider'))).toBe(true);
    expect(pop.children[2].contains(body('ctx-compact'))).toBe(true);
    expect(body('ctx-numbers')!.textContent).toBe('64K / 256K');
    expect(body('ctx-bar')).not.toBeNull();
    expect(body('ctx-compact')!.textContent).toContain('智能压缩');
    expect(body<HTMLButtonElement>('ctx-compact')!.disabled).toBe(false);
  });

  it('labels the ends of the slider and says when it compacts by itself', async () => {
    mountRing();
    await openPopover();
    const row = body('ctx-slider')!.parentElement!;
    expect(row.textContent).toContain('256K');
    expect(row.textContent).toContain('1M');
    expect(row.textContent).toContain('用到 90% 自动压缩');
    expect(slider().props()).toMatchObject({ value: 256_000, min: 256_000, max: 1_000_000, step: 8_000 });
  });

  it('asks to compact when the button is pressed', async () => {
    mountRing();
    await openPopover();
    body('ctx-compact')!.click();
    await flushPromises();
    expect(wrapper!.emitted('compact')).toHaveLength(1);
  });

  it('explains why the button is off: answering, too short, or no conversation yet', async () => {
    mountRing({ busy: true });
    await openPopover();
    expect(body<HTMLButtonElement>('ctx-compact')!.disabled).toBe(true);
    expect(body('ctx-compact')!.parentElement!.getAttribute('title')).toBe('回答结束后才能压缩');

    await wrapper!.setProps({ busy: false, canCompact: false });
    expect(body<HTMLButtonElement>('ctx-compact')!.disabled).toBe(true);
    expect(body('ctx-compact')!.parentElement!.getAttribute('title')).toBe('对话还很短，暂时不需要压缩');

    await wrapper!.setProps({ canCompact: true, disabled: true });
    expect(body<HTMLButtonElement>('ctx-compact')!.disabled).toBe(true);
    body('ctx-compact')!.click();
    await flushPromises();
    expect(wrapper!.emitted('compact')).toBeUndefined();
  });

  it('shows progress while compacting and cannot be pressed twice', async () => {
    mountRing({ compacting: true });
    await openPopover();
    expect(body('ctx-compact')!.textContent).toContain('正在压缩…');
    expect(body<HTMLButtonElement>('ctx-compact')!.disabled).toBe(true);
  });

  it('writes small and large numbers short', async () => {
    mountRing({ used: 950, limit: 1_000_000 });
    await openPopover();
    expect(body('ctx-numbers')!.textContent).toBe('950 / 1M');
  });
});

describe('the limit slider', () => {
  it('moves the numbers, the bar and the ring at once, and tells the page once the dragging pauses', async () => {
    mountRing();
    await openPopover();
    vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'] });

    await slide(512_000);
    await slide(520_000);
    expect(body('ctx-numbers')!.textContent).toBe('64K / 520K');
    expect(ring().attributes('aria-label')).toBe('上下文已用 12%，点击查看');
    expect(wrapper!.emitted('resize')).toBeUndefined();

    vi.advanceTimersByTime(349);
    expect(wrapper!.emitted('resize')).toBeUndefined();
    vi.advanceTimersByTime(2);
    expect(wrapper!.emitted('resize')).toEqual([[520_000]]);
  });

  it('keeps following the host after a save, and takes a new limit from it when nothing is pending', async () => {
    mountRing();
    await openPopover();
    await wrapper!.setProps({ limit: 640_000 });
    expect(body('ctx-numbers')!.textContent).toBe('64K / 640K');
    expect(slider().props('value')).toBe(640_000);

    vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'] });
    await slide(768_000);
    await wrapper!.setProps({ limit: 640_000 });
    expect(slider().props('value')).toBe(768_000);
    vi.advanceTimersByTime(400);
    expect(wrapper!.emitted('resize')).toEqual([[768_000]]);
  });

  it('saves a change that is still waiting when the page goes away', async () => {
    mountRing();
    await openPopover();
    vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'] });
    await slide(384_000);
    wrapper!.unmount();
    expect(wrapper!.emitted('resize')).toEqual([[384_000]]);
    vi.advanceTimersByTime(1000);
    expect(wrapper!.emitted('resize')).toHaveLength(1);
  });

  it('shows a limit outside the range at the nearest end instead of off the track', async () => {
    mountRing({ limit: 48_000 });
    await openPopover();
    expect(slider().props('value')).toBe(256_000);
    await wrapper!.setProps({ limit: 5_000_000 });
    expect(slider().props('value')).toBe(1_000_000);
  });

  it('works when there is no conversation to compact yet', async () => {
    mountRing({ disabled: true, canCompact: false });
    await openPopover();
    vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'] });
    await slide(300_000);
    vi.advanceTimersByTime(400);
    expect(wrapper!.emitted('resize')).toEqual([[300_000]]);
  });
});
