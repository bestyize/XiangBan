<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue';
import { NModal } from 'naive-ui';
import { CornerDownLeft, Folder, MessageCircle, Search, X } from 'lucide-vue-next';
import { invoke } from '../../bridge.ts';
import type { SessionSummary } from '../chat/types.ts';

const props = defineProps<{ show: boolean; desktop: boolean; spaces: { key: string; name: string; sessions: SessionSummary[] }[] }>();
const emit = defineEmits<{ 'update:show': [show: boolean]; select: [id: string] }>();
interface Hit { sessionId: string; title: string; updatedAt: number; excerpt: string; matchedField: 'title' | 'content' | 'workspace' }
interface Response { hits: Hit[]; skipped: number; cancelled: boolean }
const query = ref(''), field = ref('all'), scope = ref('all');
const hits = ref<Hit[]>([]), loading = ref(false), error = ref(''), skipped = ref(0), active = ref(0);
const input = ref<HTMLInputElement | null>(null), list = ref<HTMLElement | null>(null);
let timer: ReturnType<typeof setTimeout> | undefined, generation = 0;
const composing = ref(false);
const labels = computed(() => Object.fromEntries(props.spaces.flatMap(space => space.sessions.map(session => [session.id, space.name]))));
const names = { title: '标题', content: '内容', workspace: '工作区' };
const filters = [{ id: 'all', name: '全部' }, { id: 'title', name: '标题' }, { id: 'content', name: '内容' }];
function parts(text: string) {
  const terms = [...new Set(query.value.trim().split(/\s+/).filter(Boolean))];
  if (!terms.length) return [{ text, match: false }];
  const pattern = terms.map(term => term.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')).join('|');
  const regex = new RegExp(`(${pattern})`, 'gi');
  return text.split(regex).filter(Boolean).map(text => ({ text, match: terms.some(term => term.toLowerCase() === text.toLowerCase()) }));
}
function timeLabel(seconds: number) {
  const minutes = Math.max(0, Math.floor((Date.now() / 1000 - seconds) / 60));
  if (minutes < 1) return '刚刚';
  if (minutes < 60) return `${minutes} 分钟前`;
  if (minutes < 1440) return `${Math.floor(minutes / 60)} 小时前`;
  return new Date(seconds * 1000).toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' });
}
function schedule() {
  const token = ++generation;
  clearTimeout(timer);
  hits.value = []; error.value = ''; skipped.value = 0; active.value = 0; loading.value = false;
  if (!props.show || composing.value) return;
  if (!props.desktop) { error.value = '请在桌面应用中搜索已保存的会话'; return; }
  loading.value = true;
  timer = setTimeout(async () => {
    const space = props.spaces.find(space => space.key === scope.value);
    try {
      const response = await invoke<Response>('chat_search_sessions', { request: {
        query: query.value.trim(), field: field.value, labels: labels.value,
        sessionIds: scope.value === 'all' ? null : (space?.sessions.map(session => session.id) ?? []),
      } });
      if (token !== generation || !props.show || response.cancelled) return;
      hits.value = response.hits; skipped.value = response.skipped;
      if (list.value) list.value.scrollTop = 0;
    } catch (e) { if (token === generation) error.value = String(e); }
    finally { if (token === generation) loading.value = false; }
  }, query.value.trim() ? 240 : 0);
}
watch([() => props.show, query, field, scope, () => props.spaces], schedule, { immediate: true });
onBeforeUnmount(() => { ++generation; clearTimeout(timer); });
function pick(hit: Hit) { emit('select', hit.sessionId); emit('update:show', false); }
async function move(event: KeyboardEvent) {
  if (composing.value || event.isComposing) return;
  if (event.key === 'Enter' && hits.value[active.value]) { event.preventDefault(); pick(hits.value[active.value]); }
  else if ((event.key === 'ArrowDown' || event.key === 'ArrowUp') && hits.value.length) {
    event.preventDefault(); active.value = (active.value + (event.key === 'ArrowDown' ? 1 : -1) + hits.value.length) % hits.value.length;
    await nextTick(); list.value?.querySelector<HTMLElement>(`[data-result-index="${active.value}"]`)?.scrollIntoView?.({ block: 'nearest' });
  }
}
function reset() { query.value = ''; scope.value = 'all'; field.value = 'all'; input.value?.focus(); }
</script>

<template>
  <NModal :show="show" :auto-focus="false" @update:show="emit('update:show', $event)" @after-enter="input?.focus()">
    <!-- vueuc's focus trap locates a native DIV as its content root. -->
    <div class="search-dialog" role="dialog" aria-modal="true" aria-labelledby="workspace-search-title" data-testid="workspace-search-dialog">
      <header class="dialog-heading"><h1 id="workspace-search-title">搜索工作空间</h1><button type="button" class="icon-button" aria-label="关闭搜索" data-testid="workspace-search-close" @click="emit('update:show', false)"><X :size="15" /></button></header>
      <div class="search-field"><Search :size="17" /><input ref="input" v-model="query" role="combobox" aria-label="搜索会话标题、内容或工作区" aria-autocomplete="list" aria-controls="workspace-search-results" :aria-expanded="show" :aria-activedescendant="hits[active] ? `workspace-result-${active}` : undefined" autocomplete="off" maxlength="200" placeholder="搜索会话标题、内容或工作区…" data-testid="workspace-search-input" @keydown="move" @compositionstart="composing = true; schedule()" @compositionend="composing = false; schedule()"><button v-if="query" type="button" class="icon-button" aria-label="清空关键词" @click="query = ''; input?.focus()"><X :size="14" /></button></div>
      <div class="search-filters"><nav class="filter-tabs" aria-label="检索范围"><button v-for="item in filters" :key="item.id" type="button" :aria-pressed="field === item.id" :data-testid="`workspace-search-${item.id}`" @click="field = item.id">{{ item.name }}</button></nav><label class="scope"><Folder :size="13" /><select v-model="scope" aria-label="筛选工作区" data-testid="workspace-search-scope"><option value="all">所有工作区</option><option v-for="space in spaces" :key="space.key" :value="space.key">{{ space.name }}</option></select></label></div>
      <div class="result-meta"><span aria-live="polite">{{ loading ? '正在检索…' : query.trim() ? `找到 ${hits.length} 个会话` : '最近更新的会话' }}</span><span>{{ query.trim() ? '按相关度排序' : '检索本机已保存的会话' }}</span></div>
      <div id="workspace-search-results" ref="list" class="results" role="listbox" aria-label="检索到的会话" :aria-busy="loading">
        <div v-if="loading" class="empty" role="status"><Search :size="26" /><p>正在检索会话…</p></div>
        <div v-else-if="error" class="empty" role="alert"><h2>暂时无法检索</h2><p>{{ error }}</p><button type="button" @click="schedule">重试</button></div>
        <template v-else-if="hits.length"><button v-for="(hit, index) in hits" :id="`workspace-result-${index}`" :key="hit.sessionId" type="button" role="option" :aria-selected="active === index" tabindex="-1" class="result" :data-result-index="index" data-testid="workspace-search-result" @mousedown.prevent @mouseenter="active = index" @click="pick(hit)">
          <span class="result-icon"><MessageCircle :size="14" /></span><span class="result-main"><span class="result-title-row"><span class="result-title"><template v-for="(part, i) in parts(hit.title || '新对话')" :key="i"><mark v-if="part.match">{{ part.text }}</mark><template v-else>{{ part.text }}</template></template></span><span v-if="query.trim()" class="hit-type">{{ names[hit.matchedField] }}</span><span class="result-time">{{ timeLabel(hit.updatedAt) }}</span></span><span class="result-context"><span class="result-project"><template v-for="(part, i) in parts(labels[hit.sessionId] || '最近任务')" :key="i"><mark v-if="part.match">{{ part.text }}</mark><template v-else>{{ part.text }}</template></template></span><template v-if="hit.excerpt"><i class="result-separator" /><span class="result-excerpt"><template v-for="(part, i) in parts(hit.excerpt)" :key="i"><mark v-if="part.match">{{ part.text }}</mark><template v-else>{{ part.text }}</template></template></span></template></span></span><CornerDownLeft :size="14" class="result-enter" />
        </button></template>
        <div v-else class="empty"><Search :size="29" /><h2>{{ query.trim() ? '没有找到相关会话' : '还没有保存的会话' }}</h2><p>试试更短的关键词，或切换到所有工作区。</p><button type="button" @click="reset">清空搜索条件</button></div>
      </div>
      <footer class="dialog-footer"><template v-if="skipped"><span role="status">{{ skipped }} 个会话暂时无法读取</span><button type="button" class="retry" @click="schedule">重试</button></template><template v-else><span class="keyboard-hints"><span><kbd>↑</kbd><kbd>↓</kbd>选择</span><span><kbd>↵</kbd>打开</span><span class="esc-hint"><kbd>Esc</kbd>关闭</span></span><span class="footer-note">自动展开工作区，定位到会话</span></template></footer>
    </div>
  </NModal>
</template>

<style scoped>
:global(.n-modal-container:has(.search-dialog) .n-modal-mask){backdrop-filter:blur(3px)}
.search-dialog{display:flex;flex-direction:column;width:min(690px,calc(100vw - 36px));max-height:min(620px,calc(100dvh - 64px));border-radius:17px;background:var(--ui-surface,#fff);box-shadow:var(--ui-shadow-dialog,0 24px 80px #0003);overflow:hidden;outline:none;animation:enter .18s ease-out}.dialog-heading{display:flex;align-items:center;justify-content:space-between;padding:22px 24px 15px}.dialog-heading h1{margin:0;font-size:17px;line-height:1.5;letter-spacing:-.2px;font-weight:600}.dialog-heading .icon-button{color:var(--ui-text-tertiary,#a1a1a9);margin-right:-5px}.search-field{display:flex;align-items:center;gap:10px;margin:0 24px;padding:0 12px;border:1px solid var(--ui-border,#e2e2e7);border-radius:9px;background:var(--ui-surface-sunken,#f8f8fa);flex:none}.search-field:focus-within{border-color:var(--ui-btn,#bf5362);box-shadow:0 0 0 3px var(--ui-soft,#fbf0f2);background:var(--ui-surface,#fff)}.search-field>svg{color:var(--ui-text-secondary,#76767e);width:17px;height:17px}.search-field input{min-width:0;flex:1;height:43px;padding:0;background:none;border:0;outline:none;color:var(--ui-text-primary,#27272b);font-size:13px}.search-field input::placeholder{color:var(--ui-text-tertiary,#a1a1a9);font-size:12px}.search-field .icon-button{width:22px;height:22px;flex:none;color:var(--ui-text-tertiary,#a1a1a9)}.search-field .icon-button svg{width:14px;height:14px}.search-filters{display:flex;align-items:center;gap:14px;padding:16px 24px 12px;flex:none}.filter-tabs{display:flex;gap:4px}.filter-tabs button{padding:4px 11px;border:0;border-radius:6px;background:none;color:var(--ui-text-secondary,#76767e);font-size:11px;white-space:nowrap}.filter-tabs button[aria-pressed=true]{background:var(--ui-soft,#fbf0f2);color:var(--ui-btn,#bf5362);font-weight:550}.scope{margin-left:auto;display:flex;align-items:center;gap:6px;color:var(--ui-text-tertiary,#a1a1a9)}.scope svg{width:13px;height:13px}.scope select{max-width:180px;padding:4px 16px 4px 1px;border:0;background:var(--ui-surface,#fff);color:var(--ui-text-secondary,#76767e);font-size:11px;cursor:pointer}.result-meta{display:flex;align-items:center;justify-content:space-between;gap:10px;padding:0 25px 9px;font-size:10px;color:var(--ui-text-tertiary,#a1a1a9);flex:none}.result-meta b{color:var(--ui-text-secondary,#76767e);font-weight:500}.results{height:336px;max-height:336px;min-height:80px;overflow-y:auto;overscroll-behavior:contain;scrollbar-width:thin;scrollbar-color:var(--ui-border,#e2e2e7) transparent;border-top:1px solid var(--ui-line,#ececef);padding:5px 8px}.result{display:flex;align-items:center;gap:12px;width:100%;min-height:65px;padding:10px 15px;border:0;border-radius:9px;text-align:left;background:none;color:inherit}.result[aria-selected=true]{background:var(--ui-surface-sunken,#f8f8fa)}.result[aria-selected=true] .result-icon{color:var(--ui-btn,#bf5362);border-color:var(--ui-border,#e2e2e7);background:var(--ui-surface,#fff)}.result-icon{display:grid;place-items:center;width:29px;height:29px;flex:none;border:1px solid var(--ui-line,#ececef);border-radius:8px;color:var(--ui-text-tertiary,#a1a1a9)}.result-icon svg{width:14px;height:14px}.result-main{flex:1;min-width:0}.result-title-row{display:flex;align-items:center;gap:8px;min-width:0;margin-bottom:5px}.result-title{font-size:12px;font-weight:500;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.hit-type{color:var(--ui-text-tertiary,#a1a1a9);background:var(--ui-surface,#fff);border:1px solid var(--ui-line,#ececef);padding:0 4px;border-radius:4px;font-size:9px;white-space:nowrap;font-weight:400;flex:none}.result-time{margin-left:auto;padding-left:12px;white-space:nowrap;font-size:10px;color:var(--ui-text-tertiary,#a1a1a9);flex:none}.result-context{display:flex;align-items:center;gap:7px;font-size:11px;color:var(--ui-text-tertiary,#a1a1a9);min-width:0}.result-project{color:var(--ui-text-secondary,#76767e);white-space:nowrap;flex:none;max-width:110px;overflow:hidden;text-overflow:ellipsis}.result-separator{width:2px;height:2px;background:var(--ui-text-tertiary,#a1a1a9);border-radius:50%;flex:none}.result-excerpt{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;min-width:0}.result-enter{flex:none;width:14px;height:14px;color:var(--ui-text-tertiary,#a1a1a9);opacity:0}.result[aria-selected=true] .result-enter{opacity:1}.empty{min-height:260px;display:flex;align-items:center;justify-content:center;flex-direction:column;gap:9px;color:var(--ui-text-tertiary,#a1a1a9);text-align:center;padding:26px}.empty>svg{width:29px;height:29px;stroke-width:1.2;margin-bottom:5px}.empty h2{margin:0;font-size:13px;font-weight:500;color:var(--ui-text-secondary,#76767e)}.empty p{margin:0;font-size:11px;line-height:1.8}.empty button{background:none;border:0;color:var(--ui-btn,#bf5362);font-size:11px;margin-top:3px}.dialog-footer{display:flex;align-items:center;justify-content:space-between;gap:15px;padding:13px 24px;border-top:1px solid var(--ui-line,#ececef);flex:none;font-size:10px;color:var(--ui-text-tertiary,#a1a1a9)}.keyboard-hints{display:flex;gap:12px;align-items:center}.keyboard-hints span{display:flex;align-items:center;gap:4px}.footer-note{font-size:10px;white-space:nowrap}
.search-dialog{outline:none;color:var(--ui-text-primary,#27272b)}.search-dialog svg{flex:none}.search-dialog button{font-family:inherit;cursor:pointer}.search-dialog button:focus-visible,.scope select:focus-visible{outline:2px solid var(--ui-btn);outline-offset:3px}.search-dialog mark{padding:0 1px;border-radius:2px;background:color-mix(in srgb,var(--ui-warning,#e09a2b) 24%,transparent);color:inherit}.search-dialog kbd{font-family:inherit;font-size:10px;line-height:1.5;padding:1px 4px;border:1px solid var(--ui-border);border-radius:4px;color:var(--ui-text-secondary)}.icon-button{display:grid;place-items:center;width:25px;height:25px;padding:0;border:0;border-radius:7px;background:none;color:var(--ui-text-tertiary)}.icon-button:hover{background:var(--ui-control)}.result-main{flex:1;min-width:0}.retry{border:0;background:none;font-size:11px;color:var(--ui-btn)}
@media(max-width:480px){.search-dialog{width:calc(100vw - 20px);max-height:calc(100dvh - 32px);border-radius:14px}.dialog-heading{padding:19px 18px 14px}.dialog-heading h1{font-size:16px}.search-field{margin-inline:18px}.search-filters{padding:14px 17px 11px;gap:6px}.filter-tabs button{padding:4px 9px}.scope{gap:3px}.scope>svg{display:none}.scope select{max-width:105px;font-size:10px}.result-meta{padding-inline:19px}.results{padding-inline:5px}.result{padding:10px 12px;gap:9px}.result-icon{width:25px;height:25px;border-radius:7px}.result-title{font-size:11.5px}.result-time{font-size:9px;padding-left:5px}.result-context{gap:5px;font-size:10px}.result-project{max-width:72px}.result-enter{display:none}.dialog-footer{padding:12px 18px;gap:7px}.keyboard-hints{gap:8px}.esc-hint{display:none!important}.footer-note{white-space:normal;max-width:130px;text-align:right;font-size:9px}}

</style>
