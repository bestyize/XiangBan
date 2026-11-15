<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Attachment, PostItem } from '@/api/types'
import ReplyWorkCards from './ReplyWorkCards.vue'
import ReplyExtrasEditor from './ReplyExtrasEditor.vue'
import AttachmentList from './AttachmentList.vue'
import CommentRow from './comments/CommentRow.vue'
import { replyExtras, type ReplyWorkRef } from '@/api/replyContent'
import { api } from '@/api/forum'
import { useSession } from '@/stores/session'

const props = defineProps<{ post: PostItem; topicId: number; topicAuthor?: string; canReply: boolean; accepted?: boolean; replyToName?: string }>()
const emit = defineEmits<{ reply: [post: PostItem]; report: [post: PostItem]; go: [floor: number]; removed: [] }>()
const session = useSession()

const editing = ref(false)
const draft = ref('')
const workRefs = ref<ReplyWorkRef[]>([])
const files = ref<Attachment[]>([])
const uploading = ref(false)
const busy = ref(false)
const error = ref('')
const actions = computed(() => props.post.status === 'normal' && !editing.value)
const floorLink = (floor: number) => `/t/${props.topicId}#floor-${floor}`

async function run(action: () => Promise<void>) {
  if (busy.value || !session.requireLogin()) return
  busy.value = true
  error.value = ''
  try { await action() }
  catch (e) { error.value = e instanceof Error ? e.message : '操作失败，请重试' }
  finally { busy.value = false }
}

function like() {
  return run(async () => {
    const res = await api.likePost(props.post.id, !props.post.liked)
    props.post.liked = res.liked
    props.post.like_count = res.like_count
  })
}

function startEdit() {
  return run(async () => {
    const source = await api.postSource(props.post.id)
    draft.value = source.body
    workRefs.value = source.work_refs
    files.value = source.attachments
    editing.value = true
  })
}

function saveEdit() {
  if (uploading.value || !draft.value.trim()) return
  if (files.value.some(a => !Number.isInteger(a.price) || a.price < 0 || a.price > 10)) {
    error.value = '附件价格须为 0～10 的整数'
    return
  }
  return run(async () => {
    const res = await api.updatePost(props.post.id, draft.value, replyExtras(workRefs.value, files.value))
    props.post.body_html = res.body_html
    props.post.work_refs = res.work_refs
    props.post.attachments = res.attachments
    props.post.edited_at = new Date().toISOString()
    editing.value = false
  })
}

function remove() {
  if (!confirm('删除这条回复？删除后不能恢复。')) return
  return run(async () => {
    await api.deletePost(props.post.id)
    props.post.status = 'deleted'
    props.post.body_html = ''
    emit('removed')
  })
}
</script>

<template>
  <article v-if="post.blocked" :id="`floor-${post.floor}`" class="blocked-row">
    <span>你已屏蔽 {{ post.author.nickname }}，这条回复已折叠。</span>
    <RouterLink :to="`/u/${post.author.uid}`">去取消屏蔽</RouterLink>
  </article>
  <CommentRow
    v-else
    :id="`floor-${post.floor}`"
    :author="post.author"
    :created-at="post.created_at"
    :can-reply="actions && canReply"
    :can-like="actions"
    :like-count="post.like_count"
    :liked="post.liked"
    :busy="busy"
    @like="like"
    @reply="emit('reply', post)"
  >
    <template v-if="post.author.uid === topicAuthor" #byline><span class="op">楼主</span></template>
    <template v-if="post.reply_to_floor" #parent>
      <a :href="floorLink(post.reply_to_floor)" @click.exact.prevent="emit('go', post.reply_to_floor)">回复 {{ replyToName || `#${post.reply_to_floor}` }}</a>
    </template>
    <p v-if="post.status !== 'normal'" class="gone">
      {{ post.status === 'deleted' ? '这条回复已被作者删除' : '这条回复因违反社区规则已被隐藏' }}
    </p>
    <template v-else-if="editing">
      <textarea v-model="draft" class="textarea" rows="5" :aria-label="`编辑 #${post.floor} 楼`" />
      <ReplyExtrasEditor v-if="session.info" v-model:work-refs="workRefs" v-model:attachments="files" :max-bytes="session.info.limits.upload_max_bytes" :disabled="busy" @uploading="uploading = $event" />
      <div class="edit-actions">
        <button type="button" class="btn btn-sm" :disabled="busy || uploading" @click="editing = false; error = ''">取消</button>
        <button type="button" class="btn btn-primary btn-sm" :disabled="busy || uploading || !draft.trim()" @click="saveEdit">保存修改</button>
      </div>
    </template>
    <template v-else>
      <div class="prose post-body" v-html="post.body_html" />
      <ReplyWorkCards :items="post.work_refs ?? []" />
      <AttachmentList v-if="post.attachments?.length" :items="post.attachments" restricted />
    </template>
    <p v-if="error" class="post-error" role="alert">{{ error }}</p>
    <template #actions>
      <template v-if="actions">
        <template v-if="post.mine && !accepted">
          <button type="button" :disabled="busy" @click="startEdit">编辑</button>
          <button type="button" :disabled="busy" @click="remove">删除</button>
        </template>
        <button v-else-if="session.loggedIn" type="button" @click="emit('report', post)">举报</button>
        <span v-if="post.edited_at">已编辑</span>
      </template>
      <a class="floor" :href="floorLink(post.floor)" @click.exact.prevent="emit('go', post.floor)">#{{ post.floor }}</a>
    </template>
  </CommentRow>
</template>

<style scoped>
.op { flex: none; padding: 0 6px; border-radius: 4px; background: var(--surface-2); color: var(--ink-3); font-size: 11px; }
.post-body { margin-top: 4px; font-size: 15px; line-height: 1.6; overflow-wrap: anywhere; }
.post-body :deep(> :first-child) { margin-top: 0; }
.post-body :deep(> :last-child) { margin-bottom: 0; }
.post-body :deep(pre), .post-body :deep(table) { max-width: 100%; overflow: auto; }
.post-body :deep(table) { display: block; }
.gone, .post-error { margin: 4px 0 0; color: var(--ink-3); font-size: var(--fs-sm); }
.post-error { color: var(--danger); }
.floor { margin-inline-start: auto; font-variant-numeric: tabular-nums; }
.floor:hover { color: var(--ink); }
.edit-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 8px; }
.blocked-row { padding: 12px 0; color: var(--ink-3); font-size: var(--fs-sm); }
.blocked-row a { margin-inline-start: 8px; text-decoration: underline; }
@media (max-width: 767px) { .floor { display: none; } }
</style>
