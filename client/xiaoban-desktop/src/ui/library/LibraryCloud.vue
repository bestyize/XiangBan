<script setup lang="ts">
import { useConfirm } from './confirm.ts';
import { computed, nextTick, ref, watch } from 'vue';
import { NButton, NInput, NModal } from 'naive-ui';
import { Copy, ExternalLink, KeyRound, Link2, Pencil, ShieldCheck, Trash2 } from 'lucide-vue-next';
import ChatNameDialog from '../components/ChatNameDialog.vue';
import { folderPath, folderTree, formatWhen, guessPlatform, PLATFORMS, searchText, type LibraryLink, type LibraryStore, type Platform } from './library-store.ts';

const props = defineProps<{ library: LibraryStore; query: string; desktop: boolean; openUrl: (url: string) => void }>();
const emit = defineEmits<{ notify: [message: string, type?: 'success' | 'error' | 'info'] }>();
const confirm = useConfirm();

const FILTERS: { label: string; match: (p: Platform) => boolean }[] = [
  { label: '全部', match: () => true },
  { label: '飞书', match: p => p === 'feishu' },
  { label: '腾讯文档', match: p => p === 'tencent' },
  { label: '百度网盘', match: p => p === 'baidu' },
  { label: '其他', match: p => !['feishu', 'tencent', 'baidu'].includes(p) },
];
const filter = ref('全部');
const folders = computed(() => props.library.data.folders);
const links = computed(() => props.library.data.links.filter(l => (FILTERS.find(f => f.label === filter.value)?.match ?? (() => true))(l.platform)
  && searchText(props.query, l.title, l.url, l.kind, PLATFORMS[l.platform]?.name)));
const info = (link: LibraryLink) => PLATFORMS[link.platform] ?? PLATFORMS.other;

/* ── 添加链接 ── */
const adding = ref(false), url = ref(''), title = ref(''), folder = ref('');
const urlInput = ref<InstanceType<typeof NInput> | null>(null);
const guessed = computed(() => guessPlatform(url.value));
const folderOptions = computed(() => [{ id: '', name: '未分组' }, ...folderTree(props.library.data).filter(r => r.folder).map(r => ({ id: r.id, name: folderPath(folders.value, r.id) }))]);
function openAdd() { adding.value = true; url.value = ''; title.value = ''; folder.value = ''; void nextTick(() => urlInput.value?.focus()); }
watch(adding, open => { if (open) void nextTick(() => urlInput.value?.focus()); });
async function add() {
  if (!guessed.value) return;
  const ok = await props.library.addLink(url.value, title.value, folder.value || null);
  if (ok) { adding.value = false; emit('notify', '已添加到云文档'); }
}
const CONNECT: Platform[] = ['feishu', 'tencent', 'baidu'];

/* ── Cards ── */
const renaming = ref<LibraryLink | null>(null);
async function copy(text: string, what: string) {
  try { await navigator.clipboard.writeText(text); emit('notify', `已复制${what}`); } catch { emit('notify', '复制失败，请重试', 'error'); }
}
function remove(link: LibraryLink) {
  confirm({ title: `移除「${link.title}」？`, content: '只从资料库移除这个链接，云端的文档不受影响。', positiveText: '移除', negativeText: '取消', onPositiveClick: () => props.library.remove([link.id]) });
}
async function rename(name: string) { if (renaming.value) await props.library.renameLink(renaming.value.id, name); renaming.value = null; }
defineExpose({ openAdd });
</script>

<template>
  <div class="cloud" data-testid="library-cloud">
    <div class="lib-toolbar">
      <div class="lib-chips" aria-label="按平台筛选">
        <button v-for="f in FILTERS" :key="f.label" type="button" :class="{ active: filter === f.label }" :aria-pressed="filter === f.label" @click="filter = f.label">{{ f.label }}</button>
      </div>
      <span class="lib-muted right">只保存链接；连接账号后小伴才能读内容</span>
    </div>
    <div class="lib-scroll grid-wrap">
      <div v-if="!library.data.links.length" class="xb-empty">
        <span class="xb-empty-icon"><Link2 :size="24" /></span>
        <h3>把云文档的链接放进来</h3>
        <p>支持飞书、腾讯文档、百度网盘，以及石墨、语雀、WPS 的链接。网盘分享链接的提取码会一起保存。</p>
        <button type="button" class="xb-btn-primary" :disabled="!desktop" @click="openAdd">添加链接</button>
      </div>
      <p v-else-if="!links.length" class="lib-muted nothing">没有找到相关链接</p>
      <div v-else class="grid">
        <article v-for="link in links" :key="link.id" class="lib-card card" :data-testid="`library-link-${link.id}`">
          <div class="card-head">
            <span class="lib-badge badge" :style="{ background: info(link).hue }">{{ info(link).letter }}</span>
            <span class="card-title"><b :title="link.title">{{ link.title }}</b><span class="lib-muted">{{ info(link).name }} · {{ link.kind }}</span></span>
          </div>
          <span class="lib-muted note">{{ folderPath(folders, link.folder) }} · {{ formatWhen(link.addedMs) }}添加</span>
          <span v-if="link.code" class="code"><KeyRound :size="12" />提取码 <b>{{ link.code }}</b><button type="button" class="lib-link" @click="copy(link.code!, '提取码')">复制</button></span>
          <div class="card-foot">
            <span class="lib-pill">仅保存链接</span>
            <button type="button" class="lib-icon-btn push" aria-label="重命名" @click="renaming = link"><Pencil :size="14" /></button>
            <button type="button" class="lib-icon-btn" aria-label="复制链接" @click="copy(link.url, '链接')"><Copy :size="14" /></button>
            <button type="button" class="lib-icon-btn" aria-label="在浏览器打开" :data-testid="`library-open-${link.id}`" @click="openUrl(link.url)"><ExternalLink :size="14" /></button>
            <button type="button" class="lib-icon-btn" aria-label="移除链接" @click="remove(link)"><Trash2 :size="14" /></button>
          </div>
        </article>
      </div>
    </div>

    <NModal :show="adding" preset="card" title="添加云文档链接" style="max-width: 480px; width: 92vw;" data-testid="library-link-dialog" @update:show="value => { if (!value) adding = false; }">
      <form class="lib-form" @submit.prevent="add">
        <label>链接<NInput ref="urlInput" v-model:value="url" placeholder="粘贴飞书、腾讯文档、百度网盘等链接" clearable data-testid="library-link-url" /></label>
        <div v-if="url.trim()" class="guess" :class="{ bad: !guessed }" data-testid="library-link-guess">
          <template v-if="guessed"><span class="lib-badge badge sm" :style="{ background: PLATFORMS[guessed].hue }">{{ PLATFORMS[guessed].letter }}</span>识别为 {{ PLATFORMS[guessed].name }}</template>
          <template v-else>这不像一个网址，请粘贴以 http:// 或 https:// 开头的链接</template>
        </div>
        <label>标题<NInput v-model:value="title" placeholder="可不填，之后也能改" :maxlength="120" /></label>
        <label>放到<select v-model="folder" class="folder-select" aria-label="放到文件夹"><option v-for="o in folderOptions" :key="o.id" :value="o.id">{{ o.name }}</option></select></label>
        <div class="connect">
          <span class="lib-section"><ShieldCheck :size="14" />连接账号，小伴才能读内容</span>
          <span class="lib-muted">只保存链接时，小伴知道有这份文档，但看不到内容。连接后可以在回答里引用，凭证由小伴账号服务保管。</span>
          <div v-for="p in CONNECT" :key="p" class="connect-row"><span class="lib-badge badge sm" :style="{ background: PLATFORMS[p].hue }">{{ PLATFORMS[p].letter }}</span>{{ PLATFORMS[p].name }}<span class="lib-pill soon">即将支持</span></div>
        </div>
        <div class="lib-form-actions">
          <NButton @click="adding = false">取消</NButton>
          <NButton type="primary" attr-type="submit" :loading="library.busy" :disabled="!guessed" data-testid="library-link-add">添加</NButton>
        </div>
      </form>
    </NModal>
    <ChatNameDialog :show="!!renaming" title="重命名链接" placeholder="标题" :initial="renaming?.title ?? ''" :max-length="120" confirm-text="确定" :busy="library.busy" @close="renaming = null" @submit="rename" />
  </div>
</template>

<style scoped>
.cloud{flex:1;min-height:0;display:flex;flex-direction:column;margin-top:14px}
.right{margin-left:auto}
.grid-wrap{margin:0 -28px;padding:14px 28px 24px}
.grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(250px,1fr));gap:14px}
.card{padding:14px;display:flex;flex-direction:column;gap:10px;min-width:0}
.card-head{display:flex;align-items:center;gap:10px;min-width:0}
.badge{width:32px;height:32px;border-radius:9px;font-size:12.5px}
.badge.sm{width:22px;height:22px;border-radius:6px;font-size:10.5px}
.card-title{display:flex;flex-direction:column;gap:1px;min-width:0}
.card-title b{font-size:13.5px;font-weight:600;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.note{white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.code{display:flex;align-items:center;gap:6px;font-size:12px;color:var(--ui-text-secondary)}
.code b{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;color:var(--ui-text-primary)}
.card-foot{display:flex;align-items:center;gap:4px;padding-top:10px;border-top:1px solid var(--ui-line)}
.push{margin-left:auto}
.nothing{padding:64px 0;text-align:center;font-size:13px}
.guess{display:flex;align-items:center;gap:8px;margin-top:-6px;font-size:12.5px;color:var(--ui-text-secondary)}
.guess.bad{color:var(--ui-danger)}
.folder-select{height:34px;border:0;border-radius:8px;padding:0 10px;background:var(--ui-control);color:var(--ui-text-primary);font:inherit;font-size:13px}
.connect{display:flex;flex-direction:column;gap:8px;padding:12px;border-radius:10px;background:var(--ui-surface-sunken,var(--ui-control))}
.connect .lib-muted{line-height:1.6}
.connect-row{display:flex;align-items:center;gap:8px;font-size:13px;color:var(--ui-text-primary)}
.soon{margin-left:auto}
</style>
