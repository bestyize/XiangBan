<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ContentShell from '@/components/ContentShell.vue'
import { useHead } from '@/lib/head'

interface UsageRow { id: string; name: string; tokens: string; rank: number; share_pct: number; rank_change?: number | null; new_entry?: boolean }
interface Usage { source: string; date: string; as_of: string; total_tokens: string; other_tokens: string; previous_date_available: boolean; models: UsageRow[] }
interface Model { id: string; name: string; vendor_id: string; metrics: Record<string, number | null>; pricing: { input: number | null; output: number | null }; performance: { output_tokens_per_second: number | null } }
interface Board { source: string; as_of: string; index_version: string; models: Model[]; vendors: { id: string; name: string }[] }
const route = useRoute()
const router = useRouter()
const tab = computed(() => route.query.board === 'usage' ? 'usage' : 'capability')
const date = computed(() => typeof route.query.date === 'string' ? route.query.date : '')
const board = ref<Board | null>(null)
const usage = ref<Usage | null>(null)
const dates = ref<string[]>([])
const loading = ref(false)
const error = ref('')
const empty = ref(false)
const metric = ref('intelligence')
let sequence = 0
const metrics = [{ id: 'intelligence', label: '智能指数' }, { id: 'speed', label: '输出速度' }, { id: 'input', label: '输入价格' }, { id: 'output', label: '输出价格' }]
function value(m: Model) { return metric.value === 'speed' ? m.performance.output_tokens_per_second : metric.value === 'input' ? m.pricing.input : metric.value === 'output' ? m.pricing.output : m.metrics.intelligence ?? null }
const models = computed(() => [...(board.value?.models ?? [])].sort((a, b) => {
  const x = value(a), y = value(b)
  if (x == null) return y == null ? a.id.localeCompare(b.id) : 1
  if (y == null) return -1
  return (metric.value === 'input' || metric.value === 'output' ? x - y : y - x) || a.id.localeCompare(b.id)
}))
function number(v: number | null | undefined) { return v == null ? '暂无数据' : v.toLocaleString('zh-CN', { maximumFractionDigits: 10 }) }
function tokens(v: string) { return BigInt(v).toLocaleString('zh-CN') }
function time(v: string) { return new Date(v).toLocaleString('zh-CN') }
function setTab(value: string) { void router.replace({ path: '/models', query: { board: value } }) }
function setDate(value: string) { void router.replace({ path: '/models', query: { board: 'usage', ...(value ? { date: value } : {}) } }) }
async function get<T>(path: string): Promise<T> {
  const response = await fetch(path)
  if (response.status === 404) throw new Error('unpublished')
  if (!response.ok) throw new Error(`读取榜单失败（${response.status}），请重试`)
  return response.json()
}
async function load() {
  const mine = ++sequence
  loading.value = true; error.value = ''; empty.value = false
  try {
    if (tab.value === 'usage') {
      const [data, history] = await Promise.all([
        get<Usage>(`/api/discover/model-usage${date.value ? `?date=${encodeURIComponent(date.value)}` : ''}`),
        get<{ items: { date: string }[] }>('/api/discover/model-usage/dates'),
      ])
      if (mine === sequence) { usage.value = data; dates.value = history.items.map(d => d.date) }
    } else {
      const data = await get<Board>('/api/discover/models/v2')
      if (mine === sequence) board.value = data
    }
  } catch (e) {
    if (mine !== sequence) return
    if (e instanceof Error && e.message === 'unpublished') empty.value = true
    else error.value = e instanceof TypeError ? '网络连接失败，请重试' : e instanceof Error ? e.message : '读取失败，请重试'
  } finally { if (mine === sequence) loading.value = false }
}
watch([tab, date], load, { immediate: true })
useHead(() => ({ title: tab.value === 'usage' ? '模型调用量榜' : '模型能力榜' }))
</script>

<template>
  <ContentShell>
    <section class="rankings">
      <h1>模型排行榜</h1>
      <div class="controls" role="tablist" aria-label="排行榜类型">
        <button v-for="item in [{ id: 'capability', label: '能力榜' }, { id: 'usage', label: '调用量榜' }]" :key="item.id" role="tab" :aria-selected="tab === item.id" :class="{ selected: tab === item.id }" @click="setTab(item.id)">{{ item.label }}</button>
        <button :disabled="loading" @click="load">刷新</button>
      </div>
      <p v-if="loading" role="status">加载中…</p>
      <p v-else-if="error" role="alert">{{ error }} <button @click="load">重新加载</button></p>
      <p v-else-if="empty">{{ date ? '这一天尚无已发布榜单。' : '尚无已发布榜单，编辑审核后会在这里展示。' }} <button v-if="date" @click="setDate('')">查看最新</button></p>
      <template v-else-if="tab === 'usage' && usage">
        <div class="controls"><label>统计日（UTC） <select :value="date || usage.date" @change="setDate(($event.target as HTMLSelectElement).value)"><option v-for="day in dates" :key="day" :value="day">{{ day }}</option></select></label></div>
        <p>按输入与输出 Token 总量排名，仅统计 OpenRouter 平台流量。</p>
        <p class="note">全平台当日 {{ tokens(usage.total_tokens) }} Token · 其他模型 {{ tokens(usage.other_tokens) }} Token</p>
        <div class="table-scroll"><table><thead><tr><th>排名</th><th>模型</th><th>Token 用量</th><th>占比</th><th>日排名变化</th></tr></thead><tbody>
          <tr v-for="m in usage.models" :key="m.id"><td>{{ m.rank }}</td><td>{{ m.name }}</td><td class="numeric">{{ tokens(m.tokens) }}</td><td>{{ m.share_pct > 0 && m.share_pct < 0.01 ? '&lt;0.01' : m.share_pct.toFixed(2) }}%</td><td>{{ !usage.previous_date_available ? '暂无对比' : m.new_entry ? '新上榜' : m.rank_change == null ? '暂无对比' : m.rank_change > 0 ? `↑${m.rank_change}` : m.rank_change < 0 ? `↓${-m.rank_change}` : '持平' }}</td></tr>
        </tbody></table></div>
        <p class="note">Source: <a href="https://openrouter.ai/rankings" target="_blank" rel="noopener noreferrer">OpenRouter</a>, as of {{ usage.as_of }} · 本地时间 {{ time(usage.as_of) }}</p>
      </template>
      <template v-else-if="board">
        <div class="controls"><label>排序指标 <select v-model="metric"><option v-for="m in metrics" :key="m.id" :value="m.id">{{ m.label }}</option></select></label></div>
        <p class="note">智能指数版本 {{ board.index_version }} · 价格单位：美元/百万 Token · 速度单位：Token/秒。缺失指标不参与排名。</p>
        <div class="table-scroll"><table><thead><tr><th>排名</th><th>模型</th><th>厂商</th><th>智能指数</th><th>输入价格</th><th>输出价格</th><th>速度</th></tr></thead><tbody>
          <tr v-for="(m, index) in models" :key="m.id"><td>{{ value(m) == null ? '—' : index + 1 }}</td><td>{{ m.name }}</td><td>{{ board.vendors.find(v => v.id === m.vendor_id)?.name ?? m.vendor_id }}</td><td>{{ number(m.metrics.intelligence) }}</td><td>{{ number(m.pricing.input) }}</td><td>{{ number(m.pricing.output) }}</td><td>{{ number(m.performance.output_tokens_per_second) }}</td></tr>
        </tbody></table></div>
        <p class="note">数据来源 <a href="https://artificialanalysis.ai/" target="_blank" rel="noopener noreferrer">Artificial Analysis</a> · 采集于 {{ time(board.as_of) }}</p>
      </template>
    </section>
  </ContentShell>
</template>

<style scoped>
.rankings { padding: 20px 16px; } h1 { font-size: var(--fs-xl); margin: 0 0 16px; }
.controls { display: flex; flex-wrap: wrap; gap: 10px; align-items: center; margin: 12px 0; }
button, select { border: 1px solid var(--line, #ddd); border-radius: 8px; padding: 8px 12px; background: var(--surface, #fff); color: inherit; font: inherit; cursor: pointer; }
button.selected { color: var(--brand, #e3203c); border-color: currentColor; } button:disabled { opacity: .5; cursor: default; }
.note { color: var(--ink-3); font-size: 13px; line-height: 1.7; } .table-scroll { overflow-x: auto; }
table { width: 100%; border-collapse: collapse; font-size: 13px; } th, td { text-align: left; border-bottom: 1px solid var(--line, #eee); padding: 12px 8px; } th { white-space: nowrap; } .numeric { font-variant-numeric: tabular-nums; white-space: nowrap; }
</style>
