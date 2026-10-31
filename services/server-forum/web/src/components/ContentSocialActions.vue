<script setup lang="ts">
import { ref, watch } from 'vue'
import { contentSocialApi, CONTENT_REPORT_REASONS, type ContentKind } from '@/api/contentSocial'
import { useSession } from '@/stores/session'
import ModalDialog from './ModalDialog.vue'
const props = defineProps<{ kind: ContentKind; id: number }>()
const session = useSession()
const bookmarked = ref(false), loaded = ref(false), loading = ref(false), busy = ref(false), error = ref(''), message = ref(''), reportOpen = ref(false), reason = ref(''), detail = ref('')
let sequence = 0
async function load() {
  const current = ++sequence
  loaded.value = false; bookmarked.value = false; error.value = ''; message.value = ''; reportOpen.value = false
  if (!session.loggedIn) { loading.value = false; return }
  loading.value = true
  try { const data = await contentSocialApi.state(props.kind, props.id); if (current === sequence) { bookmarked.value = data.bookmarked; loaded.value = true } }
  catch (e) { if (current === sequence) error.value = e instanceof Error ? e.message : '读取收藏失败' }
  finally { if (current === sequence) loading.value = false }
}
function login() { session.loginOpen = true }
async function bookmark() {
  if (!session.loggedIn) { login(); return }
  if (!loaded.value || busy.value) return
  const current = sequence, kind = props.kind, id = props.id
  busy.value = true; error.value = ''; message.value = ''
  try { const data = await contentSocialApi.bookmark(kind, id, !bookmarked.value); if (current === sequence) { bookmarked.value = data.bookmarked; message.value = data.bookmarked ? '已加入作品收藏' : '已取消收藏' } }
  catch (e) { if (current === sequence) error.value = e instanceof Error ? e.message : '收藏失败，请重试' }
  finally { busy.value = false }
}
function openReport() { if (!session.loggedIn) { login(); return }; reason.value = ''; detail.value = ''; error.value = ''; reportOpen.value = true }
async function submitReport() {
  if (!reason.value || busy.value) return
  const current = sequence
  busy.value = true; error.value = ''
  try { await contentSocialApi.report(props.kind, props.id, reason.value, detail.value); if (current === sequence) { reportOpen.value = false; message.value = '已提交举报，管理员会审核处理' } }
  catch (e) { if (current === sequence) error.value = e instanceof Error ? e.message : '提交失败，请重试' }
  finally { busy.value = false }
}
watch(() => [props.kind, props.id, session.user?.uid], load, { immediate: true })
</script>
<template>
  <div class="social">
    <button class="btn" type="button" :disabled="busy || loading || (session.loggedIn && !loaded)" :aria-pressed="bookmarked" @click="bookmark">{{ bookmarked ? '已收藏 · 取消' : '收藏' }}</button>
    <button class="btn" type="button" :disabled="busy" @click="openReport">举报</button>
    <RouterLink to="/me/content-bookmarks">作品收藏</RouterLink>
    <span v-if="message" role="status">{{ message }}</span>
    <span v-if="error && !reportOpen" class="error" role="alert">{{ error }} <button type="button" class="btn btn-sm" :disabled="busy || loading" @click="load">重新读取</button></span>
  </div>
  <ModalDialog :open="reportOpen" title="举报社区内容" @close="!busy && (reportOpen = false)">
    <form class="report" @submit.prevent="submitReport">
      <p>说明仅供管理员审核，不会向内容作者公开。</p>
      <label class="field"><span>举报原因</span><select v-model="reason" class="input" required :disabled="busy"><option disabled value="">请选择</option><option v-for="[key, text] in CONTENT_REPORT_REASONS" :key="key" :value="key">{{ text }}</option></select></label>
      <label class="field"><span>补充说明（可选，最多 500 字）</span><textarea v-model="detail" class="input" rows="4" maxlength="500" :disabled="busy" /></label>
      <p v-if="error" role="alert" class="error">{{ error }}</p>
      <div class="social"><button class="btn" type="button" :disabled="busy" @click="reportOpen = false">取消</button><button class="btn btn-primary" :disabled="busy || !reason">{{ busy ? '正在提交…' : '提交举报' }}</button></div>
    </form>
  </ModalDialog>
</template>
<style scoped>
.social { display:flex; align-items:center; flex-wrap:wrap; gap:8px 12px; margin:14px 0; font-size:var(--fs-sm); }
.report { display:grid; gap:12px; }.report p { margin:0; color:var(--ink-2); }.error { color:var(--danger); overflow-wrap:anywhere; }
</style>
