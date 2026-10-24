<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { admin } from '@/lib/api';
import { formatTime, KIND_NAME } from '@/lib/format';
import { ITEM_STATUS } from '@/lib/labels';
import { toast, toastError } from '@/lib/toast';
import type { AdminCategory, Source, StagedDetail, StagedRow } from '@/lib/types';
import StatusBadge from '@/components/StatusBadge.vue';

const rows = ref<StagedRow[]>([]);
const total = ref(0);
const counts = ref<Record<string, number>>({});
const sources = ref<Source[]>([]);
const loading = ref(true);
const filter = ref({ status: 'pending', sourceId: '' as number | '', change: '', keyword: '' });
const page = ref(1);
const checked = ref<Set<string>>(new Set());
const detail = ref<StagedDetail | null>(null);
const detailTab = ref<'norm' | 'raw' | 'diff'>('norm');
const detailBusy = ref(false);
const categories = ref<Record<string, AdminCategory[]>>({});
const opts = ref({ status: 'draft', categoryId: '' as number | '' });
let kwTimer: ReturnType<typeof setTimeout> | undefined;

const TABS = [['pending', '待处理'], ['imported', '已入库'], ['ignored', '已忽略'], ['', '全部']] as const;
const allChecked = computed(() => rows.value.length > 0 && rows.value.every((r) => checked.value.has(r.id)));
const tabCount = (s: string) => (s ? counts.value[s] ?? 0 : Object.values(counts.value).reduce((a, b) => a + b, 0));

async function load() {
  loading.value = true;
  try {
    const r = await admin.staged({ status: filter.value.status || undefined, sourceId: filter.value.sourceId || undefined, change: filter.value.change || undefined, keyword: filter.value.keyword.trim() || undefined, page: page.value });
    rows.value = r.results;
    total.value = r.total;
    counts.value = r.counts;
    checked.value = new Set([...checked.value].filter((id) => r.results.some((x) => x.id === id)));
  } catch (e) {
    toastError(e);
  } finally {
    loading.value = false;
  }
}

async function ensureCategories(kind: string) {
  if (!categories.value[kind]) categories.value[kind] = (await admin.categories(kind as any)).results.filter((c) => c.active);
}

async function open(id: string) {
  detailBusy.value = true;
  try {
    const d = await admin.stagedDetail(id);
    detail.value = d;
    detailTab.value = d.existing ? 'diff' : 'norm';
    opts.value = { status: d.existing ? 'keep' : 'draft', categoryId: '' };
    await ensureCategories(d.kind);
  } catch (e) {
    toastError(e);
  } finally {
    detailBusy.value = false;
  }
}

function toggle(id: string) {
  const s = new Set(checked.value);
  s.has(id) ? s.delete(id) : s.add(id);
  checked.value = s;
}
const toggleAll = () => (checked.value = allChecked.value ? new Set() : new Set(rows.value.map((r) => r.id)));

/** 已存在的条目（有更新）保持当前状态，只有新条目套用所选状态。 */
async function importMany(ids: string[], status: string, categoryId?: number) {
  const isExisting = (id: string) => {
    const r = rows.value.find((x) => x.id === id);
    return r ? !!r.itemId : detail.value?.id === id && !!detail.value.existing;
  };
  const fresh = ids.filter((id) => !isExisting(id));
  const existing = ids.filter(isExisting);
  const results: { id: string; ok: boolean; error?: string }[] = [];
  if (fresh.length) results.push(...(await admin.importStaged(fresh, { status, categoryId })).results);
  if (existing.length) results.push(...(await admin.importStaged(existing, { categoryId })).results);
  return results;
}

async function runImport(ids: string[], status: string, categoryId?: number) {
  try {
    const results = await importMany(ids, status, categoryId);
    const ok = results.filter((r) => r.ok).length;
    const bad = results.filter((r) => !r.ok);
    if (ok) toast(`已入库 ${ok} 项`);
    if (bad.length) toast(`${bad.length} 项入库失败：${bad[0].error}${bad.length > 1 ? ' 等' : ''}`, 'bad');
    checked.value = new Set();
    await load();
    if (detail.value && ids.includes(detail.value.id)) await open(detail.value.id);
  } catch (e) {
    toastError(e);
  }
}

async function bulkIgnore() {
  try {
    const r = await admin.ignoreStaged([...checked.value]);
    toast(`已忽略 ${r.updated} 项`);
    checked.value = new Set();
    await load();
  } catch (e) {
    toastError(e);
  }
}
async function bulkRestore(ids: string[]) {
  try {
    await admin.restoreStaged(ids);
    toast('已恢复到待处理');
    checked.value = new Set();
    await load();
    detail.value = null;
  } catch (e) {
    toastError(e);
  }
}

async function importDetail() {
  const d = detail.value;
  if (!d) return;
  const status = opts.value.status === 'keep' ? undefined : opts.value.status;
  try {
    const r = (await admin.importStaged([d.id], { status, categoryId: opts.value.categoryId || undefined })).results[0];
    if (r.ok) toast(d.existing ? '已同步到内容库' : '已入库');
    else toast(r.error ?? '入库失败', 'bad');
    await load();
    await open(d.id);
  } catch (e) {
    toastError(e);
  }
}

interface Diff { label: string; from: string; to: string }
const diffs = computed<Diff[]>(() => {
  const d = detail.value;
  if (!d?.existing) return [];
  const e = d.existing, n = d.normalized;
  const out: Diff[] = [];
  const add = (label: string, from: string, to: string) => from !== to && out.push({ label, from: from || '（空）', to: to || '（空）' });
  add('名称', e.name, n.name);
  add('版本', e.version ?? '', n.version ?? '');
  add('摘要', e.summary, n.summary);
  add('标签', e.tags.join('、'), n.tags.join('、'));
  return out;
});

const configCount = computed(() => ((detail.value?.normalized.metadata as any)?.configs?.length ?? 0) as number);
watch([() => filter.value.status, () => filter.value.sourceId, () => filter.value.change], () => { page.value = 1; load(); });
watch(() => filter.value.keyword, () => { clearTimeout(kwTimer); kwTimer = setTimeout(() => { page.value = 1; load(); }, 300); });
watch(page, load);
onMounted(async () => {
  try { sources.value = (await admin.sources()).results; } catch (e) { toastError(e); }
  load();
});
</script>

<template>
  <div class="admin-page">
    <div class="page-h"><div><h1>暂存区</h1><div class="sub">采集到的内容先放在这里，核对后再入库；上游内容变化时已入库的条目会回到「待处理」</div></div></div>

    <div class="toolbar">
      <div class="seg-tabs" role="group" aria-label="状态">
        <button v-for="[v, l] in TABS" :key="v" type="button" :class="{ on: filter.status === v }" @click="filter.status = v">{{ l }}<small>{{ tabCount(v) }}</small></button>
      </div>
      <select v-model.number="filter.sourceId" class="select" aria-label="来源"><option value="">全部来源</option><option v-for="s in sources" :key="s.id" :value="s.id">{{ s.name }}</option></select>
      <select v-model="filter.change" class="select" aria-label="变化"><option value="">全部变化</option><option value="new">新增</option><option value="changed">有更新</option><option value="failed">入库失败</option></select>
      <input v-model="filter.keyword" class="input grow" type="search" placeholder="按名称、标识、标签搜索" aria-label="搜索暂存内容" />
    </div>

    <div v-if="checked.size" class="bulk" role="region" aria-label="批量操作">
      <b>已选 {{ checked.size }} 项</b><span class="grow" />
      <template v-if="filter.status === 'ignored'"><button class="btn" type="button" @click="bulkRestore([...checked])">恢复到待处理</button></template>
      <template v-else>
        <button class="btn" type="button" @click="runImport([...checked], 'draft')">入库为草稿</button>
        <button class="btn btn-primary" type="button" @click="runImport([...checked], 'published')">入库并上架</button>
        <button class="btn" type="button" @click="bulkIgnore">忽略</button>
      </template>
    </div>

    <div class="split">
      <section class="panel main">
        <div v-if="loading && !rows.length" class="empty"><span class="spinner" /> 加载中…</div>
        <div v-else-if="!rows.length" class="empty">没有符合条件的内容。先到「采集来源」运行一次采集。</div>
        <div v-else class="tbl-wrap">
          <table class="tbl">
            <thead><tr>
              <th style="width: 36px"><input type="checkbox" :checked="allChecked" aria-label="全选" @change="toggleAll" /></th>
              <th>名称</th><th>来源</th><th>上游分类</th><th>状态</th><th>采集于</th>
            </tr></thead>
            <tbody>
              <tr v-for="r in rows" :key="r.id" :class="{ on: detail?.id === r.id }">
                <td><input type="checkbox" :checked="checked.has(r.id)" :aria-label="`选择 ${r.name}`" @change="toggle(r.id)" /></td>
                <td>
                  <button type="button" class="name" @click="open(r.id)">{{ r.name }}</button>
                  <div class="sub mono">{{ r.slug }}<template v-if="r.version"> · {{ r.version }}</template></div>
                  <div v-if="r.importError" class="err">{{ r.importError }}</div>
                </td>
                <td class="muted">{{ r.sourceName }}<div class="sub">{{ KIND_NAME[r.kind] }}</div></td>
                <td class="muted">{{ r.category ?? '—' }}</td>
                <td><StatusBadge v-if="r.change" type="change" :value="r.change" /><StatusBadge v-else type="staged" :value="r.status" /></td>
                <td class="muted nowrap">{{ formatTime(r.lastSeenAt) }}</td>
              </tr>
            </tbody>
          </table>
        </div>
        <div class="pager"><span>共 {{ total }} 条 · 每页 50 条</span><span class="row"><button class="btn btn-sm" type="button" :disabled="page <= 1" @click="page--">上一页</button><button class="btn btn-sm" type="button" :disabled="page * 50 >= total" @click="page++">下一页</button></span></div>
      </section>

      <aside class="panel side">
        <div v-if="detailBusy && !detail" class="empty"><span class="spinner" /></div>
        <div v-else-if="!detail" class="empty">点击左侧的名称查看详情、对比差异并入库。</div>
        <div v-else class="panel-b stack">
          <div class="head">
            <div><h2>{{ detail.normalized.name }}</h2><div class="sub mono">{{ detail.normalized.slug }} · {{ detail.sourceName }}</div></div>
            <StatusBadge v-if="detail.importError" type="change" value="failed" />
            <StatusBadge v-else-if="detail.existing && detail.status === 'pending'" type="change" value="changed" />
            <StatusBadge v-else type="staged" :value="detail.status" />
          </div>
          <div v-if="detail.importError" class="error-box">{{ detail.importError }}</div>

          <div class="seg-tabs" role="group" aria-label="详情视图">
            <button type="button" :class="{ on: detailTab === 'norm' }" @click="detailTab = 'norm'">规范化</button>
            <button type="button" :class="{ on: detailTab === 'raw' }" @click="detailTab = 'raw'">上游原文</button>
            <button v-if="detail.existing" type="button" :class="{ on: detailTab === 'diff' }" @click="detailTab = 'diff'">与已入库对比</button>
          </div>

          <div v-if="detailTab === 'norm'" class="kv">
            <span>名称</span><span>{{ detail.normalized.name }}</span>
            <span>标识</span><span class="mono">{{ detail.normalized.namespace }} / {{ detail.normalized.slug }}</span>
            <span>摘要</span><span>{{ detail.normalized.summary || '—' }}</span>
            <span>上游分类</span><span>{{ detail.normalized.category ?? '—' }}</span>
            <span>标签</span><span>{{ detail.normalized.tags.join(' · ') || '—' }}</span>
            <span>版本</span><span>{{ detail.normalized.version ?? '—' }}</span>
            <span>许可证</span><span>{{ detail.normalized.license ?? '—' }}</span>
            <span>来源链接</span><span><a v-if="detail.normalized.origin_url" class="link" :href="detail.normalized.origin_url" target="_blank" rel="noopener noreferrer">{{ detail.normalized.origin_url }}</a><template v-else>—</template></span>
            <template v-if="configCount"><span>连接配置</span><span>{{ configCount }} 份</span></template>
          </div>
          <pre v-else-if="detailTab === 'raw'" class="pre">{{ JSON.stringify(detail.raw, null, 2) }}</pre>
          <div v-else class="diffs">
            <div v-if="!diffs.length" class="hint">规范化字段没有变化；变化可能在详情内容或安装包里，入库时会重新抓取。</div>
            <div v-for="d in diffs" :key="d.label" class="diff">
              <div class="dl">{{ d.label }}</div>
              <div class="minus"><span class="sign mono">−</span>{{ d.from }}</div>
              <div class="plus"><span class="sign mono">+</span>{{ d.to }}</div>
            </div>
          </div>

          <div v-if="detail.status !== 'ignored'" class="stack settings">
            <h3>入库设置</h3>
            <div class="field"><label for="cat">本地分类</label>
              <select id="cat" v-model.number="opts.categoryId" class="select">
                <option value="">按上游分类自动匹配{{ detail.normalized.category ? `（${detail.normalized.category}）` : '' }}</option>
                <option v-for="c in categories[detail.kind] ?? []" :key="c.id" :value="c.id">{{ c.name }}</option>
              </select>
              <span class="hint">手动选择后会记住「上游分类 → 本地分类」的映射。</span>
            </div>
            <fieldset class="radios">
              <legend class="label">入库状态</legend>
              <label v-if="detail.existing" class="check"><input v-model="opts.status" type="radio" value="keep" />保持当前状态（{{ ITEM_STATUS[detail.existing.status]?.[0] }}）</label>
              <label class="check"><input v-model="opts.status" type="radio" value="draft" />草稿</label>
              <label class="check"><input v-model="opts.status" type="radio" value="review" />待审</label>
              <label class="check"><input v-model="opts.status" type="radio" value="published" />上架</label>
            </fieldset>
            <p v-if="detail.kind === 'skill'" class="hint">入库时会下载并校验安装包；不合规的包会拒绝入库并在这里显示原因。</p>
          </div>

          <div class="row">
            <button v-if="detail.status !== 'ignored'" class="btn btn-primary grow" type="button" @click="importDetail">{{ detail.existing ? '同步到内容库' : '入库' }}</button>
            <button v-if="detail.status === 'pending'" class="btn" type="button" @click="admin.ignoreStaged([detail.id]).then(() => { toast('已忽略'); load(); detail = null; }).catch(toastError)">{{ detail.existing ? '忽略此次更新' : '忽略' }}</button>
            <button v-if="detail.status === 'ignored'" class="btn" type="button" @click="bulkRestore([detail.id])">恢复到待处理</button>
          </div>
        </div>
      </aside>
    </div>
  </div>
</template>

<style scoped>
.bulk { display: flex; flex-wrap: wrap; align-items: center; gap: 10px 12px; padding: 10px 14px; border-radius: 12px; background: #f2f2f0; }
.grow { flex: 1 1 120px; }
.name { border: 0; background: transparent; padding: 0; font-weight: 500; text-align: left; cursor: pointer; text-decoration: underline; text-decoration-color: transparent; text-underline-offset: 3px; }
.name:hover { text-decoration-color: currentColor; }
.muted { color: var(--ink-2); }
.nowrap { white-space: nowrap; }
.head { display: flex; align-items: flex-start; gap: 10px; justify-content: space-between; }
.head h2 { font-size: 17px; font-weight: 700; overflow-wrap: anywhere; }
.diffs { display: flex; flex-direction: column; gap: 10px; }
.diff { border: 1px solid var(--line); border-radius: 10px; overflow: hidden; font-size: 13px; }
.dl { padding: 4px 12px; background: var(--soft); color: var(--ink-2); font-size: 12px; }
.minus, .plus { padding: 8px 12px; display: flex; gap: 8px; overflow-wrap: anywhere; }
.minus { background: var(--bad-bg); color: #6f1f11; }
.plus { background: var(--ok-bg); color: #0b4a3a; }
.sign { flex: 0 0 auto; }
.settings { padding-top: 14px; border-top: 1px solid #f0f0ee; }
.settings h3 { font-size: 15px; font-weight: 700; }
.radios { border: 0; margin: 0; padding: 0; display: flex; flex-direction: column; }
</style>
