<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { NButton, NModal } from 'naive-ui';
import { Check, ChevronDown, Folder, FolderOpen, FolderPlus, House, TriangleAlert } from 'lucide-vue-next';
import type { ChatStore } from '../chat/chat-store.ts';

/** The folder the open conversation works in, under the message box next to the permission mode: recent folders,
 *  another one from the system picker, and showing it in Finder or Explorer. */
const props = defineProps<{ chat: ChatStore; disabled?: boolean }>();

const root = ref<HTMLElement | null>(null);
const open = ref(false);
/** A folder chosen for a conversation that already has messages: the dialog asks how to switch. */
const asking = ref<string | null>(null);

const nameOf = (path: string) => path.replace(/\\/g, '/').split('/').filter(Boolean).at(-1) || path;
const current = computed(() => props.chat.currentWorkspace);
const currentName = computed(() => (current.value ? nameOf(current.value) : '工作区'));
const same = (a: string, b: string) => a.replace(/[\\/]+$/, '') === b.replace(/[\\/]+$/, '');
const recent = computed(() => props.chat.recentWorkspaces.filter(w => w.exists));
const defaultListed = computed(() => recent.value.some(w => same(w.path, props.chat.workspace)));

function close() { open.value = false; }
function onOutside(e: PointerEvent) { if (root.value && !root.value.contains(e.target as Node)) close(); }
function onKey(e: KeyboardEvent) { if (e.key === 'Escape') close(); }
watch(open, value => {
  if (value) { document.addEventListener('pointerdown', onOutside, true); document.addEventListener('keydown', onKey); }
  else { document.removeEventListener('pointerdown', onOutside, true); document.removeEventListener('keydown', onKey); }
});
onBeforeUnmount(() => { document.removeEventListener('pointerdown', onOutside, true); document.removeEventListener('keydown', onKey); });

async function choose(path: string) {
  close();
  if (same(path, current.value)) return;
  if (await props.chat.switchWorkspace(path) === 'ask') asking.value = path;
}
async function pick() {
  close();
  const path = await props.chat.pickWorkspace();
  if (path) await choose(path);
}
async function decide(how: 'new' | 'here') {
  const path = asking.value;
  asking.value = null;
  if (path) await props.chat.switchWorkspace(path, how);
}
function reveal() { close(); void props.chat.revealWorkspace(); }
</script>

<template>
  <div ref="root" class="ws-anchor">
    <button type="button" class="ws-trigger" data-testid="chat-workspace-menu" aria-haspopup="menu" :aria-expanded="open" :title="'工作区：' + (current || '默认工作目录')" :disabled="disabled" @click="open = !open">
      <Folder :size="14" /><span class="ws-name">{{ currentName }}</span><ChevronDown :size="12" />
    </button>
    <div v-show="open" class="ws-menu" role="menu" aria-label="工作区">
      <p class="ws-path" :title="current">{{ current || '默认工作目录' }}</p>
      <div class="ws-separator" />
      <div v-if="recent.length" class="ws-section">最近使用</div>
      <button v-for="item in recent" :key="item.path" type="button" class="ws-item" role="menuitem" :title="item.path" :data-testid="`chat-workspace-recent-${item.name}`" @click="choose(item.path)">
        <TriangleAlert v-if="item.wide" :size="14" class="ws-wide" /><Folder v-else :size="14" />
        <span class="ws-item-text"><strong>{{ item.name }}</strong><small>{{ item.path }}</small></span>
        <Check v-if="same(item.path, current)" :size="14" />
      </button>
      <button v-if="chat.workspace && !defaultListed" type="button" class="ws-item" role="menuitem" data-testid="chat-workspace-default" :title="chat.workspace" @click="choose(chat.workspace)">
        <House :size="14" /><span class="ws-item-text"><strong>默认工作区</strong><small>{{ chat.workspace }}</small></span>
        <Check v-if="same(chat.workspace, current)" :size="14" />
      </button>
      <div class="ws-separator" />
      <button type="button" class="ws-item" role="menuitem" data-testid="chat-workspace-pick" @click="pick"><FolderPlus :size="14" /><span>选择文件夹…</span></button>
      <button type="button" class="ws-item" role="menuitem" data-testid="chat-workspace-reveal" @click="reveal"><FolderOpen :size="14" /><span>在文件管理器中打开</span></button>
    </div>
    <NModal :show="asking !== null" preset="card" title="切换工作区" :style="{ width: '400px', maxWidth: 'calc(100vw - 32px)' }" data-testid="chat-workspace-ask" @update:show="value => { if (!value) asking = null; }">
      <p class="ws-ask">这个对话已经在“{{ currentName }}”里聊过了。要在“{{ asking ? nameOf(asking) : '' }}”里继续，建议新开一个对话，免得前后两个文件夹的内容混在一起。</p>
      <div class="ws-ask-actions">
        <NButton data-testid="chat-workspace-here" @click="decide('here')">在本对话里切换</NButton>
        <NButton type="primary" data-testid="chat-workspace-new" @click="decide('new')">新开对话</NButton>
      </div>
    </NModal>
  </div>
</template>

<style scoped>
.ws-anchor{position:relative;display:flex;align-items:center;min-width:0}
.ws-trigger{display:flex;align-items:center;gap:5px;height:30px;padding:0 6px;border:0;border-radius:8px;background:none;color:var(--ui-text-secondary);font:inherit;font-size:12px;cursor:pointer;white-space:nowrap;min-width:0;max-width:150px}
.ws-trigger:hover,.ws-trigger[aria-expanded="true"]{background:var(--ui-surface-alt);color:var(--ui-text-primary)}
.ws-trigger:disabled{opacity:.5;cursor:not-allowed}
.ws-trigger svg{flex:none}
.ws-name{overflow:hidden;text-overflow:ellipsis}
.ws-menu{position:absolute;z-index:40;left:0;bottom:calc(100% + 8px);width:280px;max-width:calc(100vw - 32px);background:var(--ui-surface);border:1px solid var(--ui-border);border-radius:12px;box-shadow:0 12px 32px rgba(0,0,0,.14);padding:5px;box-sizing:border-box;color:var(--ui-text-primary)}
.ws-path{font-size:11px;color:var(--ui-text-tertiary);margin:5px 8px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;direction:rtl;text-align:left}
.ws-section{font-size:11px;color:var(--ui-text-tertiary);padding:6px 8px 3px}
.ws-separator{height:1px;background:var(--ui-border);margin:5px 4px}
.ws-item{display:flex;gap:9px;align-items:center;width:100%;min-height:33px;padding:5px 9px;background:none;border:0;border-radius:7px;color:var(--ui-text-primary);font:inherit;font-size:12.5px;text-align:left;cursor:pointer}
.ws-item:hover{background:var(--ui-surface-alt)}
.ws-item>svg{flex:none}
.ws-item>svg:last-child:not(:first-child){margin-left:auto;color:var(--ui-primary)}
.ws-item>span:only-of-type{flex:1}
.ws-item-text{flex:1;min-width:0;display:grid;gap:2px}
.ws-item-text strong{font-size:12px;font-weight:600;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.ws-item-text small{font-size:10.5px;color:var(--ui-text-tertiary);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.ws-wide{color:var(--ui-warning, #d08700)}
.ws-ask{font-size:13px;line-height:1.7;margin:0 0 16px;color:var(--ui-text-secondary)}
.ws-ask-actions{display:flex;justify-content:flex-end;gap:8px}
</style>
