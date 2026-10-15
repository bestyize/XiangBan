<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { NButton, NInput } from 'naive-ui';
import { ArrowDown, Check, FolderOpen, ImagePlus, LoaderCircle, MessageSquarePlus, Mic, SendHorizontal, Volume2, Shield, ShieldAlert, Sparkles, Square, Trash2, X, Zap } from 'lucide-vue-next';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import ModelSwitchDialog from '../components/ModelSwitchDialog.vue';
import GuideModal from '../components/GuideModal.vue';
import AsrStatus from '../components/AsrStatus.vue';
import MarkdownView from '../chat/MarkdownView.ts';
import { CONTINUE_TEXT } from '../chat/chat-store.ts';
import { approvalPreview, clip, formatAgo, isImeEnter, isNearBottom, prettyArguments, riskVerb, toolLabel } from '../chat/format.ts';
import { describeCall } from '../chat/view.ts';
import type { ApprovalDecision, ApprovalRequest, QuestionRequest } from '../chat/types.ts';

const props = defineProps<{ store: ControlStore }>();
const chat = props.store.chat;

const draft = ref('');
const asr = props.store.asr;
const voiceHasText = ref(false);
const finishingVoice = ref(false);
const foreground = ref(document.visibilityState !== 'hidden');
function updateVisibility() { foreground.value = document.visibilityState !== 'hidden'; }
function blurChat() { if (document.visibilityState === 'hidden') foreground.value = false; }
function focusChat() { updateVisibility(); }
const modelDialog = ref(false);
const guideDialog = ref(false);
const retryAfterSwitch = ref(false);
function chooseModel(retry = false) { retryAfterSwitch.value = retry; modelDialog.value = true; }
async function modelSwitched() { await chat.refresh(); if (retryAfterSwitch.value) await chat.retry(); props.store.notify('模型已切换', 'success'); }
const composing = ref(false);
const scroller = ref<HTMLElement | null>(null);
const stick = ref(true);
const confirmingId = ref('');
const answerText = ref('');
const examples = ['帮我看看工作目录里有什么，整理一份清单', '把壁纸换成一张安静的风景', '每天下午三点提醒我喝水', '查一下今天是几号，再告诉我这周还剩几天'];

/** Images waiting to be sent, as `data:` URLs. Cleared on send, alongside the draft. */
const attachments = ref<string[]>([]);
const filePicker = ref<HTMLInputElement | null>(null);
/** The image shown enlarged, or null. */
const preview = ref<string | null>(null);
/** How many images one message may carry, matching the host's own limit. */
const MAX_IMAGES = 8;
/** The host refuses anything bigger; checking here keeps a huge paste from being read at all. */
const MAX_IMAGE_BYTES = 3 * 1024 * 1024;
const IMAGE_TYPES = ['image/png', 'image/jpeg', 'image/gif', 'image/webp'];

const view = computed(() => chat.current());
const running = computed(() => !!view.value?.running);
const title = computed(() => (chat.currentId && chat.sessions.find(s => s.id === chat.currentId)?.title) || '新对话');
const canSend = computed(() => chat.configured && !running.value && (draft.value.trim() !== '' || attachments.value.length > 0));
const workspaceName = computed(() => chat.workspace.split(/[\\/]/).filter(Boolean).slice(-2).join('/') || chat.workspace);
const now = () => Math.floor(Date.now() / 1000);
const waiting = computed(() => !!view.value && (view.value.approvals.length > 0 || view.value.questions.length > 0));
const thinking = computed(() => {
  const v = view.value;
  if (!v?.running || waiting.value) return false;
  const last = v.items[v.items.length - 1];
  return !(last?.kind === 'assistant' && last.streaming);
});
const statusLine = computed(() => view.value?.status || (waiting.value ? '等你确认后继续' : thinking.value ? '思考中…' : ''));

const microphoneAllowed = computed(() => foreground.value && chat.configured && !running.value && (!props.store.speech || props.store.speech.status === 'idle'));
watch(() => chat.currentId, () => asr?.deactivate(), { flush: 'sync' });
watch(microphoneAllowed, allowed => { if (!allowed) asr?.deactivate(); }, { flush: 'sync' });
watch(() => props.store.page, page => { if (page !== 'chat') asr?.deactivate(); }, { flush: 'sync' });
async function toggleVoice() {
  if (!asr || finishingVoice.value) return;
  if (asr.recordingRequested) {
    const wasListening = asr.listening;
    const sessionId = chat.currentId;
    finishingVoice.value = true;
    try {
      await asr.finish();
      if (!wasListening || asr.error || props.store.page !== 'chat' || !foreground.value || chat.currentId !== sessionId || running.value) return;
      if (!voiceHasText.value) { props.store.notify('没有识别到语音，请重试', 'info'); return; }
      await submit();
    } finally { finishingVoice.value = false; }
    return;
  }
  if (!microphoneAllowed.value || props.store.page !== 'chat') return;
  voiceHasText.value = false;
  void asr.activate(text => { voiceHasText.value = true; draft.value += (draft.value && !/\s$/.test(draft.value) ? ' ' : '') + text; });
}
onMounted(() => {
  asr?.deactivate();
  void asr?.init();
  document.addEventListener('visibilitychange', updateVisibility);
  window.addEventListener('blur', blurChat); window.addEventListener('focus', focusChat);
});
onUnmounted(() => {
  asr?.deactivate(); document.removeEventListener('visibilitychange', updateVisibility);
  window.removeEventListener('blur', blurChat); window.removeEventListener('focus', focusChat);
});
onMounted(async () => {
  const first = !chat.ready;
  await chat.init();
  if (!first) await chat.refresh();
});

function toBottom() {
  const el = scroller.value;
  if (el) el.scrollTop = el.scrollHeight;
}
function onScroll() {
  if (scroller.value) stick.value = isNearBottom(scroller.value);
}
const signature = computed(() => {
  const v = view.value;
  if (!v) return '';
  const last = v.items[v.items.length - 1];
  return [v.id, v.items.length, last && 'text' in last ? last.text.length : 0, last && 'output' in last ? last.output.length : 0, v.approvals.length, v.questions.length].join('|');
});
watch(signature, async (value, before) => {
  if (value.split('|')[0] !== (before ?? '').split('|')[0]) stick.value = true;
  await nextTick();
  if (stick.value) toBottom();
});

async function submit() {
  const text = draft.value.trim();
  if ((!text && !attachments.value.length) || !canSend.value) return;
  asr?.deactivate();
  const images = [...attachments.value];
  const kept = text;
  draft.value = '';
  attachments.value = [];
  stick.value = true;
  if (!(await chat.send(text, images)) && !draft.value) {
    // The host refused, so nothing was sent: hand the whole message back untouched.
    draft.value = kept;
    attachments.value = images;
  }
  await nextTick();
  toBottom();
}

/** Reads a picked or pasted file into a `data:` URL, refusing anything the host would reject. */
function readImage(file: File): Promise<string | null> {
  return new Promise(resolve => {
    if (!IMAGE_TYPES.includes(file.type)) {
      props.store.notify('只能发送 PNG、JPEG、GIF 或 WebP 图片', 'error');
      resolve(null);
      return;
    }
    if (file.size > MAX_IMAGE_BYTES) {
      props.store.notify('这张图片太大了，请换一张小于 3MB 的', 'error');
      resolve(null);
      return;
    }
    const reader = new FileReader();
    reader.onerror = () => { props.store.notify('这张图片读不出来，请换一张', 'error'); resolve(null); };
    reader.onload = () => resolve(typeof reader.result === 'string' ? reader.result : null);
    reader.readAsDataURL(file);
  });
}

async function addImages(files: File[]) {
  if (!files.length) return;
  const room = MAX_IMAGES - attachments.value.length;
  if (room <= 0) {
    props.store.notify(`一次最多发送 ${MAX_IMAGES} 张图片`, 'error');
    return;
  }
  if (files.length > room) props.store.notify(`一次最多发送 ${MAX_IMAGES} 张图片，多余的没有加上`, 'info');
  const read = await Promise.all(files.slice(0, room).map(readImage));
  const accepted = read.filter((url): url is string => !!url);
  if (accepted.length) attachments.value = [...attachments.value, ...accepted];
}

/** Images on the clipboard arrive as files; anything else is left to the normal paste. */
function onPaste(event: ClipboardEvent) {
  const files = Array.from(event.clipboardData?.files ?? []).filter(file => file.type.startsWith('image/'));
  if (!files.length) return;
  event.preventDefault();
  void addImages(files);
}

function onPick(event: Event) {
  const input = event.target as HTMLInputElement;
  void addImages(Array.from(input.files ?? []));
  // Let the same file be chosen again after it is removed.
  input.value = '';
}

const removeAttachment = (index: number) => { attachments.value = attachments.value.filter((_, n) => n !== index); };
const openPicker = () => filePicker.value?.click();

function onKeydown(event: KeyboardEvent) {
  if (event.key !== 'Enter' || event.shiftKey || isImeEnter(event, composing.value)) return;
  event.preventDefault();
  void submit();
}
const useExample = (text: string) => { draft.value = text; };
const continueAnswer = () => { asr?.deactivate(); stick.value = true; void chat.send(CONTINUE_TEXT); };

const choose = (id: string | null) => { asr?.deactivate(); confirmingId.value = ''; void chat.select(id); };
async function remove(id: string) { confirmingId.value = ''; await chat.remove(id); }
function openSettings() { modelDialog.value = false; props.store.settingsTab = 'ai'; props.store.navigate('settings'); }

const decide = (request: ApprovalRequest, decision: ApprovalDecision) => chat.answerApproval(request, decision);
function answer(request: QuestionRequest, value: string | null) {
  answerText.value = '';
  return chat.answerQuestion(request, value);
}

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
  <ModelSwitchDialog v-model:show="modelDialog" :retry="retryAfterSwitch" @switched="modelSwitched" @settings="openSettings" />
  <GuideModal v-model:show="guideDialog" :desktop="store.desktop" @select-prompt="useExample" @open-settings="openSettings" />
  <section class="chat-page" data-testid="chat-page">
    <aside class="chat-side" aria-label="对话列表">
      <div class="side-head">
        <NButton type="primary" block data-testid="chat-new" :disabled="!chat.currentId" @click="choose(null)"><template #icon><MessageSquarePlus :size="16" /></template>新对话</NButton>
      </div>
      <ul class="session-list">
        <li v-for="s in chat.sessions" :key="s.id" :class="{ current: s.id === chat.currentId }">
          <template v-if="confirmingId === s.id">
            <div class="confirm-row" data-testid="chat-delete-confirm"><span>删除这段对话？</span><button type="button" class="link danger" :data-testid="`chat-delete-yes-${s.id}`" @click="remove(s.id)">删除</button><button type="button" class="link" @click="confirmingId = ''">取消</button></div>
          </template>
          <template v-else>
            <button type="button" class="session" :data-testid="`chat-session-${s.id}`" :aria-current="s.id === chat.currentId ? 'true' : undefined" @click="choose(s.id)">
              <span class="session-title">{{ s.title }}</span>
              <span class="session-meta">
                <LoaderCircle v-if="chat.views[s.id]?.running" class="spin" :size="12" aria-label="正在回答" />
                <i v-if="chat.views[s.id] && (chat.views[s.id].approvals.length || chat.views[s.id].questions.length)" class="attention-dot" :data-testid="`chat-attention-${s.id}`" aria-label="需要你确认"></i>
                {{ formatAgo(s.updatedAt, now()) }}
              </span>
            </button>
            <button type="button" class="session-delete" :data-testid="`chat-delete-${s.id}`" :aria-label="`删除 ${s.title}`" :disabled="chat.views[s.id]?.running" @click="confirmingId = s.id"><Trash2 :size="14" /></button>
          </template>
        </li>
        <li v-if="chat.ready && !chat.sessions.length" class="empty-list">还没有对话</li>
      </ul>
    </aside>

    <div class="chat-main">
      <header class="chat-head">
        <h2 data-testid="chat-title">{{ title }}</h2>
        <div class="head-actions">
          <button v-if="store.speech && store.speech.status !== 'idle'" type="button" class="link" data-testid="chat-speech-status" @click="store.speech.stop()"><Volume2 :size="14" />{{ store.speech.status === 'synthesizing' ? '取消合成' : '停止朗读' }}</button>
          <button
            type="button"
            class="guide-chip"
            data-testid="chat-open-guide"
            title="查看小伴 AI 玩法指南与提示词手册"
            @click="guideDialog = true"
          >
            <Sparkles :size="13" />
            <span>玩法指南</span>
          </button>
          <div class="permission-toggle" role="group" aria-label="权限模式" data-testid="chat-permission-toggle">
            <button
              type="button"
              class="perm-btn"
              :class="{ active: chat.permissionMode === 'ask' }"
              data-testid="perm-ask"
              title="默认权限：读操作自动执行，写文件、运行命令与修改设置前需确认"
              @click="chat.setPermissionMode('ask')"
            >
              <Shield :size="12" />
              <span>默认权限</span>
            </button>
            <button
              type="button"
              class="perm-btn"
              :class="{ active: chat.permissionMode === 'auto' }"
              data-testid="perm-auto"
              title="完全信任：自动执行所有操作（文件修改、命令等无需逐一确认）"
              @click="chat.setPermissionMode('auto')"
            >
              <Zap :size="12" />
              <span>完全信任</span>
            </button>
          </div>
          <button v-if="chat.workspace" type="button" class="workspace-chip" data-testid="chat-workspace" :title="`工作目录：${chat.workspace}\n点击到设置里更改`" @click="openSettings"><FolderOpen :size="14" />{{ workspaceName }}</button>
        </div>
      </header>

      <div ref="scroller" class="chat-scroll" data-testid="chat-messages" @scroll.passive="onScroll">
        <div class="chat-column">
          <p v-if="!chat.ready" class="muted state-line">正在读取对话…</p>
          <div v-else-if="!store.desktop" class="info-card" data-testid="chat-preview-only"><b>请在桌面应用中使用聊天</b><p>浏览器预览里只能看到界面。</p></div>
          <div v-else-if="!chat.configured && !view?.items.length" class="info-card" data-testid="chat-unconfigured">
            <b>先给小伴接上一个模型</b>
            <p>在“设置 → AI 模型”里填写接口地址、模型名，需要的话再填密钥。填好后回到这里就能聊天。</p>
            <NButton type="primary" data-testid="chat-go-settings" @click="openSettings">去设置</NButton>
          </div>
          <div v-else-if="!view?.items.length" class="welcome" data-testid="chat-welcome">
            <img src="/xiaoban.svg" alt="" width="56" height="56" />
            <h3>有什么可以帮你？</h3>
            <p>我能回答问题，也能在你选的工作目录里读写文件、运行命令，还能替你调整壁纸、特效和提醒。改动之前我会先问你。</p>
            <div class="examples">
              <button v-for="(text, n) in examples" :key="text" type="button" :data-testid="`chat-example-${n}`" @click="useExample(text)">
                <span class="example-tag">{{ ['📁 工作区', '🖼️ 壁纸', '🍵 关怀', '📅 日常'][n] ?? '💡 尝试' }}</span>
                <span>{{ text }}</span>
              </button>
            </div>
            <button type="button" class="more-guides-link" data-testid="chat-welcome-more-guides" @click="guideDialog = true">
              <Sparkles :size="14" />
              <span>查看更多 AI 精彩玩法与咒语手册 →</span>
            </button>
          </div>

          <template v-if="view">
            <template v-for="item in view.items" :key="item.id">
              <div v-if="item.kind === 'user'" class="msg user" data-testid="msg-user"><div class="bubble"><div v-if="item.images.length" class="bubble-images"><img v-for="(url, n) in item.images" :key="n" :src="url" :alt="`第 ${n + 1} 张图片`" :data-testid="`msg-user-image-${n}`" @click="preview = url" /></div><template v-if="item.text">{{ item.text }}</template></div></div>
              <div v-else-if="item.kind === 'assistant'" class="msg assistant" data-testid="msg-assistant">
                <img class="avatar" src="/xiaoban.svg" alt="" width="28" height="28" />
                <div class="answer"><MarkdownView :source="item.text" @open="openLink" /><span v-if="item.streaming" class="cursor" aria-hidden="true"></span>
                  <button v-if="store.speech && !item.streaming && item.text.trim()" type="button" class="link speech-button" :disabled="!store.desktop" :aria-label="store.speech.currentKey === item.id ? '停止朗读' : '朗读回复'" data-testid="chat-read-reply" @click="store.speech.currentKey === item.id ? store.speech.stop() : store.speech.speak(item.text, item.id)">
                    <Volume2 :size="14" />{{ store.speech.currentKey === item.id ? (store.speech.status === 'synthesizing' ? '取消合成' : '停止朗读') : '朗读' }}
                  </button>
                </div>
              </div>
              <template v-else-if="item.kind === 'tool'">
                <details v-if="!chat.hideToolCalls" class="tool" :class="item.status" :data-testid="`tool-${item.id}`">
                  <summary>
                    <LoaderCircle v-if="item.status === 'running'" class="spin" :size="14" /><Check v-else-if="item.status === 'done'" :size="14" /><X v-else :size="14" />
                    <b>{{ toolLabel(item.name) }}</b><span class="tool-sub">{{ describeCall(item.name, item.arguments) }}</span>
                  </summary>
                  <div class="tool-body">
                    <div v-if="item.arguments" class="tool-part"><small>内容</small><pre>{{ prettyArguments(item.arguments) }}</pre></div>
                    <div v-if="item.output" class="tool-part"><small>结果</small><pre>{{ clip(item.output) }}</pre></div>
                  </div>
                </details>
              </template>
              <div v-else-if="item.kind === 'notice'" class="notice" :class="item.tone" :data-testid="`notice-${item.tone}`" :role="item.tone === 'error' ? 'alert' : 'status'">
                {{ item.text }}
                <details v-if="item.details"><summary>错误详情</summary><span>{{ item.details }}</span></details>
                <button v-if="item.canSwitchModel && !view.running" type="button" class="link" @click="chooseModel(!!item.canRetry)">切换模型</button>
                <button v-if="item.canRetry && !view.running" type="button" class="link" @click="chat.retry()">重试</button>
                <button v-if="item.canContinue && !view.running" type="button" class="link" data-testid="chat-continue" @click="continueAnswer">继续</button>
              </div>
            </template>

            <section v-for="request in view.approvals" :key="request.id" class="card approval" :class="{ exec: request.risk === 'exec', dangerous: !!request.danger }" data-testid="chat-approval" role="alertdialog" :aria-label="`需要你确认：${toolLabel(request.tool)}`">
              <header><ShieldAlert :size="16" /><span>小伴想要{{ riskVerb(request.risk) }}：<b>{{ toolLabel(request.tool) }}</b></span></header>
              <p v-if="request.danger" class="caution danger-note" data-testid="approval-danger"><b>高风险操作：{{ request.danger }}。</b>即使在“完全信任”下，这类操作也要你确认，每次都问。</p>
              <p v-if="request.tainted" class="caution" data-testid="approval-tainted">这段对话读过网页等外部内容，其中可能夹带诱导，所以这次每一步都要你确认。</p>
              <pre class="preview" data-testid="approval-preview">{{ approvalPreview(request) }}</pre>
              <footer>
                <NButton type="primary" size="small" data-testid="approval-once" @click="decide(request, 'once')">允许一次</NButton>
                <NButton v-if="request.canRemember" size="small" data-testid="approval-always" @click="decide(request, 'always')">这段对话都允许</NButton>
                <NButton size="small" quaternary data-testid="approval-deny" @click="decide(request, 'deny')">拒绝</NButton>
              </footer>
            </section>

            <section v-for="request in view.questions" :key="request.id" class="card question" data-testid="chat-question" role="alertdialog" aria-label="小伴想问你">
              <header><span>{{ request.question }}</span></header>
              <div v-if="request.options.length" class="options"><NButton v-for="(option, n) in request.options" :key="option" size="small" :data-testid="`question-option-${n}`" @click="answer(request, option)">{{ option }}</NButton></div>
              <footer>
                <button type="button" class="link" :disabled="running" @click="chooseModel()">切换模型</button>
          <NInput v-model:value="answerText" size="small" placeholder="或者自己回答" data-testid="question-input" @keydown.enter="!isImeEnter($event, composing) && answerText.trim() && answer(request, answerText)" />
                <NButton type="primary" size="small" :disabled="!answerText.trim()" data-testid="question-submit" @click="answer(request, answerText)">回答</NButton>
                <NButton size="small" quaternary data-testid="question-skip" @click="answer(request, null)">跳过</NButton>
              </footer>
            </section>

            <p v-if="statusLine" class="status-line" data-testid="chat-status"><LoaderCircle class="spin" :size="14" />{{ statusLine }}</p>
          </template>
        </div>
      </div>

      <button v-if="!stick && view?.items.length" type="button" class="to-bottom" data-testid="chat-bottom" aria-label="回到最新" @click="stick = true; toBottom()"><ArrowDown :size="16" /></button>

      <footer class="composer" @compositionstart="composing = true" @compositionend="composing = false" @paste="onPaste">
        <div class="composer-box" :class="{ 'has-attachments': attachments.length > 0, 'voice-active': asr?.listening }">
          <AsrStatus v-if="asr" :asr="asr" chat />
          <div v-if="attachments.length" class="attachments" data-testid="chat-attachments">
            <div v-for="(url, n) in attachments" :key="n" class="thumb">
              <img :src="url" :alt="`待发送的第 ${n + 1} 张图片`" :data-testid="`chat-attachment-${n}`" @click="preview = url" />
              <button type="button" class="thumb-remove" :data-testid="`chat-attachment-remove-${n}`" :aria-label="`移除第 ${n + 1} 张图片`" :disabled="running" @click="removeAttachment(n)"><X :size="12" /></button>
            </div>
            <small class="attach-hint">{{ attachments.length }}/{{ MAX_IMAGES }} 张 · 发送时会一起交给小伴</small>
          </div>
          <div class="composer-row">
            <input ref="filePicker" type="file" accept="image/png,image/jpeg,image/gif,image/webp" multiple hidden data-testid="chat-image-input" @change="onPick" />
            <NButton quaternary circle size="small" data-testid="chat-pick-image" aria-label="选择图片" :disabled="!store.desktop || !chat.configured || running || attachments.length >= MAX_IMAGES" @click="openPicker"><template #icon><ImagePlus :size="16" /></template></NButton>
            <button v-if="finishingVoice" type="button" class="voice-finish" data-testid="chat-asr-toggle" aria-label="正在转写并发送" aria-pressed="false" disabled><LoaderCircle class="spin" :size="13" aria-hidden="true" />正在发送</button>
            <button v-else-if="asr?.recordingRequested" type="button" class="voice-finish" data-testid="chat-asr-toggle" :aria-label="asr.listening ? '结束录音并发送' : '取消准备'" aria-pressed="true" @click="toggleVoice"><span v-if="asr.listening" class="recording-dot" aria-hidden="true" /><LoaderCircle v-else class="spin" :size="13" aria-hidden="true" />{{ asr.listening ? '结束并发送' : '取消准备' }}</button>
            <NButton v-else-if="asr" quaternary circle size="small" data-testid="chat-asr-toggle" aria-label="开始语音输入" aria-pressed="false" title="开始语音输入" :disabled="!store.desktop || !asr.ready || !microphoneAllowed || asr.saving" @click="toggleVoice"><template #icon><Mic :size="16" /></template></NButton>
            <NInput v-model:value="draft" type="textarea" :autosize="{ minRows: 1, maxRows: 8 }" :disabled="!store.desktop || !chat.configured" :placeholder="chat.configured ? '问点什么，或者交给小伴去办（Enter 发送，Shift+Enter 换行，可直接粘贴图片）' : '先到设置里配置模型'" data-testid="chat-input" @keydown="onKeydown" />
            <NButton v-if="running" type="error" circle data-testid="chat-stop" aria-label="停止" @click="chat.cancel()"><template #icon><Square :size="14" /></template></NButton>
            <NButton v-else type="primary" circle data-testid="chat-send" aria-label="发送" :disabled="!canSend" @click="submit"><template #icon><SendHorizontal :size="16" /></template></NButton>
          </div>
        </div>
      </footer>
    </div>

    <div v-if="preview" class="lightbox" data-testid="chat-image-preview" role="dialog" aria-label="图片预览" @click="preview = null">
      <img :src="preview" alt="图片预览" />
      <button type="button" class="lightbox-close" data-testid="chat-image-preview-close" aria-label="关闭预览" @click="preview = null"><X :size="18" /></button>
    </div>
  </section>
</template>

<style scoped>
.speech-button{display:inline-flex;align-items:center;gap:5px;margin-top:8px;font-size:12px}

.chat-page{flex:1;min-height:0;display:grid;grid-template-columns:268px minmax(0,1fr);background:var(--ui-app-bg)}
.chat-side{border-right:1px solid var(--ui-border);display:flex;flex-direction:column;min-height:0;background:var(--ui-surface)}
.side-head{padding:16px 14px 10px}
.session-list{list-style:none;margin:0;padding:4px 8px 12px;overflow:auto;flex:1;scrollbar-width:thin}
.session-list li{position:relative;display:flex;align-items:center;border-radius:10px}
.session-list li:hover,.session-list li.current{background:var(--ui-surface-alt)}
.session{flex:1;min-width:0;text-align:left;background:none;border:0;color:var(--ui-text-primary);cursor:pointer;padding:9px 10px;display:grid;gap:3px;font:inherit}
.session-title{font-size:13.5px;font-weight:600;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.session-meta{font-size:12px;color:var(--ui-text-tertiary);display:flex;gap:6px;align-items:center}
.session-delete{opacity:0;border:0;background:none;color:var(--ui-text-tertiary);cursor:pointer;padding:6px;margin-right:4px;border-radius:6px}
.session-list li:hover .session-delete,.session-delete:focus-visible,.session-list li.current .session-delete{opacity:1}
.session-delete:hover:not(:disabled){color:var(--ui-primary)}
.session-delete:disabled{cursor:not-allowed;opacity:.3}
.confirm-row{flex:1;display:flex;gap:10px;align-items:center;padding:9px 10px;font-size:13px}
.empty-list{justify-content:center;padding:24px 0;color:var(--ui-text-tertiary);font-size:13px}
.attention-dot{width:7px;height:7px;border-radius:50%;background:var(--ui-primary);display:inline-block}
.link{border:0;background:none;color:var(--ui-primary);cursor:pointer;font:inherit;padding:0}
.link.danger{color:var(--ui-primary-pressed);font-weight:650}
.chat-main{display:flex;flex-direction:column;min-height:0;min-width:0;position:relative}
.chat-head{display:flex;align-items:center;justify-content:space-between;gap:12px;padding:14px 24px;border-bottom:1px solid var(--ui-border)}
.chat-head h2{font-size:15px;margin:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.head-actions{display:flex;align-items:center;gap:10px;flex-shrink:0}
.guide-chip{border:1px solid var(--ui-border);background:var(--ui-surface);color:var(--ui-primary);border-radius:999px;padding:4px 10px;font-size:12px;display:inline-flex;gap:5px;align-items:center;cursor:pointer;font-weight:600;transition:all .15s}
.guide-chip:hover{background:var(--ui-primary-soft);border-color:var(--ui-primary)}
.permission-toggle{display:inline-flex;padding:2px;background:var(--ui-surface-alt);border:1px solid var(--ui-line);border-radius:999px;gap:2px}
.perm-btn{border:0;background:none;color:var(--ui-text-tertiary);border-radius:999px;padding:3px 9px;font-size:12px;cursor:pointer;display:inline-flex;gap:4px;align-items:center;transition:all .15s}
.perm-btn:hover{color:var(--ui-text-primary)}
.perm-btn.active{background:var(--ui-surface);color:var(--ui-text-primary);box-shadow:0 1px 3px rgba(0,0,0,.08);font-weight:600}
.perm-btn.active svg{color:var(--ui-primary)}
.workspace-chip{border:1px solid var(--ui-border);background:var(--ui-surface);color:var(--ui-text-secondary);border-radius:999px;padding:4px 10px;font-size:12px;display:inline-flex;gap:6px;align-items:center;cursor:pointer;max-width:260px;overflow:hidden;white-space:nowrap}
.workspace-chip:hover{color:var(--ui-text-primary)}
.chat-scroll{flex:1;min-height:0;overflow-y:auto;overflow-x:hidden;scrollbar-width:thin;scrollbar-color:var(--ui-scrollbar) transparent;overscroll-behavior:contain}
.chat-column{width:100%;max-width:820px;margin:0 auto;padding:22px 24px 12px;display:grid;gap:14px;box-sizing:border-box;min-width:0;overflow-x:hidden}
.state-line{text-align:center;padding:40px 0}
.info-card,.welcome{text-align:center;padding:48px 16px;display:grid;gap:10px;justify-items:center}
.info-card{background:var(--ui-surface);border:1px solid var(--ui-border);border-radius:16px;margin-top:40px}
.info-card p,.welcome p{color:var(--ui-text-secondary);max-width:520px;line-height:1.8;margin:0;font-size:14px}
.welcome h3{margin:4px 0 0;font-size:20px}
.examples{display:grid;gap:8px;margin-top:14px;width:min(520px,100%)}
.examples button{text-align:left;border:1px solid var(--ui-border);background:var(--ui-surface);color:var(--ui-text-primary);border-radius:12px;padding:11px 14px;cursor:pointer;font:inherit;font-size:13.5px}
.examples button:hover{border-color:var(--ui-primary);background:var(--ui-primary-soft)}
.example-tag{font-size:11px;padding:1px 6px;border-radius:4px;background:var(--ui-surface-alt);color:var(--ui-text-tertiary);margin-right:8px;font-weight:normal}
.more-guides-link{display:inline-flex;align-items:center;gap:6px;border:0;background:none;color:var(--ui-primary);font-size:13px;font-weight:600;cursor:pointer;margin-top:6px;padding:5px 12px;border-radius:8px;transition:background .15s}
.more-guides-link:hover{background:var(--ui-primary-soft)}
.msg.user{display:flex;justify-content:flex-end;width:100%;max-width:100%;min-width:0;box-sizing:border-box}
.bubble{background:var(--ui-primary-soft);color:var(--ui-text-primary);border-radius:16px 16px 4px 16px;padding:10px 14px;max-width:min(80%,100%);white-space:pre-wrap;word-break:break-word;overflow-wrap:anywhere;line-height:1.7;font-size:14.5px;min-width:0;box-sizing:border-box}
.msg.assistant{display:grid;grid-template-columns:28px minmax(0,1fr);gap:12px;align-items:start;width:100%;max-width:100%;min-width:0;box-sizing:border-box}
.avatar{border-radius:8px;margin-top:2px}
.answer{width:100%;max-width:100%;min-width:0;font-size:14.5px;line-height:1.8;color:var(--ui-text-primary);overflow-wrap:anywhere;word-break:break-word;box-sizing:border-box}
.cursor{display:inline-block;width:7px;height:15px;background:var(--ui-primary);border-radius:2px;margin-left:2px;vertical-align:text-bottom;animation:blink 1s steps(2,start) infinite}
@keyframes blink{to{visibility:hidden}}
.tool{margin-left:40px;border:1px solid var(--ui-line);border-radius:10px;background:var(--ui-surface);font-size:13px;max-width:100%;min-width:0;box-sizing:border-box}
.tool summary{cursor:pointer;padding:7px 12px;display:flex;gap:8px;align-items:center;color:var(--ui-text-secondary);list-style:none;min-width:0}
.tool summary::-webkit-details-marker{display:none}
.tool summary b{color:var(--ui-text-primary);font-weight:600;white-space:nowrap;flex-shrink:0}
.tool-sub{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;min-width:0;font-family:ui-monospace,Menlo,Consolas,monospace;font-size:12px}
.tool.error summary svg{color:var(--ui-primary)}
.tool.done summary svg{color:#1fb871}
.tool-body{padding:2px 12px 10px;display:grid;gap:8px}
.tool-part small{color:var(--ui-text-tertiary)}
.tool-part pre,.preview{margin:4px 0 0;padding:9px 11px;background:var(--ui-surface-alt);border-radius:8px;overflow:auto;max-height:260px;font-size:12px;line-height:1.6;white-space:pre-wrap;overflow-wrap:anywhere;word-break:break-word;font-family:ui-monospace,Menlo,Consolas,monospace}
.notice{margin-left:40px;font-size:13px;padding:8px 12px;border-radius:10px;background:var(--ui-surface-alt);color:var(--ui-text-secondary);display:flex;gap:12px;align-items:center}
.notice.error{background:var(--ui-primary-soft);color:var(--ui-primary-pressed)}
.notice.warning{background:rgba(245,158,11,.14)}
.card{margin-left:40px;border:1px solid var(--ui-border);border-radius:14px;background:var(--ui-surface);padding:14px 16px;display:grid;gap:10px;box-shadow:var(--ui-shadow-pop);max-width:100%;min-width:0;box-sizing:border-box}
.card.exec{border-color:var(--ui-primary)}
.card header{display:flex;gap:8px;align-items:center;font-size:14px}
.card header svg{color:var(--ui-primary);flex:none}
.card footer{display:flex;gap:8px;align-items:center;flex-wrap:wrap}
.caution{margin:0;font-size:12.5px;color:var(--ui-text-secondary);background:rgba(245,158,11,.14);border-radius:8px;padding:7px 10px;line-height:1.6}
.danger-note{background:rgba(239,68,68,.12);color:var(--ui-text-primary)}
.approval.dangerous{outline:1.5px solid rgba(239,68,68,.55)}
.options{display:flex;gap:8px;flex-wrap:wrap}
.status-line{margin:0 0 0 40px;font-size:13px;color:var(--ui-text-tertiary);display:flex;gap:8px;align-items:center}
.spin{animation:spin 1s linear infinite}
@keyframes spin{to{transform:rotate(360deg)}}
.to-bottom{position:absolute;right:28px;bottom:96px;width:34px;height:34px;border-radius:50%;border:1px solid var(--ui-border);background:var(--ui-surface);color:var(--ui-text-primary);cursor:pointer;display:grid;place-items:center;box-shadow:var(--ui-shadow-pop)}
.composer{padding:10px 24px 18px;width:100%;box-sizing:border-box}
.composer-box{max-width:820px;margin:0 auto;display:flex;flex-direction:column;gap:8px;background:var(--ui-surface);border:1px solid var(--ui-border);border-radius:18px;padding:8px 10px 8px 14px}
.composer-box.voice-active{border-color:var(--ui-primary);box-shadow:0 0 0 2px var(--ui-primary-soft)}
.composer-row{display:flex;gap:10px;align-items:flex-end;min-width:0}
.composer-row :deep(.n-input){flex:1;min-width:0}
.voice-finish{display:inline-flex;align-items:center;justify-content:center;gap:7px;align-self:flex-end;min-height:28px;flex:none;padding:3px 10px;border:1px solid var(--ui-primary);border-radius:999px;background:var(--ui-primary-soft);color:var(--ui-primary);font:inherit;font-size:12px;font-weight:600;cursor:pointer;white-space:nowrap}
.voice-finish:hover{background:var(--ui-primary);color:#fff}
.voice-finish:disabled{cursor:wait;opacity:.75}
.voice-finish:focus-visible{outline:2px solid var(--ui-primary);outline-offset:2px}
.recording-dot{width:7px;height:7px;border-radius:50%;background:currentColor;animation:recording-pulse 1.3s ease-in-out infinite}
@keyframes recording-pulse{50%{opacity:.3}}
.attachments{display:flex;flex-wrap:wrap;align-items:center;gap:8px;padding:2px 2px 0}
.thumb{position:relative;width:56px;height:56px;border-radius:10px;overflow:hidden;border:1px solid var(--ui-line);background:var(--ui-surface-alt);flex:none}
.thumb img{width:100%;height:100%;object-fit:cover;cursor:zoom-in;display:block}
.thumb-remove{position:absolute;top:2px;right:2px;width:17px;height:17px;border-radius:50%;border:0;background:rgba(0,0,0,.58);color:#fff;display:grid;place-items:center;cursor:pointer;padding:0}
.thumb-remove:hover:not(:disabled){background:rgba(0,0,0,.78)}
.thumb-remove:disabled{cursor:not-allowed;opacity:.5}
.attach-hint{color:var(--ui-text-tertiary);font-size:12px}
.bubble-images{display:flex;flex-wrap:wrap;gap:6px;margin-bottom:6px}
.bubble-images img{max-width:180px;max-height:180px;border-radius:10px;border:1px solid var(--ui-line);cursor:zoom-in;display:block}
.lightbox{position:fixed;inset:0;z-index:60;background:rgba(0,0,0,.78);display:grid;place-items:center;padding:40px;cursor:zoom-out}
.lightbox img{max-width:100%;max-height:100%;border-radius:12px;box-shadow:0 18px 60px rgba(0,0,0,.5)}
.lightbox-close{position:absolute;top:18px;right:18px;width:36px;height:36px;border-radius:50%;border:0;background:rgba(255,255,255,.16);color:#fff;display:grid;place-items:center;cursor:pointer}
.lightbox-close:hover{background:rgba(255,255,255,.3)}
.composer-box :deep(.n-input){--n-border:none;--n-border-hover:none;--n-border-focus:none;--n-box-shadow-focus:none;background:transparent}
.composer-box :deep(.n-input .n-input__border),.composer-box :deep(.n-input .n-input__state-border){display:none}
.composer-box :deep(.n-input-wrapper){padding-left:0}
.chat-page :deep(.markdown){width:100%;max-width:100%;min-width:0;word-break:break-word;overflow-wrap:anywhere;box-sizing:border-box}
.chat-page :deep(.markdown>*:first-child){margin-top:0}
.chat-page :deep(.markdown>*:last-child){margin-bottom:0}
.chat-page :deep(.markdown p),.chat-page :deep(.markdown li){margin:0 0 .75em;word-break:break-word;overflow-wrap:anywhere}
.chat-page :deep(.markdown .md-heading){margin:1.1em 0 .5em;line-height:1.4;word-break:break-word;overflow-wrap:anywhere}
.chat-page :deep(.markdown h3){font-size:1.25em}.chat-page :deep(.markdown h4){font-size:1.12em}.chat-page :deep(.markdown h5),.chat-page :deep(.markdown h6){font-size:1em}
.chat-page :deep(.markdown ul),.chat-page :deep(.markdown ol){margin:0 0 .75em;padding-left:1.5em;max-width:100%;box-sizing:border-box}
.chat-page :deep(.markdown li>p){margin:.3em 0}
.chat-page :deep(.markdown blockquote){margin:0 0 .75em;padding:2px 14px;border-left:3px solid var(--ui-border);color:var(--ui-text-secondary);word-break:break-word;overflow-wrap:anywhere}
.chat-page :deep(.markdown hr){border:0;border-top:1px solid var(--ui-line);margin:1em 0}
.chat-page :deep(.markdown .md-code){background:var(--ui-surface-alt);border-radius:5px;padding:1px 6px;font-size:.88em;font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;word-break:break-word;overflow-wrap:anywhere}
.chat-page :deep(.markdown .md-link){color:var(--ui-primary);cursor:pointer;text-decoration:underline;text-underline-offset:2px;overflow-wrap:anywhere;word-break:break-word}
.chat-page :deep(.markdown .md-table){overflow:auto;margin:0 0 .75em;max-width:100%}
.chat-page :deep(.markdown table){border-collapse:collapse;font-size:13.5px}
.chat-page :deep(.markdown th),.chat-page :deep(.markdown td){border:1px solid var(--ui-line);padding:6px 12px;text-align:left}
.chat-page :deep(.markdown th){background:var(--ui-surface-alt);font-weight:600}
@media(max-width:900px){
  .chat-page{grid-template-columns:190px minmax(0,1fr)}
  .chat-column{padding:16px 14px 12px}
  .bubble{max-width:88%}
  .tool,.notice,.card,.status-line{margin-left:0}
}
@media(max-width:680px){
  .chat-page{grid-template-columns:1fr}
  .chat-side{display:none}
  .bubble{max-width:92%}
}
@media(prefers-reduced-motion:reduce){.spin,.cursor,.recording-dot{animation:none}}
</style>


