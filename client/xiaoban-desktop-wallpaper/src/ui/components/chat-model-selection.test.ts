// @vitest-environment happy-dom
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import { NButton, NSelect } from 'naive-ui';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import ChatComposerModel from './ChatComposerModel.vue';
import ModelSwitchDialog from './ModelSwitchDialog.vue';
import { createChatStore } from '../chat/chat-store.ts';
import { GATEWAY_PROVIDER, invoke, type AiConfigView, type AiTier } from '../../bridge.ts';

vi.mock('../../bridge.ts', async importOriginal => ({ ...await importOriginal<typeof import('../../bridge.ts')>(), invoke: vi.fn() }));
const ipc = vi.mocked(invoke);
const tiers: AiTier[] = [
  { tier: 'standard', displayName: '日常', description: '日常聊天', available: true, requiresPaid: false, isDefault: true },
  { tier: 'advanced', displayName: '深入', description: '深入分析', available: false, requiresPaid: true, isDefault: false },
];
const config: AiConfigView = {
  activeProvider: GATEWAY_PROVIDER, gatewayTier: 'standard', gatewayEnabled: true,
  model: 'standard', protocol: 'responses', baseUrl: '', hasKey: false, keyHint: '',
  workspace: '', defaultWorkspace: '', hideToolCalls: false, openComputerUse: false,
  providers: [{ id: 'local', name: '本地模型', model: 'local-model', protocol: 'chat', baseUrl: 'http://localhost:8000/v1', hasKey: false, keyHint: '', isCustom: true }],
};
let wrapper: VueWrapper | undefined;
beforeEach(() => {
  vi.resetAllMocks();
  ipc.mockImplementation(async (command, args) => {
    if (command === 'get_ai_config' || command === 'refresh_ai_providers') return structuredClone(config) as never;
    if (command === 'ai_gateway_tiers') return structuredClone(tiers) as never;
    if (command === 'chat_set_model') return args?.selection as never;
    throw new Error(`unexpected ${command}`);
  });
});
afterEach(() => { wrapper?.unmount(); wrapper = undefined; vi.useRealTimers(); });
function chat() {
  return createChatStore({ desktop: true, invoke: async (command, args) => {
    if (command === 'chat_set_model') { await ipc(command, args); return args?.selection as never; }
    return undefined as never;
  }, listen: async () => () => {}, notify: vi.fn() });
}
async function composer() {
  wrapper = mount(ChatComposerModel, { props: { chat: chat(), label: '小伴 AI · 日常' }, attachTo: document.body });
  await wrapper.get('[data-testid="chat-workbench-model"]').trigger('click');
  await flushPromises();
  return wrapper;
}
async function dialog() {
  wrapper = mount(ModelSwitchDialog, { props: { chat: chat(), show: true, retry: true }, attachTo: document.body });
  await flushPromises();
  return wrapper;
}

describe('gateway in the composer model menu', () => {
  it('keeps cached modes visible when reopening while both sources refresh', async () => {
    const view = await composer();
    await view.get('[data-testid="chat-workbench-model"]').trigger('click');
    ipc.mockImplementation(command => {
      if (command === 'get_ai_config') return Promise.resolve(structuredClone(config)) as never;
      return new Promise(() => {}) as never;
    });
    await view.get('[data-testid="chat-workbench-model"]').trigger('click');
    await flushPromises();
    expect(view.find('[data-testid="chat-model-xiaoban-ai-standard"]').exists()).toBe(true);
    expect(view.find('.mode-skeleton').exists()).toBe(false);
    expect(view.text()).not.toContain('读取使用模式');
    expect(view.get<HTMLButtonElement>('[data-testid="chat-model-local-local-model"]').element.disabled).toBe(false);
  });

  it('shows modes as soon as they arrive even if provider refresh is stalled', async () => {
    ipc.mockImplementation(command => {
      if (command === 'refresh_ai_providers') return new Promise(() => {}) as never;
      return Promise.resolve(structuredClone(command === 'ai_gateway_tiers' ? tiers : config)) as never;
    });
    const view = await composer();
    expect(view.get<HTMLButtonElement>('[data-testid="chat-model-xiaoban-ai-standard"]').element.disabled).toBe(false);
    expect(view.find('.mode-skeleton').exists()).toBe(false);
    expect(ipc.mock.calls.filter(([command]) => command === 'ai_gateway_tiers')).toHaveLength(1);
  });

  it('reserves mode rows on first load and lets local models switch during a slow gateway request', async () => {
    ipc.mockImplementation(command => {
      if (command === 'ai_gateway_tiers') return new Promise(() => {}) as never;
      return Promise.resolve(structuredClone(config)) as never;
    });
    const view = await composer();
    expect(view.get('.mode-skeleton').findAll('.model-choice')).toHaveLength(2);
    await view.get('[data-testid="chat-model-local-local-model"]').trigger('click');
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('chat_set_model', expect.objectContaining({ sessionId: null, selection: expect.objectContaining({ providerId: 'local' }) }));
    expect(view.emitted('switched')).toHaveLength(1);
  });

  it('ignores a stale gateway response after the menu closes and reopens', async () => {
    let finishOld!: (value: AiTier[]) => void;
    const old = new Promise<AiTier[]>(resolve => { finishOld = resolve; });
    let calls = 0;
    ipc.mockImplementation(command => {
      if (command === 'ai_gateway_tiers') return (++calls === 1 ? old : Promise.resolve(tiers)) as never;
      return Promise.resolve(structuredClone(config)) as never;
    });
    const view = await composer();
    await view.get('[data-testid="chat-workbench-model"]').trigger('click');
    await view.get('[data-testid="chat-workbench-model"]').trigger('click');
    await flushPromises();
    finishOld([{ ...tiers[0], displayName: '过期模式' }]);
    await flushPromises();
    expect(view.get('[data-testid="chat-model-xiaoban-ai-standard"]').text()).toBe('日常');
    expect(view.text()).not.toContain('过期模式');
  });

  it('shows the active mode, disables unavailable modes and keeps local providers', async () => {
    const view = await composer();
    expect(view.get('[data-testid="chat-model-xiaoban-ai-standard"]').classes()).toContain('selected');
    expect(view.get('[data-testid="chat-model-xiaoban-ai-standard"]').text()).toBe('日常');
    expect(view.get<HTMLButtonElement>('[data-testid="chat-model-xiaoban-ai-advanced"]').element.disabled).toBe(true);
    expect(view.find('[data-testid="chat-model-local-local-model"]').exists()).toBe(true);
    await view.get('[data-testid="chat-model-xiaoban-ai-standard"]').trigger('click');
    expect(ipc).not.toHaveBeenCalledWith('use_ai_gateway', expect.anything());
  });

  it('switches the conversation gateway mode without changing provider defaults', async () => {
    ipc.mockImplementation(async command => {
      if (command === 'ai_gateway_tiers') return tiers.map(t => ({ ...t, available: true })) as never;
      return structuredClone(config) as never;
    });
    const view = await composer();
    await view.get('[data-testid="chat-model-xiaoban-ai-advanced"]').trigger('click');
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('chat_set_model', expect.objectContaining({ selection: expect.objectContaining({ providerId: GATEWAY_PROVIDER, model: 'advanced' }) }));
    expect(ipc.mock.calls.some(([command]) => command === 'save_ai_provider')).toBe(false);
    expect(view.emitted('switched')).toHaveLength(1);
  });

  it('keeps other providers selectable when gateway login fails', async () => {
    ipc.mockImplementation(async command => {
      if (command === 'ai_gateway_tiers') throw { message: '请先登录小伴账号' };
      return structuredClone(config) as never;
    });
    const view = await composer();
    expect(view.text()).toContain('请先登录小伴账号');
    await view.get('[data-testid="chat-model-local-local-model"]').trigger('click');
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('chat_set_model', expect.objectContaining({ sessionId: null, selection: expect.objectContaining({ providerId: 'local', model: 'local-model' }) }));
  });

  it('hides the gateway when the site has disabled it', async () => {
    ipc.mockResolvedValue({ ...config, gatewayEnabled: false } as never);
    const view = await composer();
    expect(view.find('[data-testid="chat-model-gateway"]').exists()).toBe(false);
    expect(ipc.mock.calls.some(([command]) => command === 'ai_gateway_tiers')).toBe(false);
  });
});

describe('gateway in the retry model dialog', () => {
  it('does not request a retry in another conversation after a delayed model switch', async () => {
    const original = ipc.getMockImplementation()!;
    let finish!: () => void;
    const gate = new Promise<void>(resolve => { finish = resolve; });
    ipc.mockImplementation(async (command, args) => {
      if (command === 'chat_set_model') { await gate; return args?.selection as never; }
      return original(command, args);
    });
    const view = await dialog();
    view.getComponent(NSelect).vm.$emit('update:value', JSON.stringify(['local', 'local-model']));
    await flushPromises();
    await view.findAllComponents(NButton).find(button => button.props('type') === 'primary')!.trigger('click');
    const owner = (view.props() as { chat: ReturnType<typeof chat> }).chat;
    owner.currentId = 'another'; await flushPromises();
    finish(); await flushPromises();
    expect(view.emitted('switched')).toBeUndefined();
    expect(owner.currentId).toBe('another');
  });

  it('lists gateway modes and routes switch-and-retry to the gateway command', async () => {
    const view = await dialog();
    const select = view.getComponent(NSelect);
    expect(select.props('options')).toEqual(expect.arrayContaining([
      expect.objectContaining({ label: '小伴 AI · 日常', disabled: false }),
      expect.objectContaining({ label: '小伴 AI · 深入（暂不可用）', disabled: true }),
      expect.objectContaining({ label: expect.stringContaining('本地模型') }),
    ]));
    select.vm.$emit('update:value', JSON.stringify([GATEWAY_PROVIDER, 'standard']));
    await flushPromises();
    await view.findAllComponents(NButton).find(button => button.props('type') === 'primary')!.trigger('click');
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('chat_set_model', expect.objectContaining({ selection: expect.objectContaining({ providerId: GATEWAY_PROVIDER, model: 'standard' }) }));
    expect(view.emitted('switched')).toHaveLength(1);
    expect(view.emitted('update:show')).toEqual([[false]]);
  });

  it('does not activate an unavailable tier even if selected programmatically', async () => {
    const view = await dialog();
    view.getComponent(NSelect).vm.$emit('update:value', JSON.stringify([GATEWAY_PROVIDER, 'advanced']));
    await flushPromises();
    await view.findAllComponents(NButton).find(button => button.props('type') === 'primary')!.trigger('click');
    await flushPromises();
    expect(ipc.mock.calls.some(([command]) => command === 'use_ai_gateway')).toBe(false);
  });
});

describe('hovering a model to choose thinking strength', () => {
  const provider = {
    id: 'my-openai', name: '我的 OpenAI', model: 'gpt-6.1-sol', protocol: 'responses' as const,
    baseUrl: 'https://api.openai.com/v1', hasKey: true, keyHint: '', isCustom: true,
    replyPreference: 'default' as const,
    models: [{ id: 'gpt-6.1-sol', name: '编程模型' }, { id: 'gpt-6-luna', name: '轻量模型' }],
  };
  function useConfig(activeProvider = provider.id) {
    ipc.mockImplementation(async command => {
      if (command === 'ai_gateway_tiers') return structuredClone(tiers) as never;
      return { ...structuredClone(config), activeProvider, model: provider.model, providers: [structuredClone(provider), ...structuredClone(config.providers!)] } as never;
    });
  }
  const panel = () => document.querySelector<HTMLElement>('[data-testid="chat-model-reasoning"]');
  const strength = (value: string) => document.querySelector<HTMLButtonElement>(`[data-testid="chat-model-strength-${value}"]`)!;

  it('opens on hover without switching and can change strength on the current model', async () => {
    useConfig();
    const view = await composer();
    await view.get('[data-testid="chat-model-my-openai-gpt-6.1-sol"]').trigger('mouseenter');
    expect(panel()?.textContent).not.toContain('思考强度');
    expect(panel()?.textContent).not.toContain('编程模型');
    expect(panel()?.querySelectorAll(':scope > *').length).toBe(5);
    expect(strength('default')).toBeNull();
    expect(strength('medium').getAttribute('aria-checked')).toBe('true');
    expect(strength('faster').textContent).toContain('低');
    expect(strength('deeper').textContent).toContain('高');
    expect(ipc.mock.calls.some(([command]) => command === 'save_ai_provider')).toBe(false);
    strength('deeper').dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }));
    expect(panel()).not.toBeNull();
    strength('deeper').click();
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('chat_set_model', expect.objectContaining({ sessionId: null, selection: expect.objectContaining({
      providerId: provider.id, model: provider.model, replyPreference: 'deeper',
    }) }));
    expect(panel()).toBeNull();
    expect(view.emitted('switched')).toHaveLength(1);
  });

  it('uses the hovered model rather than the active model when choosing strength', async () => {
    useConfig(GATEWAY_PROVIDER);
    const view = await composer();
    await view.get('[data-testid="chat-model-my-openai-gpt-6-luna"]').trigger('mouseenter');
    strength('faster').click();
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('chat_set_model', expect.objectContaining({ sessionId: null, selection: expect.objectContaining({ providerId: provider.id, model: 'gpt-6-luna', replyPreference: 'faster' }) }));
    expect(ipc.mock.calls.some(([command]) => command === 'use_ai_gateway')).toBe(false);
  });

  it('keeps the submenu open while moving through the gap and closes it after leaving', async () => {
    useConfig();
    const view = await composer();
    const row = view.get('[data-testid="chat-model-my-openai-gpt-6.1-sol"]');
    await row.trigger('mouseenter');
    vi.useFakeTimers();
    await row.trigger('mouseleave');
    vi.advanceTimersByTime(100);
    panel()!.dispatchEvent(new MouseEvent('mouseenter'));
    vi.advanceTimersByTime(300);
    expect(panel()).not.toBeNull();
    panel()!.dispatchEvent(new MouseEvent('mouseleave'));
    await vi.advanceTimersByTimeAsync(200);
    expect(panel()).toBeNull();
  });

  it('opens to the left near the right window edge and stays within the bottom edge', async () => {
    useConfig();
    const view = await composer();
    const row = view.get('[data-testid="chat-model-my-openai-gpt-6.1-sol"]');
    const left = window.innerWidth - 230;
    vi.spyOn(row.element, 'getBoundingClientRect').mockReturnValue({ left, right: window.innerWidth - 12, top: window.innerHeight - 12 } as DOMRect);
    await row.trigger('mouseenter');
    expect(parseFloat(panel()!.style.left) + parseFloat(panel()!.style.width)).toBeLessThanOrEqual(left);
    expect(parseFloat(panel()!.style.top)).toBeLessThanOrEqual(window.innerHeight - 8);
    view.get('.model-panel').element.dispatchEvent(new Event('scroll'));
    await flushPromises();
    expect(panel()).toBeNull();
  });

  it('omits empty submenus for models and gateway modes without declared levels', async () => {
    const view = await composer();
    const local = view.get('[data-testid="chat-model-local-local-model"]');
    await local.trigger('mouseenter');
    expect(panel()).toBeNull();
    expect(local.attributes('aria-haspopup')).toBeUndefined();
    const gateway = view.get('[data-testid="chat-model-xiaoban-ai-standard"]');
    await gateway.trigger('mouseenter');
    expect(panel()).toBeNull();
    expect(gateway.attributes('aria-haspopup')).toBeUndefined();
  });

  it('uses a concrete supported default when medium is unavailable', async () => {
    ipc.mockImplementation(async command => {
      if (command === 'ai_gateway_tiers') return [{ ...tiers[0], reasoningEfforts: ['low', 'high', 'max'] }] as never;
      return { ...structuredClone(config), gatewayReplyPreference: 'default' } as never;
    });
    const view = await composer();
    await view.get('[data-testid="chat-model-xiaoban-ai-standard"]').trigger('mouseenter');
    expect(strength('default')).toBeNull();
    expect(strength('deeper').getAttribute('aria-checked')).toBe('true');
    expect(panel()?.querySelectorAll('[aria-checked="true"]')).toHaveLength(1);
    strength('deeper').click();
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('chat_set_model', expect.objectContaining({ selection: expect.objectContaining({ providerId: GATEWAY_PROVIDER, model: 'standard', replyPreference: 'deeper' }) }));
  });

  it('preserves a saved level for an inactive provider and defaults other models separately', async () => {
    ipc.mockImplementation(async command => {
      if (command === 'ai_gateway_tiers') return structuredClone(tiers) as never;
      return { ...structuredClone(config), providers: [{ ...provider, replyPreference: 'max' }] } as never;
    });
    const view = await composer();
    await view.get('[data-testid="chat-model-my-openai-gpt-6.1-sol"]').trigger('mouseenter');
    expect(strength('max').getAttribute('aria-checked')).toBe('true');
    await view.get('[data-testid="chat-model-my-openai-gpt-6-luna"]').trigger('mouseenter');
    expect(strength('medium').getAttribute('aria-checked')).toBe('true');
    expect(panel()?.querySelectorAll('[aria-checked="true"]')).toHaveLength(1);
  });

  it('lets the active gateway mode change strength using server capabilities', async () => {
    ipc.mockImplementation(async command => {
      if (command === 'ai_gateway_tiers') return [{...tiers[0], reasoningEfforts:['low','high','max']}] as never;
      return {...structuredClone(config), gatewayReplyPreference:'deeper'} as never;
    });
    const view = await composer();
    await view.get('[data-testid="chat-model-xiaoban-ai-standard"]').trigger('mouseenter');
    expect(strength('deeper').getAttribute('aria-checked')).toBe('true');
    expect(strength('medium')).toBeNull();
    strength('max').click();
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('chat_set_model', expect.objectContaining({ selection: expect.objectContaining({ providerId: GATEWAY_PROVIDER, model: 'standard', replyPreference: 'max' }) }));
    expect(view.emitted('switched')).toHaveLength(1);
  });

  it('supports managed aliases and compatible proxy models without domain checks', async () => {
    ipc.mockImplementation(async command => {
      if (command === 'ai_gateway_tiers') return [] as never;
      return {...structuredClone(config), providers:[{...provider, remote:true, type:'free', isCustom:false,
        baseUrl:'https://proxy.example/v1', model:'alias', models:[{id:'alias',name:'托管模型',reasoningEfforts:['medium','high']}]}]} as never;
    });
    const view = await composer();
    await view.get('[data-testid="chat-model-my-openai-alias"]').trigger('mouseenter');
    expect(strength('faster')).toBeNull();
    strength('medium').click();
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('chat_set_model', expect.objectContaining({ sessionId: null, selection: expect.objectContaining({model:'alias', replyPreference:'medium'}) }));
  });

  it('allows keyboard entry and closes the submenu before the model list on Escape', async () => {
    useConfig();
    const view = await composer();
    await view.get('[data-testid="chat-model-my-openai-gpt-6.1-sol"]').trigger('keydown', { key: 'ArrowRight' });
    expect(document.activeElement).toBe(strength('faster'));
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
    await flushPromises();
    expect(panel()).toBeNull();
    expect(view.find('.model-panel').exists()).toBe(true);
    document.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }));
    await flushPromises();
    expect(view.find('.model-panel').exists()).toBe(false);
  });
});
