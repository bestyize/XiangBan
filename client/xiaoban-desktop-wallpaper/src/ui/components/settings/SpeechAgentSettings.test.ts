// @vitest-environment happy-dom
import { flushPromises, mount } from '@vue/test-utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import SpeechAgentSettings from './SpeechAgentSettings.vue';
import { invoke, listen } from '../../../bridge.ts';

vi.mock('../../../bridge.ts', async importOriginal => ({
  ...(await importOriginal<typeof import('../../../bridge.ts')>()),
  desktop: true,
  invoke: vi.fn(),
  listen: vi.fn(async () => () => {}),
}));

const ipc = vi.mocked(invoke);

describe('SpeechAgentSettings', () => {
  beforeEach(() => {
    ipc.mockReset();
    vi.mocked(listen).mockReset().mockResolvedValue(() => {});
  });

  it('does not attach listeners or polling after the settings tab is closed during loading', async () => {
    let finish!: (value: unknown) => void;
    ipc.mockImplementation(command => command === 'speech_agent_env'
      ? new Promise(resolve => { finish = resolve; })
      : Promise.resolve(null));
    const interval = vi.spyOn(globalThis, 'setInterval');
    const wrapper = mount(SpeechAgentSettings);
    wrapper.unmount();
    finish(null);
    await flushPromises();
    expect(listen).not.toHaveBeenCalled();
    expect(interval).not.toHaveBeenCalled();
    interval.mockRestore();
  });

  it('removes a listener that finishes attaching after the tab is closed', async () => {
    let finish!: (unlisten: () => void) => void;
    const unlisten = vi.fn();
    ipc.mockImplementation(command => Promise.resolve(
      command === 'speech_agent_task_list' || command === 'speech_agent_install_guides' ? [] : null,
    ));
    vi.mocked(listen).mockImplementation(() => new Promise(resolve => { finish = resolve; }));
    const wrapper = mount(SpeechAgentSettings);
    await flushPromises();
    wrapper.unmount();
    finish(unlisten);
    await flushPromises();
    expect(unlisten).toHaveBeenCalledOnce();
  });

  it('renders environment status and allows hooks install and voice service toggle', async () => {
    let hooksInstalled = false;
    let voiceEnabled = false;

    ipc.mockImplementation(async (command: string, args?: Record<string, unknown>) => {
      if (command === 'speech_agent_env') {
        return {
          os: 'win32 10.0.26300 x64',
          node: { name: 'node', installed: true, path: '/bin/node', version: 'v24.14.0', logged_in: null, notes: [] },
          git: { name: 'git', installed: true, path: '/bin/git', version: '2.53.0', logged_in: null, notes: [] },
          claude: { name: 'claude', installed: true, path: '/bin/claude', version: '2.1.285', logged_in: true, notes: [] },
          codex: { name: 'codex', installed: true, path: '/bin/codex', version: '0.159.2', logged_in: true, notes: [] },
          desktopApps: [{ name: 'Claude', running: true }, { name: 'ChatGPT', running: true }, { name: 'Codex', running: false }],
          readyForSpeechCoding: true,
        };
      }
      if (command === 'speech_agent_install_guides') {
        return [
          {
            tool: 'claude',
            officialCommand: 'npm i -g @anthropic-ai/claude-code',
            mirrorCommand: 'npm i -g @anthropic-ai/claude-code --registry=https://registry.npmmirror.com',
            loginCommand: 'claude /login',
            documentationUrl: 'https://docs.anthropic.com',
          },
        ];
      }
      if (command === 'speech_agent_hooks_status') {
        return {
          scope: 'User',
          settingsPath: '/home/user/.claude/settings.json',
          fileExists: true,
          isInstalled: hooksInstalled,
          events: hooksInstalled ? ['SessionStart', 'Stop', 'PermissionRequest'] : [],
          otherHooks: false,
        };
      }
      if (command === 'speech_agent_hooks_install') {
        hooksInstalled = true;
        return {
          status: {
            scope: 'User',
            settingsPath: '/home/user/.claude/settings.json',
            fileExists: true,
            isInstalled: true,
            events: ['SessionStart', 'Stop', 'PermissionRequest'],
            otherHooks: false,
          },
          backup: '/home/user/.claude/settings.json.backup',
          hookExecutable: '/bin/xiaoban-hook',
        };
      }
      if (command === 'voice_service_status') {
        return {
          enabled: voiceEnabled,
          running: voiceEnabled,
          speechBroadcast: true,
          port: 58139,
          token: 'token123',
          baseUrl: 'http://127.0.0.1:58139/v1',
          mcpUrl: 'http://127.0.0.1:58139/mcp',
          mcpConfigSnippet: { mcpServers: { 'xiaoban-voice': { url: 'http://127.0.0.1:58139/mcp' } } },
        };
      }
      if (command === 'voice_service_set_enabled') {
        voiceEnabled = Boolean(args?.enabled);
        return {
          enabled: voiceEnabled,
          running: voiceEnabled,
          speechBroadcast: true,
          port: 58139,
          token: 'token123',
          baseUrl: 'http://127.0.0.1:58139/v1',
          mcpUrl: 'http://127.0.0.1:58139/mcp',
          mcpConfigSnippet: { mcpServers: { 'xiaoban-voice': { url: 'http://127.0.0.1:58139/mcp' } } },
        };
      }
      if (command === 'speech_agent_task_list') {
        return [
          {
            taskId: 'del-1',
            agent: 'claude',
            prompt: '修复测试用例',
            workspace: '/code',
            startedAtMs: 1000,
            status: 'running',
            recentOutput: '正在执行 cargo test...',
            pendingApproval: null,
            sessionId: 'uuid-1',
          },
        ];
      }
      throw new Error(`unexpected command ${command}`);
    });

    const wrapper = mount(SpeechAgentSettings);
    await flushPromises();

    expect(wrapper.text()).toContain('Claude Code CLI');
    expect(wrapper.text()).toContain('2.1.285');
    expect(wrapper.text()).toContain('OpenAI Codex CLI');
    expect(wrapper.text()).toContain('del-1');
    expect(wrapper.text()).toContain('修复测试用例');

    // 安装 hooks
    const installBtn = wrapper.get('[data-testid="hooks-install-btn"]');
    await installBtn.trigger('click');
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('speech_agent_hooks_install', { workspace: null });
    expect(wrapper.text()).toContain('已成功安装 Claude Code hooks');

    // 启用语音服务
    const voiceToggle = wrapper.get('[data-testid="voice-service-toggle"]');
    await voiceToggle.trigger('click');
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('voice_service_set_enabled', { enabled: true, speechBroadcast: null });
    wrapper.unmount();
  });
});
