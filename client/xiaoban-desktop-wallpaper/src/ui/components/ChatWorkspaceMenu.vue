<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { NButton, NModal } from 'naive-ui';
import { Check, ChevronDown, Folder, FolderInput, FolderOpen, FolderPlus, GitBranch, GitMerge, House, Trash2, TriangleAlert } from 'lucide-vue-next';
import type { ChatStore } from '../chat/chat-store.ts';
import type { WorktreeInfo } from '../chat/types.ts';

/** The folder the open conversation works in, under the message box next to the permission mode: recent folders,
 *  another one from the system picker, and showing it in Finder or Explorer. */
const props = withDefaults(defineProps<{ chat: ChatStore; disabled?: boolean;
  /** `pill`: folder and name under the message box. `icon`: a single folder button, for the conversation's title bar. */
  variant?: 'pill' | 'icon' }>(), { variant: 'pill' });

const root = ref<HTMLElement | null>(null);
const open = ref(false);
/** A folder chosen for a conversation that already has messages: the dialog asks how to switch. */
const asking = ref<string | null>(null);

const nameOf = (path: string) => path.trim().replace(/\\/g, '/').split('/').filter(Boolean).at(-1) || path;
const current = computed(() => props.chat.currentWorkspace?.trim() || '');
const same = (a: string, b: string) => a.replace(/[\\/]+$/, '') === b.replace(/[\\/]+$/, '');
const copy = computed(() => props.chat.currentWorktree);
const currentName = computed(() => copy.value ? `${nameOf(copy.value.origin)} · 独立副本`
  : !current.value || same(current.value, props.chat.workspace) ? '默认工作区' : nameOf(current.value));
/** How the open conversation stands with its isolated copy; read each time the menu opens. */
const worktree = ref<WorktreeInfo | null>(null);
const confirmingDiscard = ref(false);
const busy = ref(false);
const recent = computed(() => props.chat.recentWorkspaces.filter(w => w.exists));
const currentListed = computed(() => !!current.value && (recent.value.some(w => same(w.path, current.value)) || same(props.chat.workspace, current.value)));
const defaultListed = computed(() => recent.value.some(w => same(w.path, props.chat.workspace)));

function close() { open.value = false; }
function onOutside(e: PointerEvent) { if (root.value && !root.value.contains(e.target as Node)) close(); }
function onKey(e: KeyboardEvent) { if (e.key === 'Escape') close(); }
watch(open, value => {
  if (value) void props.chat.worktreeInfo().then(info => { worktree.value = info; });
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
async function startCopy() {
  if (busy.value) return;
  busy.value = true;
  close();
  try { await props.chat.startWorktree(); } finally { busy.value = false; }
}
async function finishCopy(action: 'merge' | 'keep' | 'discard') {
  if (busy.value) return;
  busy.value = true;
  close();
  confirmingDiscard.value = false;
  try { await props.chat.finishWorktree(action); } finally { busy.value = false; }
}
</script>

<template>
  <div ref="root" class="ws-anchor">
    <button v-if="variant === 'icon'" type="button" class="xb-icon-btn ws-icon" data-testid="chat-workspace-menu" aria-haspopup="menu" :aria-expanded="open" aria-label="移动到工作区" :title="'移动到工作区（当前：' + (current || '默认工作目录') + '）'" :disabled="disabled" @click="open = !open">
      <FolderInput :size="16" />
    </button>
    <button v-else type="button" class="ws-trigger" data-testid="chat-workspace-menu" aria-haspopup="menu" :aria-expanded="open" :title="'工作区：' + (current || '默认工作目录')" :disabled="disabled" @click="open = !open">
      <Folder :size="14" /><span class="ws-name">{{ currentName }}</span><ChevronDown :size="11" :stroke-width="2.4" />
    </button>
    <div v-show="open" class="ws-menu xb-menu" :class="`is-${variant}`" role="menu" aria-label="工作区">
      <template v-if="!currentListed">
        <p class="ws-path" :title="current">{{ current || '默认工作目录' }}</p>
        <div class="ws-separator" />
      </template>
      <div v-if="recent.length" class="ws-section">最近使用</div>
      <button v-for="item in recent" :key="item.path" type="button" class="ws-item" role="menuitem" :title="item.path" :data-testid="`chat-workspace-recent-${item.name}`" @click="choose(item.path)">
        <TriangleAlert v-if="item.wide" :size="14" class="ws-wide" /><Folder v-else :size="14" />
        <span class="ws-label">{{ item.name }}</span>
        <Check v-if="same(item.path, current)" :size="14" />
      </button>
      <button v-if="chat.workspace && !defaultListed" type="button" class="ws-item" role="menuitem" data-testid="chat-workspace-default" :title="chat.workspace" @click="choose(chat.workspace)">
        <House :size="14" /><span class="ws-label">默认工作区</span>
        <Check v-if="same(chat.workspace, current)" :size="14" />
      </button>
      <div class="ws-separator" />
      <button type="button" class="ws-item" role="menuitem" data-testid="chat-workspace-pick" @click="pick"><FolderPlus :size="14" /><span>选择文件夹…</span></button>
      <button type="button" class="ws-item" role="menuitem" data-testid="chat-workspace-reveal" @click="reveal"><FolderOpen :size="14" /><span>在文件管理器中打开</span></button>
      <template v-if="worktree">
        <div class="ws-separator" />
        <template v-if="worktree.active">
          <div class="ws-section">独立副本</div>
          <p class="ws-copy" data-testid="chat-worktree-state">分支 {{ worktree.link.branch }} · {{ worktree.status.changedCount }} 个文件未提交 · {{ worktree.status.ahead }} 个提交</p>
          <p v-if="!worktree.status.originClean" class="ws-copy is-warning" data-testid="chat-worktree-origin-dirty">原文件夹里有没提交的改动，合并前要先提交或暂存。</p>
          <button type="button" class="ws-item" role="menuitem" data-testid="chat-worktree-merge" :title="`把副本的改动并入“${worktree.status.originBranch || '当前分支'}”并清理副本`" :disabled="disabled || busy" @click="finishCopy('merge')"><GitMerge :size="14" /><span class="ws-label">合并回原文件夹</span></button>
          <button type="button" class="ws-item" role="menuitem" data-testid="chat-worktree-keep" title="不合并，副本留在磁盘上" :disabled="disabled || busy" @click="finishCopy('keep')"><Folder :size="14" /><span class="ws-label">保留副本，回到原文件夹</span></button>
          <button type="button" class="ws-item" role="menuitem" data-testid="chat-worktree-discard" title="删除副本和里面的改动" :disabled="disabled || busy" @click="confirmingDiscard = true; close()"><Trash2 :size="14" /><span class="ws-label">丢弃副本</span></button>
        </template>
        <button v-else-if="worktree.available" type="button" class="ws-item" role="menuitem" data-testid="chat-worktree-start" title="改动先留在 git 副本里，合并后才影响原文件夹" :disabled="disabled || busy" @click="startCopy"><GitBranch :size="14" /><span class="ws-label">在独立副本中工作</span></button>
        <p v-else-if="worktree.reason" class="ws-copy" data-testid="chat-worktree-unavailable">独立副本不可用：{{ worktree.reason }}</p>
      </template>
    </div>
    <NModal :show="confirmingDiscard" preset="card" title="丢弃独立副本" :style="{ width: '400px', maxWidth: 'calc(100vw - 32px)' }" data-testid="chat-worktree-discard-ask" @update:show="value => { if (!value) confirmingDiscard = false; }">
      <p class="ws-ask">副本里的所有改动（包括还没提交的）都会被删除，原文件夹不受影响。这个操作不能撤销。</p>
      <div class="ws-ask-actions">
        <NButton data-testid="chat-worktree-discard-cancel" @click="confirmingDiscard = false">取消</NButton>
        <NButton type="error" data-testid="chat-worktree-discard-confirm" @click="finishCopy('discard')">丢弃</NButton>
      </div>
    </NModal>
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
.ws-anchor{position:relative;display:flex;align-items:center;min-width:0;max-width:100%}
.ws-trigger{display:flex;align-items:center;gap:5px;height:28px;padding:0 9px;border:0;border-radius:8px;background:none;color:var(--ui-text-secondary);font:inherit;font-size:12.5px;cursor:pointer;white-space:nowrap;min-width:0;max-width:160px}
.ws-trigger:hover,.ws-trigger[aria-expanded="true"]{background:var(--ui-control);color:var(--ui-text-primary)}
.ws-trigger:disabled,.ws-icon:disabled{opacity:.5;cursor:not-allowed}
.ws-trigger svg{flex:none}
.ws-icon[aria-expanded="true"]{background:var(--ui-control);color:var(--ui-text-primary)}
.ws-name{display:block;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.ws-menu{position:absolute;left:0;bottom:calc(100% + 8px);width:232px;min-width:0;max-width:calc(100vw - 32px);box-sizing:border-box;padding:4px}
.ws-menu.is-icon{left:auto;right:0;bottom:auto;top:calc(100% + 6px)}
.ws-path{font-size:11.5px;color:var(--ui-text-tertiary);margin:6px 10px 4px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;direction:rtl;text-align:left}
.ws-section{font-size:12px;font-weight:500;color:var(--ui-text-tertiary);padding:6px 10px 3px}
.ws-separator{height:1px;background:var(--ui-line);margin:5px 6px}
.ws-item{display:flex;gap:10px;align-items:center;width:100%;height:32px;padding:0 10px;background:none;border:0;border-radius:8px;color:var(--ui-text-primary);font:inherit;font-size:13px;text-align:left;cursor:pointer}
.ws-item:hover{background:var(--ui-surface-hover)}
.ws-item>svg{flex:none;color:var(--ui-text-secondary)}
.ws-item>svg:last-child:not(:first-child){margin-left:auto;color:var(--ui-ink)}
.ws-label{flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.ws-item>svg.ws-wide{color:var(--ui-warning)}
.ws-copy{margin:2px 10px 6px;font-size:11.5px;line-height:1.6;color:var(--ui-text-tertiary)}
.ws-copy.is-warning{color:var(--ui-warning)}
.ws-item:disabled{opacity:.5;cursor:not-allowed}
.ws-ask{font-size:13px;line-height:1.7;margin:0 0 16px;color:var(--ui-text-secondary)}
.ws-ask-actions{display:flex;justify-content:flex-end;gap:8px}
</style>
