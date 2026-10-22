// @vitest-environment happy-dom
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import { NSelect } from 'naive-ui';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { reactive } from 'vue';
import AiConfigCard from './AiConfigCard.vue';
import type { ControlStore } from '../control-types.ts';
import type { AiConfigView } from '../../bridge.ts';
import { invoke } from '../../bridge.ts';

vi.mock('../../bridge.ts', async importOriginal => ({ ...await importOriginal<typeof import('../../bridge.ts')>(), invoke: vi.fn() }));
const ipc = vi.mocked(invoke);

const KEY = 'sk-live-abcdefghijklmnop9876';
const configured: AiConfigView = {
  activeProvider: 'deepseek',
  providers: [
    { id: 'deepseek', name: 'DeepSeek', protocol: 'chat', baseUrl: 'https://api.example.com/v1', model: 'model-a', hasKey: true, keyHint: '••••9876', isCustom: false },
    { id: 'zhipu', name: '智谱 BigModel', protocol: 'chat', baseUrl: 'https://open.bigmodel.cn/api/paas/v4', model: 'glm-4-flash', hasKey: false, keyHint: '', isCustom: false },
    { id: 'kimi', name: 'Kimi', protocol: 'chat', baseUrl: 'https://api.moonshot.cn/v1', model: 'moonshot-v1-auto', hasKey: false, keyHint: '', isCustom: false },
    { id: 'minimax', name: 'MiniMax', protocol: 'chat', baseUrl: 'https://api.minimax.chat/v1', model: 'MiniMax-Text-01', hasKey: false, keyHint: '', isCustom: false },
  ],
  protocol: 'chat',
  baseUrl: 'https://api.example.com/v1',
  model: 'model-a',
  hasKey: true,
  keyHint: '••••9876',
  workspace: '',
  defaultWorkspace: '/data/workspace',
  hideToolCalls: false,
  openComputerUse: false,
};

const unconfigured: AiConfigView = {
  activeProvider: 'deepseek',
  providers: [],
  protocol: 'chat',
  baseUrl: '',
  model: '',
  hasKey: false,
  keyHint: '',
  workspace: '',
  defaultWorkspace: '/data/workspace',
  hideToolCalls: false,
  openComputerUse: false,
};

const createStore = (desktop = true) => reactive({ desktop, notify: vi.fn() }) as unknown as ControlStore;
let wrapper: VueWrapper | undefined;

async function open(initial: AiConfigView | Error = configured, store = createStore()) {
  ipc.mockImplementation(async command => {
    if (command === 'get_ai_config') { if (initial instanceof Error) throw initial; return initial as never; }
    throw new Error(`unexpected command ${command}`);
  });
  wrapper = mount(AiConfigCard, { props: { store }, attachTo: document.body });
  await flushPromises();
  return store;
}

const q = <T extends Element = HTMLElement>(testid: string) => document.querySelector(`[data-testid="${testid}"]`) as T | null;
const field = (testid: string) => document.querySelector(`[data-testid="${testid}"] input`) as HTMLInputElement;

async function type(testid: string, value: string) {
  const input = field(testid);
  input.value = value;
  input.dispatchEvent(new Event('input'));
  await flushPromises();
}

async function click(testid: string) {
  q<HTMLButtonElement>(testid)!.click();
  await flushPromises();
}

async function openModal(providerId = 'deepseek') {
  await click(`ai-provider-config-${providerId}`);
  await flushPromises();
}

// Without a usable provider only 保存并启用 is offered, so the helper follows whichever exists.
const saveButton = () => q<HTMLButtonElement>('ai-save') ?? q<HTMLButtonElement>('ai-save-and-activate')!;
const pickProtocol = async (value: string) => {
  if (!q('ai-protocol-menu')) await click('ai-protocol');
  await click(`ai-protocol-${value}`);
};
/** Add a model by name, the way the dialog's input does (type, then 添加). */
const addModel = async (name: string) => { await type('ai-model', name); await click('ai-model-add'); };
const expandModel = (id: string) => click(`ai-model-expand-${id}`);

beforeEach(() => { vi.resetAllMocks(); });
afterEach(() => { wrapper?.unmount(); wrapper = undefined; });

describe('AI model settings card and provider templates', () => {
  it('uses the delivered catalogue and locks managed credentials while allowing manual models', async () => {
    const remote: AiConfigView = { ...configured, catalogueReceived: true, activeProvider: 'official-free', providers: [{
      ...configured.providers![0], id: 'official-free', name: '小伴免费', type: 'free', remote: true, available: true,
      defaultModel: 'new-recommendation', models: [{ id: 'model-a', name: '手动模型', description: '保留用户选择' }, { id: 'new-recommendation', name: '推荐模型' }],
      selectionMode: 'manual', description: '免费服务说明',
    }] };
    await open(remote);
    expect(q('ai-provider-card-deepseek')).toBeNull();
    expect(q('ai-provider-card-official-free')?.textContent).toContain('免费服务');
    await openModal('official-free');
    expect(q('ai-api-key')).toBeNull();
    expect(field('ai-base-url').disabled).toBe(true);
    expect(q('ai-model-row-model-a')).not.toBeNull();
    expect(q('ai-model-default-model-a')?.getAttribute('aria-pressed')).toBe('true');
    expect(q('ai-model-remove-model-a')).toBeNull(); // managed lists are the catalogue's
    expect(q('ai-model')).toBeNull();
  });

  it('puts the 小伴 AI card first, ahead of every provider, and tells the status line when it is in use', async () => {
    await open();
    const first = document.querySelector('[data-testid="ai-gateway-card"], [data-testid^="ai-provider-card-"]');
    expect(first?.getAttribute('data-testid')).toBe('ai-gateway-card');
    expect(q('ai-provider-card-deepseek')).not.toBeNull();
    expect(q('ai-gateway-card')?.textContent).toContain('需登录');
    expect(q('ai-status')?.textContent).not.toContain('小伴 AI');
    wrapper?.unmount();
    wrapper = undefined;

    await open({ ...configured, activeProvider: 'xiaoban-ai', gatewayTier: 'standard' });
    expect(q('ai-status')?.textContent).toContain('当前生效：小伴 AI');
    expect(q('ai-provider-card-deepseek')?.textContent).not.toContain('当前使用');
  });

  it('hides the 小伴 AI card when the site has closed that entry, and says so if chat was set to it', async () => {
    const reopen = async (view: AiConfigView) => { wrapper?.unmount(); await open(view); };
    await reopen({ ...configured, gatewayEnabled: false });
    expect(q('ai-gateway-card')).toBeNull();
    expect(q('ai-provider-card-deepseek')).not.toBeNull();

    await reopen({ ...configured, activeProvider: 'xiaoban-ai', gatewayTier: 'standard', gatewayEnabled: false });
    expect(q('ai-gateway-card')).toBeNull();
    expect(q('ai-status')?.textContent).toContain('小伴 AI 入口已被关闭');

    await reopen({ ...configured, gatewayEnabled: true });
    expect(q('ai-gateway-card')).not.toBeNull();
  });

  it('renders 4 builtin provider cards and add-custom card, showing "待配置" for providers without key', async () => {
    await open();
    expect(q('ai-provider-card-deepseek')).not.toBeNull();
    expect(q('ai-provider-card-zhipu')).not.toBeNull();
    expect(q('ai-provider-card-kimi')).not.toBeNull();
    expect(q('ai-provider-card-minimax')).not.toBeNull();
    expect(q('ai-add-custom-card')).not.toBeNull();

    // DeepSeek hasKey is true -> shows 当前使用
    expect(q('ai-provider-card-deepseek')?.textContent).toContain('当前使用');
    // zhipu hasKey is false -> shows 待配置
    expect(q('ai-provider-card-zhipu')?.textContent).toContain('待配置');
    expect(q('ai-status')!.textContent).toContain('DeepSeek');
  });

  it('shows 待配置 and not in-use when key is missing even if activeProvider is set', async () => {
    await open(unconfigured);
    expect(q('ai-provider-card-deepseek')?.textContent).toContain('待配置');
    expect(q('ai-provider-card-deepseek')?.textContent).not.toContain('当前使用');
    expect(q('ai-provider-card-deepseek')?.textContent).not.toContain('正在使用中');
    expect(q('ai-provider-activate-deepseek')).toBeNull(); // nothing to activate until it is configured
    expect(q('ai-provider-config-deepseek')?.textContent).toContain('配置');
    expect(q('ai-status')!.textContent).toContain('尚未配置');
  });

  it('shows what is saved in modal and never the key', async () => {
    await open();
    await openModal('deepseek');
    expect(field('ai-base-url').value).toBe('https://api.example.com/v1');
    expect(q('ai-model-row-model-a')).not.toBeNull();
    expect(field('ai-api-key').value).toBe('');
    expect(field('ai-api-key').placeholder).toContain('••••9876');
    expect(q('ai-endpoint-preview')!.textContent).toContain('https://api.example.com/v1/chat/completions');
    expect(document.body.textContent).not.toContain(KEY);
  });

  it('offers 启用 only for providers that are configured, and 配置 for the rest', async () => {
    await open();
    expect(q('ai-provider-activate-zhipu')).toBeNull();
    await click('ai-provider-config-zhipu');
    expect(q('ai-provider-modal')?.style.display).not.toBe('none');
    expect(q('ai-provider-dialog')?.textContent).toContain('智谱 BigModel');
  });

  it('opens modal to create a custom provider with custom name field', async () => {
    await open();
    await click('ai-add-custom-card');
    expect(q('ai-provider-name')).not.toBeNull();
    expect(field('ai-provider-name').value).toBe('');
    expect(q('ai-provider-dialog')?.textContent).toContain('添加自定义供应商');

    await type('ai-provider-name', '本地 Ollama');
    await type('ai-base-url', 'http://localhost:11434/v1');
    await addModel('llama3');

    ipc.mockResolvedValueOnce({
      ...configured,
      providers: [
        ...configured.providers!,
        { id: 'custom_1', name: '本地 Ollama', protocol: 'chat', baseUrl: 'http://localhost:11434/v1', model: 'llama3', hasKey: false, keyHint: '', isCustom: true },
      ],
    } as never);

    await click('ai-save');
    expect(ipc).toHaveBeenCalledWith('save_ai_provider', expect.objectContaining({
      name: '本地 Ollama',
      baseUrl: 'http://localhost:11434/v1',
      model: 'llama3',
      activate: false,
    }));
  });

  it('validates form errors and disables save button when invalid', async () => {
    await open(unconfigured);
    await click('ai-add-custom-card');
    await type('ai-provider-name', '');
    expect(q('ai-form-error')!.textContent).toContain('名称');
    expect(saveButton().disabled).toBe(true);

    await type('ai-provider-name', '我的服务商');
    await type('ai-base-url', 'api.example.com');
    expect(q('ai-form-error')!.textContent).toContain('格式不对');
    expect(saveButton().disabled).toBe(true);

    await type('ai-base-url', 'https://api.example.com/v1');
    expect(q('ai-form-error')!.textContent).toContain('模型名');
    expect(saveButton().disabled).toBe(true);

    await addModel('model-a');
    expect(q('ai-form-error')).toBeNull();
    expect(saveButton().disabled).toBe(false);

    await type('ai-api-key', 'sk bad key');
    expect(q('ai-form-error')!.textContent).toContain('空格');
    expect(saveButton().disabled).toBe(true);
  });

  it('offers only 保存并启用 while nothing usable is in effect', async () => {
    await open(unconfigured);
    await click('ai-add-custom-card');
    expect(q('ai-save')).toBeNull();
    expect(q('ai-save-and-activate')).not.toBeNull();
    expect(q('ai-status')?.textContent).toContain('配置好一个后会自动启用');
  });

  it('keeps the plain 保存配置 once a provider is in effect', async () => {
    await open(configured);
    await openModal('zhipu');
    expect(q('ai-save')).not.toBeNull();
    expect(q('ai-save-and-activate')).not.toBeNull();
    // The provider already in use has nothing to activate: one 保存 button.
    wrapper?.unmount();
    await open(configured);
    await openModal('deepseek');
    expect(q('ai-save')).not.toBeNull();
    expect(q('ai-save-and-activate')).toBeNull();
  });

  it('tells the user when saving what they configured made it take effect', async () => {
    const store = await open(unconfigured);
    await click('ai-add-custom-card');
    await type('ai-provider-name', '我的服务商');
    await type('ai-base-url', 'https://api.example.com/v1');
    await addModel('model-a');

    // Answer the save once it is sent, with the view the host would return for this provider.
    ipc.mockImplementationOnce(async (command, args) => {
      if (command !== 'save_ai_provider') throw new Error(`unexpected command ${command}`);
      const sent = args as { providerId: string; activate: boolean };
      expect(sent.activate).toBe(true);
      return {
        ...unconfigured,
        activeProvider: sent.providerId,
        providers: [{ id: sent.providerId, name: '我的服务商', protocol: 'responses', baseUrl: 'https://api.example.com/v1', model: 'model-a', hasKey: false, keyHint: '', isCustom: true }],
        baseUrl: 'https://api.example.com/v1',
        model: 'model-a',
      } as never;
    });

    await click('ai-save-and-activate');
    expect(store.notify).toHaveBeenCalledWith('已保存并启用 我的服务商', 'success');
  });

  it('saves provider configuration via modal', async () => {
    const store = await open(configured);
    await openModal('deepseek');
    await addModel('deepseek-reasoner');
    await click('ai-model-default-deepseek-reasoner');

    ipc.mockResolvedValueOnce({
      ...configured,
      model: 'deepseek-reasoner',
    } as never);

    await click('ai-save');
    expect(ipc).toHaveBeenCalledWith('save_ai_provider', expect.objectContaining({
      providerId: 'deepseek',
      model: 'deepseek-reasoner',
      activate: false,
      models: [expect.objectContaining({ id: 'model-a' }), expect.objectContaining({ id: 'deepseek-reasoner' })],
    }));
    expect(store.notify).toHaveBeenCalledWith('DeepSeek 配置已保存', 'success');
  });

  it('follows the protocol with its example address, but keeps an address typed by hand', async () => {
    await open(unconfigured);
    await click('ai-add-custom-card');
    // A brand new custom provider starts empty: its vendor is unknown, so there is no example.
    expect(field('ai-base-url').value).toBe('');

    // Once an example has been taken, following the protocol replaces it.
    await pickProtocol('anthropic');
    expect(field('ai-base-url').value).toBe('https://api.anthropic.com');
    await type('ai-base-url', 'https://api.anthropic.com');
    await pickProtocol('chat');
    expect(field('ai-base-url').value).toBe('https://api.openai.com/v1');
    expect(q('ai-endpoint-preview')!.textContent).toContain('https://api.openai.com/v1/chat/completions');

    // An address the user typed is theirs: switching protocol leaves it alone.
    await type('ai-base-url', 'https://gw.example.com/proxy/anthropic');
    await pickProtocol('responses');
    expect(field('ai-base-url').value).toBe('https://gw.example.com/proxy/anthropic');
    expect(q('ai-endpoint-preview')!.textContent).toContain('https://gw.example.com/proxy/anthropic/responses');
  });

  it('switches a built-in vendor to the address that serves the chosen protocol', async () => {
    await open(unconfigured);
    await openModal('deepseek');
    expect(field('ai-base-url').value).toBe('https://api.deepseek.com');

    await pickProtocol('anthropic');
    expect(field('ai-base-url').value).toBe('https://api.deepseek.com/anthropic');
    expect(q('ai-endpoint-preview')!.textContent).toContain('https://api.deepseek.com/anthropic/v1/messages');

    await pickProtocol('responses');
    expect(field('ai-base-url').value).toBe('https://api.deepseek.com');
    expect(q('ai-endpoint-preview')!.textContent).toContain('https://api.deepseek.com/responses');

    // 智谱 puts each protocol on its own path.
    await openModal('zhipu');
    expect(field('ai-base-url').value).toBe('https://open.bigmodel.cn/api/v1');
    await pickProtocol('anthropic');
    expect(field('ai-base-url').value).toBe('https://open.bigmodel.cn/api/anthropic');
    await pickProtocol('chat');
    expect(field('ai-base-url').value).toBe('https://open.bigmodel.cn/api/paas/v4');
    await pickProtocol('responses');
    expect(field('ai-base-url').value).toBe('https://open.bigmodel.cn/api/v1');
  });

  it('warns about plain http except loopback', async () => {
    await open(unconfigured);
    await click('ai-add-custom-card');
    await type('ai-base-url', 'http://192.168.1.5:8080/v1');
    expect(q('ai-http-warning')!.textContent).toContain('明文');
    await type('ai-base-url', 'http://localhost:11434/v1');
    expect(q('ai-http-warning')).toBeNull();
  });
});

describe('fetching model list in modal', () => {
  it('disables fetch button when base url is empty', async () => {
    await open(unconfigured);
    await click('ai-add-custom-card');
    expect(q<HTMLButtonElement>('ai-fetch-models')?.disabled).toBe(true);
  });

  it('fetches models, displays select dropdown and updates model when chosen', async () => {
    const store = await open(configured);
    await openModal('deepseek');
    const fetchBtn = q<HTMLButtonElement>('ai-fetch-models');
    expect(fetchBtn?.disabled).toBe(false);

    ipc.mockResolvedValueOnce([
      { id: 'deepseek-chat', name: 'DeepSeek Chat' },
      { id: 'deepseek-reasoner', name: 'DeepSeek Reasoner', maxOutputTokens: 65536, contextWindowTokens: 128000 },
      { id: 'deepseek-coder', name: 'DeepSeek Coder' },
    ] as never);
    await click('ai-fetch-models');

    expect(ipc).toHaveBeenLastCalledWith('fetch_ai_models', {
      protocol: 'chat',
      baseUrl: 'https://api.example.com/v1',
      apiKey: null,
    });
    expect(store.notify).toHaveBeenCalledWith('成功获取到 3 个模型', 'success');
    expect(q('ai-model-select-wrapper')).not.toBeNull();

    await click('ai-model-chip-deepseek-reasoner');
    expect(q('ai-model-row-deepseek-reasoner')).not.toBeNull();
    expect(q('ai-model-chip-deepseek-reasoner')).toBeNull();
    await click('ai-model-default-deepseek-reasoner');

    ipc.mockResolvedValueOnce(configured as never);
    saveButton().click();
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('save_ai_provider', expect.objectContaining({
      model: 'deepseek-reasoner',
      discoveredModel: expect.objectContaining({
        id: 'deepseek-reasoner', maxOutputTokens: 65536, contextWindowTokens: 128000,
      }),
      models: expect.arrayContaining([expect.objectContaining({ id: 'deepseek-reasoner', maxOutputTokens: 65536, contextWindowTokens: 128000 })]),
    }));
  });

  it('never saves endpoint metadata after changing the API address', async () => {
    await open(configured);
    await openModal('deepseek');
    ipc.mockResolvedValueOnce([{ id: 'model-a', name: 'model-a', maxOutputTokens: 65536 }] as never);
    await click('ai-fetch-models');
    await type('ai-base-url', 'https://different.example.com/v1');
    ipc.mockResolvedValueOnce(configured as never);
    saveButton().click();
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('save_ai_provider', expect.objectContaining({ discoveredModel: null }));
  });

  it('tests the connection with the list request and reports it inline', async () => {
    const store = await open(configured);
    await openModal('deepseek');
    ipc.mockResolvedValueOnce([{ id: 'a' }, { id: 'b' }] as never);
    await click('ai-test-connection');
    expect(ipc).toHaveBeenLastCalledWith('fetch_ai_models', { protocol: 'chat', baseUrl: 'https://api.example.com/v1', apiKey: null });
    expect(q('ai-test-result')!.textContent).toContain('连接成功');
    expect(store.notify).not.toHaveBeenCalled();
    ipc.mockRejectedValueOnce(new Error('鉴权失败（HTTP 401）'));
    await click('ai-test-connection');
    expect(q('ai-test-result')!.textContent).toContain('鉴权失败');
  });

  it('notifies error when fetch fails', async () => {
    const store = await open(configured);
    await openModal('deepseek');
    ipc.mockRejectedValueOnce(new Error('鉴权失败（HTTP 401）：请检查 API 密钥是否正确'));
    await click('ai-fetch-models');

    expect(store.notify).toHaveBeenCalledWith('鉴权失败（HTTP 401）：请检查 API 密钥是否正确', 'error');
  });
});

describe('global preferences settings', () => {
  it('toggles open-computer-use and saves preferences', async () => {
    await open();
    expect(q('ai-open-computer-use')?.getAttribute('aria-checked')).toBe('false');
    ipc.mockResolvedValueOnce({ ...configured, openComputerUse: true } as never);
    await click('ai-open-computer-use'); // applies at once, no separate save button
    expect(ipc).toHaveBeenCalledWith('save_ai_preferences', expect.objectContaining({ openComputerUse: true }));
  });

  it('resets workspace folder', async () => {
    await open({ ...configured, workspace: '/Users/me/work' });
    expect(q('ai-workspace')?.textContent).toBe('/Users/me/work');
    ipc.mockResolvedValueOnce({ ...configured, workspace: '' } as never);
    await click('ai-workspace-reset');
    expect(ipc).toHaveBeenCalledWith('save_ai_preferences', expect.objectContaining({ workspace: '' }));
    expect(q('ai-workspace')?.textContent).toBe('/data/workspace');
  });
});

describe('reply preferences', () => {
  const known: AiConfigView = {
    ...configured, activeProvider: 'custom_openai', protocol: 'responses',
    baseUrl: 'https://api.openai.com/v1', model: 'gpt-6.1-sol',
    providers: [{ id: 'custom_openai', name: '我的模型', isCustom: true, protocol: 'responses', baseUrl: 'https://api.openai.com/v1', model: 'gpt-6.1-sol', hasKey: true, keyHint: '', replyPreference: 'deeper' }],
  };

  it('keeps advanced settings folded and saves the selected provider preference', async () => {
    await open(known);
    await openModal('custom_openai');
    expect(q('ai-reply-advanced')).not.toBeNull();
    const select = wrapper!.findAllComponents(NSelect).find(s => s.attributes('data-testid') === 'ai-reply-preference')!;
    expect(select.props('value')).toBe('deeper');
    select.vm.$emit('update:value', 'faster');
    await flushPromises();
    ipc.mockResolvedValueOnce(known as never);
    saveButton().click();
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('save_ai_provider', expect.objectContaining({ providerId: 'custom_openai', replyPreference: 'faster' }));
  });

  it('keeps supported preferences when switching to a proxy', async () => {
    await open(known);
    await openModal('custom_openai');
    await type('ai-base-url', 'https://gateway.example/v1');
    expect(q('ai-reply-preference')).not.toBeNull();
    expect(q('ai-max-output-tokens')).toBeNull(); // the output limit is automatic
    ipc.mockResolvedValueOnce(known as never);
    saveButton().click();
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('save_ai_provider', expect.objectContaining({ replyPreference: 'deeper' }));
  });

  it('allows declaring capabilities for unknown models', async () => {
    await open(configured);
    await openModal('deepseek');
    expect(q('ai-reply-advanced')).not.toBeNull();
    expect(q('ai-reasoning-mode')).toBeNull(); // folded until the model's settings are opened
    await expandModel('model-a');
    expect(q('ai-reasoning-mode')).not.toBeNull();
  });

  it('persists declared grades and a selected effort for an alias model', async () => {
    const alias = structuredClone(known);
    alias.model = 'private-alias';
    alias.providers![0].model = 'private-alias';
    alias.providers![0].replyPreference = 'default';
    await open(alias);
    await openModal('custom_openai');
    const select = (id: string) => wrapper!.findAllComponents(NSelect).find(s => s.attributes('data-testid') === id)!;
    await expandModel('private-alias');
    await click('ai-reasoning-mode-manual');
    for (const level of ['low', 'max', 'medium']) await click(`ai-reasoning-level-${level}`); // default low/high/max -> medium/high
    expect(q('ai-model-row-private-alias')!.textContent).toContain('思考等级：中 / 高');
    expect(select('ai-reply-preference').props('options')).toEqual([
      {value:'default',label:'默认'}, {value:'medium',label:'中'}, {value:'deeper',label:'高 · 更深入'},
    ]);
    select('ai-reply-preference').vm.$emit('update:value', 'medium');
    await flushPromises();
    // A second model keeps its own, separate declaration (here: none, detected from the name).
    await addModel('second-model');
    ipc.mockResolvedValueOnce(alias as never);
    saveButton().click();
    await flushPromises();
    const sent = ipc.mock.calls.find(c => c[0] === 'save_ai_provider')![1] as { models: Record<string, unknown>[] };
    expect(ipc).toHaveBeenCalledWith('save_ai_provider', expect.objectContaining({
      model:'private-alias', clearReasoningEfforts:true, replyPreference:'medium',
    }));
    expect(sent.models.map(m => [m.id, m.reasoningEfforts])).toEqual([['private-alias', ['medium','high']], ['second-model', undefined]]);
  });

  it('saves several models, a model turned off, and drops a removed one', async () => {
    await open(known);
    await openModal('custom_openai');
    await addModel('m-two');
    await addModel('m-three');
    await expandModel('m-two');
    await click('ai-reasoning-mode-off');
    await click('ai-model-remove-m-three');
    ipc.mockResolvedValueOnce(known as never);
    saveButton().click();
    await flushPromises();
    const sent = ipc.mock.calls.find(c => c[0] === 'save_ai_provider')![1] as { model: string; models: Record<string, unknown>[] };
    expect(sent.model).toBe('gpt-6.1-sol');
    expect(sent.models.map(m => [m.id, m.reasoningEfforts])).toEqual([['gpt-6.1-sol', undefined], ['m-two', []]]);
  });

  it('uses automatic output management for Anthropic and hides manual token controls', async () => {
    await open(known);
    await openModal('custom_openai');
    await pickProtocol('anthropic');
    expect(q('ai-reply-advanced')).not.toBeNull();
    expect(q('ai-max-output-tokens')).toBeNull();
    expect(q('ai-reply-preference')).toBeNull();
    expect(q('ai-model-expand-gpt-6.1-sol')).toBeNull();
    expect(q('ai-reasoning-mode')).toBeNull();
  });

  it('never saves a manual output limit, even when loading a legacy configuration', async () => {
    const initial = structuredClone(known) as typeof known & { providers: Array<{ maxOutputTokens?: number }> };
    initial.providers[0].maxOutputTokens = 1024;
    await open(initial);
    await openModal('custom_openai');
    expect(q('ai-max-output-tokens')).toBeNull();
    ipc.mockResolvedValueOnce(initial as never);
    saveButton().click();
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('save_ai_provider', expect.not.objectContaining({
      maxOutputTokens: expect.anything(),
    }));
  });
});
