// @vitest-environment happy-dom
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import { reactive } from 'vue';
import { afterEach, expect, it, vi } from 'vitest';
import ChatPage from '../pages/ChatPage.vue';
import VoiceSettingsCard from '../components/VoiceSettingsCard.vue';
import { createChatStore, type ChatDeps } from '../chat/chat-store.ts';
import { createSpeechStore, speechDefaults } from '../speech/store.ts';
import { createAsrStore, type AsrDeps } from './store.ts';
import type { ControlStore } from '../control-types.ts';

function deferred<T>() { let resolve!: (value: T) => void; const promise = new Promise<T>(r => { resolve = r; }); return { promise, resolve }; }
const cleanup: (() => void)[] = [];
afterEach(() => { cleanup.splice(0).reverse().forEach(fn => fn()); });
async function open(options: { enabled?: boolean; prepare?: () => Promise<unknown>; transcribe?: () => Promise<string> } = {}) {
  let enabled = options.enabled ?? true;
  let onFrame: ((frame: Float32Array) => void) | undefined;
  const stop = vi.fn(), notify = vi.fn();
  const summary = (id: string) => ({ id, title: id, createdAt: 1, updatedAt: 1 });
  const invoke = vi.fn(async (command: string, args?: Record<string, unknown>) => {
    if (command === 'chat_overview') return { configured: true, sessions: [summary('A'), summary('B')], running: [], workspace: '', permissionMode: 'ask', hideToolCalls: false };
    if (command === 'chat_load_session') return [];
    if (command === 'chat_send') return summary(args!.sessionId as string);
    if (command === 'tts_get_settings') return { ...speechDefaults };
    if (command === 'tts_cancel') return;
    if (command === 'asr_get_status') return { enabled, modelReady: enabled, engineReady: false };
    if (command === 'asr_set_enabled') { enabled = args!.enabled as boolean; return { enabled, modelReady: true, engineReady: false }; }
    if (command === 'asr_prepare') return options.prepare ? options.prepare() : { enabled, modelReady: true, engineReady: true };
    if (command === 'asr_transcribe') return options.transcribe ? options.transcribe() : '帮我查一下天气。';
    throw new Error(command);
  });
  const listen: AsrDeps['listen'] = async () => () => {};
  const capture = vi.fn(async (callback: (frame: Float32Array) => void) => { onFrame = callback; return { stop }; });
  const asr = createAsrStore({ desktop: true, invoke: invoke as AsrDeps['invoke'], listen, notify, capture });
  const chat = createChatStore({ desktop: true, invoke: invoke as ChatDeps['invoke'], listen, notify, onActivity: () => asr.deactivate() });
  const speech = createSpeechStore({ desktop: true, invoke: invoke as AsrDeps['invoke'], notify });
  const store = reactive({ desktop: true, chat, speech, asr, settingsTab: 'voice', page: 'chat', notify, navigate: vi.fn() }) as unknown as ControlStore;
  let page: VueWrapper = mount(ChatPage, { props: { store } });
  const settings = mount(VoiceSettingsCard, { props: { store } });
  cleanup.push(() => { page.unmount(); settings.unmount(); asr.dispose(); chat.dispose(); speech.dispose(); });
  await flushPromises(); await chat.select('A'); await flushPromises();
  const pressed = () => page.get('[data-testid="chat-asr-toggle"]').attributes('aria-pressed');
  const clickMic = async () => { await page.get('[data-testid="chat-asr-toggle"]').trigger('click'); await flushPromises(); };
  function phrase() { for (let i = 0; i < 3; i++) onFrame!(new Float32Array(1600).fill(0.1)); for (let i = 0; i < 9; i++) onFrame!(new Float32Array(1600)); }
  const input = () => (page.get('[data-testid="chat-input"] textarea').element as HTMLTextAreaElement).value;
  return { store, asr, chat, speech, settings, invoke, capture, stop, notify, pressed, clickMic, phrase, input,
    page: () => page,
    remount: async () => { page.unmount(); page = mount(ChatPage, { props: { store } }); await flushPromises(); },
  };
}

it('keeps the microphone off when entering a chat with global ASR already enabled', async () => {
  const h = await open(); expect(h.asr.enabled).toBe(true); expect(h.pressed()).toBe('false');
  expect(h.page().get('[data-testid="chat-asr-toggle"]').attributes('aria-label')).toBe('开始录音');
  expect(h.page().find('[data-testid="asr-status"]').exists()).toBe(false);
  expect(h.capture).not.toHaveBeenCalled(); expect(h.invoke.mock.calls.map(([c]) => c)).not.toContain('asr_prepare');
});

it('puts recognized text in the box after ending the recording, and leaves sending to the user', async () => {
  const h = await open({ enabled: false });
  await h.settings.get('[data-testid="asr-settings-switch"]').trigger('click'); await flushPromises();
  expect(h.asr.engineReady).toBe(true); expect(h.pressed()).toBe('false'); expect(h.capture).not.toHaveBeenCalled();
  await h.clickMic(); expect(h.asr.listening).toBe(true);
  expect(h.page().get('[data-testid="chat-asr-toggle"]').text()).toBe('结束录音');
  expect(h.page().get('[data-testid="asr-status"]').text()).toContain('正在录音');
  expect(h.page().get('.composer-box').classes()).toContain('voice-active');
  h.phrase(); await flushPromises(); expect(h.input()).toBe('帮我查一下天气。');
  expect(h.invoke.mock.calls.map(([command]) => command)).not.toContain('chat_send');
  await h.clickMic(); expect(h.pressed()).toBe('false'); expect(h.asr.enabled).toBe(true);
  // The text stays in the box for the user to read and send; ending a recording never sends.
  expect(h.invoke.mock.calls.map(([command]) => command)).not.toContain('chat_send');
  expect(h.input()).toBe('帮我查一下天气。');
  expect(h.page().find('[data-testid="asr-status"]').exists()).toBe(false);
  expect(h.settings.get('[data-testid="asr-settings-switch"]').attributes('aria-checked')).toBe('true');
  expect(h.stop).toHaveBeenCalledOnce();
});

it('waits for an in-flight transcription before the text reaches the box', async () => {
  const pending = deferred<string>(); const h = await open({ transcribe: () => pending.promise });
  await h.clickMic(); h.phrase(); await flushPromises();
  await h.clickMic();
  expect(h.page().get('[data-testid="chat-asr-toggle"]').text()).toBe('正在转写');
  expect(h.input()).toBe('');
  pending.resolve('稍后完成的语音'); await flushPromises();
  expect(h.input()).toBe('稍后完成的语音');
  expect(h.invoke.mock.calls.map(([command]) => command)).not.toContain('chat_send');
});

it('keeps an existing draft when no speech was recognized', async () => {
  const h = await open();
  await h.page().get('[data-testid="chat-input"] textarea').setValue('已有文字');
  await h.clickMic(); await h.clickMic();
  expect(h.input()).toBe('已有文字');
  expect(h.invoke.mock.calls.map(([command]) => command)).not.toContain('chat_send');
  expect(h.notify).toHaveBeenCalledWith('没有识别到语音，请重试', 'info');
});

it('supports first enabling the capability by explicitly clicking the chat microphone', async () => {
  const h = await open({ enabled: false }); await h.clickMic();
  expect(h.asr.enabled).toBe(true); expect(h.asr.listening).toBe(true);
  expect(h.invoke).toHaveBeenCalledWith('asr_set_enabled', { enabled: true });
});

it('stops on switching or creating conversations and does not restore recording when returning', async () => {
  const h = await open(); await h.clickMic();
  await h.chat.select('B'); await flushPromises(); expect(h.pressed()).toBe('false'); expect(h.stop).toHaveBeenCalledOnce();
  await h.chat.select('A'); await flushPromises(); expect(h.pressed()).toBe('false'); expect(h.capture).toHaveBeenCalledOnce();
  await h.clickMic(); await h.chat.select(null); await flushPromises();
  expect(h.pressed()).toBe('false'); expect(h.asr.enabled).toBe(true);
});

it('discards old recognition results after switching conversations', async () => {
  const pending = deferred<string>(); const h = await open({ transcribe: () => pending.promise });
  await h.clickMic(); h.phrase(); await flushPromises(); expect(h.asr.recognizing).toBe(true);
  await h.chat.select('B'); pending.resolve('旧会话的话'); await flushPromises();
  expect(h.input()).toBe(''); expect(h.pressed()).toBe('false'); expect(h.capture).toHaveBeenCalledOnce();
});

it('finishes downloading after a switch without opening the microphone until clicked again', async () => {
  const pending = deferred<unknown>(); const h = await open({ prepare: () => pending.promise });
  await h.clickMic(); expect(h.asr.recordingRequested).toBe(true); expect(h.asr.preparing).toBe(true);
  await h.chat.select('B'); await flushPromises(); expect(h.pressed()).toBe('false');
  pending.resolve({ enabled: true, modelReady: true, engineReady: true }); await flushPromises();
  expect(h.asr.engineReady).toBe(true); expect(h.capture).not.toHaveBeenCalled();
  await h.clickMic(); expect(h.capture).toHaveBeenCalledOnce(); expect(h.asr.listening).toBe(true);
});

it('stops on send and stays off after the assistant completes its response', async () => {
  const h = await open(); await h.clickMic(); h.phrase(); await flushPromises();
  await h.page().get('[data-testid="chat-send"]').trigger('click'); await flushPromises();
  expect(h.pressed()).toBe('false'); expect(h.chat.current()!.running).toBe(true);
  h.chat.current()!.running = false; await flushPromises();
  expect(h.pressed()).toBe('false'); expect(h.capture).toHaveBeenCalledOnce();
});

it('does not restart after the window was hidden, returning to the page, or remounting the chat', async () => {
  const h = await open(); await h.clickMic();
  const visibility=vi.spyOn(document,'visibilityState','get').mockReturnValue('hidden');
  document.dispatchEvent(new Event('visibilitychange')); await flushPromises();
  visibility.mockRestore();
  window.dispatchEvent(new Event('focus')); await flushPromises(); expect(h.pressed()).toBe('false'); expect(h.capture).toHaveBeenCalledOnce();
  await h.clickMic(); h.store.page = 'settings'; await flushPromises(); h.store.page = 'chat'; await flushPromises();
  expect(h.pressed()).toBe('false'); expect(h.capture).toHaveBeenCalledTimes(2);
  await h.clickMic(); await h.remount(); expect(h.pressed()).toBe('false'); expect(h.capture).toHaveBeenCalledTimes(3);
});

it('stops when speech output begins and does not restart after it ends', async () => {
  const h = await open(); await h.clickMic(); h.speech.status = 'playing'; await flushPromises();
  expect(h.pressed()).toBe('false'); h.speech.status = 'idle'; await flushPromises();
  expect(h.capture).toHaveBeenCalledOnce(); expect(h.asr.enabled).toBe(true);
});


it('keeps microphone failures in sound settings across repeated attempts and allows recovery', async () => {
  const h = await open();
  const error = '当前麦克风无法开始录音，请在系统声音设置中选择可用的输入设备，或切换麦克风后重试';
  await h.page().get('[data-testid="chat-input"] textarea').setValue('保留这段草稿');
  h.capture.mockRejectedValueOnce(new Error(error)).mockRejectedValueOnce(new Error(error));
  await h.clickMic();
  expect(h.notify).not.toHaveBeenCalled();
  expect(h.pressed()).toBe('false');
  expect(h.page().find('[data-testid="asr-status"]').exists()).toBe(false);
  expect(h.input()).toBe('保留这段草稿');
  expect(h.invoke.mock.calls.map(([command]) => command)).not.toContain('chat_send');
  expect(h.page().get('[data-testid="chat-asr-toggle"]').attributes('title')).toBe('录音：说完转成文字，留在输入框，由你确认后发送');
  expect(h.settings.get('[data-testid="microphone-status"]').text()).toContain(error);
  await h.clickMic();
  expect(h.notify).not.toHaveBeenCalled();
  expect(h.page().find('[data-testid="asr-status"]').exists()).toBe(false);
  // Reconnecting the device and explicitly retrying clears the old diagnostic.
  await h.clickMic();
  expect(h.pressed()).toBe('true');
  expect(h.asr.error).toBe('');
  expect(h.page().get('[data-testid="asr-status"]').text()).toContain('正在录音');
  expect(h.notify).not.toHaveBeenCalled();
  expect(h.settings.find('[data-testid="microphone-status"]').exists()).toBe(false);
});

it('shows unsupported microphone capability only in sound settings and disables chat recording', async () => {
  const h = await open();
  h.asr.microphoneSupported = false;
  await flushPromises();
  expect(h.settings.get('[data-testid="microphone-status"]').text()).toContain('不支持麦克风');
  expect(h.page().get('[data-testid="chat-asr-toggle"]').attributes('disabled')).toBeDefined();
  await h.asr.activate(vi.fn());
  expect(h.capture).not.toHaveBeenCalled();
  expect(h.notify).not.toHaveBeenCalled();
  expect(h.page().find('[data-testid="asr-status"]').exists()).toBe(false);
});

it('clears a stale microphone error before the next capture request completes', async () => {
  const h = await open();
  h.capture.mockRejectedValueOnce(new Error('找不到可用麦克风'));
  await h.clickMic();
  const pending = deferred<{ stop: typeof h.stop }>();
  h.capture.mockImplementationOnce(() => pending.promise);
  await h.clickMic();
  expect(h.asr.error).toBe('');
  expect(h.page().get('[data-testid="asr-status"]').text()).toContain('正在准备麦克风');
  pending.resolve({ stop: h.stop }); await flushPromises();
  expect(h.asr.listening).toBe(true);
});

