// @vitest-environment happy-dom
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import { afterEach, describe, expect, it } from 'vitest';
import ChatContextRing from './ChatContextRing.vue';

const base = { used: 12_300, limit: 48_000, canCompact: true, compacting: false, busy: false, disabled: false };
let wrapper: VueWrapper | undefined;

const mountRing = (props: Partial<typeof base> = {}) => (wrapper = mount(ChatContextRing, { props: { ...base, ...props }, attachTo: document.body }));
const body = <T extends HTMLElement = HTMLElement>(testid: string) => document.body.querySelector<T>(`[data-testid="${testid}"]`);
const ring = () => wrapper!.find('[data-testid="chat-context"]');
async function openPopover() {
  await ring().trigger('click');
  await flushPromises();
}

afterEach(() => { wrapper?.unmount(); wrapper = undefined; document.body.innerHTML = ''; });

describe('the context ring', () => {
  it('is a ring filled to the share of the context in use, and says so to a screen reader', () => {
    mountRing();
    expect(ring().attributes('aria-label')).toBe('上下文已用 26%，点击查看');
    const circumference = 2 * Math.PI * 8;
    expect(wrapper!.find('[data-testid="chat-context-fill"]').attributes('stroke-dasharray')).toBe(`${((circumference * 26) / 100).toFixed(2)} ${circumference.toFixed(2)}`);
  });

  it('turns amber from 70% and red from 90%', async () => {
    mountRing({ used: 20_000 });
    expect(ring().classes()).toContain('ok');
    await wrapper!.setProps({ used: 36_000 });
    expect(ring().classes()).toContain('warn');
    await wrapper!.setProps({ used: 44_000 });
    expect(ring().classes()).toContain('full');
  });

  it('stays a full ring when the context is over the limit', () => {
    mountRing({ used: 70_000 });
    expect(ring().attributes('aria-label')).toContain('100%');
    expect(ring().classes()).toContain('full');
  });

  it('opens a popover with exactly two rows: the numbers with a bar, and the compact button', async () => {
    mountRing();
    expect(body('chat-context-pop')).toBeNull();
    await openPopover();
    const pop = body('chat-context-pop')!;
    expect(pop.children).toHaveLength(2);
    expect(body('ctx-numbers')!.textContent).toBe('12.3K / 48K');
    expect(body('ctx-bar')).not.toBeNull();
    expect(body('ctx-compact')!.textContent).toContain('智能压缩');
    expect(body<HTMLButtonElement>('ctx-compact')!.disabled).toBe(false);
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

  it('writes a short limit in full-size numbers too', async () => {
    mountRing({ used: 950, limit: 1_000_000 });
    await openPopover();
    expect(body('ctx-numbers')!.textContent).toBe('950 / 1M');
  });
});
