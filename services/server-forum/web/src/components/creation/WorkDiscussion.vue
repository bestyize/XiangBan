<script setup lang="ts">
import { onScopeDispose, ref, watch } from 'vue'
import { api, ApiError } from '@/api/forum'
import { contentSocialApi, type ContentKind, type WorkDiscussion } from '@/api/contentSocial'
import { useSession } from '@/stores/session'
import AppIcon from '@/components/AppIcon.vue'
import DockButton from '@/components/comments/DockButton.vue'
import ThreadComments from '@/components/comments/ThreadComments.vue'

const props = withDefaults(defineProps<{ id: number; kind?: ContentKind; linkedTopicId?: number | null }>(), { kind: 'pack' })
const session = useSession()
const data = ref<WorkDiscussion | null>(null)
const error = ref('')
const liking = ref(false)
let generation = 0

async function load() {
  const current = ++generation
  data.value = null
  error.value = ''
  liking.value = false
  try {
    const linked = props.linkedTopicId
    const result = await contentSocialApi.discussion(props.kind, props.id).catch(async cause => {
      if (current !== generation || !linked || !(cause instanceof ApiError) || cause.status !== 404) throw cause
      const topic = await api.topic(linked)
      return {
        topic_id: topic.id, like_count: topic.like_count, reply_count: topic.reply_count, locked: topic.locked,
        liked: topic.viewer.liked, can_reply: topic.viewer.can_reply,
      }
    })
    if (current === generation) data.value = result
  } catch (e) {
    if (current === generation) error.value = e instanceof Error ? e.message : '读取评论失败'
  }
}
watch([() => props.kind, () => props.id, () => props.linkedTopicId, () => session.user?.uid, () => session.loggedIn], load, { immediate: true })
onScopeDispose(() => { ++generation })

async function ensureTopic() {
  if (data.value?.topic_id) return data.value.topic_id
  const current = generation
  const state = await contentSocialApi.ensureDiscussion(props.kind, props.id)
  if (current !== generation || !state.topic_id) throw new Error('讨论暂不可用')
  data.value = state
  return state.topic_id
}

async function like() {
  if (liking.value || !session.requireLogin()) return
  const current = generation, on = !data.value?.liked
  liking.value = true
  error.value = ''
  try {
    const topicId = await ensureTopic()
    if (current !== generation) return
    const result = await api.likeTopic(topicId, on)
    if (current === generation && data.value) data.value = { ...data.value, ...result }
  } catch (e) {
    if (current === generation) error.value = e instanceof Error ? e.message : '操作失败'
  } finally {
    if (current === generation) liking.value = false
  }
}
function changeCount(delta: number) {
  if (data.value) data.value.reply_count = Math.max(0, data.value.reply_count + delta)
}
</script>

<template>
  <ThreadComments
    v-if="data"
    detail
    :topic-id="data.topic_id"
    :count="data.reply_count"
    :locked-reason="data.locked ? '讨论已锁定，不能再回复。' : session.loggedIn && !data.can_reply ? '当前账号暂不能参与评论。' : undefined"
    :ensure-topic="ensureTopic"
    @count-change="changeCount"
  >
    <template #actions>
      <DockButton detail :class="{ 'is-liked': data.liked }" :label="data.liked ? '取消赞' : '赞'" :text="data.like_count || '赞'" :active="data.liked" :disabled="liking" @click="like"><AppIcon name="heart" :size="19" /></DockButton>
    </template>
  </ThreadComments>
  <p v-if="error" class="discussion-error" role="alert">{{ error }} <button class="btn btn-sm" type="button" @click="load">重试</button></p>
</template>

<style scoped>
.discussion-error { margin: 12px 0; color: var(--danger); }
</style>
