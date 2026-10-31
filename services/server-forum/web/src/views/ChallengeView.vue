<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ContentShell from '@/components/ContentShell.vue'
import PagerNav from '@/components/PagerNav.vue'
import PackCard from '@/components/PackCard.vue'
import { challengesApi, type CommunityChallengeDetail } from '@/api/challenges'
import { contentApi, type MyPack } from '@/api/content'
import { ApiError } from '@/api/forum'
import { useSession } from '@/stores/session'
import { kindLabel } from '@/lib/content'
import { useHead } from '@/lib/head'
const route = useRoute(), router = useRouter(), session = useSession()
const labels = { upcoming: '即将开始', open: '投稿中', ended: '已截止', closed: '已关闭' }
const detail = ref<CommunityChallengeDetail | null>(null), loading = ref(false), error = ref(''), missing = ref(false)
const ownPacks = ref<MyPack[]>([]), ownLoading = ref(false), ownError = ref('')
const selectedId = ref(''), submitting = ref(false), submissionError = ref(''), success = ref('')
const selected = computed(() => ownPacks.value.find(pack => pack.id === Number(selectedId.value)))
const currentKey = computed(() => String(route.params.key || ''))
const timezone = Intl.DateTimeFormat().resolvedOptions().timeZone
useHead(() => ({ title: detail.value?.challenge.title ?? '挑战详情' }))
function date(value: string) { return new Date(value).toLocaleString('zh-CN') }
let sequence = 0
let ownSequence = 0
async function load() {
  const current = ++sequence; loading.value = true; error.value = ''; missing.value = false
  const requested = Number(route.query.page || 1)
  try {
    const data = await challengesApi.detail(currentKey.value, Number.isSafeInteger(requested) && requested > 0 ? requested : 1)
    if (current === sequence) detail.value = data
  } catch (e) { if (current === sequence) { detail.value = null; if (e instanceof ApiError && e.status === 404) missing.value = true; else error.value = e instanceof Error ? e.message : '挑战加载失败' } }
  finally { if (current === sequence) loading.value = false }
}
async function loadOwn() {
  const current = ++ownSequence
  if (!session.loggedIn) { ownPacks.value = []; return }
  ownLoading.value = true; ownError.value = ''
  try { const items = (await contentApi.myPacks()).items; if (current === ownSequence) ownPacks.value = items.filter(pack => pack.status === 'published') }
  catch (e) { if (current === ownSequence) ownError.value = e instanceof Error ? e.message : '自己的作品读取失败' }
  finally { if (current === ownSequence) ownLoading.value = false }
}
async function submit() {
  if (submitting.value || !selected.value) return
  const key = currentKey.value, packId = selected.value.id
  submitting.value = true; submissionError.value = ''; success.value = ''
  try {
    await challengesApi.submit(key, packId)
    if (currentKey.value !== key) return
    success.value = '作品已提交到本次挑战。'
    selectedId.value = ''
    if (route.query.page) await router.replace({ query: {} }); else await load()
  } catch (e) {
    if (currentKey.value !== key) return
    submissionError.value = e instanceof Error ? e.message : '投稿失败，请稍后重试'
    if (e instanceof ApiError && e.status === 409) await load()
  } finally { submitting.value = false }
}
watch([currentKey, () => route.query.page], (_, old) => { if (!old || currentKey.value !== old[0]) { detail.value = null; selectedId.value = ''; success.value = ''; submissionError.value = '' }; load() }, { immediate: true })
watch(() => session.loggedIn, value => { if (value) loadOwn(); else { ownSequence++; ownPacks.value = []; selectedId.value = '' } }, { immediate: true })
</script>
<template>
  <ContentShell><nav class="challenge-nav"><RouterLink to="/challenges">全部挑战</RouterLink><button class="btn btn-sm" :disabled="loading || submitting" @click="load">刷新</button></nav>
    <p v-if="loading && !detail" class="state" role="status">正在读取挑战…</p><p v-else-if="missing" class="state">挑战不存在或尚未公开。<RouterLink to="/challenges">查看其他挑战</RouterLink></p><p v-else-if="error && !detail" class="state" role="alert">{{ error }} <button class="btn" @click="load">重试</button></p>
    <template v-if="detail"><header class="challenge-header"><span class="phase">{{ labels[detail.challenge.status] }}</span><h1>{{ detail.challenge.title }}</h1><p class="meta">{{ date(detail.challenge.starts_at) }} — {{ date(detail.challenge.ends_at) }}（{{ timezone }}）</p><p class="description">{{ detail.challenge.description }}</p><p class="meta">#{{ detail.challenge.submission_tag }} · {{ detail.challenge.entry_count }} 件公开投稿</p><RouterLink v-if="detail.challenge.topic_id" class="btn" :to="`/t/${detail.challenge.topic_id}`">参与主题讨论</RouterLink></header>
      <section class="submission"><h2>提交自己的作品</h2><p v-if="detail.challenge.status !== 'open'" class="meta">{{ detail.challenge.status === 'upcoming' ? '挑战尚未开始，到开始时间后刷新页面即可投稿。' : '这个挑战已经停止接收投稿，已收录的作品仍可浏览。' }}</p><template v-else><p class="meta">只接受你已上架且公开可读的作品。同一作品在一个挑战中只收录一次。</p><button v-if="!session.loggedIn" class="btn btn-primary" @click="session.loginOpen = true">登录后选择自己的作品</button><template v-else><p v-if="ownLoading" role="status">正在读取自己的作品…</p><p v-else-if="ownError" role="alert">{{ ownError }} <button class="btn" :disabled="ownLoading" @click="loadOwn">重试</button></p><p v-else-if="!ownPacks.length" class="meta">还没有已上架作品。<RouterLink to="/publish">先发布一件作品</RouterLink></p><form v-else class="submission-form" @submit.prevent="submit"><label class="field"><span>选择自己的已上架作品</span><select v-model="selectedId" class="select" :disabled="submitting"><option value="" disabled>请选择作品</option><option v-for="pack in ownPacks" :key="pack.id" :value="String(pack.id)">{{ pack.title }} · {{ kindLabel(pack.kind) }} · v{{ pack.version }}</option></select></label><div class="submission-actions"><button class="btn btn-primary" :disabled="submitting || !selected">{{ submitting ? '正在提交…' : '提交到挑战' }}</button><RouterLink v-if="selected" :to="`/packs/${selected.id}`" target="_blank">查看所选作品</RouterLink><button class="btn" type="button" :disabled="submitting || ownLoading" @click="loadOwn">刷新作品列表</button></div></form></template></template><p v-if="submissionError" role="alert">{{ submissionError }}</p><p v-if="success" role="status">{{ success }}</p></section>
      <section class="entries"><h2>参与作品</h2><p class="meta">支持作品可以到它的讨论帖点赞。列表由投稿时间和编辑精选排序，点赞不发挑战积分。</p><p v-if="loading" role="status">正在刷新作品…</p><p v-if="!detail.entries.items.length" class="meta">还没有公开可见的投稿作品。</p><div v-else class="entries-grid"><article v-for="entry in detail.entries.items" :key="entry.pack_id" class="entry"><span v-if="entry.featured" class="editor-choice">编辑精选</span><PackCard :pack="entry.pack" /><RouterLink v-if="entry.vote_topic_id" class="btn btn-sm" :to="`/t/${entry.vote_topic_id}`">到讨论点赞{{ entry.likes === null ? '' : ` · ${entry.likes}` }}</RouterLink><span v-else class="meta">未关联作品讨论</span></article></div><PagerNav :page="detail.entries.page" :page-size="detail.entries.page_size" :total="detail.entries.total" @go="page => router.push({ query: { page } })" /></section>
    </template>
  </ContentShell>
</template>
<style scoped>
.challenge-nav { display: flex; justify-content: space-between; align-items: center; padding: 14px 20px; border-bottom: 1px solid var(--line); }.challenge-header, .submission, .entries { padding: 24px 20px; border-bottom: 1px solid var(--line); }
h1 { margin: 14px 0; font-size: var(--fs-xl); } h2 { margin: 0 0 14px; font-size: var(--fs-lg); }.phase, .editor-choice { display: inline-block; color: var(--brand); border: 1px solid currentColor; padding: 3px 10px; border-radius: 20px; font-size: var(--fs-xs); }.meta { color: var(--ink-3); font-size: var(--fs-sm); line-height: 1.6; }.description { white-space: pre-wrap; line-height: 1.8; color: var(--ink-2); }.submission-form { display: grid; gap: 12px; max-width: 600px; }.submission-actions { display: flex; flex-wrap: wrap; align-items: center; gap: 12px; }.entries-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 20px; }.entry { min-width: 0; display: grid; align-content: start; gap: 12px; }.state { padding: 40px 16px; color: var(--ink-3); text-align: center; }
@media (min-width: 768px) { .entries-grid { grid-template-columns: repeat(3, minmax(0, 1fr)); } }
@media (max-width: 420px) { .entries-grid { grid-template-columns: 1fr; } }
</style>
