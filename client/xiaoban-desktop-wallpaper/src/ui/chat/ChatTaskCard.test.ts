// @vitest-environment happy-dom
import { mount } from '@vue/test-utils';
import { expect, it } from 'vitest';
import ChatTaskCard from './ChatTaskCard.vue';
import type { TaskProgress } from './task-progress.ts';

const progress = (): TaskProgress => ({
  plan: { callId: 't1', handoff: '', steps: [
    { content: '读取资料', status: 'completed', evidence: '<script>untrusted()</script>' },
    { content: '生成汇报', status: 'in_progress', evidence: '' },
    { content: '检查交付文件', status: 'pending', evidence: '' },
  ] }, state: 'running', completed: 1, current: 1, message: '按计划继续执行', canResume: false,
});

it('keeps completed-step evidence out of the compact rows, exposes it safely and opens execution logs', async () => {
  const wrapper = mount(ChatTaskCard, { props: { progress: progress(), calls: [], current: true, showLogs: true } });
  expect(wrapper.findAll('.task-step')).toHaveLength(3);
  expect(wrapper.get('[data-testid="task-count"]').text()).toBe('已完成 1 / 3 步');
  expect(wrapper.find('[data-testid="task-step-details"]').exists()).toBe(false);
  await wrapper.get('[data-testid="task-step-0"] button').trigger('click');
  expect(wrapper.get('[data-testid="task-step-details"]').text()).toContain('<script>untrusted()</script>');
  expect(wrapper.find('script').exists()).toBe(false);
  await wrapper.get('[data-testid="task-toggle"]').trigger('click');
  expect(wrapper.find('[data-testid="task-body"]').exists()).toBe(false);
  await wrapper.get('[data-testid="task-toggle"]').trigger('click');
  await wrapper.get('[data-testid="task-logs"]').trigger('click');
  expect(wrapper.emitted('logs')).toHaveLength(1);
  wrapper.unmount();
});

it('opens actionable waiting states but keeps historical cards from controlling the current task', async () => {
  const wrapper = mount(ChatTaskCard, { props: { progress: progress(), calls: [], current: true, showLogs: false } });
  await wrapper.get('[data-testid="task-toggle"]').trigger('click');
  await wrapper.setProps({ progress: { ...progress(), state: 'waiting', message: '请补充汇报受众' } });
  expect(wrapper.get('[data-testid="task-toggle"]').attributes('aria-expanded')).toBe('true');
  await wrapper.get('.task-primary').trigger('click');
  expect(wrapper.emitted('attention')).toHaveLength(1);
  await wrapper.setProps({ progress: { ...progress(), state: 'blocked', canResume: false } });
  expect(wrapper.find('[data-testid="task-resume"]').exists()).toBe(false);
  await wrapper.setProps({ progress: { ...progress(), state: 'paused', canResume: true }, current: false });
  expect(wrapper.find('[data-testid="task-resume"]').exists()).toBe(false);
  expect(wrapper.find('[data-testid="task-adjust"]').exists()).toBe(false);
  expect(wrapper.find('[data-testid="task-logs"]').exists()).toBe(false);
  wrapper.unmount();
});

it('collapses completed tasks on finish and on history restore without an acceptance disclaimer', async () => {
  const complete: TaskProgress = { ...progress(), state: 'completed', completed: 3, current: -1,
    message: '所有步骤已完成', plan: { ...progress().plan, steps: progress().plan.steps.map(step => ({ ...step, status: 'completed' })) } };
  const wrapper = mount(ChatTaskCard, { props: { progress: progress(), calls: [], current: true, showLogs: true } });
  await wrapper.setProps({ progress: complete });
  expect(wrapper.get('[data-testid="task-state"]').text()).toBe('已完成');
  expect(wrapper.find('[data-testid="task-body"]').exists()).toBe(false);
  expect(wrapper.find('[data-testid="task-intervention"]').exists()).toBe(false);
  await wrapper.get('[data-testid="task-toggle"]').trigger('click');
  await wrapper.get('[data-testid="task-step-1"] button').trigger('click');
  expect(wrapper.get('[data-testid="task-step-details"]').text()).toBe('此步骤已完成。');
  wrapper.unmount();
  const restored = mount(ChatTaskCard, { props: { progress: complete, calls: [], current: false, showLogs: true } });
  expect(restored.find('[data-testid="task-body"]').exists()).toBe(false);
  restored.unmount();
});

it.each([['partial', '部分完成'], ['failed', '未完成']] as const)('shows %s distinctly and expands its specific reason on demand', async (state, label) => {
  const wrapper = mount(ChatTaskCard, { props: { progress: { ...progress(), state, message: '写入文件（settings.html）失败。', canResume: true }, calls: [], current: true, showLogs: true, configured: true } });
  expect(wrapper.get('[data-testid="task-state"]').text()).toBe(label);
  await wrapper.get('[data-testid="task-toggle"]').trigger('click');
  expect(wrapper.get('[data-testid="task-intervention"]').text()).toContain('settings.html');
  await wrapper.get('[data-testid="task-resume"]').trigger('click');
  expect(wrapper.emitted('resume')).toHaveLength(1);
  wrapper.unmount();
});
