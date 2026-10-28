<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { NButton, NInput, NModal } from 'naive-ui';
import { ArrowDown, ArrowUpRight, Check, ChevronDown, ChevronRight, ChevronUp, Copy, Ellipsis, FileText, FolderOpen, GripVertical, LoaderCircle, PanelRight, Plus, Pencil, RefreshCw, Search, Settings, ShieldAlert, ShieldCheck, Sparkles, Sprout, Star, Target, Trash2, Volume2, X } from 'lucide-vue-next';
import { invoke, listen } from '../../bridge.ts';
import DesignPicker from '../design/DesignPicker.vue';
import DesignWorkbench from '../design/DesignWorkbench.vue';
import type { Summary as DesignSummary } from '../design/model.ts';
import type { ControlStore } from '../control-types.ts';
import ModelSwitchDialog from '../components/ModelSwitchDialog.vue';
import ChatComposer from '../components/ChatComposer.vue';
import ChatWorkspaceMenu from '../components/ChatWorkspaceMenu.vue';
import ChatExpertChip from '../components/ChatExpertChip.vue';
import ChatNameDialog from '../components/ChatNameDialog.vue';
import ChatActivity from '../chat/ChatActivity.vue';
import ChatTaskCard from '../chat/ChatTaskCard.vue';
import ChatReasoning from '../chat/ChatReasoning.vue';
import ChatReasoningGroup from '../chat/ChatReasoningGroup.vue';
import { groupReasoning } from '../chat/reasoning-group.ts';
import ChatGoalPanel from '../chat/ChatGoalPanel.vue';
import ChatQuestion from '../chat/ChatQuestion.vue';
import ChatAnswered from '../chat/ChatAnswered.vue';
import { askRecords } from '../chat/ask-record.ts';
import GuideModal from '../components/GuideModal.vue';
import MarkdownView from '../chat/MarkdownView.ts';
import SelectionCopyMenu from '../chat/SelectionCopyMenu.vue';
import { attachmentImage } from '../chat/attachments.ts';
import { CONTINUE_TEXT } from '../chat/chat-store.ts';
import { approvalPreview, clip, formatTokens, isImeEnter, isNearBottom, prettyArguments, riskVerb, toolLabel } from '../chat/format.ts';
import { describeCall } from '../chat/view.ts';
import { silenceNotice } from '../chat/model-errors.ts';
import { replyBlocks, type ReplyBlock, type ToolCall } from '../chat/replies.ts';
import { taskProgress, type TaskProgress } from '../chat/task-progress.ts';
import type { ApprovalDecision, ApprovalRequest, ChatItem, QuestionRequest } from '../chat/types.ts';

const props = defineProps<{ store: ControlStore }>();
const chat = props.store.chat;
const designs = ref<DesignSummary[]>([]), designId = ref(''), designContextId=ref(''), designClosed = ref(false), designWidth = ref(64);
const designPicker=ref(false);
let designOpenEpoch=0,detachBusy=false;
async function openDesign(id:string){
  const epoch=++designOpenEpoch,session=chat.currentId;
  try{
    if(session)await invoke('design_link',{id,session});if(epoch!==designOpenEpoch||chat.currentId!==session)return;designContextId.value=id;
    const detached=await invoke<boolean>('design_window_status',{id});if(epoch!==designOpenEpoch||chat.currentId!==session)return;
    if(detached){await invoke('design_window_open',{id});designPicker.value=false;return;}
    designId.value=id;designClosed.value=false;designPicker.value=false;panelOpen.value=false;
  }catch(e){props.store.notify(e,'error');}
}
async function detachDesign(){if(detachBusy)return;detachBusy=true;const id=designId.value;try{await invoke('design_window_open',{id});if(designId.value===id)closeDesign();}catch(e){props.store.notify(e,'error');}finally{detachBusy=false;}}
let designSplit: { x: number; width: number; value: number } | null = null;
function startDesignSplit(event: PointerEvent) {
  if (event.button !== 0) return;
  const element = event.currentTarget as HTMLElement, parent = element.parentElement!;
  designSplit = { x: event.clientX, width: parent.clientWidth, value: designWidth.value };
  element.setPointerCapture(event.pointerId); event.preventDefault();
}
function moveDesignSplit(event: PointerEvent) { if (designSplit) designWidth.value = Math.max(35, Math.min(85, designSplit.value + (designSplit.x - event.clientX) / designSplit.width * 100)); }
function finishDesignSplit() { designSplit = null; }
function designSplitKey(event: KeyboardEvent) { if (!['ArrowLeft', 'ArrowRight'].includes(event.key)) return; event.preventDefault(); designWidth.value = Math.max(35, Math.min(85, designWidth.value + (event.key === 'ArrowLeft' ? 2 : -2))); }
let designDeleteOff:(()=>void)|undefined;
let designOff: (() => void) | undefined, designDisposed = false;
async function loadDesigns() {
  const id = chat.currentId;
  if (!id || !props.store.desktop) { designs.value = []; return; }
  try {
    const rows = await invoke<DesignSummary[]>('design_list', { session: id });
    if (chat.currentId !== id || designDisposed) return;
    const merged = new Map(designs.value.map(row => [row.id, row]));
    for (const row of Array.isArray(rows) ? rows : []) if (!merged.has(row.id) || merged.get(row.id)!.revision <= row.revision) merged.set(row.id, row);
    designs.value = [...merged.values()].sort((a, b) => b.updatedMs - a.updatedMs);
    if(!designContextId.value)designContextId.value=designs.value.find(p=>p.active)?.id??designs.value[0]?.id??'';
    if (!designClosed.value && !designId.value) {const target=designs.value.find(p=>p.id===designContextId.value);if(target&&!target.detached)designId.value=target.id;}
  } catch (e) { props.store.notify(e, 'error'); }
}
watch(() => chat.currentId, () => { designOpenEpoch++;designId.value = '';designContextId.value=''; designClosed.value = false; designs.value = []; void loadDesigns(); }, { immediate: true });
watch(()=>props.store.designOpening,id=>{if(id){designId.value=id;designContextId.value=id;designClosed.value=false;props.store.designOpening='';}},{immediate:true});

onMounted(async () => {
  if (!props.store.desktop) return;
  const off = await listen<DesignSummary>('design-changed', row => {
    if (row.sessionId !== chat.currentId && !row.sessionIds?.includes(chat.currentId ?? '')) return;
    designs.value = [row, ...designs.value.filter(p => p.id !== row.id)];
    if (!designClosed.value&&!row.detached) { designId.value = row.id;designContextId.value=row.id; panelOpen.value = false; }
  });
  if (designDisposed) off(); else designOff = off;
  const removed=await listen<{id:string}>('design-deleted',row=>{designs.value=designs.value.filter(p=>p.id!==row.id);if(designId.value===row.id)closeDesign();if(designContextId.value===row.id)designContextId.value='';});
  if(designDisposed)removed();else designDeleteOff=removed;
});
onBeforeUnmount(() => { designDisposed = true; designOff?.();designDeleteOff?.(); });
function closeDesign() { designOpenEpoch++;designId.value = ''; designClosed.value = true; }
async function designRequest(message: string) {
  if (running.value) { composer.value?.setDraft(message); props.store.notify('修改要求已放入输入框，当前任务结束后可发送', 'info'); return; }
  await chat.send(message);
}
const composer = ref<InstanceType<typeof ChatComposer> | null>(null);

const modelDialog = ref(false);
const guideDialog = ref(false);
const goalOpen = ref(false);
// A goal template from the community opens the goal form, filled in; the user still saves it themselves.
watch(() => props.store.goalTemplate, template => { if (template) goalOpen.value = true; }, { immediate: true });
const goalBusy = ref(false);
const retryAfterSwitch = ref(false);
function chooseModel(retry = false) { retryAfterSwitch.value = retry; modelDialog.value = true; }
async function modelSwitched(sessionId: string | null) {
  if (chat.currentId !== sessionId) return;
  await chat.refresh();
  await composer.value?.refreshModel();
  if (retryAfterSwitch.value && chat.currentId === sessionId) await chat.retry();
  props.store.notify('模型已切换', 'success');
}
/** 设置 → AI 模型, in the settings dialog. */
function openAiSettings() { modelDialog.value = false; closeMore(); props.store.openSettings('ai'); }

const scroller = ref<HTMLElement | null>(null);
const stick = ref(true);
const preview = ref<string | null>(null);
const examples = ['帮我看看工作目录里有什么，整理一份清单', '把壁纸换成一张安静的风景', '每天下午三点提醒我喝水', '查一下今天是几号，再告诉我这周还剩几天'];
const exampleTags = ['工作区', '壁纸', '关怀', '日常'];

const view = computed(() => chat.current());
const blocks = computed(() => replyBlocks(view.value?.items ?? []));
const running = computed(() => !!view.value?.running);
const session = computed(() => (chat.currentId ? chat.sessions.find(s => s.id === chat.currentId) ?? null : null));
const title = computed(() => session.value?.title || '新对话');
const folderName = (path: string) => path.replace(/\\/g, '/').split('/').filter(Boolean).at(-1) || path;
/** The 空间 the conversation sits in, as the sidebar names it: its folder, or 最近任务 when it has none. */
const spaceName = computed(() => {
  const path = session.value ? session.value.workspace?.trim() : chat.pendingWorkspace;
  return path ? folderName(path) : '最近任务';
});
const waiting = computed(() => !!view.value && (view.value.approvals.length > 0 || view.value.questions.length > 0));
const thinking = computed(() => {
  const v = view.value;
  if (!v?.running || waiting.value) return false;
  const last = v.items[v.items.length - 1];
  return !(last?.kind === 'assistant' && last.streaming);
});
/** Ticks once a second while an answer runs, so a silent model shows how long it has been silent. */
const clock = ref(Date.now());
let clockTimer: ReturnType<typeof setInterval> | undefined;
/** The answer was sent before any event came back (the question is shown at once), so silence counts from here too. */
let runStartedAt = 0;
watch(running, on => {
  clearInterval(clockTimer);
  clockTimer = undefined;
  if (on) { runStartedAt = Date.now(); clock.value = Date.now(); clockTimer = setInterval(() => { clock.value = Date.now(); }, 1000); }
}, { immediate: true });
onBeforeUnmount(() => clearInterval(clockTimer));
/** The model is the one being waited on: not an approval, a question, or a tool that is still running. */
const awaitingModel = computed(() => thinking.value && !view.value?.items.some(i => i.kind === 'tool' && i.status === 'running'));
const silentSeconds = computed(() => {
  const at = Math.max(view.value?.activityAt ?? 0, runStartedAt);
  return awaitingModel.value && at ? Math.max(0, Math.floor((clock.value - at) / 1000)) : 0;
});
const statusLine = computed(() => {
  const base = view.value?.status || (waiting.value ? '等你确认后继续' : thinking.value ? '思考中…' : '');
  if (!awaitingModel.value || (view.value?.status && !view.value.retry)) return base;
  return silenceNotice(silentSeconds.value, view.value?.retry ? base : '') || base;
});
const lastBlockId = computed(() => blocks.value.at(-1)?.id);
const progressByBlock = computed(() => new Map(blocks.value.map(block => [block.id, taskProgress(block,
  running.value && block.id === lastBlockId.value,
  block.id === lastBlockId.value ? view.value?.questions.length ? 'question' : view.value?.approvals.length ? 'approval' : null : null,
  block.id === lastBlockId.value ? statusLine.value : '',
)])));
const progressOf = (block: ReplyBlock) => progressByBlock.value.get(block.id) ?? null;
const planEditorOpen = ref(false);
const planInstruction = ref('');
const planRevisionBusy = ref(false);
const planSession = ref<string | null>(null);
const planSteps = ref<string[]>([]);
const planBase = ref<string[]>([]);
const planDrag = ref(-1);
const planChips = ['更简短', '面向管理层', '跳过最后一步', '多检查一遍'];
const pendingSteps = (progress: TaskProgress | null) => (progress?.plan.steps ?? []).filter(step => step.status !== 'completed').map(step => step.content);
function adjustPlan(block: ReplyBlock) {
  planSession.value = chat.currentId;
  planBase.value = pendingSteps(progressOf(block));
  planSteps.value = [...planBase.value];
  planInstruction.value = '';
  planEditorOpen.value = true;
}
const planEdited = computed(() => planSteps.value.map(text => text.trim()).filter(Boolean));
const planStepsChanged = computed(() => planEdited.value.length > 0 && JSON.stringify(planEdited.value) !== JSON.stringify(planBase.value));
const planReady = computed(() => !!planInstruction.value.trim() || planStepsChanged.value);
const planChip = (text: string) => { planInstruction.value = planInstruction.value.trim() ? `${planInstruction.value.trim()}，${text}` : text; };
function planDrop(target: number) {
  const from = planDrag.value;
  planDrag.value = -1;
  if (from < 0 || from === target) return;
  const next = [...planSteps.value];
  next.splice(target, 0, next.splice(from, 1)[0]!);
  planSteps.value = next;
}
async function savePlanRevision() {
  if (!planReady.value || planRevisionBusy.value) return;
  if (chat.currentId !== planSession.value) { props.store.notify('会话已切换，请回到原会话调整计划。', 'info'); return; }
  const parts: string[] = [];
  if (planStepsChanged.value) parts.push(`后续步骤改为：\n${planEdited.value.map((text, index) => `${index + 1}. ${text}`).join('\n')}`);
  if (planInstruction.value.trim()) parts.push(planInstruction.value.trim());
  planRevisionBusy.value = true;
  props.store.asr?.deactivate();
  try {
    if (await chat.revisePlan(parts.join('\n\n'))) { planEditorOpen.value = false; stick.value = true; }
  } finally { planRevisionBusy.value = false; }
}
function showTaskProcess(block: ReplyBlock) {
  if (!panelOpen.value) togglePanel();
  processBlockId.value = block.id;
}
async function showTaskAttention() {
  await nextTick();
  const target = scroller.value?.querySelector<HTMLElement>('[data-testid="chat-question"], [data-testid="chat-approval"]');
  target?.scrollIntoView?.({ behavior: 'smooth', block: 'center' });
  target?.querySelector<HTMLElement>('button, input')?.focus();
}
function resumeTask() {
  props.store.asr?.deactivate();
  stick.value = true;
  void chat.send('请先核对已完成的进展和当前状态，再从未完成的步骤继续原任务。结果不明的操作先检查，不要直接重放；原有授权范围不变。');
}

onMounted(async () => {
  const first = !chat.ready;
  await chat.init();
  if (!first) await chat.refresh();
});

/* ── Scrolling ───────────────────────────────────────────────────────── */
function toBottom() { const el = scroller.value; if (el) el.scrollTop = el.scrollHeight; }
function onScroll() { if (scroller.value) stick.value = isNearBottom(scroller.value); }
const signature = computed(() => {
  const v = view.value;
  if (!v) return '';
  const last = v.items[v.items.length - 1];
  return [v.id, v.items.length, last && 'text' in last ? last.text.length : 0, last && 'output' in last ? last.output.length : 0, v.approvals.length, v.questions.length, v.running].join('|');
});
watch(signature, async (value, before) => {
  if (value.split('|')[0] !== (before ?? '').split('|')[0]) stick.value = true;
  await nextTick();
  if (stick.value) toBottom();
});
const useExample = (text: string) => composer.value?.setDraft(text);
const continueAnswer = () => { props.store.asr?.deactivate(); stick.value = true; void chat.send(CONTINUE_TEXT); };
const confirmingUndo = ref('');
async function undoRun(itemId: string) { confirmingUndo.value = ''; if (view.value) await chat.undoRun(view.value.id, itemId); }

/* ── Replies ─────────────────────────────────────────────────────────── */
type Assistant = Extract<ChatItem, { kind: 'assistant' }>;
const isUserBlock = (block: ReplyBlock) => block.items[0]?.kind === 'user';
const answers = (block: ReplyBlock) => block.items.filter((item): item is Assistant => item.kind === 'assistant');
/** The 小伴 heading: a reply that says or does something, not one made only of notices. */
const speaks = (block: ReplyBlock) => block.calls.length > 0 || block.items.some(item => item.kind === 'assistant' || item.kind === 'reasoning');
const replyText = (block: ReplyBlock) => answers(block).map(item => item.text.trim()).filter(Boolean).join('\n\n');
const finished = (block: ReplyBlock) => !answers(block).some(item => item.streaming) && !(running.value && block.id === lastBlockId.value);
const replyUsage = (block: ReplyBlock) => [...answers(block)].reverse().find(item => item.usage)?.usage;
const speechKey = (block: ReplyBlock) => answers(block).at(-1)?.id ?? block.id;
interface FileCard { path: string; name: string; ext: string; changed: boolean }
function filesOf(calls: ToolCall[]): FileCard[] {
  const cards = new Map<string, FileCard>();
  for (const call of calls) {
    if (call.status !== 'done' || (call.name !== 'write_file' && call.name !== 'edit_file')) continue;
    let path = '';
    try { const args = JSON.parse(call.arguments) as { path?: unknown }; path = typeof args.path === 'string' ? args.path : ''; } catch { /* half-written arguments */ }
    if (!path) continue;
    const name = folderName(path);
    const ext = (name.includes('.') ? name.split('.').pop()! : 'txt').slice(0, 4).toUpperCase();
    cards.set(path, { path, name, ext, changed: call.name === 'edit_file' || !!cards.get(path)?.changed });
  }
  return [...cards.values()];
}
async function copyReply(block: ReplyBlock) {
  try { await navigator.clipboard.writeText(replyText(block)); props.store.notify('已复制', 'success'); }
  catch { props.store.notify('复制失败，请选中文字后复制', 'error'); }
}
/** Answers saved to 资料库 → 收藏 in this view, so the star shows they are kept. */
const savedReplies = ref(new Set<string>());
async function saveReply(block: ReplyBlock) {
  if (savedReplies.value.has(block.id)) return;
  try {
    await invoke('library_save', { item: { kind: 'answer', title: '', text: replyText(block), sessionId: chat.currentId, sessionTitle: title.value } });
    savedReplies.value = new Set([...savedReplies.value, block.id]);
    props.store.notify('已存到资料库', 'success');
  } catch (error) { props.store.notify(error, 'error'); }
}
function speakReply(block: ReplyBlock) {
  const speech = props.store.speech;
  if (!speech) return;
  const key = speechKey(block);
  if (speech.currentKey === key) speech.stop(); else speech.speak(replyText(block), key);
}
/** The question that led to this reply, sent again. */
function questionBefore(block: ReplyBlock) {
  const index = blocks.value.findIndex(item => item.id === block.id);
  const before = index > 0 ? blocks.value[index - 1].items[0] : undefined;
  return before?.kind === 'user' ? before.text.trim() : '';
}
function regenerate(block: ReplyBlock) {
  const text = questionBefore(block);
  if (!text || running.value) return;
  stick.value = true;
  void chat.send(text);
}

/* ── Title bar: more menu, rename, delete ────────────────────────────── */
const moreRoot = ref<HTMLElement | null>(null);
const moreOpen = ref(false);
function closeMore() { moreOpen.value = false; }
function onOutside(event: PointerEvent) { if (moreRoot.value && !moreRoot.value.contains(event.target as Node)) closeMore(); }
watch(moreOpen, open => { if (open) document.addEventListener('pointerdown', onOutside); else document.removeEventListener('pointerdown', onOutside); });
onBeforeUnmount(() => document.removeEventListener('pointerdown', onOutside));
function toggleGoal() { closeMore(); goalOpen.value = true; }
function openGuide() { closeMore(); guideDialog.value = true; }

const renaming = ref(false);
const renameBusy = ref(false);
function startRename() { closeMore(); if (session.value) renaming.value = true; }
async function submitRename(value: string) {
  const s = session.value;
  if (!s || renameBusy.value) return;
  renameBusy.value = true;
  const done = value.trim() === s.title ? true : await chat.rename(s.id, value);
  renameBusy.value = false;
  if (done) renaming.value = false;
}
const confirmingDelete = ref('');
function askDelete() { closeMore(); if (session.value && !running.value) confirmingDelete.value = session.value.id; }
watch(() => chat.currentId, () => { modelDialog.value = false; retryAfterSwitch.value = false; confirmingDelete.value = ''; renaming.value = false; clearSearch(); });
async function remove(id: string) {
  confirmingDelete.value = '';
  await chat.remove(id);
  if (!chat.currentId) props.store.navigate('new-task');
}
async function goalSessionCreated(id: string) { await chat.refresh(); await chat.select(id); }

/* ── Search in the conversation ──────────────────────────────────────── */
const searchOpen = ref(false);
const searchText = ref('');
const searchInput = ref<HTMLInputElement | null>(null);
const hits = ref<HTMLElement[]>([]);
const hitIndex = ref(-1);
function markHits(active: HTMLElement | null) { scroller.value?.querySelectorAll('.is-hit').forEach(el => el.classList.remove('is-hit')); active?.classList.add('is-hit'); }
function runSearch(step: number) {
  const query = searchText.value.trim().toLowerCase();
  hits.value = query && scroller.value ? [...scroller.value.querySelectorAll<HTMLElement>('[data-search]')].filter(el => (el.textContent ?? '').toLowerCase().includes(query)) : [];
  if (!hits.value.length) { hitIndex.value = -1; markHits(null); return; }
  const count = hits.value.length;
  hitIndex.value = ((hitIndex.value < 0 ? (step > 0 ? -1 : 0) : hitIndex.value) + step + count) % count;
  const el = hits.value[hitIndex.value];
  markHits(el);
  stick.value = false;
  el.scrollIntoView?.({ block: 'center', behavior: 'smooth' });
}
function onSearchInput() { hitIndex.value = -1; runSearch(1); }
function onSearchKey(event: KeyboardEvent) {
  if (event.key === 'Escape') { event.preventDefault(); clearSearch(); return; }
  if (event.key !== 'Enter' || isImeEnter(event, false)) return;
  event.preventDefault();
  runSearch(event.shiftKey ? -1 : 1);
}
function toggleSearch() {
  if (searchOpen.value) { clearSearch(); return; }
  searchOpen.value = true;
  void nextTick(() => searchInput.value?.focus());
}
function clearSearch() { searchOpen.value = false; searchText.value = ''; hits.value = []; hitIndex.value = -1; markHits(null); }

/* ── Execution panel ─────────────────────────────────────────────────── */
const PANEL_KEY = 'xiaoban.chat.exec-panel';
function savedPanel() { try { return localStorage.getItem(PANEL_KEY) === '1'; } catch { return false; } }
const panelOpen = ref(savedPanel());
const processBlockId = ref<string | null>(null);
function togglePanel() { panelOpen.value = !panelOpen.value; processBlockId.value = null; try { localStorage.setItem(PANEL_KEY, panelOpen.value ? '1' : '0'); } catch { /* remembered for this visit only */ } }
const allCalls = computed(() => (view.value?.items ?? []).filter((item): item is ToolCall => item.kind === 'tool'));
const panelCalls = computed(() => processBlockId.value ? blocks.value.find(block => block.id === processBlockId.value)?.calls ?? [] : allCalls.value);
const panelFiles = computed(() => filesOf(panelCalls.value));
watch(() => chat.currentId, () => { processBlockId.value = null; planEditorOpen.value = false; });

/* ── Asking the user ─────────────────────────────────────────────────── */
const decide = (request: ApprovalRequest, decision: ApprovalDecision) => chat.answerApproval(request, decision);
function answer(request: QuestionRequest, value: string | null) { return chat.answerQuestion(request, value); }

async function openLink(href: string) {
  const url = href.split('#')[0];
  try {
    if (!props.store.desktop) { window.open(url, '_blank', 'noopener,noreferrer'); return; }
    await invoke('open_download_url', { url });
  } catch {
    try { await navigator.clipboard.writeText(href); props.store.notify('这个地址不能直接打开，已复制到剪贴板', 'info'); } catch { props.store.notify('这个地址不能直接打开', 'error'); }
  }
}
</script>

<template>
  <ModelSwitchDialog :chat="chat" v-model:show="modelDialog" :retry="retryAfterSwitch" @switched="modelSwitched" @settings="openAiSettings" />
  <GuideModal v-model:show="guideDialog" :desktop="store.desktop" @select-prompt="useExample" @open-settings="openAiSettings" />
  <NModal v-model:show="goalOpen" :mask-closable="!goalBusy" :close-on-esc="!goalBusy">
    <div class="goal-modal-content">
      <ChatGoalPanel v-if="goalOpen" :session-id="chat.currentId" :running="running"
        :configured="chat.configured" :desktop="store.desktop" :permission-mode="chat.permissionMode"
        :session-title="title" :template="store.goalTemplate" @template-used="store.goalTemplate = null" @busy-change="goalBusy = $event"
        @close="goalOpen = false" @session-created="goalSessionCreated" @changed="chat.refresh()" />
    </div>
  </NModal>
  <ChatNameDialog :show="renaming" title="重命名对话" placeholder="给这个对话起个名字" :initial="session?.title ?? ''" :max-length="60" confirm-text="保存" :busy="renameBusy" @close="renaming = false" @submit="submitRename" />
  <NModal v-model:show="planEditorOpen" preset="card" title="调整后续计划" class="task-plan-editor" :style="{ width: '400px', maxWidth: 'calc(100vw - 32px)' }" :header-style="{ padding: '12px 14px 8px', fontSize: '13px' }" :content-style="{ padding: '0 14px' }" :footer-style="{ padding: '10px 14px 12px' }" :mask-closable="!planRevisionBusy" :close-on-esc="!planRevisionBusy" :closable="!planRevisionBusy">
    <div class="plan-steps" data-testid="task-plan-steps">
      <div v-for="(_, index) in planSteps" :key="index" class="plan-row" :class="{ dragging: planDrag === index }" draggable="true" @dragstart="planDrag = index" @dragend="planDrag = -1" @dragover.prevent @drop.prevent="planDrop(index)">
        <GripVertical :size="13" class="plan-grip" aria-hidden="true" />
        <NInput v-model:value="planSteps[index]" size="small" :maxlength="300" :disabled="planRevisionBusy" :aria-label="`后续第 ${index + 1} 步`" />
        <button type="button" class="plan-remove" :disabled="planRevisionBusy" :aria-label="`删除第 ${index + 1} 步`" @click="planSteps.splice(index, 1)"><X :size="13" /></button>
      </div>
      <button type="button" class="plan-add" :disabled="planRevisionBusy || planSteps.length >= 50" @click="planSteps.push('')"><Plus :size="13" />添加一步</button>
    </div>
    <NInput v-model:value="planInstruction" type="textarea" :maxlength="2000" :disabled="planRevisionBusy" placeholder="想怎么改？例如：汇报面向管理层，5 分钟内" :autosize="{ minRows: 2, maxRows: 5 }" data-testid="task-plan-input" />
    <div class="plan-chips"><button v-for="chip in planChips" :key="chip" type="button" :disabled="planRevisionBusy" @click="planChip(chip)">{{ chip }}</button></div>
    <template #footer><div class="plan-editor-actions"><NButton size="small" quaternary :disabled="planRevisionBusy" @click="planEditorOpen = false">取消</NButton><NButton type="primary" size="small" :disabled="!planReady" :loading="planRevisionBusy" data-testid="task-plan-submit" @click="savePlanRevision">调整并继续</NButton></div></template>
  </NModal>
  <section class="chat-page" data-testid="chat-page">
    <div class="chat-main">
      <header class="chat-head" data-tauri-drag-region>
        <nav class="head-crumbs" aria-label="位置">
          <button type="button" class="crumb-space" data-testid="chat-workspace" :disabled="!store.desktop" :title="'工作区：' + (chat.currentWorkspace || '默认工作目录') + (store.desktop ? '。点击在文件管理器中打开' : '')" @click="chat.revealWorkspace()">{{ spaceName }}</button>
          <ChevronRight :size="12" :stroke-width="2.2" class="crumb-sep" aria-hidden="true" />
          <h1 data-testid="chat-title">{{ title }}</h1>
          <ChatExpertChip v-if="chat.currentExpert && session?.expert" class="head-expert" :expert="chat.currentExpert" :desktop="store.desktop" />
        </nav>
        <div class="head-actions">
          <div v-if="store.desktop" class="design-anchor">
            <button class="xb-icon-btn" title="设计稿" aria-label="打开设计稿列表" :aria-expanded="designPicker" @click="designPicker=!designPicker"><FileText :size="16"/></button>
            <DesignPicker v-if="designPicker" :session="chat.currentId" :notify="store.notify" @close="designPicker=false" @open="openDesign" />
          </div>
          <button v-if="store.speech && store.speech.status !== 'idle'" type="button" class="head-link" data-testid="chat-speech-status" @click="store.speech.stop()"><Volume2 :size="14" />{{ store.speech.status === 'synthesizing' ? '取消合成' : '停止朗读' }}</button>
          <div v-if="searchOpen" class="head-search" role="search">
            <Search :size="14" aria-hidden="true" />
            <input ref="searchInput" v-model="searchText" type="search" placeholder="在会话中搜索" aria-label="在会话中搜索" data-testid="chat-search-input" @input="onSearchInput" @keydown="onSearchKey" />
            <small v-if="searchText.trim()" class="search-count" data-testid="chat-search-count">{{ hits.length ? `${hitIndex + 1}/${hits.length}` : '无结果' }}</small>
            <button type="button" class="xb-icon-btn is-sm" aria-label="上一个" :disabled="!hits.length" @click="runSearch(-1)"><ChevronUp :size="14" /></button>
            <button type="button" class="xb-icon-btn is-sm" aria-label="下一个" :disabled="!hits.length" @click="runSearch(1)"><ChevronDown :size="14" /></button>
          </div>
          <button type="button" class="xb-icon-btn" data-testid="chat-search-toggle" :aria-pressed="searchOpen" :aria-label="searchOpen ? '关闭搜索' : '在会话中搜索'" :title="searchOpen ? '关闭搜索' : '在会话中搜索'" :disabled="!view?.items.length && !searchOpen" @click="toggleSearch"><X v-if="searchOpen" :size="16" /><Search v-else :size="16" /></button>
          <ChatWorkspaceMenu v-if="store.desktop" variant="icon" :chat="chat" :disabled="running" />
          <button type="button" class="xb-icon-btn" data-testid="chat-panel-toggle" :aria-pressed="panelOpen" :aria-label="panelOpen ? '隐藏执行面板' : '显示执行面板'" :title="panelOpen ? '隐藏执行面板' : '显示执行面板'" @click="togglePanel"><PanelRight :size="16" /></button>
          <div ref="moreRoot" class="more-anchor">
            <button type="button" class="xb-icon-btn" data-testid="chat-more" aria-haspopup="menu" :aria-expanded="moreOpen" aria-label="更多" title="更多" @click="moreOpen = !moreOpen"><Ellipsis :size="16" /></button>
            <div v-show="moreOpen" class="xb-menu more-menu" role="menu" aria-label="更多">
              <button type="button" class="xb-menu-item" role="menuitem" data-testid="chat-menu-rename" :disabled="!session" @click="startRename"><Pencil :size="15" />重命名</button>
              <button type="button" class="xb-menu-item" role="menuitem" data-testid="chat-goal-toggle" aria-haspopup="dialog" @click="toggleGoal"><Target :size="15" />长期目标</button>
              <button type="button" class="xb-menu-item" role="menuitem" data-testid="chat-open-guide" title="查看小伴 AI 玩法指南与提示词手册" @click="openGuide"><Sparkles :size="15" />玩法指南</button>
              <button type="button" class="xb-menu-item" role="menuitem" data-testid="chat-ai-config" title="设置 → AI 模型：选择小伴 AI，或配置自己的模型服务" @click="openAiSettings"><Settings :size="15" />AI 模型配置</button>
              <div class="xb-menu-sep" />
              <button type="button" class="xb-menu-item is-danger" role="menuitem" data-testid="chat-menu-delete" :disabled="!session || running" @click="askDelete"><Trash2 :size="15" />删除对话</button>
            </div>
          </div>
        </div>
      </header>
      <div v-if="designs.length" class="design-deliveries" aria-label="设计交付物">
        <button v-for="d in designs" :key="d.id" type="button" :class="{ active: designId === d.id }" @click="openDesign(d.id)"><FileText :size="13" /><b>{{ d.title }}</b><small>{{ d.artboards }} 个画板 · 我的文档</small></button>
      </div>
      <div v-if="confirmingDelete" class="delete-bar" data-testid="chat-delete-confirm" role="alertdialog" aria-label="删除对话">
        <span>删除“{{ title }}”？删除后无法恢复。</span>
        <button type="button" class="xb-btn is-sm" @click="confirmingDelete = ''">取消</button>
        <button type="button" class="xb-btn is-sm is-danger" :data-testid="`chat-delete-yes-${confirmingDelete}`" @click="remove(confirmingDelete)">删除</button>
      </div>

      <div ref="scroller" class="chat-scroll" data-testid="chat-messages" @scroll.passive="onScroll">
        <div class="chat-column" :class="{ 'has-messages': !!view?.items.length }">
          <p v-if="!chat.ready" class="state-line">正在读取对话…</p>
          <div v-else-if="!store.desktop" class="info-card" data-testid="chat-preview-only"><b>请在桌面应用中使用聊天</b><p>浏览器预览里只能看到界面。</p></div>
          <div v-else-if="!chat.configured && !view?.items.length" class="info-card" data-testid="chat-unconfigured">
            <b>先给小伴接上一个模型</b>
            <p>在“设置 → AI 模型”里登录后可以直接用小伴 AI，也可以填写自己的接口地址、模型名和密钥。设置好后回到这里就能聊天。</p>
            <button type="button" class="xb-btn-primary" data-testid="chat-go-settings" @click="openAiSettings">去设置</button>
          </div>
          <div v-else-if="!view?.items.length" class="welcome" data-testid="chat-welcome">
            <span class="brand-mark is-lg" aria-hidden="true"><Sprout :size="22" :stroke-width="2.2" /></span>
            <h2>有什么可以帮你？</h2>
            <p>我能回答问题，也能在你选的工作目录里读写文件、运行命令，还能替你调整壁纸、特效和提醒。改动之前我会先问你。</p>
            <div class="examples">
              <span class="examples-cap">试试这样问</span>
              <button v-for="(text, n) in examples" :key="text" type="button" :data-testid="`chat-example-${n}`" @click="useExample(text)">
                <span class="example-tag">{{ exampleTags[n] ?? '尝试' }}</span><span class="example-text">{{ text }}</span><ArrowUpRight :size="14" />
              </button>
            </div>
            <button type="button" class="xb-link more-guides" data-testid="chat-welcome-more-guides" @click="guideDialog = true"><Sparkles :size="14" />查看更多 AI 玩法与咒语手册</button>
          </div>

          <template v-if="view">
            <template v-for="block in blocks" :key="block.id">
              <div v-if="isUserBlock(block)" class="msg user" data-testid="msg-user">
                <div v-for="item in block.items" :key="item.id" class="bubble" data-search>
                  <template v-if="item.kind === 'user'">
                    <small v-if="item.delivery === 'steer'" class="delivered-follow-up">已补充到当前任务</small>
                    <div v-if="item.images.length" class="bubble-images"><img v-for="(url, n) in item.images" :key="n" :src="url" :alt="`第 ${n + 1} 张图片`" :data-testid="`msg-user-image-${n}`" @click="preview = attachmentImage(url)" /></div>
                    <div v-if="item.files?.length" class="bubble-files"><span v-for="(file, n) in item.files" :key="n" class="bubble-file" :title="file.name"><FileText :size="13" />{{ file.name }} · {{ Math.ceil(file.size / 1024) }} KB</span></div>
                    <template v-if="item.text">{{ item.text }}</template>
                  </template>
                </div>
              </div>
              <section v-else class="reply" data-testid="chat-reply">
                <header v-if="speaks(block)" class="reply-head"><span class="brand-mark" aria-hidden="true"><Sprout :size="14" :stroke-width="2.2" /></span><b>小伴</b></header>
                <ChatTaskCard v-if="progressOf(block)" :progress="progressOf(block)!" :calls="block.calls" :current="block.id === lastBlockId" :show-logs="!chat.hideToolCalls" :disabled="planRevisionBusy" :configured="chat.configured" @logs="showTaskProcess(block)" @adjust="adjustPlan(block)" @resume="resumeTask" @stop="chat.cancel()" @attention="showTaskAttention" />
                <ChatActivity v-else-if="block.calls.length && !chat.hideToolCalls" :calls="block.calls" :active="view.running && block.id === lastBlockId" :waiting="waiting && block.id === lastBlockId" :status="block.id === lastBlockId ? statusLine : ''" />
                <ChatAnswered v-for="record in askRecords(block.calls)" :key="record.id" :record="record" />
                <template v-for="item in groupReasoning(block.items)" :key="item.id">
                  <ChatReasoningGroup v-if="item.kind === 'reasoning-group'" :group="item" />
                  <ChatReasoning v-else-if="item.kind === 'reasoning'" :text="item.text" :reasoning-kind="item.reasoningKind" :streaming="item.streaming" />
                  <div v-else-if="item.kind === 'assistant'" class="msg assistant" data-testid="msg-assistant">
                    <div class="answer" data-search><MarkdownView :source="item.text" @open="openLink" /><span v-if="item.streaming" class="cursor" aria-hidden="true"></span></div>
                  </div>
                  <div v-else-if="item.kind === 'notice' && (item.text || item.details)" class="notice" :class="[item.tone, { 'outcome-details': item.taskStatus && !item.text }]" :data-testid="`notice-${item.tone}`" :role="item.tone === 'error' ? 'alert' : 'status'">
                    {{ item.text }}
                    <details v-if="item.details"><summary>{{ item.detailsLabel ?? '错误详情' }}</summary><span class="notice-details">{{ item.details }}</span></details>
                    <button v-if="item.canSwitchModel && !view.running" type="button" class="xb-link" @click="chooseModel(!!item.canRetry)">切换模型</button>
                    <button v-if="item.canRetry && !view.running" type="button" class="xb-link" @click="chat.retry()">重试</button>
                    <button v-if="item.canContinue && !view.running" type="button" class="xb-link" data-testid="chat-continue" @click="continueAnswer">继续</button>
                    <template v-if="item.undoRun && !view.running">
                      <button v-if="confirmingUndo !== item.id" type="button" class="xb-link" data-testid="chat-undo-run" @click="confirmingUndo = item.id">撤销这些改动</button>
                      <template v-else>
                        <span class="undo-warning">这些文件会回到这轮回答之前的样子，之后对它们的修改也会被覆盖。</span>
                        <button type="button" class="xb-link" data-testid="chat-undo-confirm" @click="undoRun(item.id)">确认撤销</button>
                        <button type="button" class="xb-link" @click="confirmingUndo = ''">取消</button>
                      </template>
                    </template>
                  </div>
                </template>
                <details v-if="finished(block) && filesOf(block.calls).length" class="file-cards" :class="{ 'task-delivery': !!progressOf(block) }" data-testid="chat-files">
                  <summary class="file-summary" data-testid="chat-files-toggle"><ChevronRight :size="14" class="file-chevron" /><FileText :size="14" /><span>{{ progressOf(block)?.state === 'completed' ? '交付文件' : '文件变更' }}</span><span class="file-count">{{ filesOf(block.calls).length }}</span><small>{{ spaceName }}</small></summary>
                  <div class="file-list" role="list" aria-label="本轮变更的文件">
                    <div v-for="file in filesOf(block.calls)" :key="file.path" class="file-card" data-testid="chat-file-card" :title="file.path" role="listitem">
                      <span class="file-ext" aria-hidden="true">{{ file.ext }}</span>
                      <span class="file-name">{{ file.name }}</span>
                      <small class="file-status">{{ file.changed ? '已修改' : '已写入' }}</small>
                      <button v-if="store.desktop" type="button" class="xb-icon-btn is-sm" aria-label="打开文件夹" title="打开文件夹" @click="chat.revealWorkspace()"><FolderOpen :size="14" /></button>
                    </div>
                  </div>
                </details>
                <div v-if="finished(block) && replyText(block)" class="reply-actions">
                  <button type="button" class="act" data-testid="chat-copy-reply" aria-label="复制" title="复制" @click="copyReply(block)"><Copy :size="15" /></button>
                  <button type="button" class="act" :class="{ 'is-on': savedReplies.has(block.id) }" :disabled="!store.desktop" :aria-label="savedReplies.has(block.id) ? '已存到资料库' : '存到资料库'" :title="savedReplies.has(block.id) ? '已存到资料库' : '存到资料库'" data-testid="chat-save-reply" @click="saveReply(block)"><Star :size="15" :fill="savedReplies.has(block.id) ? 'currentColor' : 'none'" /></button>
                  <button v-if="store.speech" type="button" class="act" :class="{ 'is-on': store.speech.currentKey === speechKey(block) }" :disabled="!store.desktop" :aria-label="store.speech.currentKey === speechKey(block) ? '停止朗读' : '朗读'" :title="store.speech.currentKey === speechKey(block) ? (store.speech.status === 'synthesizing' ? '取消合成' : '停止朗读') : '朗读'" data-testid="chat-read-reply" @click="speakReply(block)"><Volume2 :size="15" /></button>
                  <button v-if="block.id === lastBlockId && questionBefore(block)" type="button" class="act" data-testid="chat-regenerate" aria-label="重新生成" title="重新生成：把上一个问题再发一次" :disabled="running || !chat.configured" @click="regenerate(block)"><RefreshCw :size="15" /></button>
                  <small v-if="replyUsage(block)" class="usage" data-testid="msg-usage" title="这一轮所有模型调用的合计，输入含重复发送的上下文与缓存">输入 {{ formatTokens(replyUsage(block)!.prompt) }} · 输出 {{ formatTokens(replyUsage(block)!.completion) }} tokens</small>
                </div>
              </section>
            </template>

            <section v-for="request in view.approvals" :key="request.id" class="card approval" :class="{ exec: request.risk === 'exec', dangerous: !!request.danger }" data-testid="chat-approval" role="alertdialog" :aria-label="`需要你确认：${toolLabel(request.tool)}`">
              <header><ShieldAlert :size="16" /><span>{{ request.externalPath ? '小伴请求访问工作区外路径：' : `小伴想要${riskVerb(request.risk)}：` }}<b>{{ toolLabel(request.tool) }}</b></span></header>
              <p v-if="request.externalPath" class="caution">只授权本次指定文件或目录操作（目录搜索包含子文件），不保存长期权限。切换到完全信任后，普通文件访问不再逐次确认。</p>
              <p v-if="request.danger" class="caution danger-note" data-testid="approval-danger"><b>高风险操作：{{ request.danger }}。</b>即使在“完全信任”下，这类操作也要你确认，每次都问。</p>
              <p v-if="request.tainted" class="caution" data-testid="approval-tainted">这段对话读过网页等外部内容，其中可能夹带诱导，所以这次每一步都要你确认。</p>
              <pre class="preview" data-testid="approval-preview">{{ approvalPreview(request) }}</pre>
              <footer>
                <NButton type="primary" size="small" data-testid="approval-once" @click="decide(request, 'once')">允许一次</NButton>
                <NButton v-if="request.canRemember" size="small" data-testid="approval-always" @click="decide(request, 'always')">{{ request.rememberAs ? `这段对话里 ${request.rememberAs} 都允许` : '这段对话都允许' }}</NButton>
                <NButton size="small" quaternary data-testid="approval-deny" @click="decide(request, 'deny')">拒绝</NButton>
              </footer>
            </section>

            <ChatQuestion v-for="request in view.questions" :key="request.id" :request="request" @answer="answer(request, $event)" />

            <p v-if="statusLine && (chat.hideToolCalls || !blocks.at(-1)?.calls.length)" class="status-line" data-testid="chat-status"><LoaderCircle class="spin" :size="14" />{{ statusLine }}</p>
          </template>
        </div>
      </div>

      <button v-if="!stick && view?.items.length" type="button" class="to-bottom" data-testid="chat-bottom" aria-label="回到最新" @click="stick = true; toBottom()"><ArrowDown :size="16" /></button>

      <div class="composer-wrap">
        <ChatComposer ref="composer" :design-id="designContextId||undefined" :store="store" page="chat" pet="sm" placeholder="发送消息给小伴" @sending="stick = true" @sent="toBottom" @settings="openAiSettings" @preview="preview = $event" />
        <p class="disclaimer">内容由 AI 生成，请核对重要信息</p>
      </div>
    </div>

    <aside v-if="panelOpen" class="exec-panel" data-testid="chat-exec-panel" aria-label="执行面板">
      <header class="panel-head"><b>{{ processBlockId ? '任务执行过程' : '执行面板' }}</b><button type="button" class="xb-icon-btn is-sm" aria-label="隐藏执行面板" @click="togglePanel"><X :size="14" /></button></header>
      <div class="panel-body xb-scroll">
        <section class="panel-section">
          <h3>工作区</h3>
          <p class="panel-path" :title="chat.currentWorkspace">{{ chat.currentWorkspace || '默认工作目录' }}</p>
          <button v-if="store.desktop" type="button" class="xb-btn is-sm" @click="chat.revealWorkspace()">在文件管理器中打开</button>
        </section>
        <section class="panel-section">
          <h3>权限</h3>
          <p class="panel-mode" :class="{ danger: chat.permissionMode === 'auto' }"><ShieldAlert v-if="chat.permissionMode === 'auto'" :size="14" /><ShieldCheck v-else :size="14" />{{ chat.permissionMode === 'auto' ? '完全信任：不再询问直接执行' : '默认权限：执行前先问你' }}</p>
        </section>
        <section class="panel-section">
          <h3>执行步骤<small v-if="panelCalls.length">{{ panelCalls.length }}</small></h3>
          <p v-if="!panelCalls.length" class="panel-empty">小伴还没有执行操作。读写文件、运行命令时会在这里列出来。</p>
          <ol v-else class="panel-steps">
            <li v-for="call in panelCalls" :key="call.id" :class="call.status">
              <LoaderCircle v-if="call.status === 'running'" :size="13" class="spin" /><Check v-else-if="call.status === 'done'" :size="13" /><X v-else :size="13" />
              <details class="panel-call"><summary><b>{{ toolLabel(call.name) }}</b><small :title="describeCall(call.name, call.arguments)">{{ describeCall(call.name, call.arguments) }}</small></summary><pre v-if="call.arguments">{{ clip(prettyArguments(call.arguments)) }}</pre><pre v-if="call.output">{{ clip(call.output) }}</pre></details>
            </li>
          </ol>
        </section>
        <section v-if="panelFiles.length" class="panel-section">
          <h3>改动的文件<small>{{ panelFiles.length }}</small></h3>
          <ul class="panel-files"><li v-for="file in panelFiles" :key="file.path" :title="file.path"><FileText :size="13" /><span>{{ file.name }}</span><small>{{ file.changed ? '已修改' : '已写入' }}</small></li></ul>
        </section>
      </div>
    </aside>

    <div v-if="preview" class="lightbox" data-testid="chat-image-preview" role="dialog" aria-label="图片预览" @click="preview = null">
      <img :src="preview" alt="图片预览" />
      <button type="button" class="lightbox-close" data-testid="chat-image-preview-close" aria-label="关闭预览" @click="preview = null"><X :size="18" /></button>
    </div>
    <div v-if="designId" class="design-splitter" role="separator" tabindex="0" aria-label="调整聊天和画布宽度" aria-orientation="vertical" :aria-valuenow="Math.round(designWidth)" :aria-valuemin="35" :aria-valuemax="85" @pointerdown="startDesignSplit" @pointermove="moveDesignSplit" @pointerup="finishDesignSplit" @pointercancel="finishDesignSplit" @keydown="designSplitKey" />
    <aside v-if="designId" :style="{ width: designWidth + '%' }" class="design-dock" aria-label="设计工作台"><DesignWorkbench :project-id="designId" :notify="store.notify" @close="closeDesign" @detach="detachDesign" @copied="openDesign" @request="designRequest" /></aside>
    <SelectionCopyMenu :scope="scroller" />
  </section>
</template>

<style scoped>
.design-anchor{position:relative}

.delivered-follow-up{display:block;margin-bottom:5px;font-size:10px;opacity:.7}

.design-deliveries{display:flex;gap:6px;overflow:auto;padding:4px 18px 8px;flex:none}.design-deliveries button{display:flex;align-items:center;gap:6px;padding:6px 9px;background:var(--ui-control);border:1px solid var(--ui-line);border-radius:7px;color:var(--ui-text-secondary);font-size:11px;cursor:pointer;white-space:nowrap}.design-deliveries b{font-weight:500}.design-deliveries small{font-size:10px;color:var(--ui-text-tertiary)}.design-deliveries .active{border-color:var(--ui-ink);color:var(--ui-ink)}
.design-splitter{width:4px;flex:none;cursor:col-resize;touch-action:none;background:transparent}.design-splitter:hover,.design-splitter:focus-visible{background:var(--ui-soft);outline:0}
.design-dock{width:64%;min-width:420px;max-width:calc(100% - 250px);border-left:1px solid var(--ui-line);flex:none;min-height:0;display:flex}@media(max-width:900px){.design-splitter{display:none}.design-dock{position:absolute;inset:0;width:100%!important;min-width:0;max-width:none;z-index:8;background:var(--ui-surface)}}
.plan-steps{display:flex;flex-direction:column;gap:3px;margin-bottom:8px}
.plan-row{display:flex;align-items:center;gap:6px}.plan-row.dragging{opacity:.4}.plan-grip{flex:none;color:var(--ui-text-disabled);cursor:grab}
.plan-remove{display:grid;place-items:center;flex:none;width:22px;height:22px;padding:0;border:0;border-radius:6px;background:transparent;color:var(--ui-text-tertiary);cursor:pointer}.plan-remove:hover{background:var(--ui-surface-hover);color:var(--ui-text-primary)}
.plan-add{display:inline-flex;align-items:center;gap:4px;align-self:flex-start;height:22px;margin-left:18px;padding:0 6px;border:0;border-radius:6px;background:transparent;color:var(--ui-ink);font:inherit;font-size:11.5px;cursor:pointer}.plan-add:hover{background:var(--ui-soft)}
.plan-chips{display:flex;flex-wrap:wrap;gap:5px;margin-top:8px}.plan-chips button{height:22px;padding:0 9px;border:0;border-radius:11px;background:var(--ui-control);color:var(--ui-text-secondary);font:inherit;font-size:11.5px;cursor:pointer}.plan-chips button:hover:not(:disabled){color:var(--ui-text-primary)}
.plan-editor-actions{display:flex;justify-content:flex-end;gap:6px}
.task-delivery{max-width:600px}
.panel-call{min-width:0;flex:1}.panel-call summary{list-style:none;cursor:pointer;display:flex;flex-direction:column;gap:1px}.panel-call summary::-webkit-details-marker{display:none}.panel-call pre{max-height:180px;overflow:auto;white-space:pre-wrap;overflow-wrap:anywhere;background:var(--ui-control);border-radius:7px;padding:8px;margin:7px 0 0;font:11px/1.6 var(--ui-font-mono);color:var(--ui-text-secondary)}
.goal-modal-content{width:fit-content;max-width:100%;outline:none}
.chat-page{flex:1;min-height:0;min-width:0;display:flex;background:var(--ui-surface);position:relative}
.chat-main{flex:1;display:flex;flex-direction:column;min-height:0;min-width:0;position:relative}

/* Title bar */
.chat-head{height:52px;flex-shrink:0;display:flex;align-items:center;gap:12px;padding:0 14px 0 24px;box-sizing:border-box}
.head-expert{margin-left:10px;flex:none}
.head-crumbs{display:flex;align-items:center;gap:8px;min-width:0;flex:1}
.crumb-space{flex:none;max-width:180px;border:0;background:none;padding:2px 4px;margin-left:-4px;border-radius:6px;font:inherit;font-size:13px;color:var(--ui-text-secondary);cursor:pointer;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.crumb-space:hover:not(:disabled){background:var(--ui-surface-hover);color:var(--ui-text-primary)}
.crumb-space:disabled{cursor:default}
.crumb-sep{flex:none;color:var(--ui-text-disabled)}
.chat-head h1{margin:0;font-size:14px;font-weight:600;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.head-actions{display:flex;align-items:center;gap:2px;flex-shrink:0}
.head-actions .xb-icon-btn:disabled{opacity:.4;cursor:default}
.head-actions .xb-icon-btn:disabled:hover{background:transparent}
.head-link{display:inline-flex;align-items:center;gap:5px;height:28px;padding:0 9px;margin-right:4px;border:0;border-radius:8px;background:var(--ui-soft);color:var(--ui-ink);font:inherit;font-size:12px;cursor:pointer}
.head-search{display:flex;align-items:center;gap:4px;height:30px;padding:0 4px 0 10px;margin-right:4px;border-radius:8px;background:var(--ui-control);color:var(--ui-text-tertiary)}
.head-search input{width:160px;border:0;outline:none;background:none;color:var(--ui-text-primary);font:inherit;font-size:13px}
.head-search input::-webkit-search-cancel-button{display:none}
.search-count{font-size:11.5px;color:var(--ui-text-tertiary);white-space:nowrap;font-variant-numeric:tabular-nums;margin-right:2px}
.more-anchor{position:relative;display:flex}
.more-menu{position:absolute;right:0;top:calc(100% + 6px);width:200px}
.more-menu .xb-menu-item:disabled{opacity:.45;cursor:not-allowed}
.more-menu .xb-menu-item:disabled:hover{background:transparent}
.delete-bar{display:flex;align-items:center;gap:8px;margin:0 24px 8px;padding:8px 10px 8px 14px;border-radius:10px;background:color-mix(in srgb,var(--ui-danger) 8%,var(--ui-surface));font-size:13px;color:var(--ui-text-primary)}
.delete-bar span{flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}

/* Messages */
.chat-scroll{flex:1;min-height:0;overflow-y:auto;overflow-x:hidden;scrollbar-width:thin;scrollbar-color:var(--ui-scrollbar) transparent;overscroll-behavior:contain}
.chat-column{width:100%;max-width:760px;margin:0 auto;padding:12px 40px 24px;display:flex;flex-direction:column;gap:20px;box-sizing:border-box;min-width:0}
.state-line{text-align:center;padding:40px 0;margin:0;color:var(--ui-text-tertiary);font-size:13px}
.info-card,.welcome{text-align:center;padding:48px 16px;display:grid;gap:10px;justify-items:center}
.info-card{background:var(--ui-surface-sunken);border-radius:16px;margin-top:40px}
.info-card b{font-size:15px;font-weight:600}
.info-card p,.welcome p{color:var(--ui-text-secondary);max-width:520px;line-height:1.8;margin:0;font-size:13.5px}
.welcome{padding-top:56px}
.welcome h2{margin:6px 0 0;font-size:22px;font-weight:600}
.examples{display:flex;flex-direction:column;align-items:stretch;gap:2px;margin-top:22px;width:min(560px,100%);text-align:left}
.examples-cap{font-size:12px;font-weight:500;color:var(--ui-text-secondary);padding:0 12px 6px}
.examples button{display:flex;align-items:center;gap:10px;height:38px;padding:0 12px;border:0;border-radius:10px;background:transparent;color:var(--ui-text-primary);cursor:pointer;font:inherit;font-size:13.5px;text-align:left}
.examples button:hover{background:var(--ui-control)}
.examples button svg{flex:none;color:var(--ui-text-tertiary);margin-left:auto}
.example-tag{flex:none;font-size:11px;padding:1px 6px;border-radius:5px;background:var(--ui-control);color:var(--ui-text-secondary)}
.examples button:hover .example-tag{background:var(--ui-surface)}
.example-text{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.more-guides{display:inline-flex;align-items:center;gap:6px;margin-top:10px;font-size:13px}
.brand-mark{flex:none;width:24px;height:24px;border-radius:7px;background:linear-gradient(160deg,var(--ui-btn-hi),var(--ui-btn));color:var(--ui-text-on-btn);display:grid;place-items:center;box-shadow:inset 0 1px 0 rgba(255,255,255,.25)}
.brand-mark.is-lg{width:44px;height:44px;border-radius:13px;box-shadow:inset 0 1px 0 rgba(255,255,255,.3),0 4px 12px rgba(0,0,0,.12)}

.msg.user{display:flex;flex-direction:column;align-items:flex-end;width:100%;min-width:0}
.bubble{background:var(--ui-control);color:var(--ui-text-primary);border-radius:16px;padding:11px 15px;max-width:80%;white-space:pre-wrap;word-break:break-word;overflow-wrap:anywhere;line-height:1.7;font-size:14px;min-width:0;box-sizing:border-box}
.bubble-images{display:flex;flex-wrap:wrap;gap:6px;margin-bottom:6px}
.bubble-images img{max-width:180px;max-height:180px;border-radius:10px;box-shadow:0 0 0 1px var(--ui-border);cursor:zoom-in;display:block}
.bubble-files{display:flex;flex-wrap:wrap;gap:6px;margin-bottom:6px}
.bubble-file{display:inline-flex;align-items:center;gap:5px;max-width:220px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:12px;padding:6px 9px;border-radius:8px;background:var(--ui-surface);color:var(--ui-text-secondary)}
.reply{display:flex;flex-direction:column;gap:12px;min-width:0}
.reply-head{display:flex;align-items:center;gap:8px}
.reply-head b{font-size:13.5px;font-weight:600}
.msg.assistant{min-width:0}
.answer{min-width:0;font-size:14.5px;line-height:1.8;color:var(--ui-text-primary);overflow-wrap:anywhere;word-break:break-word}
.cursor{display:inline-block;width:7px;height:15px;background:var(--ui-btn);border-radius:2px;margin-left:2px;vertical-align:text-bottom;animation:blink 1s steps(2,start) infinite}
@keyframes blink{to{visibility:hidden}}
.bubble.is-hit,.answer.is-hit{box-shadow:0 0 0 2px var(--ui-btn);border-radius:12px}
.answer.is-hit{background:var(--ui-soft)}
.file-cards{max-width:520px;min-width:0;border-radius:9px;background:var(--ui-surface-sunken);box-shadow:inset 0 0 0 1px var(--ui-line)}
.file-summary{display:flex;align-items:center;gap:7px;min-height:34px;padding:7px 10px;list-style:none;cursor:pointer;font-size:12px;color:var(--ui-text-secondary);user-select:none}
.file-summary::-webkit-details-marker{display:none}.file-summary>svg{flex:none;color:var(--ui-text-tertiary)}.file-summary:focus-visible{outline:2px solid var(--ui-btn);outline-offset:2px;border-radius:9px}
.file-summary small{margin-left:auto;min-width:0;max-width:40%;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:10.5px;color:var(--ui-text-tertiary)}
.file-count{min-width:18px;padding:0 5px;border-radius:5px;background:var(--ui-surface);color:var(--ui-text-secondary);font-size:10px;text-align:center;font-variant-numeric:tabular-nums}.file-chevron{transition:transform .15s}.file-cards[open] .file-chevron{transform:rotate(90deg)}
.file-list{max-height:240px;overflow:auto;overscroll-behavior:contain;scrollbar-width:thin;border-top:1px solid var(--ui-line)}
.file-card{display:flex;align-items:center;gap:9px;min-height:36px;padding:5px 10px;box-sizing:border-box}.file-card+.file-card{border-top:1px solid var(--ui-line)}
.file-ext{flex:none;width:26px;height:22px;border-radius:5px;background:var(--ui-surface);display:grid;place-items:center;color:var(--ui-ink);font-size:8px;font-weight:600}
.file-name{flex:1;min-width:0;font-size:12px;color:var(--ui-text-primary);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.file-status{flex:none;font-size:10.5px;color:var(--ui-text-tertiary)}.file-card .xb-icon-btn{flex:none}
@media(prefers-reduced-motion:reduce){.file-chevron{transition:none}}
.reply-actions{display:flex;align-items:center;gap:2px;margin-top:-4px}
.act{width:28px;height:28px;border:0;border-radius:7px;background:transparent;color:var(--ui-text-tertiary);display:grid;place-items:center;cursor:pointer;padding:0}
.act:hover:not(:disabled){background:var(--ui-surface-hover);color:var(--ui-text-primary)}
.act.is-on{color:var(--ui-ink);background:var(--ui-soft)}
.act:disabled{opacity:.4;cursor:not-allowed}
.usage{margin-left:8px;font-size:11.5px;color:var(--ui-text-tertiary);font-variant-numeric:tabular-nums}
.notice{font-size:13px;padding:8px 12px;border-radius:10px;background:var(--ui-surface-sunken);color:var(--ui-text-secondary);display:flex;flex-wrap:wrap;gap:12px;align-items:center;line-height:1.6}
.notice.outcome-details{padding:0;background:none;font-size:12px;color:var(--ui-text-tertiary)}
.notice.error{background:color-mix(in srgb,var(--ui-danger) 9%,var(--ui-surface));color:var(--ui-danger)}
.notice.warning{background:color-mix(in srgb,var(--ui-warning) 14%,var(--ui-surface));color:var(--ui-warning-text)}
.notice details{flex:1;min-width:0}
.notice summary{cursor:pointer}
.notice-details{display:block;margin-top:6px;white-space:pre-wrap;overflow-wrap:anywhere;max-height:240px;overflow:auto;color:var(--ui-text-secondary)}
.undo-warning{color:var(--ui-text-secondary)}
.card{border-radius:14px;background:var(--ui-surface);padding:14px 16px;display:grid;gap:10px;box-shadow:var(--ui-shadow-card),0 6px 20px rgba(0,0,0,.05);min-width:0;box-sizing:border-box}
.card.exec{box-shadow:0 0 0 1px var(--ui-btn),0 6px 20px rgba(0,0,0,.05)}
.card header{display:flex;gap:8px;align-items:center;font-size:14px}
.card header svg{color:var(--ui-ink);flex:none}
.card footer{display:flex;gap:8px;align-items:center;flex-wrap:wrap}
.caution{margin:0;font-size:12.5px;color:var(--ui-text-secondary);background:color-mix(in srgb,var(--ui-warning) 14%,transparent);border-radius:8px;padding:7px 10px;line-height:1.6}
.danger-note{background:color-mix(in srgb,var(--ui-danger) 11%,transparent);color:var(--ui-text-primary)}
.approval.dangerous{box-shadow:0 0 0 1.5px var(--ui-danger),0 6px 20px rgba(0,0,0,.05)}
.preview{margin:0;padding:9px 11px;background:var(--ui-control);border-radius:8px;overflow:auto;max-height:260px;font-size:12px;line-height:1.6;white-space:pre-wrap;overflow-wrap:anywhere;word-break:break-word;font-family:var(--ui-font-mono)}
.options{display:flex;gap:8px;flex-wrap:wrap}
.status-line{margin:0;font-size:13px;color:var(--ui-text-tertiary);display:flex;gap:8px;align-items:center}
.spin{animation:spin 1s linear infinite}
@keyframes spin{to{transform:rotate(360deg)}}
.to-bottom{position:absolute;right:28px;bottom:150px;width:34px;height:34px;border-radius:50%;border:0;background:var(--ui-surface-raised);color:var(--ui-text-primary);cursor:pointer;display:grid;place-items:center;box-shadow:var(--ui-shadow-menu)}

/* Composer */
.composer-wrap{flex-shrink:0;width:100%;max-width:840px;margin:0 auto;padding:0 40px 10px;box-sizing:border-box}
.disclaimer{margin:8px 0 0;text-align:center;font-size:11px;color:var(--ui-text-tertiary)}

/* Execution panel */
.exec-panel{width:280px;flex-shrink:0;display:flex;flex-direction:column;min-height:0;border-left:1px solid var(--ui-line);background:var(--ui-surface)}
.panel-head{height:52px;flex-shrink:0;display:flex;align-items:center;justify-content:space-between;padding:0 12px 0 18px;box-sizing:border-box}
.panel-head b{font-size:13.5px;font-weight:600}
.panel-body{flex:1;min-height:0;overflow-y:auto;padding:0 18px 18px;display:flex;flex-direction:column;gap:18px}
.panel-section{display:flex;flex-direction:column;gap:8px;align-items:flex-start}
.panel-section h3{margin:0;font-size:12px;font-weight:500;color:var(--ui-text-tertiary);display:flex;align-items:center;gap:6px}
.panel-section h3 small{font-size:11px;padding:0 6px;border-radius:999px;background:var(--ui-control);color:var(--ui-text-secondary)}
.panel-path{margin:0;font-size:12px;font-family:var(--ui-font-mono);color:var(--ui-text-secondary);word-break:break-all;line-height:1.5}
.panel-mode{margin:0;display:flex;align-items:center;gap:6px;font-size:12.5px;color:var(--ui-text-primary)}
.panel-mode svg{color:var(--ui-ink)}
.panel-mode.danger,.panel-mode.danger svg{color:var(--ui-danger)}
.panel-empty{margin:0;font-size:12px;line-height:1.6;color:var(--ui-text-tertiary)}
.panel-steps,.panel-files{list-style:none;margin:0;padding:0;display:flex;flex-direction:column;gap:2px;width:100%}
.panel-steps li{display:flex;gap:8px;align-items:flex-start;padding:5px 0;font-size:12.5px;min-width:0}
.panel-steps li>svg{flex:none;margin-top:3px;color:var(--ui-text-tertiary)}
.panel-steps li.done>svg{color:var(--ui-success)}
.panel-steps li.error>svg{color:var(--ui-danger)}
.panel-steps li>span{min-width:0;display:flex;flex-direction:column;gap:1px}
.panel-steps b{font-weight:500}
.panel-steps small{font-size:11.5px;color:var(--ui-text-tertiary);font-family:var(--ui-font-mono);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.panel-files li{display:flex;align-items:center;gap:7px;font-size:12.5px;padding:4px 0;min-width:0}
.panel-files li>svg{flex:none;color:var(--ui-ink)}
.panel-files li span{flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.panel-files li small{font-size:11px;color:var(--ui-text-tertiary)}

.lightbox{position:fixed;inset:0;z-index:60;background:rgba(0,0,0,.78);display:grid;place-items:center;padding:40px;cursor:zoom-out}
.lightbox img{max-width:100%;max-height:100%;border-radius:12px;box-shadow:0 18px 60px rgba(0,0,0,.5)}
.lightbox-close{position:absolute;top:18px;right:18px;width:36px;height:36px;border-radius:50%;border:0;background:rgba(255,255,255,.16);color:#fff;display:grid;place-items:center;cursor:pointer}
.lightbox-close:hover{background:rgba(255,255,255,.3)}

.chat-page :deep(.markdown){width:100%;max-width:100%;min-width:0;word-break:break-word;overflow-wrap:anywhere;box-sizing:border-box}
.chat-page :deep(.markdown>*:first-child){margin-top:0}
.chat-page :deep(.markdown>*:last-child){margin-bottom:0}
.chat-page :deep(.markdown p),.chat-page :deep(.markdown li){margin:0 0 .75em;word-break:break-word;overflow-wrap:anywhere}
.chat-page :deep(.markdown strong){font-weight:600}
.chat-page :deep(.markdown .md-heading){margin:1.1em 0 .5em;line-height:1.4;font-weight:600;word-break:break-word;overflow-wrap:anywhere}
.chat-page :deep(.markdown h3){font-size:1.2em}.chat-page :deep(.markdown h4){font-size:1.08em}.chat-page :deep(.markdown h5),.chat-page :deep(.markdown h6){font-size:1em}
.chat-page :deep(.markdown ul),.chat-page :deep(.markdown ol){margin:0 0 .75em;padding-left:1.5em;max-width:100%;box-sizing:border-box}
.chat-page :deep(.markdown li>p){margin:.3em 0}
.chat-page :deep(.markdown blockquote){margin:0 0 .75em;padding:2px 14px;border-left:3px solid var(--ui-border-strong);color:var(--ui-text-secondary);word-break:break-word;overflow-wrap:anywhere}
.chat-page :deep(.markdown hr){border:0;border-top:1px solid var(--ui-line);margin:1em 0}
.chat-page :deep(.markdown .md-code){background:var(--ui-soft);color:var(--ui-ink);border-radius:6px;padding:1px 6px;font-size:.88em;font-family:var(--ui-font-mono);word-break:break-word;overflow-wrap:anywhere}
.chat-page :deep(.markdown .md-link){color:var(--ui-ink);cursor:pointer;text-decoration:underline;text-underline-offset:2px;overflow-wrap:anywhere;word-break:break-word}
.chat-page :deep(.markdown .md-table){overflow:auto;margin:0 0 .75em;max-width:100%}
.chat-page :deep(.markdown table){border-collapse:collapse;font-size:13.5px}
.chat-page :deep(.markdown th),.chat-page :deep(.markdown td){border:1px solid var(--ui-line);padding:6px 12px;text-align:left}
.chat-page :deep(.markdown th){background:var(--ui-surface-sunken);font-weight:600}
@media(max-width:1100px){.exec-panel{position:absolute;right:0;top:0;bottom:0;z-index:5;box-shadow:var(--ui-shadow-menu)}}
@media(max-width:760px){
  .chat-column{padding:12px 16px 20px}
  .composer-wrap{padding:0 12px 8px}
  .bubble{max-width:90%}
  .head-search input{width:100px}
}
/* The visible composer pet extends 92px upward; keep the last message above it. */
@media(min-width:621px){.chat-column.has-messages{padding-bottom:112px}}
@media(prefers-reduced-motion:reduce){.spin,.cursor{animation:none}}
</style>
