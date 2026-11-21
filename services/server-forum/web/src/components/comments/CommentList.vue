<script setup lang="ts">
import { ref, useId } from 'vue'
import AppIcon from '@/components/AppIcon.vue'
import UserAvatar from '@/components/UserAvatar.vue'
import { useSession } from '@/stores/session'

defineProps<{ count?: number | null; detail?: boolean; prompt?: boolean }>()
defineEmits<{ compose: [] }>()
const session = useSession()
const list = ref<HTMLElement | null>(null)
const titleId = useId()

function jump(smooth = false) {
  list.value?.focus({ preventScroll: true })
  const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches
  list.value?.scrollIntoView({ block: 'start', behavior: smooth && !reduceMotion ? 'smooth' : 'auto' })
}

defineExpose({ jump })
</script>

<template>
  <section ref="list" :class="detail ? 'xhs-comments' : 'comment-list'" tabindex="-1" :aria-labelledby="titleId">
    <div v-if="detail" class="xhs-comments-head">
      <h2 :id="titleId">共 {{ count ?? 0 }} 条评论</h2>
      <slot name="tools" />
    </div>
    <h2 v-else :id="titleId">评论 <span v-if="count != null">{{ count }}</span></h2>
    <div v-if="prompt" class="xhs-inline-prompt">
      <UserAvatar :name="session.user?.nickname || '伴'" :src="session.user?.avatar_url" :size="30" :initial-size="12"
        :class="{ 'guest-avatar': !session.user }" />
      <button class="xhs-inline-pill" type="button" aria-haspopup="dialog" @click="$emit('compose')">
        <span>留下你的想法吧</span>
        <span class="xhs-inline-icons"><AppIcon name="landscape" :size="15" /><AppIcon name="smile-round" :size="15" /></span>
      </button>
    </div>
    <slot />
  </section>
</template>

<style scoped>
.comment-list { margin-top: 16px; padding: 12px 0 16px; border-top: 1px solid var(--line); scroll-margin-top: calc(var(--header-h) + 12px); }
.comment-list h2 { margin: 0; font-size: 16px; font-weight: 650; }
.comment-list h2 span { margin-inline-start: 4px; color: var(--ink-3); font-size: 14px; font-weight: 400; }
.xhs-comments { scroll-margin-top: calc(var(--header-h) + 12px); }
.xhs-comments-head h2 { margin: 0; font: inherit; }
.xhs-inline-prompt :deep(.avatar) { background: var(--brand-fill); color: var(--brand-ink); border: 0; }
.xhs-inline-prompt :deep(.guest-avatar) { background: #d93a44; color: #fff; }
</style>
