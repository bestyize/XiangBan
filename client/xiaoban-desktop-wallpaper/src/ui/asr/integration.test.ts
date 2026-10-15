// @vitest-environment happy-dom
import { flushPromises, mount } from '@vue/test-utils';
import { reactive } from 'vue';
import { expect, it, vi } from 'vitest';
import ChatPage from '../pages/ChatPage.vue';
import VoiceSettingsCard from '../components/VoiceSettingsCard.vue';
import { createChatStore, type ChatDeps } from '../chat/chat-store.ts';
import { createSpeechStore, speechDefaults } from '../speech/store.ts';
import { createAsrStore, type AsrDeps } from './store.ts';
import type { ControlStore } from '../control-types.ts';

it('shares the voice switch and inserts recognized text for confirmation without sending it', async () => {
  let enabled = false;
  let onFrame: ((frame: Float32Array) => void) | undefined;
  const stop = vi.fn(), notify = vi.fn();
  const invoke = vi.fn(async (command: string, args?: Record<string, unknown>) => {
    if (command === 'chat_overview') return { configured: true, sessions: [], running: [], workspace: '', permissionMode: 'ask', hideToolCalls: false };
    if (command === 'tts_get_settings') return { ...speechDefaults };
    if (command === 'asr_get_status') return { enabled, modelReady: false, engineReady: false };
    if (command === 'asr_set_enabled') { enabled = args!.enabled as boolean; return { enabled, modelReady: true, engineReady: false }; }
    if (command === 'asr_prepare') return { enabled, modelReady: true, engineReady: true };
    if (command === 'asr_transcribe') return '帮我查一下天气。';
    throw new Error(command);
  });
  const listen: AsrDeps['listen'] = async () => () => {};
  const asr = createAsrStore({ desktop: true, invoke: invoke as AsrDeps['invoke'], listen, notify,
    capture: async callback => { onFrame = callback; return { stop }; } });
  const chat = createChatStore({ desktop: true, invoke: invoke as ChatDeps['invoke'], listen, notify });
  const speech = createSpeechStore({ desktop: true, invoke: invoke as AsrDeps['invoke'], notify });
  const store = reactive({ desktop: true, chat, speech, asr, settingsTab: 'voice', page: 'chat', notify, navigate: vi.fn() }) as unknown as ControlStore;
  const settings = mount(VoiceSettingsCard, { props: { store } });
  const page = mount(ChatPage, { props: { store } });
  try {
    await flushPromises();
    expect(page.get('[data-testid="chat-asr-toggle"]').attributes('aria-pressed')).toBe('false');
    await settings.get('[data-testid="asr-settings-switch"]').trigger('click'); await flushPromises();
    expect(page.get('[data-testid="chat-asr-toggle"]').attributes('aria-pressed')).toBe('true');
    expect(asr.listening).toBe(true);
    for (let i = 0; i < 3; i++) onFrame!(new Float32Array(1600).fill(0.1));
    for (let i = 0; i < 9; i++) onFrame!(new Float32Array(1600));
    await flushPromises();
    expect((page.get('[data-testid="chat-input"] textarea').element as HTMLTextAreaElement).value).toBe('帮我查一下天气。');
    expect(invoke.mock.calls.map(([command]) => command)).not.toContain('chat_send');
    await page.get('[data-testid="chat-asr-toggle"]').trigger('click'); await flushPromises();
    expect(settings.get('[data-testid="asr-settings-switch"]').attributes('aria-checked')).toBe('false');
    expect(stop).toHaveBeenCalled();
  } finally { page.unmount(); settings.unmount(); asr.dispose(); chat.dispose(); speech.dispose(); }
});
