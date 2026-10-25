// @vitest-environment happy-dom
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import { reactive } from 'vue';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import { createChatStore } from '../chat/chat-store.ts';
import ChatComposer from './ChatComposer.vue';
import { mentionToken } from '../chat/mentions.ts';

vi.mock('../../bridge.ts', async original => ({ ...await original<typeof import('../../bridge.ts')>(), invoke: vi.fn() }));
const mentions = [
  { kind: 'connector', id: 'repo', name: '代码仓库', description: '查找提交' },
  { kind: 'expert', id: 'reviewer', name: '审查专家', description: '审查代码' },
  { kind: 'skill', id: 'check', name: '质量检查', description: '检查代码' },
];
let wrapper: VueWrapper;
let store: ControlStore;
let sendFails = false;
const host = vi.fn();
beforeEach(() => {
  sendFails = false;
  host.mockReset().mockImplementation(async command => {
    if (command === 'chat_new_session') return { id: 's1', title: '任务', createdAt: 1, updatedAt: 1 };
    if (command === 'chat_send') { if (sendFails) throw '暂不可用'; return { id: 's1', title: '任务', createdAt: 1, updatedAt: 1 }; }
    return { used: 0, limit: 1000, canCompact: false };
  });
  vi.mocked(invoke).mockReset().mockImplementation(async command => {
    if (command === 'ai_mentions') return mentions;
    if (command === 'ai_store_experts_installed') return [{ slug: 'reviewer', plugin: 'reviewer-plugin', name: '审查专家', profession: '代码审查', agent: 'lead' }];
    if (command === 'chat_invocable_skills') return [];
    if (command === 'get_ai_config') return { model: 'test', providers: [] };
    return [];
  });
});
afterEach(() => { wrapper?.unmount(); store?.chat.dispose(); });
async function open() {
  const chat = createChatStore({ desktop: true, invoke: host, listen: vi.fn().mockResolvedValue(() => {}), notify: vi.fn() });
  chat.configured = true;
  store = reactive({ desktop: true, page: 'new-task', chat, creationDraft: null, notify: vi.fn(), navigate: vi.fn() }) as unknown as ControlStore;
  wrapper = mount(ChatComposer, { props: { store, page: 'new-task', pet: false }, global: { stubs: {
    ChatWorkbenchTools: true, ChatComposerModel: true, ChatWorkspaceMenu: true, ChatPermissionMenu: true, ChatContextRing: true, ChatExpertChip: true,
  } } }); await flushPromises();
}
async function type(text: string) { await wrapper.get('textarea').setValue(text); await flushPromises(); }
async function key(key: string, extra = {}) { await wrapper.get('textarea').trigger('keydown', { key, ...extra }); await flushPromises(); }
async function pick(name: string) {
  const item = wrapper.findAll('[role=option]').find(b => b.text().includes(name));
  if (!item) throw new Error(`Missing ${name}`);
  await item.trigger('click'); await flushPromises();
}
describe('composer mentions', () => {
  it('does not match email addresses and accepts Chinese queries', () => {
    expect(mentionToken('me@example.com')).toBeNull(); expect(mentionToken('看 @仓库')).toEqual({ start: 2, query: '仓库' });
  });
  it('selects by keyboard, adds a removable connector tag, sends structured metadata with unchanged text', async () => {
    await open(); await type('请查提交 @');
    expect(invoke).toHaveBeenCalledWith('ai_mentions'); expect(wrapper.get('[role=listbox]').text()).toContain('审查专家');
    await key('Enter');
    expect((wrapper.get('textarea').element as HTMLTextAreaElement).value).toBe('请查提交 ');
    expect(wrapper.text()).toContain('本次优先使用 代码仓库'); expect(host).not.toHaveBeenCalledWith('chat_send', expect.anything());
    await wrapper.get('[data-testid=chat-send]').trigger('click'); await flushPromises();
    expect(host).toHaveBeenCalledWith('chat_send', { sessionId: 's1', text: '请查提交', mentions: [{ kind: 'connector', id: 'repo' }] });
    expect(wrapper.find('.mention-tags').exists()).toBe(false);
  });
  it('deduplicates, filters, and removes selected connector tags', async () => {
    await open(); await type('@仓库'); await pick('代码仓库');
    await type('@repo'); await pick('代码仓库'); expect(wrapper.findAll('.mention-tags button')).toHaveLength(1);
    await wrapper.get('[aria-label="移除连接器 代码仓库"]').trigger('click'); expect(wrapper.find('.mention-tags').exists()).toBe(false);
  });
  it('inserts skills with the slash syntax and summons experts through the existing expert path', async () => {
    await open(); await type('帮我 @质量'); await pick('质量检查');
    expect((wrapper.get('textarea').element as HTMLTextAreaElement).value).toBe('/check 帮我 ');
    await type('@审查'); await pick('审查专家');
    expect(store.chat.pendingExpert).toMatchObject({ plugin: 'reviewer-plugin', name: '审查专家', agent: 'lead' });
    expect(store.navigate).toHaveBeenCalledWith('new-task');
    expect((wrapper.get('textarea').element as HTMLTextAreaElement).value).toBe('');
  });
  it('preserves a rejected message and its connector tags for retry', async () => {
    await open(); await type('查提交 @repo'); await pick('代码仓库'); sendFails = true;
    await wrapper.get('[data-testid=chat-send]').trigger('click'); await flushPromises();
    expect((wrapper.get('textarea').element as HTMLTextAreaElement).value).toBe('查提交');
    expect(wrapper.text()).toContain('本次优先使用 代码仓库');
  });
  it('supports Escape, IME composition, and arrow keys without sending', async () => {
    await open(); await type('@'); await key('ArrowDown');
    expect(wrapper.findAll('[role=option]')[1].attributes('aria-selected')).toBe('true');
    await key('Enter', { isComposing: true }); expect(wrapper.find('[role=listbox]').exists()).toBe(true);
    await key('Escape'); expect(wrapper.find('[role=listbox]').exists()).toBe(false); expect(host).not.toHaveBeenCalledWith('chat_send', expect.anything());
  });
  it('reports inventory errors and does not send when an empty menu is confirmed', async () => {
    await open(); vi.mocked(invoke).mockRejectedValueOnce('读取引用失败'); await type('@');
    expect(store.notify).toHaveBeenCalledWith('读取引用失败', 'error');
    await key('Enter'); expect(host).not.toHaveBeenCalledWith('chat_send', expect.anything());
  });
  it('clears connector choices when a different conversation is selected', async () => {
    await open(); await type('@repo'); await pick('代码仓库'); store.chat.currentId = 'another'; await flushPromises();
    expect(wrapper.find('.mention-tags').exists()).toBe(false);
  });
});
