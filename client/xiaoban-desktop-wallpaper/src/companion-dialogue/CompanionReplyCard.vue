<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue';
import { desktop, invoke } from '../bridge.ts';
import { previewReply, type CompanionReply } from './dialogue.ts';

const props = defineProps<{ initial: CompanionReply }>();
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
  <section ref="root" class="companion-reply" aria-label="伙伴回复" data-testid="companion-reply-card" @pointerenter="pointerIn" @pointerleave="pointerOut" @focusin="focusIn" @focusout="focusOut" @keydown.esc.stop="close">
    <header><span>{{ reply.petName }}</span><button type="button" class="reply-close" aria-label="关闭伙伴回复" @click="close">×</button></header>
    <button type="button" class="reply-text" :disabled="busy" title="接着聊" data-testid="companion-reply-text" @click="continueChat">{{ reply.text }}</button>
    <div class="reply-choices" aria-label="回应伙伴"><button v-for="choice in reply.choices" :key="choice.id" type="button" :disabled="busy" :data-testid="`companion-choice-${choice.id}`" @click="choose(choice.id)">{{ choice.label }}</button></div>
    <footer><button type="button" class="reply-continue" :disabled="busy" data-testid="companion-continue" @click="continueChat">接着聊 <span aria-hidden="true">↗</span></button><span v-if="thinking" class="reply-thinking" role="status">想一想…</span></footer>
    <p v-if="error" class="reply-error" role="alert">{{ error }}</p>
  </section>
</template>

<style scoped>
.companion-reply{box-sizing:border-box;width:100%;padding:12px 14px;border-radius:16px;background:var(--ui-surface,#fff);border:1px solid var(--ui-border,#eadfe2);box-shadow:0 8px 28px #38252b16;color:var(--ui-text-primary,#393337);font-family:-apple-system,BlinkMacSystemFont,'PingFang SC','Microsoft YaHei',sans-serif;font-size:12px;line-height:1.55;text-align:left;pointer-events:auto;white-space:normal}
header{display:flex;align-items:center;justify-content:space-between;gap:8px;color:var(--ui-text-secondary,#81767b);font-size:11px}
header>span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
button{font:inherit;cursor:pointer;border:0;transition:background .15s ease}
button:disabled{cursor:wait;opacity:.6}
button:focus-visible{outline:2px solid var(--ui-btn,#d64b72);outline-offset:2px}
.reply-close{flex:none;background:none;color:inherit;font-size:18px;line-height:1;padding:3px 5px;border-radius:6px}
.reply-text{display:block;background:none;color:inherit;padding:5px 0 9px;text-align:left;width:100%;line-height:1.65;overflow-wrap:anywhere;max-height:86px;overflow:auto}
.reply-text:hover{color:var(--ui-ink,#b94f70)}
.reply-choices{display:flex;flex-wrap:wrap;gap:6px}
.reply-choices button{padding:4px 8px;border-radius:8px;background:var(--ui-control,#f6f3f4);color:var(--ui-text-secondary,#74656e);font-size:11px}
.reply-choices button:hover,.reply-close:hover{background:var(--ui-soft,#fcebf1)}
footer{display:flex;justify-content:space-between;align-items:center;gap:8px;margin-top:8px;min-height:22px}
.reply-continue{padding:2px 0;background:none;color:var(--ui-ink,#b94f70);font-size:11px;font-weight:600}
.reply-thinking{color:var(--ui-text-tertiary,#9b8d95);font-size:10px}
.reply-error{margin:6px 0 0;color:var(--ui-text-secondary,#81767b);font-size:11px}
@media(prefers-reduced-motion:reduce){button{transition:none}}
</style>
