<script setup lang="ts">
import AppIcon from '@/components/AppIcon.vue'
import { compactCount } from '@/lib/format'
import { useNarrow } from '@/lib/viewport'
import DockButton from './DockButton.vue'

withDefaults(defineProps<{ count?: number | null; disabled?: boolean; label?: string }>(), { label: '写评论…' })
defineEmits<{ compose: []; discuss: [] }>()
const narrow = useNarrow()
</script>

<template>
  <Teleport to="body" :disabled="!narrow">
    <div class="comment-dock" role="group" aria-label="评论与互动">
      <button type="button" class="dock-capsule" aria-haspopup="dialog" :disabled="disabled" @click="$emit('compose')">
        <AppIcon name="message" :size="16" /><span>{{ label }}</span>
      </button>
      <slot :narrow="narrow" />
      <DockButton
        :label="count == null ? '查看评论' : `查看评论，共 ${count} 条`"
        :text="count == null ? '评论' : compactCount(count)"
        @click="$emit('discuss')"
      ><AppIcon name="message" :size="20" /></DockButton>
    </div>
  </Teleport>
</template>

<style scoped>
.comment-dock {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-block: 16px 8px;
  padding: 6px 12px;
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 14px;
}
.dock-capsule {
  display: flex;
  flex: 1;
  align-items: center;
  gap: 8px;
  min-width: 0;
  min-height: 44px;
  padding: 0 12px;
  border: 0;
  border-radius: 22px;
  background: var(--surface-2);
  color: var(--ink-3);
  text-align: left;
}
.dock-capsule svg { flex: none; }
.dock-capsule span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.dock-capsule:disabled { opacity: 0.65; }
@media (max-width: 767px) {
  .comment-dock {
    position: fixed;
    inset: auto 0 0;
    z-index: 38;
    margin: 0;
    min-height: calc(56px + env(safe-area-inset-bottom));
    padding: 5px 8px calc(6px + env(safe-area-inset-bottom)) 12px;
    border: 0;
    border-top: 1px solid var(--line);
    border-radius: 0;
  }
}
</style>
