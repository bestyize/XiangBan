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

const saveButton = () => q<HTMLButtonElement>('ai-save')!;
const pickProtocol = async (value: string) => {
  const selects = wrapper!.findAllComponents(NSelect);
  const protoSelect = selects.find(s => s.attributes('data-testid') === 'ai-protocol');
  protoSelect?.vm.$emit('update:value', value);
  await flushPromises();
};

beforeEach(() => { vi.resetAllMocks(); });
afterEach(() => { wrapper?.unmount(); wrapper = undefined; });

describe('AI model settings card and provider templates', () => {
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

  it('shows what is saved in modal and never the key', async () => {
    await open();
    await openModal('deepseek');
    expect(field('ai-base-url').value).toBe('https://api.example.com/v1');
    expect(field('ai-model').value).toBe('model-a');
    expect(field('ai-api-key').value).toBe('');
    expect(field('ai-api-key').placeholder).toContain('••••9876');
    expect(q('ai-endpoint-preview')!.textContent).toContain('https://api.example.com/v1/chat/completions');
    expect(document.body.textContent).not.toContain(KEY);
  });

  it('prompts to configure when trying to activate unconfigured provider', async () => {
    const store = await open();
    await click('ai-provider-activate-zhipu');
    expect(store.notify).toHaveBeenCalledWith('该服务商尚未完整配置（缺少密钥或地址），请先配置', 'info');
    expect(q('ai-provider-modal')?.style.display).not.toBe('none');
  });

  it('opens modal to create a custom provider with custom name field', async () => {
    await open();
    await click('ai-add-custom-card');
    expect(q('ai-provider-name')).not.toBeNull();
    expect(field('ai-provider-name').value).toBe('自定义供应商');

    await type('ai-provider-name', '本地 Ollama');
    await type('ai-base-url', 'http://localhost:11434/v1');
    await type('ai-model', 'llama3');

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

    await type('ai-model', 'model-a');
    expect(q('ai-form-error')).toBeNull();
    expect(saveButton().disabled).toBe(false);

    await type('ai-api-key', 'sk bad key');
    expect(q('ai-form-error')!.textContent).toContain('空格');
    expect(saveButton().disabled).toBe(true);
  });

  it('saves provider configuration via modal', async () => {
    const store = await open(configured);
    await openModal('deepseek');
    await type('ai-model', 'deepseek-reasoner');

    ipc.mockResolvedValueOnce({
      ...configured,
      model: 'deepseek-reasoner',
    } as never);

    await click('ai-save');
    expect(ipc).toHaveBeenCalledWith('save_ai_provider', expect.objectContaining({
      providerId: 'deepseek',
      model: 'deepseek-reasoner',
      activate: false,
    }));
    expect(store.notify).toHaveBeenCalledWith('DeepSeek 配置已保存', 'success');
    expect(q('ai-provider-modal')?.style.display).toBe('none');
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
    expect(field('ai-base-url').value).toBe('https://open.bigmodel.cn/api/paas/v4');
    await pickProtocol('anthropic');
    expect(field('ai-base-url').value).toBe('https://open.bigmodel.cn/api/anthropic');
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

    ipc.mockResolvedValueOnce(['deepseek-chat', 'deepseek-reasoner', 'deepseek-coder'] as never);
    await click('ai-fetch-models');

    expect(ipc).toHaveBeenLastCalledWith('fetch_ai_models', {
      protocol: 'chat',
      baseUrl: 'https://api.example.com/v1',
      apiKey: null,
    });
    expect(store.notify).toHaveBeenCalledWith('成功获取到 3 个模型', 'success');
    expect(q('ai-model-select-wrapper')).not.toBeNull();

    const selects = wrapper!.findAllComponents(NSelect);
    const modelSelect = selects.find(s => s.attributes('data-testid') === 'ai-model-select');
    expect(modelSelect).toBeDefined();
    modelSelect!.vm.$emit('update:value', 'deepseek-reasoner');
    await flushPromises();

    expect(field('ai-model').value).toBe('deepseek-reasoner');
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
    await q('ai-open-computer-use')?.click();
    await flushPromises();
    expect(q<HTMLButtonElement>('ai-save-preferences')?.disabled).toBe(false);

    ipc.mockResolvedValueOnce({ ...configured, openComputerUse: true } as never);
    await click('ai-save-preferences');
    expect(ipc).toHaveBeenCalledWith('save_ai_preferences', expect.objectContaining({ openComputerUse: true }));
  });

  it('resets workspace folder', async () => {
    await open({ ...configured, workspace: '/Users/me/work' });
    expect(q('ai-workspace')?.textContent).toBe('/Users/me/work');
    await click('ai-workspace-reset');
    expect(q('ai-workspace')?.textContent).toBe('/data/workspace');
  });
});
