// @vitest-environment happy-dom
import { mount, flushPromises } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import ChatWorkbenchTools from './ChatWorkbenchTools.vue';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';

vi.mock('../../bridge.ts', async (importOriginal) => ({
  ...await importOriginal<typeof import('../../bridge.ts')>(),
  invoke: vi.fn(),
}));

function setup() {
  const store = { desktop: true, notify: vi.fn(), navigate: vi.fn() } as unknown as ControlStore;
  const wrapper = mount(ChatWorkbenchTools, {
    props: { store },
    attachTo: document.body,
  });
  return { wrapper };
}
beforeEach(() => {
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation((async (command: string) => {
    if (command === 'ai_store_experts_installed' || command === 'ai_store_installed' || command === 'chat_invocable_skills' || command === 'ai_store_connector_status' || command === 'ai_connector_installed') return [];
    return { results: [] };
  }) as typeof invoke);
});
afterEach(() => { vi.useRealTimers(); });

describe('add-capability menu hover interaction', () => {
  it('shows compact builtin tools and selects an enabled computer tool', async () => {
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async (command, args) => command === 'get_ai_config' ? { openComputerUse: true } as never : original(command, args));
    const { wrapper } = setup();
    await wrapper.get('[data-testid=chat-add-menu]').trigger('click');
    await wrapper.get('[data-testid=chat-add-tools]').trigger('click');
    expect(wrapper.get('[aria-label=工具]').text()).toContain('操作电脑操作手机设计画板');
    expect(wrapper.find('[data-testid=chat-capability-search]').exists()).toBe(false);
    await wrapper.get('[data-testid=chat-tool-computer]').trigger('click'); await flushPromises();
    expect(wrapper.emitted('use-tool')).toEqual([['computer']]);
    wrapper.unmount();
  });
  it('enables computer control in the tools menu before selecting it', async () => {
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async (command, args) => command === 'get_ai_config' ? { openComputerUse: false } as never : command === 'save_ai_preferences' ? { openComputerUse: true } as never : original(command, args));
    const { wrapper } = setup(); const store = wrapper.props('store');
    await wrapper.get('[data-testid=chat-add-menu]').trigger('click'); await wrapper.get('[data-testid=chat-add-tools]').trigger('click');
    await wrapper.get('[data-testid=chat-tool-computer]').trigger('click'); await flushPromises();
    expect(wrapper.emitted('use-tool')).toBeUndefined();
    expect(wrapper.get('[data-testid=chat-computer-settings]').text()).toContain('未启用');
    await wrapper.get('[data-testid=chat-computer-toggle]').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('save_ai_preferences', { openComputerUse: true });
    expect(wrapper.emitted('use-tool')).toEqual([['computer']]);
    expect(store.navigate).not.toHaveBeenCalled(); wrapper.unmount();
  });
  it('manages enabled computer control and can request a permission check', async () => {
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async (command, args) => command === 'get_ai_config' ? { openComputerUse: true } as never : command === 'save_ai_preferences' ? { openComputerUse: false } as never : original(command, args));
    const { wrapper } = setup();
    await wrapper.get('[data-testid=chat-add-menu]').trigger('click'); await wrapper.get('[data-testid=chat-add-tools]').trigger('click');
    await wrapper.get('[data-testid=chat-tool-computer-manage]').trigger('click'); await flushPromises();
    expect(wrapper.get('[data-testid=chat-computer-settings]').text()).toContain('已启用');
    await wrapper.get('[data-testid=chat-computer-check]').trigger('click');
    expect(wrapper.emitted('check-computer-permissions')).toHaveLength(1);
    await wrapper.get('[data-testid=chat-add-menu]').trigger('click'); await wrapper.get('[data-testid=chat-add-tools]').trigger('click');
    await wrapper.get('[data-testid=chat-tool-computer-manage]').trigger('click'); await flushPromises();
    await wrapper.get('[data-testid=chat-computer-toggle]').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('save_ai_preferences', { openComputerUse: false });
    expect(wrapper.emitted('computer-disabled')).toHaveLength(1); wrapper.unmount();
  });
  it('does not select computer control in another conversation when enabling finishes late', async () => {
    let finish!: (value: unknown) => void;
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async (command, args) => command === 'get_ai_config' ? { openComputerUse: false } as never : command === 'save_ai_preferences' ? new Promise(resolve => { finish = resolve; }) : original(command, args));
    const { wrapper } = setup(); const store = wrapper.props('store');
    Object.assign(store, { chat: { draft: { text: 'first' } } });
    await wrapper.get('[data-testid=chat-add-menu]').trigger('click'); await wrapper.get('[data-testid=chat-add-tools]').trigger('click');
    await wrapper.get('[data-testid=chat-tool-computer]').trigger('click'); await flushPromises();
    await wrapper.get('[data-testid=chat-computer-toggle]').trigger('click');
    Object.assign(store.chat, { draft: { text: 'second' } }); finish({ openComputerUse: true }); await flushPromises();
    expect(wrapper.emitted('use-tool')).toBeUndefined(); wrapper.unmount();
  });
  it('does not apply an asynchronous computer selection to a different conversation', async () => {
    let finish!: (value: unknown) => void;
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async (command, args) => command === 'get_ai_config' ? new Promise(resolve => { finish = resolve; }) : original(command, args));
    const { wrapper } = setup(); const store = wrapper.props('store');
    Object.assign(store, { chat: { draft: { text: 'first' } } });
    await wrapper.get('[data-testid=chat-add-menu]').trigger('click'); await wrapper.get('[data-testid=chat-add-tools]').trigger('click');
    await wrapper.get('[data-testid=chat-tool-computer]').trigger('click');
    Object.assign(store.chat, { draft: { text: 'second' } }); finish({ openComputerUse: true }); await flushPromises();
    expect(wrapper.emitted('use-tool')).toBeUndefined(); wrapper.unmount();
  });
  it('keeps in-flight inventory when opening a submenu or typing a search', async () => {
    let resolve!: (value: unknown) => void;
    const original=vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async (command,args)=>command==='ai_store_experts_installed' ? new Promise(r=>{resolve=r;}) : original(command,args));
    const {wrapper}=setup();
    await wrapper.get('[data-testid=chat-add-menu]').trigger('mouseenter');
    await wrapper.get('[data-testid=chat-add-experts]').trigger('mouseenter');
    expect(wrapper.text()).toContain('正在读取已安装的专家');
    expect(wrapper.text()).not.toContain('暂无已安装能力');
    await wrapper.get('[data-testid=chat-capability-search]').setValue('健康');
    resolve([{slug:'health-report',name:'健康报告',description:'报告解读',entrySkill:'health-report',ready:true}]);await flushPromises();
    expect(wrapper.get('[data-testid=chat-installed-health-report]').text()).toContain('健康报告');
    await wrapper.get('[data-testid=chat-installed-health-report]').trigger('click');
    expect(wrapper.emitted('use-expert')).toEqual([['health-report','健康报告',undefined]]);wrapper.unmount();
  });
  it('shows installed unconfigured plugins and deduplicates configured plugin services',async()=>{
    const original=vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async(command,args)=>{
      if(command==='ai_connector_installed')return [{id:'pan:main',name:'百度网盘',enabled:false,status:'disconnected',tools:[]},{id:'repo:main',name:'GitHub',enabled:true,status:'connected',tools:[{name:'search'}]}];
      if(command==='ai_store_connector_status')return [{id:'repo:main',name:'Old name',enabled:true,connected:true,toolCount:1}];
      return original(command,args);
    });
    const {wrapper}=setup();await wrapper.get('[data-testid=chat-add-menu]').trigger('click');
    await wrapper.get('[data-testid=chat-add-connectors]').trigger('click');await flushPromises();
    expect(wrapper.get('[data-testid="chat-installed-pan:main"]').text()).toContain('未连接');
    expect(wrapper.findAll('[data-testid="chat-installed-repo:main"]')).toHaveLength(1);
    await wrapper.get('[data-testid="chat-installed-repo:main"]').trigger('click');
    expect(wrapper.emitted('use-connector')).toEqual([['GitHub','repo:main']]);wrapper.unmount();
  });
  it('reports a category failure even when other inventories succeed and supports retry',async()=>{
    const original=vi.mocked(invoke).getMockImplementation()!;let fail=true;
    vi.mocked(invoke).mockImplementation(async(command,args)=>{
      if(command==='ai_store_experts_installed'){if(fail)throw new Error('read failure');return [{slug:'expert',name:'我的专家',description:'说明'}];}
      return original(command,args);
    });
    const {wrapper}=setup();await wrapper.get('[data-testid=chat-add-menu]').trigger('click');await wrapper.get('[data-testid=chat-add-experts]').trigger('click');await flushPromises();
    expect(wrapper.get('[role=alert]').text()).toContain('读取已安装专家失败');expect(wrapper.text()).not.toContain('暂无已安装能力');
    fail=false;await wrapper.get('[role=alert] button').trigger('click');await flushPromises();
    expect(wrapper.get('[data-testid=chat-installed-expert]').text()).toContain('我的专家');wrapper.unmount();
  });
  it('opens on hover and stays open while crossing the gap into the menu', async () => {
    vi.useFakeTimers();
    const { wrapper } = setup();
    const trigger = wrapper.get('[data-testid="chat-add-menu"]');
    await trigger.trigger('mouseenter');
    await flushPromises();
    expect(trigger.attributes('aria-expanded')).toBe('true');
    expect(wrapper.get('.wb-main').isVisible()).toBe(true);

    // Clicking the already hover-open trigger must not immediately dismiss the menu.
    await trigger.trigger('click');
    expect(wrapper.get('.wb-main').isVisible()).toBe(true);

    await wrapper.get('.wb-controls').trigger('mouseleave');
    await wrapper.get('.wb-main').trigger('mouseenter');
    vi.advanceTimersByTime(220);
    await flushPromises();
    expect(wrapper.get('.wb-main').isVisible()).toBe(true);
    wrapper.unmount();
  });

  it('closes after the pointer leaves the whole menu region', async () => {
    vi.useFakeTimers();
    const { wrapper } = setup();
    await wrapper.get('[data-testid="chat-add-menu"]').trigger('mouseenter');
    await flushPromises();
    await wrapper.get('.wb-controls').trigger('mouseleave');
    vi.advanceTimersByTime(220);
    await flushPromises();
    expect(wrapper.get('[data-testid="chat-add-menu"]').attributes('aria-expanded')).toBe('false');
    expect(wrapper.get('.wb-main').isVisible()).toBe(false);
    wrapper.unmount();
  });

  it('still opens when focused for keyboard access and closes on Escape', async () => {
    const { wrapper } = setup();
    await wrapper.get('[data-testid="chat-add-menu"]').trigger('focus');
    await flushPromises();
    expect(wrapper.get('.wb-main').isVisible()).toBe(true);
    await wrapper.get('[data-testid="chat-add-menu"]').trigger('keydown', { key: 'Escape' });
    expect(wrapper.get('.wb-main').isVisible()).toBe(false);
    wrapper.unmount();
  });
});
