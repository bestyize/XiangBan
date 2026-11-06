<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { api } from '@/api/forum'
import { contentSocialApi, type ContentKind, type WorkDiscussion } from '@/api/contentSocial'
import { useSession } from '@/stores/session'
import { useNarrow } from '@/lib/viewport'
import AppIcon from '@/components/AppIcon.vue'
import CommentDock from '@/components/comments/CommentDock.vue'
import DockButton from '@/components/comments/DockButton.vue'
const props = withDefaults(defineProps<{ id: number; kind?: ContentKind }>(), { kind: 'pack' })
const narrow = useNarrow()
const session = useSession(), router = useRouter(), data = ref<WorkDiscussion | null>(null), error = ref(''), busy = ref(false)
let sequence = 0
async function load() { const current = ++sequence; data.value = null; error.value = ''; try { const result = await contentSocialApi.discussion(props.kind, props.id); if (current === sequence) data.value = result } catch(e) { if (current === sequence) error.value = e instanceof Error ? e.message : '读取讨论失败' } }
watch(() => [props.id, session.user?.uid], load, { immediate:true })
async function interact(action: 'like' | 'discuss' | 'compose') {
  if (!session.loggedIn) { session.loginOpen = true; return }
  if (busy.value || !data.value) return
  const current = sequence, id = props.id; busy.value = true; error.value = ''
  try {
    const state = data.value.topic_id ? data.value : await contentSocialApi.ensureDiscussion(props.kind, id)
    if (!state.topic_id) throw new Error('讨论暂不可用')
    if (action !== 'like') { if (current === sequence) await router.push(action === 'compose' ? { path: `/t/${state.topic_id}`, query: { compose: '1' } } : `/t/${state.topic_id}`) }
    else { const result = await api.likeTopic(state.topic_id, !state.liked); if (current === sequence) data.value = { ...state, ...result } }
  } catch(e) { if (current === sequence) error.value = e instanceof Error ? e.message : '操作失败' }
  finally { busy.value = false }
}
</script>
<template>
  <!-- 手机：和帖子、日报等详情页一样用底部互动栏；桌面：保留原来的按钮。 -->
  <CommentDock v-if="narrow && data" :count="data.reply_count" :disabled="busy || data.locked" @compose="interact('compose')" @jump="interact('discuss')">
    <DockButton label="赞" :text="data.like_count || '赞'" :active="data.liked" @click="interact('like')"><AppIcon name="heart" :size="24" /></DockButton>
  </CommentDock>
  <div v-if="!narrow" class="discussion"><template v-if="data"><button class="btn" type="button" :disabled="busy" :aria-pressed="data.liked" @click="interact('like')">{{ data.liked ? '已赞' : '赞' }} {{ data.like_count }}</button><RouterLink v-if="data.topic_id" class="btn" :to="`/t/${data.topic_id}`">讨论 {{ data.reply_count }}</RouterLink><button v-else class="btn" type="button" :disabled="busy" @click="interact('discuss')">参与讨论</button></template><span v-if="error" role="alert">{{ error }} <button class="btn" type="button" @click="load">重试</button></span></div>
  <p v-else-if="error" class="discussion-error" role="alert">{{ error }} <button class="btn" type="button" @click="load">重试</button></p>
</template>
<style scoped>.discussion { display:flex; flex-wrap:wrap; gap:8px; align-items:center; margin:12px 0; }.btn { min-height:44px; }.discussion-error { margin:12px 0; }</style>
