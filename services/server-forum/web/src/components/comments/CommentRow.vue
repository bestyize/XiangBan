<script setup lang="ts">
import { computed } from 'vue'
import UserAvatar from '@/components/UserAvatar.vue'
import AppIcon from '@/components/AppIcon.vue'
import { timeAgo } from '@/lib/format'
import { commentImageId } from './images'

const props = defineProps<{
  author: { uid: string; nickname: string; avatar_url?: string | null }
  body: string
  images?: string[]
  createdAt: string
  replyToName?: string
  likeCount: number
  liked: boolean
  mine?: boolean
  busy?: boolean
  nested?: boolean
}>()
const pictures = computed(() => props.images?.filter(commentImageId) ?? [])
defineEmits<{ like: []; reply: []; remove: [] }>()
</script>

<template>
  <article class="comment" :class="{ nested }">
    <RouterLink :to="`/u/${author.uid}`" class="avatar-link" :aria-label="`查看 ${author.nickname} 的主页`">
      <UserAvatar :name="author.nickname" :src="author.avatar_url" :size="nested ? 26 : 34" />
    </RouterLink>
    <div class="content">
      <RouterLink class="nickname" :to="`/u/${author.uid}`">{{ author.nickname }}</RouterLink>
      <p v-if="body || replyToName" class="body"><span v-if="replyToName" class="reply-to">回复 {{ replyToName }}：</span>{{ body }}</p>
      <div v-if="pictures.length" class="pictures">
        <a v-for="src in pictures" :key="src" :href="src" target="_blank" rel="noopener noreferrer" aria-label="查看原图">
          <img :src="src" loading="lazy" alt="评论图片">
        </a>
      </div>
      <div class="meta">
        <time :datetime="createdAt">{{ timeAgo(createdAt) }}</time>
        <button type="button" :disabled="busy" @click="$emit('reply')">回复</button>
        <button v-if="mine" type="button" :disabled="busy" @click="$emit('remove')">删除</button>
      </div>
    </div>
    <button type="button" class="like" :class="{ on: liked }" :disabled="busy" :aria-pressed="liked" :aria-label="`${liked ? '取消赞' : '赞'}这条评论，${likeCount} 个赞`" @click="$emit('like')">
      <AppIcon name="heart" :size="19" />
      <span v-if="likeCount">{{ likeCount }}</span>
    </button>
  </article>
</template>

<style scoped>
.comment {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) 36px;
  align-items: start;
  gap: 10px;
  padding: 14px 0 9px;
}
.comment.nested { margin-inline-start: 44px; padding-top: 6px; }
.content { min-width: 0; }
.nickname { color: var(--ink-3); font-size: 13px; overflow-wrap: anywhere; }
.body { margin: 4px 0 0; font-size: 15px; line-height: 1.6; white-space: pre-wrap; overflow-wrap: anywhere; }
.reply-to { color: var(--ink-3); }
.pictures { display: flex; flex-wrap: wrap; gap: 8px; margin-block: 8px; }
.pictures img { display: block; width: 88px; max-width: 100%; height: 88px; object-fit: cover; border-radius: 8px; }
.meta { display: flex; flex-wrap: wrap; align-items: center; gap: 0 12px; color: var(--ink-3); font-size: 12px; }
.meta button { min-height: 30px; padding: 2px 0; border: 0; background: none; color: var(--ink-2); font-size: inherit; cursor: pointer; }
.like {
  display: grid;
  justify-items: center;
  align-content: start;
  gap: 2px;
  min-height: 36px;
  padding: 3px 0;
  border: 0;
  background: none;
  color: var(--ink-3);
  font-size: 11px;
  cursor: pointer;
}
.like.on { color: var(--brand); }
.like.on :deep(svg) { fill: currentColor; }
button:disabled { opacity: 0.5; cursor: default; }
</style>
