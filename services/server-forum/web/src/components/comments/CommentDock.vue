<script setup lang="ts">
import AppIcon from '@/components/AppIcon.vue'
import DockButton from './DockButton.vue'

withDefaults(defineProps<{ count?: number | null; disabled?: boolean; label?: string; hidden?: boolean }>(), { label: '说点什么…' })
defineEmits<{ compose: []; jump: [] }>()
</script>

<template>
  <!-- 手机详情页统一的底部互动栏：左边评论入口，右边是页面自己的按钮（slot）和评论数。 -->
  <Teleport to="body">
    <div v-show="!hidden" class="comment-dock" role="group" aria-label="评论与互动">
      <button type="button" class="dock-capsule" :disabled="disabled" @click="$emit('compose')">{{ label }}</button>
      <slot />
      <DockButton :label="`查看评论，共 ${count ?? 0} 条`" :text="count || '评论'" @click="$emit('jump')"><AppIcon name="chat" :size="24" /></DockButton>
    </div>
  </Teleport>
</template>

<style scoped>
.comment-dock {
  position: fixed;
  z-index: 38;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  align-items: center;
  gap: 2px;
  box-sizing: border-box;
  height: calc(56px + env(safe-area-inset-bottom));
  padding: 0 8px 0 12px;
  padding-bottom: env(safe-area-inset-bottom);
  background: var(--surface);
  border-top: 1px solid var(--line);
}
.dock-capsule {
  flex: 1;
  min-width: 0;
  height: 38px;
  margin-right: 6px;
  padding: 0 14px;
  border: 0;
  border-radius: 19px;
  background: var(--surface-2);
  color: var(--ink-3);
  font-size: 14px;
  text-align: left;
}
.dock-capsule:disabled { opacity: 0.6; }
</style>
