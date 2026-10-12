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
const configured: AiConfigView = { protocol: 'chat', baseUrl: 'https://api.example.com/v1', model: 'model-a', hasKey: true, keyHint: '••••9876', workspace: '', defaultWorkspace: '/data/workspace', permissionMode: 'ask', hideToolCalls: false, openComputerUse: false };
const unconfigured: AiConfigView = { protocol: 'chat', baseUrl: '', model: '', hasKey: false, keyHint: '', workspace: '', defaultWorkspace: '/data/workspace', permissionMode: 'ask', hideToolCalls: false, openComputerUse: false };

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
const q = <T extends Element = HTMLElement>(testid: string) => wrapper!.element.querySelector(`[data-testid="${testid}"]`) as T | null;
const field = (testid: string) => wrapper!.element.querySelector(`[data-testid="${testid}"] input`) as HTMLInputElement;
async function type(testid: string, value: string) {
  const input = field(testid);
  input.value = value;
  input.dispatchEvent(new Event('input'));
  await flushPromises();
}
async function click(testid: string) { q<HTMLButtonElement>(testid)!.click(); await flushPromises(); }
const saveButton = () => q<HTMLButtonElement>('ai-save')!;
const pickProtocol = async (value: string) => { wrapper!.findComponent(NSelect).vm.$emit('update:value', value); await flushPromises(); };

beforeEach(() => { vi.resetAllMocks(); });
afterEach(() => { wrapper?.unmount(); wrapper = undefined; });

describe('AI model settings card', () => {
  it('shows what is saved and never the key', async () => {
    await open();
    expect(field('ai-base-url').value).toBe('https://api.example.com/v1');
    expect(field('ai-model').value).toBe('model-a');
    expect(field('ai-api-key').value).toBe('');
    expect(field('ai-api-key').placeholder).toContain('••••9876');
    expect(q('ai-status')!.textContent).toBe('已配置：model-a');
    expect(q('ai-endpoint-preview')!.textContent).toContain('https://api.example.com/v1/chat/completions');
    expect(wrapper!.element.textContent).not.toContain(KEY);
    expect(ipc).toHaveBeenCalledTimes(1);
  });

  it('starts empty and unconfigured with nothing to save', async () => {
    await open(unconfigured);
    expect(q('ai-status')!.textContent).toBe('尚未配置');
    expect(field('ai-api-key').placeholder).toContain('可以不填');
    expect(saveButton().disabled).toBe(true);
    expect(q('ai-form-error')).toBeNull();
    expect(q('ai-clear-key')).toBeNull();
    expect(q('ai-endpoint-preview')).toBeNull();
  });

  it('keeps the button off until the form is valid and changed, and says what is wrong', async () => {
    await open(unconfigured);
    await type('ai-base-url', 'api.example.com');
    expect(q('ai-form-error')!.textContent).toContain('格式不对');
    expect(q('ai-endpoint-preview'), 'no address is promised for an invalid one').toBeNull();
    await type('ai-base-url', 'https://api.example.com/v1');
    expect(q('ai-form-error')!.textContent).toContain('模型名');
    expect(saveButton().disabled).toBe(true);
    await type('ai-model', 'model-a');
    expect(q('ai-form-error')).toBeNull();
    expect(saveButton().disabled).toBe(false);
    await type('ai-api-key', 'sk bad');
    expect(q('ai-form-error')!.textContent).toContain('空格');
    expect(saveButton().disabled).toBe(true);
  });

  it('shows the real address for each protocol and starts from its example', async () => {
    await open(unconfigured);
    expect(field('ai-base-url').placeholder).toBe('https://api.openai.com/v1');
    await pickProtocol('anthropic');
    expect(field('ai-base-url').placeholder).toBe('https://api.anthropic.com');
    await type('ai-base-url', 'https://api.anthropic.com/');
    expect(q('ai-endpoint-preview')!.textContent).toContain('https://api.anthropic.com/v1/messages');
    await pickProtocol('responses');
    expect(q('ai-endpoint-preview')!.textContent).toContain('https://api.anthropic.com/responses');
  });

  it('warns about plain http except to this computer', async () => {
    await open(unconfigured);
    await type('ai-base-url', 'http://192.168.1.5:8080/v1');
    expect(q('ai-http-warning')!.textContent).toContain('明文');
    await type('ai-base-url', 'http://localhost:11434/v1');
    expect(q('ai-http-warning')).toBeNull();
    await type('ai-base-url', 'https://api.example.com/v1');
    expect(q('ai-http-warning')).toBeNull();
  });

  it('saves trimmed values, leaves the key alone when none is typed, and reports it', async () => {
    const store = await open(unconfigured);
    ipc.mockResolvedValueOnce({ ...configured, hasKey: false, keyHint: '' } as never);
    await type('ai-base-url', '  https://api.example.com/v1/  ');
    await type('ai-model', ' model-a ');
    await click('ai-save');
    expect(ipc).toHaveBeenLastCalledWith('save_ai_config', { protocol: 'chat', baseUrl: 'https://api.example.com/v1/', model: 'model-a', apiKey: null, clearKey: false, workspace: null, permissionMode: 'ask', hideToolCalls: false, openComputerUse: false });
    expect(store.notify).toHaveBeenCalledWith('模型设置已保存', 'success');
    expect(field('ai-base-url').value).toBe('https://api.example.com/v1');
    expect(saveButton().disabled).toBe(true);
  });

  it('sends a typed key once, then drops it from the page', async () => {
    await open();
    ipc.mockResolvedValueOnce({ ...configured, keyHint: '••••WXYZ' } as never);
    await type('ai-api-key', `  ${KEY}\n`);
    expect(saveButton().disabled).toBe(false);
    await click('ai-save');
    expect(ipc).toHaveBeenLastCalledWith('save_ai_config', expect.objectContaining({ apiKey: KEY, clearKey: false }));
    expect(field('ai-api-key').value).toBe('');
    expect(field('ai-api-key').placeholder).toContain('••••WXYZ');
    expect(wrapper!.element.textContent).not.toContain(KEY);
    expect(document.body.innerHTML).not.toContain(KEY);
  });

  it('deletes the saved key after one more click, using what is saved and keeping unsaved edits', async () => {
    const store = await open();
    await type('ai-model', 'model-b');
    await click('ai-clear-key');
    expect(ipc).toHaveBeenCalledTimes(1);
    expect(q('ai-clear-confirm')).not.toBeNull();
    await click('ai-clear-cancel');
    expect(q('ai-clear-key')).not.toBeNull();
    expect(ipc).toHaveBeenCalledTimes(1);

    ipc.mockResolvedValueOnce({ ...configured, hasKey: false, keyHint: '' } as never);
    await click('ai-clear-key');
    await click('ai-clear-confirm');
    expect(ipc).toHaveBeenLastCalledWith('save_ai_config', { protocol: 'chat', baseUrl: 'https://api.example.com/v1', model: 'model-a', apiKey: null, clearKey: true, workspace: null, permissionMode: 'ask', hideToolCalls: false, openComputerUse: false });
    expect(store.notify).toHaveBeenCalledWith('已删除保存的密钥', 'success');
    expect(q('ai-clear-key')).toBeNull();
    expect(field('ai-model').value).toBe('model-b');
    expect(field('ai-api-key').placeholder).toContain('可以不填');
  });

  it('keeps the form and tells the user when the host refuses', async () => {
    const store = await open(unconfigured);
    await type('ai-base-url', 'https://api.example.com/v1');
    await type('ai-model', 'model-a');
    ipc.mockRejectedValueOnce('无法保存模型配置：磁盘已满');
    await click('ai-save');
    expect(store.notify).toHaveBeenCalledWith('无法保存模型配置：磁盘已满', 'error');
    expect(field('ai-base-url').value).toBe('https://api.example.com/v1');
    expect(saveButton().disabled).toBe(false);
    expect(q('ai-status')!.textContent).toBe('尚未配置');
  });

  it('lets a damaged saved file be written over', async () => {
    await open(new Error('模型配置读取失败：expected value at line 1 column 1'));
    expect(q('ai-load-error')!.textContent).toContain('读取失败');
    ipc.mockResolvedValueOnce(configured as never);
    await type('ai-base-url', 'https://api.example.com/v1');
    await type('ai-model', 'model-a');
    await click('ai-save');
    expect(ipc).toHaveBeenLastCalledWith('save_ai_config', expect.objectContaining({ baseUrl: 'https://api.example.com/v1' }));
    expect(q('ai-load-error')).toBeNull();
  });

  it('is read-only in the browser preview and never calls the host', async () => {
    await open(unconfigured, createStore(false));
    expect(ipc).not.toHaveBeenCalled();
    expect(wrapper!.element.textContent).toContain('请在桌面应用中配置');
    for (const id of ['ai-base-url', 'ai-api-key', 'ai-model']) expect(field(id).disabled).toBe(true);
    expect(saveButton().disabled).toBe(true);
  });
});

describe('AI model settings card: working folder', () => {
  const workspace = () => q('ai-workspace')!.textContent;

  it('shows the built-in folder until another one is chosen, and always warns that commands are not confined to it', async () => {
    await open();
    expect(workspace()).toBe('/data/workspace');
    expect(q('ai-workspace-reset')).toBeNull();
    expect(q('ai-workspace-note')!.textContent).toContain('命令不受这个文件夹限制');
  });

  it('keeps a picked folder as a pending change and saves it with the rest', async () => {
    await open();
    ipc.mockResolvedValueOnce('/Users/me/Documents/小伴' as never);
    await click('ai-workspace-pick');
    expect(ipc).toHaveBeenLastCalledWith('pick_workspace');
    expect(workspace()).toBe('/Users/me/Documents/小伴');
    expect(saveButton().disabled).toBe(false);
    expect(q('ai-workspace-reset')).not.toBeNull();

    ipc.mockResolvedValueOnce({ ...configured, workspace: '/Users/me/Documents/小伴' } as never);
    await click('ai-save');
    expect(ipc).toHaveBeenLastCalledWith('save_ai_config', expect.objectContaining({ workspace: '/Users/me/Documents/小伴', apiKey: null }));
    expect(workspace()).toBe('/Users/me/Documents/小伴');
    expect(saveButton().disabled).toBe(true);
  });

  it('a dismissed picker changes nothing, and an unchanged folder is not sent', async () => {
    await open();
    ipc.mockResolvedValueOnce(null as never);
    await click('ai-workspace-pick');
    expect(workspace()).toBe('/data/workspace');
    expect(saveButton().disabled).toBe(true);
    ipc.mockResolvedValueOnce(configured as never);
    await type('ai-model', 'model-b');
    await click('ai-save');
    expect(ipc).toHaveBeenLastCalledWith('save_ai_config', expect.objectContaining({ workspace: null }));
  });

  it('goes back to the built-in folder with one click, which is then saved as empty', async () => {
    await open({ ...configured, workspace: '/Users/me/work' });
    expect(workspace()).toBe('/Users/me/work');
    await click('ai-workspace-reset');
    expect(workspace()).toBe('/data/workspace');
    expect(saveButton().disabled).toBe(false);
    ipc.mockResolvedValueOnce(configured as never);
    await click('ai-save');
    expect(ipc).toHaveBeenLastCalledWith('save_ai_config', expect.objectContaining({ workspace: '' }));
    expect(q('ai-workspace-reset')).toBeNull();
  });

  it('shows the host\'s refusal for a folder that disappeared and keeps the form', async () => {
    const store = await open({ ...configured, workspace: '/gone' });
    await type('ai-model', 'model-b');
    ipc.mockRejectedValueOnce('这个文件夹不存在，或者不是文件夹');
    await click('ai-save');
    expect(store.notify).toHaveBeenCalledWith('这个文件夹不存在，或者不是文件夹', 'error');
    expect(workspace()).toBe('/gone');
  });

  it('cannot pick a folder in the browser preview', async () => {
    await open(unconfigured, createStore(false));
    expect((q('ai-workspace-pick') as HTMLButtonElement).disabled).toBe(true);
  });
});

describe('hiding tool calls setting', () => {
  it('toggles hiding tool calls and saves it', async () => {
    await open();
    expect(q('ai-hide-tool-calls')?.getAttribute('aria-checked')).toBe('false');
    await q('ai-hide-tool-calls')?.click();
    await flushPromises();
    expect(saveButton().disabled).toBe(false);
    ipc.mockResolvedValueOnce({ ...configured, hideToolCalls: true } as never);
    await click('ai-save');
    expect(ipc).toHaveBeenLastCalledWith('save_ai_config', expect.objectContaining({ hideToolCalls: true }));
  });
});

describe('open-computer-use setting', () => {
  it('toggles open-computer-use and saves it', async () => {
    await open();
    expect(q('ai-open-computer-use')?.getAttribute('aria-checked')).toBe('false');
    await q('ai-open-computer-use')?.click();
    await flushPromises();
    expect(saveButton().disabled).toBe(false);
    ipc.mockResolvedValueOnce({ ...configured, openComputerUse: true } as never);
    await click('ai-save');
    expect(ipc).toHaveBeenLastCalledWith('save_ai_config', expect.objectContaining({ openComputerUse: true }));
  });
});
