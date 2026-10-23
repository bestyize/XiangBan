<script setup lang="ts">
import { ATTACHMENT_ACCEPT, MAX_TOTAL_BYTES, attachmentBytes, attachmentImage, attachmentName, readAttachment } from '../ui/chat/attachments.ts';
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { ArrowUp, ArrowUpRight, Check, ChevronDown, Copy, Minus, Pin, Plus, RotateCcw, Settings, SquareDashed, SquarePen, TriangleAlert, X, History } from 'lucide-vue-next';
import MarkdownView from '../ui/chat/MarkdownView.ts';
import SelectionCopyMenu from '../ui/chat/SelectionCopyMenu.vue';
import type { AsrStore } from '../ui/asr/store.ts';
import type { ApprovalRequest, ChatItem } from '../ui/chat/types.ts';
import { invoke, listen } from '../bridge.ts';
import { morphGeometry, MORPH_DURATION } from './morph.ts';
import { isDark } from '../ui/appearance.ts';
import Orb from './Orb.vue';
import { hitRegion, type HitRegion } from './hit-regions.ts';
import type { AssistantStore } from './assistant-store.ts';

const morphing = ref(false), morphShell = ref<HTMLElement>();
const morphStyle = ref<Record<string, string>>({});
const morphVisible = ref(false);
interface MorphEvent { id: number; target: 'capsule' | 'ball'; step: 'leave' | 'enter' | 'show' | 'cancel'; right: boolean }
let offMorph: (() => void) | undefined, morphDisposed = false, morphId = 0;
let morphStep = '';
let morphFallback: ReturnType<typeof setTimeout> | undefined;
let morphAnimations: Animation[] = [];
let enterMorph: (() => Promise<void>) | undefined;
const reducedMotion = () => window.matchMedia?.('(prefers-reduced-motion: reduce)').matches ?? false;
function resetMorph() {
  clearTimeout(morphFallback);
  morphAnimations.splice(0).forEach(a => a.cancel());
  morphing.value = false; morphVisible.value = false; enterMorph = undefined;
  void nextTick(() => { queueHitRegions(); fit(); });
}
async function animateMorph(el: Element | undefined | null, frames: Keyframe[], duration = MORPH_DURATION, delay = 0) {
  if (!el?.animate || reducedMotion()) return;
  const animation = el.animate(frames, { duration, delay, easing: 'cubic-bezier(.22,.75,.2,1)', fill: 'both' });
  morphAnimations.push(animation);
  try { await animation.finished; } catch { /* superseded or unmounted */ }
}
async function prepareMorph(right: boolean) {
  const composer = document.querySelector<HTMLElement>('.composer');
  if (!composer) return null;
  const { full, ball } = morphGeometry(composer.getBoundingClientRect(), right);
  morphStyle.value = { left: `${full.left}px`, top: `${full.top}px`, width: `${full.width}px`, height: `${full.height}px` };
  morphVisible.value = true;
  await nextTick();
  return { full, ball };
}
const morphFrame = (frame: ReturnType<typeof morphGeometry>['full']) => ({ left: `${frame.left}px`, top: `${frame.top}px`, width: `${frame.width}px`, height: `${frame.height}px`, borderRadius: frame.borderRadius });
async function handleMorph(event: MorphEvent) {
  if (morphDisposed || event.id < morphId) return;
  morphStep = event.step;
  if (event.step === 'leave') {
    resetMorph(); morphId = event.id; closeMenus(); morphing.value = true;
    morphFallback = setTimeout(resetMorph, 1800);
    try {
      const frames = event.target === 'ball' ? await prepareMorph(event.right) : null;
      if (frames) await Promise.all([
        animateMorph(morphShell.value, [morphFrame(frames.full), morphFrame(frames.ball)]),
        animateMorph(document.querySelector('.card'), [{ opacity: 1, transform: 'translateY(0)' }, { opacity: 0, transform: 'translateY(6px)' }], 140),
      ]);
      else await animateMorph(document.querySelector('.ball'), [{ opacity: 1, transform: 'scale(1)' }, { opacity: 0, transform: 'scale(.92)' }], 100);
    } finally { if (!morphDisposed && event.id === morphId && morphStep === 'leave') await invoke('assistant_transition_ready', { id: event.id, step: 'leave' }); }
  } else if (event.step === 'enter') {
    morphAnimations.splice(0).forEach(a => a.cancel()); morphVisible.value = false;
    await nextTick();
    if (event.target === 'capsule' && inner.value) {
      await invoke('assistant_fit', { height: Math.max(156, Math.ceil(inner.value.getBoundingClientRect().height)) });
      if (event.id !== morphId || morphStep !== 'enter') return;
      await nextTick();
    }
    if (event.id !== morphId || morphStep !== 'enter') return;
    // The native window has its final rectangle, but remains hidden until this acknowledgement.
    const frames = event.target === 'capsule' && props.store.open ? await prepareMorph(event.right) : null;
    if (frames) {
      morphStyle.value = morphFrame(frames.ball);
      enterMorph = async () => {
        await Promise.all([
          animateMorph(morphShell.value, [morphFrame(frames.ball), morphFrame(frames.full)]),
          animateMorph(document.querySelector('.card'), [{ opacity: 0, transform: 'translateY(6px)' }, { opacity: 1, transform: 'translateY(0)' }], 150, 110),
        ]);
      };
    } else enterMorph = () => animateMorph(document.querySelector('.ball'), [{ opacity: 0, transform: 'scale(.92)' }, { opacity: 1, transform: 'scale(1)' }], 140);
    await nextTick();
    await invoke('assistant_transition_ready', { id: event.id, step: 'enter' });
  } else if (event.step === 'show') {
    try { await enterMorph?.(); } finally { if (event.id === morphId) { resetMorph(); if (props.store.open) field.value?.focus(); } }
  } else resetMorph();
}

const props = defineProps<{ store: AssistantStore; asr: AsrStore }>();
const orbLabel = computed(() => {
  if (props.store.phase === 'speaking') return '停止朗读';
  if (props.asr.listening) return '停止聆听';
  if (props.asr.recordingRequested) return '取消语音启动';
  if (props.asr.preparing) return '正在准备语音输入';
  if (props.asr.microphoneSupported === false) return '当前设备不支持语音输入';
  if (props.asr.enabled === false) return '开启语音输入';
  if (props.asr.microphoneError) return '重试语音输入';
  return '点击开始说话';
});
const field = ref<HTMLTextAreaElement>();
const filePicker = ref<HTMLInputElement>();
const imageReading = ref(false), submitting = ref(false), dropActive = ref(false), capturing = ref(false), picking = ref(false);
/** Pinned: a click elsewhere does not fold the capsule away. */
const pinned = ref(false);
const sessionsOpen = ref(false), sessionsHost = ref<HTMLElement>();
function closeMenus() { sessionsOpen.value = false; }
function outsideMenus(event: PointerEvent) { if (sessionsOpen.value && !sessionsHost.value?.contains(event.target as Node)) closeMenus(); }
function escape() {
  if (sessionsOpen.value) { closeMenus(); return; }
  if (capturing.value) return;
  if (props.store.input) { props.store.input = ''; return; }
  void props.store.hide();
}
async function takeScreenshot() {
  if (!canAttach.value || props.store.attachments.length >= 8) return;
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
watch(() => props.store.open, () => { imageVersion++; dropDepth = 0; dropActive.value = false; closeMenus(); });
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
const chipsShown = computed(() => props.store.open && !hasCard.value && !busy.value && !props.store.input && !props.store.attachments.length);
async function continueOther(id: string) { await props.store.switchSession(id); cardShown.value = true; void nextTick(() => field.value?.focus()); }

const canSend = computed(() => (!busy.value || props.store.phase === 'speaking') && !props.store.approval && !imageReading.value && !submitting.value && !capturing.value && !props.store.changingSession && Boolean(props.store.input.trim() || props.store.attachments.length));
const stopping = computed(() => busy.value && !canSend.value);

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

/** The text box grows with its text, up to four lines, then scrolls. */
function grow() {
  const el = field.value;
  if (!el) return;
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
const bubbleLeft = ref<boolean | null>(null);
let bubbleTimer: ReturnType<typeof setTimeout> | undefined, bubbleVersion = 0;
const bubbleWanted = computed(() => Boolean(props.store.ball && !props.store.open && props.store.notice && !dragging.value && !morphing.value));
watch(bubbleWanted, async wanted => {
  const version = ++bubbleVersion;
  clearTimeout(bubbleTimer);
  if (!wanted) {
    if (bubbleLeft.value !== null) { bubbleLeft.value = null; await invoke('assistant_ball_bubble', { open: false }).catch(() => {}); }
    return;
  }
  const left = await invoke<boolean | null>('assistant_ball_bubble', { open: true }).catch(() => null);
  if (version !== bubbleVersion) return;
  bubbleLeft.value = typeof left === 'boolean' ? left : true;
  // A finished answer says so briefly, then the unread dot is enough. Waiting and errors stay until seen.
  if (props.store.notice?.kind === 'done') bubbleTimer = setTimeout(() => props.store.dismissNotice(), 3000);
  void nextTick(queueHitRegions);
}, { immediate: true });
watch(() => props.store.notice?.text, () => {
  if (bubbleLeft.value === null || props.store.notice?.kind !== 'done') return;
  clearTimeout(bubbleTimer); bubbleTimer = setTimeout(() => props.store.dismissNotice(), 3000);
});

/* ── Click elsewhere folds the capsule away, unless something would be lost or is waiting ─── */
const canAutoHide = computed(() => props.store.open && !pinned.value && props.store.phase === 'idle' && !props.store.input.trim()
  && !props.store.attachments.length && !capturing.value && !picking.value && !imageReading.value && !submitting.value
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

let fitted = 0, raf = 0;
let observer: ResizeObserver | undefined;
function fit() {
  queueHitRegions();
  cancelAnimationFrame(raf);
  if(dragging.value || capturing.value || morphing.value)return;
  raf = requestAnimationFrame(() => {
    if(dragging.value || capturing.value || morphing.value)return;
    const el = inner.value;
    if (!el) return;
    const height = Math.max(156, Math.ceil(el.getBoundingClientRect().height));
    if (height === fitted) return;
    fitted = height;
    void invoke('assistant_fit', { height }).catch(() => {});
  });
}
watch(inner, el => {
  observer?.disconnect();
  if (el) { observer = new ResizeObserver(fit); observer.observe(el); fit(); }
});
function toBottom() { void nextTick(() => { if (scroller.value) scroller.value.scrollTop = scroller.value.scrollHeight; }); }
watch(() => props.store.open, open => {
  if (!open) return;
  fitted = 0;
  void nextTick(() => { fit(); grow(); field.value?.focus(); toBottom(); });
});
watch(hasCard, shown => { if (shown) toBottom(); });
watch(() => [props.store.answer, turns.value.length, props.store.ask?.id, props.store.approval?.id], toBottom);

const copied = ref(''), copyFailed = ref('');
let copyTimer: ReturnType<typeof setTimeout> | undefined;
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
  pulled=true;dragging.value=true;cancelAnimationFrame(raf);releasePress();
  // A widened ball window narrows back before it moves, so the saved spot is the ball's own.
  if(bubbleLeft.value!==null)props.store.dismissNotice();
  void invoke('assistant_drag').catch(()=>{dragging.value=false;fit();});
}
function dragUp(event:PointerEvent){if(press?.id===event.pointerId)releasePress();}
function captureClick(event:MouseEvent){if(pulled&&event.detail!==0){event.preventDefault();event.stopImmediatePropagation();pulled=false;}}
function dragEnded(){dragging.value=false;releasePress();fit();}
function ballClick(){void props.store.unfold();}
const onFocus=()=>{picking.value=false;clearTimeout(blurTimer);if(props.store.open&&!dragging.value&&!press&&!morphing.value)field.value?.focus();};
onMounted(()=>{
  void listen<MorphEvent>('assistant-transition', event => { void handleMorph(event).catch(() => resetMorph()); }).then(off => { if (morphDisposed) off(); else offMorph = off; });
  void nextTick(queueHitRegions);window.addEventListener('resize',queueHitRegions);
  document.addEventListener('pointerdown',outsideMenus);document.addEventListener('keydown',onAskShortcut);window.addEventListener('blur',onBlur);window.addEventListener('focus',onFocus);window.addEventListener('pointermove',dragMove);window.addEventListener('pointerup',dragUp);window.addEventListener('pointercancel',dragUp);window.addEventListener('blur',releasePress);window.addEventListener('assistant-drag-end',dragEnded);
});
onBeforeUnmount(()=>{
  morphDisposed = true; clearTimeout(morphFallback); offMorph?.(); morphAnimations.splice(0).forEach(a => a.cancel());
  cancelAnimationFrame(hitRaf);window.removeEventListener('resize',queueHitRegions);clearTimeout(blurTimer);clearTimeout(bubbleTimer);clearTimeout(copyTimer);bubbleVersion++;
  window.removeEventListener('blur',onBlur);document.removeEventListener('pointerdown',outsideMenus);document.removeEventListener('keydown',onAskShortcut);imageVersion++; releasePress();window.removeEventListener('focus',onFocus);window.removeEventListener('pointermove',dragMove);window.removeEventListener('pointerup',dragUp);window.removeEventListener('pointercancel',dragUp);window.removeEventListener('blur',releasePress);window.removeEventListener('assistant-drag-end',dragEnded);observer?.disconnect();cancelAnimationFrame(raf);
});
</script>

<template>
  <div class="stage" :class="{ light: !isDark, dragging, morphing }" @dragover="dragOver" @drop="dropImages" @keydown.esc.prevent.stop="escape" @pointerdown="dragDown" @click.capture="captureClick">
    <div v-if="store.ball && !store.open" class="ball-stage" :class="{ 'has-bubble': bubbleLeft !== null, 'bubble-left': bubbleLeft === true }" data-testid="assistant-ball-stage">
      <button type="button" class="ball" :data-phase="store.phase" :data-badge="ballBadge || undefined" title="点一下打开，按住可以拖动，右键打开设置" :aria-label="ballLabel" data-testid="assistant-ball" data-drag-handle @click="ballClick" @contextmenu.prevent="store.openSettings('assistant')">
        <Orb :phase="store.phase === 'attention' ? 'idle' : store.phase" :active="store.ball && !store.open" :size="34" />
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
    <Transition name="pop" appear @after-enter="queueHitRegions" @after-leave="queueHitRegions">
      <div v-if="store.open" ref="inner" class="inner" role="dialog" aria-label="超级小伴" data-testid="assistant">
        <Transition name="rise" @after-enter="queueHitRegions" @after-leave="queueHitRegions">
          <section v-if="hasCard" class="card" data-testid="assistant-card">
            <header class="card-head">
              <div ref="sessionsHost" class="session-host" data-no-drag>
                <button type="button" class="session-title" :aria-expanded="sessionsOpen" aria-haspopup="menu" title="切换最近的会话" data-testid="assistant-sessions" @click="sessionsOpen = !sessionsOpen"><span>{{ store.title || '新对话' }}</span><ChevronDown :size="13" :stroke-width="2.2" /></button>
                <div v-if="sessionsOpen" class="session-menu" role="menu" aria-label="最近会话" data-testid="assistant-session-menu">
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
              </div>
              <span class="head-gap" />
              <button type="button" class="head-btn" title="新对话" aria-label="新对话" data-testid="assistant-new-chat" :disabled="imageReading || submitting || capturing || store.changingSession" @click="newChat"><SquarePen :size="16" :stroke-width="1.75" /></button>
              <button type="button" class="head-btn" title="在主窗口继续" aria-label="在主窗口继续" data-testid="assistant-expand" @click="act(() => store.openChat())"><ArrowUpRight :size="16" :stroke-width="1.75" /></button>
              <button type="button" class="head-btn" :class="{ on: pinned }" :title="pinned ? '取消固定' : '固定，点别处也不收起'" :aria-label="pinned ? '取消固定' : '固定'" :aria-pressed="pinned" data-testid="assistant-pin" @click="pinned = !pinned"><Pin :size="16" :stroke-width="1.75" :fill="pinned ? 'currentColor' : 'none'" /></button>
              <button type="button" class="head-btn" title="收成小球 · Esc" aria-label="收成小球" data-testid="assistant-close" @click="act(() => store.hide())"><Minus :size="16" :stroke-width="1.9" /></button>
            </header>
            <div ref="scroller" class="card-body" :class="{ fixed: fixedBody }" data-testid="assistant-card-body">
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
          <button type="button" class="chip" data-testid="assistant-chip-screenshot" :disabled="!canAttach" @click="takeScreenshot"><SquareDashed :size="14" :stroke-width="1.75" />截图问一下</button>
          <button v-if="continueTitle" type="button" class="chip" data-testid="assistant-chip-continue" @click="revealCard"><History :size="14" :stroke-width="1.75" /><span class="chip-text">继续「{{ continueTitle }}」</span></button>
          <button v-else-if="otherRecent" type="button" class="chip" data-testid="assistant-chip-recent" @click="continueOther(otherRecent.id)"><History :size="14" :stroke-width="1.75" /><span class="chip-text">继续「{{ otherRecent.title }}」</span></button>
        </div>
        <div class="composer" :class="{ 'drop-active': dropActive }" :data-phase="store.phase" @dragenter="dragEnter" @dragleave="dragLeave" @dragover="dragOver" @drop.stop="dropImages" @paste="pasteImages">
          <input ref="filePicker" class="sr" type="file" :accept="ATTACHMENT_ACCEPT" multiple tabindex="-1" aria-label="选择要发送的附件" data-testid="assistant-image-input" @change="onPick" @cancel="picking = false" />
          <div v-if="store.attachments.length" class="image-tray" data-no-drag data-testid="assistant-attachments">
            <div v-for="(image, index) in store.attachments" :key="index" class="image-thumb"><img v-if="attachmentImage(image)" :src="attachmentImage(image)!" :alt="`待发送的图片 ${index + 1}`" draggable="false" /><span v-if="!attachmentImage(image)" class="file-chip" :title="attachmentName(image)">{{ attachmentName(image) }}</span><button type="button" :aria-label="`移除附件 ${index + 1}`" :disabled="submitting || imageReading" @click="store.attachments.splice(index, 1)"><X :size="11" /></button></div>
          </div>
          <div class="capsule">
            <button type="button" class="orb-button" :title="orbLabel + ' · 拖动可移动'" :aria-label="orbLabel" data-testid="assistant-orb" :disabled="imageReading || submitting || capturing || (asr.preparing && store.phase !== 'speaking')" data-drag-handle @click="onOrb">
              <Orb :phase="store.phase" :level="asr.level" :active="store.open" :size="32" :dark="isDark" />
            </button>
            <textarea ref="field" v-model="store.input" rows="1" :disabled="store.changingSession" class="field" autocomplete="off" spellcheck="false" :placeholder="store.phase === 'idle' ? '问小伴，或点左边的球说话…' : store.status" aria-label="向小伴提问" data-testid="assistant-input" @beforeinput="store.inputStarted()" @input="store.inputStarted(); grow()" @keydown.enter="onEnter" @compositionstart="composing = true; store.inputStarted()" @compositionend="composing = false" />
            <button type="button" class="tool" title="添加文件 · 也可以拖入或粘贴" aria-label="添加文件" data-testid="assistant-add-image" :disabled="!canAttach || store.attachments.length >= 8" @click="openPicker"><Plus :size="18" :stroke-width="1.75" /></button>
            <button type="button" class="tool" title="截图提问" aria-label="截图" data-testid="assistant-screenshot" :disabled="!canAttach || store.attachments.length >= 8" @click="takeScreenshot"><SquareDashed :size="17" :stroke-width="1.75" /></button>
            <button type="button" class="send" :class="{ 'has-action': canSend, 'is-stop': stopping }" :aria-label="canSend ? '发送' : stopping ? '停止' : '发送'" :title="canSend ? (store.phase === 'speaking' ? '发送并停止当前朗读' : '发送 · Enter') : stopping ? '停止回答' : '输入后发送'" :disabled="imageReading || capturing || (submitting && !busy) || (!canSend && !stopping)" data-testid="assistant-primary" @click="primary"><svg v-if="stopping" width="11" height="11" viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="3" width="18" height="18" rx="4" fill="currentColor" /></svg><ArrowUp v-else :size="17" :stroke-width="2.1" /></button>
          </div>
          <p v-if="dropActive || imageReading || capturing" class="image-status" aria-live="polite">{{ capturing ? '截图中，按 Esc 可以取消' : imageReading ? '正在读取附件…' : '松开添加附件' }}</p>
          <p v-else-if="store.voiceStatus" class="image-status" data-testid="assistant-voice-status" role="status">{{ store.voiceStatus }}</p>
        </div>
        <span class="sr" aria-live="polite">{{ store.status }}</span>
      </div>
    </Transition>
    <div v-if="morphVisible" ref="morphShell" class="morph-shell" :style="morphStyle" aria-hidden="true"><Orb phase="idle" :size="34" flowing /></div>
    <SelectionCopyMenu :scope="scroller" @visibility-change="queueHitRegions" />
  </div>
</template>

<style>
html,body,#app{background:transparent!important}
</style>
<style scoped>
/* Glass surfaces in the console's language; the theme colour comes from --ui-btn / --ui-soft / --ui-ink / --ui-dot. */
.stage{--glass:rgba(42,43,48,.9);--glass-card:rgba(38,39,44,.95);--edge:rgba(255,255,255,.13);--line:rgba(255,255,255,.08);--text:#F2F2F4;--text-2:#C7C7CC;--text-3:#A1A1A8;--icon:#B4B4BA;--hover:rgba(255,255,255,.08);--control:rgba(255,255,255,.08);--you:rgba(255,255,255,.08);--shadow:0 12px 34px rgba(0,0,0,.45);--code:rgba(255,255,255,.06);--warn-bg:rgba(224,154,43,.12);--warn-line:rgba(224,154,43,.3);--warn-text:#F0B65A;--fail:#FF8A8F;--scroll:rgba(255,255,255,.18);
  --accent:var(--ui-btn,#E5484D);--accent-hi:var(--ui-btn-hi,#F06A6F);--accent-soft:var(--ui-soft,#3A2225);--accent-ink:var(--ui-ink,#FF8C91);--accent-dot:var(--ui-dot,#FF6369);
  --ui-line:var(--line);--ui-surface-alt:var(--code);--ui-text-primary:var(--text);--ui-text-secondary:var(--text-2);--ui-text-tertiary:var(--text-3);--ui-primary:var(--accent-ink);
  position:relative;display:flex;flex-direction:column;justify-content:flex-end;height:100vh;color:var(--text);font:14px/1.6 -apple-system,BlinkMacSystemFont,'SF Pro Text','PingFang SC','Segoe UI','Microsoft YaHei UI',sans-serif;-webkit-font-smoothing:antialiased;user-select:none;cursor:default;pointer-events:none;color-scheme:dark}
.stage.light{--glass:rgba(255,255,255,.9);--glass-card:rgba(255,255,255,.95);--edge:rgba(0,0,0,.13);--line:rgba(0,0,0,.06);--text:#1D1D1F;--text-2:#3A3A3F;--text-3:#6E6E73;--icon:#5A5A60;--hover:rgba(0,0,0,.055);--control:#F2F2F4;--you:#F2F2F4;--shadow:0 10px 30px rgba(0,0,0,.14);--code:rgba(0,0,0,.045);--warn-bg:#FDF6EA;--warn-line:#F1DDB8;--warn-text:#7A4A00;--fail:#C4323A;--scroll:rgba(0,0,0,.18);color-scheme:light}
button{font:inherit}
.sr{position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0);white-space:nowrap}

/* Ball */
.ball-stage{display:flex;align-items:center;height:100vh;padding:0 14px;box-sizing:border-box;gap:10px}
.ball-stage:not(.has-bubble){justify-content:center;padding:0}
.ball-stage.bubble-left{flex-direction:row-reverse}
.ball{pointer-events:auto;touch-action:none;position:relative;flex:none;display:grid;place-items:center;width:44px;height:44px;padding:0;border:0;border-radius:50%;background:var(--glass);backdrop-filter:blur(20px) saturate(1.6);box-shadow:0 0 0 .5px var(--edge),0 6px 18px rgba(0,0,0,.16),inset 0 1px 0 rgba(255,255,255,.12);cursor:grab;transition:transform .2s}
.stage.light .ball{box-shadow:0 0 0 .5px var(--edge),0 6px 18px rgba(0,0,0,.14),inset 0 1px 0 rgba(255,255,255,.9)}
.ball:hover{transform:scale(1.06)}
.ball:active{transform:scale(.96)}
.ball:focus-visible{outline:2px solid var(--accent);outline-offset:3px}
.ball-ring{position:absolute;inset:-3px;width:50px;height:50px;pointer-events:none;animation:spin 1.1s linear infinite}
.ring-track{fill:none;stroke:var(--line);stroke-width:2.5}
.ring-arc{fill:none;stroke:var(--accent);stroke-width:2.5;stroke-linecap:round;stroke-dasharray:44 101}
.badge{position:absolute;top:-3px;right:-3px;display:grid;place-items:center;min-width:16px;height:16px;border-radius:8px;color:#fff;font-size:10.5px;font-weight:600;line-height:1;box-shadow:0 0 0 2px var(--glass)}
.badge.attention{background:#D98A0B;animation:nudge .5s ease-out 1}
.badge.error{background:#D93025}
.badge.unread{top:1px;right:1px;min-width:10px;width:10px;height:10px;background:var(--accent-dot)}
.bubble{pointer-events:auto;display:flex;align-items:center;gap:8px;min-width:0;max-width:330px;height:40px;padding:0 6px 0 14px;border:0;border-radius:20px;background:var(--glass);backdrop-filter:blur(20px) saturate(1.6);box-shadow:0 0 0 .5px var(--edge),var(--shadow);color:var(--text);font-size:13px;cursor:pointer;white-space:nowrap}
.bubble b{font-weight:500;flex:none}
.bubble-icon{flex:none;color:var(--accent)}
.bubble.attention .bubble-icon{color:#D98A0B}.bubble.error .bubble-icon{color:var(--fail)}
.bubble-text{min-width:0;overflow:hidden;text-overflow:ellipsis;color:var(--text-3)}
.bubble-go{flex:none;height:28px;padding:0 12px;border-radius:14px;display:grid;place-items:center;background:var(--accent-soft);color:var(--accent-ink);font-size:12.5px;font-weight:500}
.bubble.attention .bubble-go{background:var(--warn-bg);color:var(--warn-text)}
.bubble-enter-active,.bubble-leave-active{transition:opacity .2s,transform .2s}
.bubble-enter-from,.bubble-leave-to{opacity:0;transform:scale(.96)}

/* Capsule */
.inner{pointer-events:none;display:flex;flex-direction:column;gap:10px;padding:12px}
.composer{pointer-events:auto;cursor:grab;position:relative;border-radius:26px;background:var(--glass);backdrop-filter:blur(24px) saturate(1.6);box-shadow:0 0 0 .5px var(--edge),var(--shadow),inset 0 1px 0 rgba(255,255,255,.08);transition:box-shadow .2s}
.stage.light .composer{box-shadow:0 0 0 .5px var(--edge),var(--shadow),inset 0 1px 0 rgba(255,255,255,.95)}
.composer[data-phase=listening]{box-shadow:0 0 0 .5px var(--edge),0 0 0 4px var(--accent-soft),var(--shadow)}
.composer[data-phase=attention]{box-shadow:0 0 0 1px #D98A0B80,var(--shadow)}
.composer.drop-active{box-shadow:0 0 0 2px var(--accent),var(--shadow)}
.capsule{display:flex;align-items:flex-end;gap:2px;padding:8px}
.orb-button{touch-action:none;flex:none;display:grid;place-items:center;width:36px;height:36px;padding:0;border:0;border-radius:50%;background:none;cursor:grab}
.orb-button:disabled{opacity:.6}
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
.tool:focus-visible,.send:focus-visible,.orb-button:focus-visible,.head-btn:focus-visible,.chip:focus-visible,.session-title:focus-visible{outline:2px solid var(--accent);outline-offset:1px}
.image-tray{display:flex;gap:6px;flex-wrap:wrap;padding:10px 12px 0;cursor:default}
.image-thumb{position:relative;flex:none;height:42px}
.image-thumb img{width:56px;height:42px;object-fit:cover;border-radius:9px;box-shadow:0 0 0 .5px var(--edge)}
.image-thumb button{position:absolute;right:-4px;top:-4px;width:17px;height:17px;box-sizing:border-box;padding:0;display:grid;place-items:center;border:0;border-radius:50%;background:rgba(0,0,0,.6);color:#fff;cursor:pointer}
.file-chip{display:inline-flex;align-items:center;max-width:180px;height:42px;padding:0 12px;box-sizing:border-box;border-radius:9px;background:var(--control);font-size:12px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.image-status{margin:0;padding:0 18px 8px;font-size:11.5px;color:var(--text-3);cursor:default}

/* Quick actions */
.chips{pointer-events:none;display:flex;gap:8px;padding-left:4px}
.chip{pointer-events:auto;display:inline-flex;align-items:center;gap:6px;max-width:260px;height:30px;padding:0 12px;border:0;border-radius:15px;background:var(--glass);backdrop-filter:blur(20px);box-shadow:0 0 0 .5px var(--edge),0 4px 12px rgba(0,0,0,.1);color:var(--text-2);font-size:12.5px;cursor:pointer}
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
.session-title svg{flex:none;color:var(--text-3)}
.session-title:hover,.session-title[aria-expanded=true]{background:var(--hover)}
.session-menu{position:absolute;z-index:10;top:calc(100% + 4px);left:0;width:264px;padding:5px;box-sizing:border-box;border-radius:12px;background:var(--glass-card);backdrop-filter:blur(20px);box-shadow:0 0 0 .5px var(--edge),0 14px 36px rgba(0,0,0,.22);cursor:default}
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
.card-body{cursor:auto;display:flex;flex-direction:column;gap:12px;max-height:340px;overflow-y:auto;padding:14px 16px 12px;box-sizing:border-box;user-select:text;scrollbar-width:thin;scrollbar-color:var(--scroll) transparent}
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
.morph-shell{position:fixed;z-index:20;display:flex;align-items:center;justify-content:center;overflow:hidden;pointer-events:none;border-radius:26px;background:var(--glass);box-shadow:0 0 0 .5px var(--edge),var(--shadow)}
.morph-shell :deep(.orb-frame){position:absolute;left:5px}
.morphing .composer,.morphing .chips{opacity:0;pointer-events:none}
.morphing .card,.morphing .ball{opacity:0;pointer-events:none}
.morphing .inner{transform:none!important;transition:none!important}
.morphing .pop-enter-from{opacity:1}
</style>
