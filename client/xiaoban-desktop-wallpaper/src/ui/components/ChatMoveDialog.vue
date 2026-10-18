<script setup lang="ts">
import { NButton, NModal } from 'naive-ui';
import { Check, Folder, FolderPlus, Inbox } from 'lucide-vue-next';
import type { ChatGroup } from '../chat/types.ts';

defineProps<{ show: boolean; conversation: string; groups: ChatGroup[]; current: string; busy: boolean }>();
const emit = defineEmits<{ close: []; pick: [groupId: string | null]; create: [] }>();
</script>

<template>
  <NModal :show="show" preset="card" title="移动到分组" :mask-closable="!busy" style="max-width: 420px; width: 92vw;" data-testid="chat-move-dialog" @update:show="value => { if (!value && !busy) emit('close'); }">
    <p class="move-for" :title="conversation">{{ conversation }}</p>
    <div class="move-list" role="radiogroup" aria-label="目标分组">
      <button type="button" role="radio" class="move-item" :class="{ on: !current }" :aria-checked="!current" :disabled="busy" data-testid="chat-move-to-none" @click="emit('pick', null)">
        <Inbox :size="15" /><span>不放进分组</span><Check v-if="!current" :size="15" class="move-check" />
      </button>
      <button v-for="group in groups" :key="group.id" type="button" role="radio" class="move-item" :class="{ on: current === group.id }" :aria-checked="current === group.id" :disabled="busy" :data-testid="`chat-move-to-${group.id}`" @click="emit('pick', group.id)">
        <Folder :size="15" /><span>{{ group.name }}</span><Check v-if="current === group.id" :size="15" class="move-check" />
      </button>
    </div>
    <div class="move-actions">
      <NButton quaternary :disabled="busy" data-testid="chat-move-new-group" @click="emit('create')"><template #icon><FolderPlus :size="15" /></template>新建分组…</NButton>
      <NButton :disabled="busy" data-testid="chat-move-cancel" @click="emit('close')">取消</NButton>
    </div>
  </NModal>
</template>

<style scoped>
.move-for { margin: 0 0 12px; color: var(--ui-text-secondary); font-size: 13px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.move-list { display: grid; gap: 6px; max-height: 320px; overflow: auto; scrollbar-width: thin; }
.move-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border: 1px solid var(--ui-border);
  border-radius: 10px;
  background: var(--ui-surface-alt);
  color: var(--ui-text-primary);
  font: inherit;
  text-align: left;
  cursor: pointer;
}
.move-item span { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.move-item:hover:not(:disabled) { border-color: var(--ui-line); }
.move-item.on { border-color: var(--ui-primary); background: var(--ui-primary-soft); }
.move-item:disabled { cursor: not-allowed; opacity: 0.6; }
.move-item:focus-visible { outline: 2px solid var(--ui-primary); outline-offset: 2px; }
.move-check { color: var(--ui-primary); }
.move-actions { display: flex; justify-content: space-between; gap: 10px; margin-top: 16px; }
</style>
