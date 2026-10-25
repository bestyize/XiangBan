<script setup lang="ts">
import { computed, nextTick, onActivated, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { NAlert, NModal, NSkeleton } from 'naive-ui';
import { Search, RefreshCw, ArrowRight, Download, Check, Plus, Puzzle, Monitor, ChevronLeft, ChevronRight, SlidersHorizontal, Users, Layers, TriangleAlert } from 'lucide-vue-next';
import { invoke, listen, type AiConfigView } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import { type Skill, type SkillPage, type SkillReview } from '../ai-store/catalog.ts';
import RegistryCatalog from '../ai-store/RegistryCatalog.vue';
import { hueStyle } from '../ai-store/hues.ts';
import MarketGroupDialog from '../ai-store/MarketGroupDialog.vue';
import MarketUpdateDialog, { type InstalledPackage } from '../ai-store/MarketUpdateDialog.vue';
import MarketLinkDialog from '../ai-store/MarketLinkDialog.vue';
import MarketSettingsDialog from '../ai-store/MarketSettingsDialog.vue';
import {
  MARKET_SOURCE_KEY, MARKET_UPDATES_KEY, SKILLHUB_KEY, marketUnavailable, memberLabel, readPref, shortDigest, variantLabel, writePref,
  type MarketDetail, type MarketItem, type MarketLink, type UpdateInfo,
} from '../ai-store/market.ts';
import '../ai-store/store-cards.css';

const props = defineProps<{ store: ControlStore }>();
type Tab = 'experts' | 'skills' | 'connectors';
const tabs: { id: Tab; label: string }[] = [{ id: 'experts', label: '专家' }, { id: 'skills', label: '技能' }, { id: 'connectors', label: '连接器' }];
const tab = ref<Tab>('experts');
const query = ref('');
const category = ref('全部');
const installedOnly = ref(false);
const skillSort = ref('score');
const marketSort = ref('hot');
const skillSource = ref('');
// 小伴市场是默认来源；SkillHub 是补充来源。市场连不上时自动显示 SkillHub，但不改变用户的选择。
const source = ref<'market' | 'hub'>(readPref(MARKET_SOURCE_KEY, 'market') === 'hub' ? 'hub' : 'market');
const skillhubOn = ref(readPref(SKILLHUB_KEY, '1') !== '0');
const autoUpdates = ref(readPref(MARKET_UPDATES_KEY, '1') !== '0');
const marketDown = ref(false);
const effMarket = computed(() => !skillhubOn.value || (source.value === 'market' && !marketDown.value));
const packageTab = computed(() => tab.value === 'skills' || (tab.value === 'experts' && effMarket.value));
const marketCategories = ref<Record<string, { key: string; name: string }[]>>({});
const updates = ref<Record<string, UpdateInfo>>({});
const group = ref<Skill | null>(null);
const updating = ref<{ pkg: InstalledPackage; info: UpdateInfo } | null>(null);
const link = ref<MarketLink | null>(null);
const settingsOpen = ref(false);
const reviewKind = ref<'skills' | 'experts'>('skills');
const reviewVia = ref<'market' | 'hub'>('hub');
const skillCategories = ref<{ key: string; name: string }[]>([]);
const categoriesError = ref('');
const localSkills = ref<Skill[]>([]);
const brokenIcons = ref<Record<string, boolean>>({});
function skillIcon(s: Skill) {
  const url = s.iconUrl?.trim();
  return url?.startsWith('https://') && !brokenIcons.value[url] ? url : null;
}
const remote = ref<SkillPage>({ results: [], page: 1, total: null, hasMore: false });
const page = ref(1);
const loading = ref(false);
const error = ref('');
const localError = ref('');
const computerEnabled = ref(false);
const localReady = ref(false);
const registry = ref<InstanceType<typeof RegistryCatalog> | null>(null);
const skill = ref<Skill | null>(null);
const computerOpen = ref(false);
const review = ref<SkillReview | null>(null);
const reviewError = ref('');
const reviewing = ref(false);
const reviewed = ref(false);
const removing = ref(false);
const busy = ref(false);
let browseId = 0;
let categoryRequestId = 0;
let reviewId = 0;
let localLoading = false;
let timer: ReturnType<typeof setTimeout> | undefined;

const categories = computed<{ key: string; name: string }[]>(() => {
  if (effMarket.value) return installedOnly.value ? [{ key: '全部', name: '全部' }] : [{ key: '全部', name: '全部' }, ...(marketCategories.value[tab.value] ?? [])];
  if (tab.value === 'skills') return installedOnly.value ? [{ key: '全部', name: '全部' }] : [{ key: '全部', name: '全部' }, ...skillCategories.value];
  const labels = tab.value === 'experts'
    ? ['全部', '开发', '办公', '创作', '金融', '学习', '生活']
    : ['全部', '腾讯产品MCP', '搜索与信息检索', '设计与创意', '本机'];
  return labels.map(key => ({ key, name: key }));
});
const matches = (text: string) => text.toLocaleLowerCase().includes(query.value.trim().toLocaleLowerCase());
const showComputer = computed(() => ['全部', '本机'].includes(category.value) && matches('本机电脑控制 Computer Use 截图 鼠标 键盘'));
// 通过市场安装的专家和技能都装在技能目录里，按安装时记录的类型分开显示。
const localForTab = computed(() => localSkills.value.filter(s => tab.value === 'experts' ? s.marketKind === 'expert' : s.marketKind !== 'expert'));
const visibleSkills = computed(() => installedOnly.value ? localForTab.value.filter(s => matches(s.name + s.description)) : remote.value.results);
const sourceName = computed(() => effMarket.value ? '小伴市场' : 'SkillHub');
const updatable = computed(() => localForTab.value.filter(s => updates.value[s.slug]));
const installed = (slug: string) => localSkills.value.find(s => s.slug === slug);
const noun = computed(() => tab.value === 'experts' ? '专家' : '技能');
const heading = computed(() => installedOnly.value ? `已安装${noun.value}` : `发现${noun.value}`);
const description = computed(() => installedOnly.value ? visibleSkills.value.length + ` 项${noun.value}` : remote.value.total === null ? `来自 ${sourceName.value}` : `共 ${remote.value.total.toLocaleString('zh-CN')} 项匹配${noun.value} · ${sourceName.value}`);
const empty = computed(() => packageTab.value && !loading.value && !visibleSkills.value.length);

async function refreshLocal() {
  if (!props.store.desktop) { localReady.value = true; return; }
  if (localLoading) return;
  localLoading = true;
  localError.value = '';
  const [skillsResult, configResult] = await Promise.allSettled([
    invoke<Skill[]>('ai_store_installed'), invoke<AiConfigView>('get_ai_config'),
  ]);
  if (skillsResult.status === 'fulfilled') { localSkills.value = skillsResult.value; localReady.value = true; }
  else { localReady.value = false; localError.value = `无法读取已安装技能：${String(skillsResult.reason)}`; }
  if (configResult.status === 'fulfilled') computerEnabled.value = configResult.value.openComputerUse;
  else localError.value += `${localError.value ? '；' : ''}无法读取连接器配置：${String(configResult.reason)}`;
  localLoading = false;
}
async function loadSkillCategories() {
  if (!props.store.desktop || skillCategories.value.length) return;
  const id = ++categoryRequestId;
  try {
    const response = await invoke<{ results: { key: string; name: string }[] }>('ai_store_skill_categories');
    if (id !== categoryRequestId) return;
    skillCategories.value = response.results;
    categoriesError.value = '';
  } catch (e) {
    if (id === categoryRequestId) categoriesError.value = '无法加载 SkillHub 分类：' + String(e);
  }
}
async function loadMarketCategories(kind: string) {
  if (!props.store.desktop || !effMarket.value || marketCategories.value[kind]) return;
  try {
    const response = await invoke<{ results: { key: string; name: string }[] }>('ai_market_categories', { kind });
    marketCategories.value = { ...marketCategories.value, [kind]: response.results };
  } catch (e) { onMarketError(e); }
}
// 只有「连不上市场」才退回 SkillHub；市场回答了「没有」不算故障。
function onMarketError(e: unknown): boolean {
  if (!effMarket.value || !skillhubOn.value || !marketUnavailable(String(e))) return false;
  marketDown.value = true;
  return true;
}
function retryMarket() { marketDown.value = false; }
async function browse() {
  clearTimeout(timer);
  const id = ++browseId;
  if (!props.store.desktop || !packageTab.value || installedOnly.value) { loading.value = false; return; }
  loading.value = true; error.value = ''; remote.value = { results: [], page: page.value, total: null, hasMore: false };
  const viaMarket = effMarket.value;
  try {
    const keyword = query.value.trim().slice(0, 80), cat = category.value === '全部' ? '' : category.value;
    const result = viaMarket
      ? await invoke<SkillPage>('ai_market_browse', { kind: tab.value, keyword, page: page.value, category: cat, source: skillSource.value, sortBy: marketSort.value, variant: '' })
      : await invoke<SkillPage>('ai_store_browse', { keyword, page: page.value, category: cat, source: skillSource.value, sortBy: skillSort.value });
    if (id === browseId) remote.value = result;
  } catch (e) {
    if (id !== browseId) return;
    if (viaMarket && onMarketError(e)) return;
    error.value = `暂时无法加载${viaMarket ? '小伴市场' : ' SkillHub'}：${String(e)}`;
  }
  finally { if (id === browseId) loading.value = false; }
}
function scheduleBrowse() {
  clearTimeout(timer); ++browseId; loading.value = false; error.value = '';
  if (packageTab.value && !installedOnly.value) {
    remote.value = { results: [], page: page.value, total: null, hasMore: false };
    loading.value = props.store.desktop;
    timer = setTimeout(() => void browse(), 250);
  }
}
function switchTab(value: Tab) { tab.value = value; category.value = '全部'; page.value = 1; installedOnly.value = false; }
const installedCount = computed(() => packageTab.value ? localForTab.value.length : (registry.value?.installedCount ?? 0));
function pickSource(value: 'market' | 'hub') {
  if (value === 'hub' && !skillhubOn.value) return;
  source.value = value; marketDown.value = false; writePref(MARKET_SOURCE_KEY, value);
}
watch(skillhubOn, on => { writePref(SKILLHUB_KEY, on ? '1' : '0'); if (!on) { source.value = 'market'; marketDown.value = false; } });
watch(autoUpdates, on => { writePref(MARKET_UPDATES_KEY, on ? '1' : '0'); if (on) void checkUpdates(); else updates.value = {}; });
function tabKey(event: KeyboardEvent) {
  if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
  event.preventDefault();
  const index = tabs.findIndex(t => t.id === tab.value);
  const next = event.key === 'Home' ? 0 : event.key === 'End' ? tabs.length - 1 : (index + (event.key === 'ArrowRight' ? 1 : -1) + tabs.length) % tabs.length;
  switchTab(tabs[next].id);
  (event.currentTarget as HTMLElement).querySelector<HTMLButtonElement>(`#store-tab-${tab.value}`)?.focus();
}
watch([query, category, installedOnly, skillSort, marketSort, skillSource], () => { page.value = 1; scheduleBrowse(); });
watch(tab, () => { scheduleBrowse(); if (effMarket.value) void loadMarketCategories(tab.value); });
watch(effMarket, () => { category.value = '全部'; page.value = 1; scheduleBrowse(); if (effMarket.value) void loadMarketCategories(tab.value); });
watch(page, () => { scheduleBrowse(); });
function loadCategories() { if (effMarket.value) void loadMarketCategories(tab.value); else void loadSkillCategories(); }
onActivated(() => {
  void refreshLocal(); void checkUpdates(); void takeLink(); loadCategories();
  if (packageTab.value) void browse(); else { void registry.value?.refresh(); void registry.value?.refreshLocal(); }
});
let unlisten: (() => void) | undefined;
onMounted(() => {
  void refreshLocal(); loadCategories(); void checkUpdates(); void takeLink();
  if (packageTab.value) scheduleBrowse();
  void listen('market-link', () => void takeLink()).then(stop => { unlisten = stop; });
});
onBeforeUnmount(() => { clearTimeout(timer); ++browseId; ++reviewId; ++categoryRequestId; unlisten?.(); });

// ---- 更新 ----
async function checkUpdates(manual = false) {
  if (!props.store.desktop || (!manual && !autoUpdates.value)) return;
  try {
    const response = await invoke<{ results: UpdateInfo[] }>('ai_market_updates');
    updates.value = Object.fromEntries(response.results.map(u => [u.slug, u]));
    if (manual) props.store.notify(response.results.length ? `${response.results.length} 项可更新` : '已安装的插件都是最新版本');
  } catch (e) { if (manual) props.store.notify(e, 'error'); }
}
function openUpdate(s: Skill) {
  const info = updates.value[s.slug];
  if (!info || !s.namespace) return;
  updating.value = { info, pkg: { slug: s.slug, name: s.name, namespace: s.namespace, version: s.version, marketKind: s.marketKind, sha256: s.manifestSha256 ?? undefined, files: s.files ?? undefined } };
}
async function updated(message: string) { updating.value = null; props.store.notify(message); await refreshLocal(); await checkUpdates(); }
async function groupDone(message: string) { group.value = null; props.store.notify(message); await refreshLocal(); }

// ---- 网页唤起 ----
async function takeLink() {
  if (!props.store.desktop) return;
  try {
    const next = await invoke<MarketLink | null>('ai_market_take_link');
    if (!next) return;
    switchTab(next.kind); source.value = 'market'; marketDown.value = false; link.value = next;
  } catch { /* 没有待处理的链接 */ }
}
async function followLink(detail: MarketDetail, version: string | null) {
  const request = link.value;
  link.value = null;
  if (!request) return;
  const item: MarketItem = { ...detail.item, version: version ?? detail.item.version };
  if (request.kind === 'connectors') { await nextTick(); void registry.value?.open({ slug: item.slug, name: item.name, description: item.description, namespace: item.namespace, iconUrl: item.iconUrl }); return; }
  const entry: Skill = { ...item, source: item.source, variant: item.variant };
  if (item.variant && item.variant !== 'single') { group.value = entry; return; }
  void openSkill(entry, 'market', request.kind);
}

async function startConversation(text: string) {
  if (busy.value) return;
  busy.value = true;
  try {
    await props.store.chat.select(null);
    props.store.creationDraft = text;
    skill.value = null;
    props.store.navigate('chat');
  } catch (e) { props.store.notify(e, 'error'); }
  finally { busy.value = false; }
}
/** 已安装的 WorkBuddy 式专家：召唤它开一个新对话；技能和老的专家包仍是让小伴去读它的说明。 */
async function summon(s: Skill) {
  const local = installed(s.slug) ?? s;
  if (busy.value) return;
  await props.store.chat.select(null);
  props.store.chat.pendingExpert = {
    plugin: local.slug, name: local.name, profession: local.profession ?? '', description: local.description, agent: local.agent,
    quickPrompts: local.quickPrompts, tags: local.tags, hasAvatar: local.hasAvatar, kind: local.pluginKind,
  };
  skill.value = null;
  props.store.navigate('new-task');
}
function useSkill(s: Skill) {
  if (installed(s.slug)?.plugin) return summon(s);
  return startConversation(`请读取已安装的技能「${s.slug}」，按其中的流程帮助我完成以下任务：`);
}
async function openSkill(s: Skill, via: 'market' | 'hub' = effMarket.value ? 'market' : 'hub', kind: 'skills' | 'experts' = tab.value === 'experts' ? 'experts' : 'skills') {
  skill.value = s; review.value = null; reviewError.value = ''; reviewed.value = false; removing.value = false;
  reviewVia.value = via; reviewKind.value = kind;
  const id = ++reviewId;
  if (installed(s.slug)) { reviewing.value = false; return; }
  if (!s.version) { reviewing.value = false; reviewError.value = '此技能暂未提供可安装的版本。'; return; }
  reviewing.value = true;
  try {
    const result = via === 'market'
      ? await invoke<SkillReview>('ai_market_inspect', { kind, namespace: s.namespace, slug: s.slug, version: s.version })
      : await invoke<SkillReview>('ai_store_inspect', { slug: s.slug, version: s.version, ...(s.namespace ? { namespace: s.namespace } : {}) });
    if (id === reviewId) review.value = result;
  } catch (e) { if (id === reviewId) reviewError.value = String(e); }
  finally { if (id === reviewId) reviewing.value = false; }
}
async function installSkill() {
  if (!skill.value || !review.value || !reviewed.value || busy.value || !localReady.value) return;
  busy.value = true;
  const s = skill.value; const r = review.value;
  try {
    if (reviewVia.value === 'market') await invoke('ai_market_install', { kind: reviewKind.value, namespace: s.namespace, slug: s.slug, version: r.manifest.version, sha256: r.sha256 });
    else await invoke('ai_store_install', { slug: s.slug, version: r.manifest.version, sha256: r.sha256, ...(s.namespace ? { namespace: s.namespace } : {}) });
    await refreshLocal();
    props.store.notify(`已安装「${s.name}」，可在新任务中使用`);
    skill.value = null;
  } catch (e) { reviewError.value = String(e); }
  finally { busy.value = false; }
}
async function uninstallSkill() {
  if (!skill.value || !installed(skill.value.slug)?.removable || !removing.value || busy.value) return;
  busy.value = true;
  try {
    await invoke('ai_store_uninstall', { slug: skill.value.slug });
    await refreshLocal(); skill.value = null; props.store.notify('已卸载技能');
  } catch (e) { reviewError.value = String(e); }
  finally { busy.value = false; }
}
async function toggleComputer() {
  if (busy.value || !props.store.desktop) return;
  busy.value = true;
  try {
    const config = await invoke<AiConfigView>('save_ai_preferences', { openComputerUse: !computerEnabled.value });
    computerEnabled.value = config.openComputerUse;
    props.store.notify(computerEnabled.value ? '已启用电脑控制，使用时按系统提示授权' : '已停用电脑控制');
    computerOpen.value = false;
  } catch (e) { props.store.notify(e, 'error'); }
  finally { busy.value = false; }
}
function reset() { query.value = ''; category.value = '全部'; installedOnly.value = false; skillSort.value = 'score'; marketSort.value = 'hot'; skillSource.value = ''; }
const tagOf = (s: Skill) => installed(s.slug)?.source || (variantLabel(s.variant) || (effMarket.value ? (s.source === 'official' ? '小伴市场 · 官方' : '小伴市场') : s.source === 'official' ? 'SkillHub · 官方' : s.source === 'community' ? 'SkillHub · 社区' : 'SkillHub'));
const bytes = (n: number) => n < 1024 ? `${n} B` : `${(n / 1024).toFixed(1)} KB`;
</script>

<template>
  <section class="ai-store-page xb-page" data-testid="ai-store-page" aria-label="AI 插件">
    <header class="xb-page-head store-header" data-tauri-drag-region>
      <div class="xb-tabs-lg store-tabs" role="tablist" aria-label="插件类型" @keydown="tabKey">
        <button v-for="t in tabs" :id="`store-tab-${t.id}`" :key="t.id" type="button" role="tab" :aria-controls="`store-panel-${t.id}`" :aria-selected="tab === t.id" :tabindex="tab === t.id ? 0 : -1" @click="switchTab(t.id)">{{ t.label }}</button>
      </div>
      <div class="xb-page-actions">
        <form class="xb-search store-search" role="search" @submit.prevent="tab === 'skills' ? browse() : registry?.refresh()">
          <Search :size="14" aria-hidden="true" /><input v-model="query" maxlength="70" aria-label="搜索专家、技能或连接器" :placeholder="`搜索${tabs.find(t => t.id === tab)?.label}`" />
          <button v-if="query" type="button" class="clear" aria-label="清空搜索" @click="query = ''">×</button>
        </form>
        <button type="button" class="xb-btn installed-filter" :class="{ selected: installedOnly }" :aria-pressed="installedOnly" @click="installedOnly = !installedOnly; category = '全部'">{{ tab === 'connectors' ? '已配置' : '已安装' }}<span class="count">{{ installedCount }}</span><i v-if="packageTab && updatable.length" class="update-dot" role="img" aria-label="有可用更新" /></button>
        <button type="button" class="xb-icon-btn source-settings" aria-label="插件来源设置" @click="settingsOpen = true"><SlidersHorizontal :size="16" /></button>
      </div>
    </header>
    <div class="store-toolbar">
      <div class="store-filters" aria-label="按领域筛选">
        <button v-for="c in categories" :key="c.key" type="button" :class="{ active: category === c.key }" :aria-pressed="category === c.key" @click="category = c.key">{{ c.name }}</button>
      </div>
      <div class="skill-query-options">
        <div v-if="skillhubOn" class="xb-seg source-switch" role="group" aria-label="内容来源">
          <button type="button" :aria-pressed="effMarket" @click="pickSource('market')">小伴市场<i v-if="marketDown" class="down-dot" role="img" aria-label="暂时无法连接" /></button>
          <button type="button" :aria-pressed="!effMarket" @click="pickSource('hub')">SkillHub</button>
        </div>
        <template v-if="packageTab && !installedOnly">
          <label v-if="effMarket">排序 <select v-model="marketSort" aria-label="排序">
            <option value="hot">推荐</option><option value="new">最新</option>
          </select></label>
          <label v-else>排序 <select v-model="skillSort" aria-label="技能排序">
            <option value="score">综合评分</option><option value="downloads">最多下载</option>
            <option value="stars">最多收藏</option><option value="installs">最多安装</option>
            <option value="updated_at">最近更新</option>
          </select></label>
          <label>来源 <select v-model="skillSource" aria-label="技能来源">
            <option value="">全部</option><option value="official">官方</option><option value="community">社区</option>
          </select></label>
        </template>
      </div>
    </div>
    <div :id="`store-panel-${tab}`" class="store-content xb-scroll" role="tabpanel" :aria-labelledby="`store-tab-${tab}`">
      <div v-if="packageTab" class="store-section-heading"><h2>{{ heading }}</h2><span>{{ description }}</span></div>
      <NAlert v-if="!store.desktop && packageTab" class="store-alert" type="info" :show-icon="false">请在小伴桌面客户端中浏览技能和管理连接器。</NAlert>
      <NAlert v-if="marketDown" class="store-alert market-down" type="warning" :show-icon="false" role="status"><TriangleAlert :size="14" />暂时无法连接小伴市场，已先显示 SkillHub 的内容。已安装的插件不受影响。 <button class="text-action" @click="retryMarket">重试</button></NAlert>
      <NAlert v-if="packageTab && !installedOnly && updatable.length" class="store-alert update-hint" type="info" :show-icon="false" role="status">{{ updatable.length }} 个已安装{{ noun }}有新版本。 <button class="text-action" @click="installedOnly = true; category = '全部'">查看更新</button></NAlert>
      <NAlert v-if="packageTab && installedOnly && effMarket" class="store-alert" type="info" :show-icon="false" role="status"><b>{{ updatable.length }} 项可更新</b>。更新前会展示新版本与现有版本的文件差异。 <button class="text-action" @click="checkUpdates(true)">立即检查</button></NAlert>
      <NAlert v-if="categoriesError && tab === 'skills' && !installedOnly && !effMarket" class="store-alert" type="warning" :show-icon="false">{{ categoriesError }} <button class="text-action" @click="loadSkillCategories">重试</button></NAlert>
      <NAlert v-if="localError" class="store-alert" type="warning" :show-icon="false">{{ localError }} <button class="text-action" @click="refreshLocal">重试</button></NAlert>
      <NAlert v-if="error && packageTab && !installedOnly" class="store-alert" type="error" :show-icon="false">{{ error }} <button class="text-action" @click="browse">重试</button></NAlert>

      <template v-if="packageTab">
        <div v-if="loading" class="xc-grid" aria-label="正在加载技能" aria-busy="true"><NSkeleton v-for="n in 6" :key="n" height="170px" :sharp="false" /></div>
        <div v-else class="xc-grid skill-grid">
          <article v-for="s in visibleSkills" :key="s.slug" class="xc store-card skill-card" :style="hueStyle(undefined, s.slug)">
            <div class="xc-top">
              <div class="xc-ic" :class="{ 'has-img': skillIcon(s) }"><img v-if="skillIcon(s)" :src="skillIcon(s)!" alt="" loading="lazy" decoding="async" referrerpolicy="no-referrer" @error="brokenIcons[skillIcon(s)!] = true" /><Users v-else-if="s.variant === 'team'" :size="24" :stroke-width="1.9" aria-hidden="true" /><Layers v-else-if="s.variant === 'bundle'" :size="24" :stroke-width="1.9" aria-hidden="true" /><Puzzle v-else :size="24" :stroke-width="1.9" aria-hidden="true" /></div>
              <div class="xc-main">
                <div class="xc-title"><h3>{{ s.name || s.slug }}</h3><svg v-if="s.source === 'official'" class="xc-verified" role="img" aria-label="官方认证" width="14" height="14" viewBox="0 0 24 24"><path d="M12 2.5l2.4 1.8 3-.2.9 2.9 2.4 1.8-1 2.8 1 2.8-2.4 1.8-.9 2.9-3-.2L12 21.5l-2.4-1.8-3 .2-.9-2.9-2.4-1.8 1-2.8-1-2.8 2.4-1.8.9-2.9 3 .2z" fill="currentColor" /><path d="m8.5 12 2.4 2.4 4.6-4.8" fill="none" stroke="#fff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" /></svg></div>
                <p class="xc-desc">{{ s.description || (tab === 'experts' ? '查看专家说明与使用方法' : '查看技能说明与使用方法') }}</p>
              </div>
            </div>
            <div class="xc-foot">
              <span class="xc-tag" :class="{ 'xc-tag-group': s.variant && s.variant !== 'single' }">{{ tagOf(s) }}</span>
              <span v-if="s.variant && s.variant !== 'single'" class="xc-meta">{{ memberLabel(s.variant, s.memberCount) }}</span>
              <span v-else-if="updates[s.slug]" class="xc-meta">v{{ s.version }} → v{{ updates[s.slug].latest }}</span>
              <span v-else-if="s.version" class="xc-meta">v{{ s.version }}</span>
              <div class="xc-act">
                <button v-if="updates[s.slug] && installed(s.slug)" class="small-action xc-pill xc-pill-fill" :disabled="!store.desktop || !localReady" @click="openUpdate(installed(s.slug)!)">更新</button>
                <button v-else class="small-action xc-pill" :class="{ installed: installed(s.slug), 'is-done': installed(s.slug) }" :disabled="!store.desktop || !localReady" @click="s.variant && s.variant !== 'single' ? (group = s) : openSkill(s)"><Check v-if="installed(s.slug)" :size="12" :stroke-width="3" /><Plus v-else :size="12" :stroke-width="2.6" />{{ installed(s.slug) ? '已安装' : '安装' }}</button>
              </div>
            </div>
          </article>
        </div>
        <div v-if="!installedOnly && !error && !loading && store.desktop && (page > 1 || remote.hasMore)" class="store-pagination"><button :disabled="page <= 1" aria-label="上一页" @click="page--"><ChevronLeft :size="16" /></button><span>第 {{ page }} 页</span><button :disabled="!remote.hasMore" aria-label="下一页" @click="page++"><ChevronRight :size="16" /></button></div>
      </template>
      <RegistryCatalog v-else ref="registry" :kind="tab as 'experts' | 'connectors'" :query="query" :category="category" :store="store" :installed-only="installedOnly" :source="effMarket ? 'market' : 'hub'" @unavailable="onMarketError">
        <template v-if="tab === 'connectors' && showComputer" #prefix>
        <article v-if="showComputer" class="xc store-card connector-card" style="--h-hi:#7C8CF8;--h:#4A57D0">
          <div class="xc-top">
            <div class="xc-ic"><Monitor :size="24" :stroke-width="1.9" aria-hidden="true" /></div>
            <div class="xc-main"><div class="xc-title"><h3>本机电脑控制</h3></div><p class="xc-desc">通过截图、鼠标和键盘操作电脑上的应用</p></div>
          </div>
          <div class="xc-foot">
            <span class="xc-tag">Computer Use</span>
            <span class="connection-status" :class="{ enabled: computerEnabled, 'xb-ok': computerEnabled }">{{ computerEnabled ? '已启用' : '未启用' }}</span>
            <div class="xc-act"><button class="small-action xc-pill" :class="{ 'is-done': computerEnabled }" :disabled="!store.desktop || !!localError" @click="computerOpen = true">{{ computerEnabled ? '管理' : '启用' }}</button></div>
          </div>
        </article>
        </template>
      </RegistryCatalog>
      <div v-if="empty && !error && (!packageTab || store.desktop)" class="xb-empty store-empty"><div class="xb-empty-icon"><Search :size="24" /></div><h3>{{ installedOnly ? `暂无匹配的已安装${noun}` : '没有找到匹配内容' }}</h3><p>试试其他关键词或分类</p><button class="xb-btn is-sm" @click="reset">清除筛选</button></div>
    </div>

    <NModal :show="!!skill" preset="card" :title="skill?.name || skill?.slug" class="store-dialog skill-dialog" :mask-closable="!busy" :closable="!busy" style="width:680px;max-width:calc(100vw - 36px)" @update:show="!$event && !busy && (skill = null)">
      <template v-if="skill"><p>{{ skill.description }}</p><div class="detail-tags"><span class="store-tag">{{ installed(skill.slug)?.source || (reviewVia === 'market' ? '小伴市场' : 'SkillHub') }}</span><span v-if="skill.version" class="store-tag">v{{ skill.version }}</span></div>
        <div v-if="reviewing" class="review-loading"><RefreshCw :size="17" />正在读取固定版本的文件与脚本…</div>
        <NAlert v-if="reviewError" type="error" :show-icon="false">{{ reviewError }} <button v-if="!installed(skill.slug)" class="text-action" :disabled="busy" @click="openSkill(skill, reviewVia, reviewKind)">重试</button></NAlert>
        <template v-if="review && !installed(skill.slug)">
          <h4>技能说明</h4><pre class="review-content">{{ review.content }}</pre>
          <h4>文件清单 <small>{{ review.manifest.fileCount }} 个文件 · {{ bytes(review.manifest.totalBytes) }}</small></h4><ul class="review-files"><li v-for="file in review.manifest.files" :key="file.path"><div><code>{{ file.path }}</code><span>{{ bytes(file.size) }}{{ file.executableScript ? ' · 脚本' : '' }}</span></div><small>SHA256 {{ file.sha256 }}</small></li></ul>
          <p v-if="reviewVia === 'market'" class="version-lock"><span>版本锁定</span><code>清单摘要 {{ shortDigest(review.sha256) }}</code><span class="lock-ok"><Check :size="12" :stroke-width="3" />已逐文件重新计算，与小伴市场记录一致</span></p>
          <details v-for="script in review.scripts" :key="script.path" class="script-preview"><summary>{{ script.path }} · 脚本预览{{ script.previewTruncated ? '（已截断）' : '' }}</summary><pre>{{ script.preview || '此脚本暂无文本预览，请在确认来源后再安装。' }}</pre></details>
          <p class="detail-note">安装不会执行脚本。脚本运行时需单独确认，并使用本机账户权限；文件校验不代表安全认证。</p>
          <label class="review-confirm"><input v-model="reviewed" type="checkbox" :disabled="busy" />我已查看此版本的说明、文件清单和脚本预览，同意安装。</label>
        </template>
        <div v-if="installed(skill.slug)" class="installed-detail"><Check :size="17" /><p>{{ installed(skill.slug)?.plugin ? '这位专家已安装，召唤后会开一个由 TA 负责的新对话。' : '此技能已安装，可在聊天中新建任务使用。' }}</p></div>
        <p v-if="removing" class="detail-note">确认卸载此技能？卸载后新任务将无法加载它，进行中的任务可能受影响。</p>
        <div class="dialog-actions"><template v-if="installed(skill.slug)"><button v-if="installed(skill.slug)?.removable" class="store-secondary" :disabled="busy" @click="removing ? uninstallSkill() : (removing = true)">{{ busy ? '正在卸载…' : removing ? '确认卸载' : '卸载' }}</button><button class="store-primary" :disabled="busy" @click="useSkill(skill)">{{ installed(skill.slug)?.plugin ? '召唤这位专家' : '使用技能' }} <ArrowRight :size="14" /></button></template><button v-else class="store-primary" :disabled="!review || !reviewed || reviewing || busy || !localReady" @click="installSkill">{{ busy ? '正在安装…' : '确认安装' }}<Download :size="14" /></button></div>
      </template>
    </NModal>
    <NModal v-model:show="computerOpen" preset="card" title="本机电脑控制" class="store-dialog" :mask-closable="!busy" :closable="!busy" style="width:460px;max-width:calc(100vw - 36px)">
      <p>允许小伴在任务中调用内置 Computer Use 引擎，通过截图、鼠标与键盘操作应用。</p><p class="detail-note">首次使用按系统提示授予屏幕录制与辅助功能权限。开关表示能力已启用，实际权限以系统授权为准。</p><div class="dialog-actions"><button class="store-secondary" :disabled="busy" @click="computerOpen = false">取消</button><button class="store-primary" :disabled="busy" @click="toggleComputer">{{ busy ? '正在保存…' : computerEnabled ? '停用电脑控制' : '启用电脑控制' }}</button></div>
    </NModal>
    <MarketGroupDialog :item="group" :kind="tab === 'experts' ? 'experts' : 'skills'" @close="group = null" @done="groupDone" />
    <MarketUpdateDialog :pkg="updating?.pkg ?? null" :info="updating?.info ?? null" @close="updating = null" @done="updated" />
    <MarketLinkDialog :link="link" @close="link = null" @go="followLink" />
    <MarketSettingsDialog :show="settingsOpen" v-model:skillhub="skillhubOn" v-model:auto-updates="autoUpdates" @close="settingsOpen = false" />
  </section>
</template>

<style scoped>
.ai-store-page{display:flex;flex-direction:column;flex:1;min-height:0;min-width:0;padding-bottom:0;color:var(--ui-text-primary);background:var(--ui-surface);container-type:inline-size}
.store-header{flex:none}
.store-header .xb-page-actions .xb-search{width:220px;height:32px;padding:0 10px;gap:8px;border-radius:9px;background:var(--ui-control);box-sizing:border-box}
.store-search .clear{border:0;background:none;color:var(--ui-text-tertiary);font-size:16px;line-height:1;cursor:pointer;padding:0}
.installed-filter .count{min-width:18px;height:18px;padding:0 5px;box-sizing:border-box;border-radius:999px;background:var(--ui-soft);color:var(--ui-ink);font-size:11px;font-weight:600;display:inline-grid;place-items:center}
.installed-filter.selected{box-shadow:inset 0 0 0 1.5px var(--ui-btn);color:var(--ui-ink)}
.store-toolbar{flex:none;margin-top:18px;display:flex;align-items:center;gap:12px;padding-bottom:12px;border-bottom:1px solid var(--ui-line);flex-wrap:wrap}
.store-filters{display:flex;align-items:center;flex-wrap:wrap;gap:4px;min-width:0}
.store-filters button{height:30px;padding:0 12px;border:0;border-radius:8px;background:transparent;color:var(--ui-text-secondary);font:inherit;font-size:13px;white-space:nowrap;cursor:pointer}
.store-filters button:hover{background:var(--ui-control)}
.store-filters button.active{background:var(--ui-soft);color:var(--ui-ink);font-weight:600}
.skill-query-options{margin-left:auto;display:flex;align-items:center;gap:12px}
.skill-query-options label{display:flex;align-items:center;gap:6px;white-space:nowrap;font-size:12px;color:var(--ui-text-secondary)}
.skill-query-options select{max-width:120px;height:28px;border:0;border-radius:8px;padding:0 8px;background:var(--ui-control);color:var(--ui-text-primary);font:inherit;font-size:12px}
.store-content{flex:1;min-height:0;overflow-y:auto;margin:0 -28px;padding:16px 28px 28px}
.store-section-heading{display:flex;align-items:baseline;gap:10px;margin:0 0 12px}
.store-section-heading h2{font-size:14px;font-weight:600;margin:0}
.store-section-heading>span{font-size:12px;color:var(--ui-text-tertiary)}
.store-alert{margin-bottom:14px}
.text-action{border:0;background:none;color:var(--ui-ink);font:inherit;font-size:12px;cursor:pointer}
.store-empty{min-height:230px}
.store-pagination{display:flex;align-items:center;justify-content:center;gap:14px;margin:24px 0 0;font-size:12px;color:var(--ui-text-secondary)}
.store-pagination button{display:grid;place-items:center;width:30px;height:30px;border:0;border-radius:8px;background:var(--ui-control);color:var(--ui-text-primary);cursor:pointer}
.store-pagination button:disabled{opacity:.45;cursor:not-allowed}
.connection-status{display:inline-flex;align-items:center;margin-left:4px;font-size:12px;color:var(--ui-text-secondary);white-space:nowrap}
.connection-status.xb-ok{color:var(--ui-success-text)}
.update-dot,.down-dot{display:inline-block;width:7px;height:7px;border-radius:50%;flex:none}
.update-dot{background:var(--ui-dot)}
.down-dot{background:var(--ui-warning);margin-left:6px}
.source-switch{margin-right:2px}
.source-settings{border-radius:8px;box-shadow:0 0 0 1px var(--ui-border)}
.market-down :deep(.n-alert-body),.market-down{display:flex;align-items:center;gap:8px}
.xc-pill-fill{color:#fff!important;background:linear-gradient(var(--ui-btn-hi),var(--ui-btn))!important;box-shadow:var(--ui-shadow-btn)!important}
.xc-tag-group{background:var(--ui-soft);color:var(--ui-ink)}
.version-lock{display:flex;align-items:center;gap:10px;flex-wrap:wrap;margin:10px 0;padding:11px 14px;border-radius:12px;box-shadow:0 0 0 1px var(--ui-border);font-size:12px;color:var(--ui-text-secondary)}
.version-lock code{color:var(--ui-text-primary)}
.lock-ok{display:inline-flex;align-items:center;gap:4px;color:var(--ui-success-text)}
</style>
