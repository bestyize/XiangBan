<script setup lang="ts">
import { computed } from 'vue'
import UserAvatar from '@/components/UserAvatar.vue'
import AppIcon from '@/components/AppIcon.vue'
import { compactCount, formatDateTime, timeAgo } from '@/lib/format'
import { commentImageId } from './images'

const props = withDefaults(defineProps<{
  author: { uid: string; nickname: string; avatar_url?: string | null }
  body?: string
  images?: string[]
  createdAt: string
  replyToName?: string
  likeCount: number
  liked: boolean
  mine?: boolean
  busy?: boolean
  canReply?: boolean
  canLike?: boolean
}>(), { canReply: true, canLike: true })

const pictures = computed(() => props.images?.filter(commentImageId) ?? [])
defineEmits<{ like: []; reply: []; remove: [] }>()
</script>

<template>
  <article class="comment">
    <RouterLink :to="`/u/${author.uid}`" class="avatar-link" :aria-label="`查看 ${author.nickname} 的主页`">
      <UserAvatar :name="author.nickname" :src="author.avatar_url" :size="32" />
    </RouterLink>
    <div class="content">
      <div class="byline">
        <RouterLink class="nickname" :to="`/u/${author.uid}`">{{ author.nickname }}</RouterLink>
        <slot name="byline" />
      </div>
      <div v-if="replyToName || $slots.parent" class="parent">
        <slot name="parent">回复 {{ replyToName }}</slot>
      </div>
      <slot>
        <p v-if="body" class="body">{{ body }}</p>
        <div v-if="pictures.length" class="pictures">
          <a v-for="src in pictures" :key="src" :href="src" target="_blank" rel="noopener noreferrer" aria-label="查看原图">
            <img :src="src" loading="lazy" alt="评论图片">
          </a>
        </div>
      </slot>
      <div class="meta">
        <time :datetime="createdAt" :title="formatDateTime(createdAt)">{{ timeAgo(createdAt) }}</time>
        <button v-if="canReply" type="button" :disabled="busy" @click="$emit('reply')">回复</button>
        <button v-if="mine" type="button" :disabled="busy" @click="$emit('remove')">删除</button>
        <slot name="actions" />
      </div>
    </div>
    <button v-if="canLike" type="button" class="like" :class="{ on: liked }" :disabled="busy" :aria-pressed="liked" :aria-label="`${liked ? '取消赞' : '赞'}这条评论，${likeCount} 个赞`" @click="$emit('like')">
      <AppIcon name="heart" :size="20" />
      <span v-if="likeCount">{{ compactCount(likeCount) }}</span>
    </button>
  </article>
</template>

<style scoped>
.comment {
  display: grid;
  grid-template-columns: 32px minmax(0, 1fr) auto;
  align-items: start;
  column-gap: 12px;
  padding-top: 12px;
  scroll-margin-block: 8px;
}
.comment::after { content: ''; grid-column: 2 / -1; border-bottom: 1px solid var(--line); }
.content { min-width: 0; }
.byline { display: flex; align-items: center; gap: 6px; }
.nickname { overflow: hidden; font-size: 14px; font-weight: 600; text-overflow: ellipsis; white-space: nowrap; }
.parent {
  display: inline-block;
  max-width: 100%;
  margin-top: 4px;
  padding: 3px 8px;
  overflow: hidden;
  border-radius: 8px;
  background: var(--surface-2);
  color: var(--ink-3);
  font-size: 12px;
  text-overflow: ellipsis;
  vertical-align: top;
  white-space: nowrap;
}
.body { margin: 4px 0 0; font-size: 15px; line-height: 1.6; white-space: pre-wrap; overflow-wrap: anywhere; }
.pictures { display: flex; flex-wrap: wrap; gap: 8px; margin-block: 8px; }
.pictures img { display: block; width: 88px; max-width: 100%; height: 88px; object-fit: cover; border-radius: 8px; }
.meta { display: flex; flex-wrap: wrap; align-items: center; min-height: 36px; gap: 0 12px; color: var(--ink-3); font-size: 12px; }
.meta :deep(button) { min-height: 36px; padding: 0; border: 0; background: none; color: inherit; font: inherit; }
.meta :deep(button:hover) { color: var(--ink); }
.like {
  display: grid;
  justify-items: center;
  align-content: center;
  gap: 1px;
  width: 40px;
  min-height: 44px;
  padding: 0;
  border: 0;
  background: none;
  color: var(--ink-3);
  font-size: 12px;
}
.like.on { color: var(--brand); }
.like.on :deep(svg) { fill: currentColor; }
button:disabled { opacity: 0.5; cursor: default; }
</style>
