<script setup lang="ts">
import type { MentionItem } from '../chat/mentions.ts';
defineProps<{ items: MentionItem[]; active: number; loading: boolean }>();
const emit = defineEmits<{ pick: [item: MentionItem]; hover: [index: number] }>();
const labels = { connector: '连接器', expert: '专家', skill: '技能' };
</script>
<template>
  <div class="mention-menu" role="listbox" aria-label="引用专家、技能或连接器" data-testid="chat-mention-menu" @mousedown.prevent>
    <span v-if="!items.length" role="status">{{ loading ? '正在读取…' : '没有匹配的已安装能力' }}</span>
    <button v-for="(item, index) in items" :key="`${item.kind}:${item.id}`" type="button" role="option" :class="{ active: index === active }" :aria-selected="index === active" :title="item.description || undefined" :aria-description="item.description || undefined" @click="emit('pick', item)" @mousemove="emit('hover', index)">
      <img v-if="item.iconUrl?.startsWith('data:image/')" :src="item.iconUrl" alt="" /><b>@{{ item.name }}</b><small>{{ labels[item.kind] }}</small>
    </button>
  </div>
</template>
<style scoped>
.mention-menu{position:absolute;left:14px;width:260px;box-sizing:border-box;bottom:calc(100% - 4px);z-index:20;max-height:264px;overflow-y:auto;padding:6px;border-radius:14px;background:var(--ui-surface);box-shadow:0 0 0 1px var(--ui-border),0 12px 32px rgba(0,0,0,.14);scrollbar-width:thin}.mention-menu>span{display:block;padding:8px;color:var(--ui-text-secondary);font-size:13px}button{display:flex;align-items:center;gap:10px;width:100%;padding:8px 10px;border:0;border-radius:9px;background:none;color:var(--ui-text-primary);font:inherit;font-size:13px;text-align:left;cursor:pointer}button.active{background:var(--ui-soft)}b{flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;color:var(--ui-ink);white-space:nowrap}small{padding:1px 7px;border-radius:999px;background:var(--ui-control);color:var(--ui-text-secondary);white-space:nowrap}img{width:20px;height:20px;object-fit:contain}
</style>
