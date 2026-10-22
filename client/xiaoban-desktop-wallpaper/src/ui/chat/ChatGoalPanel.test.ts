// @vitest-environment happy-dom
import { flushPromises, mount } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import ChatGoalPanel from './ChatGoalPanel.vue';

vi.mock('../../bridge.ts', () => ({ invoke: vi.fn() }));
import { invoke } from '../../bridge.ts';

const baseProps = {
  sessionId: 's1', running: false, configured: true, desktop: true, permissionMode: 'ask' as const,
};
const contract = {
  outcome: '修复构建', verification: '测试通过', constraints: '不改 API', boundaries: 'src', stopWhen: '需密钥',
};
const goal = (state: string) => ({
  version: 1, sessionId: 's1', contract, state,
  attempts: 1, lastRunId: 'r-1', updatedAtMs: 1, note: '',
});

const wrappers: ReturnType<typeof mount>[] = [];
function open(props: Partial<typeof baseProps> = {}) {
  const wrapper = mount(ChatGoalPanel, { props: { ...baseProps, ...props } });
  wrappers.push(wrapper);
  return wrapper;
}

beforeEach(() => vi.resetAllMocks());
afterEach(() => { wrappers.splice(0).forEach(w => w.unmount()); });

describe('durable goal workbench', () => {
  it('requires a saved contract and never presents model completion as verified success', async () => {
    vi.mocked(invoke).mockImplementation(async command => {
      if (command === 'chat_goal_get') return null as never;
      if (command === 'chat_goal_set') return { ...goal('draft'), attempts: 0, lastRunId: null } as never;
      if (command === 'chat_goal_start') return goal('running') as never;
      throw new Error('unexpected '+ command);
    });
    const wrapper = open();
    await flushPromises();
    expect(wrapper.find('[data-testid="goal-start"]').exists()).toBe(false);
    await wrapper.get('[data-testid="goal-outcome"] textarea').setValue('修复构建');
    await wrapper.findAll('button').find(b => b.text().includes('保存目标'))!.trigger('click');
    await flushPromises();
    expect(vi.mocked(invoke)).toHaveBeenCalledWith('chat_goal_set', expect.objectContaining({
      sessionId: 's1', contract: expect.objectContaining({ outcome: '修复构建' }),
    }));
    await wrapper.get('[data-testid="goal-start"]').trigger('click');
    await flushPromises();
    expect(wrapper.get('[data-testid="goal-state"]').text()).toBe('执行中');
    expect(wrapper.text()).not.toContain('已完成');
  });

  it('blocks uncertain continuation until a human records the reconciliation evidence', async () => {
    vi.mocked(invoke).mockImplementation(async command => {
      if (command === 'chat_goal_get') return { ...goal('needs_review'), note:'已有写入可能成功，禁止重放' } as never;
      if (command === 'chat_goal_acknowledge') return { ...goal('paused'), note:'检查了外部状态' } as never;
      throw new Error('unexpected '+ command);
    });
    const wrapper = open();
    await flushPromises();
    expect(wrapper.get('[data-testid="goal-state"]').text()).toContain('需人工核验');
    expect(wrapper.text()).not.toContain('继续目标（新回合）');
    await wrapper.get('[data-testid="goal-evidence"] textarea').setValue('检查了外部状态');
    await wrapper.findAll('button').find(b => b.text().includes('已核验中断现场'))!.trigger('click');
    await flushPromises();
    expect(vi.mocked(invoke)).toHaveBeenCalledWith('chat_goal_acknowledge', {
      sessionId: 's1', evidence: '检查了外部状态',
    });
    expect(wrapper.get('[data-testid="goal-state"]').text()).toContain('已暂停');
    expect(wrapper.text()).toContain('继续目标（新回合）');
  });
});
