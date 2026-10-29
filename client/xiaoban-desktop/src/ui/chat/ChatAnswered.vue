<script setup lang="ts">
import { Check, SkipForward } from 'lucide-vue-next';
import type { AskRecord } from './ask-record.ts';

defineProps<{ record: AskRecord }>();
</script>

<template>
  <div class="answered" data-testid="chat-answered" :class="{ skipped: record.answer === null }">
    <span class="mark" aria-hidden="true"><SkipForward v-if="record.answer === null" :size="9" :stroke-width="2.4" /><Check v-else :size="10" :stroke-width="3" /></span>
    <span class="question" :title="record.question">{{ record.question }}</span>
    <span v-if="record.answer === null" class="answer">已跳过</span>
    <span v-else class="answer" :title="record.answer"><i aria-hidden="true">→</i>{{ record.answer }}</span>
  </div>
</template>

<style scoped>
.answered{display:flex;align-items:center;gap:8px;min-height:30px;padding:0 10px;box-sizing:border-box;border-radius:9px;background:var(--ui-control);font-size:13px;color:var(--ui-text-secondary)}
.mark{flex:none;width:16px;height:16px;display:grid;place-items:center;border-radius:50%;background:var(--ui-soft);color:var(--ui-ink)}
.skipped .mark{background:var(--ui-surface-hover);color:var(--ui-text-tertiary)}
.question{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.answer{flex:0 1 auto;min-width:0;display:inline-flex;align-items:center;gap:6px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:var(--ui-ink);font-weight:500}
.answer i{font-style:normal;font-weight:400;color:var(--ui-text-tertiary)}
.skipped .answer{color:var(--ui-text-tertiary);font-weight:400}
</style>
