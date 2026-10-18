<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { NButton, NPopover, NProgress, NSlider } from 'naive-ui';
import { Sparkles } from 'lucide-vue-next';
import { CONTEXT_MAX, CONTEXT_MIN, CONTEXT_STEP, contextLevel, contextPercent, formatTokens } from '../chat/format.ts';

const props = defineProps<{
  used: number;
  limit: number;
  canCompact: boolean;
  /** This conversation is being compacted right now. */
  compacting: boolean;
  /** The assistant is answering, so the conversation cannot be changed under it. */
  busy: boolean;
  /** Nothing to compact: no desktop, or no conversation yet. */
  disabled: boolean;
}>();
const emit = defineEmits<{ compact: []; resize: [tokens: number] }>();

const RADIUS = 8;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;
/** The slider is saved once it has rested this long, not on every pixel of a drag. */
const SAVE_AFTER_MS = 350;

const clampLimit = (value: number) => Math.min(CONTEXT_MAX, Math.max(CONTEXT_MIN, value));

// What the slider shows. It leads the host's number while a change is on its way, so the bar and the ring follow the thumb.
const shown = ref(clampLimit(props.limit));
let pending: ReturnType<typeof setTimeout> | null = null;
watch(() => props.limit, value => { if (pending === null) shown.value = clampLimit(value); });

function slide(value: number) {
  shown.value = value;
  if (pending !== null) clearTimeout(pending);
  pending = setTimeout(() => { pending = null; emit('resize', shown.value); }, SAVE_AFTER_MS);
}
onBeforeUnmount(() => {
  if (pending === null) return;
  clearTimeout(pending);
  pending = null;
  emit('resize', shown.value);
});

const percent = computed(() => contextPercent(props.used, shown.value));
const level = computed(() => contextLevel(props.used, shown.value));
const status = computed(() => (level.value === 'full' ? 'error' : level.value === 'warn' ? 'warning' : 'default'));
const dash = computed(() => `${((CIRCUMFERENCE * percent.value) / 100).toFixed(2)} ${CIRCUMFERENCE.toFixed(2)}`);
const reason = computed(() => {
  if (props.compacting) return '';
  if (props.busy) return '回答结束后才能压缩';
  if (props.disabled || !props.canCompact) return '对话还很短，暂时不需要压缩';
  return '';
});
</script>

<template>
  <NPopover trigger="click" placement="top-end" :show-arrow="false" style="width: 264px;">
    <template #trigger>
      <button type="button" class="ctx-ring" :class="level" :aria-label="`上下文已用 ${percent}%，点击查看`" data-testid="chat-context">
        <svg viewBox="0 0 22 22" width="22" height="22" aria-hidden="true">
          <circle class="ctx-track" cx="11" cy="11" r="8" fill="none" stroke-width="3" />
          <circle class="ctx-fill" cx="11" cy="11" r="8" fill="none" stroke-width="3" stroke-linecap="round" :stroke-dasharray="dash" transform="rotate(-90 11 11)" data-testid="chat-context-fill" />
        </svg>
      </button>
    </template>
    <div class="ctx-pop" data-testid="chat-context-pop">
      <div class="ctx-usage" :title="used > shown ? '超出的部分会从最早的对话开始省略' : undefined">
        <div class="ctx-line"><span>上下文</span><b data-testid="ctx-numbers">{{ formatTokens(used) }} / {{ formatTokens(shown) }}</b></div>
        <NProgress type="line" :percentage="percent" :status="status" :show-indicator="false" :height="6" :border-radius="3" data-testid="ctx-bar" />
      </div>
      <div class="ctx-size">
        <NSlider :value="shown" :min="CONTEXT_MIN" :max="CONTEXT_MAX" :step="CONTEXT_STEP" :format-tooltip="formatTokens" aria-label="上下文上限" data-testid="ctx-slider" @update:value="slide" />
        <div class="ctx-ends"><span>{{ formatTokens(CONTEXT_MIN) }}</span><small>用到 90% 自动压缩</small><span>{{ formatTokens(CONTEXT_MAX) }}</span></div>
      </div>
      <div :title="reason">
        <NButton block secondary size="small" :loading="compacting" :disabled="disabled || busy || compacting || !canCompact" data-testid="ctx-compact" @click="emit('compact')">
          <template #icon><Sparkles :size="14" /></template>
          {{ compacting ? '正在压缩…' : '智能压缩' }}
        </NButton>
      </div>
    </div>
  </NPopover>
</template>

<style scoped>
.ctx-ring { display: inline-flex; align-items: center; justify-content: center; width: 30px; height: 30px; padding: 0; border: 0; border-radius: 50%; background: none; cursor: pointer; flex-shrink: 0; }
.ctx-ring:hover { background: var(--ui-surface-alt); }
.ctx-ring:focus-visible { outline: 2px solid var(--ui-primary); outline-offset: 1px; }
.ctx-track { stroke: var(--ui-border); }
.ctx-fill { stroke: var(--ui-primary); transition: stroke-dasharray 0.3s ease; }
.ctx-ring.warn .ctx-fill { stroke: #f0a020; }
.ctx-ring.full .ctx-fill { stroke: var(--ui-danger, #d03050); }
.ctx-pop { display: grid; gap: 14px; padding: 2px 0; }
.ctx-usage { display: grid; gap: 8px; }
.ctx-line { display: flex; justify-content: space-between; align-items: baseline; font-size: 13px; color: var(--ui-text-secondary); }
.ctx-line b { color: var(--ui-text-primary); font-variant-numeric: tabular-nums; }
.ctx-size { display: grid; gap: 4px; }
.ctx-ends { display: flex; justify-content: space-between; align-items: baseline; font-size: 11px; color: var(--ui-text-tertiary); font-variant-numeric: tabular-nums; }
</style>
