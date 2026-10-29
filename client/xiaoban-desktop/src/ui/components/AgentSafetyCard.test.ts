// @vitest-environment happy-dom
import { flushPromises, mount } from '@vue/test-utils';
import { NInputNumber, NSwitch } from 'naive-ui';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { reactive } from 'vue';
import AgentSafetyCard from './AgentSafetyCard.vue';
import type { ControlStore } from '../control-types.ts';
import { invoke } from '../../bridge.ts';

vi.mock('../../bridge.ts', async importOriginal => ({ ...await importOriginal<typeof import('../../bridge.ts')>(), invoke: vi.fn() }));
const ipc = vi.mocked(invoke);

function view(over: Record<string, unknown> = {}) {
  return {
    sandboxAvailable: true, commandSandbox: false, sandboxNetwork: true, hooks: '', hooksPath: '/data/chat/hooks.json',
    grants: [{ session: 's1', title: '整理笔记', tool: 'shell', kind: 'npm test' }, { session: 's1', title: '整理笔记', tool: 'write_file', kind: null }],
    ...over,
  };
}

async function open(settings = view()) {
  const store = reactive({ desktop: true, notify: vi.fn() }) as unknown as ControlStore;
  ipc.mockImplementation(async (command: string) => {
    if (command === 'chat_agent_settings') return settings as never;
    if (command === 'chat_save_hooks') return 1 as never;
    return undefined as never;
  });
  const wrapper = mount(AgentSafetyCard, { props: { store } });
  await flushPromises();
  return { wrapper, store };
}

describe('安全与权限', () => {
  beforeEach(() => ipc.mockReset());

  it('starts with no execution limits and saves the ones the user sets', async () => {
    const { wrapper } = await open();
    const inputs = wrapper.findAllComponents(NInputNumber);
    expect(inputs.map(input => input.props('value'))).toEqual([null, null, null, null]);
    expect(wrapper.get('[data-testid="execution-save"]').attributes('disabled')).toBeDefined();
    inputs[2]!.vm.$emit('update:value', 45);
    inputs[3]!.vm.$emit('update:value', 1_000_000);
    await flushPromises();
    await wrapper.get('[data-testid="execution-save"]').trigger('click');
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('chat_set_execution_limits', { limits: {
      maxTurns: null, maxToolCalls: null, maxMinutes: 45, maxReportedTokens: 1_000_000,
    } });
    wrapper.unmount();
  });

  it('lists saved grants by kind and revokes one', async () => {
    const { wrapper } = await open();
    const rows = wrapper.findAll('[data-testid="grant-row"]');
    expect(rows.map(r => r.text())).toEqual([expect.stringContaining('运行命令：npm test'), expect.stringContaining('写入文件')]);
    expect(rows[0]!.text()).toContain('整理笔记');
    await rows[0]!.get('[data-testid="grant-revoke"]').trigger('click');
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('chat_revoke_grant', { sessionId: 's1', tool: 'shell', kind: 'npm test' });
  });

  it('switches network for the sandbox, or says there is none on this system', async () => {
    const { wrapper } = await open(view({ commandSandbox: true }));
    await wrapper.findAllComponents(NSwitch)[1]!.vm.$emit('update:value', false);
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('chat_set_sandbox_network', { allowed: false });
    const other = await open(view({ sandboxAvailable: false }));
    expect(other.wrapper.find('[data-testid="sandbox-unavailable"]').exists()).toBe(true);
    expect(other.wrapper.find('[data-testid="sandbox-network"]').exists()).toBe(false);
  });

  it('keeps command isolation off by default and lets the user opt in', async () => {
    const { wrapper } = await open();
    expect(wrapper.getComponent(NSwitch).props('value')).toBe(false);
    expect(wrapper.find('[data-testid="sandbox-network"]').exists()).toBe(false);
    expect(wrapper.text()).toContain('完全信任使用当前用户权限');
    await wrapper.getComponent(NSwitch).vm.$emit('update:value', true);
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('chat_set_command_sandbox', { enabled: true });
    wrapper.unmount();
  });

  it('offers the experimental sandbox on Windows and turns it on', async () => {
    const { wrapper } = await open(view({ sandboxAvailable: false, platform: 'windows', windowsSandbox: false }));
    expect(wrapper.find('[data-testid="sandbox-unavailable"]').exists()).toBe(false);
    expect(wrapper.get('[data-testid="windows-sandbox-note"]').text()).toContain('低完整性');
    await wrapper.getComponent(NSwitch).vm.$emit('update:value', true);
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('chat_set_windows_sandbox', { enabled: true });
  });

  it('saves hooks only after a change and reports what the host says', async () => {
    const { wrapper, store } = await open();
    const save = wrapper.get('[data-testid="hooks-save"]');
    expect(save.attributes('disabled')).toBeDefined();
    await wrapper.get('[data-testid="hooks-editor"] textarea').setValue('{"hooks":[{"event":"before_tool","command":"exit 0"}]}');
    await wrapper.get('[data-testid="hooks-save"]').trigger('click');
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('chat_save_hooks', { text: '{"hooks":[{"event":"before_tool","command":"exit 0"}]}' });
    expect(store.notify).toHaveBeenCalledWith(expect.stringContaining('1 条 hook'), 'success');
  });
});
