<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { ArrowUp, ArrowUpRight, Square, X } from 'lucide-vue-next';
import MarkdownView from '../ui/chat/MarkdownView.ts';
import type { AsrStore } from '../ui/asr/store.ts';
import type { ApprovalRequest } from '../ui/chat/types.ts';
import { invoke } from '../bridge.ts';
import { isDark } from '../ui/appearance.ts';
import Orb from './Orb.vue';
import type { AssistantStore } from './assistant-store.ts';

const props = defineProps<{ store: AssistantStore; asr: AsrStore }>();
const field = ref<HTMLInputElement>();
const inner = ref<HTMLElement>();
const scroller = ref<HTMLElement>();
const composing = ref(false);
const quiet = computed(() => props.store.phase === 'idle');

const busy = computed(() => ['thinking', 'answering', 'speaking'].includes(props.store.phase));
const hasCard = computed(() => Boolean(
  props.store.hint || props.store.question || props.store.answer || props.store.working || props.store.failure
  || props.store.approval || props.store.ask || props.store.phase === 'thinking',
));
const canSend = computed(() => Boolean(props.store.input.trim()));

function approvalLine(request: ApprovalRequest) {
  const raw = typeof request.arguments === 'string' ? request.arguments : JSON.stringify(request.arguments);
  const text = `${request.tool} ${raw ?? ''}`.trim();
  return text.length > 180 ? `${text.slice(0, 180)}…` : text;
}
function send() {
  const text = props.store.input.trim();
  if (!text) return;
  if (props.store.ask) { props.store.input = ''; void props.store.reply(text); return; }
  void props.store.submit();
}
function onEnter(event: KeyboardEvent) {
  if (event.isComposing || composing.value) return;
  event.preventDefault();
  send();
}
function primary() {
  if (canSend.value) send();
  else if (busy.value || props.store.phase === 'attention') props.store.interrupt();
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
  raf = requestAnimationFrame(() => {
    const el = inner.value;
    if (!el) return;
    const height = Math.ceil(el.getBoundingClientRect().height);
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

/** The ball opens on a click and moves when it is pressed and pulled; the two are told apart by how far the pointer went. */
let press: { x: number; y: number } | null = null;
let pulled = false;
function ballDown(event: PointerEvent) {
  if (event.button !== 0) return;
  press = { x: event.clientX, y: event.clientY };
  pulled = false;
}
function release() { press = null; }
function ballMove(event: PointerEvent) {
  if (!press || pulled || Math.hypot(event.clientX - press.x, event.clientY - press.y) < 4) return;
  pulled = true; press = null;
  void invoke('assistant_drag').catch(() => {});
}
function ballClick() {
  if (pulled) { pulled = false; return; }
  void props.store.unfold();
}

/** Anything that is not a control or readable text moves the whole capsule, including the clear margin around it. */
function drag(event: MouseEvent) {
  if ((event.target as Element).closest('button, input, a, pre, code, .card-body')) return;
  event.preventDefault();
  void invoke('assistant_drag').catch(() => {});
}

const onFocus = () => { if (props.store.open) field.value?.focus(); };
onMounted(() => window.addEventListener('focus', onFocus));
onBeforeUnmount(() => { window.removeEventListener('focus', onFocus); observer?.disconnect(); cancelAnimationFrame(raf); });
</script>

<template>
  <div class="stage" :class="{ light: !isDark }" @keydown.esc.prevent="store.hide()" @mousedown.left="drag">
    <div v-if="store.ball && !store.open" class="ball-stage" data-testid="assistant-ball-stage">
      <button type="button" class="ball" :data-phase="store.phase" title="点一下提问，按住可以拖动，右键打开设置" aria-label="打开小伴助手" data-testid="assistant-ball" @pointerdown="ballDown" @pointermove="ballMove" @pointerup="release" @pointercancel="release" @click="ballClick" @contextmenu.prevent="store.openSettings('assistant')">
        <Orb :phase="store.phase" :level="0" :active="true" :size="48" :dark="isDark" :fps="quiet ? 20 : 60" />
      </button>
    </div>
    <Transition name="pop" appear>
      <div v-if="store.open" ref="inner" class="inner" role="dialog" aria-label="小伴助手" data-testid="assistant">
        <Transition name="rise">
          <section v-if="hasCard" class="card" data-testid="assistant-card">
            <div ref="scroller" class="card-body">
              <p v-if="store.hint" class="hint" data-testid="assistant-hint">
                {{ store.hint }}
                <button v-if="store.hintTab" type="button" class="link" @click="store.openSettings(store.hintTab)">{{ store.hintTab === 'ai' ? '去配置' : '去设置' }}</button>
              </p>
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
          </section>
        </Transition>
        <div class="capsule" :data-phase="store.phase">
          <button type="button" class="orb-button" :title="store.phase === 'speaking' ? '停止朗读' : store.phase === 'listening' ? '暂停聆听' : '点一下开始说话'" :aria-label="store.phase === 'listening' ? '暂停聆听' : '开始说话'" data-testid="assistant-orb" @click="onOrb">
            <Orb :phase="store.phase" :level="asr.level" :active="store.open" :size="52" :dark="isDark" />
          </button>
          <input ref="field" v-model="store.input" class="field" type="text" autocomplete="off" spellcheck="false" :placeholder="store.status" aria-label="向小伴提问" data-testid="assistant-input" @keydown.enter="onEnter" @compositionstart="composing = true" @compositionend="composing = false" />
          <button v-if="canSend || busy || store.phase === 'attention'" type="button" class="round primary" :aria-label="canSend ? '发送' : '停止'" :title="canSend ? '发送' : '停止'" data-testid="assistant-primary" @click="primary"><ArrowUp v-if="canSend" :size="18" /><Square v-else :size="14" /></button>
          <button type="button" class="round" title="在完整聊天中继续" aria-label="在完整聊天中继续" data-testid="assistant-expand" @click="store.openChat()"><ArrowUpRight :size="17" /></button>
          <button type="button" class="round" title="收起 (Esc)" aria-label="收起" data-testid="assistant-close" @click="store.hide()"><X :size="17" /></button>
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
.ball{display:grid;place-items:center;width:64px;height:64px;padding:0;border-radius:50%;background:var(--capsule);border:1px solid var(--line);cursor:pointer;transition:border-color .3s,transform .2s}
.ball:hover{transform:scale(1.06)}
.ball:active{transform:scale(.96)}
.ball:focus-visible{outline:2px solid var(--link);outline-offset:2px}
.ball[data-phase=listening]{border-color:var(--listen)}
.ball[data-phase=thinking],.ball[data-phase=answering],.ball[data-phase=speaking]{border-color:var(--busy)}
.ball[data-phase=attention]{border-color:var(--attention)}
.inner{display:flex;flex-direction:column;gap:10px;padding:12px}
.card{border-radius:22px;background:var(--card);border:1px solid var(--line);overflow:hidden}
.card-body{cursor:auto;max-height:390px;overflow-y:auto;padding:16px 20px 18px;user-select:text;scrollbar-width:thin;scrollbar-color:var(--scroll) transparent}
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
.capsule{display:flex;align-items:center;gap:8px;height:68px;padding:0 12px 0 8px;border-radius:34px;background:var(--capsule);border:1px solid var(--line);transition:border-color .3s}
.capsule[data-phase=listening]{border-color:var(--listen)}
.capsule[data-phase=thinking],.capsule[data-phase=answering],.capsule[data-phase=speaking]{border-color:var(--busy)}
.capsule[data-phase=attention]{border-color:var(--attention)}
.orb-button,.round,.gate button,.link{cursor:pointer}
.orb-button{flex:none;width:52px;height:52px;padding:0;border:0;border-radius:50%;background:none;cursor:pointer}
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
@media(prefers-reduced-motion:reduce){.pop-enter-active,.rise-enter-active{transition:none}.dots i,.cursor{animation:none}}
</style>
