<script setup lang="ts">
import { computed, nextTick, onScopeDispose, reactive, ref, watch } from 'vue'
import { api } from '@/api/forum'
import { replyExtras, type ReplyWorkRef } from '@/api/replyContent'
import type { Attachment, Page, PostItem } from '@/api/types'
import { useSession } from '@/stores/session'
import AppIcon from '@/components/AppIcon.vue'
import DetailCommentForm from './DetailCommentForm.vue'
import PagerNav from '@/components/PagerNav.vue'
import PostEntry from '@/components/PostEntry.vue'
import ReplyExtrasEditor from '@/components/ReplyExtrasEditor.vue'
import ReportDialog from '@/components/ReportDialog.vue'
import RequestState from '@/components/community/RequestState.vue'
import { useCommunityResource } from '@/components/community/resource'
import { MOBILE_SITE_QUERY, useNarrow } from '@/lib/viewport'
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
  detail?: boolean
}>()
const emit = defineEmits<{ 'count-change': [delta: number] }>()
const session = useSession()
const narrow = useNarrow(MOBILE_SITE_QUERY)
const inline = computed(() => props.detail && !narrow.value)
const open = ref(false)
const page = ref(1)
const newestFirst = ref(!!props.detail)
const targetFloor = ref<number>()
const reply = ref<PostItem | null>(null)
const reportTarget = ref<{ type: 'post'; id: number } | null>(null)
const workRefs = ref<ReplyWorkRef[]>([])
const files = ref<Attachment[]>([])
const uploading = ref(false)
const sending = ref(false)
const sendError = ref('')
const draft = ref('')
const inlineEditor = ref<InstanceType<typeof DetailCommentForm>>()
const extrasKey = ref(0)
const list = ref<InstanceType<typeof CommentList> | null>(null)
const composer = ref<InstanceType<typeof CommentComposer> | null>(null)
let generation = 0

const { data, loading, error, load, refresh } = useCommunityResource<Page<PostItem>>(async () =>
  props.topicId ? reactive(await api.posts(props.topicId, page.value)) : { items: [], page: 1, page_size: PAGE_SIZE, total: 0 },
)
const names = computed(() => new Map(data.value?.items.map(item => [item.id, item.author.nickname])))
const orderedPosts = computed(() => newestFirst.value ? data.value?.items.slice().reverse() : data.value?.items)
const lastPage = computed(() => Math.max(1, Math.ceil((data.value?.total ?? 0) / (data.value?.page_size ?? PAGE_SIZE))))
const displayPage = computed(() => newestFirst.value ? lastPage.value - page.value + 1 : page.value)

function resetDraft() {
  composer.value?.reset()
  draft.value = ''
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
watch(() => props.topicId, async () => {
  if (sending.value) return
  await load()
  if (newestFirst.value && !targetFloor.value && page.value === 1 && lastPage.value > 1) {
    page.value = lastPage.value
    await load()
  }
}, { immediate: true })
onScopeDispose(() => { ++generation })

function jumpToComments(smooth = false) { list.value?.jump(smooth) }

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

function toggleSort() {
  newestFirst.value = !newestFirst.value
  targetFloor.value = undefined
  const first = newestFirst.value ? lastPage.value : 1
  if (first !== page.value) void go(first)
}

async function openComposer() {
  open.value = true
  await nextTick()
  composer.value?.focus()
}

async function startReply(item: PostItem | null = null) {
  if (props.lockedReason || (item && !session.requireLogin())) return
  reply.value = item
  sendError.value = ''
  if (inline.value) inlineEditor.value?.focus()
  else await openComposer()
}
defineExpose({ startReply, goToFloor, jumpToComments })

function submitInline(body: string) {
  if (workRefs.value.length || files.value.length || composer.value?.hasImages) {
    void openComposer()
  } else {
    void submit(body)
  }
}

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
  <CommentDock :count="count" :disabled="!!lockedReason" :detail="detail"
    :label="lockedReason ? '暂不可评论' : detail ? '说点什么…' : undefined"
    @compose="startReply()" @discuss="inline ? inlineEditor?.focus() : jumpToComments(detail)">
    <template v-if="detail" #compose>
      <DetailCommentForm ref="inlineEditor" v-model="draft" :limit="10000" :locked="!!lockedReason" :busy="sending"
        :uploading="uploading" :reply-to="reply?.author.nickname" @expand="openComposer"
        @submit="submitInline" @cancel-reply="reply = null" />
    </template>
    <template #default="{ narrow }"><slot name="actions" :narrow="narrow" /></template>
  </CommentDock>
  <div v-if="detail && narrow" class="xhs-divider" />
  <CommentList id="replies" ref="list" :count="count" :detail="detail" :class="detail ? narrow ? 'xhs-h5-comments' : 'xhs-pc-comments' : undefined"
    :prompt="detail && narrow && !lockedReason" @compose="startReply()">
    <template v-if="detail" #tools>
      <button type="button" class="xhs-sort-btn" :disabled="!data || loading"
        aria-label="最新评论优先" :aria-pressed="newestFirst" @click="toggleSort">
        <AppIcon name="sort" :size="14" />
      </button>
    </template>
    <p v-if="lockedReason" class="comment-note">{{ lockedReason }}</p>
    <p v-if="inline && !open && sendError" class="inline-error" role="alert">{{ sendError }}</p>
    <RequestState :loading="loading" :error="error" @retry="go(page, targetFloor)" />
    <template v-if="data">
      <div :class="detail ? 'xhs-comments-list' : 'comment-entries'">
        <div v-for="item in orderedPosts" :key="item.id">
          <slot name="before-post" :post="item" />
          <PostEntry
            :post="item"
            :topic-id="topicId!"
            :topic-author="topicAuthor"
            :can-reply="!lockedReason"
            :accepted="acceptedId === item.id"
            :reply-to-name="item.reply_to ? names.get(item.reply_to) : undefined"
            :detail="detail"
            :class="{ targeted: targetFloor === item.floor }"
            @reply="startReply(item)"
            @go="goToFloor"
            @report="post => reportTarget = { type: 'post', id: post.id }"
            @removed="removed(item)"
          />
          <slot name="after-post" :post="item" />
        </div>
      </div>
      <CommentEmpty v-if="!data.items.length" :read-only="!!lockedReason" />
      <PagerNav :page="displayPage" :page-size="data.page_size" :total="data.total" @go="go(newestFirst ? lastPage - $event + 1 : $event)" />
    </template>
  </CommentList>
  <CommentSheet :open="open" :detail="detail && narrow" @close="open = false">
    <CommentComposer
      ref="composer"
      v-model:draft="draft"
      :limit="10000"
      :detail="detail && narrow"
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
.comment-entries { display: contents; }
.xhs-sort-btn:disabled { opacity: .5; cursor: default; }
.inline-error { margin: 0 0 12px; color: var(--danger); font-size: 13px; }
</style>
