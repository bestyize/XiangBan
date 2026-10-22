<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { Check, ChevronRight, Camera, FilePlus2, Plus, Search, Sparkles, Wrench, Plug, ArrowUpRight } from 'lucide-vue-next';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import type { Skill, SkillPage } from '../ai-store/catalog.ts';

type Kind = 'experts' | 'skills' | 'connectors';
interface Expert { slug: string; name: string; description: string; entrySkill?: string; ready?: boolean }
interface Connector { id: string; name: string; connected: boolean; enabled: boolean; toolCount: number }
interface CatalogEntry { slug: string; name: string; description: string }
interface CatalogPage { results: CatalogEntry[] }
type Entry = { id: string; title: string; detail: string; installed: boolean; ready: boolean; entrySkill?: string };

const props = defineProps<{ store: ControlStore; disabled?: boolean; fileDisabled?: boolean; screenshotDisabled?: boolean }>();
const emit = defineEmits<{
  file: []; screenshot: [];
  'use-expert': [entrySkill: string, name: string];
  'use-skill': [slug: string, name: string];
  'use-connector': [name: string];
}>();
const root = ref<HTMLElement | null>(null);
let closeTimer: ReturnType<typeof setTimeout> | undefined;
const panel = ref<'add' | null>(null);
const kind = ref<Kind | null>(null);
const term = ref('');
const loading = ref(false);
const error = ref('');
const experts = ref<Expert[]>([]);
const skills = ref<Skill[]>([]);
const connectors = ref<Connector[]>([]);
const discovered = ref<CatalogEntry[]>([]);
let requestId = 0;
let searchTimer: ReturnType<typeof setTimeout> | undefined;
const labels: Record<Kind, string> = { experts: '专家', skills: '技能', connectors: '连接器' };

const installedEntries = computed<Entry[]>(() => {
  const q = term.value.trim().toLowerCase();
  const matches = (value: string) => value.toLowerCase().includes(q);
  if (kind.value === 'experts') return experts.value.filter(e => matches(e.name + ' ' + e.description)).map(e => ({
    id: e.slug, title: e.name, detail: e.description, installed: true, ready: e.ready !== false, entrySkill: e.entrySkill,
  }));
  if (kind.value === 'skills') return skills.value.filter(e => matches(e.name + ' ' + e.slug + ' ' + e.description)).map(e => ({
    id: e.slug, title: e.name, detail: e.description, installed: true, ready: true,
  }));
  if (kind.value === 'connectors') return connectors.value.filter(e => matches(e.name + ' ' + e.id)).map(e => ({
    id: e.id, title: e.name, detail: e.connected && e.enabled ? `已连接 · ${e.toolCount} 个工具` : '尚未连接或未启用',
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
function close() { cancelClose(); panel.value = null; kind.value = null; term.value = ''; ++requestId; clearTimeout(searchTimer); }
function onOutside(e: PointerEvent) { if (root.value && !root.value.contains(e.target as Node)) close(); }
function onKey(e: KeyboardEvent) { if (e.key === 'Escape') close(); }
watch(panel, p => {
  if (p) { document.addEventListener('pointerdown', onOutside); document.addEventListener('keydown', onKey); }
  else { document.removeEventListener('pointerdown', onOutside); document.removeEventListener('keydown', onKey); }
});
onBeforeUnmount(() => { ++requestId; clearTimeout(searchTimer); cancelClose(); document.removeEventListener('pointerdown', onOutside); document.removeEventListener('keydown', onKey); });
function openAddMenu() {
  cancelClose();
  if (props.disabled || panel.value === 'add') return;
  close(); panel.value = 'add';
  void refreshInstalled();
}
async function refreshInstalled() {
  if (!props.store.desktop) return;
  const id = ++requestId;
  const results = await Promise.allSettled([
    invoke<Expert[]>('ai_store_experts_installed'),
    invoke<Skill[]>('ai_store_installed'),
    invoke<Connector[]>('ai_store_connector_status'),
  ]);
  if (id !== requestId) return;
  if (results[0].status === 'fulfilled') experts.value = results[0].value;
  if (results[1].status === 'fulfilled') skills.value = results[1].value;
  if (results[2].status === 'fulfilled') connectors.value = results[2].value;
  error.value = results.every(r => r.status === 'rejected') ? '读取已安装能力失败，请重试' : '';
}
function chooseKind(next: Kind) {
  if (kind.value === next) return;
  kind.value = next; term.value = ''; discovered.value = []; error.value = '';
  ++requestId; clearTimeout(searchTimer);
}
function scheduleSearch() {
  clearTimeout(searchTimer);
  const id = ++requestId, q = term.value.trim().slice(0,80), selected = kind.value;
  discovered.value = []; error.value = ''; loading.value = false;
  if (!selected || !q || !props.store.desktop) return;
  searchTimer = setTimeout(async () => {
    loading.value = true;
    try {
      const result = selected === 'skills'
        ? await invoke<SkillPage>('ai_store_browse', { keyword: q, page: 1, category: '', source: '', sortBy: 'score' })
        : await invoke<CatalogPage>('ai_store_catalog', { kind: selected, keyword: q, page: 1, category: '' });
      if (id === requestId) discovered.value = result.results.map(e => ({ slug: e.slug, name: e.name, description: e.description }));
    } catch {
      if (id === requestId) error.value = '搜索暂时不可用，可以前往 AI 插件查找';
    } finally { if (id === requestId) loading.value = false; }
  }, 250);
}
function useEntry(e: Entry) {
  if (!e.installed) { openStore(); return; }
  if (!e.ready) { props.store.notify('该能力尚未就绪，请先到 AI 插件完成配置', 'info'); openStore(); return; }
  if (kind.value === 'experts') emit('use-expert', e.entrySkill || e.id, e.title);
  if (kind.value === 'skills') emit('use-skill', e.id, e.title);
  if (kind.value === 'connectors') emit('use-connector', e.title);
  close();
}
function openStore() { close(); props.store.navigate('ai-store'); }
function attachFile() { close(); emit('file'); }
function takeScreenshot() { close(); emit('screenshot'); }
</script>

<template>
  <div ref="root" class="wb-controls" @mouseenter="cancelClose" @mouseleave="scheduleClose" @focusout="onFocusOut">
    <div class="wb-anchor">
      <button type="button" class="wb-tool wb-plus" data-testid="chat-add-menu" aria-label="添加附件或能力" aria-haspopup="menu" :aria-expanded="panel === 'add'" :disabled="disabled" @mouseenter="openAddMenu" @focus="openAddMenu" @click="openAddMenu"><Plus :size="18" /></button>
      <div v-show="panel === 'add'" class="wb-menu xb-menu wb-main" role="menu" aria-label="添加" @mouseenter="cancelClose">
        <button type="button" class="wb-menu-item" data-testid="chat-add-screenshot" role="menuitem" :disabled="screenshotDisabled" @click="takeScreenshot"><Camera :size="15" /><span>截图</span></button>
        <button type="button" class="wb-menu-item" data-testid="chat-pick-image" role="menuitem" :disabled="fileDisabled" @click="attachFile"><FilePlus2 :size="15" /><span>文件</span></button>
        <div class="wb-separator" />
        <button type="button" class="wb-menu-item" data-testid="chat-add-experts" role="menuitem" :class="{ selected: kind === 'experts' }" @mouseenter="chooseKind('experts')" @focus="chooseKind('experts')" @click="chooseKind('experts')"><Sparkles :size="15" /><span>专家</span><ChevronRight :size="14" /></button>
        <button type="button" class="wb-menu-item" data-testid="chat-add-skills" role="menuitem" :class="{ selected: kind === 'skills' }" @mouseenter="chooseKind('skills')" @focus="chooseKind('skills')" @click="chooseKind('skills')"><Wrench :size="15" /><span>技能</span><ChevronRight :size="14" /></button>
        <button type="button" class="wb-menu-item" data-testid="chat-add-connectors" role="menuitem" :class="{ selected: kind === 'connectors' }" @mouseenter="chooseKind('connectors')" @focus="chooseKind('connectors')" @click="chooseKind('connectors')"><Plug :size="15" /><span>连接器</span><ChevronRight :size="14" /></button>
        <div v-if="kind" class="wb-menu xb-menu wb-submenu" role="group" :aria-label="labels[kind]" @mouseenter="cancelClose">
          <label class="wb-search"><Search :size="14" /><input v-model="term" data-testid="chat-capability-search" :placeholder="`搜索${labels[kind]}…`" maxlength="80" @input="scheduleSearch" /></label>
          <div class="wb-results">
            <div class="wb-section">已安装的{{ labels[kind] }}</div>
            <button v-for="entry in installedEntries" :key="'i-'+entry.id" type="button" class="wb-result" :data-testid="`chat-installed-${entry.id}`" @click="useEntry(entry)">
              <span class="wb-result-text"><strong>{{ entry.title }}</strong><small>{{ entry.detail || entry.id }}</small></span>
              <Check v-if="entry.ready" :size="14" /><ChevronRight v-else :size="14" />
            </button>
            <div v-if="!installedEntries.length" class="wb-empty">{{ term ? '没有匹配的已安装能力' : '暂无已安装能力' }}</div>
            <template v-if="term">
              <div class="wb-section">发现更多</div>
              <div v-if="loading" class="wb-empty">搜索中…</div>
              <button v-for="entry in remoteEntries" :key="'r-'+entry.id" type="button" class="wb-result" @click="openStore">
                <span class="wb-result-text"><strong>{{ entry.title }}</strong><small>{{ entry.detail || '前往 AI 插件查看' }}</small></span><ArrowUpRight :size="14" />
              </button>
              <div v-if="error" class="wb-empty">{{ error }}</div>
              <div v-else-if="!loading && !remoteEntries.length && term" class="wb-empty">可在 AI 插件中继续查找</div>
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
.wb-main{left:0;bottom:calc(100% + 8px);width:170px;min-width:0}
.wb-menu-item{display:flex;gap:10px;align-items:center;width:100%;height:34px;padding:0 10px;background:none;border:0;border-radius:8px;color:var(--ui-text-primary);font:inherit;font-size:13px;text-align:left;cursor:pointer;white-space:nowrap}
.wb-menu-item>svg:first-child{color:var(--ui-text-secondary)}
.wb-menu-item:disabled{opacity:.45;cursor:not-allowed}.wb-menu-item span:first-of-type{flex:1}.wb-menu-item:hover,.wb-menu-item.selected{background:var(--ui-surface-hover)}
.wb-menu-item.selected{background:var(--ui-soft);color:var(--ui-ink)}
.wb-menu-item.selected>svg{color:var(--ui-ink)}
.wb-menu-item svg:last-child:not(:first-child){margin-left:auto;color:var(--ui-text-tertiary)}
.wb-separator{height:1px;background:var(--ui-line);margin:5px 6px}
.wb-submenu{left:calc(100% + 7px);bottom:0;width:min(280px,calc(100vw - 210px));min-width:205px;max-height:360px;display:flex;flex-direction:column}
.wb-search{display:flex;gap:7px;align-items:center;margin:3px 3px 5px;border-radius:8px;background:var(--ui-control);padding:7px 9px;color:var(--ui-text-tertiary)}
.wb-search input{background:none;border:0;outline:none;min-width:0;flex:1;color:var(--ui-text-primary);font:inherit;font-size:12.5px}
.wb-results{flex:1;min-height:0;overflow-y:auto;scrollbar-width:thin}
.wb-section{font-size:11.5px;font-weight:500;color:var(--ui-text-tertiary);padding:8px 8px 4px}
.wb-result{display:flex;align-items:center;gap:8px;width:100%;padding:8px;border-radius:8px;border:0;background:none;color:var(--ui-text-primary);font:inherit;cursor:pointer;text-align:left}
.wb-result:hover{background:var(--ui-surface-hover)}
.wb-result-text{flex:1;min-width:0;display:grid;gap:3px}
.wb-result-text strong{font-size:12.5px;font-weight:500;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.wb-result-text small{font-size:11px;color:var(--ui-text-tertiary);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.wb-result>svg{flex:none;color:var(--ui-text-tertiary)}
.wb-empty{padding:11px 8px;font-size:11.5px;color:var(--ui-text-tertiary)}
.wb-store-link{display:flex;align-items:center;justify-content:center;gap:5px;padding:9px 5px 6px;border:0;border-top:1px solid var(--ui-line);background:none;color:var(--ui-ink);font:inherit;font-size:12.5px;cursor:pointer}
@media(max-width:650px){.wb-submenu{left:0;bottom:calc(100% + 8px);width:min(280px,calc(100vw - 45px))}}
</style>
