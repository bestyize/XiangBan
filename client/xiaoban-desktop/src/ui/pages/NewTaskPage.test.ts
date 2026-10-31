// @vitest-environment happy-dom
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { reactive } from 'vue';
import NewTaskPage from './NewTaskPage.vue';
import { createChatStore, type ChatDeps } from '../chat/chat-store.ts';
import type { ControlStore } from '../control-types.ts';
import type { Overview } from '../chat/types.ts';

vi.mock('../../bridge.ts', async importOriginal => ({ ...await importOriginal<typeof import('../../bridge.ts')>(), invoke: vi.fn() }));
import { defaults, invoke as bridgeInvoke } from '../../bridge.ts';

const overview = (patch: Partial<Overview> = {}): Overview => ({ configured: true, sessions: [], running: [], workspace: '/Users/me/小伴/工作区', permissionMode: 'ask', hideToolCalls: false, ...patch });
let wrapper: VueWrapper | undefined;

async function open(patch: Partial<Overview> = {}, page = 'new-task', handed: { draft: string | null; attachments: unknown[] | null } = { draft: null, attachments: null }) {
  const calls: string[] = [];
  const invoke = vi.fn(async (command: string, args?: Record<string, unknown>) => {
    calls.push(command);
    if (command === 'chat_overview') return overview(patch) as never;
    if (command === 'chat_context') return { used: 0, limit: 256_000, canCompact: false } as never;
    if (command === 'get_ai_config') return { model: 'm', activeProvider: 'demo', providers: [{ id: 'demo', name: 'DeepSeek', models: [{ id: 'm', name: 'deepseek-v4-pro' }] }] } as never;
    if (command === 'chat_set_permission_mode') return (args?.mode ?? 'ask') as never;
    if (command === 'chat_new_session') return { id: 's1', title: '新对话', createdAt: 1, updatedAt: 1 } as never;
    if (command === 'chat_send') return { id: 's1', title: '整理周报', createdAt: 1, updatedAt: 1 } as never;
    throw new Error(`unexpected ${command}`);
  });
  vi.mocked(bridgeInvoke).mockImplementation(invoke as never);
  const deps: ChatDeps = { desktop: true, invoke, notify: vi.fn(), listen: async () => () => {} };
  const chat = createChatStore(deps);
  const asr = reactive({ ready: true, microphoneSupported: true, saving: false, recordingRequested: false, listening: false, preparing: false, recognizing: false, enabled: true, error: '', init: vi.fn(async () => {}), activate: vi.fn(async () => {}), deactivate: vi.fn(), finish: vi.fn(async () => {}) });
  const store = reactive({ desktop: true, chat, asr, page, settingsTab: 'general', creationDraft: handed.draft, creationAttachments: handed.attachments, navigate: vi.fn(), notify: vi.fn(), openSettings: vi.fn() }) as unknown as ControlStore;
  wrapper = mount(NewTaskPage, { props: { store }, attachTo: document.body });
  await flushPromises();
  return { store, chat, asr, calls };
}
const q = <T extends HTMLElement = HTMLElement>(id: string) => wrapper!.element.querySelector(`[data-testid="${id}"]`) as T | null;
const textarea = () => wrapper!.element.querySelector('[data-testid="chat-input"] textarea') as HTMLTextAreaElement;
async function click(id: string) { q(id)!.click(); await flushPromises(); }

beforeEach(() => { vi.resetAllMocks(); localStorage.clear(); });
afterEach(() => { wrapper?.unmount(); wrapper = undefined; vi.useRealTimers(); });

describe('新建任务', () => {
  it('immediately hides and restores the shared composer pet when settings change', async () => {
    const { store } = await open();
    expect(wrapper!.findComponent({ name: 'ChatComposerPet' }).exists()).toBe(true);
    store.settings = { ...defaults, chatComposerPetEnabled: false };
    await flushPromises();
    expect(wrapper!.findComponent({ name: 'ChatComposerPet' }).exists()).toBe(false);
    store.settings = { ...defaults, chatComposerPetEnabled: true };
    await flushPromises();
    expect(wrapper!.findComponent({ name: 'ChatComposerPet' }).exists()).toBe(true);
  });
  it('greets by the time of day', async () => {
    vi.useFakeTimers({ toFake: ['Date'] });
    vi.setSystemTime(new Date(2026, 9, 22, 15, 0));
    await open();
    expect(q('new-task-greeting')!.textContent).toBe('下午好，今天让小伴帮你做点什么？');
    wrapper!.unmount();
    vi.setSystemTime(new Date(2026, 9, 22, 9, 0));
    await open();
    expect(q('new-task-greeting')!.textContent).toContain('上午好');
    wrapper!.unmount();
    vi.setSystemTime(new Date(2026, 9, 22, 21, 0));
    await open();
    expect(q('new-task-greeting')!.textContent).toContain('晚上好');
  });

  it('takes a file handed over from 资料库 into the box, unsent, along with its prompt', async () => {
    const file = { name: '预算.xlsx', size: 286_000, mime: 'application/vnd', dataUrl: 'data:application/vnd;base64,AA==' };
    const { store, calls } = await open({}, 'new-task', { draft: '请参考资料库里的「预算.xlsx」：', attachments: [file] });
    expect(textarea().value).toBe('请参考资料库里的「预算.xlsx」：');
    expect(q('chat-attachments')!.textContent).toContain('预算.xlsx');
    expect(store.creationAttachments).toBeNull();
    expect(calls).not.toContain('chat_send');
  });

  it('switches the quick starts with the scene and puts one in the box without sending', async () => {
    const h = await open();
    expect(q('new-task-scene-work')!.getAttribute('aria-selected')).toBe('true');
    expect(q('new-task-chip-写文档')).not.toBeNull();
    await click('new-task-scene-study');
    expect(q('new-task-chip-写文档')).toBeNull();
    expect(q('new-task-chip-翻译')).not.toBeNull();
    expect(localStorage.getItem('xiaoban.new-task.scene')).toBe('study');
    await click('new-task-try-0');
    expect(textarea().value).toBe('用费曼学习法给我讲讲什么是熵');
    expect(h.calls).not.toContain('chat_send');
    expect(q<HTMLButtonElement>('chat-send')!.disabled).toBe(false);
  });

  it('shares the composer: workspace and permission menus, an empty context ring and the model', async () => {
    await open();
    expect(q('chat-workspace-menu')).not.toBeNull();
    expect(q('chat-permission-toggle')!.textContent).toContain('默认权限');
    expect(q('chat-context')!.tagName).toBe('SPAN');
    expect(q('chat-context')!.getAttribute('aria-label')).toBe('开始对话后显示上下文用量');
    expect(q('chat-workbench-model')!.textContent).toContain('deepseek-v4-pro');
    expect(q('chat-workbench-model')!.textContent).toContain('DE');
    expect(wrapper!.text()).toContain('内容由 AI 生成，请核对重要信息');
  });

  it('turns the permission button red in full trust and links to the safety settings', async () => {
    const h = await open();
    await click('chat-permission-toggle');
    await click('perm-auto');
    expect(h.chat.permissionMode).toBe('auto');
    expect(q('chat-permission-toggle')!.textContent).toContain('完全信任');
    expect(q('chat-permission-toggle')!.classList.contains('is-full')).toBe(true);
    await click('chat-permission-toggle');
    await click('perm-settings');
    expect(h.store.openSettings).toHaveBeenCalledWith('safety');
  });

  it('allows voice input on the new-task page, not only on the chat page', async () => {
    const h = await open();
    expect(q<HTMLButtonElement>('chat-asr-toggle')!.disabled).toBe(false);
    await click('chat-asr-toggle');
    expect(h.asr.activate).toHaveBeenCalled();
  });

  it('keeps voice off while the page is not the one shown', async () => {
    const h = await open({}, 'chat');
    expect(q<HTMLButtonElement>('chat-asr-toggle')!.disabled).toBe(true);
    await click('chat-asr-toggle');
    expect(h.asr.activate).not.toHaveBeenCalled();
  });

  it('starts the conversation with the first message', async () => {
    const h = await open();
    const t = textarea();
    t.value = '整理周报';
    t.dispatchEvent(new Event('input', { bubbles: true }));
    await flushPromises();
    await click('chat-send');
    expect(h.calls).toContain('chat_new_session');
    expect(h.calls).toContain('chat_send');
    expect(h.chat.currentId).toBe('s1');
  });

  it('asks to set up a model before anything can be sent', async () => {
    const h = await open({ configured: false });
    expect(q('new-task-unconfigured')).not.toBeNull();
    expect(textarea().disabled).toBe(true);
    await click('new-task-go-settings');
    expect(h.store.openSettings).toHaveBeenCalledWith('ai');
  });
});

describe('a summoned expert', () => {
  it('replaces the greeting and scenes with its own introduction and three starter prompts, and fills the box from them', async () => {
    const { chat } = await open();
    chat.pendingExpert = { plugin: 'accounting-supervisor', name: 'AI 会计主管', profession: '会计主管', description: '聚焦会计核算与财务分析', tags: ['税务核算', '财务分析'], quickPrompts: ['请审核这笔分录', '编制本月报表', '做比率分析', '多余的第四条'] };
    await flushPromises();
    expect(q('new-task-expert')?.textContent).toContain('AI 会计主管');
    expect(q('new-task-expert')?.textContent).toContain('会计主管');
    expect(q('new-task-expert')?.textContent).toContain('税务核算');
    expect(q('new-task-greeting')).toBeNull();
    expect(q('new-task-scene-work')).toBeNull();
    expect(wrapper!.element.querySelectorAll('[data-testid^="new-task-try-"]')).toHaveLength(3);
    await click('new-task-try-0');
    expect(textarea().value).toBe('请审核这笔分录');
    // Cancelling the pick brings the ordinary page back.
    await click('chat-expert-clear');
    expect(q('new-task-greeting')).not.toBeNull();
    expect(q('new-task-scene-work')).not.toBeNull();
  });
});
