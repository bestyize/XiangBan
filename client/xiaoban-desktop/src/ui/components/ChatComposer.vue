<script setup lang="ts">
import { sessionModelConfig } from '../chat/session-model.ts';
/**
 * The message box shared by 新建任务 and the conversation page: attachments, a top row, the text, and one row of tools.
 * Top row: the skills chosen for this message on the left (chips, gone once sent); on the right the expert the
 * conversation is bound to and the connectors it should prefer (both stay for the whole conversation).
 * Tool row: ＋ (files, screenshot, experts, skills, connectors), the working folder (new task only), the permission
 * mode, then the context ring, the model, voice input and send / stop on the right.
 * Each page keeps its own instance (both stay alive), so voice input only runs on the page that is shown.
 */
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { NInput } from 'naive-ui';
import { ArrowUp, ChevronDown, ListPlus, AudioLines, LoaderCircle, Mic, MicOff, Square, X, Zap, Monitor, Smartphone, PanelsTopLeft } from 'lucide-vue-next';
import { GATEWAY_PROVIDER, invoke, type AiConfigView, type AiTier } from '../../bridge.ts';
import type { ControlPage, ControlStore } from '../control-types.ts';
import ChatWorkbenchTools from './ChatWorkbenchTools.vue';
import ChatMessageQueue from './ChatMessageQueue.vue';
import ChatSlashMenu from './ChatSlashMenu.vue';
import ChatMentionMenu from './ChatMentionMenu.vue';
import { useComposerMentions, type MentionItem } from '../chat/mentions.ts';
import ChatExpertChip from './ChatExpertChip.vue';
import ChatConnectorCluster from './ChatConnectorCluster.vue';
import ChatComposerPet from './ChatComposerPet.vue';
import ChatComposerModel from './ChatComposerModel.vue';
import ChatWorkspaceMenu from './ChatWorkspaceMenu.vue';
import ChatPermissionMenu from './ChatPermissionMenu.vue';
import ChatContextRing from './ChatContextRing.vue';
import AsrStatus from './AsrStatus.vue';
import { ATTACHMENT_ACCEPT, MAX_ATTACHMENTS, MAX_TOTAL_BYTES, attachmentBytes, attachmentImage, attachmentName, readAttachment, type PendingAttachment } from '../chat/attachments.ts';
import { isImeEnter } from '../chat/format.ts';
import { MAX_SKILLS, dropSlash, filterSkills, slashToken, withSkills, type SlashSkill } from '../chat/slash.ts';
import { builtinToolLabels, withBuiltinTool, type BuiltinTool } from '../chat/builtin-tools.ts';

const props = withDefaults(defineProps<{
  store: ControlStore;
  designId?:string;
  /** The page this box lives on; voice input and incoming drafts only act while it is the page shown. */
  page: Extract<ControlPage, 'chat' | 'new-task'>;
  /** Show the working-folder menu in the tool row (the new-task page). */
  workspace?: boolean;
  placeholder?: string;
  minRows?: number;
  /** The companion peeking over the top edge of the box. */
  pet?: 'lg' | 'sm' | false;
}>(), { workspace: false, placeholder: '', minRows: 1, pet: 'sm' });
const emit = defineEmits<{ sending: []; sent: []; settings: []; preview: [url: string] }>();

const chat = props.store.chat;
const asr = props.store.asr;
const draft = computed({ get: () => chat.draft.text, set: text => { chat.draft.text = text; } });
const composing = ref(false);
const shown = computed(() => props.store.page === props.page);

/* ── Model label ─────────────────────────────────────────────────────── */
const modelLabel = ref('编程模型');
const modelProvider = ref('');
let modelLabelGeneration = 0;
async function readModelLabel() {
  if (!props.store.desktop) return;
  const generation = ++modelLabelGeneration;
  try {
    const config = sessionModelConfig(await invoke<AiConfigView>('get_ai_config'), chat.model)!;
    if (generation !== modelLabelGeneration) return;
    if (config.activeProvider === GATEWAY_PROVIDER) {
      modelLabel.value = '小伴 AI'; modelProvider.value = '小伴 AI';
      try {
        const tiers = await invoke<AiTier[]>('ai_gateway_tiers');
        if (generation !== modelLabelGeneration) return;
        const tier = tiers.find(t => t.tier === (config.gatewayTier || config.model));
        if (tier) modelLabel.value = `小伴 AI · ${tier.displayName}`;
      } catch { /* Keep a readable gateway label when login or the network is unavailable. */ }
      return;
    }
    const provider = config.providers?.find(p => p.id === config.activeProvider);
    modelProvider.value = provider?.name ?? '';
    modelLabel.value = provider?.models?.find(m => m.id === config.model)?.name || config.model || '编程模型';
  } catch { /* Preserve the last known label while preferences are unavailable. */ }
}
async function modelSwitched() { await chat.refresh(); await readModelLabel(); props.store.notify('模型已切换', 'success'); }

/* ── State of the open conversation ──────────────────────────────────── */
const view = computed(() => chat.current());
const running = computed(() => !!view.value?.running);
const compacting = computed(() => !!chat.currentId && chat.compactingId === chat.currentId);
const attachments = computed({ get: () => chat.draft.attachments, set: value => { chat.draft.attachments = value; } });
const attachmentReading = ref(false);
const capturing = ref(false);
let attachmentVersion = 0;
const filePicker = ref<HTMLInputElement | null>(null);
const MAX_IMAGES = MAX_ATTACHMENTS;
const skills = computed({ get: () => chat.draft.skills, set: value => { chat.draft.skills = value; } });
const selectedTool = computed({ get: () => chat.draft.tool ?? null, set: (value: BuiltinTool | null) => { chat.draft.tool = value; } });
const canSend = computed(() => chat.configured && !chat.sending && !compacting.value && !attachmentReading.value && !capturing.value && !mentions.selecting.value && (draft.value.trim() !== '' || attachments.value.length > 0 || skills.value.length > 0));

/* ── Voice ───────────────────────────────────────────────────────────── */
const voiceHasText = ref(false);
const finishingVoice = ref(false);
const foreground = ref(document.visibilityState !== 'hidden');
function updateVisibility() { foreground.value = document.visibilityState !== 'hidden'; }
function blurWindow() { if (document.visibilityState === 'hidden') foreground.value = false; }
const microphoneAllowed = computed(() => shown.value && foreground.value && chat.configured && !running.value && (!props.store.speech || props.store.speech.status === 'idle'));
/** 实时对话 owns the microphone and the voice while it runs, across the new-task and chat pages. */
const live = props.store.live;
const liveActive = computed(() => Boolean(live?.active));
watch(() => chat.currentId, (id, before) => {
  // Another conversation was opened; the first message of a new one only gives it an id.
  if (before && id !== before) live?.end();
  if (shown.value && !liveActive.value) asr?.deactivate();
  attachmentVersion++;
  deliveryMenu.value = false;
  if (shown.value) void readModelLabel();
}, { flush: 'sync' });
watch(microphoneAllowed, allowed => { if (!allowed && shown.value && !liveActive.value) asr?.deactivate(); }, { flush: 'sync' });
watch(() => chat.model, () => { if (shown.value) void readModelLabel(); }, { deep: true });
watch(shown, now => { if (!now && !liveActive.value) asr?.deactivate(); else if (now) void readModelLabel(); }, { flush: 'sync' });
/** 实时对话 sits beside send; it waits while an answer is being written or a dictation is under way. */
const offersLive = computed(() => Boolean(live && asr) && props.store.desktop && asr!.microphoneSupported !== false);
const liveDisabled = computed(() => !chat.configured || running.value || compacting.value || attachmentReading.value || Boolean(asr?.recordingRequested) || finishingVoice.value);
const liveNow = ref(Date.now());
let liveClockTimer: ReturnType<typeof setInterval> | undefined;
watch(liveActive, active => {
  clearInterval(liveClockTimer); liveClockTimer = undefined;
  if (active) { liveNow.value = Date.now(); liveClockTimer = setInterval(() => { liveNow.value = Date.now(); }, 1000); }
}, { immediate: true });
const liveClock = computed(() => {
  const seconds = live?.since ? Math.max(0, Math.floor((liveNow.value - live.since) / 1000)) : 0;
  return `${String(Math.floor(seconds / 60)).padStart(2, '0')}:${String(seconds % 60).padStart(2, '0')}`;
});
const liveHint = computed(() => {
  switch (live?.status) {
    case '已静音': return '小伴听不到你，点麦克风恢复';
    case '在说': return '直接开口就能打断它';
    case '在想': return '马上回答';
    case '等你确认': return '在上面的对话里确认后继续';
    case '正在连接…': return '第一次使用需要允许麦克风';
    default: return live?.heard || live?.partial ? `${[live.heard, live.partial].filter(Boolean).join(' ')}…` : '说完停一下就会回答';
  }
});
async function startLive() { if (offersLive.value && !liveDisabled.value) await live?.start(); }
async function toggleVoice() {
  if (!asr || finishingVoice.value) return;
  if (asr.recordingRequested) {
    const wasListening = asr.listening;
    const sessionId = chat.currentId;
    finishingVoice.value = true;
    try {
      await asr.finish();
      asr.deactivate();
      if (!wasListening || asr.error || !shown.value || !foreground.value || chat.currentId !== sessionId) return;
      // Recording only fills the box: the user reads what was heard and sends it. Sending on its own is the live conversation.
      if (!voiceHasText.value) props.store.notify('没有识别到语音，请重试', 'info');
    } finally { finishingVoice.value = false; }
    return;
  }
  if (!microphoneAllowed.value) return;
  voiceHasText.value = false;
  const owner = chat.draft;
  void asr.activate(text => { voiceHasText.value = true; owner.text += (owner.text && !/\s$/.test(owner.text) ? ' ' : '') + text; });
}
onMounted(() => {
  if (shown.value) asr?.deactivate();
  void asr?.init();
  void readModelLabel();
  document.addEventListener('visibilitychange', updateVisibility);
  window.addEventListener('blur', blurWindow); window.addEventListener('focus', updateVisibility);
});
onUnmounted(() => {
  attachmentVersion++;
  clearInterval(liveClockTimer);
  if (shown.value && !liveActive.value) asr?.deactivate();
  document.removeEventListener('visibilitychange', updateVisibility);
  window.removeEventListener('blur', blurWindow); window.removeEventListener('focus', updateVisibility);
});

/* ── Sending ─────────────────────────────────────────────────────────── */
const deliveryMenu = ref(false);
function deliveryBlur(event: FocusEvent) { if (!(event.currentTarget as HTMLElement).contains(event.relatedTarget as Node | null)) deliveryMenu.value = false; }
async function submit(mode: 'queue' | 'steer' | 'replace' = 'queue') {
  deliveryMenu.value = false;
  const owner = chat.draft;
  const text = draft.value.trim();
  if ((!text && !attachments.value.length && !skills.value.length) || !canSend.value) return;
  asr?.deactivate();
  const images = [...attachments.value];
  const references = [...mentions.selected.value];
  const chosen = [...skills.value];
  const tool = selectedTool.value;
  draft.value = '';
  attachments.value = [];
  skills.value = [];
  selectedTool.value = null;
  emit('sending');
  const message = withSkills(withBuiltinTool(text, tool), chosen.map(s => s.name));
  const sent = chat.currentId
    ? await chat.enqueue(message, images, references.map(({ kind, id }) => ({ kind, id })), mode, props.designId)
    : await chat.send(message, images, undefined, references.map(({ kind, id }) => ({ kind, id })), props.designId);
  if (!sent) {
    // Keep both the refused message and anything typed while the host was responding.
    owner.text = [text, owner.text].filter(Boolean).join('\n\n');
    owner.attachments = [...images, ...owner.attachments];
    owner.skills = [...chosen, ...owner.skills.filter(skill => !chosen.some(old => old.name === skill.name))];
    if (!owner.tool) owner.tool = tool;
  }
  await nextTick();
  if (sent) emit('sent');
}
/* ── `/` menu: skills the user can start by name ───────────────────── */
const slashSkills = ref<SlashSkill[]>([]);
let slashLoaded = false;
const slashIndex = ref(0);
const slashDismissed = ref('');
const token = computed(() => slashToken(draft.value));
const slashItems = computed(() => (token.value && props.store.desktop ? filterSkills(slashSkills.value, token.value.query) : []));
const slashOpen = computed(() => !!token.value && slashItems.value.length > 0 && slashDismissed.value !== draft.value);
async function loadSlashSkills() {
  if (slashLoaded || !props.store.desktop) return;
  slashLoaded = true;
  try { slashSkills.value = await invoke<SlashSkill[]>('chat_invocable_skills'); } catch { slashLoaded = false; }
}
watch(token, (now) => { slashIndex.value = 0; if (now) void loadSlashSkills(); });
watch(slashItems, (items) => { if (slashIndex.value >= items.length) slashIndex.value = 0; });
function pickSkill(name: string) {
  const at = token.value;
  if (!at) return;
  draft.value = dropSlash(draft.value, at);
  addSkill(name, slashSkills.value.find(s => s.name === name)?.displayName);
  slashDismissed.value = '';
  void nextTick(() => input.value?.focus());
}
function onSlashKey(event: KeyboardEvent): boolean {
  if (!slashOpen.value || isImeEnter(event, composing.value)) return false;
  const count = slashItems.value.length;
  if (event.key === 'ArrowDown') slashIndex.value = (slashIndex.value + 1) % count;
  else if (event.key === 'ArrowUp') slashIndex.value = (slashIndex.value + count - 1) % count;
  else if (event.key === 'Enter' || event.key === 'Tab') pickSkill(slashItems.value[slashIndex.value].name);
  else if (event.key === 'Escape') slashDismissed.value = draft.value;
  else return false;
  event.preventDefault();
  return true;
}
function onKeydown(event: KeyboardEvent) {
  if (mentions.keydown(event, composing.value)) return;
  if (onSlashKey(event)) return;
  if (event.key !== 'Enter' || event.shiftKey || isImeEnter(event, composing.value)) return;
  event.preventDefault();
  void submit();
}

/* ── Attachments ─────────────────────────────────────────────────────── */
async function addFiles(files: File[]) {
  if (!files.length || attachmentReading.value) return;
  const version = attachmentVersion;
  attachmentReading.value = true;
  try {
    for (const file of files) {
      if (version !== attachmentVersion) break;
      if (attachments.value.length >= MAX_IMAGES) { props.store.notify('一次最多添加 8 个附件', 'info'); break; }
      try {
        const item = await readAttachment(file);
        if (version !== attachmentVersion) break;
        if (attachments.value.length >= MAX_IMAGES) { props.store.notify('一次最多添加 8 个附件', 'info'); break; }
        if (attachments.value.reduce((sum, a) => sum + attachmentBytes(a), 0) + attachmentBytes(item) > MAX_TOTAL_BYTES) { props.store.notify('附件合计不能超过 30MB', 'error'); continue; }
        attachments.value.push(item);
      } catch (error) { props.store.notify(error instanceof Error ? error.message : String(error), 'error'); }
    }
  } finally { attachmentReading.value = false; }
}
/** Images on the clipboard arrive as files; anything else is left to the normal paste. */
function onPaste(event: ClipboardEvent) {
  const files = Array.from(event.clipboardData?.files ?? []);
  if (!files.length) return;
  event.preventDefault();
  void addFiles(files);
}
function onPick(event: Event) {
  const input = event.target as HTMLInputElement;
  void addFiles(Array.from(input.files ?? []));
  // Let the same file be chosen again after it is removed.
  input.value = '';
}
const removeAttachment = (index: number) => { attachments.value = attachments.value.filter((_, n) => n !== index); };
const openPicker = () => filePicker.value?.click();
async function takeScreenshot() {
  if (!props.store.desktop || capturing.value || !chat.configured || attachments.value.length >= MAX_IMAGES) return;
  const version = attachmentVersion;
  capturing.value = true;
  try {
    const image = await invoke<string | null>('chat_screenshot');
    if (version !== attachmentVersion || !image) return;
    if (!image.startsWith('data:image/')) throw new Error('截图结果格式错误');
    if (attachments.value.length >= MAX_IMAGES) throw new Error('一次最多添加 8 个附件');
    if (attachments.value.reduce((sum, a) => sum + attachmentBytes(a), 0) + attachmentBytes(image) > MAX_TOTAL_BYTES) throw new Error('附件合计不能超过 30MB');
    attachments.value.push(image);
  } catch (error) { if (version === attachmentVersion) props.store.notify(error instanceof Error ? error.message : String(error), 'error'); }
  finally { capturing.value = false; }
}

/* ── Prompts put into the box ────────────────────────────────────────── */
const input = ref<InstanceType<typeof NInput> | null>(null);
function setDraft(text: string) { draft.value = text; void nextTick(() => input.value?.focus()); }
function insertPrompt(instruction: string) { draft.value = draft.value.trim() ? instruction + '\n\n' + draft.value : instruction; }
/** 召唤专家：专家是一场对话的身份，所以总是开一个新对话；已经开始的对话不会中途换人。
 *  工作流型专家通过 workflow 绑定，后端在整个对话中加载已验证的工作流。 */
async function useExpert(entrySkill: string, name: string, expert?: { slug: string; name: string; profession?: string; description?: string; agent?: string | null; quickPrompts?: string[]; tags?: string[]; hasAvatar?: boolean; kind?: 'agent' | 'team'; plugin?: string }) {
  if (running.value) { props.store.notify('这个对话还在回答，请等它结束或先停止，再召唤专家', 'info'); return; }
  if (chat.currentId) await chat.select(null);
  if (chat.currentId) return;
  chat.pendingExpert = expert?.plugin
    ? { plugin: expert.plugin, name: expert.name, profession: expert.profession ?? '', description: expert.description, agent: expert.agent, quickPrompts: expert.quickPrompts, tags: expert.tags, hasAvatar: expert.hasAvatar, kind: expert.kind }
    : { plugin: '', workflow: entrySkill, name, profession: '', description: expert?.description, hasAvatar: false };
  props.store.navigate('new-task');
  await nextTick();
  input.value?.focus();
}
/** 技能变成输入框上方的 chip；发送时写成消息开头的 `/名称`，后端会把这个技能的说明交给小伴。 */
function addSkill(slug: string, title?: string) {
  if (skills.value.some(s => s.name === slug)) return;
  if (skills.value.length >= MAX_SKILLS) { props.store.notify(`一条消息最多用 ${MAX_SKILLS} 个技能`, 'info'); return; }
  skills.value.push({ name: slug, title: title || slug });
}
const useSkill = (slug: string, title?: string) => { addSkill(slug, title); void nextTick(() => input.value?.focus()); };
const removeSkill = (slug: string) => { skills.value = skills.value.filter(s => s.name !== slug); };
const addSummary = computed(() => ({
  skills: skills.value.length > 1 ? `${skills.value.length} 个技能` : skills.value[0]?.title ?? '',
  experts: chat.currentExpert?.name ?? '',
  connectors: mentions.selected.value.length ? `${mentions.selected.value.length} 个已启用` : '',
  tools: selectedTool.value ? builtinToolLabels[selectedTool.value] : '',
}));
const useTool = (tool: BuiltinTool) => { selectedTool.value = tool; void nextTick(() => input.value?.focus()); };
function checkComputerPermissions() { useTool('computer'); insertPrompt('请检查电脑控制所需的系统权限，并告诉我检查结果。'); }
function computerDisabled() { if (selectedTool.value === 'computer') selectedTool.value = null; }
const useConnector = (name: string, id: string) => {
  if (!mentions.selected.value.some(item => item.id === id)) mentions.selected.value.push({ kind: 'connector', id, name, description: '' });
};
/** The switches in the connector list: choose a connector for this conversation, or drop it. */
function toggleConnector(item: { id: string; name: string }) {
  if (mentions.selected.value.some(i => i.id === item.id)) mentions.selected.value = mentions.selected.value.filter(i => i.id !== item.id);
  else useConnector(item.name, item.id);
}
const mentions = useComposerMentions(draft, () => props.store.desktop && shown.value, async (item: MentionItem, isCurrent) => {
  if (item.kind === 'skill') { useSkill(item.id, item.name); return true; }
  if (item.kind === 'expert') {
    if (running.value) { props.store.notify('这个对话还在回答，请等它结束或先停止，再召唤专家', 'info'); return false; }
    type Expert = NonNullable<Parameters<typeof useExpert>[2]> & { entrySkill?: string };
    const experts = await invoke<Expert[]>('ai_store_experts_installed');
    if (!isCurrent()) return false;
    const expert = experts.find(e => e.slug === item.id || e.plugin === item.id);
    if (!expert) throw new Error('这位专家已不可用，请刷新后重试');
    await useExpert(expert.entrySkill || expert.slug, expert.name, expert);
  }
  return true;
}, error => props.store.notify(String(error), 'error'), computed({ get: () => chat.draft.connectors, set: value => { chat.draft.connectors = value; } }), () => chat.draft);
/** A prompt handed over by another page (装扮 → 用 AI 创作…) lands in the box that is shown, unsent. */
watch([() => props.store.creationDraft, () => chat.configured, shown], ([text, configured, now]) => {
  if (!text || !configured || !now) return;
  draft.value = text;
  props.store.creationDraft = null;
  const handed = props.store.creationAttachments ?? [];
  props.store.creationAttachments = null;
  for (const item of handed) if (attachments.value.length < MAX_IMAGES) attachments.value.push(item);
}, { immediate: true });

const placeholderText = computed(() => !props.store.desktop ? '请在桌面应用中使用小伴 AI' : !chat.configured ? '先到设置里配置模型' : props.placeholder || '问点什么，或交给小伴去办就行');
defineExpose({ setDraft, insertPrompt, refreshModel: readModelLabel, draft });
</script>

<template>
  <footer class="composer" :class="[`page-${page}`]" @compositionstart="composing = true" @compositionend="composing = false" @paste="onPaste" @dragover.prevent @drop.prevent="addFiles(Array.from($event.dataTransfer?.files ?? []))">
    <ChatMessageQueue :chat="chat" :running="running" />
    <div class="composer-content">
    <ChatComposerPet v-if="pet && store.settings?.chatComposerPetEnabled !== false" :size="pet" :paused="store.settings?.paused ?? false" />
    <div v-if="live && liveActive" class="composer-box live-bar" :class="{ muted: live.muted }" data-testid="chat-live">
      <span class="live-orb" :class="live.status === '在说' ? 'speak' : live.status === '在听' ? 'listen' : 'wait'" aria-hidden="true"><i></i><i></i><i></i><i></i></span>
      <div class="live-text" role="status" aria-live="polite" data-testid="chat-live-status"><b :title="live.status">{{ live.status }}</b><span :title="liveHint">{{ liveHint }}</span></div>
      <span class="live-clock" aria-label="通话时长">{{ liveClock }}</span>
      <button type="button" class="live-mute" :class="{ on: live.muted }" :aria-label="live.muted ? '取消静音' : '静音'" :title="live.muted ? '取消静音' : '静音'" :aria-pressed="live.muted" data-testid="chat-live-mute" @click="live.toggleMute()"><MicOff v-if="live.muted" :size="16" /><Mic v-else :size="16" /></button>
      <button type="button" class="live-end" aria-label="结束通话" title="结束通话" data-testid="chat-live-end" @click="live.end()"><Square :size="10" fill="currentColor" aria-hidden="true" /><span class="live-end-label">结束</span></button>
    </div>
    <div v-else class="composer-box" :class="{ 'voice-active': asr?.listening }">
      <AsrStatus v-if="asr" :asr="asr" chat />
      <div v-if="attachments.length" class="attachments" data-testid="chat-attachments">
        <div v-for="(url, n) in attachments" :key="n" class="thumb">
          <img v-if="attachmentImage(url)" :src="attachmentImage(url)!" :alt="`待发送的第 ${n + 1} 张图片`" :data-testid="`chat-attachment-${n}`" @click="emit('preview', attachmentImage(url)!)" />
          <span v-if="!attachmentImage(url)" class="file-chip" :title="attachmentName(url)">📄 {{ attachmentName(url) }}</span>
          <button type="button" class="thumb-remove" :data-testid="`chat-attachment-remove-${n}`" :aria-label="`移除第 ${n + 1} 个附件`" :disabled="attachmentReading" @click="removeAttachment(n)"><X :size="12" /></button>
        </div>
        <small class="attach-hint">{{ attachments.length }}/{{ MAX_IMAGES }} 个附件 · 发送时会一起交给小伴</small>
      </div>
      <input ref="filePicker" type="file" :accept="ATTACHMENT_ACCEPT" multiple hidden data-testid="chat-image-input" @change="onPick" />
      <div v-if="skills.length || selectedTool || chat.currentExpert || mentions.selected.value.length" class="composer-top" data-testid="chat-composer-top">
        <ul class="skill-chips" aria-label="这条消息用的技能和工具">
          <li v-for="skill in skills" :key="skill.name" class="skill-chip" :data-testid="`chat-skill-${skill.name}`"><Zap :size="14" aria-hidden="true" /><span :title="`/${skill.name}`">{{ skill.title }}</span><button type="button" class="chip-x" :aria-label="`移除技能 ${skill.title}`" @click="removeSkill(skill.name)"><X :size="11" :stroke-width="2.6" /></button></li>
          <li v-if="selectedTool" class="skill-chip" data-testid="chat-selected-tool"><Monitor v-if="selectedTool === 'computer'" :size="14" aria-hidden="true" /><Smartphone v-else-if="selectedTool === 'phone'" :size="14" aria-hidden="true" /><PanelsTopLeft v-else :size="14" aria-hidden="true" /><span>{{ builtinToolLabels[selectedTool] }}</span><button type="button" class="chip-x" :aria-label="`移除工具 ${builtinToolLabels[selectedTool]}`" data-testid="chat-remove-tool" @click="selectedTool = null"><X :size="11" :stroke-width="2.6" /></button></li>
        </ul>
        <div class="composer-ctx">
          <ChatExpertChip v-if="chat.currentExpert" compact :expert="chat.currentExpert" :removable="!chat.currentId" :desktop="store.desktop" @clear="chat.pendingExpert = null" />
          <ChatConnectorCluster :selected="mentions.selected.value" :desktop="store.desktop" :disabled="running" :down="page === 'new-task'" @toggle="toggleConnector" @manage="store.navigate('ai-store')" />
        </div>
      </div>
      <ChatSlashMenu v-if="slashOpen" :items="slashItems" :active="slashIndex" @pick="pickSkill" @hover="slashIndex = $event" />
      <ChatMentionMenu v-if="mentions.open.value" :items="mentions.items.value" :active="mentions.index.value" :loading="mentions.loading.value" @pick="mentions.pick" @hover="mentions.index.value = $event" />
      <div class="composer-editor">
        <NInput ref="input" v-model:value="draft" type="textarea" :autosize="{ minRows, maxRows: 8 }" :disabled="!store.desktop || !chat.configured" :placeholder="placeholderText" data-testid="chat-input" @keydown="onKeydown" />
      </div>
      <div class="composer-row">
        <ChatWorkbenchTools :store="store" :summary="addSummary" :disabled="!store.desktop" :file-disabled="!store.desktop || !chat.configured || attachmentReading || attachments.length >= MAX_IMAGES" :screenshot-disabled="!store.desktop || !chat.configured || capturing || attachments.length >= MAX_IMAGES" @file="openPicker" @screenshot="takeScreenshot" @use-expert="useExpert" @use-skill="useSkill" @use-connector="useConnector" @use-tool="useTool" @check-computer-permissions="checkComputerPermissions" @computer-disabled="computerDisabled" />
        <ChatWorkspaceMenu v-if="workspace" :chat="chat" :disabled="running || !store.desktop" />
        <ChatPermissionMenu :store="store" :disabled="!store.desktop" />
        <span class="composer-spacer" />
        <ChatContextRing :session-id="chat.currentId" v-if="store.desktop" :inert="!chat.currentId && page === 'new-task'" :used="chat.context.used" :limit="chat.context.limit" :can-compact="chat.context.canCompact" :compacting="compacting" :busy="running" :disabled="!chat.currentId" @compact="chat.compact()" @resize="(tokens, id) => chat.setContextSize(tokens, id)" />
        <ChatComposerModel :chat="chat" :label="modelLabel" :provider="modelProvider" :disabled="!store.desktop || running || chat.sending" @switched="modelSwitched" @settings="emit('settings')" />
        <button v-if="finishingVoice" type="button" class="voice-finish" data-testid="chat-asr-toggle" aria-label="正在转写" aria-pressed="false" disabled><LoaderCircle class="spin" :size="13" aria-hidden="true" />正在转写</button>
        <button v-else-if="asr?.recordingRequested" type="button" class="voice-finish" data-testid="chat-asr-toggle" :aria-label="asr.listening ? '结束录音' : '取消准备'" aria-pressed="true" @click="toggleVoice"><span v-if="asr.listening" class="recording-dot" aria-hidden="true" /><LoaderCircle v-else class="spin" :size="13" aria-hidden="true" />{{ asr.listening ? '结束录音' : '取消准备' }}</button>
        <button v-else-if="asr" type="button" class="xb-icon-btn voice" data-testid="chat-asr-toggle" aria-label="开始录音" aria-pressed="false" title="录音：说完转成文字，留在输入框，由你确认后发送" :disabled="!store.desktop || !asr.ready || asr.microphoneSupported === false || !microphoneAllowed || asr.saving" @click="toggleVoice"><Mic :size="17" /></button>
        <button v-if="offersLive" type="button" class="xb-icon-btn live-start" data-testid="chat-live-start" aria-label="实时对话" title="实时对话：像打电话一样和小伴说，随时可以打断" :disabled="liveDisabled" @click="startLive"><AudioLines :size="18" :stroke-width="2" /></button>
        <button v-if="running" type="button" class="xb-icon-btn stop-run" data-testid="chat-stop" aria-label="停止" title="停止当前任务并暂停队列" @click="chat.cancel()"><Square :size="13" fill="currentColor" /></button>
        <div class="delivery-control" @focusout="deliveryBlur" @keydown.esc="deliveryMenu = false">
          <div v-if="deliveryMenu && running" class="delivery-menu" role="menu" aria-label="消息发送方式">
            <button type="button" role="menuitem" @click="submit('queue')">加入队列<span>当前任务结束后发送</span></button>
            <button type="button" role="menuitem" @click="submit('steer')">补充当前任务<span>下一次模型请求时接收</span></button>
            <button type="button" role="menuitem" @click="submit('replace')">停止并发送<span>其余排队消息暂停</span></button>
          </div>
          <button type="button" class="send-btn" data-testid="chat-send" :aria-label="running || chat.queue?.items.length ? '加入队列' : '发送'" :title="running || chat.queue?.items.length ? '加入队列（Enter）' : '发送（Enter）'" :disabled="!canSend" @click="submit()"><ListPlus v-if="running || chat.queue?.items.length" :size="17" /><ArrowUp v-else :size="17" :stroke-width="2.4" /></button>
          <button v-if="running" type="button" class="delivery-toggle" data-testid="chat-delivery-menu" aria-label="选择发送方式" :aria-expanded="deliveryMenu" :disabled="!canSend" @click="deliveryMenu = !deliveryMenu"><ChevronDown :size="12" /></button>
        </div>
      </div>
    </div>
    </div>
  </footer>
</template>

<style scoped>
.delivery-control{position:relative;display:flex;align-items:center;gap:2px}.delivery-toggle{display:grid;place-items:center;border:0;border-radius:5px;background:none;color:var(--ui-text-secondary);width:16px;height:28px;padding:0;cursor:pointer}.delivery-toggle:disabled{opacity:.35;cursor:default}.delivery-toggle:hover:not(:disabled){background:var(--ui-surface-hover)}.delivery-menu{position:absolute;bottom:44px;right:0;width:205px;padding:5px;background:var(--ui-surface);border:1px solid var(--ui-border);border-radius:12px;box-shadow:0 8px 30px rgba(0,0,0,.12);z-index:35}.delivery-menu button{display:block;width:100%;border:0;border-radius:8px;padding:9px 10px;text-align:left;background:none;color:var(--ui-text-primary);font:inherit;font-size:12px;cursor:pointer}.delivery-menu button:hover{background:var(--ui-surface-hover)}.delivery-menu span{display:block;margin-top:3px;color:var(--ui-text-tertiary);font-size:10px}.stop-run{flex:none}

.composer-content{position:relative;container:chat-composer / inline-size}
.composer{position:relative;width:100%;box-sizing:border-box}
.composer-box{position:relative;display:flex;flex-direction:column;gap:4px;background:var(--ui-surface);border-radius:18px;box-shadow:0 0 0 1px var(--ui-border),0 2px 4px rgba(0,0,0,.03),0 12px 32px rgba(0,0,0,.07);padding:14px 12px 8px 16px;transition:box-shadow .15s}
html[data-theme=dark] .composer-box{background:var(--ui-surface-alt);box-shadow:0 0 0 1px var(--ui-border-strong),0 12px 32px rgba(0,0,0,.35)}
.composer-box:focus-within{box-shadow:0 0 0 1px var(--ui-border-strong),0 2px 4px rgba(0,0,0,.03),0 12px 32px rgba(0,0,0,.09)}
.composer-box.voice-active{box-shadow:0 0 0 1.5px var(--ui-btn),0 0 0 4px var(--ui-soft),0 12px 32px rgba(0,0,0,.07)}
.page-new-task .composer-box{padding:16px 14px 10px 18px}
.composer-top{display:flex;align-items:flex-start;gap:8px;min-width:0;min-height:28px;margin-bottom:4px}
.skill-chips{flex:1;min-width:0;display:flex;flex-wrap:wrap;gap:6px;margin:0;padding:0;list-style:none}
.skill-chip{display:inline-flex;align-items:center;gap:6px;max-width:100%;height:28px;padding:0 4px 0 9px;border-radius:8px;background:var(--ui-soft);color:var(--ui-ink);font-size:13px;font-weight:500}
.skill-chip svg{flex:none}
.skill-chip span{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.chip-x{flex:none;display:grid;place-items:center;width:20px;height:20px;padding:0;border:0;border-radius:6px;background:transparent;color:inherit;cursor:pointer}
.chip-x:hover:not(:disabled){background:var(--ui-control-hover)}
.chip-x:disabled{opacity:.5;cursor:not-allowed}
.chip-x:focus-visible{outline:2px solid var(--ui-btn);outline-offset:1px}
.composer-ctx{display:flex;align-items:center;gap:2px;flex:none;min-width:0;margin-left:auto;max-width:70%}
.composer-editor{display:flex;min-width:0;padding:0 2px 2px}
.composer-editor :deep(.n-input){flex:1;min-width:0;width:100%;--n-border:none;--n-border-hover:none;--n-border-focus:none;--n-box-shadow-focus:none;--n-color:transparent;--n-color-focus:transparent;--n-color-disabled:transparent;background:transparent;font-size:14.5px}
.page-new-task .composer-editor :deep(.n-input){font-size:15px}
.composer-editor :deep(.n-input .n-input__border),.composer-editor :deep(.n-input .n-input__state-border){display:none}
.composer-editor :deep(.n-input-wrapper){padding:0}
.composer-editor :deep(.n-input__textarea-el),.composer-editor :deep(.n-input__placeholder){line-height:1.6;padding:0}
.composer-row{display:flex;gap:2px;align-items:center;min-width:0}
.composer-spacer{flex:1;min-width:4px}
.voice{color:var(--ui-text-secondary)}
.voice:disabled{opacity:.45;cursor:not-allowed}
.live-start{color:var(--ui-ink)}
.live-start:hover:not(:disabled){background:var(--ui-soft);color:var(--ui-ink)}
.live-start:disabled{opacity:.45;cursor:not-allowed}
.send-btn{flex:none;width:34px;height:34px;margin-left:4px;border-radius:50%;border:0;display:grid;place-items:center;cursor:pointer;color:var(--ui-text-on-btn);background:linear-gradient(var(--ui-btn-hi),var(--ui-btn));box-shadow:inset 0 1px 0 rgba(255,255,255,.3),0 2px 6px rgba(0,0,0,.15)}
.page-chat .send-btn{width:32px;height:32px}
.send-btn:hover:not(:disabled){filter:brightness(1.05)}
.send-btn:disabled{background:var(--ui-control-hover);color:var(--ui-text-disabled);box-shadow:none;cursor:not-allowed}
.send-btn:focus-visible{outline:2px solid var(--ui-btn);outline-offset:2px}
.voice-finish{display:inline-flex;align-items:center;justify-content:center;gap:7px;min-height:28px;flex:none;padding:3px 10px;border:0;border-radius:999px;background:var(--ui-soft);color:var(--ui-ink);font:inherit;font-size:12px;font-weight:500;cursor:pointer;white-space:nowrap}
.voice-finish:hover{filter:brightness(.97)}
.voice-finish:disabled{cursor:wait;opacity:.75}
.recording-dot{width:7px;height:7px;border-radius:50%;background:var(--ui-danger);animation:recording-pulse 1.3s ease-in-out infinite}
@keyframes recording-pulse{50%{opacity:.3}}
.spin{animation:spin 1s linear infinite}
@keyframes spin{to{transform:rotate(360deg)}}
.attachments{display:flex;flex-wrap:wrap;align-items:center;gap:8px;padding:0 0 6px}
.thumb{position:relative;width:56px;height:56px;border-radius:10px;overflow:hidden;box-shadow:0 0 0 1px var(--ui-border);background:var(--ui-control);flex:none}
.thumb img{width:100%;height:100%;object-fit:cover;cursor:zoom-in;display:block}
.thumb:has(.file-chip){width:auto;max-width:210px}
.file-chip{display:inline-block;max-width:200px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:12px;padding:8px 22px 8px 8px;border-radius:8px}
.thumb-remove{position:absolute;top:3px;right:3px;width:17px;height:17px;border-radius:50%;border:0;background:rgba(0,0,0,.58);color:#fff;display:grid;place-items:center;cursor:pointer;padding:0}
.thumb-remove:hover:not(:disabled){background:rgba(0,0,0,.78)}
.thumb-remove:disabled{cursor:not-allowed;opacity:.5}
.attach-hint{color:var(--ui-text-tertiary);font-size:12px}
.live-bar{flex-direction:row;align-items:center;gap:12px;padding:10px 10px 10px 12px;box-shadow:0 0 0 1.5px var(--ui-btn),0 0 0 4px var(--ui-soft),0 12px 32px rgba(0,0,0,.07)}
.live-bar.muted{box-shadow:0 0 0 1px var(--ui-border),0 12px 32px rgba(0,0,0,.07)}
.live-orb{flex:none;width:44px;height:44px;border-radius:50%;display:flex;align-items:center;justify-content:center;gap:3px;background:var(--ui-soft);color:var(--ui-btn)}
.live-bar.muted .live-orb{background:var(--ui-control);color:var(--ui-text-tertiary)}
.live-orb i{display:block;width:3.5px;height:6px;border-radius:2px;background:currentColor}
.live-orb.listen i{animation:live-listen 1.1s ease-in-out infinite}
.live-orb.speak i{animation:live-speak .75s ease-in-out infinite}
.live-orb.wait i{height:5px;animation:live-wait 1.2s ease-in-out infinite}
.live-orb i:nth-child(2){animation-delay:.12s}.live-orb i:nth-child(3){animation-delay:.24s}.live-orb i:nth-child(4){animation-delay:.36s}
.live-bar.muted .live-orb i{animation:none;height:3px}
@keyframes live-listen{50%{height:13px}}
@keyframes live-speak{50%{height:22px}}
@keyframes live-wait{50%{opacity:.3}}
.live-text{flex:1;min-width:0;display:flex;flex-direction:column;gap:1px}
.live-text b{font-size:14.5px;font-weight:600;color:var(--ui-text-primary);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.live-text span{font-size:12px;color:var(--ui-text-secondary);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.live-clock{flex:none;font:12.5px 'SF Mono',Menlo,Consolas,monospace;font-variant-numeric:tabular-nums;color:var(--ui-text-secondary)}
.live-mute{flex:none;width:40px;height:40px;border:0;border-radius:50%;display:grid;place-items:center;background:var(--ui-control);color:var(--ui-text-primary);cursor:pointer}
.live-mute:hover{background:var(--ui-control-hover)}
.live-mute.on{background:var(--ui-text-primary);color:var(--ui-surface)}
.live-end{flex:none;display:inline-flex;align-items:center;justify-content:center;gap:6px;height:40px;padding:0 16px;border:0;border-radius:20px;background:var(--ui-text-primary);color:var(--ui-surface);font:inherit;font-size:13.5px;font-weight:500;white-space:nowrap;cursor:pointer}
.live-end:hover{filter:brightness(1.1)}
.live-mute:focus-visible,.live-end:focus-visible{outline:2px solid var(--ui-btn);outline-offset:2px}
/* Split views shrink the composer without changing the window width. */
@container chat-composer (max-width:360px){
  .composer-box.live-bar{gap:8px;padding:10px}
  .live-orb{width:32px;height:32px}
  .live-mute,.live-end{width:36px;height:36px;padding:0}
  .live-end-label{display:none}
}
@container chat-composer (max-width:260px){.live-orb{display:none}}
@container chat-composer (max-width:200px){.live-clock{display:none}}
@media(max-width:620px){.composer-pet{display:none}}
@media(prefers-reduced-motion:reduce){.spin,.recording-dot,.live-orb i{animation:none}}
</style>
