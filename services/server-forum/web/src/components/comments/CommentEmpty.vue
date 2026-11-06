<script setup lang="ts">
import AppIcon from '@/components/AppIcon.vue'
</script>

<template>
  <div class="comment-empty">
    <AppIcon name="chat" :size="40" />
    <strong>还没有评论</strong>
    <span>说点什么，做第一个留言的人</span>
  </div>
</template>

<style scoped>
.comment-empty {
  display: grid;
  justify-items: center;
  gap: 6px;
  padding: 36px 0 28px;
  text-align: center;
  color: var(--ink-3);
}
.comment-empty :deep(svg) { margin-bottom: 6px; opacity: 0.45; stroke-width: 1.3; }
.comment-empty strong { font-size: var(--fs-md); font-weight: 600; color: var(--ink-2); }
.comment-empty span { font-size: var(--fs-sm); }
</style>
