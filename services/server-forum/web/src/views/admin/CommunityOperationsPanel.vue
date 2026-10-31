<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { AdminApiError } from '@/api/admin'
import { operationsApi, type LinkHealthPage, type OperationsSummary } from '@/api/communityOperations'
import { errorText, fmtTime, notify } from './shared'
const tab = ref<'overview' | 'links'>('overview')
const summary = ref<OperationsSummary | null>(null)
const links = ref<LinkHealthPage | null>(null)
const summaryError = ref('')
const linksError = ref('')
const actionError = ref('')
const summaryLoading = ref(false)
const linksLoading = ref(false)
const busy = ref('')
const page = ref(1)
const status = ref('')
const enabled = ref(false)
const hours = ref(24)
const settingsDirty = ref(false)
const pollNotice = ref('')
let disposed = false
let summarySeq = 0
let linksSeq = 0
let timer: ReturnType<typeof setTimeout> | undefined
let pollRun = ''
const KIND: Record<string, string> = { banner: '首页横幅', vendor: '服务商', model: '模型目录', quota: '免费额度', launch: '模型上新', hotword: '热门话题', nav: 'AI 导航', doc: '学习文档', tool: 'AI 工具', news: 'AI 日报', repo_issue: '开源热榜', model_board: '模型能力榜', model_usage: '模型调用量榜', promo: '限免活动' }
const HEALTH: Record<string, string> = { unchecked: '未检查', healthy: '可访问', restricted: '访问受限', broken: '不存在或已移除', rate_limited: '来源限流', unavailable: '来源异常', blocked: '安全策略阻止', dns_error: '解析异常', network_error: '连接异常', timeout: '检查超时', attention: '需人工核对' }
const FRESH: Record<string, string> = { fresh: '更新正常', stale: '更新过期', never: '尚未成功', disabled: '已停用' }
const RUN: Record<string, string> = { running: '检查中', succeeded: '检查完成', partial: '部分完成', failed: '运行中断' }
const tone = (state: string) => ['healthy', 'fresh', 'succeeded'].includes(state) ? 'valid' : ['unchecked', 'disabled'].includes(state) ? 'line' : ['broken', 'failed'].includes(state) ? 'danger' : 'suspect'
const active = computed(() => !!links.value?.settings.active_run && Date.parse(links.value.settings.lease_until || '') > Date.now())
const totals = computed(() => ({ published: summary.value?.columns.reduce((n, c) => n + c.published, 0) || 0, pending: summary.value?.columns.reduce((n, c) => n + c.pending, 0) || 0, stale: summary.value?.sources.filter(s => s.freshness === 'stale' || s.freshness === 'never').length || 0, attention: Object.entries(summary.value?.links.statuses || {}).reduce((n, [k, v]) => n + (['healthy', 'unchecked'].includes(k) ? 0 : v), 0) }))
const showError = (e: unknown) => !(e instanceof AdminApiError && e.status === 401)
async function loadSummary() {
  const mine = ++summarySeq
  summaryLoading.value = true
  summaryError.value = ''
  try { const data = await operationsApi.summary(); if (!disposed && mine === summarySeq) summary.value = data }
  catch (e) { if (!disposed && mine === summarySeq && showError(e)) summaryError.value = errorText(e) }
  finally { if (!disposed && mine === summarySeq) summaryLoading.value = false }
}
async function loadLinks(): Promise<boolean> {
  const mine = ++linksSeq
  linksLoading.value = true
  linksError.value = ''
  try {
    const data = await operationsApi.links(page.value, status.value)
    if (disposed || mine !== linksSeq) return false
    links.value = data
    if (!settingsDirty.value) { enabled.value = data.settings.enabled; hours.value = data.settings.interval_hours }
    return true
  } catch (e) { if (!disposed && mine === linksSeq && showError(e)) linksError.value = errorText(e); return false }
  finally { if (!disposed && mine === linksSeq) linksLoading.value = false }
}
function stopPoll() { if (timer) clearTimeout(timer); timer = undefined; pollRun = '' }
function poll(id: string, remaining: number) {
  timer = setTimeout(async () => {
    if (disposed || pollRun !== id) return
    const ok = await loadLinks()
    if (disposed || pollRun !== id) return
    const run = links.value?.runs.find(r => r.id === id)
    if (!ok) { stopPoll(); pollNotice.value = '自动刷新已停止，请手动刷新检查结果。'; return }
    if (run && run.status !== 'running') { stopPoll(); pollNotice.value = ''; void loadSummary(); return }
    if (remaining <= 1) { stopPoll(); pollNotice.value = '本轮短暂刷新已结束，任务可继续运行；请稍后手动刷新。'; return }
    poll(id, remaining - 1)
  }, 5000)
}
async function run() {
  if (busy.value || active.value) return
  busy.value = 'run'; actionError.value = ''; stopPoll(); pollNotice.value = ''
  try {
    const data = await operationsApi.run()
    if (disposed) return
    notify(`已启动检查，本轮选择 ${data.selected_count} / ${data.total_candidates} 个链接`)
    await loadLinks()
    if (disposed) return
    pollRun = data.run_id; pollNotice.value = '正在短暂刷新运行状态，最多十次。'; poll(data.run_id, 10)
  } catch (e) { if (!disposed && showError(e)) { actionError.value = errorText(e); await loadLinks() } }
  finally { if (!disposed) busy.value = '' }
}
async function save() {
  if (busy.value) return
  if (!Number.isInteger(hours.value) || hours.value < 24 || hours.value > 720) { actionError.value = '自动检查周期须为 24–720 小时。'; return }
  busy.value = 'settings'; actionError.value = ''
  try {
    await operationsApi.settings(enabled.value, hours.value)
    if (disposed) return
    settingsDirty.value = false; notify('外链检查计划已保存'); await loadLinks()
  } catch (e) { if (!disposed && showError(e)) actionError.value = errorText(e) }
  finally { if (!disposed) busy.value = '' }
}
function search() { page.value = 1; void loadLinks() }
function goto(p: number) { page.value = p; void loadLinks() }
onMounted(() => { void loadSummary(); void loadLinks() })
onUnmounted(() => { disposed = true; ++summarySeq; ++linksSeq; stopPoll() })
</script>
<template>
  <div class="adm-toolbar">
    <div class="adm-seg" aria-label="运营视图"><button :class="{ 'is-on': tab === 'overview' }" @click="tab = 'overview'">运营概览</button><button :class="{ 'is-on': tab === 'links' }" @click="tab = 'links'">外链检查</button></div>
    <RouterLink class="btn" to="/admin/collection">来源配置与采集</RouterLink><RouterLink class="btn" to="/admin/discover">栏目编辑与审核</RouterLink>
  </div>
  <template v-if="tab === 'overview'">
    <div class="adm-toolbar"><p class="muted">按已发布内容、草稿及采集记录汇总 · {{ fmtTime(summary?.as_of) }}</p><button class="btn" :disabled="summaryLoading" @click="loadSummary">{{ summaryLoading ? '刷新中…' : '刷新汇总' }}</button></div>
    <div v-if="summaryError" class="adm-state error-text" role="alert">{{ summaryError }} <button class="btn" :disabled="summaryLoading" @click="loadSummary">重试</button></div>
    <p v-if="summaryLoading && !summary" class="muted" role="status">正在读取运营数据…</p>
    <template v-if="summary">
      <div class="panel adm-card"><div class="adm-stats"><div class="adm-stat"><b>{{ totals.published }}</b><span>当前已发布条目</span></div><div class="adm-stat" :class="{ 'is-alert': totals.pending > 0 }"><b>{{ totals.pending }}</b><span>待审草稿</span></div><div class="adm-stat" :class="{ 'is-alert': totals.stale > 0 }"><b>{{ totals.stale }}</b><span>启用来源待更新</span></div><div class="adm-stat" :class="{ 'is-alert': totals.attention > 0 }"><b>{{ totals.attention }}</b><span>窗口内待核对链接</span></div></div></div>
      <h2 class="adm-section-title">采集来源</h2><p class="muted">{{ summary.freshness_rule }} 运行失败与更新过期分别显示。</p>
      <div class="panel adm-card"><div class="ops-scroll"><table class="adm-table"><thead><tr><th>来源</th><th>最近成功 / 下一次</th><th>近 24 小时</th><th>待审</th></tr></thead><tbody><tr v-for="source in summary.sources" :key="source.id"><td data-label="来源"><div class="adm-title">{{ source.title }}</div><div class="adm-sub"><span class="adm-tag" :class="tone(source.freshness)">{{ FRESH[source.freshness] }}</span><span v-if="source.active" class="adm-tag line">采集中</span><span>{{ source.interval_minutes }} 分钟</span></div><p v-if="source.last_error" class="error-text ops-message">{{ source.last_error }}</p></td><td data-label="最近成功 / 下一次"><div>{{ fmtTime(source.last_success_at) }}</div><div class="adm-sub">{{ source.enabled ? fmtTime(source.next_run_at) : '自动采集已停用' }}</div></td><td data-label="近 24 小时"><div>成功 {{ source.succeeded_24h }}</div><div class="adm-sub">失败 {{ source.failed_24h }}</div></td><td data-label="待审">{{ source.pending }}</td></tr><tr v-if="!summary.sources.length"><td colspan="4" class="adm-empty">尚无采集来源。</td></tr></tbody></table></div></div>
      <h2 class="adm-section-title">栏目内容</h2><p class="muted">“待审”含已发布条目的编辑副本；未来发布日期与下架条目不计入当前已发布数。样例数只统计显式标记的线上样例。</p>
      <div class="panel adm-card"><div class="ops-scroll"><table class="adm-table"><thead><tr><th>栏目</th><th>已发布</th><th>待审 / 定时</th><th>未来 / 下架</th><th>样例 / 最后更新</th></tr></thead><tbody><tr v-for="column in summary.columns" :key="column.kind"><td data-label="栏目" class="adm-title">{{ KIND[column.kind] || column.kind }}</td><td data-label="已发布">{{ column.published }}</td><td data-label="待审 / 定时">{{ column.pending }} / {{ column.scheduled }}</td><td data-label="未来 / 下架">{{ column.future }} / {{ column.unlisted }}</td><td data-label="样例 / 最后更新">{{ column.sample }}<div class="adm-sub">{{ fmtTime(column.last_live_update) }}</div></td></tr><tr v-if="!summary.columns.length"><td colspan="5" class="adm-empty">尚无栏目内容。</td></tr></tbody></table></div></div>
      <h2 class="adm-section-title">外链状态</h2><div class="panel adm-card ops-note"><p>当前候选窗口 {{ summary.links.total }} 个去重链接。</p><div class="adm-sub"><span v-for="(count, state) in summary.links.statuses" :key="state" class="adm-tag" :class="tone(String(state))">{{ HEALTH[state] || state }} {{ count }}</span></div><p class="muted">{{ summary.candidate_scope }}</p><button class="btn" @click="tab = 'links'">查看并运行外链检查</button></div>
    </template>
  </template>
  <template v-else>
    <p class="muted">检查已维护的导航、文档、工具主页及来源链接。结果仅供人工核对，不会修改、下架或发布内容；访问受限不代表链接失效。</p>
    <div v-if="linksError" class="adm-state error-text" role="alert">{{ linksError }} <button class="btn" :disabled="linksLoading" @click="loadLinks">重试</button></div>
    <p v-if="linksLoading && !links" class="muted" role="status">正在读取外链状态…</p>
    <template v-if="links">
      <section class="panel adm-card ops-note"><h2 class="adm-section-title ops-heading">检查计划</h2><form @submit.prevent="save"><div class="adm-form-grid"><label class="field"><span>自动检查</span><select class="select" v-model="enabled" :disabled="!!busy" @change="settingsDirty = true"><option :value="false">停用（默认）</option><option :value="true">启用低频检查</option></select></label><label class="field"><span>周期（24–720 小时）</span><input class="input" v-model.number="hours" type="number" min="24" max="720" step="1" required :disabled="!!busy" @input="settingsDirty = true"></label></div><div class="adm-toolbar"><button class="btn" type="submit" :disabled="!!busy || !settingsDirty">{{ busy === 'settings' ? '保存中…' : '保存计划' }}</button><span class="muted">{{ links.settings.enabled ? `下次检查：${fmtTime(links.settings.next_run_at)}` : '当前仅手动检查' }}</span></div></form><div class="adm-toolbar"><button class="btn btn-primary" :disabled="!!busy || active || linksLoading" @click="run">{{ busy === 'run' ? '启动中…' : active ? '检查正在运行' : '手动检查一批' }}</button><button class="btn" :disabled="linksLoading || !!busy" @click="loadLinks">{{ linksLoading ? '刷新中…' : '刷新结果' }}</button></div><p class="muted">每轮最多 {{ links.max_links_per_run }} 个链接，优先未检查或最早检查的链接；并发 4，最多 {{ links.max_run_seconds }} 秒。每个链接只读取响应状态，最多两次跳转。</p><p class="muted">{{ links.candidate_scope }}</p><p v-if="actionError" class="error-text" role="alert">{{ actionError }}</p><p v-if="pollNotice" class="muted" role="status">{{ pollNotice }}</p></section>
      <h2 class="adm-section-title">检查结果</h2><div class="adm-toolbar"><label class="field ops-filter"><span>状态筛选</span><select class="select" v-model="status" :disabled="linksLoading" @change="search"><option value="">全部状态</option><option v-for="(label, value) in HEALTH" :key="value" :value="value">{{ label }}</option></select></label><span class="muted">{{ links.total }} 个链接 · 第 {{ page }} 页</span></div>
      <div class="panel adm-card"><div class="ops-scroll"><table class="adm-table ops-links"><thead><tr><th>链接及维护位置</th><th>检查结果</th><th>最近检查 / 成功</th></tr></thead><tbody><tr v-for="item in links.items" :key="item.id"><td data-label="链接及维护位置"><div class="adm-title">{{ item.host }}</div><div class="adm-mono ops-url">{{ item.url }}</div><details class="ops-details"><summary>维护位置（{{ item.refs.length }}）</summary><div v-for="(reference, index) in item.refs" :key="index" class="adm-sub">{{ KIND[reference.kind] || reference.kind }} · {{ reference.key }} · {{ reference.field }} · {{ reference.scope === 'published' ? '线上条目' : '编辑来源' }}</div></details></td><td data-label="检查结果"><span class="adm-tag" :class="tone(item.status)">{{ HEALTH[item.status] || item.status }}</span><div class="adm-sub"><span v-if="item.http_status">HTTP {{ item.http_status }}</span><span v-if="item.duration_ms !== null">{{ item.duration_ms }} ms</span></div><p v-if="item.error" class="ops-message">{{ item.error }}</p><div v-if="item.failures" class="adm-sub">连续未通过 {{ item.failures }} 次</div></td><td data-label="最近检查 / 成功">{{ fmtTime(item.last_checked_at) }}<div class="adm-sub">成功 {{ fmtTime(item.last_success_at) }}</div></td></tr><tr v-if="!links.items.length"><td colspan="3" class="adm-empty">当前筛选下暂无链接。</td></tr></tbody></table></div></div>
      <div class="adm-toolbar ops-pagination"><button class="btn" :disabled="page <= 1 || linksLoading" @click="goto(page - 1)">上一页</button><button class="btn" :disabled="page * links.page_size >= links.total || linksLoading" @click="goto(page + 1)">下一页</button></div>
      <h2 class="adm-section-title">最近十次运行</h2><div class="panel adm-card"><div class="ops-scroll"><table class="adm-table"><thead><tr><th>开始 / 结束</th><th>运行状态</th><th>结果</th></tr></thead><tbody><tr v-for="record in links.runs" :key="record.id"><td data-label="开始 / 结束">{{ fmtTime(record.started_at) }}<div class="adm-sub">{{ fmtTime(record.finished_at) }}</div></td><td data-label="运行状态"><span class="adm-tag" :class="tone(record.status)">{{ RUN[record.status] || record.status }}</span><p v-if="record.error" class="ops-message">{{ record.error }}</p></td><td data-label="结果"><div>已检查 {{ record.checked_count }} / 本轮 {{ record.selected_count }}</div><div class="adm-sub">可访问 {{ record.healthy_count }} · 待核对 {{ record.attention_count }} · 策略阻止 {{ record.blocked_count }}</div></td></tr><tr v-if="!links.runs.length"><td colspan="3" class="adm-empty">尚未执行外链检查。</td></tr></tbody></table></div></div>
    </template>
  </template>
</template>
<style scoped>
.ops-scroll{overflow-x:auto;max-width:100%}.ops-scroll .adm-table{min-width:610px}.ops-links{table-layout:fixed}.ops-links th:first-child{width:44%}.ops-links th:nth-child(2){width:30%}.ops-note{padding:22px}.ops-note p{font-size:13px;line-height:1.7;overflow-wrap:anywhere}.ops-heading{margin-top:0}.ops-message{font-size:12px;line-height:1.6;margin:6px 0 0;overflow-wrap:anywhere;max-width:420px}.ops-url{font-size:12px;margin-top:5px}.ops-details{font-size:12px;margin-top:8px;color:var(--ink-2)}.ops-details summary{cursor:pointer}.ops-details .adm-sub{margin-top:6px}.ops-filter{margin:0;min-width:170px}.ops-pagination{justify-content:flex-end;margin-top:14px}.ops-note .adm-form-grid{max-width:640px}.ops-note form{margin-bottom:20px}.ops-note form .adm-toolbar{margin-top:16px}.ops-note>.adm-toolbar{margin-top:16px}.ops-note .adm-toolbar:last-child{margin-bottom:0}@media(max-width:1023px){.ops-scroll .adm-table{min-width:0}.ops-scroll .adm-table td{display:block}.ops-scroll .adm-table td+td{margin-top:12px}.ops-scroll .adm-table td[data-label]::before{display:block;margin-bottom:5px}}@media(max-width:700px){.ops-note{padding:18px}}
</style>
