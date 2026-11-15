<script setup lang="ts">
import { computed, nextTick, onScopeDispose, reactive, ref, watch } from 'vue'
import { api } from '@/api/forum'
import { replyExtras, type ReplyWorkRef } from '@/api/replyContent'
import type { Attachment, Page, PostItem } from '@/api/types'
import { useSession } from '@/stores/session'
import PagerNav from '@/components/PagerNav.vue'
import PostEntry from '@/components/PostEntry.vue'
import ReplyExtrasEditor from '@/components/ReplyExtrasEditor.vue'
import ReportDialog from '@/components/ReportDialog.vue'
import RequestState from '@/components/community/RequestState.vue'
import { useCommunityResource } from '@/components/community/resource'
import CommentComposer from './CommentComposer.vue'
import CommentDock from './CommentDock.vue'
import CommentEmpty from './CommentEmpty.vue'
import CommentList from './CommentList.vue'
import CommentSheet from './CommentSheet.vue'

const PAGE_SIZE = 50
const props = defineProps<{
  topicId: number | null
  count: number
  topicAuthor?: string
  acceptedId?: number | null
  lockedReason?: string
  ensureTopic?: () => Promise<number>
}>()
const emit = defineEmits<{ 'count-change': [delta: number] }>()
const session = useSession()
const open = ref(false)
const page = ref(1)
const targetFloor = ref<number>()
const reply = ref<PostItem | null>(null)
const reportTarget = ref<{ type: 'post'; id: number } | null>(null)
const workRefs = ref<ReplyWorkRef[]>([])
const files = ref<Attachment[]>([])
const uploading = ref(false)
const sending = ref(false)
const sendError = ref('')
const extrasKey = ref(0)
const list = ref<InstanceType<typeof CommentList> | null>(null)
const composer = ref<InstanceType<typeof CommentComposer> | null>(null)
let generation = 0

const { data, loading, error, load, refresh } = useCommunityResource<Page<PostItem>>(async () =>
  props.topicId ? reactive(await api.posts(props.topicId, page.value)) : { items: [], page: 1, page_size: PAGE_SIZE, total: 0 },
)
const names = computed(() => new Map(data.value?.items.map(item => [item.id, item.author.nickname])))

function resetDraft() {
  composer.value?.reset()
  reply.value = null
  workRefs.value = []
  files.value = []
  uploading.value = false
  sendError.value = ''
  ++extrasKey.value
}

watch([() => session.user?.uid, () => session.loggedIn], () => {
  ++generation
  resetDraft()
  sending.value = false
  reportTarget.value = null
  void load()
})
watch(() => props.topicId, () => { if (!sending.value) void load() }, { immediate: true })
onScopeDispose(() => { ++generation })

function jumpToComments() { list.value?.jump() }

function scrollToFloor(floor: number) {
  document.getElementById(`floor-${floor}`)?.scrollIntoView({ block: 'center' })
}

async function go(pageNumber: number, floor?: number) {
  const samePage = page.value === pageNumber
  page.value = pageNumber
  targetFloor.value = floor
  if (!floor && !samePage) jumpToComments()
  await (samePage ? refresh() : load())
  await nextTick()
  if (floor && targetFloor.value === floor && data.value?.page === pageNumber) {
    scrollToFloor(floor)
  }
}

function goToFloor(floor: number) {
  if (data.value?.items.some(item => item.floor === floor)) {
    targetFloor.value = floor
    scrollToFloor(floor)
  } else return go(Math.ceil(floor / PAGE_SIZE), floor)
}

async function startReply(item: PostItem | null = null) {
  if (props.lockedReason || (item && !session.requireLogin())) return
  reply.value = item
  sendError.value = ''
  open.value = true
  await nextTick()
  composer.value?.focus()
}
defineExpose({ startReply, goToFloor, jumpToComments })

async function submit(body: string) {
  if (props.lockedReason || sending.value || uploading.value || !session.requireLogin()) return
  if (files.value.some(a => !Number.isInteger(a.price) || a.price < 0 || a.price > 10)) {
    sendError.value = '附件价格须为 0～10 的整数'
    return
  }
  const current = generation
  const parent = reply.value?.id
  const extras = replyExtras(workRefs.value, files.value)
  sending.value = true
  sendError.value = ''
  try {
    const id = props.topicId ?? await props.ensureTopic?.()
    if (current !== generation) return
    if (!id) throw new Error('讨论暂不可用')
    const { floor } = await api.reply(id, body, parent, extras)
    if (current !== generation) return
    resetDraft()
    open.value = false
    emit('count-change', 1)
    await go(Math.ceil(floor / PAGE_SIZE), floor)
  } catch (e) {
    if (current === generation) sendError.value = e instanceof Error ? e.message : '发布失败，请重试'
  } finally {
    if (current === generation) sending.value = false
  }
}

function removed(item: PostItem) {
  if (reply.value?.id === item.id) reply.value = null
  emit('count-change', -1)
}
</script>

<template>
  <CommentDock :count="count" :disabled="!!lockedReason" :label="lockedReason ? '暂不可评论' : undefined" @compose="startReply()" @discuss="jumpToComments">
    <template #default="{ narrow }"><slot name="actions" :narrow="narrow" /></template>
  </CommentDock>
  <CommentList id="replies" ref="list" :count="count">
    <p v-if="lockedReason" class="comment-note">{{ lockedReason }}</p>
    <RequestState :loading="loading" :error="error" @retry="go(page, targetFloor)" />
    <template v-if="data">
      <div v-for="item in data.items" :key="item.id">
        <slot name="before-post" :post="item" />
        <PostEntry
          :post="item"
          :topic-id="topicId!"
          :topic-author="topicAuthor"
          :can-reply="!lockedReason"
          :accepted="acceptedId === item.id"
          :reply-to-name="item.reply_to ? names.get(item.reply_to) : undefined"
          :class="{ targeted: targetFloor === item.floor }"
          @reply="startReply(item)"
          @go="goToFloor"
          @report="post => reportTarget = { type: 'post', id: post.id }"
          @removed="removed(item)"
        />
        <slot name="after-post" :post="item" />
      </div>
      <CommentEmpty v-if="!data.items.length" :read-only="!!lockedReason" />
      <PagerNav :page="data.page" :page-size="data.page_size" :total="data.total" @go="go" />
    </template>
  </CommentList>
  <CommentSheet :open="open" @close="open = false">
    <CommentComposer
      ref="composer"
      :limit="10000"
      markdown
      :locked-reason="lockedReason"
      :reply-to="reply?.author.nickname"
      :busy="sending"
      :uploading-extras="uploading"
      :error="sendError"
      @submit="submit"
      @cancel-reply="reply = null"
      @close="open = false"
    >
      <template v-if="session.info" #more>
        <ReplyExtrasEditor
          :key="extrasKey"
          v-model:work-refs="workRefs"
          v-model:attachments="files"
          :max-bytes="session.info.limits.upload_max_bytes"
          :disabled="sending"
          embedded
          @uploading="uploading = $event"
        />
      </template>
    </CommentComposer>
  </CommentSheet>
  <ReportDialog :target="reportTarget" @close="reportTarget = null" />
</template>

<style scoped>
.comment-note { margin: 4px 0; color: var(--ink-3); font-size: var(--fs-sm); }
.targeted { background: var(--brand-soft); border-radius: 8px; }
</style>
