// @vitest-environment happy-dom
import { mount, flushPromises } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import CompanionReplyCard from './CompanionReplyCard.vue';
import type { CompanionReply } from './dialogue.ts';
import corpus from './corpus.json';
const host = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock('../bridge.ts', () => ({ desktop: true, invoke: host.invoke }));
const initial: CompanionReply = { id: 41, petName: '小葵', text: '先歇一会儿，我在这里。', choices: [{ id: 'tired', label: '有点累了' }], model: false };
beforeEach(() => { host.invoke.mockReset(); host.invoke.mockResolvedValue(null); });
afterEach(() => { vi.useRealTimers(); document.body.replaceChildren(); });

describe('companion conversation card', () => {
  it('shows an instant local reply and retains it if the model fails', async () => {
    host.invoke.mockRejectedValue(new Error('offline'));
    const wrapper = mount(CompanionReplyCard, { props: { initial } }); await flushPromises();
    expect(wrapper.get('.reply-text').text()).toBe(initial.text);
    expect(host.invoke).toHaveBeenCalledWith('companion_dialogue_model', { id: 41 });
    expect(host.invoke).not.toHaveBeenCalledWith('companion_dialogue_continue', expect.anything());
    wrapper.unmount();
  });
  it('accepts a model reply only for the current interaction', async () => {
    let finish!: (value: CompanionReply) => void;
    host.invoke.mockImplementation((command: string) => command === 'companion_dialogue_model' ? new Promise(resolve => { finish = resolve; }) : Promise.resolve(null));
    const wrapper = mount(CompanionReplyCard, { props: { initial } });
    expect(wrapper.get('.reply-text').text()).toBe(initial.text);
    await wrapper.setProps({ initial: { ...initial, id: 42, text: '新的这一句' } });
    finish({ ...initial, model: true, text: '旧的模型回复' }); await flushPromises();
    expect(wrapper.get('.reply-text').text()).toBe('新的这一句');
    wrapper.unmount();
  });
  it('continues a selected local topic and only opens a session on explicit request', async () => {
    host.invoke.mockImplementation(async (command: string) => command === 'companion_dialogue_choose' ? { ...initial, id: 42, text: '那我们先歇一下。' } : null);
    const wrapper = mount(CompanionReplyCard, { props: { initial } }); await flushPromises();
    await wrapper.get('[data-testid="companion-choice-tired"]').trigger('click'); await flushPromises();
    expect(host.invoke).toHaveBeenCalledWith('companion_dialogue_choose', { id: 41, choice: 'tired' });
    expect(wrapper.get('.reply-text').text()).toContain('先歇一下');
    expect(host.invoke.mock.calls.some(c => c[0] === 'companion_dialogue_continue')).toBe(false);
    await wrapper.get('[data-testid="companion-continue"]').trigger('click'); await flushPromises();
    expect(host.invoke).toHaveBeenCalledWith('companion_dialogue_continue', { id: 42 });
    expect(wrapper.emitted('close')).toHaveLength(1); wrapper.unmount();
  });
  it('merges duplicate handoff clicks and preserves the card when opening fails', async () => {
    let fail!: (reason: Error) => void;
    host.invoke.mockImplementation((command: string) => command === 'companion_dialogue_continue' ? new Promise((_, reject) => { fail = reject; }) : Promise.resolve(null));
    const wrapper = mount(CompanionReplyCard, { props: { initial } }); await flushPromises();
    await wrapper.get('[data-testid="companion-continue"]').trigger('click');
    await wrapper.get('[data-testid="companion-continue"]').trigger('click');
    expect(host.invoke.mock.calls.filter(c => c[0] === 'companion_dialogue_continue')).toHaveLength(1);
    fail(new Error('cannot open')); await flushPromises();
    expect(wrapper.get('[role="alert"]').text()).toContain('再试一次');
    expect(wrapper.emitted('close')).toBeUndefined(); wrapper.unmount();
  });
  it('pauses expiry while the user reads or focuses the card', async () => {
    vi.useFakeTimers();
    const wrapper = mount(CompanionReplyCard, { props: { initial } }); await flushPromises();
    await wrapper.trigger('pointerenter'); vi.advanceTimersByTime(30_000);
    expect(wrapper.emitted('close')).toBeUndefined();
    await wrapper.trigger('pointerleave'); vi.advanceTimersByTime(12_000); await flushPromises();
    expect(wrapper.emitted('close')).toHaveLength(1);
    expect(host.invoke).toHaveBeenCalledWith('companion_dialogue_close', { id: 41 }); wrapper.unmount();
  });
  it('ships distinct authored replies and every local branch has content', () => {
    const lines = Object.values(corpus.scenes).flat();
    expect(lines).toHaveLength(336); expect(new Set(lines).size).toBe(336);
    for (const scene of ['first', 'return', 'repeat', 'tired', 'happy', 'quiet', 'busy', 'plan', 'play', 'sleep']) expect(corpus.scenes[scene as keyof typeof corpus.scenes].length).toBeGreaterThan(10);
    expect(lines.every(s => s.length < 110 && s.trim().length > 0)).toBe(true);
  });
});
