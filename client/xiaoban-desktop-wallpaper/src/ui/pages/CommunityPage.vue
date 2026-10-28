<script setup lang="ts">
import { onActivated, onMounted, ref } from 'vue';
import { NAlert, NButton, NSkeleton } from 'naive-ui';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import { KIND_LABEL, startApply, startScheme, SUPPORTED_KINDS, trackEvent, TRUST_LABEL, type SchemeItem } from '../community/apply.ts';

defineOptions({ name: 'CommunityPage' });
const props = defineProps<{ store: ControlStore }>();

interface PackRow { id: number; kind: string; title: string; summary: string; trust_level: number; apply_count: number; one_click: boolean }
interface WeeklyItem { type: string; id?: number; kind?: string; title?: string | null; summary?: string; note?: string }
interface Weekly { week: string; title: string; summary: string; items: WeeklyItem[] }

/** 桌面端现在能应用的类型，其余类型先在网页上看。 */
const KINDS = [{ key: '', label: '全部' }, { key: 'schemes', label: '方案' }, { key: 'effect', label: '特效' }, { key: 'reminder', label: '提醒' }, { key: 'routine', label: '例行事项' }, { key: 'task', label: '任务模板' }, { key: 'goal', label: '目标模板' }];
const kind = ref('');
const packs = ref<PackRow[]>([])
interface SchemeRow { id: number; title: string; author: { nickname: string | null }; items: SchemeItem[] }
const schemes = ref<SchemeRow[]>([]);
const weekly = ref<Weekly | null>(null);
const loading = ref(true);
const error = ref('');

const text = (e: unknown) => (e instanceof Error ? e.message : typeof e === 'string' ? e : '社区暂时无法访问');

async function loadWeekly() {
  try {
    const list = await invoke<{ items: { week: string }[] }>('community_browse', { path: '/api/weekly?page=1' });
    const latest = list.items[0];
    weekly.value = latest ? await invoke<Weekly>('community_browse', { path: `/api/weekly/${latest.week}` }) : null;
  } catch { weekly.value = null; }
}
async function loadPacks() {
  loading.value = true;
  error.value = '';
  try {
    if (kind.value === 'schemes') {
      schemes.value = (await invoke<{ items: SchemeRow[] }>('community_browse', { path: '/api/schemes?page=1' })).items;
      packs.value = [];
      return;
    }
    schemes.value = [];
    const query = kind.value ? `kind=${kind.value}&sort=hot` : 'sort=hot';
    const page = await invoke<{ items: PackRow[] }>('community_browse', { path: `/api/packs?${query}` });
    packs.value = page.items;
  } catch (e) { error.value = text(e); packs.value = []; }
  finally { loading.value = false; }
}
function pick(next: string) { kind.value = next; void loadPacks(); }
function replicate(row: SchemeRow) {
  if (!startScheme(row.title, row.items)) props.store.notify('这个方案里的内容桌面端暂时都不能应用', 'info');
}
function open(row: PackRow) { void startApply({ packId: row.id }); }
function openWeeklyPack(item: WeeklyItem) { if (item.type === 'pack' && item.id) void startApply({ packId: item.id }); }
const supported = (k: string) => SUPPORTED_KINDS.includes(k);

const statsOn = ref(true);
async function loadStats() { try { statsOn.value = await invoke<boolean>('community_analytics_get'); } catch { /* 保持默认显示 */ } }
async function toggleStats() {
  const next = !statsOn.value;
  try { await invoke('community_analytics_set', { on: next }); statsOn.value = next; } catch (e) { props.store.notify(e, 'error'); }
}
function refresh() { if (props.store.desktop) { trackEvent('desktop_community_open'); void loadWeekly(); void loadPacks(); void loadStats(); } else loading.value = false; }
onMounted(refresh);
onActivated(refresh);
</script>

<template>
  <section class="community-page">
    <h1>社区</h1>
    <NAlert v-if="!store.desktop" type="info" :show-icon="false">社区内容需要在桌面应用中浏览。</NAlert>

    <template v-else>
      <div v-if="weekly" class="community-weekly">
        <p class="community-eyebrow">本期周刊 · {{ weekly.week }}</p>
        <h2>{{ weekly.title }}</h2>
        <p v-if="weekly.summary" class="community-lead">{{ weekly.summary }}</p>
        <ul>
          <li v-for="(item, i) in weekly.items.filter(x => x.type !== 'text')" :key="i">
            <button v-if="item.type === 'pack' && supported(item.kind ?? '')" type="button" class="community-link" @click="openWeeklyPack(item)">{{ item.title }}</button>
            <span v-else>{{ item.title }}</span>
            <small v-if="item.note"> — {{ item.note }}</small>
          </li>
        </ul>
      </div>

      <div class="community-tabs" role="tablist" aria-label="内容类型">
        <button v-for="k in KINDS" :key="k.key" type="button" role="tab" :aria-selected="kind === k.key" :class="{ on: kind === k.key }" @click="pick(k.key)">{{ k.label }}</button>
      </div>

      <NAlert v-if="error" type="warning" :show-icon="false">{{ error }}<NButton size="tiny" quaternary @click="loadPacks">重试</NButton></NAlert>
      <NSkeleton v-else-if="loading && !packs.length" text :repeat="4" />
      <ul v-else-if="kind === 'schemes' && schemes.length" class="community-list">
        <li v-for="row in schemes" :key="row.id">
          <div class="community-main">
            <strong>{{ row.title }}</strong>
            <span class="community-tag">{{ row.author.nickname }}</span>
            <span class="community-tag">{{ row.items.length }} 项内容</span>
          </div>
          <NButton size="small" @click="replicate(row)">复刻</NButton>
        </li>
      </ul>
      <p v-else-if="!packs.length" class="community-empty">这里还没有内容。</p>
      <ul v-else class="community-list">
        <li v-for="row in packs" :key="row.id">
          <div class="community-main">
            <strong>{{ row.title }}</strong>
            <span class="community-tag">{{ KIND_LABEL[row.kind] ?? row.kind }}</span>
            <span class="community-tag">{{ TRUST_LABEL[row.trust_level] ?? '未审核' }}</span>
            <p v-if="row.summary">{{ row.summary }}</p>
          </div>
          <NButton v-if="supported(row.kind)" size="small" @click="open(row)">查看并应用</NButton>
          <small v-else class="community-tag">请在网页查看</small>
        </li>
      </ul>
      <p class="community-foot">
        匿名统计：只记匿名设备做过哪类操作的次数，不关联账号，不记录内容、搜索词和文件名。
        <button type="button" class="community-link" :aria-pressed="statsOn" @click="toggleStats">{{ statsOn ? '已开启，点击关闭' : '已关闭，点击开启' }}</button>
      </p>
    </template>
  </section>
</template>

<style scoped>
.community-page { padding: 20px 24px; max-width: 880px; margin: 0 auto; }
h1 { margin: 0 0 12px; font-size: 20px; font-weight: 600; }
.community-weekly { margin-bottom: 18px; padding-bottom: 12px; border-bottom: 1px solid var(--xb-line, #eee); }
.community-weekly h2 { margin: 2px 0 0; font-size: 16px; font-weight: 600; }
.community-eyebrow { margin: 0; font-size: 12px; color: var(--xb-text-3, #888); }
.community-lead { margin: 6px 0; color: var(--xb-text-2, #666); font-size: 13px; }
.community-weekly ul { margin: 6px 0 0; padding-left: 18px; font-size: 13px; }
.community-link { border: 0; background: none; padding: 0; color: inherit; text-decoration: underline; cursor: pointer; font: inherit; }
.community-tabs { display: flex; gap: 16px; margin-bottom: 8px; }
.community-tabs button { border: 0; background: none; padding: 4px 0 6px; color: var(--xb-text-3, #888); border-bottom: 2px solid transparent; cursor: pointer; }
.community-tabs button.on { color: inherit; font-weight: 600; border-bottom-color: var(--xb-accent, #ff2442); }
.community-list { list-style: none; margin: 0; padding: 0; }
.community-list li { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 10px 0; border-bottom: 1px solid var(--xb-line, #eee); }
.community-main p { margin: 2px 0 0; font-size: 13px; color: var(--xb-text-2, #666); }
.community-tag { margin-left: 8px; font-size: 12px; color: var(--xb-text-3, #888); }
.community-foot { margin-top: 20px; font-size: 12px; color: var(--xb-text-3, #888); }
.community-empty { color: var(--xb-text-3, #888); }
</style>
