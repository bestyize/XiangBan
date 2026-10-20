// @vitest-environment happy-dom
import { flushPromises, mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import ChatActivity from './ChatActivity.vue';
import type { ToolCall } from './replies.ts';

const call = (id: string, status: ToolCall['status'] = 'done'): ToolCall => ({ kind: 'tool', id, name: 'shell', arguments: '{"command":"ls"}', output: '文件清单', status });

describe('compact execution progress', () => {
  it('shows one live operation, retains parallel work, and keeps errors visible after completion', async () => {
    const wrapper = mount(ChatActivity, { props: { calls: [call('slow', 'running'), call('fast')], active: true } });
    expect(wrapper.findAll('[data-testid^="tool-"]')).toHaveLength(1);
    expect(wrapper.find('[data-testid="tool-slow"]').exists()).toBe(true);
    expect(wrapper.findAll('pre')).toHaveLength(0);
    await wrapper.setProps({ active: false, calls: [call('slow', 'error'), call('fast')] });
    expect(wrapper.find('[data-testid="activity-live"]').exists()).toBe(false);
    expect(wrapper.find('[data-testid="activity-toggle"]').text()).toContain('1 次异常');
    await wrapper.find('button').trigger('click');
    expect(wrapper.findAll('details')).toHaveLength(2);
    wrapper.unmount();
  });

  it('does not drag the reader to the bottom while inspecting history or an open result', async () => {
    const wrapper = mount(ChatActivity, { props: { calls: [call('one')], active: true } });
    await wrapper.find('button').trigger('click');
    await flushPromises();
    const history = wrapper.get('[data-testid="activity-history"]').element as HTMLElement;
    Object.defineProperty(history, 'scrollHeight', { value: 900, configurable: true });
    Object.defineProperty(history, 'clientHeight', { value: 200, configurable: true });
    history.scrollTop = 20;
    await wrapper.get('[data-testid="activity-history"]').trigger('scroll');
    await wrapper.setProps({ calls: [call('one'), call('two', 'running')] });
    await flushPromises();
    expect(history.scrollTop).toBe(20);

    // Return to the end, then open a result; output updates must not interrupt reading it.
    history.scrollTop = 700;
    await wrapper.get('[data-testid="activity-history"]').trigger('scroll');
    (wrapper.get('details').element as HTMLDetailsElement).open = true;
    await wrapper.setProps({ calls: [call('one'), { ...call('two'), output: '新结果'.repeat(200) }] });
    await flushPromises();
    expect(history.scrollTop).toBe(700);
    wrapper.unmount();
  });
});
