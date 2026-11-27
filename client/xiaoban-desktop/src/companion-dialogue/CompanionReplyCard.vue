<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue';
import { desktop, invoke } from '../bridge.ts';
import { previewReply, type CompanionReply } from './dialogue.ts';

const props = defineProps<{
  initial: CompanionReply;
  tail?: 'left' | 'right' | 'bottom-right' | 'none';
}>();
const emit = defineEmits<{ close: [] }>();
const reply = ref(props.initial), thinking = ref(false), busy = ref(false), error = ref('');
const root = ref<HTMLElement | null>(null);
let version = 0, timer: ReturnType<typeof setTimeout> | undefined, hovered = false, focused = false;
function arm() {
  clearTimeout(timer);
  if (!hovered && !focused && !busy.value) timer = setTimeout(close, 12_000);
}
function close() {
  ++version; clearTimeout(timer);
  if (desktop) void invoke('companion_dialogue_close', { id: reply.value.id }).catch(() => {});
  emit('close');
}
function focusOut() { queueMicrotask(() => { focused = !!root.value?.contains(document.activeElement); arm(); }); }
function pointerIn() { hovered = true; clearTimeout(timer); }
function pointerOut() { hovered = false; arm(); }
function focusIn() { focused = true; clearTimeout(timer); }
async function show(value: CompanionReply) {
  const ticket = ++version; reply.value = value; error.value = ''; arm();
  if (!desktop || value.model) { thinking.value = false; return; }
  thinking.value = true;
  try {
    const generated = await invoke<CompanionReply | null>('companion_dialogue_model', { id: value.id });
    if (ticket === version && generated?.id === reply.value.id) { reply.value = generated; arm(); }
  } catch { /* The instant local reply remains available for unavailable or slow providers. */ }
  finally { if (ticket === version) thinking.value = false; }
}
async function choose(choice: string) {
  if (busy.value) return;
  busy.value = true; thinking.value = false; ++version; clearTimeout(timer);
  try {
    const value = desktop ? await invoke<CompanionReply>('companion_dialogue_choose', { id: reply.value.id, choice }) : previewReply(choice);
    busy.value = false;
    if (value) void show(value);
  } catch { error.value = '这一句没接上，再点一下试试吧。'; }
  finally { busy.value = false; arm(); }
}
async function continueChat() {
  if (busy.value) return;
  if (!desktop) { error.value = '在桌面应用里可以继续这段会话。'; return; }
  busy.value = true; thinking.value = false; ++version; clearTimeout(timer);
  try { await invoke('companion_dialogue_continue', { id: reply.value.id }); emit('close'); }
  catch { error.value = '暂时没能打开会话，请再试一次。'; }
  finally { busy.value = false; arm(); }
}
watch(() => props.initial, value => { void show(value); }, { immediate: true });
onBeforeUnmount(() => { ++version; clearTimeout(timer); if (desktop) void invoke('companion_dialogue_close', { id: reply.value.id }).catch(() => {}); });
</script>

<template>
  <section
    ref="root"
    class="companion-reply"
    :data-tail="tail && tail !== 'none' ? tail : undefined"
    aria-label="伙伴回复"
    data-testid="companion-reply-card"
    @pointerenter="pointerIn"
    @pointerleave="pointerOut"
    @focusin="focusIn"
    @focusout="focusOut"
    @keydown.esc.stop="close"
  >
    <header class="reply-head">
      <span class="reply-dot" aria-hidden="true"></span>
      <span class="reply-name">{{ reply.petName }}</span>
      <span v-if="reply.model" class="reply-badge" title="已由当前对话模型自然润色">已润色</span>
      <button type="button" class="reply-close" title="关闭伙伴回复" aria-label="关闭伙伴回复" @click="close">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.1" stroke-linecap="round" aria-hidden="true">
          <path d="M18 6 6 18M6 6l12 12" />
        </svg>
      </button>
    </header>
    <button type="button" class="reply-text" :disabled="busy" title="接着聊" data-testid="companion-reply-text" @click="continueChat">{{ reply.text }}</button>
    <div class="reply-choices" aria-label="回应伙伴">
      <button
        v-for="choice in reply.choices"
        :key="choice.id"
        type="button"
        :disabled="busy"
        :data-testid="`companion-choice-${choice.id}`"
        @click="choose(choice.id)"
      >{{ choice.label }}</button>
    </div>
    <footer class="reply-foot">
      <button type="button" class="reply-continue" :disabled="busy" data-testid="companion-continue" @click="continueChat">
        接着聊
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M7 17 17 7M9 7h8v8" />
        </svg>
      </button>
      <span v-if="thinking" class="reply-thinking" role="status">
        <span class="reply-dots" aria-hidden="true"><i></i><i></i><i></i></span>
        想一想…
      </span>
    </footer>
    <p v-if="error" class="reply-error" role="alert">{{ error }}</p>
  </section>
</template>

<style scoped>
.companion-reply {
  position: relative;
  box-sizing: border-box;
  width: 100%;
  padding: 14px 15px 13px;
  border-radius: 22px;
  background: #FFFDFB;
  box-shadow:
    0 0 0 1px rgba(140, 108, 86, .13),
    0 14px 30px rgba(72, 48, 34, .12),
    0 3px 8px rgba(72, 48, 34, .05);
  color: var(--ui-text-primary, #1D1D1F);
  font-family: var(--ui-font-family, -apple-system, 'SF Pro Text', 'PingFang SC', 'Microsoft YaHei UI', sans-serif);
  font-size: 13px;
  line-height: 1.56;
  text-align: left;
  pointer-events: auto;
  white-space: normal;
  display: flex;
  flex-direction: column;
  gap: 9px;
}
.companion-reply[data-tail]::after {
  content: "";
  position: absolute;
  width: 10px;
  height: 10px;
  background: inherit;
  transform: rotate(45deg);
  pointer-events: none;
}
.companion-reply[data-tail="left"]::after {
  left: -5px;
  top: 28px;
  border-radius: 0 0 0 2.5px;
  box-shadow: -1px 1px 0 rgba(140, 108, 86, .13);
}
.companion-reply[data-tail="right"]::after {
  right: -5px;
  top: 28px;
  border-radius: 0 2.5px 0 0;
  box-shadow: 1px -1px 0 rgba(140, 108, 86, .13);
}
.companion-reply[data-tail="bottom-right"]::after {
  right: 26px;
  bottom: -5px;
  border-radius: 0 0 2.5px 0;
  box-shadow: 1px 1px 0 rgba(140, 108, 86, .13);
}

.reply-head {
  display: flex;
  align-items: center;
  gap: 6px;
  min-height: 20px;
}
.reply-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex: none;
  background: var(--ui-dot, #E5484D);
  box-shadow: 0 0 0 3px var(--ui-soft, #FCEBEC);
}
.reply-name {
  font-size: 11.5px;
  font-weight: 600;
  color: var(--ui-text-secondary, #68686E);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.reply-badge {
  flex: none;
  font-size: 10.5px;
  font-weight: 500;
  padding: 1px 7px;
  border-radius: 999px;
  background: var(--ui-soft, #FCEBEC);
  color: var(--ui-ink, #B42A33);
  white-space: nowrap;
}
button {
  font: inherit;
  cursor: pointer;
  border: 0;
  transition: background .15s ease, color .15s ease, transform .12s ease;
}
button:disabled { cursor: wait; opacity: .6; }
button:focus-visible { outline: 2px solid var(--ui-btn, #D93A44); outline-offset: 2px; }

.reply-close {
  margin-left: auto;
  margin-right: -4px;
  margin-top: -2px;
  width: 24px;
  height: 24px;
  border-radius: 999px;
  flex: none;
  display: grid;
  place-items: center;
  background: transparent;
  color: var(--ui-text-tertiary, #8E8E95);
}
.reply-close:hover {
  background: rgba(164, 132, 110, .12);
  color: var(--ui-text-primary, #1D1D1F);
}

.reply-text {
  display: block;
  width: 100%;
  padding: 0;
  background: none;
  color: var(--ui-text-primary, #1D1D1F);
  font-size: 13px;
  line-height: 1.58;
  text-align: left;
  overflow-wrap: anywhere;
  max-height: 88px;
  overflow-y: auto;
  border-radius: 6px;
}
.reply-text:hover { color: var(--ui-ink, #B42A33); }

.reply-choices {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.reply-choices button {
  height: 26px;
  padding: 0 11px;
  border-radius: 999px;
  background: rgba(164, 132, 110, .10);
  color: var(--ui-text-secondary, #68686E);
  font-size: 11.5px;
  font-weight: 500;
  display: inline-flex;
  align-items: center;
}
.reply-choices button:hover {
  background: var(--ui-soft, #FCEBEC);
  color: var(--ui-ink, #B42A33);
}
.reply-choices button:active { transform: scale(.97); }

.reply-foot {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
  min-height: 24px;
  padding-top: 2px;
}
.reply-continue {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  height: 24px;
  padding: 0 8px 0 6px;
  margin-left: -6px;
  border-radius: 999px;
  background: transparent;
  color: var(--ui-ink, #B42A33);
  font-size: 11.5px;
  font-weight: 600;
}
.reply-continue:hover { background: var(--ui-soft, #FCEBEC); }

.reply-thinking {
  margin-left: auto;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  color: var(--ui-text-tertiary, #8E8E95);
  font-size: 11px;
}
.reply-dots {
  display: inline-flex;
  gap: 2.5px;
  align-items: center;
}
.reply-dots i {
  width: 3.5px;
  height: 3.5px;
  border-radius: 50%;
  background: var(--ui-dot, #E5484D);
  animation: reply-pulse 1.2s infinite ease-in-out;
}
.reply-dots i:nth-child(2) { animation-delay: .18s; }
.reply-dots i:nth-child(3) { animation-delay: .36s; }
@keyframes reply-pulse {
  0%, 80%, 100% { opacity: .25; transform: scale(.85); }
  40% { opacity: 1; transform: scale(1.1); }
}

.reply-error {
  margin: 0;
  padding: 6px 10px;
  border-radius: 10px;
  background: rgba(217, 58, 68, .08);
  color: var(--ui-danger, #D93A44);
  font-size: 11px;
  line-height: 1.4;
}

:global(html[data-theme="dark"]) .companion-reply {
  background: #292624;
  box-shadow:
    inset 0 1px 0 rgba(255, 236, 220, .08),
    0 0 0 1px rgba(255, 224, 198, .11),
    0 18px 40px rgba(0, 0, 0, .54);
}
:global(html[data-theme="dark"]) .companion-reply[data-tail="left"]::after {
  box-shadow: -1px 1px 0 rgba(255, 224, 198, .11);
}
:global(html[data-theme="dark"]) .companion-reply[data-tail="right"]::after {
  box-shadow: 1px -1px 0 rgba(255, 224, 198, .11);
}
:global(html[data-theme="dark"]) .companion-reply[data-tail="bottom-right"]::after {
  box-shadow: 1px 1px 0 rgba(255, 224, 198, .11);
}
:global(html[data-theme="dark"]) .reply-choices button {
  background: rgba(255, 224, 198, .08);
}
:global(html[data-theme="dark"]) .reply-choices button:hover {
  background: var(--ui-soft, #3A2225);
  color: var(--ui-ink, #FF8C91);
}
:global(html[data-theme="dark"]) .reply-close:hover {
  background: rgba(255, 224, 198, .14);
}
:global(html[data-theme="dark"]) .reply-error {
  background: rgba(229, 72, 77, .14);
}

@media (prefers-reduced-motion: reduce) {
  button,
  .reply-dots i { animation: none; transition: none; }
}
</style>
