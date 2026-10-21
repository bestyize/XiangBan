<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { NAlert, NModal, NSkeleton } from 'naive-ui';
import { Puzzle, Plug, ArrowRight, Download, ChevronLeft, ChevronRight } from 'lucide-vue-next';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import type { SkillReview } from './catalog.ts';

interface Entry { slug: string; name: string; description: string; category?: string; iconUrl?: string | null; skillCount?: number }
interface Page { results: Entry[]; page: number; total: number; hasMore: boolean }
interface Pack { slug: string; name: string; description: string; entrySkill: string; ready: boolean; sha256: string; dependencies: { slug: string; namespace?: string; version: string; sha256: string }[] }
interface PackReview extends Pack { content: string; skills: SkillReview[] }
interface Connection { id: string; name: string; connected: boolean; enabled: boolean; toolCount: number; error?: string }
interface ConnectorDetail { readme: string; configs: { name: string; config: Record<string, unknown> }[] }
const props = defineProps<{ kind: 'experts' | 'connectors'; query: string; category: string; store: ControlStore }>();
const data = ref<Page>({ results: [], page: 1, total: 0, hasMore: false });
const page = ref(1), loading = ref(false), error = ref(''), localError = ref(''), installedOnly = ref(false);
const packs = ref<Pack[]>([]), connections = ref<Connection[]>([]);
const selected = ref<Entry | null>(null), packReview = ref<PackReview | null>(null), connectorDetail = ref<ConnectorDetail | null>(null);
const detailLoading = ref(false), detailError = ref(''), confirmed = ref(false), busy = ref(false), removing = ref(false);
const configText = ref(''), configIndex = ref(0);
const brokenIcons = ref<Record<string, boolean>>({});
const icon = (entry: Entry) => !brokenIcons.value[entry.slug] && entry.iconUrl?.startsWith('https://') ? entry.iconUrl : null;
let requestId = 0, detailId = 0, localId = 0, timer: ReturnType<typeof setTimeout> | undefined;
const pack = (slug: string) => packs.value.find(p => p.slug === slug);
const connection = (slug: string) => connections.value.find(c => c.id === slug);
const scene: Record<string, string> = { 开发: 'tech', 办公: 'business', 创作: 'creative', 金融: 'finance', 生活: 'lifestyle', 学习: 'education' };
const entries = computed(() => {
  if (!installedOnly.value) return data.value.results;
  const local: Entry[] = props.kind === 'experts' ? packs.value : connections.value.map(c => ({ slug: c.id, name: c.name, description: c.error || (c.toolCount + ' 个工具') }));
  const q = props.query.trim().toLowerCase();
  return local.filter(e => (e.name + e.description).toLowerCase().includes(q));
});
async function refreshLocal() {
  const id = ++localId;
  if (!props.store.desktop) return;
  const kind = props.kind;
  try {
    if (kind === 'experts') {
      const result = await invoke<Pack[]>('ai_store_experts_installed');
      if (id === localId) packs.value = result;
    } else {
      const result = await invoke<Connection[]>('ai_store_connector_status');
      if (id === localId) connections.value = result;
    }
    if (id === localId) localError.value = '';
  } catch (e) { if (id === localId) localError.value = '无法读取本地状态：' + String(e); }
}
async function refresh() {
  clearTimeout(timer);
  const id = ++requestId;
  if (!props.store.desktop || installedOnly.value || props.category === '本机') { loading.value = false; return; }
  loading.value = true; error.value = '';
  data.value = { results: [], page: page.value, total: 0, hasMore: false };
  try {
    const result = await invoke<Page>('ai_store_catalog', {
      kind: props.kind, keyword: props.query.trim().slice(0, 80), page: page.value,
      category: props.category === '全部' ? '' : props.kind === 'experts' ? (scene[props.category] || props.category) : props.category,
    });
    if (id === requestId) data.value = result;
  } catch (e) { if (id === requestId) error.value = '暂时无法加载 SkillHub：' + String(e); }
  finally { if (id === requestId) loading.value = false; }
}
watch(() => [props.kind, props.query, props.category, installedOnly.value], () => {
  ++requestId; page.value = 1; clearTimeout(timer); error.value = '';
  data.value = { results: [], page: 1, total: 0, hasMore: false };
  loading.value = props.store.desktop && !installedOnly.value && props.category !== '本机';
  timer = setTimeout(() => void refresh(), 250);
}, { immediate: true });
watch(page, () => { void refresh(); });
watch(() => props.kind, () => {
  ++detailId; selected.value = null; configText.value = ''; installedOnly.value = false;
  void refreshLocal();
}, { immediate: true });
onBeforeUnmount(() => { clearTimeout(timer); ++requestId; ++detailId; ++localId; configText.value = ''; });
function close() { if (!busy.value) { selected.value = null; configText.value = ''; ++detailId; } }
async function open(entry: Entry) {
  if (busy.value) return;
  selected.value = entry; packReview.value = null; connectorDetail.value = null;
  confirmed.value = false; removing.value = false; configText.value = ''; detailError.value = '';
  const id = ++detailId, kind = props.kind;
  if (kind === 'experts' && pack(entry.slug)) { detailLoading.value = false; return; }
  detailLoading.value = true;
  try {
    if (kind === 'experts') {
      const review = await invoke<PackReview>('ai_store_expert_review', { slug: entry.slug });
      if (id === detailId) packReview.value = review;
    } else {
      const detail = await invoke<ConnectorDetail>('ai_store_connector_detail', { slug: entry.slug });
      if (id === detailId) { connectorDetail.value = detail; configIndex.value = 0; chooseConfig(); }
    }
  } catch (e) { if (id === detailId) detailError.value = String(e); }
  finally { if (id === detailId) detailLoading.value = false; }
}
function chooseConfig() {
  configText.value = JSON.stringify(connectorDetail.value?.configs[configIndex.value]?.config || { url: '' }, null, 2);
  confirmed.value = false;
}
async function installPack() {
  const review = packReview.value;
  if (!review || !confirmed.value || busy.value || localError.value) return;
  busy.value = true;
  try {
    await invoke('ai_store_expert_install', { slug: review.slug, sha256: review.sha256 });
    await refreshLocal(); selected.value = null; props.store.notify('专家包已安装，可按工作流开始任务');
  } catch (e) { detailError.value = String(e); }
  finally { busy.value = false; }
}
async function removePack() {
  if (!selected.value || busy.value) return;
  if (!removing.value) { removing.value = true; return; }
  busy.value = true;
  try {
    await invoke('ai_store_expert_remove', { slug: selected.value.slug });
    await refreshLocal(); selected.value = null; props.store.notify('已移除专家工作流，共用技能保留');
  } catch (e) { detailError.value = String(e); }
  finally { busy.value = false; }
}
async function usePack(entry: Entry) {
  const installed = pack(entry.slug);
  if (!installed?.ready || busy.value || localError.value) return;
  busy.value = true;
  try {
    await props.store.chat.select(null);
    props.store.creationDraft = '请先用 load_skill 加载已安装的专家工作流「' + installed.entrySkill + '」，再按需加载其中的依赖技能，帮助我完成以下任务：';
    props.store.navigate('chat'); selected.value = null;
  } catch (e) { props.store.notify(e, 'error'); }
  finally { busy.value = false; }
}
async function connect() {
  if (!selected.value || !confirmed.value || busy.value) return;
  busy.value = true; detailError.value = '';
  try {
    const config: unknown = JSON.parse(configText.value);
    await invoke('ai_store_connector_connect', { id: selected.value.slug, name: selected.value.name, config, confirmed: true });
    await refreshLocal(); configText.value = ''; selected.value = null;
    props.store.notify('连接成功，工具将在新任务中可用');
  } catch (e) { detailError.value = String(e); }
  finally { busy.value = false; }
}
async function connectionAction(action: 'disconnect' | 'reconnect') {
  if (!selected.value || busy.value) return;
  busy.value = true; detailError.value = '';
  try {
    await invoke('ai_store_connector_' + action, { id: selected.value.slug });
    await refreshLocal(); selected.value = null; configText.value = '';
    props.store.notify(action === 'disconnect' ? '已断开连接，工具停止可用' : '已重新连接');
  } catch (e) { detailError.value = String(e); }
  finally { busy.value = false; }
}
defineExpose({ refresh, refreshLocal });
</script>

<template>
  <div class="registry">
    <div class="registry-heading"><h2>{{ kind === 'experts' ? '专家包' : 'MCP 连接器' }}</h2><span>来自 SkillHub</span><button class="small-action installed-filter" :class="{ selected: installedOnly }" :aria-pressed="installedOnly" @click="installedOnly = !installedOnly">{{ kind === 'experts' ? '已安装' : '已配置' }}</button></div>
    <NAlert v-if="!store.desktop" type="info" :show-icon="false">请在小伴桌面客户端中浏览和安装。</NAlert>
    <NAlert v-if="error" type="error" :show-icon="false">{{ error }} <button class="text-action" @click="refresh">重试</button></NAlert>
    <NAlert v-if="localError" type="warning" :show-icon="false">{{ localError }} <button class="text-action" @click="refreshLocal">重试</button></NAlert>
    <div v-if="loading" class="store-grid" aria-busy="true"><NSkeleton v-for="n in 8" :key="n" height="200px" /></div>
    <div v-else class="store-grid">
      <slot name="prefix" />
      <article v-for="entry in entries" :key="entry.slug" class="store-card" :class="kind === 'experts' ? 'expert-card' : 'connector-card'">
        <div class="card-top"><div class="entry-icon"><img v-if="icon(entry)" :src="icon(entry)!" alt="" referrerpolicy="no-referrer" @error="brokenIcons[entry.slug] = true" /><Puzzle v-else-if="kind === 'experts'" :size="25" /><Plug v-else :size="25" /></div><span class="connection-status">{{ kind === 'experts' ? (pack(entry.slug) ? (pack(entry.slug)?.ready ? '已安装' : '依赖缺失') : (entry.skillCount || 0) + ' 个技能') : (connection(entry.slug)?.connected ? '已连接 · ' + connection(entry.slug)?.toolCount + ' 个工具' : connection(entry.slug) ? '未连接' : '未配置') }}</span></div>
        <h3>{{ entry.name }}</h3><p class="entry-desc">{{ entry.description }}</p>
        <div class="card-actions"><span class="store-tag">{{ entry.category || 'SkillHub' }}</span><button class="small-action" :disabled="busy || !store.desktop || !!localError" @click="kind === 'experts' && pack(entry.slug)?.ready ? usePack(entry) : open(entry)">{{ kind === 'experts' ? (pack(entry.slug)?.ready ? '使用专家' : '查看与安装') : connection(entry.slug) ? '管理' : '连接' }}<ArrowRight :size="12" /></button><button v-if="kind === 'experts' && pack(entry.slug)" class="text-action" aria-label="管理专家包" @click="open(entry)">管理</button></div>
      </article>
    </div>
    <div v-if="!loading && !error && store.desktop && !entries.length && category !== '本机'" class="store-empty">没有找到匹配的{{ kind === 'experts' ? '专家包' : '连接器' }}</div>
    <div v-if="!installedOnly && !loading && !error && (page > 1 || data.hasMore)" class="pagination"><button :disabled="page <= 1" aria-label="上一页" @click="page--"><ChevronLeft :size="16" /></button><span>第 {{ page }} 页</span><button :disabled="!data.hasMore" aria-label="下一页" @click="page++"><ChevronRight :size="16" /></button></div>
    <NModal :show="!!selected" preset="card" :title="selected?.name" class="registry-dialog" :closable="!busy" :mask-closable="!busy" style="width:720px;max-width:calc(100vw - 36px)" @update:show="!$event && close()">
      <template v-if="selected">
        <p>{{ selected.description }}</p>
        <div v-if="detailLoading" class="review-loading">正在读取说明与依赖…</div>
        <NAlert v-if="detailError" type="error" :show-icon="false">{{ detailError }} <button v-if="!busy" class="text-action" @click="open(selected)">重新读取</button></NAlert>
        <template v-if="kind === 'experts'">
          <template v-if="pack(selected.slug)">
            <p>{{ pack(selected.slug)?.ready ? '工作流与依赖技能已就绪。' : '依赖缺失或版本发生变化，请移除工作流后重新安装。' }}</p>
            <ul><li v-for="dep in pack(selected.slug)?.dependencies" :key="dep.slug">{{ dep.namespace ? '@' + dep.namespace + '/' : '' }}{{ dep.slug }} · {{ dep.version }}</li></ul>
            <p class="detail-note">这是技能工作流。依赖服务、凭据及本机环境仍需满足；不代表拥有独立成长的多智能体专家团。</p>
            <div class="dialog-actions"><button class="store-secondary" :disabled="busy" @click="removePack">{{ removing ? '确认移除（保留共用技能）' : '移除工作流' }}</button><button class="store-primary" :disabled="busy || !pack(selected.slug)?.ready || !!localError" @click="usePack(selected)">使用专家 <ArrowRight :size="14" /></button></div>
          </template>
          <template v-else-if="packReview">
            <h4>工作流</h4><pre class="review-content">{{ packReview.content }}</pre>
            <h4>依赖技能与固定版本</h4>
            <details v-for="(item, i) in packReview.skills" :key="item.slug" class="dependency-review"><summary>{{ packReview.dependencies[i]?.namespace ? '@' + packReview.dependencies[i]?.namespace + '/' : '' }}{{ item.slug }} · {{ item.manifest.version }} · {{ item.manifest.fileCount }} 个文件</summary><pre>{{ item.content }}</pre><ul><li v-for="file in item.manifest.files" :key="file.path">{{ file.path }}<small>SHA256 {{ file.sha256 }}</small></li></ul><details v-for="script in item.scripts" :key="script.path"><summary>{{ script.path }}{{ script.previewTruncated ? '（预览已截断）' : '' }}</summary><pre>{{ script.preview || '暂无脚本文本预览' }}</pre></details></details>
            <p class="detail-note">安装会校验整个工作流和各依赖版本，不执行脚本或覆盖不同版本。脚本及外部工具使用时另行确认。</p>
            <label class="review-confirm"><input v-model="confirmed" type="checkbox" :disabled="busy" />我已审查工作流、依赖来源、文件和脚本，同意安装。</label>
            <div class="dialog-actions"><button class="store-primary" :disabled="busy || !confirmed || !!localError" @click="installPack">{{ busy ? '正在安装…' : '安装专家包' }}<Download :size="14" /></button></div>
          </template>
        </template>
        <template v-else>
          <div v-if="connection(selected.slug)" class="connection-summary"><p>{{ connection(selected.slug)?.connected ? '已连接，发现 ' + connection(selected.slug)?.toolCount + ' 个工具。' : '已保存配置，可以重新连接。' }}</p><div class="dialog-actions"><button v-if="connection(selected.slug)?.connected" class="store-secondary" :disabled="busy" @click="connectionAction('disconnect')">断开连接</button><button v-else class="store-secondary" :disabled="busy" @click="connectionAction('reconnect')">使用已保存配置重连</button></div></div>
          <template v-if="connectorDetail">
          <details><summary>服务说明与凭据获取方式</summary><pre class="review-content">{{ connectorDetail.readme }}</pre></details>
          <h4>{{ connection(selected.slug) ? '重新配置' : '连接配置' }}</h4>
          <select v-if="connectorDetail.configs.length > 1" v-model="configIndex" aria-label="选择连接方式" @change="chooseConfig"><option v-for="(c, i) in connectorDetail.configs" :key="i" :value="i">{{ c.name }} · {{ c.config.type || c.config.transport || (c.config.command ? 'stdio' : 'HTTP') }}</option></select>
          <p>填写服务所需的地址或凭据，支持标准 MCP 配置 JSON。</p>
          <textarea v-model="configText" aria-label="MCP 连接配置" spellcheck="false" :disabled="busy" @input="confirmed = false" />
          <p class="detail-note">支持 stdio 和 Streamable HTTP，暂不支持旧式 SSE。本地服务会启动配置中的命令（如 npx 可能下载依赖）；凭据保存在本机，不放进 Agent 提示词。</p>
          <label class="review-confirm"><input v-model="confirmed" type="checkbox" :disabled="busy" />我已检查服务来源与配置，允许小伴连接此服务。</label>
          <div class="dialog-actions"><button class="store-primary" :disabled="busy || !confirmed || !configText.trim()" @click="connect">{{ busy ? '正在连接…' : '保存并连接' }}<Plug :size="14" /></button></div>
          </template>
        </template>
      </template>
    </NModal>
  </div>
</template>

<style scoped>
.registry-heading{display:flex;align-items:center;gap:10px;margin-bottom:20px}.registry-heading h2{font-size:15px;margin:0}.registry-heading>span{font-size:11px;color:var(--ui-text-tertiary)}.registry-heading>.small-action{margin-left:auto}
.store-grid{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:15px}.store-card{display:flex;flex-direction:column;min-width:0;min-height:215px;padding:17px;border:1px solid var(--ui-border);border-radius:13px;background:var(--ui-surface)}.entry-icon img{width:100%;height:100%;object-fit:cover;border-radius:12px}.card-top{display:flex;align-items:center;justify-content:space-between;gap:8px}.entry-icon{display:grid;place-items:center;width:45px;height:45px;border-radius:12px;background:var(--ui-primary-soft);color:var(--ui-primary)}h3{font-size:13px;margin:15px 0 8px}.connection-status,.store-tag{font-size:10px;color:var(--ui-text-tertiary);padding:5px 7px;border-radius:6px;background:var(--ui-surface-alt)}.entry-desc{font-size:11px;line-height:1.8;color:var(--ui-text-secondary);display:-webkit-box;-webkit-box-orient:vertical;-webkit-line-clamp:3;overflow:hidden;margin:0 0 17px}.card-actions{display:flex;align-items:center;gap:8px;margin-top:auto}.card-actions>.small-action{margin-left:auto}
button{font:inherit;cursor:pointer}button:disabled{opacity:.45;cursor:not-allowed}.small-action,.store-secondary{display:inline-flex;align-items:center;justify-content:center;gap:5px;border:1px solid var(--ui-border);border-radius:7px;background:var(--ui-surface);color:var(--ui-text-primary);padding:7px 9px;font-size:11px}.small-action.selected{color:var(--ui-primary);background:var(--ui-primary-soft)}.store-primary{display:inline-flex;align-items:center;justify-content:center;gap:7px;background:var(--ui-primary);color:white;border:0;border-radius:8px;padding:9px 14px;font-size:12px}.text-action{color:var(--ui-primary);background:none;border:0;font-size:11px}.store-empty{padding:55px 10px;text-align:center;color:var(--ui-text-tertiary);font-size:12px}.pagination{display:flex;align-items:center;justify-content:center;gap:15px;margin:25px}.pagination button{border:0;background:var(--ui-surface-alt);border-radius:6px;padding:7px}.pagination span{font-size:11px}
.registry-dialog p,.registry-dialog li{font-size:12px;line-height:1.8;color:var(--ui-text-secondary)}.registry-dialog h4{font-size:13px;margin:20px 0 10px}.registry-dialog details{font-size:12px;margin:12px 0}.registry-dialog summary{cursor:pointer;overflow-wrap:anywhere}.review-content,.dependency-review pre{white-space:pre-wrap;overflow-wrap:anywhere;max-height:260px;overflow:auto;padding:12px;background:var(--ui-surface-alt);border-radius:8px;font-size:11px;line-height:1.7}.dependency-review{border-bottom:1px solid var(--ui-border);padding:10px 0}.dependency-review small{display:block;font-size:10px;overflow-wrap:anywhere}.detail-note{padding:12px;background:var(--ui-surface-alt);border-radius:9px}.review-confirm{display:flex;align-items:flex-start;gap:8px;font-size:12px;line-height:1.8}.review-confirm input{margin-top:5px;accent-color:var(--ui-primary)}.dialog-actions{display:flex;gap:9px;justify-content:flex-end;margin-top:20px}.review-loading{padding:20px 0;font-size:12px}textarea{box-sizing:border-box;width:100%;height:200px;resize:vertical;background:var(--ui-surface-alt);color:var(--ui-text-primary);border:1px solid var(--ui-border);border-radius:9px;font:11px/1.7 monospace;padding:12px}select{width:100%;padding:8px;border:1px solid var(--ui-border);background:var(--ui-surface);color:var(--ui-text-primary);border-radius:7px}
@container(max-width:1050px){.store-grid{grid-template-columns:repeat(3,minmax(0,1fr))}}@container(max-width:780px){.store-grid{grid-template-columns:repeat(2,minmax(0,1fr))}}@container(max-width:480px){.store-grid{grid-template-columns:1fr}}
</style>
