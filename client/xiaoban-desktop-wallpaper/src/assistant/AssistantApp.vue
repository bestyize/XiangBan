<script setup lang="ts">
import { ATTACHMENT_ACCEPT, MAX_TOTAL_BYTES, attachmentBytes, attachmentImage, attachmentName, readAttachment } from '../ui/chat/attachments.ts';
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { ArrowUp, Square, X, Copy, Check, ImagePlus, Settings, ChevronDown, ChevronRight, MessageSquare, MessageSquarePlus, ScanLine, Plus } from 'lucide-vue-next';
import MarkdownView from '../ui/chat/MarkdownView.ts';
import SelectionCopyMenu from '../ui/chat/SelectionCopyMenu.vue';
import type { AsrStore } from '../ui/asr/store.ts';
import type { ApprovalRequest } from '../ui/chat/types.ts';
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
    resetMorph(); morphId = event.id; closeTools(); morphing.value = true;
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
const field = ref<HTMLInputElement>();
const filePicker = ref<HTMLInputElement>();
const imageReading = ref(false), submitting = ref(false), dropActive = ref(false), capturing = ref(false);
const toolsOpen = ref(false), toolsPinned = ref(false);
const toolsHost = ref<HTMLElement>(), toolbar = ref<HTMLElement>(), primaryButton = ref<HTMLButtonElement>();
let toolsOpenTimer: ReturnType<typeof setTimeout> | undefined, toolsCloseTimer: ReturnType<typeof setTimeout> | undefined;
function clearToolTimers() { clearTimeout(toolsOpenTimer); clearTimeout(toolsCloseTimer); }
function closeTools() { clearToolTimers(); toolsPinned.value = false; toolsOpen.value = false; }
function showTools() { clearToolTimers(); if (!capturing.value) toolsOpen.value = true; }
function enterTools() { clearTimeout(toolsCloseTimer); if (!toolsOpen.value) { clearTimeout(toolsOpenTimer); toolsOpenTimer = setTimeout(showTools, 220); } }
function leaveTools() {
  clearTimeout(toolsOpenTimer);
  if (!toolsPinned.value && !toolsHost.value?.contains(document.activeElement)) toolsCloseTimer = setTimeout(closeTools, 400);
}
function focusTools() { clearToolTimers(); }
function toolsFocusOut() { void nextTick(() => { if (!toolsHost.value?.contains(document.activeElement) && !toolsPinned.value) closeTools(); }); }
function outsideTools(event: PointerEvent) { if (!toolsHost.value?.contains(event.target as Node)) closeTools(); }
function escape() {
  if (toolsOpen.value) { closeTools(); primaryButton.value?.focus(); return; }
  if (!capturing.value) void props.store.hide();
}
function toolbarKeys(event: KeyboardEvent) {
  const buttons = Array.from(toolbar.value?.querySelectorAll<HTMLButtonElement>('button:not(:disabled)') ?? []);
  if (!buttons.length) return;
  let at = buttons.indexOf(document.activeElement as HTMLButtonElement);
  if (event.key === 'ArrowRight' || event.key === 'ArrowLeft') { event.preventDefault(); at = (at + (event.key === 'ArrowRight' ? 1 : -1) + buttons.length) % buttons.length; buttons[at].focus(); }
  if (event.key === 'Home' || event.key === 'End') { event.preventDefault(); buttons[event.key === 'Home' ? 0 : buttons.length - 1].focus(); }
}
async function primaryKeys(event: KeyboardEvent) {
  if (!['ArrowUp', 'ArrowDown'].includes(event.key)) return;
  event.preventDefault(); showTools(); await nextTick(); toolbar.value?.querySelector<HTMLButtonElement>('button:not(:disabled)')?.focus();
}
async function toolAction(action: 'settings' | 'hide' | 'new' | 'chat' | 'image' | 'screenshot') {
  closeTools();
  if (action === 'image') { openPicker(); return; }
  if (action === 'screenshot') { await takeScreenshot(); return; }
  try {
    if (action === 'settings') await props.store.openSettings('assistant');
    if (action === 'hide') await props.store.hide();
    if (action === 'new') await props.store.newConversation();
    if (action === 'chat') await props.store.openChat();
  } catch (error) { props.store.note(error instanceof Error ? error.message : String(error)); }
}
async function takeScreenshot() {
  if (!canAttach.value || props.store.attachments.length >= 8) return;
  props.store.prepareImages();
  const version = imageVersion;
  capturing.value = true;
  try {
    await nextTick();
    const image = await invoke<string | null>('assistant_screenshot');
    if (image && props.store.open && version === imageVersion) props.store.attachments.push(image);
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
// Toolbar visibility never changes native window geometry. Headroom is reserved on open.
let imageVersion = 0, dropDepth = 0;

const canAttach = computed(() => !busy.value && !props.store.ask && !props.store.approval && !imageReading.value && !submitting.value && !capturing.value && !props.store.changingSession);
watch(() => props.store.open, () => { imageVersion++; dropDepth = 0; dropActive.value = false; closeTools(); });
watch(() => props.store.changingSession, changing => { if (changing) imageVersion++; });
function openPicker() {
  if (!canAttach.value) return;
  props.store.prepareImages(); filePicker.value?.click();
}
async function addImages(files: File[]) {
  if (!files.length || !props.store.open) return;
  if (!canAttach.value) { props.store.note('请等待当前操作完成后再添加图片。'); return; }
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
const dragging=ref(false),copyState=ref<'idle'|'copying'|'copied'|'failed'>('idle');
let copyTimer:ReturnType<typeof setTimeout>|undefined,copyVersion=0;
const cardDismissed = ref(false);
watch(() => props.store.question, () => { cardDismissed.value = false;copyVersion++;copyState.value='idle';clearTimeout(copyTimer); });
watch(() => props.store.phase, (phase, before) => {
  if (phase === 'thinking' && !['thinking', 'answering'].includes(before)) cardDismissed.value = false;
});
watch(() => props.store.open, open => { if (open) cardDismissed.value = false; });
watch(() => [props.store.approval?.id, props.store.ask?.id, props.store.hint, props.store.failure], () => { cardDismissed.value = false; });

const busy = computed(() => ['thinking', 'answering', 'speaking'].includes(props.store.phase));
const hasCard = computed(() => !cardDismissed.value && Boolean(
  props.store.hint || props.store.question || props.store.questionImages?.length || props.store.questionFiles?.length || props.store.answer || props.store.working || props.store.failure
  || props.store.approval || props.store.ask || props.store.phase === 'thinking',
));
const canSend = computed(() => !busy.value && !props.store.approval && !imageReading.value && !submitting.value && !capturing.value && !props.store.changingSession && Boolean(props.store.input.trim() || props.store.attachments.length));

function approvalLine(request: ApprovalRequest) {
  const raw = typeof request.arguments === 'string' ? request.arguments : JSON.stringify(request.arguments);
  const text = `${request.tool} ${raw ?? ''}`.trim();
  return text.length > 180 ? `${text.slice(0, 180)}…` : text;
}
async function send() {
  const text = props.store.input.trim();
  if (!canSend.value) return;
  if (props.store.ask) { if (text) { props.store.input = ''; void props.store.reply(text); } return; }
  if (busy.value || props.store.approval) return;
  cardDismissed.value = false; submitting.value = true;
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
  if (event.isComposing || composing.value) return;
  event.preventDefault();
  send();
}
function primary() {
  if (canSend.value) { closeTools(); void send(); }
  else if (busy.value) { closeTools(); props.store.interrupt(); }
  else if (toolsOpen.value) closeTools();
  else { showTools(); toolsPinned.value = true; }
}
function onOrb() {
  if (props.store.phase === 'speaking') props.store.interrupt();
  else void props.store.toggleListening();
}
async function openLink(href: string) {
  try { await invoke('open_download_url', { url: href.split('#')[0] }); } catch { props.store.note('这个地址不能直接打开。'); }
}

let hitRaf=0,lastRegions='';
function syncHitRegions(){
  const regions:HitRegion[]=[];
  document.querySelectorAll<HTMLElement>('.composer,.card,.ball,.tools-toolbar,.selection-copy-menu').forEach(el=>{
    const radius=el.classList.contains('composer')?32:el.classList.contains('ball')?24:el.classList.contains('card')?18:el.classList.contains('selection-copy-menu')?9:16;
    const region=hitRegion(el.getBoundingClientRect(),window.innerHeight,radius);
    if(region)regions.push(region);
  });
  // Only a narrow corridor beneath the visible toolbar is interactive while crossing to it.
  if(toolsOpen.value&&toolbar.value&&primaryButton.value){
    const bar=toolbar.value.getBoundingClientRect(),button=primaryButton.value.getBoundingClientRect();
    if(bar.width&&button.width){const gap=hitRegion({left:button.left,bottom:button.top,width:button.width,height:Math.max(0,button.top-bar.bottom)},window.innerHeight,0,2);if(gap)regions.push(gap);}
  }
  const signature=JSON.stringify(regions);if(signature===lastRegions)return;lastRegions=signature;
  void invoke('assistant_set_regions',{regions}).catch(()=>{lastRegions='';});
}
function queueHitRegions(){cancelAnimationFrame(hitRaf);hitRaf=requestAnimationFrame(syncHitRegions);}
watch([toolsOpen,()=>props.store.open,()=>props.store.ball,hasCard],()=>void nextTick(queueHitRegions));

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
    // Keep enough space for the toolbar even before hovering. Measuring its animated
    // top caused a resize/reposition loop, so toolsOpen is deliberately absent here.
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
watch(() => props.store.open, open => {
  if (!open) return;
  fitted = 0;
  void nextTick(() => { fit(); field.value?.focus(); });
});
watch(() => props.store.answer, () => { void nextTick(() => { if (scroller.value) scroller.value.scrollTop = scroller.value.scrollHeight; }); });
watch(() => props.store.ask?.id, id => { if (id) void nextTick(() => { if (scroller.value) scroller.value.scrollTop = scroller.value.scrollHeight; }); });

async function copyAnswer(){
  const text=props.store.answer;if(!text.trim()||copyState.value==='copying')return;
  const version=++copyVersion;clearTimeout(copyTimer);copyState.value='copying';
  try{await navigator.clipboard.writeText(text);if(version===copyVersion)copyState.value='copied';}
  catch{if(version===copyVersion)copyState.value='failed';}
  finally{if(version===copyVersion)copyTimer=setTimeout(()=>copyState.value='idle',2000);}
}

// Only the orb and empty chrome are drag handles; text and controls retain native selection.
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
  void invoke('assistant_drag').catch(()=>{dragging.value=false;fit();});
}
function dragUp(event:PointerEvent){if(press?.id===event.pointerId)releasePress();}
function captureClick(event:MouseEvent){if(pulled&&event.detail!==0){event.preventDefault();event.stopImmediatePropagation();pulled=false;}}
function dragEnded(){dragging.value=false;releasePress();fit();}
function ballClick(){void props.store.unfold();}
const onFocus=()=>{if(props.store.open&&!dragging.value&&!press&&!morphing.value)field.value?.focus();};
onMounted(()=>{
  void listen<MorphEvent>('assistant-transition', event => { void handleMorph(event).catch(() => resetMorph()); }).then(off => { if (morphDisposed) off(); else offMorph = off; });
  void nextTick(queueHitRegions);window.addEventListener('resize',queueHitRegions);
  document.addEventListener('pointerdown',outsideTools);document.addEventListener('keydown',onAskShortcut);window.addEventListener('blur',closeTools);window.addEventListener('focus',onFocus);window.addEventListener('pointermove',dragMove);window.addEventListener('pointerup',dragUp);window.addEventListener('pointercancel',dragUp);window.addEventListener('blur',releasePress);window.addEventListener('assistant-drag-end',dragEnded);
});
onBeforeUnmount(()=>{
  morphDisposed = true; clearTimeout(morphFallback); offMorph?.(); morphAnimations.splice(0).forEach(a => a.cancel());
  cancelAnimationFrame(hitRaf);window.removeEventListener('resize',queueHitRegions);
  clearToolTimers();window.removeEventListener('blur',closeTools);document.removeEventListener('pointerdown',outsideTools);document.removeEventListener('keydown',onAskShortcut);imageVersion++; releasePress();copyVersion++;clearTimeout(copyTimer);window.removeEventListener('focus',onFocus);window.removeEventListener('pointermove',dragMove);window.removeEventListener('pointerup',dragUp);window.removeEventListener('pointercancel',dragUp);window.removeEventListener('blur',releasePress);window.removeEventListener('assistant-drag-end',dragEnded);observer?.disconnect();cancelAnimationFrame(raf);
});
</script>

<template>
  <div class="stage" :class="{ light: !isDark,dragging,morphing }" @dragover="dragOver" @drop="dropImages" @keydown.esc.prevent.stop="escape" @pointerdown="dragDown" @click.capture="captureClick">
    <div v-if="store.ball && !store.open" class="ball-stage" data-testid="assistant-ball-stage">
      <button type="button" class="ball" data-phase="idle" title="点一下提问，按住可以拖动，右键打开设置" aria-label="打开超级小伴" data-testid="assistant-ball" data-drag-handle @click="ballClick" @contextmenu.prevent="store.openSettings('assistant')">
        <Orb phase="idle" :active="store.ball && !store.open" :size="38" />
      </button>
    </div>
    <Transition name="pop" appear @after-enter="queueHitRegions" @after-leave="queueHitRegions">
      <div v-if="store.open" ref="inner" class="inner" role="dialog" aria-label="超级小伴" data-testid="assistant">
        <Transition name="rise" @after-enter="queueHitRegions" @after-leave="queueHitRegions">
          <section v-if="hasCard" class="card" data-testid="assistant-card">
            <button type="button" class="round card-close" title="暂时关闭对话框" aria-label="暂时关闭对话框" data-testid="assistant-card-close" @click="cardDismissed = true"><X :size="15" /></button>
            <div ref="scroller" class="card-body">
              <p v-if="store.hint" class="hint" data-testid="assistant-hint">
                {{ store.hint }}
                <button v-if="store.hintTab" type="button" class="link" @click="store.openSettings(store.hintTab)">{{ store.hintTab === 'ai' ? '去配置' : '去设置' }}</button>
              </p>
              <div v-if="store.questionFiles?.length" class="sent-files"><span v-for="(file, n) in store.questionFiles" :key="n" class="file-chip" :title="file.name">📄 {{ file.name }}</span></div>
              <div v-if="store.questionImages?.length" class="sent-images"><img v-for="(image, index) in store.questionImages" :key="index" :src="image" :alt="`已发送的图片 ${index + 1}`" draggable="false" /></div>
              <p v-if="store.question" class="you">{{ store.question }}</p>
              <div v-if="store.answer" class="reply"><MarkdownView :source="store.answer" @open="openLink" /><span v-if="store.phase === 'answering'" class="cursor" aria-hidden="true"></span></div>
              <p v-else-if="store.failure" class="failure">{{ store.failure }}</p>
              <p v-else-if="store.phase === 'thinking' || store.working" class="working"><span class="dots" aria-hidden="true"><i></i><i></i><i></i></span><template v-if="store.working">{{ store.working }}…</template></p>
              <p v-if="store.answer && store.failure" class="failure">{{ store.failure }}</p>
              <div v-if="store.approval" class="gate" data-testid="assistant-approval">
                <p class="gate-title">{{ store.approval.danger ? `这个操作有风险：${store.approval.danger}` : '这一步需要你许可' }}</p>
                <p class="gate-tool">{{ approvalLine(store.approval) }}</p>
                <div class="gate-actions"><button type="button" class="ghost" @click="store.decide('deny')">拒绝</button><button type="button" class="solid" @click="store.decide('once')">允许这一次</button></div>
              </div>
              <div v-if="store.ask" class="ask-user" data-testid="assistant-question" role="group" :aria-label="store.ask.question">
                <div class="ask-user-overline">
                  <span class="ask-user-tag"><span class="ask-user-mark" aria-hidden="true">✦</span>需要你的选择</span>
                  <span class="ask-user-type">{{ store.ask.options.length ? '单选问题' : '自由回答' }}</span>
                </div>
                <p class="ask-user-question">{{ store.ask.question }}</p>
                <div v-if="store.ask.options.length" class="ask-user-options" role="group" aria-label="回答选项">
                  <button v-for="(option, index) in store.ask.options" :key="`${index}-${option}`" type="button" class="ask-user-option" @click="store.reply(option)">
                    <span class="ask-user-index" aria-hidden="true">{{ index < 26 ? 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'[index] : index + 1 }}</span>
                    <span class="ask-user-option-text">{{ option }}</span>
                    <ChevronRight class="ask-user-option-arrow" :size="15" :stroke-width="1.8" aria-hidden="true" />
                  </button>
                </div>
                <p v-else class="ask-user-help">可以直接在下方输入你的回答。</p>
                <div class="ask-user-footer">
                  <span v-if="store.ask.options.length" class="ask-user-tip">按 <kbd>1–{{ Math.min(9, store.ask.options.length) }}</kbd> 快捷选择 · 或在下方输入</span>
                  <span v-else class="ask-user-tip">支持自由输入回答</span>
                  <button type="button" class="ask-user-skip" @click="store.reply(null)">跳过此问题 <span aria-hidden="true">→</span></button>
                </div>
              </div>
            </div>
            <footer v-if="store.answer.trim()" class="card-footer"><button type="button" class="copy-answer" data-testid="assistant-copy" :disabled="copyState==='copying'" :aria-label="copyState==='copied'?'已复制回复':copyState==='failed'?'复制失败，点击重试':'复制回复'" title="复制 AI 回复" @pointerdown.prevent @click="copyAnswer"><Check v-if="copyState==='copied'" :size="14"/><Copy v-else :size="14"/><span aria-live="polite">{{ copyState==='copied'?'已复制':copyState==='failed'?'复制失败，重试':copyState==='copying'?'复制中…':'复制' }}</span></button></footer>
          </section>
        </Transition>
        <div class="composer" :class="{ 'drop-active': dropActive }" @dragenter="dragEnter" @dragleave="dragLeave" @dragover="dragOver" @drop.stop="dropImages" @paste="pasteImages">
        <input ref="filePicker" class="sr" type="file" :accept="ATTACHMENT_ACCEPT" multiple tabindex="-1" aria-label="选择要发送的附件" data-testid="assistant-image-input" @change="onPick" />
        <div v-if="store.attachments.length" class="image-tray" data-no-drag data-testid="assistant-attachments">
          <div v-for="(image, index) in store.attachments" :key="index" class="image-thumb"><img v-if="attachmentImage(image)" :src="attachmentImage(image)!" :alt="`待发送的图片 ${index + 1}`" draggable="false" /><span v-if="!attachmentImage(image)" class="file-chip" :title="attachmentName(image)">📄 {{ attachmentName(image) }}</span><button type="button" :aria-label="`移除附件 ${index + 1}`" :disabled="submitting || imageReading" @click="store.attachments.splice(index, 1)"><X :size="11" /></button></div>
        </div>
        <div class="capsule" :data-phase="store.phase">
          <button type="button" class="orb-button" :title="store.phase === 'speaking' ? '停止朗读' : store.phase === 'listening' ? '暂停聆听 · 拖动可移动' : '点一下开始说话 · 拖动可移动'" :aria-label="store.phase === 'speaking' ? '停止朗读' : store.phase === 'listening' ? '暂停聆听' : '开始说话'" data-testid="assistant-orb" :disabled="imageReading || submitting || capturing" data-drag-handle @click="onOrb">
            <Orb :phase="store.phase" :level="asr.level" :active="store.open" :size="34" :dark="isDark" />
          </button>
          <input ref="field" v-model="store.input" :disabled="store.changingSession" class="field" type="text" autocomplete="off" spellcheck="false" :placeholder="store.status" aria-label="向小伴提问" data-testid="assistant-input" @keydown.enter="onEnter" @compositionstart="composing = true" @compositionend="composing = false" />
          <div ref="toolsHost" class="tools-host" data-no-drag @pointerenter="enterTools" @pointerleave="leaveTools" @focusin="focusTools" @focusout="toolsFocusOut">
            <div v-if="toolsOpen" ref="toolbar" id="assistant-toolbar" class="tools-toolbar" role="toolbar" aria-label="小伴工具" aria-orientation="horizontal" data-testid="assistant-toolbar" @animationend="queueHitRegions" @keydown="toolbarKeys" @pointerenter="focusTools" @pointerleave="leaveTools">
              <button type="button" class="tool-button" title="去超级小伴设置页" aria-label="去超级小伴设置页" data-testid="assistant-settings" @click="toolAction('settings')"><Settings :size="16" :stroke-width="1.7" /></button>
              <button type="button" class="tool-button" title="收起小伴" aria-label="收起小伴" data-testid="assistant-close" @click="toolAction('hide')"><ChevronDown :size="16" :stroke-width="1.7" /></button>
              <button type="button" class="tool-button" title="新会话" aria-label="新会话" data-testid="assistant-new-chat" :disabled="imageReading || submitting || capturing || store.changingSession" @click="toolAction('new')"><MessageSquarePlus :size="16" :stroke-width="1.7" /></button>
              <button type="button" class="tool-button" title="去聊天详情页" aria-label="去聊天详情页" data-testid="assistant-expand" @click="toolAction('chat')"><MessageSquare :size="16" :stroke-width="1.7" /></button>
              <button type="button" class="tool-button" title="添加附件" aria-label="添加附件" data-testid="assistant-add-image" :disabled="!canAttach || store.attachments.length >= 8" @click="toolAction('image')"><ImagePlus :size="16" :stroke-width="1.7" /></button>
              <button type="button" class="tool-button" title="截图" aria-label="截图" data-testid="assistant-screenshot" :disabled="!canAttach || store.attachments.length >= 8" @click="toolAction('screenshot')"><ScanLine :size="16" :stroke-width="1.7" /></button>
            </div>
            <button ref="primaryButton" type="button" class="round main-action" :class="{ 'has-action': canSend, 'is-stop': busy, 'tools-open': toolsOpen && !canSend && !busy }" :aria-label="busy ? '停止' : canSend ? '发送' : toolsOpen ? '收起工具' : '展开工具'" :title="busy ? '停止回复 · 悬停展开工具' : canSend ? '发送 · 悬停展开工具' : '展开工具'" :aria-expanded="toolsOpen" aria-controls="assistant-toolbar" aria-keyshortcuts="ArrowUp ArrowDown" :disabled="imageReading || capturing || (submitting && !busy)" data-testid="assistant-primary" @keydown="primaryKeys" @click="primary"><Square v-if="busy" :size="12" fill="currentColor" :stroke-width="0" /><ArrowUp v-else-if="canSend" :size="18" /><Plus v-else :size="18" /></button>
          </div>
        </div>
        <p v-if="dropActive || imageReading || capturing" class="image-status" aria-live="polite">{{ capturing ? '截图中，按 Esc 可以取消' : imageReading ? '正在读取附件…' : '松开添加附件' }}</p>
        </div>
        <span class="sr" aria-live="polite">{{ store.status }}</span>
      </div>
    </Transition>
    <div v-if="morphVisible" ref="morphShell" class="morph-shell" :style="morphStyle" aria-hidden="true"><Orb phase="idle" :size="38" flowing /></div>
    <SelectionCopyMenu :scope="scroller" @visibility-change="queueHitRegions" />
  </div>
</template>

<style>
html,body,#app{background:transparent!important}
</style>
<style scoped>
.file-chip{display:inline-block;max-width:180px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:12px;padding:10px 14px;border-radius:8px;background:var(--button)}.image-thumb:has(.file-chip){width:auto;max-width:190px}.sent-files{display:flex;gap:6px;flex-wrap:wrap;margin-bottom:8px}
.stage{--card:#292C31;--capsule:linear-gradient(175deg,#3E4147 0%,#35383E 48%,#2F3238 100%);--line:#A6B5CB22;--text:#DFE3E9;--text-soft:#878E99;--working:#A4AEBD;--placeholder:#919CAB;--field:#DFE3E9;--button:#353A43;--button-hover:#434750;--button-fg:#BAC4D1;--button-fg-hover:#DFE3E9;--code:#FFFFFF0B;--link:#B0C4D7;--hint:#D7BE92;--failure:#EEA5AD;--gate-bg:#C6AE7D0F;--gate-line:#C6AE7D30;--gate-title:#DDC8A5;--gate-tool:#BCC4CF;--ghost:#FFFFFF0D;--ghost-fg:#DFE3E9;--scroll:#FFFFFF25;--dot:#A8B6C6;--listen:#B2C8EA50;--busy:#A6B5CB30;--attention:#C6AE7D60;--panel-shadow:0 5px 14px #00000014,0 1px 2px #00000016;--capsule-shadow:inset 0 1px 0 #E8F0FF15,inset 0 -1px 0 #11182126,0 8px 25px #00000025;--stop:#484E56;--stop-fg:#B1BBC7;--stop-hover:#535B65;--stop-fg-hover:#CCD5DF;--ui-line:var(--line);--ui-surface-alt:var(--code);--ui-text-primary:var(--text);--ui-text-secondary:var(--working);--ui-text-tertiary:var(--text-soft);--ui-primary:var(--link);position:relative;display:flex;flex-direction:column;justify-content:flex-end;height:100vh;color:var(--text);font:14px/1.6 -apple-system,BlinkMacSystemFont,"PingFang SC","Segoe UI","Microsoft YaHei UI",sans-serif;-webkit-font-smoothing:antialiased;user-select:none;cursor:default;pointer-events:none;color-scheme:dark}
.stage.light{--card:#FFFFFF;--capsule:linear-gradient(175deg,#FEFFFF 0%,#F3F6F9 45%,#EAF0F5 100%);--line:#D5DDE6;--text:#414851;--text-soft:#929BA7;--working:#718092;--placeholder:#A0A9B4;--field:#414851;--button:#EAEFF4;--button-hover:#DFE6ED;--button-fg:#657181;--button-fg-hover:#414851;--code:#71849A0D;--link:#657F95;--hint:#947445;--failure:#B95264;--gate-bg:#B9944B0B;--gate-line:#B9944B30;--gate-title:#886B3E;--gate-tool:#657181;--ghost:#71849A0D;--ghost-fg:#414851;--scroll:#71849A35;--dot:#899BAE;--listen:#B3C2D3;--busy:#C4CFDA;--attention:#C6AE7D;--panel-shadow:0 3px 12px #68788A07,0 1px 2px #58677805;--capsule-shadow:inset 0 1px 0 #FFFFFFF2,inset 0 -1px 0 #8F9DAC0C,0 7px 24px #71849A12;--stop:#E6EBEF;--stop-fg:#778491;--stop-hover:#DDE4EA;--stop-fg-hover:#5F7080;color-scheme:light}
.ball-stage{display:grid;place-items:center;height:100vh}
.ball{pointer-events:auto;touch-action:none;display:grid;place-items:center;width:48px;height:48px;padding:0;border-radius:50%;background:var(--capsule);box-shadow:var(--capsule-shadow);border:1px solid var(--line);cursor:pointer;transition:border-color .3s,transform .2s}
.ball:hover{transform:scale(1.06)}
.ball:active{transform:scale(.96)}
.ball:focus-visible{outline:2px solid var(--link);outline-offset:2px}
.ball[data-phase=listening]{border-color:var(--listen)}
.ball[data-phase=thinking],.ball[data-phase=answering],.ball[data-phase=speaking]{border-color:var(--busy)}
.ball[data-phase=attention]{border-color:var(--attention)}
.inner{pointer-events:none;display:flex;flex-direction:column;gap:12px;padding:12px}
.card{pointer-events:auto;cursor:grab;position:relative;border-radius:18px;background:var(--card);border:1px solid var(--line);box-shadow:var(--panel-shadow);overflow:hidden}
.card .card-close{position:absolute;z-index:1;top:8px;right:8px;width:28px;height:28px}
.card-body{cursor:auto;max-height:390px;overflow-y:auto;padding:16px 44px 18px 20px;user-select:text;scrollbar-width:thin;scrollbar-color:var(--scroll) transparent}
.you{margin:0 0 8px;font-size:12.5px;color:var(--text-soft);display:-webkit-box;-webkit-line-clamp:2;line-clamp:2;-webkit-box-orient:vertical;overflow:hidden}
.reply{font-size:15px;line-height:1.75;color:var(--text);overflow-wrap:anywhere}
.reply :deep(p){margin:0 0 .7em}
.reply :deep(p:last-child){margin-bottom:0}
.reply :deep(.md-code){background:var(--code);border-radius:5px;padding:1px 6px;font-size:.88em;font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,monospace}
.reply :deep(.md-link){color:var(--link);cursor:pointer;text-decoration:underline;text-underline-offset:2px}
.reply :deep(.md-heading){margin:1em 0 .4em;font-size:15px}
.reply :deep(ul),.reply :deep(ol){margin:.3em 0 .7em;padding-left:1.4em}
.cursor{display:inline-block;width:7px;height:15px;margin-left:3px;vertical-align:-2px;border-radius:2px;background:var(--link);animation:blink 1s steps(2,start) infinite}
.working{margin:0;display:flex;align-items:center;gap:10px;color:var(--working);font-size:13.5px}
.dots{display:inline-flex;gap:4px}
.dots i{width:6px;height:6px;border-radius:50%;background:var(--dot);animation:dot 1.2s ease-in-out infinite}
.dots i:nth-child(2){animation-delay:.15s}.dots i:nth-child(3){animation-delay:.3s}
.hint{margin:0 0 10px;color:var(--hint);font-size:13.5px}
.failure{margin:0;color:var(--failure);font-size:13.5px}
.link{margin-left:8px;padding:0;border:0;background:none;color:var(--link);font:inherit;text-decoration:underline;text-underline-offset:2px;cursor:pointer}
.gate{margin-top:14px;padding:14px;border-radius:14px;background:var(--gate-bg);border:1px solid var(--gate-line)}
.gate-title{margin:0 0 6px;font-size:13.5px;color:var(--gate-title)}
.gate-tool{margin:0 0 12px;font:12px/1.5 ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;color:var(--gate-tool);overflow-wrap:anywhere}
.gate-actions{display:flex;gap:8px;justify-content:flex-end}
.gate-actions.wrap{flex-wrap:wrap;justify-content:flex-start}
.gate button{height:32px;padding:0 14px;border-radius:16px;border:1px solid transparent;font:inherit;font-size:13px;cursor:pointer}
.gate .ghost{background:var(--ghost);color:var(--ghost-fg)}
.gate .ghost.dim{color:var(--text-soft)}
.gate .solid{background:var(--button);color:var(--text);font-weight:600}
/* Isolated Ask User styling: the approval gate and composer retain their existing appearance. */
.stage{--ask-accent:#EC9DA3;--ask-accent-bg:#4D383E;--ask-divider:#FFFFFF1E;--ask-border:#40454D;--ask-row:#292C31;--ask-hover:#3C4149;--ask-key-bg:#34383F;--ask-key-border:#4A5057;--ask-key-fg:#9EA7AD;--ask-arrow:#8D969E}
.stage.light{--ask-accent:#D75D64;--ask-accent-bg:#FAEDED;--ask-divider:#E8EAE9;--ask-border:#E3E6E4;--ask-row:#FFFFFF;--ask-hover:#F7F8F7;--ask-key-bg:#F7F8F7;--ask-key-border:#E4E7E5;--ask-key-fg:#929A9A;--ask-arrow:#B7BEBC}
.ask-user{margin-top:16px;padding-top:16px;border-top:1px solid var(--ask-divider)}
.ask-user:first-child{margin-top:0;padding-top:0;border-top:0}
.ask-user-overline{display:flex;align-items:center;justify-content:space-between;gap:12px;margin-bottom:9px;font-size:11px}
.ask-user-tag{display:inline-flex;align-items:center;gap:6px;color:var(--ask-accent);font-weight:650}
.ask-user-mark{display:grid;place-items:center;width:15px;height:15px;border-radius:5px;background:var(--ask-accent-bg);color:var(--ask-accent);font-size:11px;line-height:1}
.ask-user-type{font-size:10px;color:var(--text-soft)}
.ask-user-question{margin:0 0 15px;color:var(--text);font-size:15px;font-weight:650;line-height:1.5;letter-spacing:-.02em;overflow-wrap:anywhere}
.ask-user-options{display:flex;flex-direction:column;gap:7px}
.ask-user-option{box-sizing:border-box;display:flex;align-items:center;gap:11px;width:100%;min-height:46px;padding:10px 11px;text-align:left;border:1px solid var(--ask-border);border-radius:11px;background:var(--ask-row);color:var(--text);font:inherit;cursor:pointer;transition:background .15s,border-color .15s}
.ask-user-option:hover{background:var(--ask-hover);border-color:var(--ask-accent)}
.ask-user-option:active{background:var(--ask-accent-bg)}
.ask-user-option:focus-visible{outline:2px solid var(--ask-accent);outline-offset:2px}
.ask-user-index{box-sizing:border-box;display:grid;place-items:center;flex:none;width:24px;height:24px;border:1px solid var(--ask-key-border);border-radius:7px;background:var(--ask-key-bg);color:var(--ask-key-fg);font-size:11px;font-weight:700}
.ask-user-option:hover .ask-user-index{border-color:var(--ask-accent);color:var(--ask-accent)}
.ask-user-option-text{flex:1;min-width:0;overflow-wrap:anywhere;font-size:12.8px;font-weight:550;line-height:1.45}
.ask-user-option-arrow{flex:none;color:var(--ask-arrow);transition:color .15s,transform .15s}
.ask-user-option:hover .ask-user-option-arrow{color:var(--ask-accent);transform:translateX(2px)}
.ask-user-help{margin:0;color:var(--working);font-size:12.5px;line-height:1.75}
.ask-user-footer{display:flex;align-items:center;justify-content:space-between;gap:12px;min-height:22px;margin-top:13px}
.ask-user-tip{color:var(--text-soft);font-size:10.5px;line-height:1.5}
.ask-user-tip kbd{padding:0 4px;border:1px solid var(--ask-border);border-radius:4px;font:inherit;font-size:10px}
.ask-user-skip{flex:none;padding:4px 0 4px 8px;border:0;border-radius:5px;background:transparent;color:var(--text-soft);font:inherit;font-size:11px;cursor:pointer}
.ask-user-skip:hover{color:var(--text)}
.ask-user-skip:focus-visible{outline:2px solid var(--ask-accent);outline-offset:2px}
@media(prefers-reduced-motion:reduce){.ask-user-option,.ask-user-option-arrow{transition:none}}
.composer{pointer-events:auto;cursor:grab;position:relative;border-radius:32px;background:var(--capsule);border:1px solid var(--line);box-shadow:var(--capsule-shadow);transition:border-color .2s}
.composer.drop-active{border-color:var(--link);box-shadow:inset 0 0 0 1px var(--link)}
.tools-host{position:relative;flex:none;cursor:default}
.tools-toolbar{position:absolute;right:-3px;bottom:calc(100% + 9px);display:flex;align-items:center;gap:4px;padding:6px;border-radius:16px;background:var(--card);border:1px solid var(--line);box-shadow:var(--panel-shadow);z-index:5;animation:toolbar-in .16s ease-out;cursor:default}
.tools-toolbar::after{content:"";position:absolute;top:100%;left:0;right:0;height:12px}
.tool-button{display:grid;place-items:center;width:32px;height:32px;flex:none;padding:0;border:0;border-radius:10px;background:transparent;color:var(--button-fg);cursor:pointer;transition:background .15s,color .15s}
.tool-button:hover:not(:disabled){background:var(--button-hover);color:var(--button-fg-hover)}
.tool-button:focus-visible{outline:2px solid var(--link);outline-offset:1px}
.tool-button:disabled{opacity:.35;cursor:default}
.round.main-action{background:var(--button);color:var(--button-fg)}
.round.main-action.has-action{background:var(--text);color:var(--card)}
.round.main-action.is-stop{background:var(--stop);color:var(--stop-fg);box-shadow:inset 0 0 0 1px #8593A00D}
.round.main-action.is-stop:hover{background:var(--stop-hover);color:var(--stop-fg-hover)}
.main-action.is-stop :deep(svg){width:12px;height:12px;fill:currentColor;stroke:none}
.main-action svg{transition:transform .18s}
.main-action.tools-open svg{transform:rotate(45deg)}
.main-action:disabled{opacity:.45;cursor:default}
@keyframes toolbar-in{from{opacity:0;transform:translateY(4px)}to{opacity:1;transform:translateY(0)}}
.image-tray{display:flex;gap:8px;flex-wrap:wrap;padding:12px 16px 2px;cursor:default}
.image-thumb{position:relative;width:44px;height:44px;flex:none}
.image-thumb img,.sent-images img{width:44px;height:44px;object-fit:cover;border-radius:8px;background:var(--button)}
.image-thumb button{position:absolute;right:-4px;top:-4px;width:17px;height:17px;box-sizing:border-box;padding:0;line-height:1;display:grid;place-items:center;border:1px solid var(--line);border-radius:50%;background:var(--card);color:var(--text);cursor:pointer}
.image-thumb button :deep(svg){display:block;width:11px;height:11px;flex-shrink:0}
.image-thumb button:focus-visible{outline:2px solid var(--link)}
.sent-images{display:flex;gap:6px;flex-wrap:wrap;margin-bottom:8px}
.image-status{font-size:11px;color:var(--link);margin:0;padding:0 20px 8px;cursor:default}
.capsule{display:flex;align-items:center;gap:8px;height:64px;padding:0 15px 0 13px;border-radius:34px;background:transparent;border:0;transition:border-color .3s}
.composer:has(.capsule[data-phase=listening]){border-color:var(--listen)}
.composer:has(.capsule[data-phase=thinking]),.composer:has(.capsule[data-phase=answering]),.composer:has(.capsule[data-phase=speaking]){border-color:var(--busy)}
.composer:has(.capsule[data-phase=attention]){border-color:var(--attention)}
.round,.gate button,.link{cursor:pointer}.orb-button,.ball{cursor:grab}.stage.dragging,.stage.dragging *{cursor:grabbing!important}
.orb-button{touch-action:none;flex:none;width:34px;height:34px;padding:0;border:0;border-radius:50%;background:none;cursor:pointer}
.orb-button:focus-visible,.round:focus-visible{outline:2px solid var(--link);outline-offset:2px}
.field{cursor:text;flex:1;min-width:0;height:44px;padding:0 4px;border:0;outline:0;background:transparent;color:var(--field);font:inherit;font-size:14px;caret-color:var(--field);user-select:text}
.field:focus,.field:focus-visible{outline:none;box-shadow:none}
.field::placeholder{color:var(--placeholder)}
.round{flex:none;width:36px;height:36px;display:grid;place-items:center;padding:0;border:0;border-radius:50%;background:var(--button);color:var(--button-fg);cursor:pointer;transition:background .2s,color .2s}
.round:hover{background:var(--button-hover);color:var(--button-fg-hover)}
.round.primary{background:linear-gradient(135deg,#5ad7ff,#7a86ff);color:#06101f}
.round.primary:hover{filter:brightness(1.1)}
.sr{position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0);white-space:nowrap}
.pop-enter-active{transition:opacity .2s ease,transform .28s cubic-bezier(.2,.9,.3,1.12)}
.pop-enter-from{opacity:0;transform:translateY(14px) scale(.97)}
.rise-enter-active{transition:opacity .22s ease,transform .26s ease}
.rise-enter-from{opacity:0;transform:translateY(8px)}
@keyframes blink{to{visibility:hidden}}
@keyframes dot{0%,80%,100%{opacity:.3;transform:translateY(0)}40%{opacity:1;transform:translateY(-3px)}}
@media(prefers-reduced-motion:reduce){.pop-enter-active,.rise-enter-active{transition:none}.tools-toolbar{animation:none}.main-action svg{transition:none}.dots i,.cursor{animation:none}}
.morph-shell{position:fixed;z-index:20;display:flex;align-items:center;justify-content:center;overflow:hidden;pointer-events:none;border-radius:32px;border:1px solid var(--line);background:var(--capsule);box-shadow:var(--capsule-shadow)}
.morph-shell :deep(.orb-frame){position:absolute;left:5px}
.morphing .composer{opacity:0;pointer-events:none}
.morphing .card,.morphing .ball{opacity:0;pointer-events:none}
.morphing .inner{transform:none!important;transition:none!important}
.morphing .pop-enter-from{opacity:1}
.morphing .tools-toolbar{display:none}
</style>


<style scoped>
.card-footer{display:flex;justify-content:flex-end;padding:0 12px 9px}.copy-answer{display:inline-flex;align-items:center;gap:5px;padding:4px 7px;border:0;border-radius:7px;background:transparent;color:var(--text-soft);font:inherit;font-size:11px;line-height:1.5;cursor:pointer}.copy-answer:hover{background:var(--button-hover);color:var(--text)}.copy-answer:focus-visible{outline:2px solid var(--link);outline-offset:1px}.copy-answer:disabled{opacity:.5;cursor:default}
</style>

