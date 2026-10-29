<script setup lang="ts">
import { ref } from 'vue';
import { BrainCircuit, ChevronRight } from 'lucide-vue-next';
import ChatReasoning from './ChatReasoning.vue';
import type { ReasoningGroup } from './reasoning-group.ts';

defineProps<{ group: ReasoningGroup }>();
const expanded = ref(false);
</script>

<template>
  <section class="reasoning-group" data-testid="chat-reasoning-group" :aria-label="`思考 ${group.items.length} 段`">
    <button type="button" class="toggle" data-testid="chat-reasoning-group-toggle" :aria-expanded="expanded" @click="expanded = !expanded">
      <BrainCircuit :size="14" aria-hidden="true" />
      <span class="title">思考 {{ group.items.length }} 段</span>
      <span class="progress">已完成</span>
      <ChevronRight :size="13" class="chevron" :class="{ expanded }" aria-hidden="true" />
    </button>
    <div v-if="expanded" class="list">
      <ChatReasoning v-for="item in group.items" :key="item.id" :text="item.text" :reasoning-kind="item.reasoningKind" :streaming="false" />
    </div>
  </section>
</template>

<style scoped>
.reasoning-group{min-width:0;max-width:100%;color:var(--ui-text-secondary);font-size:12px}
.toggle{display:flex;align-items:center;gap:7px;max-width:100%;min-height:26px;padding:3px 0;border:0;background:transparent;color:var(--ui-text-secondary);cursor:pointer;text-align:left;font:inherit}
.toggle:hover{color:var(--ui-text-primary)}
.toggle:focus-visible{outline:2px solid var(--ui-primary);outline-offset:2px;border-radius:4px}
.title{font-weight:500}.progress{font-size:11px;color:var(--ui-text-tertiary)}
.chevron{margin-left:3px;color:var(--ui-text-tertiary);transition:transform .16s ease}.chevron.expanded{transform:rotate(90deg)}
.list{margin:2px 0 6px 7px;padding-left:12px;border-left:2px solid var(--ui-line)}
@media (prefers-reduced-motion:reduce){.chevron{transition:none}}
</style>
