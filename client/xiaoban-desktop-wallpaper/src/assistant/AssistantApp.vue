<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { ArrowUp, Square, X, Copy, Check, ImagePlus, Settings, ChevronDown, MessageSquare, ScanLine, Plus } from 'lucide-vue-next';
import MarkdownView from '../ui/chat/MarkdownView.ts';
import type { AsrStore } from '../ui/asr/store.ts';
import type { ApprovalRequest } from '../ui/chat/types.ts';
import { invoke } from '../bridge.ts';
import { isDark } from '../ui/appearance.ts';
import Orb from './Orb.vue';
import type { AssistantStore } from './assistant-store.ts';

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
async function toolAction(action: 'settings' | 'hide' | 'chat' | 'image' | 'screenshot') {
  closeTools();
  if (action === 'image') { openPicker(); return; }
  if (action === 'screenshot') { await takeScreenshot(); return; }
  try {
    if (action === 'settings') await props.store.openSettings('assistant');
    if (action === 'hide') await props.store.hide();
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
  } catch (error) { if (version === imageVersion) props.store.note(error instanceof Error ? error.message : String(error)); }
  finally { capturing.value = false; void nextTick(() => field.value?.focus()); }
}
// Toolbar visibility never changes native window geometry. Headroom is reserved on open.
let imageVersion = 0, dropDepth = 0;
const imageTypes = ['image/png', 'image/jpeg', 'image/gif', 'image/webp'];
const canAttach = computed(() => !busy.value && !props.store.ask && !props.store.approval && !imageReading.value && !submitting.value && !capturing.value);
watch(() => props.store.open, () => { imageVersion++; dropDepth = 0; dropActive.value = false; closeTools(); });
function openPicker() {
  if (!canAttach.value) return;
  props.store.prepareImages(); filePicker.value?.click();
}
function readImage(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => typeof reader.result === 'string' ? resolve(reader.result) : reject(Error('图片读取失败，请重试。'));
    reader.onerror = reader.onabort = () => reject(Error('图片读取失败，请重试。'));
    reader.readAsDataURL(file);
  });
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
      if (props.store.attachments.length >= 8) { warning = '一次最多添加 8 张图片。'; break; }
      if (!imageTypes.includes(file.type)) { warning = '支持 PNG、JPEG、GIF 和 WebP 图片。'; continue; }
      if (file.size > 3 * 1024 * 1024) { warning = '单张图片需小于 3MB，请换一张。'; continue; }
      try {
        const image = await readImage(file);
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
  const files = Array.from(event.clipboardData?.files ?? []).filter(file => file.type.startsWith('image/'));
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
const quiet = computed(() => props.store.phase === 'idle');

const busy = computed(() => ['thinking', 'answering', 'speaking'].includes(props.store.phase));
const hasCard = computed(() => !cardDismissed.value && Boolean(
  props.store.hint || props.store.question || props.store.questionImages?.length || props.store.answer || props.store.working || props.store.failure
  || props.store.approval || props.store.ask || props.store.phase === 'thinking',
));
const canSend = computed(() => !busy.value && !props.store.approval && !imageReading.value && !submitting.value && !capturing.value && Boolean(props.store.input.trim() || props.store.attachments.length));

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

let fitted = 0, raf = 0;
let observer: ResizeObserver | undefined;
function fit() {
  cancelAnimationFrame(raf);
  if(dragging.value || capturing.value)return;
  raf = requestAnimationFrame(() => {
    if(dragging.value || capturing.value)return;
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
const onFocus=()=>{if(props.store.open&&!dragging.value&&!press)field.value?.focus();};
onMounted(()=>{
  document.addEventListener('pointerdown',outsideTools);window.addEventListener('blur',closeTools);window.addEventListener('focus',onFocus);window.addEventListener('pointermove',dragMove);window.addEventListener('pointerup',dragUp);window.addEventListener('pointercancel',dragUp);window.addEventListener('blur',releasePress);window.addEventListener('assistant-drag-end',dragEnded);
});
onBeforeUnmount(()=>{
  clearToolTimers();window.removeEventListener('blur',closeTools);document.removeEventListener('pointerdown',outsideTools);imageVersion++; releasePress();copyVersion++;clearTimeout(copyTimer);window.removeEventListener('focus',onFocus);window.removeEventListener('pointermove',dragMove);window.removeEventListener('pointerup',dragUp);window.removeEventListener('pointercancel',dragUp);window.removeEventListener('blur',releasePress);window.removeEventListener('assistant-drag-end',dragEnded);observer?.disconnect();cancelAnimationFrame(raf);
});
</script>

<template>
  <div class="stage" :class="{ light: !isDark,dragging }" @dragover="dragOver" @drop="dropImages" @keydown.esc.prevent.stop="escape" @pointerdown="dragDown" @click.capture="captureClick">
    <div v-if="store.ball && !store.open" class="ball-stage" data-testid="assistant-ball-stage">
      <button type="button" class="ball" :data-phase="store.phase" title="点一下提问，按住可以拖动，右键打开设置" aria-label="打开超级小伴" data-testid="assistant-ball" data-drag-handle @click="ballClick" @contextmenu.prevent="store.openSettings('assistant')">
        <Orb :phase="store.phase" :level="0" :active="true" :size="48" :dark="isDark" :fps="quiet ? 20 : 60" />
      </button>
    </div>
    <Transition name="pop" appear>
      <div v-if="store.open" ref="inner" class="inner" role="dialog" aria-label="超级小伴" data-testid="assistant">
        <Transition name="rise">
          <section v-if="hasCard" class="card" data-testid="assistant-card">
            <button type="button" class="round card-close" title="暂时关闭对话框" aria-label="暂时关闭对话框" data-testid="assistant-card-close" @click="cardDismissed = true"><X :size="15" /></button>
            <div ref="scroller" class="card-body">
              <p v-if="store.hint" class="hint" data-testid="assistant-hint">
                {{ store.hint }}
                <button v-if="store.hintTab" type="button" class="link" @click="store.openSettings(store.hintTab)">{{ store.hintTab === 'ai' ? '去配置' : '去设置' }}</button>
              </p>
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
              <div v-if="store.ask" class="gate" data-testid="assistant-question">
                <p class="gate-title">{{ store.ask.question }}</p>
                <div class="gate-actions wrap"><button v-for="option in store.ask.options" :key="option" type="button" class="ghost" @click="store.reply(option)">{{ option }}</button><button type="button" class="ghost dim" @click="store.reply(null)">跳过</button></div>
              </div>
            </div>
            <footer v-if="store.answer.trim()" class="card-footer"><button type="button" class="copy-answer" data-testid="assistant-copy" :disabled="copyState==='copying'" :aria-label="copyState==='copied'?'已复制回复':copyState==='failed'?'复制失败，点击重试':'复制回复'" title="复制 AI 回复" @pointerdown.prevent @click="copyAnswer"><Check v-if="copyState==='copied'" :size="14"/><Copy v-else :size="14"/><span aria-live="polite">{{ copyState==='copied'?'已复制':copyState==='failed'?'复制失败，重试':copyState==='copying'?'复制中…':'复制' }}</span></button></footer>
          </section>
        </Transition>
        <div class="composer" :class="{ 'drop-active': dropActive }" @dragenter="dragEnter" @dragleave="dragLeave" @dragover="dragOver" @drop.stop="dropImages" @paste="pasteImages">
        <input ref="filePicker" class="sr" type="file" accept="image/png,image/jpeg,image/gif,image/webp" multiple tabindex="-1" aria-label="选择要发送的图片" data-testid="assistant-image-input" @change="onPick" />
        <div v-if="store.attachments.length" class="image-tray" data-no-drag data-testid="assistant-attachments">
          <div v-for="(image, index) in store.attachments" :key="index" class="image-thumb"><img :src="image" :alt="`待发送的图片 ${index + 1}`" draggable="false" /><button type="button" :aria-label="`移除图片 ${index + 1}`" :disabled="submitting || imageReading" @click="store.attachments.splice(index, 1)"><X :size="11" /></button></div>
        </div>
        <div class="capsule" :data-phase="store.phase">
          <button type="button" class="orb-button" :title="store.phase === 'speaking' ? '停止朗读' : store.phase === 'listening' ? '暂停聆听 · 拖动可移动' : '点一下开始说话 · 拖动可移动'" :aria-label="store.phase === 'listening' ? '暂停聆听' : '开始说话'" data-testid="assistant-orb" :disabled="imageReading || submitting || capturing" data-drag-handle @click="onOrb">
            <Orb :phase="store.phase" :level="asr.level" :active="store.open" :size="52" :dark="isDark" />
          </button>
          <input ref="field" v-model="store.input" class="field" type="text" autocomplete="off" spellcheck="false" :placeholder="store.status" aria-label="向小伴提问" data-testid="assistant-input" @keydown.enter="onEnter" @compositionstart="composing = true" @compositionend="composing = false" />
          <div ref="toolsHost" class="tools-host" data-no-drag @pointerenter="enterTools" @pointerleave="leaveTools" @focusin="focusTools" @focusout="toolsFocusOut">
            <div v-if="toolsOpen" ref="toolbar" id="assistant-toolbar" class="tools-toolbar" role="toolbar" aria-label="小伴工具" aria-orientation="horizontal" data-testid="assistant-toolbar" @keydown="toolbarKeys" @pointerenter="focusTools" @pointerleave="leaveTools">
              <button type="button" class="tool-button" title="去超级小伴设置页" aria-label="去超级小伴设置页" data-testid="assistant-settings" @click="toolAction('settings')"><Settings :size="16" :stroke-width="1.7" /></button>
              <button type="button" class="tool-button" title="收起小伴" aria-label="收起小伴" data-testid="assistant-close" @click="toolAction('hide')"><ChevronDown :size="16" :stroke-width="1.7" /></button>
              <button type="button" class="tool-button" title="去聊天详情页" aria-label="去聊天详情页" data-testid="assistant-expand" @click="toolAction('chat')"><MessageSquare :size="16" :stroke-width="1.7" /></button>
              <button type="button" class="tool-button" title="选择图片" aria-label="选择图片" data-testid="assistant-add-image" :disabled="!canAttach || store.attachments.length >= 8" @click="toolAction('image')"><ImagePlus :size="16" :stroke-width="1.7" /></button>
              <button type="button" class="tool-button" title="截图" aria-label="截图" data-testid="assistant-screenshot" :disabled="!canAttach || store.attachments.length >= 8" @click="toolAction('screenshot')"><ScanLine :size="16" :stroke-width="1.7" /></button>
            </div>
            <button ref="primaryButton" type="button" class="round main-action" :class="{ 'has-action': canSend || busy, 'tools-open': toolsOpen && !canSend && !busy }" :aria-label="busy ? '停止' : canSend ? '发送' : toolsOpen ? '收起工具' : '展开工具'" :title="busy ? '停止 · 悬停展开工具' : canSend ? '发送 · 悬停展开工具' : '展开工具'" :aria-expanded="toolsOpen" aria-controls="assistant-toolbar" aria-keyshortcuts="ArrowUp ArrowDown" :disabled="imageReading || submitting || capturing" data-testid="assistant-primary" @keydown="primaryKeys" @click="primary"><Square v-if="busy" :size="14" /><ArrowUp v-else-if="canSend" :size="18" /><Plus v-else :size="18" /></button>
          </div>
        </div>
        <p v-if="dropActive || imageReading || capturing" class="image-status" aria-live="polite">{{ capturing ? '截图中，按 Esc 可以取消' : imageReading ? '正在读取图片…' : '松开添加图片' }}</p>
        </div>
        <span class="sr" aria-live="polite">{{ store.status }}</span>
      </div>
    </Transition>
  </div>
</template>

<style>
html,body,#app{background:transparent!important}
</style>
<style scoped>
.stage{--card:#141925;--capsule:#151a27;--line:rgba(160,190,255,.2);--text:#eaf0fb;--text-soft:#8a94ab;--working:#9fb0cf;--placeholder:#7d88a2;--field:#f3f6fd;--button:rgba(255,255,255,.07);--button-hover:rgba(255,255,255,.14);--button-fg:#b9c3d8;--button-fg-hover:#fff;--code:rgba(255,255,255,.08);--link:#7fd6ff;--hint:#ffd9a0;--failure:#ff9aa6;--gate-bg:rgba(255,190,100,.08);--gate-line:rgba(255,190,100,.28);--gate-title:#ffe2b4;--gate-tool:#c3cce0;--ghost:rgba(255,255,255,.08);--ghost-fg:#dfe6f5;--scroll:rgba(255,255,255,.18);--dot:#8fb7ff;--listen:rgba(84,226,255,.7);--busy:rgba(140,150,255,.6);--attention:rgba(255,190,100,.7);--ui-line:var(--line);--ui-surface-alt:var(--code);--ui-text-primary:var(--text);--ui-text-secondary:var(--working);--ui-text-tertiary:var(--text-soft);--ui-primary:var(--link);position:relative;display:flex;flex-direction:column;justify-content:flex-end;height:100vh;color:var(--text);font:14px/1.6 -apple-system,BlinkMacSystemFont,"PingFang SC","Segoe UI","Microsoft YaHei UI",sans-serif;-webkit-font-smoothing:antialiased;user-select:none;cursor:grab}
.stage.light{--card:#ffffff;--capsule:#ffffff;--line:rgba(40,60,100,.28);--text:#1d2433;--text-soft:#6b7487;--working:#55627d;--placeholder:#98a1b3;--field:#1d2433;--button:rgba(30,45,80,.07);--button-hover:rgba(30,45,80,.13);--button-fg:#566079;--button-fg-hover:#1d2433;--code:rgba(30,45,80,.07);--link:#1a6fd4;--hint:#9a5b00;--failure:#c8324a;--gate-bg:rgba(255,170,60,.12);--gate-line:rgba(220,140,30,.42);--gate-title:#7a4700;--gate-tool:#4a556d;--ghost:rgba(30,45,80,.07);--ghost-fg:#2a3550;--scroll:rgba(30,45,80,.24);--dot:#5b7fe0;--listen:rgba(0,160,200,.8);--busy:rgba(90,100,230,.7);--attention:rgba(230,140,20,.85)}
.ball-stage{display:grid;place-items:center;height:100vh}
.ball{touch-action:none;display:grid;place-items:center;width:64px;height:64px;padding:0;border-radius:50%;background:var(--capsule);border:1px solid var(--line);cursor:pointer;transition:border-color .3s,transform .2s}
.ball:hover{transform:scale(1.06)}
.ball:active{transform:scale(.96)}
.ball:focus-visible{outline:2px solid var(--link);outline-offset:2px}
.ball[data-phase=listening]{border-color:var(--listen)}
.ball[data-phase=thinking],.ball[data-phase=answering],.ball[data-phase=speaking]{border-color:var(--busy)}
.ball[data-phase=attention]{border-color:var(--attention)}
.inner{display:flex;flex-direction:column;gap:10px;padding:12px}
.card{position:relative;border-radius:22px;background:var(--card);border:1px solid var(--line);overflow:hidden}
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
.gate .solid{background:linear-gradient(135deg,#ffcf7a,#ff9f5a);color:#2a1604;font-weight:600}
.composer{position:relative;border-radius:34px;background:var(--capsule);border:1px solid var(--line);transition:border-color .2s}
.composer.drop-active{border-color:var(--link);box-shadow:inset 0 0 0 1px var(--link)}
.tools-host{position:relative;flex:none;cursor:default}
.tools-toolbar{position:absolute;right:-3px;bottom:calc(100% + 9px);display:flex;align-items:center;gap:4px;padding:6px;border-radius:16px;background:var(--card);border:1px solid var(--line);box-shadow:0 5px 20px rgba(0,0,0,.12);z-index:5;animation:toolbar-in .16s ease-out;cursor:default}
.tools-toolbar::after{content:"";position:absolute;top:100%;left:0;right:0;height:12px}
.tool-button{display:grid;place-items:center;width:32px;height:32px;flex:none;padding:0;border:0;border-radius:10px;background:transparent;color:var(--button-fg);cursor:pointer;transition:background .15s,color .15s}
.tool-button:hover:not(:disabled){background:var(--button-hover);color:var(--button-fg-hover)}
.tool-button:focus-visible{outline:2px solid var(--link);outline-offset:1px}
.tool-button:disabled{opacity:.35;cursor:default}
.round.main-action{background:var(--button);color:var(--button-fg)}
.round.main-action.has-action{background:var(--text);color:var(--card)}
.main-action svg{transition:transform .18s}
.main-action.tools-open svg{transform:rotate(45deg)}
.main-action:disabled{opacity:.45;cursor:default}
@keyframes toolbar-in{from{opacity:0;transform:translateY(4px)}to{opacity:1;transform:translateY(0)}}
.image-tray{display:flex;gap:8px;flex-wrap:wrap;padding:12px 16px 2px;cursor:default}
.image-thumb{position:relative;width:44px;height:44px;flex:none}
.image-thumb img,.sent-images img{width:44px;height:44px;object-fit:cover;border-radius:8px;background:var(--button)}
.image-thumb button{position:absolute;right:-4px;top:-4px;width:17px;height:17px;padding:0;display:grid;place-items:center;border:1px solid var(--line);border-radius:50%;background:var(--card);color:var(--text);cursor:pointer}
.image-thumb button:focus-visible{outline:2px solid var(--link)}
.sent-images{display:flex;gap:6px;flex-wrap:wrap;margin-bottom:8px}
.image-status{font-size:11px;color:var(--link);margin:0;padding:0 20px 8px;cursor:default}
.capsule{display:flex;align-items:center;gap:8px;height:68px;padding:0 12px 0 8px;border-radius:34px;background:transparent;border:0;transition:border-color .3s}
.composer:has(.capsule[data-phase=listening]){border-color:var(--listen)}
.composer:has(.capsule[data-phase=thinking]),.composer:has(.capsule[data-phase=answering]),.composer:has(.capsule[data-phase=speaking]){border-color:var(--busy)}
.composer:has(.capsule[data-phase=attention]){border-color:var(--attention)}
.round,.gate button,.link{cursor:pointer}.orb-button,.ball{cursor:grab}.stage.dragging,.stage.dragging *{cursor:grabbing!important}
.orb-button{touch-action:none;flex:none;width:52px;height:52px;padding:0;border:0;border-radius:50%;background:none;cursor:pointer}
.orb-button:focus-visible,.round:focus-visible{outline:2px solid var(--link);outline-offset:2px}
.field{cursor:text;flex:1;min-width:0;height:44px;padding:0 4px;border:0;outline:0;background:transparent;color:var(--field);font:inherit;font-size:16px;caret-color:var(--link);user-select:text}
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
</style>


<style scoped>
.card-footer{display:flex;justify-content:flex-end;padding:0 12px 9px}.copy-answer{display:inline-flex;align-items:center;gap:5px;padding:4px 7px;border:0;border-radius:7px;background:transparent;color:var(--text-soft);font:inherit;font-size:11px;line-height:1.5;cursor:pointer}.copy-answer:hover{background:var(--button-hover);color:var(--text)}.copy-answer:focus-visible{outline:2px solid var(--link);outline-offset:1px}.copy-answer:disabled{opacity:.5;cursor:default}
</style>
