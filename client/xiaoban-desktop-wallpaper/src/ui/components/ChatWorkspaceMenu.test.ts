// @vitest-environment happy-dom
import { mount, flushPromises } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import { reactive } from 'vue';
import ChatWorkspaceMenu from './ChatWorkspaceMenu.vue';
import type { ChatStore } from '../chat/chat-store.ts';

function setup(answer: 'done' | 'ask' = 'done', picked: string | null = '/p/picked') {
  const chat = reactive({
    workspace: '/default',
    currentWorkspace: '/p/site',
    recentWorkspaces: [
      { path: '/p/site', name: 'site', exists: true, wide: false },
      { path: '/p/blog', name: 'blog', exists: true, wide: false },
      { path: '/p/gone', name: 'gone', exists: false, wide: false },
    ],
    switchWorkspace: vi.fn(async (_path: string, how?: string) => (how ? 'done' : answer)),
    pickWorkspace: vi.fn(async () => picked),
    revealWorkspace: vi.fn(async () => {}),
  }) as unknown as ChatStore;
  const wrapper = mount(ChatWorkspaceMenu, { props: { chat }, attachTo: document.body, global: { stubs: {
    Modal: { name: 'Modal', props: ['show'], template: '<div v-if="show"><slot /></div>' },
  } } });
  return { wrapper, chat };
}
const click = async (wrapper: ReturnType<typeof setup>['wrapper'], id: string) => { await wrapper.get(`[data-testid="${id}"]`).trigger('click'); await flushPromises(); };

describe('the folder menu under the message box', () => {
  it('shows the folder, the recent ones that still exist and the default one', async () => {
    const { wrapper } = setup();
    expect(wrapper.get('[data-testid="chat-workspace-menu"]').text()).toContain('site');
    await click(wrapper, 'chat-workspace-menu');
    expect(wrapper.find('[data-testid="chat-workspace-recent-blog"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="chat-workspace-recent-gone"]').exists()).toBe(false); // a folder that is gone is not offered
    expect(wrapper.find('[data-testid="chat-workspace-default"]').exists()).toBe(true);
    wrapper.unmount();
  });

  it('shows 默认工作区 for the configured default folder without changing its path', async () => {
    const { wrapper, chat } = setup();
    Object.assign(chat, { workspace: '/p/workspace', currentWorkspace: '/p/workspace' });
    await flushPromises();
    const button = wrapper.get('[data-testid="chat-workspace-menu"]');
    expect(button.text()).toContain('默认工作区');
    expect(button.attributes('title')).toBe('工作区：/p/workspace');
    Object.assign(chat, { currentWorkspace: '/p/site' });
    await flushPromises();
    expect(button.text()).toContain('site');
    expect(button.text()).not.toContain('默认工作区');
    wrapper.unmount();
  });

  it('switches at once when the conversation has no messages', async () => {
    const { wrapper, chat } = setup('done');
    await click(wrapper, 'chat-workspace-menu');
    await click(wrapper, 'chat-workspace-recent-blog');
    expect(chat.switchWorkspace).toHaveBeenCalledWith('/p/blog');
    expect(wrapper.find('[data-testid="chat-workspace-ask"]').exists()).toBe(false);
    wrapper.unmount();
  });

  it('asks how to switch a conversation with messages, and does what was chosen', async () => {
    for (const [button, how] of [['chat-workspace-new', 'new'], ['chat-workspace-here', 'here']] as const) {
      const { wrapper, chat } = setup('ask');
      await click(wrapper, 'chat-workspace-menu');
      await click(wrapper, 'chat-workspace-pick');
      expect(chat.pickWorkspace).toHaveBeenCalled();
      expect(wrapper.find('[data-testid="chat-workspace-ask"]').exists()).toBe(true);
      await click(wrapper, button);
      expect(chat.switchWorkspace).toHaveBeenLastCalledWith('/p/picked', how);
      expect(wrapper.find('[data-testid="chat-workspace-ask"]').exists()).toBe(false);
      wrapper.unmount();
    }
  });

  it('choosing the current folder or cancelling the picker changes nothing', async () => {
    const { wrapper, chat } = setup('done', null);
    await click(wrapper, 'chat-workspace-menu');
    await click(wrapper, 'chat-workspace-recent-site');
    await click(wrapper, 'chat-workspace-menu');
    await click(wrapper, 'chat-workspace-pick');
    expect(chat.switchWorkspace).not.toHaveBeenCalled();
    await click(wrapper, 'chat-workspace-menu');
    await click(wrapper, 'chat-workspace-reveal');
    expect(chat.revealWorkspace).toHaveBeenCalled();
    wrapper.unmount();
  });
});
