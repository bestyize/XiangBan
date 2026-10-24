<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { admin } from '@/lib/api';
import { formatCount, formatSize, formatTime, KIND_NAME, shortHash } from '@/lib/format';
import { ORIGIN_LABEL, SOURCE_LABEL } from '@/lib/labels';
import { toast, toastError } from '@/lib/toast';
import { VARIANT_LABEL } from '@/lib/tabs';
import type { AdminCategory, AdminItem, AdminMember, AdminVersion, Kind } from '@/lib/types';
import StatusBadge from '@/components/StatusBadge.vue';

const KINDS: Kind[] = ['skill', 'expert', 'connector'];
const kind = ref<Kind>('skill');
const filter = ref({ status: '', variant: '', categoryId: '' as number | '', keyword: '' });
const page = ref(1);
const items = ref<AdminItem[]>([]);
const total = ref(0);
const loading = ref(true);
const categories = ref<AdminCategory[]>([]);
const current = ref<{ item: AdminItem; versions: AdminVersion[]; members: AdminMember[] } | null>(null);
const busy = ref(false);
let kwTimer: ReturnType<typeof setTimeout> | undefined;

interface Form { name: string; summary: string; description: string; categoryId: number | ''; tags: string; license: string; iconUrl: string; source: string; featured: boolean; prompts: string; members: string }
const form = ref<Form>({ name: '', summary: '', description: '', categoryId: '', tags: '', license: '', iconUrl: '', source: 'official', featured: false, prompts: '', members: '' });
const upload = ref({ version: '', changelog: '', file: null as File | null });
const creating = ref(false);
const draft = ref({ slug: '', name: '', summary: '', categoryId: '' as number | '', variant: 'single', members: '' });

/** 每行一个「命名空间/标识」。 */
function parseMembers(text: string): { namespace: string; slug: string }[] {
  return text.split('\n').map((l) => l.trim()).filter(Boolean).map((l) => {
    const [namespace, slug] = l.includes('/') ? l.split('/', 2) : ['xiaoban', l];
    return { namespace: namespace.trim(), slug: slug.trim() };
  });
}
const variantOptions = computed(() => (kind.value === 'expert' ? [['single', '单个专家'], ['team', '专家团']] : kind.value === 'skill' ? [['single', '单个技能'], ['bundle', '套件']] : [['single', '连接器']]));

async function load() {
  loading.value = true;
  try {
    const r = await admin.items({ kind: kind.value, status: filter.value.status || undefined, variant: filter.value.variant || undefined, categoryId: filter.value.categoryId || undefined, keyword: filter.value.keyword.trim() || undefined, page: page.value });
    items.value = r.results;
    total.value = r.total;
  } catch (e) {
    toastError(e);
  } finally {
    loading.value = false;
  }
}

async function loadCategories() {
  try { categories.value = (await admin.categories(kind.value)).results; } catch (e) { toastError(e); }
}

function fillForm(it: AdminItem) {
  form.value = {
    name: it.name, summary: it.summary, description: it.description ?? '', categoryId: it.categoryId ?? '', tags: it.tags.join('，'),
    license: it.license ?? '', iconUrl: it.iconUrl ?? '', source: it.source, featured: it.featured, prompts: (it.prompts ?? []).join('\n'), members: '',
  };
}

async function open(id: string) {
  try {
    const r = await admin.item(id);
    current.value = r;
    fillForm(r.item);
    form.value.members = r.members.map((m) => `${m.namespace}/${m.slug}`).join('\n');
    upload.value = { version: '', changelog: '', file: null };
  } catch (e) {
    toastError(e);
  }
}

async function save() {
  const c = current.value;
  if (!c) return;
  busy.value = true;
  try {
    const f = form.value;
    await admin.updateItem(c.item.id, {
      name: f.name, summary: f.summary, description: f.description, categoryId: f.categoryId || undefined,
      tags: f.tags.split(/[,，、\n]/).map((t) => t.trim()).filter(Boolean), license: f.license, iconUrl: f.iconUrl, source: f.source, featured: f.featured,
      prompts: f.prompts.split('\n').map((t) => t.trim()).filter(Boolean),
      members: c.item.variant === 'single' ? undefined : parseMembers(f.members),
    });
    toast('已保存');
    await Promise.all([load(), open(c.item.id)]);
  } catch (e) {
    toastError(e);
  } finally {
    busy.value = false;
  }
}

async function setStatus(status: string) {
  const c = current.value;
  if (!c) return;
  try {
    await admin.setItemStatus(c.item.id, status);
    toast(status === 'published' ? '已上架' : status === 'offline' ? '已下架' : '状态已更新');
    await Promise.all([load(), open(c.item.id)]);
  } catch (e) {
    toastError(e);
  }
}

async function remove() {
  const c = current.value;
  if (!c || !window.confirm(`确定删除「${c.item.name}」吗？它的所有版本记录会一并删除，此操作不可恢复。`)) return;
  try {
    await admin.deleteItem(c.item.id);
    toast('已删除');
    current.value = null;
    await load();
  } catch (e) {
    toastError(e);
  }
}

async function yank(v: AdminVersion) {
  try {
    await admin.yankVersion(v.id, !v.yanked);
    toast(v.yanked ? '已恢复' : '已撤回，前台不再提供下载');
    await open(current.value!.item.id);
    await load();
  } catch (e) {
    toastError(e);
  }
}

async function doUpload() {
  const c = current.value;
  if (!c || !upload.value.file) return;
  busy.value = true;
  try {
    const fd = new FormData();
    fd.set('version', upload.value.version.trim());
    fd.set('changelog', upload.value.changelog);
    fd.set('file', upload.value.file);
    const r = await admin.uploadVersion(c.item.id, fd);
    toast(`已上传 ${r.version}（${r.files} 个文件）`);
    upload.value = { version: '', changelog: '', file: null };
    await Promise.all([load(), open(c.item.id)]);
  } catch (e) {
    toastError(e);
  } finally {
    busy.value = false;
  }
}

async function createItem() {
  busy.value = true;
  try {
    const r = await admin.createItem({ kind: kind.value, slug: draft.value.slug.trim(), name: draft.value.name.trim(), summary: draft.value.summary, categoryId: draft.value.categoryId || undefined, variant: draft.value.variant, members: draft.value.variant === 'single' ? [] : parseMembers(draft.value.members) });
    toast('已创建草稿');
    creating.value = false;
    draft.value = { slug: '', name: '', summary: '', categoryId: '', variant: 'single', members: '' };
    await load();
    await open(r.id);
  } catch (e) {
    toastError(e);
  } finally {
    busy.value = false;
  }
}

const hasVersion = computed(() => current.value?.versions.some((v) => !v.yanked) ?? false);
const statusActions = computed(() => {
  const s = current.value?.item.status;
  return [
    s !== 'published' && { to: 'published', label: '上架', disabled: kind.value === 'skill' && current.value?.item.variant === 'single' && !hasVersion.value },
    s !== 'review' && s !== 'published' && { to: 'review', label: '设为待审' },
    s === 'published' && { to: 'offline', label: '下架' },
    s !== 'draft' && s !== 'published' && { to: 'draft', label: '设为草稿' },
  ].filter(Boolean) as { to: string; label: string; disabled?: boolean }[];
});

watch(kind, () => { page.value = 1; filter.value = { status: '', variant: '', categoryId: '', keyword: '' }; current.value = null; creating.value = false; loadCategories(); load(); });
watch([() => filter.value.status, () => filter.value.variant, () => filter.value.categoryId], () => { page.value = 1; load(); });
watch(() => filter.value.keyword, () => { clearTimeout(kwTimer); kwTimer = setTimeout(() => { page.value = 1; load(); }, 300); });
watch(page, load);
onMounted(() => { loadCategories(); load(); });
</script>

<template>
  <div class="admin-page">
    <div class="page-h">
      <div><h1>内容库</h1><div class="sub">已入库的技能、专家和连接器；前台只展示「已上架」</div></div>
      <button class="btn btn-primary" type="button" @click="creating = !creating">手动新建</button>
    </div>

    <form v-if="creating" class="panel create" @submit.prevent="createItem">
      <div class="row">
        <div class="field"><label for="ns">标识（小写字母、数字、连字符）</label><input id="ns" v-model="draft.slug" class="input mono-input" required pattern="[a-z0-9][a-z0-9-]{0,62}[a-z0-9]" /></div>
        <div class="field"><label for="nn">名称</label><input id="nn" v-model="draft.name" class="input" required /></div>
        <div v-if="variantOptions.length > 1" class="field"><label for="nv">形态</label><select id="nv" v-model="draft.variant" class="select"><option v-for="[v, l] in variantOptions" :key="v" :value="v">{{ l }}</option></select></div>
        <div class="field"><label for="nc">分类</label><select id="nc" v-model.number="draft.categoryId" class="select"><option value="">不分类</option><option v-for="c in categories" :key="c.id" :value="c.id">{{ c.name }}</option></select></div>
      </div>
      <div class="field"><label for="nsu">摘要</label><input id="nsu" v-model="draft.summary" class="input" /></div>
      <div v-if="draft.variant !== 'single'" class="field"><label for="nm">{{ draft.variant === 'team' ? '团队成员（专家）' : '套件包含的技能' }}：每行一个「命名空间/标识」，最多 24 个</label><textarea id="nm" v-model="draft.members" class="textarea mono-input" rows="4" placeholder="xiaoban/product-manager" required /></div>
      <div class="row"><button class="btn btn-primary" type="submit" :disabled="busy">创建为草稿</button><button class="btn" type="button" @click="creating = false">取消</button></div>
    </form>

    <div class="toolbar">
      <div class="seg-tabs" role="group" aria-label="类型"><button v-for="k in KINDS" :key="k" type="button" :class="{ on: kind === k }" @click="kind = k">{{ KIND_NAME[k] }}</button></div>
      <select v-model="filter.status" class="select" aria-label="状态"><option value="">全部状态</option><option value="published">已上架</option><option value="draft">草稿</option><option value="review">待审</option><option value="offline">已下架</option></select>
      <select v-if="variantOptions.length > 1" v-model="filter.variant" class="select" aria-label="形态"><option value="">全部形态</option><option v-for="[v, l] in variantOptions" :key="v" :value="v">{{ l }}</option></select>
      <select v-model.number="filter.categoryId" class="select" aria-label="分类"><option value="">全部分类</option><option v-for="c in categories" :key="c.id" :value="c.id">{{ c.name }}</option></select>
      <input v-model="filter.keyword" class="input grow" type="search" placeholder="按名称、标识、标签搜索" aria-label="搜索内容库" />
    </div>

    <div class="split">
      <section class="panel main">
        <div v-if="loading && !items.length" class="empty"><span class="spinner" /> 加载中…</div>
        <div v-else-if="!items.length" class="empty">这里还没有{{ KIND_NAME[kind] }}。到「暂存区」入库，或手动新建。</div>
        <div v-else class="tbl-wrap">
          <table class="tbl">
            <thead><tr><th>名称</th><th>状态</th><th>来源</th><th>版本</th><th style="text-align: right">安装</th></tr></thead>
            <tbody>
              <tr v-for="it in items" :key="it.id" class="click" :class="{ on: current?.item.id === it.id }" tabindex="0" @click="open(it.id)" @keydown.enter="open(it.id)">
                <td><div class="nm">{{ it.name }}<span v-if="it.variant !== 'single'" class="vt">{{ VARIANT_LABEL[it.variant] }}</span><span v-if="it.featured" class="star">精选</span></div><div class="sub mono">{{ it.slug }}</div></td>
                <td><StatusBadge type="item" :value="it.status" /></td>
                <td class="muted">{{ ORIGIN_LABEL[it.origin] ?? it.origin }}</td>
                <td class="num muted">{{ it.version ?? '—' }}</td>
                <td class="num muted" style="text-align: right">{{ formatCount(it.downloads) }}</td>
              </tr>
            </tbody>
          </table>
        </div>
        <div class="pager"><span>共 {{ total }} 条 · 每页 50 条</span><span class="row"><button class="btn btn-sm" type="button" :disabled="page <= 1" @click="page--">上一页</button><button class="btn btn-sm" type="button" :disabled="page * 50 >= total" @click="page++">下一页</button></span></div>
      </section>

      <aside class="panel side">
        <div v-if="!current" class="empty">点击左侧的条目进行编辑。</div>
        <div v-else class="panel-b stack">
          <div class="head">
            <h2>{{ current.item.name }}</h2>
            <span class="kindtag">{{ KIND_NAME[current.item.kind] }}</span>
            <StatusBadge type="item" :value="current.item.status" />
          </div>
          <div class="sub mono">{{ current.item.namespace }} / {{ current.item.slug }} · {{ ORIGIN_LABEL[current.item.origin] ?? current.item.origin }} · 更新 {{ formatTime(current.item.updatedAt) }}</div>

          <div class="field"><label for="en">名称</label><input id="en" v-model="form.name" class="input" /></div>
          <div class="field"><label for="es">摘要</label><textarea id="es" v-model="form.summary" class="textarea" rows="2" /></div>
          <div class="row">
            <div class="field"><label for="ec">分类</label><select id="ec" v-model.number="form.categoryId" class="select"><option value="">未分类</option><option v-for="c in categories" :key="c.id" :value="c.id">{{ c.name }}</option></select></div>
            <div class="field"><label for="el">许可证</label><input id="el" v-model="form.license" class="input" /></div>
          </div>
          <div class="field"><label for="et">标签（逗号分隔）</label><input id="et" v-model="form.tags" class="input" /></div>
          <div class="row">
            <div class="field"><label for="eso">来源标记</label><select id="eso" v-model="form.source" class="select"><option v-for="(l, v) in SOURCE_LABEL" :key="v" :value="v">{{ l }}</option></select></div>
            <div class="field"><label for="ei">图标地址（HTTPS）</label><input id="ei" v-model="form.iconUrl" class="input mono-input" /></div>
          </div>
          <div class="field"><label for="ep">试试这样用（每行一条，最多 8 条）</label><textarea id="ep" v-model="form.prompts" class="textarea" rows="3" /></div>
          <div class="field"><label for="ed">{{ kind === 'skill' ? 'SKILL.md / 详细说明' : kind === 'expert' ? '工作流正文' : 'README' }}</label><textarea id="ed" v-model="form.description" class="textarea mono-input" rows="5" /></div>
          <label class="check"><input v-model="form.featured" type="checkbox" />首页精选</label>

          <div v-if="current.item.variant !== 'single'" class="stack members">
            <div class="field"><label for="em">{{ current.item.variant === 'team' ? '团队成员（专家）' : '套件包含的技能' }}：每行一个「命名空间/标识」，顺序即前台展示顺序</label><textarea id="em" v-model="form.members" class="textarea mono-input" rows="4" /></div>
            <ul v-if="current.members.length" class="mlist">
              <li v-for="m in current.members" :key="`${m.namespace}/${m.slug}`"><span>{{ m.name ?? '（不存在）' }} <span class="mono hint">{{ m.namespace }}/{{ m.slug }}</span></span><StatusBadge v-if="m.status" type="item" :value="m.status" /></li>
            </ul>
            <p class="hint">成员必须是已存在的单个条目；上架前所有成员都要已上架。</p>
          </div>

          <div v-if="kind === 'skill' && current.item.variant === 'single'" class="stack versions">
            <div class="vh"><h3>版本</h3><span class="hint">撤回后前台不再提供下载</span></div>
            <div v-if="!current.versions.length" class="hint">还没有版本。上传一个符合规则的 ZIP 后才能上架。</div>
            <div v-for="v in current.versions" :key="v.id" class="ver" :class="{ yanked: v.yanked }">
              <span class="mono">{{ v.version }}</span>
              <span class="hint">{{ formatSize(v.size) }} · {{ v.fileCount }} 个文件 · {{ shortHash(v.manifestSha256, 6) }}</span>
              <span v-if="v.yanked" class="badge badge-bad">已撤回</span>
              <span v-else-if="v.version === current.item.version" class="badge badge-ok">最新</span>
              <button class="btn btn-sm push" type="button" @click="yank(v)">{{ v.yanked ? '恢复' : '撤回' }}</button>
            </div>
            <form class="upload" @submit.prevent="doUpload">
              <div class="row">
                <div class="field" style="max-width: 140px"><label for="uv">新版本号</label><input id="uv" v-model="upload.version" class="input mono-input" placeholder="1.0.0" required /></div>
                <div class="field"><label for="uf">ZIP 安装包</label><input id="uf" class="input" type="file" accept=".zip,application/zip" required @change="upload.file = ($event.target as HTMLInputElement).files?.[0] ?? null" /></div>
              </div>
              <div class="field"><label for="uc">更新说明</label><input id="uc" v-model="upload.changelog" class="input" /></div>
              <button class="btn btn-sm" type="submit" :disabled="busy || !upload.file || !upload.version">上传新版本</button>
              <span class="hint"> 服务端按桌面客户端的规则校验（≤512 个文件、12 MiB，根目录需有 SKILL.md）。</span>
            </form>
          </div>

          <div v-if="kind === 'skill' && current.item.variant === 'single' && !hasVersion && current.item.status !== 'published'" class="notice">技能需要至少一个未撤回的版本才能上架。</div>

          <div class="actions">
            <button class="btn btn-primary" type="button" :disabled="busy" @click="save">保存修改</button>
            <button v-for="a in statusActions" :key="a.to" class="btn" type="button" :disabled="a.disabled" @click="setStatus(a.to)">{{ a.label }}</button>
            <button class="btn btn-danger push" type="button" @click="remove">删除</button>
          </div>
        </div>
      </aside>
    </div>
  </div>
</template>

<style scoped>
.create { padding: 16px 20px; display: flex; flex-direction: column; gap: 12px; }
.grow { flex: 1 1 220px; }
.nm { display: flex; align-items: center; gap: 8px; font-weight: 500; }
.vt { font-size: 12px; font-weight: 500; background: #141414; color: #fff; border-radius: 5px; padding: 0 6px; }
.members { padding-top: 14px; border-top: 1px solid #f0f0ee; gap: 8px; }
.mlist { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 6px; }
.mlist li { display: flex; align-items: center; justify-content: space-between; gap: 8px; padding: 8px 12px; border-radius: 10px; background: var(--soft); font-size: 13px; }
.star { font-size: 12px; font-weight: 500; color: var(--warn-ink); background: var(--warn-bg); border-radius: 5px; padding: 0 6px; }
.muted { color: var(--ink-2); }
.head { display: flex; flex-wrap: wrap; align-items: center; gap: 8px 10px; }
.head h2 { font-size: 18px; font-weight: 700; overflow-wrap: anywhere; }
.kindtag { padding: 1px 8px; border-radius: 5px; font-size: 12px; background: #f0f0ee; color: var(--ink-2); }
.versions { padding-top: 14px; border-top: 1px solid #f0f0ee; gap: 10px; }
.vh { display: flex; align-items: baseline; justify-content: space-between; }
.vh h3 { font-size: 15px; font-weight: 700; }
.ver { display: flex; flex-wrap: wrap; align-items: center; gap: 4px 12px; padding: 8px 12px; border-radius: 10px; background: var(--soft); }
.ver.yanked { color: var(--ink-3); }
.push { margin-left: auto; }
.upload { display: flex; flex-direction: column; gap: 10px; padding: 12px; border: 2px dashed var(--line-2); border-radius: 12px; }
.actions { display: flex; flex-wrap: wrap; gap: 8px; padding-top: 14px; border-top: 1px solid #f0f0ee; }
</style>
