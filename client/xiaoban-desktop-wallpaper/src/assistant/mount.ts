import { createApp } from 'vue';
import { desktop, invoke, listen } from '../bridge.ts';
import { createAsrStore } from '../ui/asr/store.ts';
import { createChatStore } from '../ui/chat/chat-store.ts';
import { applyUiTokens } from '../ui/design-tokens.ts';
import { createSpeechStore } from '../ui/speech/store.ts';
import AssistantApp from './AssistantApp.vue';
import { createAssistantStore, type AssistantStore } from './assistant-store.ts';

export async function mountAssistant() {
  applyUiTokens(true);
  document.documentElement.style.background = 'transparent';
  document.body.style.background = 'transparent';
  document.body.className = 'assistant';
  const host = document.querySelector<HTMLElement>('#app');
  if (!host) throw new Error('助手的挂载节点不存在');
  // This window has no toast area: what the stores would announce is shown as the capsule's hint instead.
  const notify = (message: unknown, type: 'success' | 'error' | 'info' = 'success') => {
    if (type !== 'success') assistant.note(message instanceof Error ? message.message : String(message));
  };
  let assistant!: AssistantStore;
  const chat = createChatStore({
    desktop, invoke, listen, notify,
    onReply: (session, text, key) => assistant.replied(session, text, key),
    onActivity: () => assistant.stopped(),
  });
  const asr = createAsrStore({ desktop, invoke, listen, notify });
  const speech = createSpeechStore({ desktop, invoke, notify });
  assistant = createAssistantStore({ desktop, invoke, listen, chat, asr, speech });
  const app = createApp(AssistantApp, { store: assistant, asr });
  app.mount(host);
  window.addEventListener('beforeunload', () => { assistant.dispose(); app.unmount(); }, { once: true });
  await assistant.init();
}
