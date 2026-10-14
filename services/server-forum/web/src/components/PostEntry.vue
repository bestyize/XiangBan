<script setup lang="ts">
import { ref } from 'vue'
import type { PostItem } from '@/api/types'
import UserAvatar from './UserAvatar.vue'
import { api } from '@/api/forum'
import { useSession } from '@/stores/session'
import { useToast } from '@/stores/toast'
import { TRUST_LABEL, formatDateTime, timeAgo } from '@/lib/format'

const props = defineProps<{ post: PostItem; topicAuthor: string; canReply: boolean; accepted?: boolean }>()
const emit = defineEmits<{ reply: [post: PostItem]; report: [post: PostItem]; removed: [] }>()
const session = useSession()
const toast = useToast()

const editing = ref(false)
const draft = ref('')
const busy = ref(false)

async function like() {
  if (!session.requireLogin()) return
  try {
    const res = await api.likePost(props.post.id, !props.post.liked)
    props.post.liked = res.liked
    props.post.like_count = res.like_count
  } catch (e) {
    toast.error(e)
  }
}

async function startEdit() {
  try {
    draft.value = (await api.postSource(props.post.id)).body
    editing.value = true
  } catch (e) {
    toast.error(e)
  }
}

async function saveEdit() {
  busy.value = true
  try {
    const res = await api.updatePost(props.post.id, draft.value)
    props.post.body_html = res.body_html
    props.post.edited_at = new Date().toISOString()
    editing.value = false
  } catch (e) {
    toast.error(e)
  } finally {
    busy.value = false
  }
}

async function remove() {
  if (!confirm('删除这条回复？删除后不能恢复。')) return
  try {
    await api.deletePost(props.post.id)
    props.post.status = 'deleted'
    props.post.body_html = ''
    emit('removed')
  } catch (e) {
    toast.error(e)
  }
}
</script>

<template>
  <article :id="`floor-${post.floor}`" class="post">
    <RouterLink :to="`/u/${post.author.uid}`" class="avatar" :aria-label="post.author.nickname">
      <UserAvatar :name="post.author.nickname" :src="post.author.avatar_url" :size="34" />
    </RouterLink>
    <div class="main">
      <header class="head">
        <RouterLink :to="`/u/${post.author.uid}`" class="name">{{ post.author.nickname }}</RouterLink>
        <span v-if="post.author.uid === topicAuthor" class="op">楼主</span>
        <span v-if="post.author.trust_level >= 2" class="tl">{{ TRUST_LABEL[post.author.trust_level] }}</span>
        <a v-if="post.reply_to_floor" :href="`#floor-${post.reply_to_floor}`" class="replyto">回复 #{{ post.reply_to_floor }}</a>
        <time :datetime="post.created_at" :title="formatDateTime(post.created_at)" class="time">{{ timeAgo(post.created_at) }}</time>
        <a :href="`#floor-${post.floor}`" class="floor">#{{ post.floor }}</a>
      </header>

      <p v-if="post.status !== 'normal'" class="gone">
        {{ post.status === 'deleted' ? '这条回复已被作者删除' : '这条回复因违反社区规则已被隐藏' }}
      </p>
      <template v-else-if="editing">
        <textarea v-model="draft" class="textarea" rows="5" :aria-label="`编辑 #${post.floor} 楼`" />
        <div class="edit-actions">
          <button type="button" class="btn btn-sm" @click="editing = false">取消</button>
          <button type="button" class="btn btn-primary btn-sm" :disabled="busy" @click="saveEdit">保存修改</button>
        </div>
      </template>
      <div v-else class="prose body" v-html="post.body_html" />

      <footer v-if="post.status === 'normal' && !editing" class="actions">
        <button type="button" class="act" :class="{ on: post.liked }" :aria-pressed="post.liked" @click="like">
          <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M8 13.5s-5-3-5-7a2.8 2.8 0 0 1 5-1.7A2.8 2.8 0 0 1 13 6.5c0 4-5 7-5 7z" /></svg>
          {{ post.like_count || '赞' }}
        </button>
        <button v-if="canReply" type="button" class="act" @click="emit('reply', post)">回复</button>
        <template v-if="post.mine && !accepted">
          <button type="button" class="act" @click="startEdit">编辑</button>
          <button type="button" class="act" @click="remove">删除</button>
        </template>
        <button v-else-if="session.loggedIn" type="button" class="act quiet" @click="emit('report', post)">举报</button>
        <span v-if="post.edited_at" class="edited">已编辑</span>
      </footer>
    </div>
  </article>
</template>

<style scoped>
.post {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 12px;
  padding: 14px 4px;
  scroll-margin-top: calc(var(--header-h) + 12px);
}
.post:target {
  background: color-mix(in srgb, var(--brand) 8%, transparent);
}
.main {
  min-width: 0;
}
.head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px 10px;
  font-size: var(--fs-xs);
  color: var(--ink-3);
  margin-bottom: 6px;
}
.name {
  font-size: var(--fs-sm);
  font-weight: 700;
  color: var(--ink);
}
.op,
.tl {
  padding: 0 6px;
  border-radius: 4px;
  font-weight: 700;
  font-size: 11px;
  line-height: 18px;
}
.op {
  background: var(--ink);
  color: var(--paper);
}
.tl {
  border: 1px solid var(--line-strong);
  color: var(--ink-2);
}
.replyto {
  color: var(--ink-2);
  text-decoration: underline;
  text-underline-offset: 2px;
}
.floor {
  margin-left: auto;
  font-family: var(--font-display);
  font-variant-numeric: tabular-nums;
}
.floor:hover,
.replyto:hover {
  color: var(--ink);
}
.body {
  font-size: 15px;
  line-height: 1.75;
}
.gone {
  margin: 0;
  font-size: var(--fs-sm);
  color: var(--ink-3);
  font-style: italic;
}
.actions {
  display: flex;
  align-items: center;
  gap: 2px;
  margin: 8px 0 0 -8px;
}
.act {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  border: 0;
  background: none;
  padding: 3px 8px;
  border-radius: 5px;
  font-size: var(--fs-xs);
  font-weight: 600;
  color: var(--ink-3);
}
.act:hover {
  background: var(--surface-2);
  color: var(--ink);
}
.act svg {
  width: 14px;
  height: 14px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.5;
}
.act.on {
  color: var(--danger);
}
.act.on svg {
  fill: currentColor;
}
.quiet {
  opacity: 0.8;
}
.edited {
  margin-left: 8px;
  font-size: 11px;
  color: var(--ink-3);
}
.edit-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 8px;
}
@media (max-width: 767px) {
  .post {
    padding: 14px 0;
    gap: 10px;
  }
}
</style>
