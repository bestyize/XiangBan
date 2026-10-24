<script setup lang="ts">
import { ATTACHMENT_ACCEPT, MAX_TOTAL_BYTES, attachmentBytes, attachmentImage, attachmentName, readAttachment } from '../ui/chat/attachments.ts';
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { ArrowUp, ArrowUpRight, AudioLines, Check, ChevronDown, Copy, Mic, MicOff, Minus, Pin, Plus, RotateCcw, Settings, SquareDashed, SquarePen, Star, TriangleAlert, X, History } from 'lucide-vue-next';
import MarkdownView from '../ui/chat/MarkdownView.ts';
import SelectionCopyMenu from '../ui/chat/SelectionCopyMenu.vue';
import type { AsrStore } from '../ui/asr/store.ts';
import type { ApprovalRequest, ChatItem } from '../ui/chat/types.ts';
import { invoke } from '../bridge.ts';
import { ballDisc, BALL_DIAMETER, boxOf, glassFrame, HoverIntent, MORPH_DURATION, MORPH_EASING, orbFrame, type Box } from './morph.ts';
import { isDark } from '../ui/appearance.ts';
import Orb from './Orb.vue';
import { hitRegion, type HitRegion } from './hit-regions.ts';
import type { AssistantStore, StageLayout } from './assistant-store.ts';

/* ── Ball ⇄ capsule ──────────────────────────────────────────────────
 * Both live in one window that stays put. The glass grows out of the ball into the capsule while a copy of the mark
 * travels to the capsule's voice button; going back plays the same in reverse. Asked to go the other way halfway,
 * the running animations simply turn around. */
type MorphKind = 'expand' | 'collapse';
const morph = ref<MorphKind | null>(null);
const morphing = computed(() => morph.value !== null);
const glass = ref<HTMLElement>(), travellingOrb = ref<HTMLElement>();
let morphAnimations: Animation[] = [], morphToken = 0;
const reducedMotion = () => window.matchMedia?.('(prefers-reduced-motion: reduce)').matches ?? false;
function capsuleBoxes(): { capsule: Box; orb: Box } | null {
  const composer = document.querySelector<HTMLElement>('.composer'), orb = composer?.querySelector<HTMLElement>('.orb-button > :first-child');
  if (!composer || !orb) return null;
  return { capsule: boxOf(composer.getBoundingClientRect()), orb: boxOf(orb.getBoundingClientRect()) };
}
function stopMorph() {
  morphToken++;
  morphAnimations.splice(0).forEach(animation => animation.cancel());
  morph.value = null;
}
async function runMorph(kind: MorphKind, frames: { glass: Keyframe[]; orb: Keyframe[]; card?: Keyframe[] }) {
  const token = ++morphToken;
  morphAnimations.splice(0).forEach(animation => animation.cancel());
  morph.value = kind;
  await nextTick();
  if (token !== morphToken) return;
  const timing: KeyframeAnimationOptions = { duration: MORPH_DURATION, easing: MORPH_EASING, fill: 'both' };
  const card = document.querySelector('.card');
  morphAnimations = [
    glass.value?.animate?.(frames.glass, timing),
    travellingOrb.value?.animate?.(frames.orb, timing),
    frames.card ? card?.animate?.(frames.card, { duration: 150, easing: 'ease-out', fill: 'both' }) : undefined,
  ].filter((animation): animation is Animation => Boolean(animation));
  await settleMorph(token);
}
/**
 * Waits for the running animations, then shows the real thing. A WebView may pause animations while its window is
 * covered, so the real thing shows in any case a little after the animation should have ended.
 */
async function settleMorph(token: number) {
  const limit = new Promise<void>(resolve => setTimeout(resolve, MORPH_DURATION + 250));
  try { await Promise.race([Promise.all(morphAnimations.map(animation => animation.finished)), limit]); } catch { return; }
  if (token !== morphToken) return;
  const finished = morph.value;
  // The real surfaces swap in (or, collapsing, leave) in the same render that removes the stand-ins. Only then are
  // the animations let go: cancelling first would put a faded card back on screen for a frame.
  const done = morphAnimations.splice(0);
  morph.value = null;
  void nextTick(() => {
    done.forEach(animation => animation.cancel());
    queueHitRegions();
    if (finished === 'expand') {
      // The glass was the capsule's surface all along; only what sits on it fades in.
      document.querySelectorAll('.capsule > :not(.orb-button), .chips').forEach(el => el.animate?.([{ opacity: 0 }, { opacity: 1 }], { duration: 140, easing: 'ease-out' }));
      document.querySelector('.card')?.animate?.([{ opacity: 0, transform: 'translateY(6px)' }, { opacity: 1, transform: 'none' }], { duration: 160, easing: 'ease-out' });
      if (!props.store.peek) field.value?.focus();
    }
  });
}
/** Runs the other way from wherever the current animations are. */
function turnAround(kind: MorphKind) {
  if (morph.value === kind || !morphAnimations.length) return false;
  morph.value = kind;
  morphAnimations.forEach(animation => animation.reverse());
  void settleMorph(++morphToken);
  return true;
}
function expandFromBall(ball: NonNullable<StageLayout['ball']>) {
  if (turnAround('expand')) return;
  const disc = ballDisc(ball);
  morph.value = 'expand';
  void nextTick(() => {
    const target = capsuleBoxes();
    if (!target || morph.value !== 'expand') { stopMorph(); return; }
    void runMorph('expand', {
      glass: [glassFrame(disc, BALL_DIAMETER / 2, 0), glassFrame(target.capsule, 26, 1)],
      orb: [orbFrame(disc), orbFrame(target.orb)],
    });
  });
}
/** Called before the capsule leaves the page, while it can still be measured. */
function collapseToBall(ball: NonNullable<StageLayout['ball']>) {
  if (turnAround('collapse')) return;
  const from = capsuleBoxes();
  if (!from) { stopMorph(); return; }
  const disc = ballDisc(ball);
  void runMorph('collapse', {
    glass: [glassFrame(from.capsule, 26, 1), glassFrame(disc, BALL_DIAMETER / 2, 0)],
    orb: [orbFrame(from.orb), orbFrame(disc)],
    card: [{ opacity: 1, transform: 'none' }, { opacity: 0, transform: 'translateY(6px)' }],
  });
}

/* ── Moving the window under a showing ball (after a drag, or to a new screen) ── */
function showMovedBall() {
  movingBall.value = false; queueHitRegions();
  clearTimeout(fadeEnd); fadeEnd = setTimeout(() => { ballFade.value = false; }, 150);
}
const movingBall = ref(false);
/** The ball fades only around a window move; after a morph it must appear at once, the stand-ins being gone. */
const ballFade = ref(false);
let fadeEnd: ReturnType<typeof setTimeout> | undefined;
let movingFallback: ReturnType<typeof setTimeout> | undefined, movingGiveUp: ReturnType<typeof setTimeout> | undefined;

const props = defineProps<{ store: AssistantStore; asr: AsrStore }>();
const orbLabel = computed(() => {
  if (props.store.phase === 'speaking') return '停止朗读';
  if (props.asr.listening) return '结束录音';
  if (props.asr.recordingRequested) return '取消语音启动';
  if (props.asr.preparing) return '正在准备语音输入';
  if (props.asr.microphoneSupported === false) return '当前设备不支持语音输入';
  if (props.asr.enabled === false) return '开启语音输入';
  if (props.asr.microphoneError) return '重试语音输入';
  return '点击开始录音';
});
const field = ref<HTMLTextAreaElement>();
const filePicker = ref<HTMLInputElement>();
const imageReading = ref(false), submitting = ref(false), dropActive = ref(false), capturing = ref(false), picking = ref(false);
/** Pinned: a click elsewhere does not fold the capsule away. */
const pinned = ref(false);
const sessionsOpen = ref(false), sessionsHost = ref<HTMLElement>(), sessionsPanel = ref<HTMLElement>();
function closeMenus() { sessionsOpen.value = false; }
function outsideMenus(event: PointerEvent) {
  const target = event.target as Node;
  if (sessionsOpen.value && !sessionsHost.value?.contains(target) && !sessionsPanel.value?.contains(target)) closeMenus();
}
function escape() {
  if (sessionsOpen.value) { closeMenus(); return; }
  if (capturing.value) return;
  if (props.store.live) { props.store.endLive(); return; }
  if (props.store.input) { props.store.input = ''; return; }
  void props.store.hide();
}
const awaitingCapturePermission = ref(false);
const CAPTURE_PERMISSION_WAIT = '已打开系统截图授权设置。允许小伴录制屏幕后返回，将自动继续截图。';
let capturePermissionRetryUsed = false, capturePermissionVersion = 0;
async function takeScreenshot(permissionRetry = false) {
  if (!canAttach.value || props.store.attachments.length >= 8) return;
  if (!permissionRetry) { capturePermissionRetryUsed = false; awaitingCapturePermission.value = false; }
  if (props.store.hint === CAPTURE_PERMISSION_WAIT) props.store.note('');
  props.store.prepareImages();
  const version = imageVersion;
  capturing.value = true;
  try {
    await nextTick();
    const image = await invoke<string | null>('assistant_screenshot');
    if (image && props.store.open && version === imageVersion) { props.store.attachments.push(image); }
  } catch (error) {
    if (version === imageVersion) {
      const message = error instanceof Error ? error.message : String(error);
      const permissionPrefix = 'SCREEN_CAPTURE_PERMISSION_REQUIRED:';
      if (message.startsWith(permissionPrefix)) {
        if (!capturePermissionRetryUsed && props.store.open) {
          awaitingCapturePermission.value = true;
          capturePermissionVersion = version;
          try {
            await invoke('open_screen_capture_settings');
            if (version === imageVersion && props.store.open) {
              props.store.note(CAPTURE_PERMISSION_WAIT, 'assistant');
            }
          } catch (permissionError) {
            if (version === imageVersion && props.store.open) {
              awaitingCapturePermission.value = false;
              props.store.note(`无法打开系统授权设置：${String(permissionError)}。请在系统设置的隐私与安全性中允许小伴录制屏幕。`, 'assistant');
            }
          }
        } else {
          props.store.note('截图授权尚未生效。请完成屏幕录制授权；若已允许，请完全退出小伴后重新打开。', 'assistant');
        }
        return;
      }
      // A failed capture is not always a permission failure. Show the repair link only
      // for macOS permission or screencapture failures; never reset automatically.
      const permissionIssue = /macOS 截图失败|屏幕录制权限|截图权限|could not create image from rect|could not create image from display/i.test(message);
      props.store.note(message, permissionIssue ? 'assistant' : '');
    }
  }
  finally { capturing.value = false; void nextTick(() => field.value?.focus()); }
}
let imageVersion = 0, dropDepth = 0;

const canAttach = computed(() => !busy.value && !props.store.ask && !props.store.approval && !imageReading.value && !submitting.value && !capturing.value && !props.store.changingSession);
watch(() => props.store.open, () => { imageVersion++; awaitingCapturePermission.value = false; dropDepth = 0; dropActive.value = false; closeMenus(); });
watch(() => props.store.currentId, () => { imageVersion++; awaitingCapturePermission.value = false; });
watch(() => props.store.changingSession, changing => { if (changing) imageVersion++; });
function openPicker() {
  if (!canAttach.value) return;
  props.store.prepareImages(); picking.value = true; filePicker.value?.click();
}
async function addImages(files: File[]) {
  if (!files.length || !props.store.open) return;
  if (!canAttach.value) { props.store.note('请等待当前操作完成后再选择文件。'); return; }
  props.store.prepareImages();
  const version = imageVersion;
  imageReading.value = true;
  let warning = '';
  try {
    for (const file of files) {
      if (version !== imageVersion) break;
      if (props.store.attachments.length >= 8) { warning = '一次最多添加 8 个附件。'; break; }
      try {
        const image = await readAttachment(file);
        if (props.store.attachments.reduce((sum, a) => sum + attachmentBytes(a), 0) + attachmentBytes(image) > MAX_TOTAL_BYTES) { warning = '附件合计不能超过 30MB'; continue; }
        if (version === imageVersion) props.store.attachments.push(image);
      } catch (error) { warning = (error as Error).message; }
    }
    if (warning && version === imageVersion) props.store.note(warning);
  } finally { imageReading.value = false; }
}
function onPick(event: Event) {
  const input = event.target as HTMLInputElement;
  picking.value = false;
  void addImages(Array.from(input.files ?? [])); input.value = '';
}
function hasFiles(event: DragEvent) { return Array.from(event.dataTransfer?.types ?? []).includes('Files'); }
function dragEnter(event: DragEvent) { if (hasFiles(event)) { event.preventDefault(); dropDepth++; dropActive.value = canAttach.value; } }
function dragOver(event: DragEvent) { if (hasFiles(event)) { event.preventDefault(); if (event.dataTransfer) event.dataTransfer.dropEffect = canAttach.value ? 'copy' : 'none'; } }
function dragLeave() { dropDepth = Math.max(0, dropDepth - 1); if (!dropDepth) dropActive.value = false; }
function dropImages(event: DragEvent) {
  event.preventDefault(); dropDepth = 0; dropActive.value = false;
  void addImages(Array.from(event.dataTransfer?.files ?? []));
}
function pasteImages(event: ClipboardEvent) {
  const files = Array.from(event.clipboardData?.files ?? []);
  if (files.length) { event.preventDefault(); void addImages(files); }
}
const inner = ref<HTMLElement>();
const scroller = ref<HTMLElement>();
const composing = ref(false);
const dragging = ref(false);

const busy = computed(() => ['thinking', 'answering', 'speaking'].includes(props.store.phase));

/** What the card shows of the open conversation: questions, answers and errors, in order. */
type Turn = Extract<ChatItem, { kind: 'user' | 'assistant' | 'notice' }>;
const turns = computed(() => props.store.items.filter((item): item is Turn =>
  item.kind === 'user' || (item.kind === 'assistant' && Boolean(item.text.trim())) || (item.kind === 'notice' && item.tone !== 'info')));
const lastAnswer = computed(() => { for (let i = turns.value.length - 1; i >= 0; i--) { const t = turns.value[i]; if (t.kind === 'user') return null; if (t.kind === 'assistant') return t; } return null; });

/** The conversation card. Opening the capsule shows only the capsule unless an answer waits unread or something needs the user. */
const cardShown = ref(false);
let unreadWhenFolded = false;
watch(() => props.store.unread, unread => { if (unread) unreadWhenFolded = true; }, { immediate: true });
const needsUser = computed(() => Boolean(props.store.approval || props.store.ask || props.store.hint));
function revealCard() { cardShown.value = true; }
watch(() => props.store.open, open => {
  if (!open) return;
  cardShown.value = unreadWhenFolded || busy.value || needsUser.value;
  unreadWhenFolded = false;
}, { immediate: true });
watch(needsUser, need => { if (need) cardShown.value = true; });
watch(() => props.store.phase, phase => { if (phase === 'thinking') cardShown.value = true; });
const hasCard = computed(() => cardShown.value && (turns.value.length > 0 || needsUser.value || props.store.phase === 'thinking' || Boolean(props.store.working)));
/** While an answer streams the card body keeps one height, so the text scrolls instead of resizing the window. */
const fixedBody = computed(() => turns.value.length > 0 && busy.value);

/** Quick actions above a capsule that has no card. */
const continueTitle = computed(() => (turns.value.length ? props.store.title || '上次的对话' : ''));
const otherRecent = computed(() => props.store.recent.find(session => session.id !== props.store.currentId && session.title));
const chipsShown = computed(() => props.store.open && !hasCard.value && !busy.value && !props.store.input && !props.store.attachments.length && Boolean(continueTitle.value || otherRecent.value));
async function continueOther(id: string) { await props.store.switchSession(id); cardShown.value = true; void nextTick(() => field.value?.focus()); }

const canSend = computed(() => (!busy.value || props.store.phase === 'speaking') && !props.store.approval && !imageReading.value && !submitting.value && !capturing.value && !props.store.changingSession && Boolean(props.store.input.trim() || props.store.attachments.length));
const stopping = computed(() => busy.value && !canSend.value);
/** 实时对话 sits beside send while voice is available; it waits while an answer is being written. */
const offersLive = computed(() => props.store.voiceAvailable);
const liveDisabled = computed(() => busy.value || imageReading.value || capturing.value || submitting.value || props.store.changingSession);

/* ── 实时对话 ─────────────────────────────────────────────────────── */
const liveNow = ref(Date.now());
let liveClockTimer: ReturnType<typeof setInterval> | undefined;
watch(() => props.store.live, live => {
  clearInterval(liveClockTimer); liveClockTimer = undefined;
  if (live) { liveNow.value = Date.now(); liveClockTimer = setInterval(() => { liveNow.value = Date.now(); }, 1000); }
  void nextTick(queueHitRegions);
}, { immediate: true });
onBeforeUnmount(() => clearInterval(liveClockTimer));
const liveClock = computed(() => {
  const seconds = props.store.liveSince ? Math.max(0, Math.floor((liveNow.value - props.store.liveSince) / 1000)) : 0;
  return `${String(Math.floor(seconds / 60)).padStart(2, '0')}:${String(seconds % 60).padStart(2, '0')}`;
});
const liveHint = computed(() => {
  const status = props.store.liveStatus;
  if (status === '已静音') return '小伴听不到你，点麦克风恢复';
  if (status === '在说') return '直接开口就能打断它';
  if (status === '在想') return props.store.working ? `${props.store.working}…` : '马上回答';
  if (status === '等你确认') return '在上面的卡片里确认后继续';
  if (status === '正在连接…') return props.store.voiceStatus || '第一次使用需要允许麦克风';
  const heard = props.store.voiceStatus || props.store.input;
  return heard || '说完停一下就会回答';
});
async function startLive() { closeMenus(); await act(() => props.store.startLive()); }

function approvalLine(request: ApprovalRequest) {
  const raw = typeof request.arguments === 'string' ? request.arguments : JSON.stringify(request.arguments);
  const text = `${request.tool} ${raw ?? ''}`.trim();
  return text.length > 180 ? `${text.slice(0, 180)}…` : text;
}
async function send() {
  const text = props.store.input.trim();
  if (!canSend.value) return;
  if (props.store.ask) { if (text) { props.store.input = ''; void props.store.reply(text); } return; }
  if ((busy.value && props.store.phase !== 'speaking') || props.store.approval) return;
  cardShown.value = true; submitting.value = true;
  try { await props.store.submit(); } finally { submitting.value = false; }
}
// Number shortcuts apply only to the active Ask User question, never to text inputs.
function onAskShortcut(event: KeyboardEvent) {
  if (!props.store.open || !props.store.ask || event.defaultPrevented || event.repeat || event.isComposing || event.ctrlKey || event.altKey || event.metaKey) return;
  if (event.target instanceof Element && event.target.closest('input, textarea, select, [contenteditable]')) return;
  if (!/^[1-9]$/.test(event.key)) return;
  const option = props.store.ask.options[Number(event.key) - 1];
  if (option === undefined) return;
  event.preventDefault();
  void props.store.reply(option);
}
function onEnter(event: KeyboardEvent) {
  if (event.isComposing || composing.value || event.shiftKey) return;
  event.preventDefault();
  void send();
}
function primary() {
  if (canSend.value) void send();
  else if (busy.value) props.store.interrupt();
}
function onOrb() {
  if (props.store.phase === 'speaking') props.store.interrupt();
  else void props.store.toggleListening();
}
async function openLink(href: string) {
  try { await invoke('open_download_url', { url: href.split('#')[0] }); } catch { props.store.note('这个地址不能直接打开。'); }
}
async function act(action: () => Promise<void> | void) {
  closeMenus();
  try { await action(); } catch (error) { props.store.note(error instanceof Error ? error.message : String(error)); }
}
async function newChat() { await act(() => props.store.newConversation()); cardShown.value = false; void nextTick(() => field.value?.focus()); }
async function pickSession(id: string) { await act(() => props.store.switchSession(id)); cardShown.value = true; }

/**
 * The text box grows with its text, up to four lines, then scrolls. Empty, it is one line whatever its width:
 * measured while the window is still narrow, the placeholder would wrap and leave the box tall.
 */
function grow() {
  const el = field.value;
  if (!el) return;
  if (!el.value) { el.style.height = ''; return; }
  el.style.height = 'auto';
  el.style.height = `${Math.min(el.scrollHeight, 96)}px`;
}
watch(() => props.store.input, () => void nextTick(grow), { immediate: true });

/* ── The ball and the line beside it ─────────────────────────────── */
const ballBadge = computed(() => {
  if (props.store.phase === 'attention') return 'attention';
  if (props.store.notice?.kind === 'error') return 'error';
  if (props.store.unread) return 'unread';
  return '';
});
const ballRing = computed(() => ['thinking', 'answering'].includes(props.store.phase));
const ballLabel = computed(() => ({ attention: '超级小伴 · 等你确认', error: '超级小伴 · 出错了', unread: '超级小伴 · 有新回答' } as Record<string, string>)[ballBadge.value]
  ?? (ballRing.value ? '超级小伴 · 正在回答' : '打开超级小伴'));
/** The notice sits on the side of the ball that faces the middle of the screen; null while none shows. */
const bubbleOn = ref(false);
const bubbleLeft = computed(() => bubbleOn.value ? props.store.layout.right : null);
let bubbleTimer: ReturnType<typeof setTimeout> | undefined;
const bubbleWanted = computed(() => Boolean(props.store.ball && !props.store.open && props.store.notice && !dragging.value && !morphing.value && !movingBall.value));
watch(bubbleWanted, wanted => {
  clearTimeout(bubbleTimer);
  bubbleOn.value = wanted;
  // A finished answer says so briefly, then the unread dot is enough. Waiting and errors stay until seen.
  if (wanted && props.store.notice?.kind === 'done') bubbleTimer = setTimeout(() => props.store.dismissNotice(), 3000);
  void nextTick(queueHitRegions);
}, { immediate: true });
watch(() => props.store.notice?.text, () => {
  if (bubbleLeft.value === null || props.store.notice?.kind !== 'done') return;
  clearTimeout(bubbleTimer); bubbleTimer = setTimeout(() => props.store.dismissNotice(), 3000);
});

/* ── Click elsewhere folds the capsule away, unless something would be lost or is waiting ─── */
const canAutoHide = computed(() => props.store.open && !pinned.value && !props.store.live && props.store.phase === 'idle' && !props.store.input.trim()
  && !props.store.attachments.length && !capturing.value && !awaitingCapturePermission.value && !picking.value && !imageReading.value && !submitting.value
  && !dragging.value && !morphing.value && !props.store.changingSession && !props.asr.recordingRequested);
let blurTimer: ReturnType<typeof setTimeout> | undefined;
function onBlur() {
  closeMenus();
  clearTimeout(blurTimer);
  blurTimer = setTimeout(() => { if (!document.hasFocus() && canAutoHide.value) void props.store.hide(); }, 160);
}

let hitRaf=0,lastRegions='';
function syncHitRegions(){
  const regions:HitRegion[]=[];
  // Mid-morph the stand-ins are what shows; the real surfaces take over when it settles.
  if(morphing.value)return;
  document.querySelectorAll<HTMLElement>('.composer,.card,.ball,.bubble,.chip,.session-menu,.selection-copy-menu').forEach(el=>{
    const radius=el.classList.contains('composer')?26:el.classList.contains('ball')?22:el.classList.contains('card')?18:el.classList.contains('bubble')||el.classList.contains('chip')?18:el.classList.contains('session-menu')?12:9;
    const region=hitRegion(el.getBoundingClientRect(),window.innerHeight,radius);
    if(region)regions.push(region);
  });
  const signature=JSON.stringify(regions);if(signature===lastRegions)return;lastRegions=signature;
  void invoke('assistant_set_regions',{regions}).catch(()=>{lastRegions='';});
}
function queueHitRegions(){cancelAnimationFrame(hitRaf);hitRaf=requestAnimationFrame(syncHitRegions);}
watch([sessionsOpen,()=>props.store.open,()=>props.store.ball,hasCard,chipsShown,bubbleLeft],()=>void nextTick(queueHitRegions));

// The window keeps its size; only the parts that take the mouse change with the content.
let observer: ResizeObserver | undefined;
watch(inner, el => {
  observer?.disconnect();
  if (el) { observer = new ResizeObserver(queueHitRegions); observer.observe(el); queueHitRegions(); }
});
function toBottom() { void nextTick(() => { if (scroller.value) scroller.value.scrollTop = scroller.value.scrollHeight; }); }
watch(() => props.store.open, open => {
  if (!open) return;
  // A glance on hover leaves the caret alone; the morph focuses the text box when it settles.
  void nextTick(() => { grow(); if (!morphing.value && !props.store.peek) field.value?.focus(); toBottom(); });
});
// Before the DOM changes, so a closing capsule can still be measured.
watch(() => props.store.open, (open, was) => {
  if (open === was) return;
  const ball = props.store.layout.ball;
  if (!ball || reducedMotion()) { stopMorph(); return; }
  if (open) expandFromBall(ball); else collapseToBall(ball);
}, { flush: 'pre' });
watch(() => props.store.reanchoring, id => {
  clearTimeout(movingFallback); clearTimeout(movingGiveUp);
  if (id) {
    // Fade the ball out, then let the host move the window under it. Should the host never say it is done, the
    // ball comes back anyway rather than staying invisible.
    clearTimeout(fadeEnd); ballFade.value = true; movingBall.value = true; hover.cancel();
    movingFallback = setTimeout(() => props.store.reanchorReady(id), 100);
    movingGiveUp = setTimeout(showMovedBall, 1200);
    return;
  }
  // Moved: the page has the new layout; show the ball once it has been drawn there.
  requestAnimationFrame(() => requestAnimationFrame(showMovedBall));
});
watch(hasCard, shown => { if (shown) toBottom(); });
watch(() => [props.store.answer, turns.value.length, props.store.ask?.id, props.store.approval?.id], toBottom);

const copied = ref(''), copyFailed = ref('');
let copyTimer: ReturnType<typeof setTimeout> | undefined;
const savedAnswers = ref(new Set<string>());
async function saveAnswer(id: string, text: string) {
  if (savedAnswers.value.has(id) || !text.trim()) return;
  if (await props.store.saveAnswer(text)) savedAnswers.value = new Set([...savedAnswers.value, id]);
}
async function copyText(id: string, text: string) {
  if (!text.trim()) return;
  clearTimeout(copyTimer);
  try { await navigator.clipboard.writeText(text); copied.value = id; copyFailed.value = ''; }
  catch { copyFailed.value = id; copied.value = ''; }
  finally { copyTimer = setTimeout(() => { copied.value = ''; copyFailed.value = ''; }, 2000); }
}

// Only the orb, the ball and empty chrome are drag handles; text and controls retain native selection.
let press:{id:number;x:number;y:number;capture:Element}|null=null,pulled=false;
function releasePress(){const p=press;press=null;if(p?.capture.hasPointerCapture?.(p.id))p.capture.releasePointerCapture(p.id);}
function dragDown(event:PointerEvent){
  if(event.button!==0)return;pulled=false;
  const target=event.target as Element,handle=target.closest('[data-drag-handle]');
  if(!handle&&!target.closest('.composer,.card'))return;
  if(!handle&&target.closest('button,input,textarea,select,a,pre,code,.card-body,[contenteditable],[data-no-drag]'))return;
  event.preventDefault();const capture=handle??target;
  press={id:event.pointerId,x:event.clientX,y:event.clientY,capture};capture.setPointerCapture?.(event.pointerId);
}
function dragMove(event:PointerEvent){
  if(!press||event.pointerId!==press.id)return;
  if(event.buttons===0){releasePress();return;}
  if(Math.hypot(event.clientX-press.x,event.clientY-press.y)<6)return;
  pulled=true;dragging.value=true;releasePress();hover.cancel();hoverBlocked=true;
  if(bubbleLeft.value!==null)props.store.dismissNotice();
  void invoke('assistant_drag').catch(()=>{dragging.value=false;queueHitRegions();});
}
function dragUp(event:PointerEvent){if(press?.id===event.pointerId)releasePress();}
function captureClick(event:MouseEvent){if(pulled&&event.detail!==0){event.preventDefault();event.stopImmediatePropagation();pulled=false;}}
function dragEnded(){dragging.value=false;releasePress();queueHitRegions();}
function ballClick(){hover.cancel();hoverBlocked=true;void props.store.unfold();}

/* ── Resting the pointer on the ball opens the capsule for a glance ──
 * It does not take the keyboard focus and folds away once the pointer has been gone a moment, unless the user
 * clicks or types in it. Crossing the ball on the way somewhere else does nothing. */
let hoverBlocked = false, peekTimer: ReturnType<typeof setTimeout> | undefined;
const PEEK_LINGER = 600;
const hoverAllowed = () => props.store.prefs.hover && props.store.ball && !props.store.open && Boolean(props.store.layout.ball)
  && !hoverBlocked && !dragging.value && !press && !morphing.value && !movingBall.value && !capturing.value;
const hover = new HoverIntent(() => { if (hoverAllowed()) void props.store.unfold(true); });
function ballEnter(event: PointerEvent) { if (hoverAllowed()) hover.enter(event.clientX, event.clientY); }
function ballMove(event: PointerEvent) { if (hoverAllowed()) hover.move(event.clientX, event.clientY); else hover.cancel(); }
function ballLeave() { hover.cancel(); hoverBlocked = false; }
// Closed with the pointer still on the ball, or dropped after a drag: it has to leave before resting counts again.
watch(() => props.store.open, open => { if (!open) { hoverBlocked = true; clearTimeout(peekTimer); } });
watch(() => props.store.hovering, inside => {
  clearTimeout(peekTimer);
  if (inside) return;
  hover.cancel(); hoverBlocked = false;
  if (props.store.open && props.store.peek) peekTimer = setTimeout(() => {
    if (props.store.open && props.store.peek && !props.store.hovering && !pinned.value && !dragging.value) void props.store.hide();
  }, PEEK_LINGER);
});
function engage() { if (props.store.peek) { clearTimeout(peekTimer); props.store.engage(); } }
const onFocus=()=>{
  picking.value=false;clearTimeout(blurTimer);
  if(props.store.open&&!dragging.value&&!press&&!morphing.value)field.value?.focus();
  if(awaitingCapturePermission.value && !capturing.value && canAttach.value && props.store.open && capturePermissionVersion === imageVersion) {
    awaitingCapturePermission.value = false;
    capturePermissionRetryUsed = true;
    void takeScreenshot(true);
  }
};
onMounted(()=>{
  void nextTick(queueHitRegions);window.addEventListener('resize',queueHitRegions);window.addEventListener('resize',grow);
  document.addEventListener('pointerdown',outsideMenus);document.addEventListener('keydown',onAskShortcut);window.addEventListener('blur',onBlur);window.addEventListener('focus',onFocus);window.addEventListener('pointermove',dragMove);window.addEventListener('pointerup',dragUp);window.addEventListener('pointercancel',dragUp);window.addEventListener('blur',releasePress);window.addEventListener('assistant-drag-end',dragEnded);
});
onBeforeUnmount(()=>{
  stopMorph(); hover.cancel(); clearTimeout(peekTimer); clearTimeout(movingFallback); clearTimeout(movingGiveUp); clearTimeout(fadeEnd);
  cancelAnimationFrame(hitRaf);window.removeEventListener('resize',queueHitRegions);window.removeEventListener('resize',grow);clearTimeout(blurTimer);clearTimeout(bubbleTimer);clearTimeout(copyTimer);
  window.removeEventListener('blur',onBlur);document.removeEventListener('pointerdown',outsideMenus);document.removeEventListener('keydown',onAskShortcut);imageVersion++; releasePress();window.removeEventListener('focus',onFocus);window.removeEventListener('pointermove',dragMove);window.removeEventListener('pointerup',dragUp);window.removeEventListener('pointercancel',dragUp);window.removeEventListener('blur',releasePress);window.removeEventListener('assistant-drag-end',dragEnded);observer?.disconnect();
});
</script>

<template>
  <div class="stage" :class="{ light: !isDark, dragging, [`morph-${morph}`]: morphing, 'moving-ball': movingBall, 'ball-fade': ballFade }" @dragover="dragOver" @drop="dropImages" @keydown.esc.prevent.stop="escape" @pointerdown.capture="engage" @keydown.capture="engage" @pointerdown="dragDown" @click.capture="captureClick">
    <Transition name="ball-in">
    <div v-if="store.ball && !store.open && store.layout.ball" class="ball-stage" :class="{ 'bubble-left': bubbleLeft === true }" :style="{ left: `${store.layout.ball.x}px`, top: `${store.layout.ball.y}px`, width: `${store.layout.ball.size}px`, height: `${store.layout.ball.size}px` }" data-testid="assistant-ball-stage">
      <button type="button" class="ball" @pointerenter="ballEnter" @pointermove="ballMove" @pointerleave="ballLeave" :data-phase="store.phase" :data-badge="ballBadge || undefined" title="点一下打开，按住可以拖动，右键打开设置" :aria-label="ballLabel" data-testid="assistant-ball" data-drag-handle @click="ballClick" @contextmenu.prevent="store.openSettings('assistant')">
        <Orb :phase="store.phase === 'attention' ? 'idle' : store.phase" :active="store.ball && !store.open" :muted="ballBadge === 'error'" :size="44" />
        <svg v-if="ballRing" class="ball-ring" viewBox="0 0 50 50" aria-hidden="true" data-testid="assistant-ball-ring"><circle cx="25" cy="25" r="23" class="ring-track" /><circle cx="25" cy="25" r="23" class="ring-arc" /></svg>
        <span v-if="ballBadge" class="badge" :class="ballBadge" aria-hidden="true" data-testid="assistant-ball-badge"><template v-if="ballBadge === 'attention'">!</template><X v-else-if="ballBadge === 'error'" :size="9" :stroke-width="3.4" /></span>
      </button>
      <Transition name="bubble">
        <button v-if="bubbleLeft !== null && store.notice" type="button" class="bubble" :class="store.notice.kind" data-testid="assistant-bubble" @click="ballClick">
          <Check v-if="store.notice.kind === 'done'" :size="15" :stroke-width="2.2" class="bubble-icon" />
          <TriangleAlert v-else :size="15" :stroke-width="2" class="bubble-icon" />
          <b>{{ store.notice.kind === 'done' ? '已完成' : store.notice.kind === 'attention' ? '需要你处理' : '没有完成' }}</b>
          <span class="bubble-text">{{ store.notice.text }}</span>
          <span class="bubble-go">{{ store.notice.kind === 'done' ? '查看' : '去处理' }}</span>
        </button>
      </Transition>
    </div>
    </Transition>
    <!-- With a ball, the morph's stand-ins do the showing; the capsule must come and go in the same frame as them. -->
    <Transition name="pop" :css="!store.layout.ball" appear @after-enter="queueHitRegions" @after-leave="queueHitRegions">
      <div v-if="store.open || morph === 'collapse'" ref="inner" class="inner" role="dialog" aria-label="超级小伴" data-testid="assistant">
        <Transition name="rise" @after-enter="queueHitRegions" @after-leave="queueHitRegions">
          <section v-if="hasCard" class="card" data-testid="assistant-card">
            <header class="card-head">
              <div ref="sessionsHost" class="session-host" data-no-drag>
                <button type="button" class="session-title" :aria-expanded="sessionsOpen" aria-haspopup="menu" title="切换最近的会话" data-testid="assistant-sessions" @click="sessionsOpen = !sessionsOpen"><span>{{ store.title || '新对话' }}</span><ChevronDown :size="13" :stroke-width="2.2" /></button>
              </div>
              <span class="head-gap" />
              <button type="button" class="head-btn" title="新对话" aria-label="新对话" data-testid="assistant-new-chat" :disabled="imageReading || submitting || capturing || store.changingSession" @click="newChat"><SquarePen :size="16" :stroke-width="1.75" /></button>
              <button type="button" class="head-btn" title="在主窗口继续" aria-label="在主窗口继续" data-testid="assistant-expand" @click="act(() => store.openChat())"><ArrowUpRight :size="16" :stroke-width="1.75" /></button>
              <button type="button" class="head-btn" :class="{ on: pinned }" :title="pinned ? '取消固定' : '固定，点别处也不收起'" :aria-label="pinned ? '取消固定' : '固定'" :aria-pressed="pinned" data-testid="assistant-pin" @click="pinned = !pinned"><Pin :size="16" :stroke-width="1.75" :fill="pinned ? 'currentColor' : 'none'" /></button>
              <button type="button" class="head-btn" title="收成小球 · Esc" aria-label="收成小球" data-testid="assistant-close" @click="act(() => store.hide())"><Minus :size="16" :stroke-width="1.9" /></button>
            </header>
            <!-- The switcher takes the conversation's place inside the card: a dropdown would hide under the capsule and run off the window. -->
            <div v-if="sessionsOpen" ref="sessionsPanel" class="session-menu" data-no-drag role="menu" aria-label="最近会话" data-testid="assistant-session-menu">
              <p class="menu-label">最近会话</p>
              <button v-for="session in store.recent" :key="session.id" type="button" class="menu-item" role="menuitemradio" :aria-checked="session.id === store.currentId" :disabled="store.changingSession || busy" @click="pickSession(session.id)">
                <Check v-if="session.id === store.currentId" :size="14" :stroke-width="2.2" class="menu-check" /><span v-else class="menu-check" />
                <span class="menu-text">{{ session.title || '新对话' }}</span>
              </button>
              <p v-if="!store.recent.length" class="menu-empty">还没有会话</p>
              <div class="menu-sep" />
              <button type="button" class="menu-item" role="menuitem" @click="act(() => store.openChat())"><ArrowUpRight :size="14" :stroke-width="1.8" class="menu-check" />在主窗口查看全部</button>
              <button type="button" class="menu-item" role="menuitem" data-testid="assistant-settings" @click="act(() => store.openSettings('assistant'))"><Settings :size="14" :stroke-width="1.8" class="menu-check" />超级小伴设置…</button>
            </div>
            <div v-show="!sessionsOpen" ref="scroller" class="card-body" :class="{ fixed: fixedBody }" data-testid="assistant-card-body">
              <template v-for="item in turns" :key="item.id">
                <div v-if="item.kind === 'user'" class="you-row">
                  <div v-if="item.files?.length" class="sent-files"><span v-for="(file, n) in item.files" :key="n" class="file-chip" :title="file.name">{{ file.name }}</span></div>
                  <div v-if="item.images.length" class="sent-images"><img v-for="(image, index) in item.images" :key="index" :src="image" :alt="`已发送的图片 ${index + 1}`" draggable="false" /></div>
                  <p v-if="item.text" class="you">{{ item.text }}</p>
                </div>
                <div v-else-if="item.kind === 'assistant'" class="reply">
                  <MarkdownView :source="item.text" @open="openLink" /><span v-if="item.streaming" class="cursor" aria-hidden="true"></span>
                  <div v-if="!item.streaming" class="reply-actions" :class="{ last: item === lastAnswer }">
                    <button type="button" class="reply-btn" :data-testid="item === lastAnswer ? 'assistant-copy' : undefined" :aria-label="copied === item.id ? '已复制回复' : copyFailed === item.id ? '复制失败，点击重试' : '复制回复'" @pointerdown.prevent @click="copyText(item.id, item.text)"><Check v-if="copied === item.id" :size="13" /><Copy v-else :size="13" :stroke-width="1.8" /><span aria-live="polite">{{ copied === item.id ? '已复制' : copyFailed === item.id ? '复制失败，重试' : '复制' }}</span></button>
                    <button type="button" class="reply-btn" :class="{ on: savedAnswers.has(item.id) }" :aria-label="savedAnswers.has(item.id) ? '已存到资料库' : '存到资料库'" :title="savedAnswers.has(item.id) ? '已存到资料库' : '存到资料库'" :data-testid="item === lastAnswer ? 'assistant-save' : undefined" @pointerdown.prevent @click="saveAnswer(item.id, item.text)"><Star :size="13" :stroke-width="1.8" :fill="savedAnswers.has(item.id) ? 'currentColor' : 'none'" /><span>{{ savedAnswers.has(item.id) ? '已收藏' : '存到资料库' }}</span></button>
                    <button v-if="item === lastAnswer && !busy" type="button" class="reply-btn" aria-label="重新生成" title="重新生成" data-testid="assistant-retry" @click="act(() => store.retry())"><RotateCcw :size="13" :stroke-width="1.8" /></button>
                  </div>
                </div>
                <p v-else class="failure" :class="{ warn: item.tone === 'warning' }">{{ item.text }}</p>
              </template>
              <p v-if="store.phase === 'thinking' || store.working" class="working"><span class="dots" aria-hidden="true"><i></i><i></i><i></i></span><template v-if="store.working">{{ store.working }}…</template></p>
              <p v-if="store.hint" class="hint" data-testid="assistant-hint">
                {{ store.hint }}
                <button v-if="store.hintTab" type="button" class="link" @click="store.openSettings(store.hintTab)">{{ store.hintTab === 'ai' ? '去配置' : '去设置' }}</button>
              </p>
              <div v-if="store.approval" class="gate" data-testid="assistant-approval">
                <p class="gate-title"><TriangleAlert :size="15" :stroke-width="2" />{{ store.approval.danger ? `这个操作有风险：${store.approval.danger}` : '这一步需要你许可' }}</p>
                <p class="gate-tool">{{ approvalLine(store.approval) }}</p>
                <div class="gate-actions"><button type="button" class="ghost" @click="store.decide('deny')">拒绝</button><button type="button" class="ghost" @click="store.decide('always')">本次会话都允许</button><button type="button" class="solid" @click="store.decide('once')">允许这一次</button></div>
              </div>
              <div v-if="store.ask" class="ask-user" data-testid="assistant-question" role="group" :aria-label="store.ask.question">
                <p class="ask-user-question">{{ store.ask.question }}<span v-if="store.ask.options.length" class="ask-user-tip">按 <kbd>1–{{ Math.min(9, store.ask.options.length) }}</kbd> 快捷选择</span></p>
                <div v-if="store.ask.options.length" class="ask-user-options" role="group" aria-label="回答选项">
                  <button v-for="(option, index) in store.ask.options" :key="`${index}-${option}`" type="button" class="ask-user-option" @click="store.reply(option)">
                    <span class="ask-user-index" aria-hidden="true">{{ index + 1 }}</span>
                    <span class="ask-user-option-text">{{ option }}</span>
                  </button>
                </div>
                <p v-else class="ask-user-help">可以直接在下方输入你的回答。</p>
                <button type="button" class="ask-user-skip" @click="store.reply(null)">跳过此问题 →</button>
              </div>
            </div>
          </section>
        </Transition>
        <div v-if="chipsShown" class="chips" data-testid="assistant-chips">
          <button v-if="continueTitle" type="button" class="chip" data-testid="assistant-chip-continue" @click="revealCard"><History :size="14" :stroke-width="1.75" /><span class="chip-text">继续「{{ continueTitle }}」</span></button>
          <button v-else-if="otherRecent" type="button" class="chip" data-testid="assistant-chip-recent" @click="continueOther(otherRecent.id)"><History :size="14" :stroke-width="1.75" /><span class="chip-text">继续「{{ otherRecent.title }}」</span></button>
        </div>
        <div class="composer" :class="{ 'drop-active': dropActive }" :data-phase="store.phase" @dragenter="dragEnter" @dragleave="dragLeave" @dragover="dragOver" @drop.stop="dropImages" @paste="pasteImages">
          <input ref="filePicker" class="sr" type="file" :accept="ATTACHMENT_ACCEPT" multiple tabindex="-1" aria-label="选择要发送的附件" data-testid="assistant-image-input" @change="onPick" @cancel="picking = false" />
          <div v-if="store.attachments.length" class="image-tray" data-no-drag data-testid="assistant-attachments">
            <div v-for="(image, index) in store.attachments" :key="index" class="image-thumb"><img v-if="attachmentImage(image)" :src="attachmentImage(image)!" :alt="`待发送的图片 ${index + 1}`" draggable="false" /><span v-if="!attachmentImage(image)" class="file-chip" :title="attachmentName(image)">{{ attachmentName(image) }}</span><button type="button" :aria-label="`移除附件 ${index + 1}`" :disabled="submitting || imageReading" @click="store.attachments.splice(index, 1)"><X :size="11" /></button></div>
          </div>
          <div v-if="store.live" class="capsule live" data-testid="assistant-live">
            <span class="live-orb" data-drag-handle><Orb :phase="store.muted ? 'idle' : store.phase === 'attention' ? 'idle' : store.phase" :level="asr.level" :active="!store.muted" :muted="store.muted" :size="34" /></span>
            <div class="live-text" data-drag-handle role="status" aria-live="polite" data-testid="assistant-live-status"><b>{{ store.liveStatus }}</b><span>{{ liveHint }}</span></div>
            <span class="live-clock" aria-label="通话时长">{{ liveClock }}</span>
            <button type="button" class="live-mute" :class="{ on: store.muted }" :aria-label="store.muted ? '取消静音' : '静音'" :title="store.muted ? '取消静音' : '静音'" :aria-pressed="store.muted" data-testid="assistant-live-mute" @click="act(() => store.toggleMute())"><MicOff v-if="store.muted" :size="16" :stroke-width="1.9" /><Mic v-else :size="16" :stroke-width="1.9" /></button>
            <button type="button" class="live-end" data-testid="assistant-live-end" @click="store.endLive()"><svg width="11" height="11" viewBox="0 0 24 24" aria-hidden="true"><rect x="4" y="4" width="16" height="16" rx="3.5" fill="currentColor" /></svg>结束</button>
          </div>
          <div v-else class="capsule">
            <button type="button" class="orb-button" :title="orbLabel + ' · 拖动可移动'" :aria-label="orbLabel" data-testid="assistant-orb" :disabled="imageReading || submitting || capturing || (asr.preparing && store.phase !== 'speaking')" data-drag-handle @click="onOrb">
              <Orb :phase="store.phase" :level="asr.level" :active="store.open" :size="32" />
            </button>
            <textarea ref="field" v-model="store.input" rows="1" :disabled="store.changingSession" class="field" autocomplete="off" spellcheck="false" :placeholder="store.phase === 'idle' ? '问小伴，或点左边的球录音…' : store.status" aria-label="向小伴提问" data-testid="assistant-input" @beforeinput="store.inputStarted()" @input="store.inputStarted(); grow()" @keydown.enter="onEnter" @compositionstart="composing = true; store.inputStarted()" @compositionend="composing = false" />
            <button type="button" class="tool" title="添加文件 · 也可以拖入或粘贴" aria-label="添加文件" data-testid="assistant-add-image" :disabled="!canAttach || store.attachments.length >= 8" @click="openPicker"><Plus :size="18" :stroke-width="1.75" /></button>
            <button type="button" class="tool" title="截图提问" aria-label="截图" data-testid="assistant-screenshot" :disabled="!canAttach || store.attachments.length >= 8" @click="takeScreenshot()"><SquareDashed :size="17" :stroke-width="1.75" /></button>
            <button v-if="offersLive" type="button" class="tool live-start" aria-label="实时对话" title="实时对话：像打电话一样和小伴说，随时可以打断" data-testid="assistant-live-start" :disabled="liveDisabled" @click="startLive"><AudioLines :size="18" :stroke-width="2" /></button>
            <button type="button" class="send" :class="{ 'has-action': canSend, 'is-stop': stopping }" :aria-label="canSend ? '发送' : stopping ? '停止' : '发送'" :title="canSend ? (store.phase === 'speaking' ? '发送并停止当前朗读' : '发送 · Enter') : stopping ? '停止回答' : '输入后发送'" :disabled="imageReading || capturing || (submitting && !busy) || (!canSend && !stopping)" data-testid="assistant-primary" @click="primary"><svg v-if="stopping" width="11" height="11" viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="3" width="18" height="18" rx="4" fill="currentColor" /></svg><ArrowUp v-else :size="17" :stroke-width="2.1" /></button>
          </div>
          <p v-if="dropActive || imageReading || capturing" class="image-status" aria-live="polite">{{ capturing ? '截图中，按 Esc 可以取消' : imageReading ? '正在读取附件…' : '松开添加附件' }}</p>
          <p v-else-if="store.voiceStatus && !store.live" class="image-status" data-testid="assistant-voice-status" role="status">{{ store.voiceStatus }}</p>
        </div>
        <span class="sr" aria-live="polite">{{ store.status }}</span>
      </div>
    </Transition>
    <template v-if="morphing">
      <div ref="glass" class="morph-glass" aria-hidden="true" />
      <div ref="travellingOrb" class="morph-orb" aria-hidden="true"><Orb phase="idle" :active="false" :size="44" /></div>
    </template>
    <SelectionCopyMenu :scope="scroller" @visibility-change="queueHitRegions" />
  </div>
</template>

<style>
html,body,#app{background:transparent!important}
</style>
<style scoped>
/* Glass surfaces in the console's language; the theme colour comes from --ui-btn / --ui-soft / --ui-ink / --ui-dot. */
.stage{--glass:rgba(42,43,48,.9);--glass-card:rgba(38,39,44,.95);--edge:rgba(255,255,255,.13);--line:rgba(255,255,255,.08);--text:#F2F2F4;--text-2:#C7C7CC;--text-3:#A1A1A8;--icon:#B4B4BA;--hover:rgba(255,255,255,.08);--control:rgba(255,255,255,.08);--you:rgba(255,255,255,.08);--shadow:0 1px 3px rgba(0,0,0,.28),0 4px 12px rgba(0,0,0,.3);--code:rgba(255,255,255,.06);--warn-bg:rgba(224,154,43,.12);--warn-line:rgba(224,154,43,.3);--warn-text:#F0B65A;--fail:#FF8A8F;--scroll:rgba(255,255,255,.18);
  --accent:var(--ui-btn,#E5484D);--accent-hi:var(--ui-btn-hi,#F06A6F);--accent-soft:var(--ui-soft,#3A2225);--accent-ink:var(--ui-ink,#FF8C91);--accent-dot:var(--ui-dot,#FF6369);
  --ui-line:var(--line);--ui-surface-alt:var(--code);--ui-text-primary:var(--text);--ui-text-secondary:var(--text-2);--ui-text-tertiary:var(--text-3);--ui-primary:var(--accent-ink);
  position:relative;display:flex;flex-direction:column;justify-content:flex-end;height:100vh;color:var(--text);font:14px/1.6 -apple-system,BlinkMacSystemFont,'SF Pro Text','PingFang SC','Segoe UI','Microsoft YaHei UI',sans-serif;-webkit-font-smoothing:antialiased;user-select:none;cursor:default;pointer-events:none;color-scheme:dark}
.stage.light{--glass:rgba(255,255,255,.9);--glass-card:rgba(255,255,255,.95);--edge:rgba(0,0,0,.13);--line:rgba(0,0,0,.06);--text:#1D1D1F;--text-2:#3A3A3F;--text-3:#6E6E73;--icon:#5A5A60;--hover:rgba(0,0,0,.055);--control:#F2F2F4;--you:#F2F2F4;--shadow:0 1px 3px rgba(0,0,0,.07),0 4px 12px rgba(0,0,0,.08);--code:rgba(0,0,0,.045);--warn-bg:#FDF6EA;--warn-line:#F1DDB8;--warn-text:#7A4A00;--fail:#C4323A;--scroll:rgba(0,0,0,.18);color-scheme:light}
button{font:inherit}
.sr{position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0);white-space:nowrap}

/* Ball */
/* The ball sits where the host says in the stage; a notice opens beside it toward the middle of the screen. */
.ball-stage{position:absolute;display:grid;place-items:center}
.ball-fade .ball-stage{transition:opacity .09s ease}
.moving-ball .ball-stage{opacity:0}
.ball-in-enter-active{transition:opacity .18s ease,transform .18s ease}
.ball-in-enter-from{opacity:0;transform:scale(.9)}
.ball{pointer-events:auto;touch-action:none;position:relative;flex:none;display:grid;place-items:center;width:44px;height:44px;padding:0;border:0;border-radius:50%;background:none;box-shadow:0 0 0 .5px var(--edge),0 1px 3px rgba(0,0,0,.2),0 3px 8px rgba(0,0,0,.18);cursor:grab;transition:transform .2s}
.stage.light .ball{box-shadow:0 0 0 .5px var(--edge),0 1px 3px rgba(0,0,0,.1),0 3px 8px rgba(0,0,0,.12)}
.ball:hover{transform:scale(1.06)}
.ball:active{transform:scale(.96)}
.ball:focus-visible{outline:2px solid var(--accent);outline-offset:3px}
.ball-ring{position:absolute;inset:-6px;width:56px;height:56px;pointer-events:none;animation:spin 1.1s linear infinite}
.ring-track{fill:none;stroke:var(--line);stroke-width:2.5}
.ring-arc{fill:none;stroke:var(--accent);stroke-width:2.5;stroke-linecap:round;stroke-dasharray:44 101}
.badge{position:absolute;top:-3px;right:-3px;display:grid;place-items:center;min-width:16px;height:16px;border-radius:8px;color:#fff;font-size:10.5px;font-weight:600;line-height:1;box-shadow:0 0 0 2px var(--glass)}
.badge.attention{background:#D98A0B;animation:nudge .5s ease-out 1}
.badge.error{background:#D93025}
.badge.unread{top:1px;right:1px;min-width:10px;width:10px;height:10px;background:var(--accent-dot)}
.bubble{position:absolute;top:50%;margin-top:-20px;left:calc(100% + 2px);width:max-content;white-space:nowrap;pointer-events:auto;display:flex;align-items:center;gap:8px;min-width:0;max-width:330px;height:40px;padding:0 6px 0 14px;border:0;border-radius:20px;background:var(--glass);backdrop-filter:blur(20px) saturate(1.6);box-shadow:0 0 0 .5px var(--edge),var(--shadow);color:var(--text);font-size:13px;cursor:pointer;white-space:nowrap}
.bubble-left .bubble{left:auto;right:calc(100% + 2px)}
.bubble b{font-weight:500;flex:none}
.bubble-icon{flex:none;color:var(--accent)}
.bubble.attention .bubble-icon{color:#D98A0B}.bubble.error .bubble-icon{color:var(--fail)}
.bubble-text{min-width:0;overflow:hidden;text-overflow:ellipsis;color:var(--text-3)}
.bubble-go{flex:none;height:28px;padding:0 12px;border-radius:14px;display:grid;place-items:center;background:var(--accent-soft);color:var(--accent-ink);font-size:12.5px;font-weight:500}
.bubble.attention .bubble-go{background:var(--warn-bg);color:var(--warn-text)}
.bubble-enter-active,.bubble-leave-active{transition:opacity .2s,transform .2s}
.bubble-enter-from,.bubble-leave-to{opacity:0;transform:scale(.96)}

/* Capsule */
/* The native window is transparent and clips at its edges: shadows stay within this margin. */
.inner{pointer-events:none;display:flex;flex-direction:column;gap:10px;padding:14px 16px 20px}
.composer{pointer-events:auto;cursor:grab;position:relative;border-radius:26px;background:var(--glass);backdrop-filter:blur(24px) saturate(1.6);box-shadow:0 0 0 .5px var(--edge),var(--shadow),inset 0 1px 0 rgba(255,255,255,.08);transition:box-shadow .2s}
.stage.light .composer{box-shadow:0 0 0 .5px var(--edge),var(--shadow),inset 0 1px 0 rgba(255,255,255,.95)}
.composer[data-phase=listening]{box-shadow:0 0 0 .5px var(--edge),0 0 0 4px var(--accent-soft),var(--shadow)}
.composer[data-phase=attention]{box-shadow:0 0 0 1px #D98A0B80,var(--shadow)}
.composer.drop-active{box-shadow:0 0 0 2px var(--accent),var(--shadow)}
.capsule{display:flex;align-items:flex-end;gap:2px;padding:8px}
.orb-button{touch-action:none;flex:none;display:grid;place-items:center;width:36px;height:36px;padding:0;border:0;border-radius:50%;background:none;cursor:grab}
.orb-button:disabled{opacity:.6}
.field:placeholder-shown{overflow:hidden}
.field{cursor:text;flex:1;min-width:0;height:36px;max-height:96px;margin:0;padding:8px 6px;box-sizing:border-box;border:0;outline:0;resize:none;background:transparent;color:var(--text);font:inherit;font-size:14px;line-height:20px;caret-color:var(--accent);user-select:text;scrollbar-width:thin;scrollbar-color:var(--scroll) transparent}
.field::placeholder{color:var(--text-3)}
.tool{flex:none;display:grid;place-items:center;width:32px;height:32px;margin-bottom:2px;padding:0;border:0;border-radius:9px;background:transparent;color:var(--icon);cursor:pointer;transition:background .15s,color .15s}
.tool:hover:not(:disabled){background:var(--hover);color:var(--text)}
.tool:disabled{opacity:.35;cursor:default}
.send{flex:none;display:grid;place-items:center;width:32px;height:32px;margin:0 2px 2px 4px;padding:0;border:0;border-radius:50%;background:var(--control);color:var(--text-3);cursor:default;transition:filter .15s}
.send.has-action{background:linear-gradient(var(--accent-hi),var(--accent));color:#fff;box-shadow:inset 0 1px 0 rgba(255,255,255,.28),0 1px 2px rgba(0,0,0,.18);cursor:pointer}
.send.has-action:hover{filter:brightness(1.06)}
.send.is-stop{background:var(--control);color:var(--text);cursor:pointer}
.send.is-stop:hover{background:var(--hover)}
.send:disabled:not(.is-stop){opacity:.8}
.tool.live-start{color:var(--accent-ink)}
.tool.live-start:hover:not(:disabled){background:var(--accent-soft);color:var(--accent-ink)}
.capsule.live{align-items:center;gap:10px;padding:8px 8px 8px 9px}
.live-orb{flex:none;display:grid;place-items:center;width:36px;height:36px;cursor:grab}
.live-text{flex:1;min-width:0;display:flex;flex-direction:column;line-height:1.35;cursor:grab}
.live-text b{font-size:13.5px;font-weight:600;color:var(--text)}
.live-text span{font-size:12px;color:var(--text-3);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.live-clock{flex:none;font:12px 'SF Mono',Menlo,Consolas,monospace;font-variant-numeric:tabular-nums;color:var(--text-3)}
.live-mute{flex:none;display:grid;place-items:center;width:34px;height:34px;padding:0;border:0;border-radius:50%;background:var(--control);color:var(--text);cursor:pointer}
.live-mute:hover{background:var(--hover)}
.live-mute.on{background:var(--text);color:var(--glass-card)}
.live-end{flex:none;display:inline-flex;align-items:center;gap:5px;height:34px;padding:0 14px;border:0;border-radius:17px;background:var(--text);color:var(--glass-card);font:inherit;font-size:13px;font-weight:500;cursor:pointer}
.live-end:hover{filter:brightness(1.1)}
.live-mute:focus-visible,.live-end:focus-visible{outline:2px solid var(--accent);outline-offset:2px}
.tool:focus-visible,.send:focus-visible,.orb-button:focus-visible,.head-btn:focus-visible,.chip:focus-visible,.session-title:focus-visible{outline:2px solid var(--accent);outline-offset:1px}
.image-tray{display:flex;gap:6px;flex-wrap:wrap;padding:10px 12px 0;cursor:default}
.image-thumb{position:relative;flex:none;height:42px}
.image-thumb img{width:56px;height:42px;object-fit:cover;border-radius:9px;box-shadow:0 0 0 .5px var(--edge)}
.image-thumb button{position:absolute;right:-4px;top:-4px;width:17px;height:17px;box-sizing:border-box;padding:0;display:grid;place-items:center;border:0;border-radius:50%;background:rgba(0,0,0,.6);color:#fff;cursor:pointer}
.file-chip{display:inline-flex;align-items:center;max-width:180px;height:42px;padding:0 12px;box-sizing:border-box;border-radius:9px;background:var(--control);font-size:12px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.image-status{margin:0;padding:0 18px 8px;font-size:11.5px;color:var(--text-3);cursor:default}

/* Quick actions */
.chips{pointer-events:none;display:flex;gap:8px;padding-left:4px}
.chip{pointer-events:auto;display:inline-flex;align-items:center;gap:6px;max-width:260px;height:30px;padding:0 12px;border:0;border-radius:15px;background:var(--glass);backdrop-filter:blur(20px);box-shadow:0 0 0 .5px var(--edge),0 1px 3px rgba(0,0,0,.06),0 3px 8px rgba(0,0,0,.07);color:var(--text-2);font-size:12.5px;cursor:pointer}
.chip svg{flex:none;color:var(--icon)}
.chip:hover:not(:disabled){color:var(--text)}
.chip:disabled{opacity:.5;cursor:default}
.chip-text{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}

/* Card */
.card{pointer-events:auto;position:relative;border-radius:18px;background:var(--glass-card);backdrop-filter:blur(24px) saturate(1.6);box-shadow:0 0 0 .5px var(--edge),var(--shadow);cursor:grab}
.card-head{display:flex;align-items:center;gap:4px;height:44px;padding:0 8px 0 8px;border-bottom:1px solid var(--line)}
.head-gap{flex:1}
.session-host{position:relative;min-width:0}
.session-title{display:inline-flex;align-items:center;gap:5px;max-width:220px;height:28px;padding:0 8px;border:0;border-radius:8px;background:transparent;color:var(--text);font-size:13px;font-weight:600;cursor:pointer}
.session-title span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.session-title svg{flex:none;color:var(--text-3);transition:transform .15s}
.session-title[aria-expanded=true] svg{transform:rotate(180deg)}
.session-title:hover,.session-title[aria-expanded=true]{background:var(--hover)}
.session-menu{max-height:340px;overflow-y:auto;padding:6px 8px 8px;box-sizing:border-box;cursor:default;scrollbar-width:thin;scrollbar-color:var(--scroll) transparent}
.menu-label{margin:0;padding:6px 9px 3px;font-size:11.5px;color:var(--text-3)}
.menu-empty{margin:0;padding:6px 9px;font-size:12.5px;color:var(--text-3)}
.menu-item{display:flex;align-items:center;gap:9px;width:100%;height:32px;padding:0 9px;border:0;border-radius:7px;background:transparent;color:var(--text);font-size:13px;text-align:left;cursor:pointer}
.menu-item:hover:not(:disabled){background:var(--hover)}
.menu-item[aria-checked=true]{background:var(--accent-soft);color:var(--accent-ink)}
.menu-item:disabled{opacity:.5;cursor:default}
.menu-check{flex:none;width:14px;color:var(--icon)}
.menu-item[aria-checked=true] .menu-check{color:var(--accent-ink)}
.menu-text{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.menu-sep{height:1px;margin:4px 8px;background:var(--line)}
.head-btn{flex:none;display:grid;place-items:center;width:28px;height:28px;padding:0;border:0;border-radius:8px;background:transparent;color:var(--icon);cursor:pointer}
.head-btn:hover:not(:disabled){background:var(--hover);color:var(--text)}
.head-btn.on{background:var(--accent-soft);color:var(--accent-ink)}
.head-btn:disabled{opacity:.4;cursor:default}
.card-body{cursor:auto;display:flex;flex-direction:column;gap:12px;max-height:min(340px,calc(100vh - 250px));overflow-y:auto;padding:14px 16px 12px;box-sizing:border-box;user-select:text;scrollbar-width:thin;scrollbar-color:var(--scroll) transparent}
/* While an answer streams the body keeps one height: the text grows inside it and scrolls,
   so the card, the composer and the native window do not resize while it arrives. */
.card-body.fixed{height:340px}
.you-row{align-self:flex-end;max-width:84%;display:flex;flex-direction:column;align-items:flex-end;gap:6px}
.you{margin:0;padding:7px 12px;border-radius:14px 14px 4px 14px;background:var(--you);font-size:13px;line-height:1.55;color:var(--text);overflow-wrap:anywhere;white-space:pre-wrap}
.you-row:last-of-type .you{background:var(--accent-soft);color:var(--accent-ink)}
.sent-files,.sent-images{display:flex;gap:6px;flex-wrap:wrap;justify-content:flex-end}
.sent-images img{width:56px;height:42px;object-fit:cover;border-radius:8px}
.reply{position:relative;font-size:13.5px;line-height:1.7;color:var(--text);overflow-wrap:anywhere}
.reply :deep(p){margin:0 0 .6em}
.reply :deep(p:last-child){margin-bottom:0}
.reply :deep(.md-code){background:var(--code);border-radius:5px;padding:1px 6px;font-size:.88em;font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,monospace}
.reply :deep(.md-link){color:var(--accent-ink);cursor:pointer;text-decoration:underline;text-underline-offset:2px}
.reply :deep(.md-heading){margin:.8em 0 .3em;font-size:14px}
.reply :deep(ul),.reply :deep(ol){margin:.2em 0 .6em;padding-left:1.4em}
.reply-actions{display:flex;gap:2px;margin-top:4px;opacity:0;transition:opacity .15s}
.reply:hover .reply-actions,.reply-actions.last,.reply-actions:focus-within{opacity:1}
/* Earlier answers keep their actions in a small floating bar, so the conversation has no empty rows. */
.reply-actions:not(.last){position:absolute;top:-8px;right:0;margin:0;padding:2px;border-radius:9px;background:var(--glass-card);box-shadow:0 0 0 .5px var(--edge),0 4px 12px rgba(0,0,0,.08)}
.reply-btn{display:inline-flex;align-items:center;gap:5px;height:26px;padding:0 8px;border:0;border-radius:7px;background:transparent;color:var(--text-3);font-size:12px;cursor:pointer}
.reply-btn:hover{background:var(--hover);color:var(--text)}
.reply-btn.on{color:var(--accent-ink)}
.cursor{display:inline-block;width:7px;height:15px;margin-left:2px;vertical-align:-2px;border-radius:2px;background:var(--accent-dot);animation:blink 1s steps(2,start) infinite}
.working{margin:0;display:flex;align-items:center;gap:10px;color:var(--text-3);font-size:13px}
.dots{display:inline-flex;gap:4px}
.dots i{width:6px;height:6px;border-radius:50%;background:var(--text-3);animation:dot 1.2s ease-in-out infinite}
.dots i:nth-child(2){animation-delay:.15s}.dots i:nth-child(3){animation-delay:.3s}
.hint{margin:0;color:var(--warn-text);font-size:13px}
.failure{margin:0;color:var(--fail);font-size:13px}
.failure.warn{color:var(--warn-text)}
.link{margin-left:8px;padding:0;border:0;background:none;color:var(--accent-ink);text-decoration:underline;text-underline-offset:2px;cursor:pointer}
.gate{padding:12px 14px;border-radius:12px;background:var(--warn-bg);box-shadow:inset 0 0 0 1px var(--warn-line)}
.gate-title{display:flex;align-items:center;gap:7px;margin:0;font-size:13px;font-weight:600;color:var(--warn-text)}
.gate-tool{margin:8px 0 10px;padding:7px 10px;border-radius:8px;background:var(--code);font:12px/1.5 ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;color:var(--text-2);overflow-wrap:anywhere}
.gate-actions{display:flex;gap:8px;justify-content:flex-end;flex-wrap:wrap}
.gate button{height:30px;padding:0 14px;border:0;border-radius:8px;font-size:12.5px;font-weight:500;cursor:pointer}
.gate .ghost{background:var(--glass-card);color:var(--text);box-shadow:0 0 0 1px var(--edge)}
.gate .solid{background:linear-gradient(var(--accent-hi),var(--accent));color:#fff;box-shadow:inset 0 1px 0 rgba(255,255,255,.28),0 1px 2px rgba(0,0,0,.18)}
.ask-user{display:flex;flex-direction:column;gap:6px}
.ask-user-question{margin:0;font-size:13.5px;font-weight:600;line-height:1.5;color:var(--text);overflow-wrap:anywhere}
.ask-user-tip{margin-left:8px;font-size:12px;font-weight:400;color:var(--text-3)}
.ask-user-tip kbd{padding:0 4px;border-radius:4px;background:var(--control);font:inherit;font-size:11px}
.ask-user-options{display:flex;flex-direction:column;gap:6px}
.ask-user-option{display:flex;align-items:center;gap:10px;width:100%;min-height:36px;padding:6px 10px;box-sizing:border-box;border:0;border-radius:9px;background:var(--control);color:var(--text);font-size:13px;text-align:left;cursor:pointer}
.ask-user-option:hover{background:var(--accent-soft);color:var(--accent-ink)}
.ask-user-index{flex:none;display:grid;place-items:center;width:20px;height:20px;border-radius:6px;background:var(--glass-card);box-shadow:0 0 0 .5px var(--edge);font-size:11px;font-weight:600;color:var(--text-3)}
.ask-user-option-text{flex:1;min-width:0;overflow-wrap:anywhere}
.ask-user-help{margin:0;font-size:12.5px;color:var(--text-3)}
.ask-user-skip{align-self:flex-end;padding:4px 6px;border:0;border-radius:6px;background:transparent;color:var(--text-3);font-size:12px;cursor:pointer}
.ask-user-skip:hover{color:var(--text)}

.stage.dragging,.stage.dragging *{cursor:grabbing!important}
.pop-enter-active{transition:opacity .2s ease,transform .28s cubic-bezier(.2,.9,.3,1.12)}
.pop-enter-from{opacity:0;transform:translateY(14px) scale(.97)}
.rise-enter-active{transition:opacity .22s ease,transform .26s ease}
.rise-enter-from{opacity:0;transform:translateY(8px)}
@keyframes blink{to{visibility:hidden}}
@keyframes dot{0%,80%,100%{opacity:.3;transform:translateY(0)}40%{opacity:1;transform:translateY(-3px)}}
@keyframes spin{to{transform:rotate(360deg)}}
@keyframes nudge{0%{transform:translateY(0)}40%{transform:translateY(-3px)}100%{transform:translateY(0)}}
@media(prefers-reduced-motion:reduce){.pop-enter-active,.rise-enter-active,.bubble-enter-active,.bubble-leave-active{transition:none}.dots i,.cursor,.ball-ring,.badge{animation:none}}
/* Stand-ins for the morph: the capsule's glass and the travelling mark, drawn above everything. */
.morph-glass{position:fixed;left:0;top:0;z-index:20;pointer-events:none;background:var(--glass);backdrop-filter:blur(24px) saturate(1.6);box-shadow:0 0 0 .5px var(--edge),var(--shadow)}
.morph-orb{position:fixed;left:0;top:0;z-index:21;width:44px;height:44px;pointer-events:none;will-change:transform}
.morph-expand .composer,.morph-expand .chips,.morph-expand .card,.morph-collapse .composer,.morph-collapse .chips{opacity:0}
.morph-collapse .ball-stage{opacity:0!important}
.morph-expand .inner,.morph-collapse .inner{pointer-events:none}
.pop-leave-active{transition:opacity .18s ease,transform .18s ease}
.pop-leave-to{opacity:0;transform:translateY(10px) scale(.98)}
</style>
