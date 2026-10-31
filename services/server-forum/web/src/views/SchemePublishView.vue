<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import ContentShell from '@/components/ContentShell.vue'
import { contentApi, type MyPack } from '@/api/content'
import { schemesApi } from '@/api/schemes'
import { ApiError } from '@/api/forum'
import { kindLabel } from '@/lib/content'
import { useHead } from '@/lib/head'
useHead(() => ({ title: '发布方案' }))
const router = useRouter()
const title = ref(''), summary = ref(''), query = ref('')
const packs = ref<MyPack[]>([])
const selected = ref<{ pack_id: number; version: number; title: string; kind: string }[]>([])
const loading = ref(false), busy = ref(false), loadError = ref(''), error = ref(''), uncertain = ref(false)
const filtered = computed(() => { const term = query.value.trim().toLowerCase(); return packs.value.filter(pack => !term || `${pack.title} ${pack.slug} ${kindLabel(pack.kind)}`.toLowerCase().includes(term)) })
async function load() {
  loading.value = true; loadError.value = ''
  try { packs.value = (await contentApi.myPacks()).items.filter(pack => pack.status === 'published') }
  catch (e) { loadError.value = e instanceof Error ? e.message : '读取自己的作品失败' }
  finally { loading.value = false }
}
function add(pack: MyPack) {
  if (selected.value.length >= 20 || selected.value.some(item => item.pack_id === pack.id)) return
  selected.value.push({ pack_id: pack.id, version: pack.version, title: pack.title, kind: pack.kind })
}
function move(index: number, step: number) {
  const target = index + step
  if (target < 0 || target >= selected.value.length) return
  selected.value.splice(target, 0, selected.value.splice(index, 1)[0]!)
}
async function publish() {
  if (busy.value) return
  error.value = ''; uncertain.value = false
  if (!title.value.trim()) { error.value = '请填写方案标题'; return }
  if (!selected.value.length || selected.value.length > 20) { error.value = '请选择 1–20 件自己的已上架作品'; return }
  busy.value = true
  let posting = false
  try {
    const latest = (await contentApi.myPacks()).items
    const available = new Set(latest.filter(pack => pack.status === 'published').map(pack => pack.id))
    if (selected.value.some(item => !available.has(item.pack_id))) throw new Error('选中的作品已下架或不再属于当前账号，请刷新作品列表并移除不可用项')
    const items = selected.value.map(item => ({ pack_id: item.pack_id, version: item.version }))
    posting = true
    const result = await schemesApi.publish({ title: title.value.trim(), summary: summary.value.trim(), items })
    await router.replace(`/schemes/${result.id}`)
  } catch (e) {
    uncertain.value = posting && e instanceof ApiError && e.status === 0
    error.value = uncertain.value ? '连接已中断，发布结果尚未确认。请先到“我的方案”核对，再决定是否重试。' : e instanceof Error ? e.message : '方案发布失败，请稍后重试'
  } finally { busy.value = false }
}
onMounted(load)
</script>
<template>
  <ContentShell>
    <form class="scheme-publish" @submit.prevent="publish">
      <div class="publish-heading"><h1>发布方案</h1><RouterLink to="/schemes">查看社区方案</RouterLink></div>
      <p class="hint">选择 1–20 件你已经上架的作品组成方案。方案只保存所选作品与版本，原作品仍由各自的详情页提供。</p>
      <label class="field"><span>方案标题</span><input v-model="title" class="input" required maxlength="60" :disabled="busy"></label>
      <label class="field"><span>简介（选填）</span><textarea v-model="summary" class="input" maxlength="500" rows="4" :disabled="busy" /></label>
      <h2>已选择 {{ selected.length }} / 20</h2>
      <p v-if="!selected.length" class="hint">从下方作品列表添加，顺序将保留在方案中。</p>
      <ol class="selected-list"><li v-for="(item, index) in selected" :key="item.pack_id"><span>{{ item.title }} · {{ kindLabel(item.kind) }} · v{{ item.version }}</span><span class="selected-actions"><button class="btn btn-sm" type="button" :disabled="busy || index === 0" @click="move(index, -1)">上移</button><button class="btn btn-sm" type="button" :disabled="busy || index === selected.length - 1" @click="move(index, 1)">下移</button><button class="btn btn-sm" type="button" :disabled="busy" @click="selected.splice(index, 1)">移除</button></span></li></ol>
      <section class="picker"><div class="publish-heading"><h2>自己的已上架作品</h2><button class="btn btn-sm" type="button" :disabled="loading || busy" @click="load">刷新作品</button></div><input v-model="query" class="input" placeholder="筛选标题、标识或类型" aria-label="筛选自己的作品" :disabled="busy">
        <p v-if="loading" class="hint" role="status">正在读取作品…</p><p v-else-if="loadError" role="alert">{{ loadError }} <button class="btn" type="button" @click="load">重试</button></p><p v-else-if="!packs.length" class="hint">还没有已上架作品。<RouterLink to="/publish">先发布内容包</RouterLink></p><p v-else-if="!filtered.length" class="hint">没有匹配的作品，请调整筛选词。</p>
        <ul v-else class="candidate-list"><li v-for="pack in filtered" :key="pack.id"><span><RouterLink :to="`/packs/${pack.id}`" target="_blank">{{ pack.title }}</RouterLink> · {{ kindLabel(pack.kind) }} · v{{ pack.version }}</span><button class="btn btn-sm" type="button" :disabled="busy || selected.length >= 20 || selected.some(item => item.pack_id === pack.id)" @click="add(pack)">{{ selected.some(item => item.pack_id === pack.id) ? '已选择' : '加入方案' }}</button></li></ul>
      </section>
      <p v-if="error" role="alert">{{ error }} <RouterLink v-if="uncertain" to="/me/works?tab=schemes">查看我的方案</RouterLink></p>
      <button class="btn btn-primary" :disabled="busy || loading || !!loadError || selected.length === 0">{{ busy ? '正在核对并发布…' : '发布方案' }}</button>
    </form>
  </ContentShell>
</template>
<style scoped>
.scheme-publish { padding: 20px 16px; display: grid; gap: 16px; }
.publish-heading { display: flex; justify-content: space-between; gap: 12px; align-items: center; flex-wrap: wrap; }
h1, h2 { margin: 0; } h1 { font-size: var(--fs-xl); } h2 { font-size: var(--fs-lg); }
.hint { margin: 0; color: var(--ink-3); font-size: var(--fs-sm); line-height: 1.6; }
.selected-list, .candidate-list { margin: 0; padding: 0; list-style: none; }
.selected-list li, .candidate-list li { display: flex; justify-content: space-between; align-items: center; gap: 12px; flex-wrap: wrap; padding: 10px 0; border-bottom: 1px solid var(--line); }
.selected-actions { display: flex; gap: 6px; }
.picker { display: grid; gap: 12px; padding: 16px; border: 1px solid var(--line); border-radius: 8px; }
</style>
