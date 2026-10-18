<script setup lang="ts">
import { computed } from 'vue';
import { NButton, NPopover, NProgress } from 'naive-ui';
import { Sparkles } from 'lucide-vue-next';
import { contextLevel, contextPercent, formatTokens } from '../chat/format.ts';

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
const emit = defineEmits<{ compact: [] }>();

const RADIUS = 8;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

const percent = computed(() => contextPercent(props.used, props.limit));
const level = computed(() => contextLevel(props.used, props.limit));
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
      <div class="ctx-usage" :title="used > limit ? '超出的部分会从最早的对话开始省略' : undefined">
        <div class="ctx-line"><span>上下文</span><b data-testid="ctx-numbers">{{ formatTokens(used) }} / {{ formatTokens(limit) }}</b></div>
        <NProgress type="line" :percentage="percent" :status="status" :show-indicator="false" :height="6" :border-radius="3" data-testid="ctx-bar" />
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
.ctx-pop { display: grid; gap: 12px; padding: 2px 0; }
.ctx-usage { display: grid; gap: 8px; }
.ctx-line { display: flex; justify-content: space-between; align-items: baseline; font-size: 13px; color: var(--ui-text-secondary); }
.ctx-line b { color: var(--ui-text-primary); font-variant-numeric: tabular-nums; }
</style>
