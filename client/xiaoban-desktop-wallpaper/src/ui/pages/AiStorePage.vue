<script setup lang="ts">
import { computed, onActivated, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { NAlert, NModal, NSkeleton } from 'naive-ui';
import { Search, SlidersHorizontal, RefreshCw, ArrowRight, Download, Check, Puzzle, Monitor, ChevronLeft, ChevronRight } from 'lucide-vue-next';
import { invoke, type AiConfigView } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import { experts, connectors, type Expert, type Connector, type Skill, type SkillPage, type SkillReview } from '../ai-store/catalog.ts';
import { connectorIcon } from '../ai-store/connector-icons.ts';
import ExpertAvatar from '../ai-store/ExpertAvatar.vue';

const props = defineProps<{ store: ControlStore }>();
type Tab = 'experts' | 'skills' | 'connectors';
const tabs: { id: Tab; label: string }[] = [{ id: 'experts', label: '专家' }, { id: 'skills', label: '技能' }, { id: 'connectors', label: '连接器' }];
const tab = ref<Tab>('experts');
const query = ref('');
const category = ref('全部');
const installedOnly = ref(false);
const localSkills = ref<Skill[]>([]);
const remote = ref<SkillPage>({ results: [], page: 1, total: null, hasMore: false });
const page = ref(1);
const loading = ref(false);
const error = ref('');
const localError = ref('');
const computerEnabled = ref(false);
const localReady = ref(false);
const expert = ref<Expert | null>(null);
const skill = ref<Skill | null>(null);
const connector = ref<Connector | null>(null);
const computerOpen = ref(false);
const review = ref<SkillReview | null>(null);
const reviewError = ref('');
const reviewing = ref(false);
const reviewed = ref(false);
const removing = ref(false);
const busy = ref(false);
let browseId = 0;
let reviewId = 0;
let localLoading = false;
let timer: ReturnType<typeof setTimeout> | undefined;

const categories = computed(() => tab.value === 'experts' ? ['全部', '研究', '开发', '创作', '生活'] : tab.value === 'connectors' ? ['全部', '办公', '开发', '创作', '本机'] : installedOnly.value ? ['全部'] : ['全部', '办公', '创作', '工具', '自动化']);
const matches = (text: string) => text.toLocaleLowerCase().includes(query.value.trim().toLocaleLowerCase());
const visibleExperts = computed(() => experts.filter(e => (category.value === '全部' || e.cat === category.value) && matches([e.name, e.desc, ...e.tags].join(' '))));
const visibleConnectors = computed(() => connectors.filter(c => (category.value === '全部' || c.cat === category.value) && matches(c.name + c.desc)));
const showComputer = computed(() => ['全部', '本机'].includes(category.value) && matches('本机电脑控制 Computer Use 截图 鼠标 键盘'));
const visibleSkills = computed(() => installedOnly.value ? localSkills.value.filter(s => matches(s.name + s.description)) : remote.value.results);
const installed = (slug: string) => localSkills.value.find(s => s.slug === slug);
const heading = computed(() => tab.value === 'experts' ? '专家预设' : tab.value === 'connectors' ? '全部连接器' : installedOnly.value ? '已安装技能' : '发现技能');
const description = computed(() => tab.value === 'experts' ? '按擅长领域选择，开启一段专业对话' : tab.value === 'connectors' ? '连接你的工具与数据' : installedOnly.value ? `${visibleSkills.value.length} 项技能` : '来自 SkillHub');
const empty = computed(() => tab.value === 'experts' ? !visibleExperts.value.length : tab.value === 'connectors' ? !visibleConnectors.value.length && !showComputer.value : !loading.value && !visibleSkills.value.length);

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
async function browse() {
  clearTimeout(timer);
  const id = ++browseId;
  if (!props.store.desktop || tab.value !== 'skills' || installedOnly.value) { loading.value = false; return; }
  loading.value = true; error.value = ''; remote.value = { results: [], page: page.value, total: null, hasMore: false };
  try {
    const keyword = [query.value.trim(), category.value === '全部' ? '' : category.value].filter(Boolean).join(' ').slice(0, 80);
    const result = await invoke<SkillPage>('ai_store_browse', { keyword, page: page.value });
    if (id === browseId) remote.value = result;
  } catch (e) { if (id === browseId) error.value = `暂时无法加载 SkillHub：${String(e)}`; }
  finally { if (id === browseId) loading.value = false; }
}
function scheduleBrowse() {
  clearTimeout(timer); ++browseId; loading.value = false; error.value = '';
  if (tab.value === 'skills' && !installedOnly.value) {
    remote.value = { results: [], page: page.value, total: null, hasMore: false };
    loading.value = props.store.desktop;
    timer = setTimeout(() => void browse(), 250);
  }
}
function switchTab(value: Tab) { tab.value = value; category.value = '全部'; page.value = 1; }
function tabKey(event: KeyboardEvent) {
  if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
  event.preventDefault();
  const index = tabs.findIndex(t => t.id === tab.value);
  const next = event.key === 'Home' ? 0 : event.key === 'End' ? tabs.length - 1 : (index + (event.key === 'ArrowRight' ? 1 : -1) + tabs.length) % tabs.length;
  switchTab(tabs[next].id);
  (event.currentTarget as HTMLElement).querySelector<HTMLButtonElement>(`#store-tab-${tab.value}`)?.focus();
}
watch([query, category, installedOnly], () => { page.value = 1; scheduleBrowse(); });
watch(tab, () => { scheduleBrowse(); });
watch(page, () => { scheduleBrowse(); });
onActivated(() => { void refreshLocal(); if (tab.value === 'skills') void browse(); });
onMounted(() => { void refreshLocal(); });
onBeforeUnmount(() => { clearTimeout(timer); ++browseId; ++reviewId; });

async function startConversation(text: string) {
  if (busy.value) return;
  busy.value = true;
  try {
    await props.store.chat.select(null);
    props.store.creationDraft = text;
    expert.value = null; skill.value = null;
    props.store.navigate('chat');
  } catch (e) { props.store.notify(e, 'error'); }
  finally { busy.value = false; }
}
function talkToExpert(e: Expert) {
  return startConversation(`请作为${e.name}帮助我。${e.desc}\n请先了解我的具体目标、背景和约束，再提供适合我的方案。\n我的需求是：`);
}
function useSkill(s: Skill) { return startConversation(`请读取已安装的技能「${s.slug}」，按其中的流程帮助我完成以下任务：`); }
async function openSkill(s: Skill) {
  skill.value = s; review.value = null; reviewError.value = ''; reviewed.value = false; removing.value = false;
  const id = ++reviewId;
  if (installed(s.slug)) { reviewing.value = false; return; }
  if (!s.version) { reviewing.value = false; reviewError.value = '此技能暂未提供可安装的版本。'; return; }
  reviewing.value = true;
  try {
    const result = await invoke<SkillReview>('ai_store_inspect', { slug: s.slug, version: s.version });
    if (id === reviewId) review.value = result;
  } catch (e) { if (id === reviewId) reviewError.value = String(e); }
  finally { if (id === reviewId) reviewing.value = false; }
}
async function installSkill() {
  if (!skill.value || !review.value || !reviewed.value || busy.value || !localReady.value) return;
  busy.value = true;
  const s = skill.value; const r = review.value;
  try {
    await invoke('ai_store_install', { slug: s.slug, version: r.manifest.version, sha256: r.sha256 });
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
function reset() { query.value = ''; category.value = '全部'; installedOnly.value = false; }
const bytes = (n: number) => n < 1024 ? `${n} B` : `${(n / 1024).toFixed(1)} KB`;
</script>

<template>
  <section class="ai-store-page" data-testid="ai-store-page" aria-label="AI 商店">
    <header class="store-header">
      <form class="store-search" role="search" @submit.prevent="browse">
        <Search :size="17" aria-hidden="true" /><input v-model="query" maxlength="70" aria-label="搜索专家、技能或连接器" :placeholder="`搜索${tabs.find(t => t.id === tab)?.label}…`" />
        <button v-if="query" type="button" aria-label="清空搜索" @click="query = ''">×</button>
      </form>
      <div class="store-tabs" role="tablist" aria-label="商店分类" @keydown="tabKey">
        <button v-for="t in tabs" :id="`store-tab-${t.id}`" :key="t.id" type="button" role="tab" :aria-controls="`store-panel-${t.id}`" :aria-selected="tab === t.id" :tabindex="tab === t.id ? 0 : -1" :class="{ active: tab === t.id }" @click="switchTab(t.id)">{{ t.label }}</button>
      </div>
    </header>
    <div :id="`store-panel-${tab}`" class="store-content" role="tabpanel" :aria-labelledby="`store-tab-${tab}`">
      <div class="store-toolbar">
        <div class="store-filters" aria-label="按领域筛选">
          <button v-for="c in categories" :key="c" :class="{ active: category === c }" :aria-pressed="category === c" @click="category = c">{{ c }}</button>
        </div>
        <button v-if="tab === 'skills'" class="installed-filter" :class="{ selected: installedOnly }" :aria-pressed="installedOnly" @click="installedOnly = !installedOnly; category = '全部'"><SlidersHorizontal :size="13" />已安装<span>{{ localSkills.length }}</span></button>
      </div>
      <div class="store-section-heading"><h2>{{ heading }}</h2><span>{{ description }}</span></div>
      <NAlert v-if="!store.desktop && tab !== 'experts'" type="info" :show-icon="false">请在小伴桌面客户端中浏览技能和管理连接器。</NAlert>
      <NAlert v-if="localError" class="store-alert" type="warning" :show-icon="false">{{ localError }} <button class="text-action" @click="refreshLocal">重试</button></NAlert>
      <NAlert v-if="error && tab === 'skills' && !installedOnly" class="store-alert" type="error" :show-icon="false">{{ error }} <button class="text-action" @click="browse">重试</button></NAlert>

      <div v-if="tab === 'experts'" class="store-grid expert-grid">
        <article v-for="e in visibleExperts" :key="e.id" class="store-card expert-card">
          <div class="expert-top"><ExpertAvatar :id="e.id" /><div class="expert-meta"><h3>{{ e.name }}</h3><span class="store-tag">{{ e.role }}</span></div></div>
          <p class="expert-desc">{{ e.desc }}</p><div class="expert-special">擅长 <b>{{ e.tags.join(' · ') }}</b></div>
          <div class="card-actions"><button class="store-primary" :disabled="busy" @click="talkToExpert(e)">与专家对话 <ArrowRight :size="13" /></button><button class="store-secondary" :aria-label="`查看${e.name}详情`" @click="expert = e">详情</button></div>
        </article>
      </div>
      <template v-else-if="tab === 'skills'">
        <div v-if="loading" class="store-grid" aria-label="正在加载技能" aria-busy="true"><NSkeleton v-for="n in 8" :key="n" height="183px" :sharp="false" /></div>
        <div v-else class="store-grid skill-grid">
          <article v-for="s in visibleSkills" :key="s.slug" class="store-card skill-card">
            <div class="skill-top"><div class="skill-icon"><Puzzle :size="23" :stroke-width="1.7" /></div><div class="skill-copy"><h3>{{ s.name || s.slug }}</h3><span>{{ s.version ? `v${s.version}` : s.slug }}</span></div></div>
            <p class="skill-desc">{{ s.description || '查看技能说明与使用方法' }}</p>
            <div class="skill-bottom"><span class="store-tag">{{ s.source || 'SkillHub' }}</span><button class="small-action" :class="{ installed: installed(s.slug) }" :disabled="!store.desktop || !localReady" @click="openSkill(s)"><Check v-if="installed(s.slug)" :size="12" /><Download v-else :size="12" />{{ installed(s.slug) ? '已安装' : '安装' }}</button></div>
          </article>
        </div>
        <div v-if="!installedOnly && !error && !loading && store.desktop && (page > 1 || remote.hasMore)" class="store-pagination"><button :disabled="page <= 1" aria-label="上一页" @click="page--"><ChevronLeft :size="16" /></button><span>第 {{ page }} 页</span><button :disabled="!remote.hasMore" aria-label="下一页" @click="page++"><ChevronRight :size="16" /></button></div>
      </template>
      <div v-else class="store-grid connector-grid">
        <article v-if="showComputer" class="store-card connector-card">
          <div class="connector-top"><div class="provider-icon computer"><Monitor :size="25" /></div><span class="connection-status" :class="{ enabled: computerEnabled }">{{ computerEnabled ? '已启用' : '未启用' }}</span></div>
          <h3>本机电脑控制</h3><p class="connector-desc">通过截图、鼠标和键盘操作电脑上的应用</p>
          <div class="connector-bottom"><small>Computer Use</small><button class="small-action" :disabled="!store.desktop || !!localError" @click="computerOpen = true">{{ computerEnabled ? '管理' : '启用' }}</button></div>
        </article>
        <article v-for="c in visibleConnectors" :key="c.id" class="store-card connector-card">
          <div class="connector-top"><div class="provider-icon" :class="c.id"><img :src="connectorIcon(c.id)" alt="" /></div><span class="connection-status">尚未接入</span></div>
          <h3>{{ c.name }}</h3><p class="connector-desc">{{ c.desc }}</p>
          <div class="connector-bottom"><small>{{ c.kind }}</small><button class="small-action neutral" :aria-label="`查看${c.name}连接器详情`" @click="connector = c">查看详情</button></div>
        </article>
      </div>
      <div v-if="empty && !error && (tab !== 'skills' || store.desktop)" class="store-empty"><Search :size="26" /><b>{{ installedOnly ? '暂无匹配的已安装技能' : '没有找到匹配内容' }}</b><span>试试其他关键词或分类</span><button class="text-action" @click="reset">清除筛选</button></div>
    </div>

    <NModal :show="!!expert" preset="card" :title="expert?.name" class="store-dialog" style="width:510px;max-width:calc(100vw - 36px)" @update:show="!$event && (expert = null)">
      <template v-if="expert"><div class="detail-intro"><ExpertAvatar :id="expert.id" /><div><span class="store-tag">专家预设</span><p>{{ expert.desc }}</p></div></div><h4>擅长领域</h4><div class="detail-tags"><span v-for="tag in expert.tags" :key="tag" class="store-tag">{{ tag }}</span></div><h4>建议搭配的技能</h4><p>{{ expert.skills.join('、') }}</p><h4>相关连接器</h4><p>{{ expert.connectors.join('、') }}</p><p class="detail-note">以角色预设开启新会话。关联能力需安装或授权后使用。</p><div class="dialog-actions"><button class="store-secondary" @click="expert = null; switchTab('skills')">查看技能</button><button class="store-primary" :disabled="busy" @click="talkToExpert(expert)">与专家对话 <ArrowRight :size="14" /></button></div></template>
    </NModal>
    <NModal :show="!!skill" preset="card" :title="skill?.name || skill?.slug" class="store-dialog skill-dialog" :mask-closable="!busy" :closable="!busy" style="width:680px;max-width:calc(100vw - 36px)" @update:show="!$event && !busy && (skill = null)">
      <template v-if="skill"><p>{{ skill.description }}</p><div class="detail-tags"><span class="store-tag">{{ installed(skill.slug)?.source || 'SkillHub' }}</span><span v-if="skill.version" class="store-tag">v{{ skill.version }}</span></div>
        <div v-if="reviewing" class="review-loading"><RefreshCw :size="17" />正在读取固定版本的文件与脚本…</div>
        <NAlert v-if="reviewError" type="error" :show-icon="false">{{ reviewError }} <button v-if="!installed(skill.slug)" class="text-action" :disabled="busy" @click="openSkill(skill)">重试</button></NAlert>
        <template v-if="review && !installed(skill.slug)">
          <h4>技能说明</h4><pre class="review-content">{{ review.content }}</pre>
          <h4>文件清单 <small>{{ review.manifest.fileCount }} 个文件 · {{ bytes(review.manifest.totalBytes) }}</small></h4><ul class="review-files"><li v-for="file in review.manifest.files" :key="file.path"><div><code>{{ file.path }}</code><span>{{ bytes(file.size) }}{{ file.executableScript ? ' · 脚本' : '' }}</span></div><small>SHA256 {{ file.sha256 }}</small></li></ul>
          <details v-for="script in review.scripts" :key="script.path" class="script-preview"><summary>{{ script.path }} · 脚本预览{{ script.previewTruncated ? '（已截断）' : '' }}</summary><pre>{{ script.preview || '此脚本暂无文本预览，请在确认来源后再安装。' }}</pre></details>
          <p class="detail-note">安装不会执行脚本。脚本运行时需单独确认，并使用本机账户权限；文件校验不代表安全认证。</p>
          <label class="review-confirm"><input v-model="reviewed" type="checkbox" :disabled="busy" />我已查看此版本的说明、文件清单和脚本预览，同意安装。</label>
        </template>
        <div v-if="installed(skill.slug)" class="installed-detail"><Check :size="17" /><p>此技能已安装，可在聊天中新建任务使用。</p></div>
        <p v-if="removing" class="detail-note">确认卸载此技能？卸载后新任务将无法加载它，进行中的任务可能受影响。</p>
        <div class="dialog-actions"><template v-if="installed(skill.slug)"><button v-if="installed(skill.slug)?.removable" class="store-secondary" :disabled="busy" @click="removing ? uninstallSkill() : (removing = true)">{{ busy ? '正在卸载…' : removing ? '确认卸载' : '卸载' }}</button><button class="store-primary" :disabled="busy" @click="useSkill(skill)">使用技能 <ArrowRight :size="14" /></button></template><button v-else class="store-primary" :disabled="!review || !reviewed || reviewing || busy || !localReady" @click="installSkill">{{ busy ? '正在安装…' : '确认安装' }}<Download :size="14" /></button></div>
      </template>
    </NModal>
    <NModal :show="!!connector" preset="card" :title="connector?.name" class="store-dialog" style="width:460px;max-width:calc(100vw - 36px)" @update:show="!$event && (connector = null)">
      <template v-if="connector"><p>{{ connector.desc }}</p><h4>计划支持的能力</h4><ul class="connector-scopes"><li v-for="scope in connector.scopes" :key="scope">{{ scope }}</li></ul><p class="detail-note">此连接器尚未接入。开放后可在这里授权账号、选择权限并管理连接。</p><div class="dialog-actions"><button class="store-secondary" @click="connector = null">知道了</button><button class="store-primary" disabled>暂未开放</button></div></template>
    </NModal>
    <NModal v-model:show="computerOpen" preset="card" title="本机电脑控制" class="store-dialog" :mask-closable="!busy" :closable="!busy" style="width:460px;max-width:calc(100vw - 36px)">
      <p>允许小伴在任务中调用内置 Computer Use 引擎，通过截图、鼠标与键盘操作应用。</p><p class="detail-note">首次使用按系统提示授予屏幕录制与辅助功能权限。开关表示能力已启用，实际权限以系统授权为准。</p><div class="dialog-actions"><button class="store-secondary" :disabled="busy" @click="computerOpen = false">取消</button><button class="store-primary" :disabled="busy" @click="toggleComputer">{{ busy ? '正在保存…' : computerEnabled ? '停用电脑控制' : '启用电脑控制' }}</button></div>
    </NModal>
  </section>
</template>

<style scoped>
.ai-store-page{--store-red:var(--ui-primary);--store-soft:var(--ui-primary-soft);display:flex;flex-direction:column;flex:1;min-height:0;min-width:0;color:var(--ui-text-primary);background:var(--ui-surface);container-type:inline-size}
.store-header{padding:30px 38px 0;flex:none}.store-search{display:flex;align-items:center;gap:10px;width:min(480px,100%);height:40px;padding:0 12px;border:1px solid transparent;background:var(--ui-surface-alt);border-radius:10px;color:var(--ui-text-tertiary)}
.store-search:focus-within{border-color:var(--store-red);background:var(--ui-surface)}.store-search input{flex:1;min-width:0;font:inherit;font-size:12px;color:var(--ui-text-primary);background:transparent;border:0;outline:0}.store-search input::placeholder{color:var(--ui-text-tertiary)}
.ai-store-page button,.store-dialog button{font:inherit;cursor:pointer;border:0;color:inherit;background:none}.ai-store-page button:disabled,.store-dialog button:disabled{opacity:.5;cursor:not-allowed}.store-search button{font-size:20px;color:var(--ui-text-tertiary)}
.store-tabs{display:flex;width:420px;max-width:100%;padding:4px;background:var(--ui-surface-alt);border-radius:11px;margin:17px 0 23px;gap:2px}.store-tabs button{flex:1;height:35px;border-radius:8px;font-size:13px;font-weight:600;color:var(--ui-text-secondary)}.store-tabs button.active{background:var(--store-red);color:#fff;box-shadow:0 3px 9px color-mix(in srgb,var(--store-red) 20%,transparent)}.store-tabs button:hover:not(.active){background:var(--ui-border)}
.store-content{overflow:auto;flex:1;min-height:0;padding:0 38px 38px;scrollbar-width:thin;scrollbar-color:var(--ui-scrollbar) transparent}.store-toolbar{display:flex;align-items:center;justify-content:space-between;gap:12px;margin-bottom:18px}.store-filters{display:flex;align-items:center;flex-wrap:wrap;gap:7px}.store-filters button{padding:8px 13px;border-radius:8px;background:var(--ui-surface-alt);color:var(--ui-text-secondary);font-size:11px;white-space:nowrap}.store-filters button.active{background:var(--store-soft);color:var(--ui-primary-text);font-weight:700}.store-filters button:hover{color:var(--store-red)}
.installed-filter{display:flex;align-items:center;gap:5px;white-space:nowrap;font-size:11px!important;color:var(--ui-text-secondary)!important}.installed-filter.selected{color:var(--ui-primary-text)!important}.installed-filter span{padding:1px 5px;background:var(--ui-surface-alt);border-radius:5px}.store-section-heading{display:flex;align-items:center;justify-content:space-between;gap:12px;margin:0 0 14px}.store-section-heading h2{font-size:15px;letter-spacing:-.2px;margin:0;font-weight:650}.store-section-heading>span{font-size:11px;color:var(--ui-text-tertiary)}
.store-grid{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:12px;align-items:stretch}.store-card{min-width:0;background:var(--ui-surface);border:1px solid var(--ui-border);border-radius:13px;padding:16px;box-shadow:0 3px 18px color-mix(in srgb,var(--ui-text-primary) 3%,transparent);transition:box-shadow .18s,border-color .18s,transform .18s}.store-card:hover{border-color:color-mix(in srgb,var(--store-red) 24%,var(--ui-border));box-shadow:0 9px 28px color-mix(in srgb,var(--ui-text-primary) 6%,transparent);transform:translateY(-2px)}
.store-card h3{font-size:13px;font-weight:700;line-height:1.5;margin:0;overflow-wrap:anywhere}.expert-card{display:flex;flex-direction:column;min-height:223px;padding:15px}.expert-top{display:flex;align-items:flex-start;gap:11px}.expert-meta{min-width:0;padding-top:5px}.expert-meta h3{margin:0 0 7px}.expert-desc{color:var(--ui-text-secondary);font-size:11px;line-height:1.75;margin:12px 0 10px;min-height:38px}.expert-special{font-size:10px;color:var(--ui-text-tertiary);margin-bottom:14px}.expert-special b{font-weight:500;color:var(--ui-text-secondary);margin-left:5px}.store-tag{display:inline-block;color:var(--ui-text-tertiary);background:var(--ui-surface-alt);border-radius:5px;font-size:10px;padding:5px 7px;line-height:1.3}
.card-actions{display:flex;gap:7px;margin-top:auto}.card-actions .store-primary{flex:1;min-width:0;padding:0 10px}.store-primary,.store-secondary{display:inline-flex;align-items:center;justify-content:center;gap:6px;height:35px;min-height:35px;padding:0 15px;border-radius:9px;font-size:11px!important;font-weight:650!important;white-space:nowrap}.ai-store-page .store-primary,.store-dialog .store-primary{background:var(--ui-primary);color:#fff}.store-primary:hover:enabled{background:var(--ui-primary-hover)}.ai-store-page .store-secondary,.store-dialog .store-secondary{background:var(--ui-surface-alt);color:var(--ui-text-secondary)}.card-actions .store-secondary{padding:0 10px}
.skill-card{display:flex;flex-direction:column;min-height:183px}.skill-top{display:flex;align-items:flex-start;gap:10px}.skill-icon{width:44px;height:44px;border-radius:12px;background:var(--store-soft);color:var(--ui-primary-text);display:grid;place-items:center;flex:none}.skill-copy{min-width:0}.skill-copy h3{display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden}.skill-copy>span{display:block;font-size:10px;color:var(--ui-text-tertiary);margin-top:5px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.skill-desc{font-size:11px;color:var(--ui-text-secondary);line-height:1.65;margin:12px 0;display:-webkit-box;-webkit-line-clamp:3;-webkit-box-orient:vertical;overflow:hidden;overflow-wrap:anywhere}.skill-bottom{display:flex;justify-content:space-between;align-items:center;gap:8px;margin-top:auto;padding-top:5px}.small-action{display:flex;align-items:center;justify-content:center;gap:4px;padding:8px 10px;border-radius:8px;font-size:11px!important;font-weight:650!important;white-space:nowrap}.ai-store-page .small-action{color:var(--ui-primary-text);background:var(--store-soft)}.ai-store-page .small-action.installed,.ai-store-page .small-action.neutral{color:var(--ui-text-secondary);background:var(--ui-surface-alt)}
.connector-card{display:flex;flex-direction:column;min-height:193px;padding:17px}.connector-top{display:flex;align-items:flex-start;justify-content:space-between;gap:6px}.provider-icon{width:45px;height:45px;display:grid;place-items:center;flex:none;border-radius:12px;background:var(--ui-surface-alt);overflow:hidden}.provider-icon img{width:29px;height:29px}.provider-icon.computer{color:var(--ui-primary-text);background:var(--store-soft)}.connection-status{font-size:10px;padding:5px 6px;background:var(--ui-surface-alt);color:var(--ui-text-tertiary);border-radius:7px}.connection-status.enabled{color:var(--ui-accent)}.connector-card h3{margin:14px 0 6px}.connector-desc{font-size:11px;color:var(--ui-text-secondary);line-height:1.6;margin:0 0 12px;overflow-wrap:anywhere}.connector-bottom{display:flex;align-items:center;justify-content:space-between;gap:5px;margin-top:auto}.connector-bottom small{font-size:10px;color:var(--ui-text-tertiary)}
.store-alert{margin-bottom:16px}.text-action{color:var(--ui-primary-text)!important;font-size:12px!important}.store-empty{min-height:230px;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:12px;color:var(--ui-text-tertiary)}.store-empty b{font-size:13px;color:var(--ui-text-secondary)}.store-empty>span{font-size:11px}.store-pagination{display:flex;align-items:center;justify-content:center;gap:14px;margin:26px 0 0;font-size:12px;color:var(--ui-text-secondary)}.store-pagination button{display:grid;place-items:center;width:30px;height:30px;border-radius:8px;background:var(--ui-surface-alt)}
.store-dialog p{font-size:12px;line-height:1.85;color:var(--ui-text-secondary);margin:12px 0;overflow-wrap:anywhere}.store-dialog h4{font-size:13px;margin:22px 0 10px}.store-dialog h4 small{font-size:11px;font-weight:400;margin-left:8px;color:var(--ui-text-tertiary)}.detail-intro{display:flex;gap:15px;align-items:center}.detail-tags{display:flex;gap:7px;flex-wrap:wrap}.store-dialog .detail-note{padding:12px;background:var(--ui-surface-alt);border-radius:10px;color:var(--ui-text-secondary);font-size:11px}.dialog-actions{display:flex;gap:9px;justify-content:flex-end;margin-top:23px}.dialog-actions .store-primary{font-size:12px!important}.review-loading,.installed-detail{display:flex;align-items:center;gap:9px;font-size:12px;color:var(--ui-text-secondary);padding:15px 0}.review-content,.script-preview pre{font-size:11px;line-height:1.7;white-space:pre-wrap;overflow-wrap:anywhere;max-height:240px;overflow:auto;padding:12px;background:var(--ui-surface-alt);border-radius:8px}.review-files{list-style:none;margin:0;padding:0;max-height:200px;overflow:auto}.review-files li{padding:8px 0;border-bottom:1px solid var(--ui-border)}.review-files li>div{display:flex;justify-content:space-between;gap:12px;font-size:11px}.review-files code{overflow-wrap:anywhere}.review-files small{display:block;font-size:9px;color:var(--ui-text-tertiary);word-break:break-all;margin-top:4px}.script-preview{font-size:11px;padding:10px 0}.script-preview summary{cursor:pointer;overflow-wrap:anywhere}.review-confirm{display:flex;gap:8px;align-items:flex-start;font-size:12px;line-height:1.7}.review-confirm input{accent-color:var(--ui-primary);margin-top:4px;flex:none}.connector-scopes{font-size:12px;color:var(--ui-text-secondary);line-height:2;padding-left:20px}
@container(max-width:1120px){.store-grid{grid-template-columns:repeat(3,minmax(0,1fr))}}
@container(max-width:820px){.store-grid{grid-template-columns:repeat(2,minmax(0,1fr))}.store-header{padding:24px 24px 0}.store-content{padding:0 24px 28px}}
@container(max-width:520px){.store-grid{grid-template-columns:minmax(0,1fr)}.store-header{padding:20px 16px 0}.store-content{padding:0 16px 24px}.store-toolbar{align-items:flex-start}.store-filters{gap:5px}.store-filters button{padding:7px 10px}.store-section-heading>span{display:none}.installed-filter{padding-top:7px!important}}
@media(prefers-reduced-motion:reduce){.store-card{transition:none}.store-card:hover{transform:none}}
</style>
