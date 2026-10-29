<script setup lang="ts">
/** 输入框上方的 `/` 菜单：列出用户能启动的技能，上下键选择，回车或点击插入。 */
import { SOURCE_LABEL, type SlashSkill } from '../chat/slash.ts';

defineProps<{ items: SlashSkill[]; active: number }>();
const emit = defineEmits<{ pick: [name: string]; hover: [index: number] }>();
const origin = (item: SlashSkill) => (item.source === 'plugin' && item.plugin ? item.plugin : SOURCE_LABEL[item.source] ?? '');
</script>

<template>
  <div class="slash-menu" role="listbox" aria-label="可调用的技能" data-testid="chat-slash-menu" @mousedown.prevent>
    <button
      v-for="(item, index) in items" :id="`slash-option-${index}`" :key="item.name" type="button" role="option" class="slash-item"
      :class="{ active: index === active }" :aria-selected="index === active" :data-testid="`chat-slash-${item.name}`"
      @click="emit('pick', item.name)" @mousemove="emit('hover', index)"
    >
      <span class="slash-name">/{{ item.name }}</span>
      <span v-if="item.displayName !== item.name" class="slash-title">{{ item.displayName }}</span>
      <span class="slash-desc">{{ item.description }}</span>
      <span v-if="origin(item)" class="slash-origin">{{ origin(item) }}</span>
    </button>
  </div>
</template>

<style scoped>
.slash-menu{position:absolute;left:14px;right:14px;bottom:calc(100% - 4px);z-index:20;max-height:264px;overflow-y:auto;overscroll-behavior:none;padding:6px;border-radius:14px;background:var(--ui-surface);box-shadow:0 0 0 1px var(--ui-border),0 12px 32px rgba(0,0,0,.14);scrollbar-width:thin}
html[data-theme=dark] .slash-menu{background:var(--ui-surface-alt)}
.slash-item{display:grid;grid-template-columns:auto auto minmax(0,1fr) auto;align-items:baseline;gap:10px;width:100%;padding:8px 10px;border:0;border-radius:9px;background:none;color:var(--ui-text-primary);font:inherit;font-size:13px;text-align:left;cursor:pointer}
.slash-item.active{background:var(--ui-soft)}
.slash-name{font-weight:600;color:var(--ui-ink)}
.slash-title{color:var(--ui-text-secondary)}
.slash-desc{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:var(--ui-text-tertiary);font-size:12px}
.slash-origin{padding:1px 7px;border-radius:999px;background:var(--ui-control);color:var(--ui-text-secondary);font-size:11px;white-space:nowrap}
</style>
