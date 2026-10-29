import { createApp, watchEffect } from 'vue';
import { desktop, emit, invoke, listen } from '../bridge.ts';
import { createVoiceOwner } from '../voice/owner.ts';
import { createDictation, type Dictation } from '../voice/dictation.ts';
import { createAsrStore } from '../ui/asr/store.ts';
import { loadTurnModels } from '../voice/turn/models.ts';
import { createChatStore } from '../ui/chat/chat-store.ts';
import { followAccent, followHost, isDark } from '../ui/appearance.ts';
import { applyUiTokens } from '../ui/design-tokens.ts';
import { createSpeechStore } from '../ui/speech/store.ts';
import AssistantApp from './AssistantApp.vue';
import { createAssistantStore, type AssistantStore } from './assistant-store.ts';

export async function mountAssistant() {
  watchEffect(() => applyUiTokens(isDark.value));
  document.documentElement.style.background = 'transparent';
  document.body.style.background = 'transparent';
  document.body.className = 'assistant';
  const host = document.querySelector<HTMLElement>('#app');
  if (!host) throw new Error('超级小伴的挂载节点不存在');
  // Chat and playback notices use the reply card; recording notices are temporary.
  const notify = (message: unknown, type: 'success' | 'error' | 'info' = 'success') => {
    if (type !== 'success') assistant.note(message instanceof Error ? message.message : String(message));
  };
  let assistant!: AssistantStore;
  let dictation!: Dictation;
  // The console's conversation has its own microphone and voice; whichever window uses them last has them.
  const voice = createVoiceOwner({ me: 'assistant', emit, listen, onLost: () => { dictation.micLost(); assistant.yieldVoice(); } });
  const chat = createChatStore({
    desktop, invoke, listen, notify,
    onReply: (session, text, key) => assistant.replied(session, text, key),
    onActivity: () => assistant.stopped(),
  });
  const asr = createAsrStore({ desktop, invoke, listen, claimVoice: voice.claim, turnModels: () => loadTurnModels(invoke), notify: (message, type = 'success') => {
    if (type !== 'success') assistant.noteVoice(message instanceof Error ? message.message : String(message));
  } });
  const speech = createSpeechStore({ desktop, invoke, notify, claimVoice: voice.claim });
  assistant = createAssistantStore({ desktop, invoke, listen, chat, asr, speech, onAppearance: followHost, onAccent: followAccent });
  dictation = createDictation({
    asr,
    inject: request => invoke('speech_agent_inject', { request }),
    notify: message => assistant.note(message),
  });
  try { dictation.target = localStorage.getItem('xiaoban.dictation.target') || 'Claude'; } catch { /* default target */ }
  const app = createApp(AssistantApp, { store: assistant, asr, dictation });
  app.mount(host);
  window.addEventListener('beforeunload', () => { void dictation.stop(); voice.dispose(); assistant.dispose(); app.unmount(); }, { once: true });
  await assistant.init();
}

