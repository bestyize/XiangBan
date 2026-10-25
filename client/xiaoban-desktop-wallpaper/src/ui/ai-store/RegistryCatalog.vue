<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { NAlert, NModal, NSkeleton } from 'naive-ui';
import { Plug, ArrowRight, Download, Check, Plus, Sparkles, ChevronLeft, ChevronRight } from 'lucide-vue-next';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import type { SkillReview } from './catalog.ts';
import ConnectorConfig from './ConnectorConfig.vue';
import MarketConnectorDetail from './MarketConnectorDetail.vue';
import { connectorAuth, type InstalledConnector, type MarketConnector } from './connector-plugins.ts';
import type { UpdateInfo } from './market.ts';
import { categoryIcon, hueStyle } from './hues.ts';
import './store-cards.css';

interface Entry { slug: string; name: string; description: string; category?: string; iconUrl?: string | null; skillCount?: number; namespace?: string; version?: string; connector?: MarketConnector }
interface Page { results: Entry[]; page: number; total: number; hasMore: boolean }
interface Pack { slug: string; name: string; description: string; entrySkill: string; ready: boolean; sha256: string; dependencies: { slug: string; namespace?: string; version: string; sha256: string }[] }
interface PackReview extends Pack { content: string; skills: SkillReview[] }
interface Connection { id: string; name: string; connected: boolean; enabled: boolean; toolCount: number; error?: string }
interface ConnectorDetail { readme: string; configs: { name: string; config: Record<string, unknown> }[]; connector?: MarketConnector; version?: string }
interface ConnectionResult { status?:string; connected?:boolean; flowId?:string; authorizationUrl?:string }
const props = withDefaults(defineProps<{ kind: 'experts' | 'connectors'; query: string; category: string; store: ControlStore; installedOnly?: boolean; source?: 'market' | 'hub'; updates?: Record<string, UpdateInfo> }>(), { installedOnly: false, source: 'hub' });
const emit = defineEmits<{ unavailable: [message: string]; installed: []; changed: []; center: [plugin: string] }>();
const fromMarket = () => props.source === 'market' && props.kind === 'connectors';
const data = ref<Page>({ results: [], page: 1, total: 0, hasMore: false });
const page = ref(1), loading = ref(false), error = ref(''), localError = ref('');
const installedOnly = computed(() => props.installedOnly);
const installedCount = computed(() => props.kind === 'experts' ? packs.value.length : connections.value.length);
const packs = ref<Pack[]>([]), connections = ref<Connection[]>([]);
const plugins = ref<InstalledConnector[]>([]), pluginError = ref('');
const pluginReady = ref(false);
const plugin = (slug: string) => plugins.value.find(p => p.plugin === slug);
const isPlugin = (entry: Entry) => fromMarket() && !!entry.connector;
const authLabel = (entry: Entry) => entry.connector?.authTypes.map(a => connectorAuth[a]).join('、');
const pluginUpdate = (entry: Entry) => {
  const info = props.updates?.[entry.slug];
  return plugin(entry.slug) && info?.hasUpdate && ['connector', 'connectors'].includes(info.kind) && info.namespace === entry.namespace ? info.latest : undefined;
};
const selected = ref<Entry | null>(null), packReview = ref<PackReview | null>(null), connectorDetail = ref<ConnectorDetail | null>(null);
const marketPlugin = computed(() => fromMarket() ? connectorDetail.value?.connector || selected.value?.connector : undefined);
const detailLoading = ref(false), detailError = ref(''), confirmed = ref(false), busy = ref(false), removing = ref(false);
const configText = ref(''), configIndex = ref(0);
const configValid=ref(false), authorization=ref<{flowId:string;authorizationUrl:string}|null>(null);
let authTimer:ReturnType<typeof setTimeout>|undefined;
let pollingFlow:string|undefined;
watch(configText,()=>{confirmed.value=false;});
function abandonAuthorization(){clearTimeout(authTimer);const flow=authorization.value;authorization.value=null;if(flow)void invoke('ai_store_connector_auth_cancel',{flowId:flow.flowId}).catch(()=>{});}
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
  pluginReady.value = false;
  try {
    if (kind === 'experts') {
      const result = await invoke<Pack[]>('ai_store_experts_installed');
      if (id === localId) packs.value = result;
    } else {
      const [legacy, installed] = await Promise.allSettled([
        invoke<Connection[]>('ai_store_connector_status'),
        fromMarket() ? invoke<InstalledConnector[]>('ai_connector_installed') : Promise.resolve([]),
      ]);
      if (id !== localId) return;
      if (installed.status === 'fulfilled') { plugins.value = installed.value; pluginError.value = ''; pluginReady.value = true; }
      else pluginError.value = '无法读取已安装连接器：' + String(installed.reason);
      if (legacy.status === 'rejected') throw legacy.reason;
      connections.value = legacy.value;
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
    const keyword = props.query.trim().slice(0, 80), category = props.category === '全部' ? '' : props.kind === 'experts' ? (scene[props.category] || props.category) : props.category;
    const result = fromMarket()
      ? await invoke<Page>('ai_market_browse', { kind: 'connectors', keyword, page: page.value, category, source: '', sortBy: '', variant: '' })
      : await invoke<Page>('ai_store_catalog', { kind: props.kind, keyword, page: page.value, category });
    if (id === requestId) data.value = result;
  } catch (e) {
    if (id === requestId) {
      error.value = (fromMarket() ? '暂时无法加载小伴市场：' : '暂时无法加载 SkillHub：') + String(e);
      if (fromMarket()) emit('unavailable', String(e));
    }
  }
  finally { if (id === requestId) loading.value = false; }
}
watch(() => [props.kind, props.query, props.category, installedOnly.value, props.source], () => {
  ++requestId; page.value = 1; clearTimeout(timer); error.value = '';
  data.value = { results: [], page: 1, total: 0, hasMore: false };
  loading.value = props.store.desktop && !installedOnly.value && props.category !== '本机';
  timer = setTimeout(() => void refresh(), 250);
}, { immediate: true });
watch(page, () => { void refresh(); });
watch(() => [props.kind, props.source], () => {
  abandonAuthorization();
  ++detailId; selected.value = null; configText.value = '';
  void refreshLocal();
}, { immediate: true });
onBeforeUnmount(() => { abandonAuthorization();clearTimeout(timer); ++requestId; ++detailId; ++localId; configText.value = ''; });
async function close() { if (!busy.value) {if(authorization.value&&!await cancelAuthorization())return;selected.value = null; configText.value = ''; ++detailId; } }
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
      const detail = fromMarket() && entry.namespace
        ? await marketConnector(entry)
        : await invoke<ConnectorDetail>('ai_store_connector_detail', { slug: entry.slug });
      if (id === detailId) { connectorDetail.value = detail; configIndex.value = 0; if (!detail.connector) chooseConfig(); }
    }
  } catch (e) { if (id === detailId) detailError.value = String(e); }
  finally { if (id === detailId) detailLoading.value = false; }
}
async function marketConnector(entry: Entry): Promise<ConnectorDetail> {
  const detail = await invoke<{ item: { description?: string; version?: string; connector?: MarketConnector; metadata?: { configs?: ConnectorDetail['configs'] } } }>('ai_market_detail', { kind: 'connectors', namespace: entry.namespace, slug: entry.slug });
  return { readme: detail.item.description || entry.description, configs: detail.item.metadata?.configs ?? [], connector: detail.item.connector || entry.connector, version: entry.version || detail.item.version };
}
function pluginInstalled() {
  void refreshLocal(); emit('installed');
}
function pluginConnectionChanged() { void refreshLocal(); emit('changed'); }
function goToCenter() {
  if (!selected.value || busy.value) return;
  const slug = selected.value.slug;
  void close(); emit('center', slug);
}
function chooseConfig() {
  configText.value = JSON.stringify(connectorDetail.value?.configs[configIndex.value]?.config || { url: '' }, null, 2);
  confirmed.value = false;
}
async function cancelAuthorization(){
  const flow=authorization.value;if(!flow)return true;clearTimeout(authTimer);busy.value=true;
  try{await invoke('ai_store_connector_auth_cancel',{flowId:flow.flowId});authorization.value=null;chooseConfig();return true;}
  catch(e){detailError.value='取消授权失败：'+String(e);return false;}
  finally{busy.value=false;}
}
async function openAuthorizationPage(){
  if(!authorization.value)return;
  try{await invoke('open_download_url',{url:authorization.value.authorizationUrl});}
  catch(e){detailError.value='无法打开浏览器，请重试：'+String(e);}
}
async function pollAuthorization(){
  clearTimeout(authTimer);
  const flow=authorization.value;if(!flow||pollingFlow===flow.flowId)return;
  pollingFlow=flow.flowId;
  try{
    const result=await invoke<{status:string;error?:string}>('ai_store_connector_auth_status',{flowId:flow.flowId});
    if(authorization.value?.flowId!==flow.flowId)return;
    if(result.status==='connected'){authorization.value=null;await refreshLocal();selected.value=null;configText.value='';props.store.notify('授权完成，连接成功');}
    else if(result.status==='error'||result.status==='cancelled'){authorization.value=null;chooseConfig();detailError.value=result.error||'授权已取消，可重新连接。';}
    else authTimer=setTimeout(()=>void pollAuthorization(),1000);
  }catch(e){if(authorization.value?.flowId===flow.flowId)detailError.value='检查授权状态失败：'+String(e);}
  finally{if(pollingFlow===flow.flowId)pollingFlow=undefined;}
}
async function handleConnectionResult(result:ConnectionResult|undefined,context:number){
  if(result?.status==='authorization_required'&&result.flowId&&result.authorizationUrl){
    if(context!==detailId){await invoke('ai_store_connector_auth_cancel',{flowId:result.flowId});return;}
    authorization.value={flowId:result.flowId,authorizationUrl:result.authorizationUrl};
    configText.value='';await openAuthorizationPage();authTimer=setTimeout(()=>void pollAuthorization(),1000);return;
  }
  if(result?.status!=='connected'&&result?.connected!==true)throw new Error('连接未返回成功状态，请重试。');
  await refreshLocal();configText.value='';selected.value=null;props.store.notify('连接成功，工具将在新任务中可用');
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
  if (!selected.value || !confirmed.value || busy.value || !configValid.value) return;
  busy.value = true; detailError.value = '';
  const context=detailId;
  try {
    const config: unknown = JSON.parse(configText.value);
    const result=await invoke<ConnectionResult>('ai_store_connector_connect', { id: selected.value.slug, name: selected.value.name, config, confirmed: true });
    await handleConnectionResult(result,context);
  } catch (e) { detailError.value = String(e); }
  finally { busy.value = false; }
}
async function connectionAction(action: 'disconnect' | 'reconnect') {
  if (!selected.value || busy.value) return;
  busy.value = true; detailError.value = '';
  const context=detailId;
  try {
    const result=await invoke<ConnectionResult>('ai_store_connector_' + action, { id: selected.value.slug });
    if(action==='reconnect'){await handleConnectionResult(result,context);return;}
    await refreshLocal(); selected.value = null; configText.value = '';
    props.store.notify(action === 'disconnect' ? '已断开连接，工具停止可用' : '已重新连接');
  } catch (e) { detailError.value = String(e); }
  finally { busy.value = false; }
}
defineExpose({ refresh, refreshLocal, installedCount, open });
</script>

<template>
  <div class="registry">
    <NAlert v-if="!store.desktop" type="info" :show-icon="false">请在小伴桌面客户端中浏览和安装。</NAlert>
    <NAlert v-if="error" type="error" :show-icon="false">{{ error }} <button class="text-action" @click="refresh">重试</button></NAlert>
    <NAlert v-if="localError" type="warning" :show-icon="false">{{ localError }} <button class="text-action" @click="refreshLocal">重试</button></NAlert>
    <NAlert v-if="fromMarket() && pluginError" type="warning" :show-icon="false">{{ pluginError }} <button class="text-action" @click="refreshLocal">重试</button></NAlert>
    <div v-if="loading" class="xc-grid" aria-busy="true"><NSkeleton v-for="n in 6" :key="n" height="170px" :sharp="false" /></div>
    <div v-else class="xc-grid">
      <slot name="prefix" />
      <article v-for="entry in entries" :key="entry.slug" class="xc store-card" :class="[kind === 'experts' ? 'expert-card' : 'connector-card', { 'market-plugin-card': isPlugin(entry) }]" :style="hueStyle(entry.category, entry.slug)">
        <div class="xc-top">
          <div class="xc-ic" :class="{ 'has-img': icon(entry), 'is-contain': icon(entry) && kind === 'connectors' }"><img v-if="icon(entry)" :src="icon(entry)!" alt="" referrerpolicy="no-referrer" @error="brokenIcons[entry.slug] = true" /><component :is="categoryIcon(kind === 'experts' ? entry.category : undefined)" v-else-if="kind === 'experts'" :size="24" :stroke-width="1.9" aria-hidden="true" /><Plug v-else :size="24" :stroke-width="1.9" aria-hidden="true" /></div>
          <div class="xc-main"><div class="xc-title"><h3>{{ isPlugin(entry) ? entry.connector?.displayName.zh || entry.connector?.displayName.en || entry.name : entry.name }}</h3></div><p class="xc-desc">{{ isPlugin(entry) ? entry.connector?.displayDescription.zh || entry.connector?.displayDescription.en || entry.description : entry.description }}</p></div>
        </div>
        <div class="xc-foot">
          <span class="xc-tag">{{ entry.category || (source === 'market' && kind === 'connectors' ? '小伴市场' : 'SkillHub') }}</span>
          <span v-if="kind === 'experts' && entry.skillCount !== undefined" class="xc-meta"><Sparkles :size="13" aria-hidden="true" />{{ entry.skillCount }} 个技能</span>
          <template v-if="isPlugin(entry)"><span class="xc-meta">{{ authLabel(entry) }}</span><span class="xc-meta">{{ entry.connector?.skillCount }} 个随包技能</span><span class="connection-status">{{ plugin(entry.slug) ? '已安装' : '未安装' }}</span></template>
          <span v-else-if="kind === 'connectors'" class="connection-status" :class="{ 'xb-ok': connection(entry.slug)?.connected }">{{ connection(entry.slug)?.connected ? '已连接 · ' + connection(entry.slug)?.toolCount + ' 个工具' : connection(entry.slug) ? '未连接' : '未配置' }}</span>
          <div class="xc-act">
            <button v-if="kind === 'experts' && pack(entry.slug)" class="xc-link" aria-label="管理专家包" @click="open(entry)">管理</button>
            <button class="small-action xc-pill" :class="{ 'is-done': kind === 'experts' ? !!pack(entry.slug) : isPlugin(entry) ? !!plugin(entry.slug) : !!connection(entry.slug) }" :disabled="busy || !store.desktop || (isPlugin(entry) ? !pluginReady || !!pluginError : !!localError)" @click="kind === 'experts' && pack(entry.slug)?.ready ? usePack(entry) : open(entry)">
              <template v-if="kind === 'experts'"><template v-if="pack(entry.slug)?.ready"><Check :size="12" :stroke-width="3" />使用专家</template><template v-else-if="pack(entry.slug)">依赖缺失</template><template v-else><Plus :size="12" :stroke-width="2.6" />安装</template></template>
              <template v-else-if="isPlugin(entry)">{{ pluginUpdate(entry) ? '更新' : plugin(entry.slug) ? '已安装' : '安装' }}</template>
              <template v-else>{{ connection(entry.slug) ? '管理' : '连接' }}<ArrowRight v-if="!connection(entry.slug)" :size="12" /></template>
            </button>
          </div>
        </div>
      </article>
    </div>
    <div v-if="!loading && !error && store.desktop && !entries.length && category !== '本机'" class="xb-empty store-empty"><p>没有找到匹配的{{ kind === 'experts' ? '专家包' : '连接器' }}</p></div>
    <div v-if="!installedOnly && !loading && !error && (page > 1 || data.hasMore)" class="pagination"><button :disabled="page <= 1" aria-label="上一页" @click="page--"><ChevronLeft :size="16" /></button><span>第 {{ page }} 页</span><button :disabled="!data.hasMore" aria-label="下一页" @click="page++"><ChevronRight :size="16" /></button></div>
    <NModal :show="!!selected" preset="card" :title="marketPlugin?.displayName.zh || marketPlugin?.displayName.en || selected?.name" class="registry-dialog" :class="{ 'store-dialog': !!marketPlugin }" :closable="!busy" :mask-closable="!busy" style="width:720px;max-width:calc(100vw - 36px)" @update:show="!$event && close()">
      <template v-if="selected">
        <p>{{ marketPlugin?.displayDescription.zh || marketPlugin?.displayDescription.en || selected.description }}</p>
        <div v-if="detailLoading" class="review-loading">正在读取说明与依赖…</div>
        <NAlert v-if="detailError" type="error" :show-icon="false">{{ detailError }} <button v-if="!busy&&!authorization" class="text-action" @click="open(selected)">重新读取</button></NAlert>
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
        <template v-else-if="marketPlugin">
          <MarketConnectorDetail v-if="connectorDetail && !detailLoading" :key="selected.slug + (selected.version || '')" :connector="marketPlugin" :slug="selected.slug" :namespace="selected.namespace" :version="connectorDetail.version" :installed="!!plugin(selected.slug)" :update-version="pluginUpdate(selected)" :local-error="pluginError || (!pluginReady ? '正在读取已安装连接器…' : '')" @busy="busy = $event" @installed="pluginInstalled" @changed="pluginConnectionChanged" @center="goToCenter" />
        </template>
        <template v-else>
          <div v-if="authorization" class="authorization-wait" role="status"><h4>等待网页授权</h4><p>请在服务商页面登录并确认权限。完成后会自动检测连接结果。</p><div class="dialog-actions"><button class="store-secondary" :disabled="busy" @click="cancelAuthorization">取消授权</button><button class="store-secondary" :disabled="busy" @click="pollAuthorization">检查结果</button><button class="store-primary" :disabled="busy" @click="openAuthorizationPage">重新打开授权页</button></div></div>
          <template v-else>
          <div v-if="connection(selected.slug)" class="connection-summary"><p>{{ connection(selected.slug)?.connected ? '已连接，发现 ' + connection(selected.slug)?.toolCount + ' 个工具。' : '已保存配置，可以重新连接。' }}</p><div class="dialog-actions"><button v-if="connection(selected.slug)?.connected" class="store-secondary" :disabled="busy" @click="connectionAction('disconnect')">断开连接</button><button v-else class="store-secondary" :disabled="busy" @click="connectionAction('reconnect')">使用已保存配置重连</button></div></div>
          <template v-if="connectorDetail">
          <details><summary>服务说明与凭据获取方式</summary><pre class="review-content">{{ connectorDetail.readme }}</pre></details>
          <h4>{{ connection(selected.slug) ? '重新配置' : '连接配置' }}</h4>
          <select v-if="connectorDetail.configs.length > 1" v-model="configIndex" aria-label="选择连接方式" @change="chooseConfig"><option v-for="(c, i) in connectorDetail.configs" :key="i" :value="i">{{ c.name }} · {{ c.config.type || c.config.transport || (c.config.command ? 'stdio' : 'HTTP') }}</option></select>
          <ConnectorConfig v-model="configText" :disabled="busy" @valid="configValid=$event" />
          <p class="detail-note">支持 stdio 和 Streamable HTTP。标准网页登录授权会在浏览器中完成；部分服务需填写密钥或独立注册小伴客户端。本地服务会启动配置中的命令；凭据仅保存在本机。</p>
          <label class="review-confirm"><input v-model="confirmed" type="checkbox" :disabled="busy" />我已检查服务来源与配置，允许小伴连接此服务。</label>
          <div class="dialog-actions"><button class="store-primary" :disabled="busy || !confirmed || !configText.trim() || !configValid" @click="connect">{{ busy ? '正在连接…' : '连接服务' }}<Plug :size="14" /></button></div>
          </template>
          </template>
        </template>
      </template>
    </NModal>
  </div>
</template>

<style scoped>
.registry{min-width:0}
.market-plugin-card .xc-foot{flex-wrap:wrap}
.market-plugin-card .xc-meta{white-space:normal;overflow-wrap:anywhere}
.registry :deep(.n-alert){margin-bottom:14px}
.text-action{color:var(--ui-ink);background:none;border:0;font:inherit;font-size:12px;cursor:pointer}
.pagination{display:flex;align-items:center;justify-content:center;gap:15px;margin:24px 0 4px;font-size:12px;color:var(--ui-text-secondary)}
.pagination button{display:grid;place-items:center;width:30px;height:30px;border:0;border-radius:8px;background:var(--ui-control);color:var(--ui-text-primary);cursor:pointer}
.pagination button:disabled{opacity:.45;cursor:not-allowed}
.connection-status{display:inline-flex;align-items:center;margin-left:4px;font-size:12px;color:var(--ui-text-secondary);white-space:nowrap}
.connection-status.xb-ok{color:var(--ui-success-text)}
.store-empty{min-height:200px}
</style>
