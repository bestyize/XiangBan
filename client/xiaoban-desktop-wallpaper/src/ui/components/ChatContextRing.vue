<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { NPopover, NSlider } from 'naive-ui';
import { LoaderCircle, Sparkles } from 'lucide-vue-next';
import { CONTEXT_MAX, CONTEXT_MIN, CONTEXT_STEP, contextLevel, contextPercent, formatTokens } from '../chat/format.ts';

const props = defineProps<{
  sessionId?: string | null;
  used: number;
  limit: number;
  canCompact: boolean;
  /** This conversation is being compacted right now. */
  compacting: boolean;
  /** The assistant is answering, so the conversation cannot be changed under it. */
  busy: boolean;
  /** Nothing to compact: no desktop, or no conversation yet. */
  disabled: boolean;
  /** No conversation to measure yet (the new-task page): an empty ring that does not open. */
  inert?: boolean;
}>();
const emit = defineEmits<{ compact: []; resize: [tokens: number, sessionId: string | null] }>();

const RADIUS = 8;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;
/** The slider is saved once it has rested this long, not on every pixel of a drag. */
const SAVE_AFTER_MS = 350;

const clampLimit = (value: number) => Math.min(CONTEXT_MAX, Math.max(CONTEXT_MIN, value));

// What the slider shows. It leads the host's number while a change is on its way, so the bar and the ring follow the thumb.
const shown = ref(clampLimit(props.limit));
let pending: ReturnType<typeof setTimeout> | null = null;
let pendingSession: string | null = null;
let pendingLimit = shown.value;
watch(() => props.limit, value => { if (pending === null) shown.value = clampLimit(value); });

function slide(value: number) {
  shown.value = value;
  if (pending !== null) clearTimeout(pending);
  pendingSession = props.sessionId ?? null; pendingLimit = value;
  pending = setTimeout(() => { pending = null; emit('resize', pendingLimit, pendingSession); }, SAVE_AFTER_MS);
}
watch(() => props.sessionId, () => {
  if (pending !== null) { clearTimeout(pending); pending = null; emit('resize', pendingLimit, pendingSession); }
  shown.value = clampLimit(props.limit); show.value = false;
});
onBeforeUnmount(() => {
  if (pending === null) return;
  clearTimeout(pending);
  pending = null;
  emit('resize', pendingLimit, pendingSession);
});

const percent = computed(() => (props.inert ? 0 : contextPercent(props.used, shown.value)));
const level = computed(() => contextLevel(props.used, shown.value));
const show = ref(false);
watch(() => props.inert, inert => { if (inert) show.value = false; });
const dash = computed(() => `${((CIRCUMFERENCE * percent.value) / 100).toFixed(2)} ${CIRCUMFERENCE.toFixed(2)}`);
const reason = computed(() => {
  if (props.compacting) return '';
  if (props.busy) return '回答结束后才能压缩';
  if (props.disabled || !props.canCompact) return '对话还很短，暂时不需要压缩';
  return '';
});
</script>

<template>
  <span v-if="inert" class="ctx-ring is-inert" role="img" aria-label="开始对话后显示上下文用量" title="开始对话后显示上下文用量" data-testid="chat-context">
    <svg viewBox="0 0 22 22" width="22" height="22" aria-hidden="true"><circle class="ctx-track" cx="11" cy="11" r="8" fill="none" stroke-width="3" /></svg>
  </span>
  <NPopover v-else v-model:show="show" trigger="click" placement="top-end" raw :show-arrow="false" :style="{ background: 'transparent', boxShadow: 'none', borderRadius: '14px', marginBottom: '10px' }">
    <template #trigger>
      <button type="button" class="ctx-ring" :class="level" :aria-expanded="show" :aria-label="`上下文已用 ${percent}%，点击查看`" data-testid="chat-context">
        <svg viewBox="0 0 22 22" width="22" height="22" aria-hidden="true">
          <circle class="ctx-track" cx="11" cy="11" r="8" fill="none" stroke-width="3" />
          <circle class="ctx-fill" cx="11" cy="11" r="8" fill="none" stroke-width="3" stroke-linecap="round" :stroke-dasharray="dash" transform="rotate(-90 11 11)" data-testid="chat-context-fill" />
        </svg>
      </button>
    </template>
    <div class="ctx-pop" :class="level" role="dialog" aria-label="上下文用量" data-testid="chat-context-pop">
      <div class="ctx-usage" :title="used > shown ? '超出的部分会从最早的对话开始省略' : undefined">
        <div class="ctx-line"><span>上下文</span><b data-testid="ctx-numbers">{{ formatTokens(used) }} / {{ formatTokens(shown) }}</b></div>
        <div class="ctx-bar" role="progressbar" :aria-valuenow="percent" aria-valuemin="0" aria-valuemax="100" data-testid="ctx-bar"><i :style="{ width: `${percent}%` }" /></div>
      </div>
      <div class="ctx-size">
        <NSlider :value="shown" :min="CONTEXT_MIN" :max="CONTEXT_MAX" :step="CONTEXT_STEP" :format-tooltip="formatTokens" aria-label="上下文上限" data-testid="ctx-slider" @update:value="slide" />
        <div class="ctx-ends"><span>{{ formatTokens(CONTEXT_MIN) }}</span><small>用到 90% 自动压缩</small><span>{{ formatTokens(CONTEXT_MAX) }}</span></div>
      </div>
      <div :title="reason">
        <button type="button" class="ctx-compact" :disabled="disabled || busy || compacting || !canCompact" data-testid="ctx-compact" @click="emit('compact')">
          <LoaderCircle v-if="compacting" :size="14" class="ctx-spin" /><Sparkles v-else :size="14" />
          {{ compacting ? '正在压缩…' : '智能压缩' }}
        </button>
      </div>
    </div>
  </NPopover>
</template>

<style scoped>
.ctx-ring{display:inline-grid;place-items:center;width:30px;height:30px;padding:0;border:0;border-radius:50%;background:none;cursor:pointer;flex-shrink:0}
.ctx-ring:hover,.ctx-ring[aria-expanded="true"]{background:var(--ui-control)}
.ctx-ring:focus-visible{outline:2px solid var(--ui-btn);outline-offset:1px}
.ctx-ring.is-inert{cursor:default;opacity:.8}
.ctx-ring.is-inert:hover{background:none}
.ctx-track{stroke:var(--ui-control-hover)}
.ctx-fill{stroke:var(--ui-btn);transition:stroke-dasharray .3s ease}
.ctx-ring.warn .ctx-fill{stroke:var(--ui-warning)}
.ctx-ring.full .ctx-fill{stroke:var(--ui-danger)}
.ctx-pop{width:264px;box-sizing:border-box;padding:14px;border-radius:14px;background:var(--ui-surface-raised);-webkit-backdrop-filter:blur(20px);backdrop-filter:blur(20px);box-shadow:var(--ui-shadow-menu);color:var(--ui-text-primary);display:grid;gap:14px;font-family:var(--ui-font-family)}
.ctx-usage{display:grid;gap:8px}
.ctx-line{display:flex;justify-content:space-between;align-items:baseline;font-size:13px;color:var(--ui-text-secondary)}
.ctx-line b{color:var(--ui-text-primary);font-weight:600;font-variant-numeric:tabular-nums}
.ctx-bar{height:6px;border-radius:3px;background:color-mix(in srgb,var(--ui-text-primary) 8%,transparent);overflow:hidden}
.ctx-bar i{display:block;height:100%;border-radius:3px;background:var(--ui-btn);transition:width .3s}
.ctx-pop.warn .ctx-bar i{background:var(--ui-warning)}
.ctx-pop.full .ctx-bar i{background:var(--ui-danger)}
.ctx-size{display:grid;gap:4px}
.ctx-ends{display:flex;justify-content:space-between;align-items:baseline;font-size:11px;color:var(--ui-text-tertiary);font-variant-numeric:tabular-nums}
.ctx-ends small{font-size:11px}
.ctx-compact{width:100%;height:30px;border-radius:8px;border:0;background:color-mix(in srgb,var(--ui-text-primary) 6%,transparent);color:var(--ui-text-primary);font:inherit;font-size:13px;font-weight:500;display:inline-flex;align-items:center;justify-content:center;gap:6px;cursor:pointer}
.ctx-compact:hover:not(:disabled){background:color-mix(in srgb,var(--ui-text-primary) 10%,transparent)}
.ctx-compact:disabled{color:var(--ui-text-disabled);cursor:not-allowed}
.ctx-spin{animation:ctx-spin 1s linear infinite}
@keyframes ctx-spin{to{transform:rotate(360deg)}}
@media(prefers-reduced-motion:reduce){.ctx-spin{animation:none}}
</style>
