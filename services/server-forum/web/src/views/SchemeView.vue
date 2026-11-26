<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import DetailLayout from '@/components/community/DetailLayout.vue'
import ContentSocialActions from '@/components/ContentSocialActions.vue'
import SchemeReferences from '@/components/SchemeReferences.vue'
import WorkDiscussion from '@/components/creation/WorkDiscussion.vue'
import { schemesApi, type CommunityScheme } from '@/api/schemes'
import { confirmDialog } from '@/ui'
import { ApiError } from '@/api/forum'
import { useSession } from '@/stores/session'
import { useHead } from '@/lib/head'
import { timeAgo } from '@/lib/format'
const route = useRoute(), router = useRouter(), session = useSession()
const scheme = ref<CommunityScheme | null>(null), loading = ref(false), missing = ref(false), error = ref(''), busy = ref(false), copied = ref(false), copyError = ref(false)
const owner = computed(() => scheme.value?.author.uid === session.user?.uid)
const link = computed(() => typeof window === 'undefined' ? '' : `${window.location.origin}/schemes/${scheme.value?.id ?? ''}`)
useHead(() => ({ title: scheme.value?.title ?? '方案详情' }))
let sequence = 0
async function load() {
  const current = ++sequence
  scheme.value = null; loading.value = true; missing.value = false; error.value = ''; copied.value = false; copyError.value = false
  try {
    const id = Number(route.params.id)
    if (!Number.isSafeInteger(id) || id <= 0) { missing.value = true; return }
    const data = await schemesApi.detail(id)
    if (current === sequence) scheme.value = data
  } catch (e) { if (current === sequence) { if (e instanceof ApiError && e.status === 404) missing.value = true; else error.value = e instanceof Error ? e.message : '方案加载失败' } }
  finally { if (current === sequence) loading.value = false }
}
function back() {
  if (window.history.state?.back) router.back()
  else void router.push('/schemes')
}
async function copy() { try { await navigator.clipboard.writeText(link.value); copied.value = true; copyError.value = false } catch { copyError.value = true } }
async function remove() {
  if (!scheme.value || !await confirmDialog({ title: `下架「${scheme.value.title}」？`, description: '下架后公开链接不再可访问，引用的原作品会保留。', confirmLabel: '确认下架', danger: true })) return
  busy.value = true; error.value = ''
  try { await schemesApi.remove(scheme.value.id); await router.replace('/me/works?tab=schemes') }
  catch (e) { error.value = e instanceof Error ? e.message : '下架失败，请稍后重试' }
  finally { busy.value = false }
}
watch(() => route.params.id, load, { immediate: true })
</script>
<template>
  <DetailLayout :ready="!!scheme" :title="scheme?.title" :author="scheme?.author.nickname || '社区作者'"
    :author-to="scheme ? '/u/' + encodeURIComponent(scheme.author.uid) : undefined"
    :share="scheme ? { title: scheme.title, author: scheme.author.nickname, badge: '桌面方案', excerpt: scheme.summary, url: link } : undefined"
    @back="back">
    <template #state>
      <p v-if="loading" class="state" role="status">正在读取方案…</p>
      <p v-else-if="missing" class="state">方案不存在或已下架。<RouterLink to="/schemes">查看其他方案</RouterLink></p>
      <p v-else-if="error" class="state" role="alert">{{ error }} <button class="btn" @click="load">重试</button></p>
    </template>
    <template v-if="scheme" #default>
      <p v-if="scheme.summary" class="xhs-prose summary">{{ scheme.summary }}</p>
      <div class="xhs-hashtags"><RouterLink to="/schemes">#社区方案</RouterLink></div>
      <div class="xhs-meta-row scheme-meta">
        <time :datetime="scheme.created_at">{{ timeAgo(scheme.created_at) }}发布</time>
        <span>{{ scheme.items.length }} 项作品</span><span v-if="scheme.apply_count > 0">{{ scheme.apply_count }} 次复刻</span>
      </div>
      <h2>方案中的作品</h2>
      <SchemeReferences :items="scheme.items" />
      <p class="hint">版本由作者发布方案时选定。当前不可用的作品会保留占位。</p>
      <ContentSocialActions kind="scheme" :id="scheme.id" />
      <div class="scheme-actions">
        <RouterLink class="btn" to="/schemes/new">发布方案</RouterLink>
        <button class="btn" type="button" @click="copy">{{ copied ? '链接已复制' : '复制方案链接' }}</button>
        <button v-if="owner" class="btn" :disabled="busy" @click="remove">{{ busy ? '正在下架…' : '下架自己的方案' }}</button>
      </div>
      <p v-if="error" role="alert">{{ error }} <button class="btn" :disabled="busy" @click="load">重新读取</button></p>
      <label v-if="copyError" class="field"><span>请手动复制方案链接</span><input class="input" readonly :value="link" @focus="($event.target as HTMLInputElement).select()"></label>
    </template>
    <template v-if="scheme" #comments><WorkDiscussion kind="scheme" :id="scheme.id" /></template>
  </DetailLayout>
</template>
<style scoped>
h2 { margin: 24px 0 14px; font-size: var(--fs-lg); }
.scheme-meta { justify-content: flex-start; flex-wrap: wrap; }
.scheme-actions { display: flex; gap: 8px 16px; flex-wrap: wrap; margin: 12px 0; }
.hint { color: var(--ink-3); font-size: var(--fs-sm); }
.summary { margin: 0; white-space: pre-wrap; }
.state { padding: 40px 16px; text-align: center; color: var(--ink-3); }
</style>
