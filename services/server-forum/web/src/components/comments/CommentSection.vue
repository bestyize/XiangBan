<script setup lang="ts">
import { computed, nextTick, onScopeDispose, ref, watch } from 'vue'
import type { NewsComment } from '@/api/discover'
import { useSession } from '@/stores/session'
import PagerNav from '@/components/PagerNav.vue'
import RequestState from '@/components/community/RequestState.vue'
import { useCommunityResource } from '@/components/community/resource'
import CommentComposer from './CommentComposer.vue'
import CommentRow from './CommentRow.vue'
import CommentEmpty from './CommentEmpty.vue'
import CommentDock from './CommentDock.vue'
import CommentList from './CommentList.vue'
import CommentSheet from './CommentSheet.vue'
import type { CommentAdapter } from './types'

const props = defineProps<{ scope: string; adapter: CommentAdapter; column?: boolean }>()
const session = useSession()
const page = ref(1)
const total = ref<number | null>(null)
const reply = ref<NewsComment | null>(null)
const busy = ref(false)
const sending = ref(false)
const listError = ref('')
const sendError = ref('')
const open = ref(false)
const list = ref<InstanceType<typeof CommentList> | null>(null)
const composer = ref<InstanceType<typeof CommentComposer> | null>(null)
let context = 0

const { data, loading, error, load, refresh } = useCommunityResource(() => props.adapter.list(page.value))
const names = computed(() => new Map(data.value?.items.map(item => [item.id, item.author.nickname])))
watch(data, result => { if (result) total.value = result.total })
watch([() => props.scope, () => session.user?.uid, () => session.loggedIn], ([scope], previous) => {
  ++context
  if (scope !== previous?.[0]) open.value = false
  page.value = 1
  total.value = null
  reply.value = null
  busy.value = false
  sending.value = false
  listError.value = ''
  sendError.value = ''
  composer.value?.reset()
  void load()
}, { immediate: true })
onScopeDispose(() => { ++context })

async function startReply(item: NewsComment | null = null) {
  if (item && !session.requireLogin()) return
  reply.value = item
  sendError.value = ''
  open.value = true
  await nextTick()
  composer.value?.focus()
}

function go(pageNumber: number) {
  if (pageNumber === page.value) return refresh()
  page.value = pageNumber
  list.value?.jump()
  return load()
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
    reply.value = null
    open.value = false
    await go(1)
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
function remove(item: NewsComment) {
  if (!window.confirm('删除这条评论？')) return
  if (reply.value?.id === item.id) reply.value = null
  const target = data.value?.items.length === 1 ? Math.max(1, page.value - 1) : page.value
  void act(() => props.adapter.remove(item.id), target)
}
</script>

<template>
  <div :class="{ 'comment-column': column }">
    <CommentDock :count="total" @compose="startReply()" @discuss="list?.jump()">
      <template #default="{ narrow }"><slot v-if="narrow" name="actions" /></template>
    </CommentDock>
    <CommentList id="comments" ref="list" :count="total">
      <RequestState :loading="loading" :error="error" @retry="load" />
      <p v-if="listError" class="list-error" role="alert">{{ listError }}</p>
      <template v-if="data">
        <CommentRow
          v-for="item in data.items"
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
          @like="likes(item)"
          @reply="startReply(item)"
          @remove="remove(item)"
        />
        <CommentEmpty v-if="!data.items.length" />
        <PagerNav :page="data.page" :page-size="data.page_size" :total="data.total" @go="go" />
      </template>
    </CommentList>
    <CommentSheet :open="open" @close="open = false">
      <CommentComposer
        ref="composer"
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
.list-error { margin: 12px 0; color: var(--danger); }
.comment-list { padding-inline: 16px; background: var(--surface); }
@media (min-width: 768px) { .comment-list { padding-inline: 24px; border: 1px solid var(--line); border-radius: 14px; } }
@media (max-width: 767px) { .comment-list { margin-inline: calc(-1 * var(--comments-bleed, 8px)); } }
</style>
