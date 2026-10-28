<script setup lang="ts">
import { computed, ref } from 'vue';
import { BrainCircuit, ChevronRight, LoaderCircle } from 'lucide-vue-next';

const props = defineProps<{
  text: string;
  reasoningKind: 'summary' | 'text';
  streaming: boolean;
}>();
const expanded = ref(false);
const title = computed(() => props.reasoningKind === 'summary' ? '思考摘要' : '模型提供的思考内容');
const preview = computed(() => props.text.trim().replace(/\s+/g, ' ').slice(-105));
</script>

<template>
  <section class="reasoning" data-testid="chat-reasoning" :aria-label="title">
    <button
      type="button" class="toggle" data-testid="chat-reasoning-toggle"
      :aria-expanded="expanded" @click="expanded = !expanded"
    >
      <BrainCircuit :size="14" aria-hidden="true" />
      <span class="title">{{ title }}</span>
      <span v-if="streaming" class="progress"><LoaderCircle :size="12" class="spin" />正在生成</span>
      <span v-else class="progress">已完成</span>
      <ChevronRight :size="13" class="chevron" :class="{ expanded }" aria-hidden="true" />
    </button>
    <div v-if="expanded" class="content" data-testid="chat-reasoning-content">
      {{ text }}
    </div>
    <div v-else-if="streaming && preview" class="preview" aria-live="off">{{ preview }}</div>
  </section>
</template>

<style scoped>
.reasoning{margin:0;min-width:0;max-width:100%;color:var(--ui-text-secondary);font-size:12px}
.toggle{display:flex;align-items:center;gap:7px;max-width:100%;min-height:29px;padding:4px 0;border:0;background:transparent;color:var(--ui-text-secondary);cursor:pointer;text-align:left;font:inherit}
.toggle:hover{color:var(--ui-text-primary)}
.toggle:focus-visible{outline:2px solid var(--ui-primary);outline-offset:2px;border-radius:4px}
.title{font-weight:500}.progress{display:inline-flex;align-items:center;gap:4px;font-size:11px;color:var(--ui-text-tertiary)}
.chevron{margin-left:3px;color:var(--ui-text-tertiary);transition:transform .16s ease}
.chevron.expanded{transform:rotate(90deg)}
.content{margin:5px 0 12px 7px;padding:8px 12px;max-height:260px;overflow:auto;border-left:2px solid var(--ui-line);color:var(--ui-text-secondary);white-space:pre-wrap;overflow-wrap:anywhere;line-height:1.65;font-size:12px}
.preview{max-width:600px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;margin:0 0 8px 20px;color:var(--ui-text-tertiary);font-size:11px}
.spin{animation:reasoning-spin 1s linear infinite}
@keyframes reasoning-spin{to{transform:rotate(360deg)}}
@media (prefers-reduced-motion:reduce){.spin{animation:none}.chevron{transition:none}}
</style>
