// @vitest-environment happy-dom
import { mount, flushPromises } from '@vue/test-utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { NSwitch } from 'naive-ui';
import { reactive } from 'vue';
import AssistantPage from './AssistantPage.vue';
import { defaults } from '../../bridge.ts';
import type { ControlPage, ControlStore } from '../control-types.ts';

const status = {
  enabled: true, shortcut: 'Alt+Space', shortcutLabel: '⌥ Space', ball: true, open: 'voice', read: 'voice', hintSeen: true, registered: true, error: '',
  shortcuts: [{ value: 'Alt+Space', label: '⌥ Space' }, { value: 'Ctrl+Alt+Space', label: '⌃ ⌥ Space' }],
};
const invoke = vi.fn();
vi.mock('../../bridge.ts', async importOriginal => ({ ...(await importOriginal<typeof import('../../bridge.ts')>()), invoke: (...args: unknown[]) => invoke(...args) }));

function setup(voice: Record<string, unknown> = {}, settings: Record<string, unknown> = {}, platform = 'macos') {
  const asr = reactive({ enabled: true, modelReady: true, engineReady: true, warming: false, preparing: false, phase: 'idle', downloaded: 0, total: 100, error: '', saving: false,
    init: vi.fn(async () => {}), setEnabled: vi.fn(async (enabled: boolean) => { asr.enabled = enabled; }), retry: vi.fn(async () => {}), ...voice });
  const store = reactive({
    desktop: true, info: { platform }, settings: { ...JSON.parse(JSON.stringify(defaults)), autostart: true, ...settings }, asr, settingsTab: 'general', page: 'assistant',
    notify: vi.fn(), refreshSettings: vi.fn(async () => {}), navigate: vi.fn((page: ControlPage) => { store.page = page; }),
  }) as unknown as ControlStore;
  return { wrapper: mount(AssistantPage, { props: { store }, global: { stubs: {
    NModal: { name: 'NModal', props: ['show'], template: '<div v-if="show" data-testid="capture-reset-dialog"><slot /></div>' },
  } } }), store, asr };
}
const click = async (wrapper: ReturnType<typeof setup>['wrapper'], id: string) => { await wrapper.get(`[data-testid="${id}"]`).trigger('click'); await flushPromises(); };

describe('assistant page', () => {
  beforeEach(() => { invoke.mockReset(); invoke.mockImplementation(async (command: string) => (command === 'assistant_status' || command === 'assistant_configure' ? status : undefined)); });

  it('only offers the reset fallback on macOS and does not run tccutil without confirmation', async () => {
    const windows = setup({}, {}, 'windows');
    await flushPromises();
    expect(windows.wrapper.find('[data-testid="assistant-capture-settings"]').exists()).toBe(false);
    const mac = setup();
    await flushPromises();
    expect(mac.wrapper.find('[data-testid="assistant-capture-settings"]').exists()).toBe(true);
    await click(mac.wrapper, 'assistant-capture-reset-tcc');
    expect(mac.wrapper.find('[data-testid="capture-reset-dialog"]').exists()).toBe(true);
    expect(invoke).not.toHaveBeenCalledWith('reset_screen_capture_permission');
    await click(mac.wrapper, 'assistant-capture-reset-cancel');
    expect(mac.wrapper.find('[data-testid="capture-reset-dialog"]').exists()).toBe(false);
    expect(invoke).not.toHaveBeenCalledWith('reset_screen_capture_permission');
  });

  it('resets only after confirming, then tells the user to quit and reauthorize', async () => {
    const { wrapper, store } = setup();
    await flushPromises();
    await click(wrapper, 'assistant-capture-reset-tcc');
    await click(wrapper, 'assistant-capture-reset-confirm');
    expect(invoke).toHaveBeenCalledWith('reset_screen_capture_permission');
    expect(wrapper.get('[data-testid="assistant-capture-reset-notice"]').text()).toContain('完全退出');
    expect(store.notify).toHaveBeenCalledWith(expect.stringContaining('重新授权'));
    expect(wrapper.find('[data-testid="capture-reset-dialog"]').exists()).toBe(false);
  });

  it('reports reset failure without claiming permissions were cleared', async () => {
    invoke.mockImplementation(async (command: string) => {
      if (command === 'assistant_status') return status;
      if (command === 'reset_screen_capture_permission') throw new Error('tccutil failed');
    });
    const { wrapper, store } = setup();
    await flushPromises();
    await click(wrapper, 'assistant-capture-reset-tcc');
    await click(wrapper, 'assistant-capture-reset-confirm');
    expect(store.notify).toHaveBeenCalledWith(expect.any(Error), 'error');
    expect(wrapper.find('[data-testid="assistant-capture-reset-notice"]').exists()).toBe(false);
    expect(wrapper.find('[data-testid="capture-reset-dialog"]').exists()).toBe(true);
  });

  it('can open only the screen recording privacy settings', async () => {
    const { wrapper } = setup();
    await flushPromises();
    await click(wrapper, 'assistant-capture-open-settings');
    expect(invoke).toHaveBeenCalledWith('open_screen_capture_settings');
  });

  it('shows the shortcut the host reports and whether it took effect', async () => {
    const { wrapper } = setup();
    await flushPromises();
    expect(wrapper.get('[data-testid="assistant-shortcut-label"]').text()).toBe('⌥ Space');
    expect(wrapper.find('[data-testid="assistant-registered"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="assistant-shortcut-error"]').exists()).toBe(false);
  });

  it('warns when the shortcut could not be registered', async () => {
    invoke.mockImplementation(async (command: string) => (command === 'assistant_status' ? { ...status, registered: false, error: '快捷键 ⌥ Space 无法使用' } : undefined));
    const { wrapper } = setup();
    await flushPromises();
    expect(wrapper.get('[data-testid="assistant-shortcut-error"]').text()).toContain('无法使用');
    expect(wrapper.find('[data-testid="assistant-registered"]').exists()).toBe(false);
  });

  it('opens the capsule on "try it now"', async () => {
    const { wrapper } = setup();
    await flushPromises();
    await click(wrapper, 'assistant-try');
    expect(invoke).toHaveBeenCalledWith('assistant_try');
  });

  it('sends only what changed, and refreshes the app settings afterwards', async () => {
    const { wrapper, store } = setup();
    await flushPromises();
    await click(wrapper, 'assistant-reset');
    expect(invoke).toHaveBeenCalledWith('assistant_configure', { patch: { resetPosition: true } });
    expect(store.refreshSettings).toHaveBeenCalled();
  });

  it('turns speech on and off through the speech store, not the assistant host', async () => {
    const { wrapper, asr } = setup({ enabled: false, modelReady: false, engineReady: false });
    await flushPromises();
    expect(wrapper.get('[data-testid="assistant-voice-line"]').text()).toContain('后台自动下载');
    const voice = wrapper.findAllComponents(NSwitch).find(item => item.attributes('aria-label') === '用语音提问');
    expect(voice).toBeTruthy();
    voice!.vm.$emit('update:value', true);
    await flushPromises();
    expect(asr.setEnabled).toHaveBeenCalledWith(true);
    expect(invoke).not.toHaveBeenCalledWith('assistant_configure', expect.objectContaining({ patch: expect.objectContaining({ enabled: true }) }));
  });

  it('says in plain words what the speech engine is doing', async () => {
    const line = async (voice: Record<string, unknown>) => {
      const { wrapper } = setup(voice);
      await flushPromises();
      return wrapper.get('[data-testid="assistant-voice-line"]').text();
    };
    expect(await line({ engineReady: true })).toContain('已就绪');
    expect(await line({ engineReady: false, warming: true })).toContain('后台准备');
    expect(await line({ engineReady: false, preparing: true, phase: 'downloading' })).toContain('正在下载');
    expect(await line({ engineReady: false, modelReady: false })).toContain('后台自动下载');
    expect(await line({ error: '魔搭模型下载失败' })).toContain('魔搭模型下载失败');
    // speech off: the model is fetched for everyone, so the page says where that stands
    expect(await line({ enabled: false, modelReady: true, engineReady: false })).toContain('模型已经准备好');
    expect(await line({ enabled: false, modelReady: false, engineReady: false, warming: true })).toContain('正在后台下载');
  });

  it('offers a retry only when speech failed', async () => {
    const failed = setup({ error: '下载失败' });
    await flushPromises();
    await click(failed.wrapper, 'assistant-voice-retry');
    expect(failed.asr.retry).toHaveBeenCalled();
    const fine = setup();
    await flushPromises();
    expect(fine.wrapper.find('[data-testid="assistant-voice-retry"]').exists()).toBe(false);
  });

  it('points out that the shortcut needs the app running when it does not start with the computer', async () => {
    const off = setup({}, { autostart: false });
    await flushPromises();
    expect(off.wrapper.find('[data-testid="assistant-autostart-note"]').exists()).toBe(true);
    const on = setup();
    await flushPromises();
    expect(on.wrapper.find('[data-testid="assistant-autostart-note"]').exists()).toBe(false);
  });
});
