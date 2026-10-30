<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { Check, ChevronRight, Camera, Paperclip, Plus, Search, UserRound, Zap, Plug, ArrowUpRight, Monitor, Smartphone, PanelsTopLeft, Hammer, Settings2 } from 'lucide-vue-next';
import { invoke, type AiConfigView } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import type { Skill, SkillPage } from '../ai-store/catalog.ts';
import { loadConnectorInventory, type ConnectorInventoryEntry } from '../chat/connector-inventory.ts';
import type { BuiltinTool } from '../chat/builtin-tools.ts';

type CatalogKind = 'experts' | 'skills' | 'connectors';
type Kind = CatalogKind | 'tools';
interface Expert { slug: string; name: string; description: string; entrySkill?: string; ready?: boolean; plugin?: string; profession?: string; agent?: string | null; quickPrompts?: string[]; tags?: string[]; hasAvatar?: boolean; kind?: 'agent' | 'team' }
interface CatalogEntry { slug: string; name: string; description: string }
interface CatalogPage { results: CatalogEntry[] }
type Entry = { id: string; title: string; detail: string; installed: boolean; ready: boolean; entrySkill?: string; expert?: Expert };

/** 一级菜单右侧显示当前选择的摘要，例如技能名、专家名、已启用的连接器数。 */
interface Summary { skills?: string; experts?: string; connectors?: string; tools?: string }
const props = defineProps<{ store: ControlStore; disabled?: boolean; fileDisabled?: boolean; screenshotDisabled?: boolean; summary?: Summary }>();
const emit = defineEmits<{
  file: []; screenshot: [];
  'use-expert': [entrySkill: string, name: string, expert?: Expert];
  'use-skill': [slug: string, name: string];
  'use-connector': [name: string, id: string];
  'use-tool': [tool: BuiltinTool];
  'check-computer-permissions': [];
  'computer-disabled': [];
}>();
const root = ref<HTMLElement | null>(null);
let closeTimer: ReturnType<typeof setTimeout> | undefined;
const panel = ref<'add' | null>(null);
const kind = ref<Kind | null>(null);
const term = ref('');
const loading = ref(false);
const searchError = ref('');
const installedErrors = ref<Partial<Record<CatalogKind, string>>>({});
const installedLoading = ref<Record<CatalogKind, boolean>>({ experts: false, skills: false, connectors: false });
const error = computed(() => [kind.value && kind.value !== 'tools' ? installedErrors.value[kind.value] : '', searchError.value].filter(Boolean).join('；'));
const experts = ref<Expert[]>([]);
const skills = ref<Skill[]>([]);
const connectors = ref<ConnectorInventoryEntry[]>([]);
const discovered = ref<CatalogEntry[]>([]);
const computerPanel = ref(false);
const computerEnabled = ref(false);
const computerBusy = ref(false);
let computerSelectOnEnable = false;
let computerOwner: unknown;
let requestId = 0;
let installedId = 0;
let searchTimer: ReturnType<typeof setTimeout> | undefined;
const labels: Record<Kind, string> = { experts: '专家', skills: '技能', connectors: '连接器', tools: '工具' };

const installedEntries = computed<Entry[]>(() => {
  const q = term.value.trim().toLowerCase();
  const matches = (value: string) => value.toLowerCase().includes(q);
  if (kind.value === 'experts') return experts.value.filter(e => matches(e.name + ' ' + e.description)).map(e => ({
    id: e.slug, title: e.name, detail: [e.profession, e.description].filter(Boolean).join(' · '), installed: true, ready: e.ready !== false, entrySkill: e.entrySkill, expert: e.plugin ? e : undefined,
  }));
  if (kind.value === 'skills') return skills.value.filter(e => matches(e.name + ' ' + e.slug + ' ' + e.description)).map(e => ({
    id: e.slug, title: e.name, detail: e.description, installed: true, ready: true,
  }));
  if (kind.value === 'connectors') return connectors.value.filter(e => matches(e.name + ' ' + e.id)).map(e => ({
    id: e.id, title: e.name, detail: e.detail || (e.connected && e.enabled ? `已连接 · ${e.toolCount} 个工具` : '尚未连接或未启用'),
    installed: true, ready: e.connected && e.enabled,
  }));
  return [];
});
const remoteEntries = computed<Entry[]>(() => {
  const existing = new Set(installedEntries.value.map(e => e.id));
  return discovered.value.filter(e => !existing.has(e.slug)).map(e => ({
    id: e.slug, title: e.name, detail: e.description, installed: false, ready: false,
  }));
});
function cancelClose() { if (closeTimer !== undefined) clearTimeout(closeTimer); closeTimer = undefined; }
function scheduleClose() {
  cancelClose();
  closeTimer = setTimeout(() => {
    closeTimer = undefined;
    const active = document.activeElement;
    const trigger = root.value?.querySelector('.wb-plus');
    if (active && active !== trigger && root.value?.contains(active)) return;
    close();
  }, 180);
}
function onFocusOut(event: FocusEvent) {
  if (event.relatedTarget instanceof Node && root.value?.contains(event.relatedTarget)) return;
  scheduleClose();
}
function close() { cancelClose(); panel.value = null; kind.value = null; term.value = ''; computerPanel.value = false; ++requestId; ++installedId; loading.value = false; clearTimeout(searchTimer); }
function onOutside(e: PointerEvent) { if (root.value && !root.value.contains(e.target as Node)) close(); }
function onKey(e: KeyboardEvent) { if (e.key === 'Escape') close(); }
watch(panel, p => {
  if (p) { document.addEventListener('pointerdown', onOutside); document.addEventListener('keydown', onKey); }
  else { document.removeEventListener('pointerdown', onOutside); document.removeEventListener('keydown', onKey); }
});
watch(() => props.store.chat?.draft, () => { computerPanel.value = false; ++requestId; });
onBeforeUnmount(() => { ++requestId; ++installedId; clearTimeout(searchTimer); cancelClose(); document.removeEventListener('pointerdown', onOutside); document.removeEventListener('keydown', onKey); });
function openAddMenu() {
  cancelClose();
  if (props.disabled || panel.value === 'add') return;
  close(); panel.value = 'add';
  void refreshInstalled();
}
async function refreshInstalled() {
  if (!props.store.desktop) return;
  const id = ++installedId;
  installedLoading.value = { experts: true, skills: true, connectors: true }; installedErrors.value = {};
  experts.value = []; skills.value = []; connectors.value = [];
  async function read(k: CatalogKind, fetch: () => Promise<void>) {
    try { await fetch(); }
    catch { if (id === installedId) installedErrors.value[k] = `读取已安装${labels[k]}失败，请重试`; }
    finally { if (id === installedId) installedLoading.value[k] = false; }
  }
  await Promise.all([
    read('experts', async () => { const result = await invoke<Expert[]>('ai_store_experts_installed'); if (!Array.isArray(result)) throw new Error(); if (id === installedId) experts.value = result; }),
    read('skills', async () => { const result = await invoke<{ name: string; displayName: string; description: string }[]>('chat_invocable_skills'); if (!Array.isArray(result)) throw new Error(); if (id === installedId) skills.value = result.map(i => ({ slug: i.name, name: i.displayName, description: i.description }) as Skill); }),
    read('connectors', async () => { const result = await loadConnectorInventory(); if (id === installedId) { connectors.value = result.items; installedErrors.value.connectors = result.warning; } }),
  ]);
}
function chooseKind(next: Kind) {
  if (kind.value === next) return;
  kind.value = next; term.value = ''; discovered.value = []; searchError.value = ''; loading.value = false; computerPanel.value = false;
  ++requestId; clearTimeout(searchTimer);
}
function scheduleSearch() {
  clearTimeout(searchTimer);
  const id = ++requestId, q = term.value.trim().slice(0,80), selected = kind.value;
  discovered.value = []; searchError.value = ''; loading.value = false;
  if (!selected || selected === 'tools' || !q || !props.store.desktop) return;
  searchTimer = setTimeout(async () => {
    loading.value = true;
    try {
      const result = selected === 'skills'
        ? await invoke<SkillPage>('ai_store_browse', { keyword: q, page: 1, category: '', source: '', sortBy: 'score' })
        : await invoke<CatalogPage>('ai_store_catalog', { kind: selected, keyword: q, page: 1, category: '' });
      if (id === requestId) discovered.value = result.results.map(e => ({ slug: e.slug, name: e.name, description: e.description }));
    } catch {
      if (id === requestId) searchError.value = '搜索暂时不可用，可以前往 AI 插件查找';
    } finally { if (id === requestId) loading.value = false; }
  }, 250);
}
function useEntry(e: Entry) {
  if (!e.installed) { openStore(); return; }
  if (!e.ready) { props.store.notify('该能力尚未就绪，请先到 AI 插件完成配置', 'info'); openStore(); return; }
  if (kind.value === 'experts') emit('use-expert', e.entrySkill || e.id, e.title, e.expert);
  if (kind.value === 'skills') emit('use-skill', e.id, e.title);
  if (kind.value === 'connectors') emit('use-connector', e.title, e.id);
  close();
}
function openStore() { close(); props.store.navigate('ai-store'); }
function attachFile() { close(); emit('file'); }
function takeScreenshot() { close(); emit('screenshot'); }
async function readComputer(select: boolean) {
  if (computerBusy.value) return;
  computerBusy.value = true;
  const current = requestId, owner = props.store.chat?.draft;
  try {
    const config = await invoke<AiConfigView>('get_ai_config');
    if (current !== requestId || kind.value !== 'tools' || !panel.value || owner !== props.store.chat?.draft) return;
    computerEnabled.value = config.openComputerUse;
    if (select && computerEnabled.value) { emit('use-tool', 'computer'); close(); return; }
    computerSelectOnEnable = select;
    computerOwner = owner;
    computerPanel.value = true;
  } catch {
    if (current === requestId && kind.value === 'tools' && owner === props.store.chat?.draft) props.store.notify('无法读取电脑控制状态，请重试', 'error');
  } finally { computerBusy.value = false; }
}
async function toggleComputer() {
  if (computerBusy.value) return;
  computerBusy.value = true;
  const current = requestId, enabled = !computerEnabled.value;
  try {
    const config = await invoke<AiConfigView>('save_ai_preferences', { openComputerUse: enabled });
    props.store.notify(config.openComputerUse ? '已启用电脑控制，使用时按系统提示授权' : '已停用电脑控制');
    if (current !== requestId || kind.value !== 'tools' || !panel.value || computerOwner !== props.store.chat?.draft) return;
    computerEnabled.value = config.openComputerUse;
    if (!config.openComputerUse) emit('computer-disabled');
    if (config.openComputerUse && computerSelectOnEnable) emit('use-tool', 'computer');
    close();
  } catch (e) {
    props.store.notify(`无法保存电脑控制设置：${String(e)}`, 'error');
  } finally { computerBusy.value = false; }
}
function useTool(tool: BuiltinTool) {
  if (tool === 'computer') { void readComputer(true); return; }
  emit('use-tool', tool); close();
}
function checkComputerPermissions() {
  if (computerOwner !== props.store.chat?.draft) { close(); return; }
  emit('check-computer-permissions'); close();
}
</script>

<template>
  <div ref="root" class="wb-controls" @mouseenter="cancelClose" @mouseleave="scheduleClose" @focusout="onFocusOut">
    <div class="wb-anchor">
      <button type="button" class="wb-tool wb-plus" data-testid="chat-add-menu" aria-label="添加附件或能力" aria-haspopup="menu" :aria-expanded="panel === 'add'" :disabled="disabled" @mouseenter="openAddMenu" @focus="openAddMenu" @click="openAddMenu"><Plus :size="18" /></button>
      <div v-show="panel === 'add'" class="wb-menu xb-menu wb-main" role="menu" aria-label="添加" @mouseenter="cancelClose">
        <button type="button" class="wb-menu-item" data-testid="chat-add-screenshot" role="menuitem" :disabled="screenshotDisabled" @click="takeScreenshot"><i class="wb-tile" aria-hidden="true"><Camera :size="15" /></i><span>截图</span></button>
        <button type="button" class="wb-menu-item" data-testid="chat-pick-image" role="menuitem" :disabled="fileDisabled" @click="attachFile"><i class="wb-tile" aria-hidden="true"><Paperclip :size="15" /></i><span>添加文件或图片</span></button>
        <div class="wb-separator" />
        <button type="button" class="wb-menu-item" data-testid="chat-add-skills" role="menuitem" :class="{ selected: kind === 'skills' }" @mouseenter="chooseKind('skills')" @focus="chooseKind('skills')" @click="chooseKind('skills')"><i class="wb-tile cap" aria-hidden="true"><Zap :size="15" /></i><span>技能</span><small v-if="summary?.skills" class="wb-summary">{{ summary.skills }}</small><ChevronRight :size="14" /></button>
        <button type="button" class="wb-menu-item" data-testid="chat-add-experts" role="menuitem" :class="{ selected: kind === 'experts' }" @mouseenter="chooseKind('experts')" @focus="chooseKind('experts')" @click="chooseKind('experts')"><i class="wb-tile cap" aria-hidden="true"><UserRound :size="15" /></i><span>专家</span><small v-if="summary?.experts" class="wb-summary">{{ summary.experts }}</small><ChevronRight :size="14" /></button>
        <button type="button" class="wb-menu-item" data-testid="chat-add-connectors" role="menuitem" :class="{ selected: kind === 'connectors' }" @mouseenter="chooseKind('connectors')" @focus="chooseKind('connectors')" @click="chooseKind('connectors')"><i class="wb-tile cap" aria-hidden="true"><Plug :size="15" /></i><span>连接器</span><small v-if="summary?.connectors" class="wb-summary">{{ summary.connectors }}</small><ChevronRight :size="14" /></button>
        <button type="button" class="wb-menu-item" data-testid="chat-add-tools" role="menuitem" :class="{ selected: kind === 'tools' }" @mouseenter="chooseKind('tools')" @focus="chooseKind('tools')" @click="chooseKind('tools')"><i class="wb-tile cap" aria-hidden="true"><Hammer :size="15" /></i><span>工具</span><small v-if="summary?.tools" class="wb-summary">{{ summary.tools }}</small><ChevronRight :size="14" /></button>
        <div v-if="kind === 'tools'" class="wb-menu xb-menu wb-submenu wb-builtin-tools" :class="{ 'wb-computer-panel': computerPanel }" role="group" :aria-label="computerPanel ? '电脑控制' : '工具'" @mouseenter="cancelClose">
          <template v-if="!computerPanel">
            <div class="wb-computer-row"><button type="button" class="wb-menu-item" data-testid="chat-tool-computer" :disabled="computerBusy" @click="useTool('computer')"><Monitor :size="15" /><span>操作电脑</span></button><button type="button" class="wb-computer-manage" data-testid="chat-tool-computer-manage" aria-label="管理电脑控制" title="管理电脑控制" :disabled="computerBusy" @click="readComputer(false)"><Settings2 :size="15" /></button></div>
            <button type="button" class="wb-menu-item" data-testid="chat-tool-phone" @click="useTool('phone')"><Smartphone :size="15" /><span>操作手机</span></button>
            <button type="button" class="wb-menu-item" data-testid="chat-tool-design" @click="useTool('design')"><PanelsTopLeft :size="15" /><span>设计画板</span></button>
          </template>
          <div v-else class="wb-computer-info" data-testid="chat-computer-settings">
            <strong>电脑控制 · {{ computerEnabled ? '已启用' : '未启用' }}</strong>
            <p>允许小伴操作电脑应用。首次使用时按系统提示授权屏幕录制和辅助功能，实际权限以系统为准。</p>
            <button v-if="computerEnabled" type="button" class="wb-computer-check" data-testid="chat-computer-check" :disabled="computerBusy" @click="checkComputerPermissions">让小伴检查权限</button>
            <div class="wb-computer-actions"><button type="button" class="wb-computer-secondary" :disabled="computerBusy" @click="computerPanel = false">返回</button><button type="button" class="wb-computer-primary" data-testid="chat-computer-toggle" :disabled="computerBusy" @click="toggleComputer">{{ computerBusy ? '正在保存…' : computerEnabled ? '停用' : '启用' }}</button></div>
          </div>
        </div>
        <div v-else-if="kind" class="wb-menu xb-menu wb-submenu" role="group" :aria-label="labels[kind]" @mouseenter="cancelClose">
          <label class="wb-search"><Search :size="14" /><input v-model="term" data-testid="chat-capability-search" :placeholder="`搜索${labels[kind]}…`" maxlength="80" @input="scheduleSearch" /></label>
          <div class="wb-results">
            <div class="wb-section">已安装的{{ labels[kind] }}</div>
            <button v-for="entry in installedEntries" :key="'i-'+entry.id" type="button" class="wb-result" :data-testid="`chat-installed-${entry.id}`" :title="entry.detail || entry.id" :aria-description="entry.detail || entry.id" @click="useEntry(entry)">
              <span class="wb-result-text"><strong>{{ entry.title }}</strong></span>
              <Check v-if="entry.ready" :size="14" /><ChevronRight v-else :size="14" />
            </button>
            <div v-if="installedLoading[kind]" class="wb-empty" role="status">正在读取已安装的{{ labels[kind] }}…</div>
            <div v-else-if="!installedEntries.length && !installedErrors[kind]" class="wb-empty">{{ term ? '没有匹配的已安装能力' : '暂无已安装能力' }}</div>
            <div v-if="error" class="wb-empty" role="alert">{{ error }} <button v-if="installedErrors[kind]" type="button" class="wb-store-link" @click="refreshInstalled">重试</button></div>
            <template v-if="term">
              <div class="wb-section">发现更多</div>
              <div v-if="loading" class="wb-empty">搜索中…</div>
              <button v-for="entry in remoteEntries" :key="'r-'+entry.id" type="button" class="wb-result" :title="entry.detail || '前往 AI 插件查看'" :aria-description="entry.detail || '前往 AI 插件查看'" @click="openStore">
                <span class="wb-result-text"><strong>{{ entry.title }}</strong></span><ArrowUpRight :size="14" />
              </button>
              <div v-if="!error && !loading && !remoteEntries.length && term" class="wb-empty">可在 AI 插件中继续查找</div>
            </template>
          </div>
          <button type="button" class="wb-store-link" data-testid="chat-open-ai-store" @click="openStore">打开 AI 插件 <ArrowUpRight :size="13" /></button>
        </div>
      </div>
    </div>
  </div>
</template>


<style scoped>
.wb-controls{display:flex;align-items:center;gap:2px;flex:none}
.wb-anchor{position:relative;display:flex;align-items:center}
.wb-tool{display:flex;align-items:center;justify-content:center;gap:5px;height:30px;border:0;border-radius:8px;background:none;color:var(--ui-text-secondary);font:inherit;font-size:12px;cursor:pointer;white-space:nowrap}
.wb-tool:hover,.wb-tool[aria-expanded="true"]{background:var(--ui-control);color:var(--ui-text-primary)}
.wb-tool:disabled{opacity:.5;cursor:not-allowed}
.wb-plus{width:30px;padding:0}
.wb-menu{position:absolute;box-sizing:border-box}
.wb-main{left:0;bottom:calc(100% + 8px);width:248px;min-width:0}
.wb-menu-item{display:flex;gap:10px;align-items:center;width:100%;height:34px;padding:0 10px;background:none;border:0;border-radius:8px;color:var(--ui-text-primary);font:inherit;font-size:13px;text-align:left;cursor:pointer;white-space:nowrap}
.wb-menu-item>svg:first-child{color:var(--ui-text-secondary)}
.wb-main>.wb-menu-item{height:38px}
.wb-tile{display:grid;place-items:center;flex:none;width:26px;height:26px;border-radius:7px;background:var(--ui-control);color:var(--ui-text-secondary);font-style:normal}
.wb-tile.cap{background:var(--ui-soft);color:var(--ui-ink)}
.wb-summary{max-width:96px;overflow:hidden;text-overflow:ellipsis;font-size:12px;color:var(--ui-text-tertiary)}
.wb-menu-item:disabled{opacity:.45;cursor:not-allowed}.wb-menu-item span:first-of-type{flex:1}.wb-menu-item:hover,.wb-menu-item.selected{background:var(--ui-surface-hover)}
.wb-menu-item.selected{background:var(--ui-soft);color:var(--ui-ink)}
.wb-menu-item.selected>svg{color:var(--ui-ink)}
.wb-menu-item svg:last-child:not(:first-child){margin-left:auto;color:var(--ui-text-tertiary)}
.wb-separator{height:1px;background:var(--ui-line);margin:5px 6px}
.wb-submenu{left:calc(100% + 7px);bottom:0;width:min(280px,calc(100vw - 210px));min-width:205px;max-height:360px;display:flex;flex-direction:column}
.wb-builtin-tools{width:168px;min-width:0;max-height:none}
.wb-computer-row{display:flex;align-items:center}.wb-computer-row>.wb-menu-item{min-width:0;flex:1}.wb-computer-manage{display:grid;place-items:center;flex:none;width:30px;height:30px;padding:0;border:0;border-radius:8px;background:none;color:var(--ui-text-secondary);cursor:pointer}.wb-computer-manage:hover,.wb-computer-manage:focus-visible{background:var(--ui-surface-hover)}.wb-computer-manage:disabled{opacity:.45;cursor:not-allowed}
.wb-computer-panel{width:min(260px,calc(100vw - 45px))}.wb-computer-info{padding:8px;color:var(--ui-text-primary);font-size:12px}.wb-computer-info strong{font-size:13px;font-weight:500}.wb-computer-info p{line-height:1.5;color:var(--ui-text-secondary);margin:8px 0}.wb-computer-check{padding:2px 0;border:0;background:none;color:var(--ui-ink);font:inherit;cursor:pointer}.wb-computer-actions{display:flex;justify-content:flex-end;gap:8px;margin-top:12px}.wb-computer-actions button{border:0;border-radius:8px;padding:6px 12px;font:inherit;cursor:pointer}.wb-computer-secondary{background:var(--ui-control);color:var(--ui-text-primary)}.wb-computer-primary{background:var(--ui-btn);color:#fff}.wb-computer-actions button:disabled{opacity:.5;cursor:not-allowed}
.wb-search{display:flex;gap:7px;align-items:center;margin:3px 3px 5px;border-radius:8px;background:var(--ui-control);padding:7px 9px;color:var(--ui-text-tertiary)}
.wb-search input{background:none;border:0;outline:none;min-width:0;flex:1;color:var(--ui-text-primary);font:inherit;font-size:12.5px}
.wb-results{flex:1;min-height:0;overflow-y:auto;scrollbar-width:thin}
.wb-section{font-size:11.5px;font-weight:500;color:var(--ui-text-tertiary);padding:8px 8px 4px}
.wb-result{display:flex;align-items:center;gap:8px;width:100%;padding:8px;border-radius:8px;border:0;background:none;color:var(--ui-text-primary);font:inherit;cursor:pointer;text-align:left}
.wb-result:hover{background:var(--ui-surface-hover)}
.wb-result-text{flex:1;min-width:0;display:grid}
.wb-result-text strong{font-size:12.5px;font-weight:500;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.wb-result>svg{flex:none;color:var(--ui-text-tertiary)}
.wb-empty{padding:11px 8px;font-size:11.5px;color:var(--ui-text-tertiary)}
.wb-store-link{display:flex;align-items:center;justify-content:center;gap:5px;padding:9px 5px 6px;border:0;border-top:1px solid var(--ui-line);background:none;color:var(--ui-ink);font:inherit;font-size:12.5px;cursor:pointer}
@media(max-width:650px){.wb-submenu{left:0;bottom:calc(100% + 8px);width:min(280px,calc(100vw - 45px))}.wb-builtin-tools{width:168px}.wb-computer-panel{width:min(260px,calc(100vw - 45px))}}
</style>
