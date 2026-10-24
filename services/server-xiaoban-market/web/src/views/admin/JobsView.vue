<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { admin } from '@/lib/api';
import { formatDuration, formatTime } from '@/lib/format';
import { TRIGGER_LABEL } from '@/lib/labels';
import { toast, toastError } from '@/lib/toast';
import type { Job, JobLog, Source } from '@/lib/types';
import StatusBadge from '@/components/StatusBadge.vue';

const route = useRoute();
const router = useRouter();
const jobs = ref<Job[]>([]);
const total = ref(0);
const sources = ref<Source[]>([]);
const filter = ref({ sourceId: '' as number | '', status: '' });
const page = ref(1);
const selected = ref<Job | null>(null);
const logs = ref<JobLog[]>([]);
const logBox = ref<HTMLElement | null>(null);
const loading = ref(true);
const create = ref({ sourceId: '' as number | '', keyword: '', maxPages: '' });
let timer: ReturnType<typeof setInterval> | undefined;

const selectedId = computed(() => (route.query.job ? String(route.query.job) : ''));
const active = (j: Job | null) => j?.status === 'running' || j?.status === 'queued';

async function loadList() {
  try {
    const r = await admin.jobs({ sourceId: filter.value.sourceId || undefined, status: filter.value.status || undefined, page: page.value });
    jobs.value = r.results;
    total.value = r.total;
  } catch (e) {
    toastError(e);
  } finally {
    loading.value = false;
  }
}

async function loadDetail(reset = false) {
  if (!selectedId.value) {
    selected.value = null;
    logs.value = [];
    return;
  }
  if (reset) logs.value = [];
  try {
    const after = logs.value.length ? logs.value[logs.value.length - 1].id : 0;
    const r = await admin.job(selectedId.value, after);
    selected.value = r.job;
    if (r.logs.length) {
      logs.value.push(...r.logs);
      await nextTick();
      logBox.value?.scrollTo({ top: logBox.value.scrollHeight });
    }
  } catch (e) {
    toastError(e);
    selected.value = null;
  }
}

function pick(id: string) {
  router.replace({ query: { ...route.query, job: id } });
}

async function start() {
  if (!create.value.sourceId) return;
  try {
    const params: Record<string, unknown> = {};
    if (create.value.keyword.trim()) params.keyword = create.value.keyword.trim();
    if (Number(create.value.maxPages) > 0) params.max_pages = Number(create.value.maxPages);
    const r = await admin.startJob(create.value.sourceId, params);
    toast('采集任务已开始');
    pick(r.id);
    await loadList();
  } catch (e) {
    toastError(e);
  }
}

async function cancel() {
  if (!selected.value) return;
  try {
    await admin.cancelJob(selected.value.id);
    toast('已请求取消，当前页处理完后停止');
  } catch (e) {
    toastError(e);
  }
}

const tiles = computed(() => {
  const s = selected.value?.stats ?? {};
  return [
    { label: '页数', value: s.pages ?? 0 }, { label: '抓取', value: s.fetched ?? 0 }, { label: '新增', value: s.new ?? 0, tone: 'ok' },
    { label: '更新', value: s.changed ?? 0, tone: 'info' }, { label: '跳过', value: s.skipped ?? 0, tone: 'warn' }, { label: '失败', value: s.failed ?? 0, tone: (s.failed ?? 0) > 0 ? 'bad' : '' },
  ];
});
const lines = computed(() => [selected.value?.stats.imported ? `已入库 ${selected.value.stats.imported} 项` : '', selected.value?.stats.import_failed ? `入库失败 ${selected.value.stats.import_failed} 项` : ''].filter(Boolean).join(' · '));

onMounted(async () => {
  try { sources.value = (await admin.sources()).results; } catch (e) { toastError(e); }
  await Promise.all([loadList(), loadDetail(true)]);
  timer = setInterval(async () => {
    if (jobs.value.some((j) => active(j)) || active(selected.value)) {
      await Promise.all([loadList(), loadDetail()]);
    }
  }, 2000);
});
onBeforeUnmount(() => clearInterval(timer));
watch(selectedId, () => loadDetail(true));
watch([filter, page], loadList, { deep: true });
</script>

<template>
  <div class="admin-page">
    <div class="page-h"><div><h1>采集任务</h1><div class="sub">每个来源同一时间只运行一个任务；服务重启时未完成的任务会标记为失败</div></div></div>

    <form class="toolbar panel create" @submit.prevent="start">
      <div class="field"><label for="cs">来源</label>
        <select id="cs" v-model.number="create.sourceId" class="select"><option value="">选择来源</option><option v-for="s in sources.filter((s) => s.enabled && s.configured)" :key="s.id" :value="s.id">{{ s.name }}</option></select>
      </div>
      <div class="field"><label for="ck">关键词（可选）</label><input id="ck" v-model="create.keyword" class="input" /></div>
      <div class="field" style="max-width: 120px"><label for="cm">最多页数</label><input id="cm" v-model="create.maxPages" class="input" inputmode="numeric" placeholder="默认" /></div>
      <button class="btn btn-primary" type="submit" :disabled="!create.sourceId">新建采集任务</button>
    </form>

    <div class="split">
      <section class="panel main">
        <div class="toolbar tb">
          <select v-model.number="filter.sourceId" class="select" aria-label="来源"><option value="">全部来源</option><option v-for="s in sources" :key="s.id" :value="s.id">{{ s.name }}</option></select>
          <select v-model="filter.status" class="select" aria-label="状态"><option value="">全部状态</option><option value="running">进行中</option><option value="succeeded">成功</option><option value="failed">失败</option><option value="cancelled">已取消</option></select>
        </div>
        <div v-if="loading" class="empty"><span class="spinner" /> 加载中…</div>
        <div v-else-if="!jobs.length" class="empty">没有符合条件的任务。</div>
        <div v-else class="tbl-wrap">
          <table class="tbl">
            <thead><tr><th>状态</th><th>来源</th><th>触发</th><th>开始</th><th>耗时</th><th>新增 / 更新 / 失败</th></tr></thead>
            <tbody>
              <tr v-for="j in jobs" :key="j.id" class="click" :class="{ on: j.id === selectedId }" tabindex="0" @click="pick(j.id)" @keydown.enter="pick(j.id)">
                <td><StatusBadge type="job" :value="j.status" /></td>
                <td>{{ j.sourceName }}</td>
                <td>{{ TRIGGER_LABEL[j.trigger] ?? j.trigger }}</td>
                <td>{{ formatTime(j.createdAt) }}</td>
                <td>{{ formatDuration(j.startedAt, j.finishedAt) }}</td>
                <td class="num">{{ j.stats.new ?? 0 }} / {{ j.stats.changed ?? 0 }} / {{ j.stats.failed ?? 0 }}</td>
              </tr>
            </tbody>
          </table>
        </div>
        <div v-if="total > 30" class="pager"><span>共 {{ total }} 条</span><span class="row"><button class="btn btn-sm" type="button" :disabled="page <= 1" @click="page--">上一页</button><button class="btn btn-sm" type="button" :disabled="page * 30 >= total" @click="page++">下一页</button></span></div>
      </section>

      <aside class="panel side">
        <div v-if="!selected" class="empty">选择左侧的任务查看进度和日志。</div>
        <div v-else class="panel-b stack">
          <div class="head">
            <h2>{{ selected.sourceName }}</h2>
            <StatusBadge type="job" :value="selected.status" />
            <span class="by">{{ TRIGGER_LABEL[selected.trigger] ?? selected.trigger }}<template v-if="selected.createdBy"> · {{ selected.createdBy }}</template></span>
          </div>
          <div v-if="active(selected)" class="progress" role="progressbar" aria-label="采集进行中"><i /></div>
          <div class="tiles">
            <div v-for="t in tiles" :key="t.label" class="tile"><b :class="t.tone">{{ t.value }}</b><span>{{ t.label }}</span></div>
          </div>
          <p v-if="lines" class="hint">{{ lines }}</p>
          <div v-if="selected.error" class="error-box">{{ selected.error }}</div>
          <div>
            <div class="log-h"><h3>运行日志</h3><span v-if="active(selected)" class="hint">每 2 秒刷新</span></div>
            <div ref="logBox" class="logs" tabindex="0" role="log" aria-label="运行日志">
              <div v-if="!logs.length" class="hint dim">暂无日志</div>
              <div v-for="l in logs" :key="l.id" class="line"><span class="ts">{{ new Date(l.ts).toLocaleTimeString('zh-CN', { hour12: false }) }}</span> <span :class="['lv', l.level]">{{ l.level.toUpperCase().padEnd(5, ' ') }}</span> <span>{{ l.message }}</span></div>
            </div>
          </div>
          <div class="row">
            <button v-if="active(selected)" class="btn btn-danger" type="button" @click="cancel">取消任务</button>
            <RouterLink class="btn" :to="{ name: 'admin-staging' }">查看暂存区</RouterLink>
          </div>
        </div>
      </aside>
    </div>
  </div>
</template>

<style scoped>
.create { padding: 16px 20px; align-items: flex-end; }
.create .field { flex: 1 1 160px; }
.tb { padding: 14px 16px; border-bottom: 1px solid var(--line); }
.head { display: flex; flex-wrap: wrap; align-items: center; gap: 8px 10px; }
.head h2 { font-size: 18px; font-weight: 700; }
.by { margin-left: auto; font-size: 13px; color: var(--ink-2); }
.progress { height: 6px; border-radius: 999px; background: var(--wash); overflow: hidden; }
.progress i { display: block; height: 100%; width: 35%; background: var(--ink); border-radius: 999px; animation: slide 1.4s ease-in-out infinite; }
@keyframes slide { 0% { transform: translateX(-100%); } 100% { transform: translateX(300%); } }
@media (prefers-reduced-motion: reduce) { .progress i { animation: none; width: 100%; opacity: 0.4; } }
.tiles { display: grid; grid-template-columns: repeat(6, minmax(0, 1fr)); gap: 8px; }
.tile { padding: 10px 4px; border-radius: 10px; background: var(--soft); text-align: center; display: flex; flex-direction: column; }
.tile b { font-size: 20px; line-height: 1.2; }
.tile b.ok { color: var(--ok-ink); } .tile b.info { color: var(--info-ink); } .tile b.warn { color: var(--warn-ink); } .tile b.bad { color: var(--bad-ink); }
.tile span { font-size: 12px; color: var(--ink-2); }
.log-h { display: flex; justify-content: space-between; align-items: baseline; margin-bottom: 6px; }
.log-h h3 { font-size: 15px; font-weight: 700; }
.logs { padding: 12px 14px; border-radius: 12px; background: #141414; color: #dde7e3; font-family: var(--mono); font-size: 12px; line-height: 1.8; max-height: 320px; overflow: auto; }
.line { white-space: pre-wrap; overflow-wrap: anywhere; }
.ts { color: #8fa39c; }
.lv { font-weight: 500; white-space: pre; }
.lv.info { color: #8fd9c5; } .lv.warn { color: #f2c46b; } .lv.error { color: #ff9c8a; }
.dim { color: #8fa39c; }
@media (max-width: 1100px) { .tiles { grid-template-columns: repeat(3, minmax(0, 1fr)); } }
</style>
