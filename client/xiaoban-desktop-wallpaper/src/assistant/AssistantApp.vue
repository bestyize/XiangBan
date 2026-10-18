<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { ArrowUp, ArrowUpRight, Square, X } from 'lucide-vue-next';
import MarkdownView from '../ui/chat/MarkdownView.ts';
import type { AsrStore } from '../ui/asr/store.ts';
import type { ApprovalRequest } from '../ui/chat/types.ts';
import { invoke } from '../bridge.ts';
import Orb from './Orb.vue';
import type { AssistantStore } from './assistant-store.ts';

const props = defineProps<{ store: AssistantStore; asr: AsrStore }>();
const field = ref<HTMLInputElement>();
const inner = ref<HTMLElement>();
const scroller = ref<HTMLElement>();
const composing = ref(false);

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

const onBlur = () => props.store.blurred();
const onFocus = () => { if (props.store.open) field.value?.focus(); };
onMounted(() => { window.addEventListener('blur', onBlur); window.addEventListener('focus', onFocus); });
onBeforeUnmount(() => { window.removeEventListener('blur', onBlur); window.removeEventListener('focus', onFocus); observer?.disconnect(); cancelAnimationFrame(raf); });
</script>

<template>
  <div class="stage" @keydown.esc.prevent="store.hide()" @pointerenter="store.hovering = true" @pointerleave="store.hovering = false" @pointermove.passive="store.touch()" @keydown="store.touch()">
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
            <Orb :phase="store.phase" :level="asr.level" :active="store.open" :size="52" />
          </button>
          <input ref="field" v-model="store.input" class="field" type="text" autocomplete="off" spellcheck="false" :placeholder="store.status" aria-label="向小伴提问" data-testid="assistant-input" @keydown.enter="onEnter" @compositionstart="composing = true" @compositionend="composing = false" />
          <button v-if="canSend || busy || store.phase === 'attention'" type="button" class="round primary" :aria-label="canSend ? '发送' : '停止'" :title="canSend ? '发送' : '停止'" data-testid="assistant-primary" @click="primary"><ArrowUp v-if="canSend" :size="18" /><Square v-else :size="14" /></button>
          <button type="button" class="round" title="在完整聊天中继续" aria-label="在完整聊天中继续" data-testid="assistant-expand" @click="store.expand()"><ArrowUpRight :size="17" /></button>
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
.stage{--ui-line:rgba(160,190,255,.14);--ui-surface-alt:rgba(255,255,255,.07);--ui-text-primary:#eef2fa;--ui-text-secondary:#b4bdd0;--ui-text-tertiary:#8a94ab;--ui-primary:#7fd6ff;position:relative;display:flex;flex-direction:column;justify-content:flex-end;height:100vh;color:#e8edf7;font:14px/1.6 -apple-system,BlinkMacSystemFont,"PingFang SC","Segoe UI","Microsoft YaHei UI",sans-serif;-webkit-font-smoothing:antialiased;user-select:none}
.inner{display:flex;flex-direction:column;gap:10px;padding:24px}
.card{border-radius:22px;background:linear-gradient(180deg,rgba(25,30,44,.97),rgba(15,19,29,.97));border:1px solid rgba(160,190,255,.14);box-shadow:0 18px 50px rgba(2,6,16,.5),inset 0 1px 0 rgba(255,255,255,.06);overflow:hidden}
.card-body{max-height:390px;overflow-y:auto;padding:16px 20px 18px;user-select:text;scrollbar-width:thin;scrollbar-color:rgba(255,255,255,.18) transparent}
.you{margin:0 0 8px;font-size:12.5px;color:#8a94ab;display:-webkit-box;-webkit-line-clamp:2;line-clamp:2;-webkit-box-orient:vertical;overflow:hidden}
.reply{font-size:15px;line-height:1.75;color:#eaf0fb;overflow-wrap:anywhere}
.reply :deep(p){margin:0 0 .7em}
.reply :deep(p:last-child){margin-bottom:0}
.reply :deep(.md-code){background:rgba(255,255,255,.08);border-radius:5px;padding:1px 6px;font-size:.88em;font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,monospace}
.reply :deep(.md-link){color:#7fd6ff;cursor:pointer;text-decoration:underline;text-underline-offset:2px}
.reply :deep(.md-heading){margin:1em 0 .4em;font-size:15px}
.reply :deep(ul),.reply :deep(ol){margin:.3em 0 .7em;padding-left:1.4em}
.cursor{display:inline-block;width:7px;height:15px;margin-left:3px;vertical-align:-2px;border-radius:2px;background:#7fd6ff;animation:blink 1s steps(2,start) infinite}
.working{margin:0;display:flex;align-items:center;gap:10px;color:#9fb0cf;font-size:13.5px}
.dots{display:inline-flex;gap:4px}
.dots i{width:6px;height:6px;border-radius:50%;background:#8fb7ff;animation:dot 1.2s ease-in-out infinite}
.dots i:nth-child(2){animation-delay:.15s}.dots i:nth-child(3){animation-delay:.3s}
.hint{margin:0 0 10px;color:#ffd9a0;font-size:13.5px}
.failure{margin:0;color:#ff9aa6;font-size:13.5px}
.link{margin-left:8px;padding:0;border:0;background:none;color:#7fd6ff;font:inherit;text-decoration:underline;text-underline-offset:2px;cursor:pointer}
.gate{margin-top:14px;padding:14px;border-radius:14px;background:rgba(255,190,100,.08);border:1px solid rgba(255,190,100,.28)}
.gate-title{margin:0 0 6px;font-size:13.5px;color:#ffe2b4}
.gate-tool{margin:0 0 12px;font:12px/1.5 ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;color:#c3cce0;overflow-wrap:anywhere}
.gate-actions{display:flex;gap:8px;justify-content:flex-end}
.gate-actions.wrap{flex-wrap:wrap;justify-content:flex-start}
.gate button{height:32px;padding:0 14px;border-radius:16px;border:1px solid transparent;font:inherit;font-size:13px;cursor:pointer}
.gate .ghost{background:rgba(255,255,255,.08);color:#dfe6f5}
.gate .ghost.dim{color:#8a94ab}
.gate .solid{background:linear-gradient(135deg,#ffcf7a,#ff9f5a);color:#2a1604;font-weight:600}
.capsule{display:flex;align-items:center;gap:8px;height:68px;padding:0 12px 0 8px;border-radius:34px;background:linear-gradient(180deg,rgba(27,32,48,.98),rgba(14,18,28,.98));border:1px solid rgba(160,190,255,.16);box-shadow:0 16px 44px rgba(2,6,16,.55),0 0 0 1px rgba(0,0,0,.4),inset 0 1px 0 rgba(255,255,255,.07);transition:border-color .3s,box-shadow .3s}
.capsule[data-phase=listening]{border-color:rgba(84,226,255,.5);box-shadow:0 16px 44px rgba(2,6,16,.55),0 0 0 1px rgba(0,0,0,.4),0 0 30px rgba(84,226,255,.2),inset 0 1px 0 rgba(255,255,255,.07)}
.capsule[data-phase=thinking],.capsule[data-phase=answering],.capsule[data-phase=speaking]{border-color:rgba(140,150,255,.42);box-shadow:0 16px 44px rgba(2,6,16,.55),0 0 0 1px rgba(0,0,0,.4),0 0 30px rgba(130,140,255,.18),inset 0 1px 0 rgba(255,255,255,.07)}
.capsule[data-phase=attention]{border-color:rgba(255,190,100,.5)}
.orb-button{flex:none;width:52px;height:52px;padding:0;border:0;border-radius:50%;background:none;cursor:pointer}
.orb-button:focus-visible,.round:focus-visible{outline:2px solid #7fd6ff;outline-offset:2px}
.field{flex:1;min-width:0;height:44px;padding:0 4px;border:0;outline:0;background:transparent;color:#f3f6fd;font:inherit;font-size:16px;caret-color:#7fd6ff;user-select:text}
.field::placeholder{color:#7d88a2}
.round{flex:none;width:36px;height:36px;display:grid;place-items:center;padding:0;border:0;border-radius:50%;background:rgba(255,255,255,.07);color:#b9c3d8;cursor:pointer;transition:background .2s,color .2s}
.round:hover{background:rgba(255,255,255,.14);color:#fff}
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
