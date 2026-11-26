<script setup lang="ts">
import { computed, nextTick, onScopeDispose, ref, watch } from 'vue'
import type { NewsComment } from '@/api/discover'
import { useSession } from '@/stores/session'
import { MOBILE_SITE_QUERY, useNarrow } from '@/lib/viewport'
import AppIcon from '@/components/AppIcon.vue'
import PagerNav from '@/components/PagerNav.vue'
import RequestState from '@/components/community/RequestState.vue'
import { confirmDialog } from '@/ui'
import { useCommunityResource } from '@/components/community/resource'
import CommentComposer from './CommentComposer.vue'
import CommentRow from './CommentRow.vue'
import CommentEmpty from './CommentEmpty.vue'
import CommentDock from './CommentDock.vue'
import CommentList from './CommentList.vue'
import CommentSheet from './CommentSheet.vue'
import DetailCommentForm from './DetailCommentForm.vue'
import type { CommentAdapter } from './types'

const props = defineProps<{ scope: string; adapter: CommentAdapter; column?: boolean; detail?: boolean }>()
const session = useSession()
const narrow = useNarrow(MOBILE_SITE_QUERY)
const inline = computed(() => props.detail && !narrow.value)
const page = ref(1)
const total = ref<number | null>(null)
const reply = ref<NewsComment | null>(null)
const busy = ref(false)
const sending = ref(false)
const listError = ref('')
const sendError = ref('')
const open = ref(false)
const draft = ref('')
const oldestFirst = ref(false)
const list = ref<InstanceType<typeof CommentList> | null>(null)
const composer = ref<InstanceType<typeof CommentComposer> | null>(null)
const inlineEditor = ref<InstanceType<typeof DetailCommentForm>>()
let context = 0

const { data, loading, error, load, refresh } = useCommunityResource(() => props.adapter.list(page.value))
const names = computed(() => new Map(data.value?.items.map(item => [item.id, item.author.nickname])))
const items = computed(() => {
  const result = data.value?.items ?? []
  return oldestFirst.value ? result.slice().reverse() : result
})
const lastPage = computed(() => data.value ? Math.max(1, Math.ceil(data.value.total / data.value.page_size)) : 1)
const displayPage = computed(() => oldestFirst.value ? lastPage.value - page.value + 1 : page.value)
watch(data, result => { if (result) total.value = result.total })
watch([() => props.scope, () => session.user?.uid, () => session.loggedIn], ([scope], previous) => {
  ++context
  if (scope !== previous?.[0]) open.value = false
  page.value = 1
  oldestFirst.value = false
  total.value = null
  reply.value = null
  busy.value = false
  sending.value = false
  listError.value = ''
  sendError.value = ''
  composer.value?.reset()
  draft.value = ''
  void load()
}, { immediate: true })
onScopeDispose(() => { ++context })

function startReply(item: NewsComment | null = null) {
  if (item && !session.requireLogin()) return
  reply.value = item
  sendError.value = ''
  if (inline.value) inlineEditor.value?.focus()
  else void openComposer()
}

async function openComposer() {
  open.value = true
  await nextTick()
  composer.value?.focus()
}

function submitInline(body: string) {
  if (composer.value?.hasImages) void openComposer()
  else void submit(body, [])
}

function go(pageNumber: number) {
  if (pageNumber === page.value) return refresh()
  page.value = pageNumber
  list.value?.jump()
  return load()
}

function toggleSort() {
  oldestFirst.value = !oldestFirst.value
  const first = oldestFirst.value ? lastPage.value : 1
  if (first !== page.value) void go(first)
}

async function submit(body: string, images: string[]) {
  if (busy.value || sending.value || !session.requireLogin()) return
  const current = context
  sending.value = true
  sendError.value = ''
  try {
    await props.adapter.post(body, reply.value?.id, images.length ? images : undefined)
    if (current !== context) return
    composer.value?.reset()
    draft.value = ''
    reply.value = null
    open.value = false
    await go(oldestFirst.value ? lastPage.value : 1)
    if (current === context) list.value?.jump()
  } catch (e) {
    if (current === context) sendError.value = e instanceof Error ? e.message : '发布失败，请重试'
  } finally { if (current === context) sending.value = false }
}

async function act(work: () => Promise<unknown>, targetPage = page.value) {
  if (busy.value || sending.value || !session.requireLogin()) return
  const current = context
  busy.value = true
  listError.value = ''
  try {
    await work()
    if (current === context) await go(targetPage)
  } catch (e) {
    if (current === context) listError.value = e instanceof Error ? e.message : '操作失败，请重试'
  } finally { if (current === context) busy.value = false }
}

const likes = (item: NewsComment) => void act(() => props.adapter.like(item.id, !item.liked))
async function remove(item: NewsComment) {
  if (!await confirmDialog({ title: '删除这条评论？', description: '删除后其他人将看不到该内容，操作无法撤销。', confirmLabel: '确认删除', danger: true })) return
  if (reply.value?.id === item.id) reply.value = null
  const target = data.value?.items.length === 1 ? Math.max(1, page.value - 1) : page.value
  void act(() => props.adapter.remove(item.id), target)
}
</script>

<template>
  <div :class="{ 'comment-column': column && !detail }">
    <CommentDock :count="total" :detail="detail" :label="detail ? '说点什么…' : undefined"
      @compose="startReply()" @discuss="inline ? inlineEditor?.focus() : list?.jump(detail)">
      <template v-if="detail" #compose>
        <DetailCommentForm ref="inlineEditor" v-model="draft" :limit="2000" :busy="busy || sending" :reply-to="reply?.author.nickname"
          @expand="openComposer" @submit="submitInline" @cancel-reply="reply = null" />
      </template>
      <template #default="{ narrow: mobileDock }"><slot v-if="detail || mobileDock" name="actions" /></template>
    </CommentDock>
    <div v-if="detail && narrow" class="xhs-divider" />
    <CommentList id="comments" ref="list" :count="total" :detail="detail"
      :class="detail ? narrow ? 'xhs-h5-comments' : 'xhs-pc-comments' : undefined"
      :prompt="detail && narrow" @compose="startReply()">
      <template v-if="detail" #tools>
        <button type="button" class="xhs-sort-btn" aria-label="最新评论优先" :aria-pressed="!oldestFirst"
          :disabled="!data || loading || busy || sending" @click="toggleSort"><AppIcon name="sort" :size="14" /></button>
      </template>
      <RequestState :loading="loading" :error="error" @retry="load" />
      <p v-if="listError" class="list-error" role="alert">{{ listError }}</p>
      <p v-if="inline && !open && sendError" class="list-error" role="alert">{{ sendError }}</p>
      <template v-if="data">
        <div :class="detail ? 'xhs-comments-list' : 'comment-entries'">
          <CommentRow
            v-for="item in items"
            :key="item.id"
            :author="item.author"
            :body="item.body"
            :images="item.images"
            :created-at="item.created_at"
            :reply-to-name="item.reply_to ? (names.get(item.reply_to) ?? `评论 #${item.reply_to}`) : undefined"
            :like-count="item.like_count"
            :liked="item.liked"
            :mine="item.mine"
            :busy="busy || sending"
            :detail="detail"
            @like="likes(item)"
            @reply="startReply(item)"
            @remove="remove(item)"
          />
        </div>
        <CommentEmpty v-if="!data.items.length" />
        <PagerNav :page="displayPage" :page-size="data.page_size" :total="data.total"
          @go="go(oldestFirst ? lastPage - $event + 1 : $event)" />
      </template>
    </CommentList>
    <CommentSheet :open="open" :detail="detail && narrow" @close="open = false">
      <CommentComposer
        ref="composer"
        v-model:draft="draft"
        :detail="detail && narrow"
        :limit="2000"
        :reply-to="reply?.author.nickname"
        :busy="sending"
        :error="sendError"
        @submit="submit"
        @cancel-reply="reply = null"
        @close="open = false"
      />
    </CommentSheet>
  </div>
</template>

<style scoped>
.comment-column { max-width: 760px; margin-inline: auto; }
.comment-entries { display: contents; }
.xhs-sort-btn:disabled { opacity: .5; cursor: default; }
.list-error { margin: 12px 0; color: var(--danger); }
.comment-list { padding-inline: 16px; background: var(--surface); }
@media (min-width: 768px) { .comment-list { padding-inline: 24px; border: 1px solid var(--line); border-radius: 14px; } }
@media (max-width: 767px) { .comment-list { margin-inline: calc(-1 * var(--comments-bleed, 8px)); } }
</style>
