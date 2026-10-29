<script setup lang="ts">
import { nextTick, onMounted, ref, useTemplateRef } from 'vue';
import { ArrowUp, CircleHelp, CornerDownLeft, Pencil } from 'lucide-vue-next';
import ChatConnectorRequest from './ChatConnectorRequest.vue';
import { isImeEnter } from './format.ts';
import type { QuestionRequest } from './types.ts';

const props = defineProps<{ request: QuestionRequest }>();
const emit = defineEmits<{ answer: [value: string | null] }>();

const root = useTemplateRef<HTMLElement>('root');
const input = useTemplateRef<HTMLInputElement>('input');
const text = ref('');
const composing = ref(false);

function submit() { const value = text.value.trim(); if (value) emit('answer', value); }

/** Number keys, arrows and Esc work while the focus is on the card; the composer keeps its own keys. */
function onKey(event: KeyboardEvent) {
  if (event.metaKey || event.ctrlKey || event.altKey || isImeEnter(event, composing.value)) return;
  const typing = event.target === input.value;
  if (event.key === 'Escape') {
    if (typing && text.value) { text.value = ''; return; }
    if (!props.request.connectors) { event.preventDefault(); emit('answer', null); }
    return;
  }
  if (typing) {
    if (event.key === 'Enter') { event.preventDefault(); submit(); }
    else if (event.key === 'ArrowUp' && !text.value) { event.preventDefault(); move(-1); }
    return;
  }
  if (props.request.connectors) return;
  if (/^[1-9]$/.test(event.key)) {
    const option = props.request.options[Number(event.key) - 1];
    if (option !== undefined) { event.preventDefault(); emit('answer', option); }
  } else if (event.key === 'ArrowDown' || event.key === 'ArrowUp') { event.preventDefault(); move(event.key === 'ArrowDown' ? 1 : -1); }
}

/** Arrow keys walk the options and then the input; the focus ring is the selection. */
function move(step: number) {
  const stops = [...(root.value?.querySelectorAll<HTMLElement>('.q-option') ?? []), ...(input.value ? [input.value] : [])];
  if (!stops.length) return;
  const at = stops.indexOf(document.activeElement as HTMLElement);
  const next = at < 0 ? (step > 0 ? 0 : stops.length - 1) : Math.min(stops.length - 1, Math.max(0, at + step));
  stops[next]?.focus();
}

onMounted(async () => {
  await nextTick();
  const active = document.activeElement;
  const editing = active instanceof HTMLTextAreaElement || active instanceof HTMLInputElement ? active.value.length > 0 : false;
  if (!editing) root.value?.focus({ preventScroll: true });
});
</script>

<template>
  <section ref="root" class="question" tabindex="-1" data-testid="chat-question" role="alertdialog" aria-label="小伴想问你" @keydown="onKey" @compositionstart="composing = true" @compositionend="composing = false">
    <header class="q-head">
      <span class="q-mark" aria-hidden="true"><CircleHelp :size="13" :stroke-width="2.2" /></span>
      <b>小伴想问你</b>
      <button v-if="!request.connectors" type="button" class="q-skip" data-testid="question-skip" @click="emit('answer', null)">跳过<kbd>Esc</kbd></button>
    </header>
    <p class="q-text">{{ request.question }}</p>
    <ChatConnectorRequest v-if="request.connectors" :request="request" @answer="emit('answer', $event)" />
    <template v-else>
      <div v-if="request.options.length" class="q-options" role="group" aria-label="回答选项">
        <button v-for="(option, n) in request.options" :key="`${n}-${option}`" type="button" class="q-option" :data-testid="`question-option-${n}`" @click="emit('answer', option)">
          <span class="q-index" aria-hidden="true">{{ n + 1 }}</span>
          <span class="q-label">{{ option }}</span>
          <CornerDownLeft :size="14" class="q-enter" aria-hidden="true" />
        </button>
      </div>
      <label class="q-other">
        <Pencil :size="14" aria-hidden="true" />
        <input ref="input" v-model="text" type="text" data-testid="question-input" :placeholder="request.options.length ? '其他想法，回车发送' : '输入你的回答，回车发送'" aria-label="自己回答" />
        <button type="button" class="q-send" data-testid="question-submit" aria-label="发送回答" :disabled="!text.trim()" @click="submit"><ArrowUp :size="14" :stroke-width="2.4" /></button>
      </label>
    </template>
  </section>
</template>

<style scoped>
.question{display:flex;flex-direction:column;gap:8px;min-width:0;box-sizing:border-box;padding:10px 12px 12px;border-radius:14px;background:var(--ui-surface);box-shadow:0 0 0 1px var(--ui-border),0 6px 20px rgba(0,0,0,.06);outline:0}
.q-head{display:flex;align-items:center;gap:8px;height:24px;font-size:12px;color:var(--ui-ink)}
.q-head b{font-weight:600}
.q-mark{width:20px;height:20px;display:grid;place-items:center;border-radius:50%;background:var(--ui-soft);color:var(--ui-ink)}
.q-skip{margin-left:auto;display:inline-flex;align-items:center;gap:6px;height:24px;padding:0 6px 0 8px;border:0;border-radius:7px;background:transparent;color:var(--ui-text-secondary);font:inherit;cursor:pointer}
.q-skip:hover{background:var(--ui-surface-hover);color:var(--ui-text-primary)}
.q-skip kbd{display:inline-grid;place-items:center;min-width:16px;height:16px;padding:0 4px;box-sizing:border-box;border-radius:4px;background:var(--ui-surface);box-shadow:0 0 0 1px var(--ui-border-strong);font-size:10px;font-weight:600;line-height:1;color:var(--ui-text-secondary)}
.q-text{margin:0;padding:0 2px;font-size:14.5px;font-weight:600;line-height:1.55;color:var(--ui-text-primary);overflow-wrap:anywhere}
.q-options{display:flex;flex-direction:column;gap:4px}
.q-option{display:flex;align-items:center;gap:10px;width:100%;min-height:34px;padding:6px 10px;box-sizing:border-box;border:0;border-radius:9px;background:var(--ui-control);color:var(--ui-text-primary);font:inherit;font-size:13px;line-height:1.45;text-align:left;cursor:pointer}
.q-index{flex:none;width:20px;height:20px;display:grid;place-items:center;border-radius:6px;background:var(--ui-surface);box-shadow:0 0 0 1px var(--ui-border);font-size:11px;font-weight:600;color:var(--ui-text-secondary);font-variant-numeric:tabular-nums}
.q-label{flex:1;min-width:0;overflow-wrap:anywhere}
.q-enter{flex:none;display:none;color:var(--ui-ink)}
.q-option:hover,.q-option:focus-visible{background:var(--ui-soft);color:var(--ui-ink);outline:0}
.q-option:hover .q-index,.q-option:focus-visible .q-index{background:var(--ui-btn);color:var(--ui-text-on-btn);box-shadow:none}
.q-option:hover .q-enter,.q-option:focus-visible .q-enter{display:block}
.q-other{display:flex;align-items:center;gap:10px;height:36px;padding:0 6px 0 10px;box-sizing:border-box;border-radius:9px;background:var(--ui-surface);box-shadow:0 0 0 1px var(--ui-border-strong);color:var(--ui-text-tertiary);cursor:text}
.q-other:focus-within{box-shadow:0 0 0 1.5px var(--ui-btn)}
.q-other svg{flex:none;margin:0 3px}
.q-other input{flex:1;min-width:0;border:0;outline:0;background:transparent;color:var(--ui-text-primary);font:inherit;font-size:13px}
.q-other input::placeholder{color:var(--ui-text-tertiary)}
.q-send{flex:none;width:24px;height:24px;display:grid;place-items:center;padding:0;border:0;border-radius:50%;background:var(--ui-btn);color:var(--ui-text-on-btn);cursor:pointer}
.q-send:disabled{background:var(--ui-control);color:var(--ui-text-disabled);cursor:default}
@media (prefers-reduced-motion:no-preference){.q-option,.q-other{transition:background .12s,box-shadow .12s}}
</style>
