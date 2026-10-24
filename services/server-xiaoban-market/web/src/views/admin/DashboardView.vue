<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { admin } from '@/lib/api';
import { formatTime, formatDuration } from '@/lib/format';
import { intervalLabel, TRIGGER_LABEL } from '@/lib/labels';
import { toastError } from '@/lib/toast';
import type { Job, Source, Stats } from '@/lib/types';
import StatusBadge from '@/components/StatusBadge.vue';

const stats = ref<Stats | null>(null);
const sources = ref<Source[]>([]);
const jobs = ref<Job[]>([]);
const loading = ref(true);

async function load() {
  loading.value = true;
  try {
    [stats.value, sources.value, jobs.value] = await Promise.all([admin.stats(), admin.sources().then((r) => r.results), admin.jobs({}).then((r) => r.results.slice(0, 6))]);
  } catch (e) {
    toastError(e);
  } finally {
    loading.value = false;
  }
}
onMounted(load);

const week = computed(() => {
  const w = stats.value?.publishedThisWeek ?? {};
  const total = (w.skill ?? 0) + (w.expert ?? 0) + (w.connector ?? 0);
  return { total, hint: `技能 ${w.skill ?? 0} · 专家 ${w.expert ?? 0} · 连接器 ${w.connector ?? 0}` };
});

const todo = computed(() => {
  const list: { tag: string; tone: string; title: string; body: string; to: string }[] = [];
  for (const s of sources.value.filter((s) => !s.configured && s.enabled)) {
    list.push({ tag: '待配置', tone: 'warn', title: `${s.name} 接口尚未配置`, body: '填写列表接口并试采一页，确认字段映射后再采集。', to: 'admin-sources' });
  }
  if (stats.value?.upstreamChanged) list.push({ tag: '有更新', tone: 'info', title: `${stats.value.upstreamChanged} 个已入库条目的上游内容已变化`, body: '到暂存区对比差异，决定是否同步。', to: 'admin-staging' });
  if (stats.value?.importFailed) list.push({ tag: '入库失败', tone: 'bad', title: `${stats.value.importFailed} 项入库失败`, body: '安装包未通过校验或标识冲突，需要处理或忽略。', to: 'admin-staging' });
  return list;
});

function sourceState(s: Source): [string, string] {
  if (!s.enabled) return ['已停用', 'idle'];
  if (!s.configured) return ['待配置', 'warn'];
  if (s.lastJob?.status === 'failed') return ['上次失败', 'bad'];
  if (s.lastJob?.status === 'running') return ['采集中', 'info'];
  return ['正常', 'ok'];
}
function resultText(s: Source): string {
  const st = s.lastJob?.stats;
  return st ? `+${st.new ?? 0} ~${st.changed ?? 0} ✕${st.failed ?? 0}` : '—';
}
const statLine = (j: Job) => `抓取 ${j.stats.fetched ?? 0} · 新增 ${j.stats.new ?? 0} · 更新 ${j.stats.changed ?? 0}`;
</script>

<template>
  <div class="admin-page">
    <div class="page-h">
      <div><h1>总览</h1><div class="sub">汇总采集、暂存区和内容库的当前状态</div></div>
      <button class="btn" type="button" :disabled="loading" @click="load">刷新</button>
    </div>

    <section class="cards">
      <RouterLink :to="{ name: 'admin-staging' }" class="stat panel"><span>待审核暂存</span><b>{{ stats?.pendingStaged ?? '—' }}</b><small>等待入库或忽略</small></RouterLink>
      <RouterLink :to="{ name: 'admin-library' }" class="stat panel"><span>本周上架</span><b>{{ stats ? week.total : '—' }}</b><small>{{ week.hint }}</small></RouterLink>
      <RouterLink :to="{ name: 'admin-staging' }" class="stat panel"><span>上游有更新</span><b class="warn">{{ stats?.upstreamChanged ?? '—' }}</b><small>已入库条目的上游内容变化</small></RouterLink>
      <RouterLink :to="{ name: 'admin-jobs' }" class="stat panel"><span>24 小时内失败</span><b class="bad">{{ stats?.failedJobs24h ?? '—' }}</b><small>采集任务失败次数</small></RouterLink>
    </section>

    <div class="split">
      <section class="panel main">
        <div class="panel-h"><h2>来源状态</h2><RouterLink class="link" :to="{ name: 'admin-sources' }">管理来源 →</RouterLink></div>
        <div class="tbl-wrap">
          <table class="tbl">
            <thead><tr><th>来源</th><th>上次采集</th><th>结果</th><th>状态</th></tr></thead>
            <tbody>
              <tr v-for="s in sources" :key="s.id">
                <td><div>{{ s.name }}</div><div class="sub">{{ s.adapter === 'skillhub' ? 'SkillHub 适配器' : '通用 JSON 适配器' }} · {{ intervalLabel(s.intervalMinutes) }}</div></td>
                <td>{{ s.lastJob ? formatTime(s.lastJob.createdAt) : '—' }}</td>
                <td class="num">{{ resultText(s) }}</td>
                <td><span class="badge" :class="`badge-${sourceState(s)[1]}`">{{ sourceState(s)[0] }}</span></td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>

      <section class="panel side">
        <div class="panel-h"><h2>待处理</h2></div>
        <div v-if="!todo.length" class="empty">暂时没有需要处理的事项。</div>
        <RouterLink v-for="t in todo" :key="t.title" :to="{ name: t.to }" class="todo">
          <span class="badge" :class="`badge-${t.tone}`">{{ t.tag }}</span>
          <b>{{ t.title }}</b>
          <small>{{ t.body }}</small>
        </RouterLink>
      </section>
    </div>

    <section class="panel">
      <div class="panel-h"><h2>最近任务</h2><RouterLink class="link" :to="{ name: 'admin-jobs' }">全部任务 →</RouterLink></div>
      <div v-if="!jobs.length" class="empty">还没有采集任务。</div>
      <div v-else class="tbl-wrap">
        <table class="tbl">
          <tbody>
            <tr v-for="j in jobs" :key="j.id">
              <td>{{ j.sourceName }}</td>
              <td>{{ TRIGGER_LABEL[j.trigger] ?? j.trigger }}</td>
              <td>{{ formatTime(j.createdAt) }} · {{ formatDuration(j.startedAt, j.finishedAt) }}</td>
              <td class="num">{{ statLine(j) }}</td>
              <td><StatusBadge type="job" :value="j.status" /></td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>
  </div>
</template>

<style scoped>
.cards { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 16px; }
.stat { display: flex; flex-direction: column; gap: 2px; padding: 18px 20px; }
.stat span { font-size: 13px; color: var(--ink-2); }
.stat b { font-size: 34px; line-height: 1.2; font-weight: 700; }
.stat b.warn { color: var(--warn-ink); }
.stat b.bad { color: var(--bad-ink); }
.stat small { font-size: 13px; color: var(--ink-2); }
.todo { display: flex; flex-direction: column; align-items: flex-start; gap: 4px; padding: 14px 20px; border-top: 1px solid #f0f0ee; }
.todo small { color: var(--ink-2); font-size: 13px; }
@media (max-width: 900px) { .cards { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
</style>
