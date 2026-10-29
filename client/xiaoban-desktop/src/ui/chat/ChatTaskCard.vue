<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { ArrowUpRight, Check, ChevronDown, CirclePause, CircleQuestionMark, Ellipsis, LoaderCircle, ShieldAlert, Square, TriangleAlert } from 'lucide-vue-next';
import { toolLabel } from './format.ts';
import { describeCall } from './view.ts';
import type { ToolCall } from './replies.ts';
import type { TaskProgress, TaskStep } from './task-progress.ts';

const props = defineProps<{ progress: TaskProgress; calls: ToolCall[]; current: boolean; showLogs: boolean; disabled?: boolean; configured?: boolean }>();
const emit = defineEmits<{ logs: []; adjust: []; resume: []; stop: []; attention: [] }>();
const expanded = ref(['running', 'waiting', 'approval'].includes(props.progress.state));
const stepOpen = ref(new Set<number>());
const menuOpen = ref(false);
const menuRoot = ref<HTMLElement | null>(null);
const labels = { running: '执行中', waiting: '等待输入', approval: '等待授权', blocked: '执行受阻', paused: '已暂停', completed: '已完成', partial: '部分完成', failed: '未完成' };
const active = computed(() => ['running', 'waiting', 'approval'].includes(props.progress.state));
const attention = computed(() => ['waiting', 'approval', 'blocked', 'partial', 'failed'].includes(props.progress.state));
const statusVisible = computed(() => !['running', 'completed'].includes(props.progress.state));
const latest = computed(() => [...props.calls].reverse().find(call => call.name !== 'todo'));
const runningTool = computed(() => latest.value?.status === 'running' ? latest.value : null);
const total = computed(() => props.progress.plan.steps.length);
const progressLabel = computed(() => `已完成 ${props.progress.completed} / ${total.value} 步`);
const canAdjust = computed(() => props.current && props.progress.state !== 'completed');
const hasMenu = computed(() => canAdjust.value || props.showLogs);
const headIcon = computed(() => ({ running: LoaderCircle, waiting: CircleQuestionMark, approval: ShieldAlert, paused: CirclePause, blocked: TriangleAlert, partial: TriangleAlert, failed: TriangleAlert, completed: Check })[props.progress.state]);
const collapsedStep = computed(() => {
  if (props.progress.state === 'completed') return '';
  const step = props.progress.plan.steps[props.progress.current];
  return props.progress.state === 'running' && step ? step.content : props.progress.message;
});
const segClass = (index: number) => props.progress.plan.steps[index]?.status === 'completed' ? 'done' : index === props.progress.current && props.progress.state !== 'completed' ? 'now' : '';

// Elapsed time is only known for steps seen starting while this card was on screen; restored history shows none.
const now = ref(Date.now());
const stepStart = new Map<string, number>();
const stepKey = (index: number) => `${index}:${props.progress.plan.steps[index]?.content ?? ''}`;
let runStart: number | null = null;
const finished = ref<number | null>(null);
let timer: ReturnType<typeof setInterval> | undefined;
watch([active, () => props.progress.current, () => props.progress.plan.callId], ([on, index]) => {
  if (on) {
    runStart ??= Date.now();
    if (index >= 0 && !stepStart.has(stepKey(index))) stepStart.set(stepKey(index), Date.now());
  }
  clearInterval(timer);
  timer = on ? setInterval(() => { now.value = Date.now(); }, 1000) : undefined;
}, { immediate: true });
watch(() => props.progress.state, state => { if (state === 'completed' && runStart !== null && finished.value === null) finished.value = Date.now(); });
onBeforeUnmount(() => { clearInterval(timer); document.removeEventListener('pointerdown', away, true); });
const span = (ms: number) => { const s = Math.max(0, Math.round(ms / 1000)); return s < 60 ? `${s} 秒` : `${Math.floor(s / 60)} 分 ${s % 60} 秒`; };
const duration = computed(() => props.progress.state === 'completed' && runStart !== null && finished.value !== null ? span(finished.value - runStart) : '');
function elapsed(index: number) {
  const start = stepStart.get(stepKey(index));
  const seconds = start === undefined ? 0 : Math.floor((now.value - start) / 1000);
  return seconds < 3 ? '' : seconds < 60 ? `${seconds}s` : `${Math.floor(seconds / 60)}m${seconds % 60}s`;
}
function tail(index: number, step: TaskStep) {
  if (index === props.progress.current) {
    if (props.progress.state !== 'running') return labels[props.progress.state];
    const tool = runningTool.value;
    const live = tool ? [toolLabel(tool.name), describeCall(tool.name, tool.arguments)].filter(Boolean).join(' · ') : props.progress.message;
    return [live, elapsed(index)].filter(Boolean).join(' · ');
  }
  return step.status === 'completed' ? step.evidence.split('\n')[0]!.trim() : '';
}

function away(event: Event) { if (menuRoot.value && !menuRoot.value.contains(event.target as Node)) menuOpen.value = false; }
watch(menuOpen, open => { if (open) document.addEventListener('pointerdown', away, true); else document.removeEventListener('pointerdown', away, true); });
function choose(action: 'adjust' | 'logs') { menuOpen.value = false; if (action === 'adjust') emit('adjust'); else emit('logs'); }

watch(() => props.progress.state, (state, before) => {
  // Bring questions and failures into view, without reopening a reader's collapsed card on every update.
  if (state === 'completed' && state !== before) expanded.value = false;
  else if (state !== before && ['running', 'waiting', 'approval', 'blocked', 'partial', 'failed'].includes(state)) expanded.value = true;
});
watch(() => props.progress.plan.callId, () => { stepOpen.value = new Set(); });
function toggleStep(index: number) {
  const next = new Set(stepOpen.value);
  if (next.has(index)) next.delete(index); else next.add(index);
  stepOpen.value = next;
}
</script>

<template>
  <section class="task-card" :class="progress.state" data-testid="chat-task-card" aria-label="任务执行进度">
    <header class="task-head">
      <button type="button" class="task-toggle" data-testid="task-toggle" :aria-expanded="expanded" @click="expanded = !expanded">
        <span class="task-symbol" aria-hidden="true"><component :is="headIcon" :size="progress.state === 'running' ? 18 : 13" :stroke-width="progress.state === 'completed' ? 3 : 2.2" :class="{ spin: progress.state === 'running' }" /></span>
        <b class="task-title">{{ progress.state === 'completed' ? '任务完成' : '任务执行' }}</b>
        <span class="task-state" :class="{ visible: statusVisible }" data-testid="task-state" role="status">{{ labels[progress.state] }}</span>
        <span v-if="duration" class="task-meta">用时 {{ duration }}</span>
        <span v-if="!expanded && collapsedStep" class="task-now">{{ collapsedStep }}</span>
        <span v-else class="task-grow"></span>
        <span class="task-count" aria-hidden="true">{{ progress.completed }}/{{ total }}</span>
        <span class="sr-only" data-testid="task-count">{{ progressLabel }}</span>
        <span v-if="progress.state !== 'completed'" class="task-segs" :class="{ dense: total > 12 }" :style="{ '--n': total }" role="progressbar" :aria-valuenow="progress.completed" :aria-valuemin="0" :aria-valuemax="total" :aria-valuetext="progressLabel" aria-label="任务步骤完成进度"><i v-for="(_, index) in progress.plan.steps" :key="index" :class="segClass(index)"></i></span>
      </button>
      <button v-if="current && active" type="button" class="task-icon stop" data-testid="task-stop" aria-label="停止" title="停止" :disabled="disabled" @click="emit('stop')"><Square :size="11" fill="currentColor" /></button>
      <div v-if="hasMenu" ref="menuRoot" class="task-more" @keydown.esc="menuOpen = false">
        <button type="button" class="task-icon" data-testid="task-more" aria-haspopup="menu" :aria-expanded="menuOpen" aria-label="更多" title="更多" @click="menuOpen = !menuOpen"><Ellipsis :size="14" /></button>
        <div v-if="menuOpen" class="xb-menu task-menu" role="menu" aria-label="任务操作">
          <button v-if="canAdjust" type="button" class="xb-menu-item" role="menuitem" data-testid="task-adjust" :disabled="disabled || configured === false" @click="choose('adjust')">调整计划</button>
          <button v-if="showLogs" type="button" class="xb-menu-item" role="menuitem" data-testid="task-logs" @click="choose('logs')">执行详情<ArrowUpRight :size="12" class="xb-menu-hint" /></button>
        </div>
      </div>
      <button type="button" class="task-icon" tabindex="-1" aria-hidden="true" @click="expanded = !expanded"><ChevronDown :size="14" class="chevron" :class="{ expanded }" /></button>
    </header>
    <div v-if="expanded" class="task-body" data-testid="task-body">
      <ol class="task-steps">
        <li v-for="(step, index) in progress.plan.steps" :key="index" class="task-step" :class="{ done: step.status === 'completed', active: index === progress.current, stopped: !active, attention: index === progress.current && attention }" :data-testid="`task-step-${index}`">
          <button type="button" class="step-button" :aria-expanded="stepOpen.has(index)" @click="toggleStep(index)">
            <span class="step-mark" aria-hidden="true"><Check v-if="step.status === 'completed'" :size="10" :stroke-width="3" /><TriangleAlert v-else-if="index === progress.current && attention && progress.state !== 'approval' && progress.state !== 'waiting'" :size="10" :stroke-width="2.4" /><ShieldAlert v-else-if="index === progress.current && progress.state === 'approval'" :size="10" :stroke-width="2.4" /><CircleQuestionMark v-else-if="index === progress.current && progress.state === 'waiting'" :size="10" :stroke-width="2.4" /><LoaderCircle v-else-if="index === progress.current && active" :size="16" class="spin" /><CirclePause v-else-if="index === progress.current && !active" :size="10" :stroke-width="2.4" /></span>
            <span class="step-title">{{ step.content }}</span>
            <small v-if="tail(index, step)" class="step-tail">{{ tail(index, step) }}</small>
          </button>
          <div v-if="stepOpen.has(index)" class="step-details" data-testid="task-step-details"><template v-if="step.evidence"><b>执行详情</b><p>{{ step.evidence }}</p></template><p v-else>{{ step.status === 'completed' ? '此步骤已完成。' : index === progress.current ? progress.message : '完成前面的步骤后继续。' }}</p></div>
        </li>
      </ol>
      <div v-if="attention || progress.state === 'paused'" class="task-intervention" :class="{ caution: attention }" data-testid="task-intervention">
        <div class="intervention-text"><span>{{ progress.message }}</span><span v-if="progress.plan.handoff" class="handoff">{{ progress.plan.handoff }}</span></div>
        <button v-if="current && (progress.state === 'waiting' || progress.state === 'approval')" type="button" class="task-primary" @click="emit('attention')">{{ progress.state === 'waiting' ? '回答问题' : '查看待确认操作' }}</button>
        <button v-else-if="current && progress.canResume" type="button" class="task-primary" data-testid="task-resume" :disabled="disabled || configured === false" @click="emit('resume')">继续完成</button>
      </div>
    </div>
  </section>
</template>

<style scoped>
.task-card{position:relative;width:100%;max-width:600px;min-width:0;box-sizing:border-box;border:1px solid var(--ui-border);border-radius:12px;background:var(--ui-surface);font-size:12.5px}
.task-head{display:flex;align-items:center;gap:4px;height:40px;padding:0 8px 0 12px}
.task-toggle{flex:1;min-width:0;display:flex;align-items:center;gap:10px;height:100%;padding:0;border:0;background:none;color:var(--ui-text-primary);font:inherit;text-align:left;cursor:pointer}
.task-symbol{display:grid;place-items:center;flex:none;width:20px;height:20px;border-radius:50%;color:var(--ui-btn)}
.completed .task-symbol{background:var(--ui-soft);color:var(--ui-ink)}
.waiting .task-symbol,.approval .task-symbol,.blocked .task-symbol,.partial .task-symbol{border-radius:6px;background:color-mix(in srgb,var(--ui-warning) 16%,transparent);color:var(--ui-warning-text)}
.failed .task-symbol{border-radius:6px;background:color-mix(in srgb,var(--ui-danger) 12%,transparent);color:var(--ui-danger)}
.paused .task-symbol{border-radius:6px;background:var(--ui-control);color:var(--ui-text-secondary)}
.task-title{flex:none;font-size:13px;font-weight:600}
.task-state{position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0);white-space:nowrap}
.task-state.visible{position:static;width:auto;height:auto;overflow:visible;clip:auto;flex:none;font-size:12px;font-weight:500;color:var(--ui-warning-text)}
.paused .task-state.visible{color:var(--ui-text-secondary)}.failed .task-state.visible{color:var(--ui-danger)}
.task-meta{flex:none;font-size:12px;color:var(--ui-text-secondary)}
.task-now{flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:var(--ui-text-secondary)}.task-grow{flex:1}
.task-count{flex:none;font-size:12px;color:var(--ui-text-secondary);font-variant-numeric:tabular-nums}
.sr-only{position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0);white-space:nowrap}
.task-segs{display:flex;flex:none;gap:3px;width:min(88px,calc(var(--n) * 17px - 3px));align-items:center}.task-segs.dense{gap:2px}
.task-segs i{display:block;flex:1;min-width:2px;height:4px;border-radius:2px;background:color-mix(in srgb,var(--ui-text-primary) 12%,transparent)}
.task-segs i.done{background:var(--ui-btn)}.task-segs i.now{background:var(--ui-btn);opacity:.45}
.task-icon{display:grid;place-items:center;flex:none;width:24px;height:24px;padding:0;border:0;border-radius:7px;background:transparent;color:var(--ui-text-tertiary);cursor:pointer}
.task-icon:hover:not(:disabled){background:var(--ui-surface-hover);color:var(--ui-text-primary)}
.task-icon.stop{margin-left:2px;background:var(--ui-surface);box-shadow:0 0 0 1px var(--ui-border-strong);color:var(--ui-text-secondary)}.task-icon.stop:hover:not(:disabled){background:var(--ui-surface);color:var(--ui-danger)}
.chevron{transition:transform .15s}.chevron.expanded{transform:rotate(180deg)}
.task-more{position:relative;display:flex}.task-menu{position:absolute;right:0;top:28px;min-width:132px}.task-menu .xb-menu-item{min-height:30px;font-size:12.5px}.task-menu .xb-menu-hint{margin-left:auto}
.task-steps{display:flex;flex-direction:column;gap:2px;margin:0;padding:0 12px 10px;list-style:none}
.task-step{position:relative}.task-step:not(:last-child):before{content:'';position:absolute;left:7.5px;top:22px;bottom:-4px;width:1px;background:var(--ui-line)}
.step-button{position:relative;display:flex;align-items:center;gap:10px;width:calc(100% + 16px);min-height:28px;margin:0 -8px;padding:0 8px;box-sizing:border-box;border:0;border-radius:8px;background:none;color:inherit;font:inherit;text-align:left;cursor:pointer}
.step-button:hover{background:var(--ui-surface-hover)}
.active .step-button{background:var(--ui-soft)}.attention .step-button{background:color-mix(in srgb,var(--ui-warning) 10%,var(--ui-surface))}.stopped.active .step-button{background:var(--ui-control)}
.step-mark{position:relative;z-index:1;display:grid;place-items:center;flex:none;width:16px;height:16px;box-sizing:border-box;border-radius:50%;background:var(--ui-surface);box-shadow:inset 0 0 0 1.5px var(--ui-border-strong);color:var(--ui-text-disabled)}
.done .step-mark{background:var(--ui-soft);box-shadow:none;color:var(--ui-ink)}
.active:not(.done) .step-mark{background:transparent;box-shadow:none;color:var(--ui-ink)}
.attention:not(.done) .step-mark{background:color-mix(in srgb,var(--ui-warning) 18%,var(--ui-surface));color:var(--ui-warning-text)}
.stopped.active:not(.done):not(.attention) .step-mark{background:var(--ui-control);color:var(--ui-text-secondary)}
.step-title{flex:1;min-width:0;padding:4px 0;font-size:12.5px;line-height:19px;color:var(--ui-text-tertiary);overflow-wrap:anywhere}.done .step-title{color:var(--ui-text-secondary)}.active .step-title{color:var(--ui-text-primary);font-weight:500}
.step-tail{flex:0 1 auto;min-width:0;max-width:46%;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:11px;color:var(--ui-text-tertiary);font-variant-numeric:tabular-nums}.active .step-tail{color:var(--ui-text-secondary)}.attention .step-tail{color:var(--ui-warning-text)}
.step-details{margin:2px 0 4px 26px;padding:8px 10px;background:var(--ui-control);border-radius:7px;font-size:11.5px;line-height:1.7;color:var(--ui-text-secondary);overflow-wrap:anywhere;white-space:pre-wrap}.step-details b{font-weight:500}.step-details p{margin:3px 0 0}
.task-intervention{display:flex;align-items:center;gap:10px;margin:-4px 12px 10px 38px;padding:6px 6px 6px 10px;border-radius:8px;background:var(--ui-control);font-size:12px;line-height:1.6;color:var(--ui-text-secondary)}
.task-intervention.caution{background:color-mix(in srgb,var(--ui-warning) 10%,var(--ui-surface));color:var(--ui-warning-text)}
.intervention-text{flex:1;min-width:0;display:flex;flex-direction:column;gap:2px;overflow-wrap:anywhere}.handoff{white-space:pre-wrap;font-size:11.5px;opacity:.85}
.task-primary{flex:none;height:26px;padding:0 11px;border:0;border-radius:7px;background:linear-gradient(var(--ui-btn-hi),var(--ui-btn));box-shadow:var(--ui-shadow-btn);color:var(--ui-text-on-btn);font:inherit;font-size:12px;font-weight:500;cursor:pointer}
button:focus-visible{outline:2px solid var(--ui-primary);outline-offset:2px}button:disabled{opacity:.45;cursor:default}
.spin{animation:task-spin 1s linear infinite}@keyframes task-spin{to{transform:rotate(360deg)}}
@media(prefers-reduced-motion:reduce){.spin{animation:none}.chevron{transition:none}}
@media(max-width:430px){.task-head{padding-left:10px}.task-segs{display:none}.task-intervention{margin-left:12px;flex-wrap:wrap}}
</style>
