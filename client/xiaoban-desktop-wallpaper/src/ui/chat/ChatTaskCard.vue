<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { Check, ChevronDown, CirclePause, ListChecks, LoaderCircle, TriangleAlert } from 'lucide-vue-next';
import { toolLabel } from './format.ts';
import { describeCall } from './view.ts';
import type { ToolCall } from './replies.ts';
import type { TaskProgress } from './task-progress.ts';

const props = defineProps<{ progress: TaskProgress; calls: ToolCall[]; current: boolean; showLogs: boolean; disabled?: boolean; configured?: boolean }>();
const emit = defineEmits<{ logs: []; adjust: []; resume: []; stop: []; attention: [] }>();
const expanded = ref(true);
const stepOpen = ref(new Set<number>());
const labels = { running: '执行中', waiting: '等待输入', approval: '等待授权', blocked: '执行受阻', paused: '已暂停', completed: '已完成' };
const active = computed(() => ['running', 'waiting', 'approval'].includes(props.progress.state));
const attention = computed(() => ['waiting', 'approval', 'blocked'].includes(props.progress.state));
const latest = computed(() => [...props.calls].reverse().find(call => call.name !== 'todo'));
const runningTool = computed(() => latest.value?.status === 'running' ? latest.value : null);
const progressLabel = computed(() => `已完成 ${props.progress.completed} / ${props.progress.plan.steps.length} 步`);
watch(() => props.progress.state, (state, before) => {
  // Bring questions and failures into view, without reopening a reader's collapsed card on every update.
  if (state !== before && ['waiting', 'approval', 'blocked'].includes(state)) expanded.value = true;
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
        <span class="task-symbol" aria-hidden="true"><Check v-if="progress.state === 'completed'" :size="16" /><ListChecks v-else :size="16" /></span>
        <span class="task-heading"><b>任务执行</b><small data-testid="task-count">{{ progressLabel }}</small></span>
        <ChevronDown :size="12" class="chevron" :class="{ expanded }" aria-hidden="true" />
      </button>
      <span class="task-badge" data-testid="task-state" role="status"><i aria-hidden="true"></i>{{ labels[progress.state] }}</span>
    </header>
    <div class="task-track" role="progressbar" :aria-valuenow="progress.completed" :aria-valuemin="0" :aria-valuemax="progress.plan.steps.length" :aria-valuetext="progressLabel" aria-label="任务步骤完成进度"><i :style="{ width: `${progress.completed / progress.plan.steps.length * 100}%` }"></i></div>
    <div v-if="expanded" data-testid="task-body">
      <ol class="task-steps">
        <li v-for="(step, index) in progress.plan.steps" :key="index" class="task-step" :class="{ done: step.status === 'completed', active: index === progress.current, stopped: !active }" :data-testid="`task-step-${index}`">
          <button type="button" class="step-button" :aria-expanded="stepOpen.has(index)" @click="toggleStep(index)">
            <span class="step-mark" aria-hidden="true"><Check v-if="step.status === 'completed'" :size="11" :stroke-width="2.7" /><TriangleAlert v-else-if="index === progress.current && attention" :size="11" /><LoaderCircle v-else-if="index === progress.current && active" :size="16" class="spin" /><CirclePause v-else-if="index === progress.current && !active" :size="14" /><template v-else>{{ index + 1 }}</template></span>
            <span class="step-content"><span class="step-title">{{ step.content }}</span><small v-if="index === progress.current && runningTool" class="step-live">{{ toolLabel(runningTool.name) }}<span v-if="describeCall(runningTool.name, runningTool.arguments)"> · {{ describeCall(runningTool.name, runningTool.arguments) }}</span></small></span>
            <small class="step-state">{{ step.status === 'completed' ? '已完成' : index === progress.current ? labels[progress.state] : '待执行' }}</small>
          </button>
          <div v-if="stepOpen.has(index)" class="step-details" data-testid="task-step-details"><template v-if="step.evidence"><b>小伴记录的执行依据</b><p>{{ step.evidence }}</p></template><p v-else>{{ step.status === 'completed' ? '这一步已记录为完成，未附执行依据。可查看执行过程核对结果。' : index === progress.current ? progress.message : '完成前面的步骤后继续。' }}</p></div>
        </li>
      </ol>
      <div v-if="attention || progress.state === 'paused'" class="task-intervention" :class="{ caution: attention }" data-testid="task-intervention">
        <p>{{ progress.message }}</p><p v-if="progress.plan.handoff" class="handoff">{{ progress.plan.handoff }}</p>
        <button v-if="current && (progress.state === 'waiting' || progress.state === 'approval')" type="button" class="task-primary" @click="emit('attention')">{{ progress.state === 'waiting' ? '回答问题' : '查看待确认操作' }}</button>
        <button v-else-if="current && progress.canResume" type="button" class="task-primary" data-testid="task-resume" :disabled="disabled || configured === false" @click="emit('resume')">核对后继续</button>
      </div>
      <footer class="task-footer"><span>{{ active && progress.state === 'running' ? progress.message : progressLabel }}</span><button v-if="current && active" type="button" data-testid="task-stop" :disabled="disabled" @click="emit('stop')">停止</button><button v-if="current && progress.state !== 'completed'" type="button" data-testid="task-adjust" :disabled="disabled || configured === false" @click="emit('adjust')">调整计划</button><button v-if="showLogs" type="button" data-testid="task-logs" @click="emit('logs')">查看过程 ↗</button></footer>
    </div>
    <p v-else class="task-collapsed">{{ progress.state === 'running' && progress.current >= 0 ? `当前步骤：${progress.plan.steps[progress.current]!.content}` : progress.message }}</p>
  </section>
</template>

<style scoped>
.task-card{width:100%;max-width:600px;min-width:0;box-sizing:border-box;border:1px solid var(--ui-border);border-radius:11px;overflow:hidden;background:var(--ui-surface);font-size:12px}
.task-head{display:flex;align-items:center;gap:10px;padding:11px 14px}.task-toggle{flex:1;min-width:0;display:flex;align-items:center;gap:9px;padding:0;border:0;background:none;color:var(--ui-text-primary);font:inherit;text-align:left;cursor:pointer}
.task-symbol{display:grid;place-items:center;flex:none;width:28px;height:28px;border-radius:8px;background:var(--ui-soft);color:var(--ui-ink)}.task-heading{display:flex;flex-direction:column;gap:2px}.task-heading b{font-size:12.5px;font-weight:600;line-height:1.35}.task-heading small{font-size:10px;color:var(--ui-text-tertiary);font-variant-numeric:tabular-nums}.chevron{color:var(--ui-text-tertiary);transition:transform .15s}.chevron.expanded{transform:rotate(180deg)}
.task-badge{display:inline-flex;align-items:center;gap:5px;padding:3px 7px;flex:none;border-radius:5px;background:var(--ui-soft);color:var(--ui-ink);font-size:10px;white-space:nowrap}.task-badge i{width:4px;height:4px;border-radius:50%;background:currentColor}.waiting .task-badge,.approval .task-badge,.blocked .task-badge{background:color-mix(in srgb,var(--ui-warning) 12%,transparent);color:var(--ui-warning-text)}.paused .task-badge{background:var(--ui-control);color:var(--ui-text-secondary)}
.task-track{height:2px;margin:0 14px;background:var(--ui-line)}.task-track>i{display:block;height:100%;background:var(--ui-btn);transition:width .2s}
.task-steps{list-style:none;margin:0;padding:6px 14px 4px}.task-step{position:relative}.task-step:not(:last-child):before{content:'';position:absolute;left:8.5px;width:1px;top:23px;bottom:-2px;background:var(--ui-line)}
.step-button{position:relative;display:flex;align-items:flex-start;gap:10px;width:100%;padding:6px 0;border:0;background:none;color:inherit;font:inherit;text-align:left;cursor:pointer}.step-mark{position:relative;display:grid;place-items:center;flex:none;width:18px;height:18px;box-sizing:border-box;border:1px solid var(--ui-border);border-radius:50%;background:var(--ui-surface);color:var(--ui-text-disabled);font-size:9px}.done .step-mark{border-color:transparent;background:var(--ui-soft);color:var(--ui-ink)}.active:not(.done) .step-mark{border-color:transparent;color:var(--ui-ink)}.stopped.active:not(.done) .step-mark{color:var(--ui-warning-text)}
.step-content{flex:1;min-width:0}.step-title{font-size:12px;line-height:19px;color:var(--ui-text-tertiary);overflow-wrap:anywhere}.done .step-title{color:var(--ui-text-secondary)}.active .step-title{color:var(--ui-text-primary);font-weight:500}.step-state{font-size:9px;line-height:19px;color:var(--ui-text-tertiary);white-space:nowrap;flex:none}.active .step-state{color:var(--ui-ink)}.step-live{display:block;max-width:100%;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:10px;color:var(--ui-text-tertiary);line-height:1.5;margin-top:1px}
.step-details{margin:0 0 5px 28px;padding:8px 10px;background:var(--ui-control);border-radius:7px;font-size:10.5px;line-height:1.7;color:var(--ui-text-secondary);overflow-wrap:anywhere;white-space:pre-wrap}.step-details b{font-weight:500}.step-details p{margin:3px 0 0}
.task-intervention{margin:4px 14px 11px 42px;padding:10px 12px;border-radius:8px;background:var(--ui-control);font-size:11px;line-height:1.7;color:var(--ui-text-secondary)}.task-intervention.caution{background:color-mix(in srgb,var(--ui-warning) 9%,var(--ui-surface));color:var(--ui-warning-text)}.task-intervention p{margin:0;overflow-wrap:anywhere}.task-intervention .handoff{margin-top:5px;white-space:pre-wrap}.task-primary{margin-top:8px;padding:6px 9px;border:0;border-radius:6px;background:var(--ui-btn);color:var(--ui-text-on-btn);font:inherit;font-size:10.5px;cursor:pointer}
.task-footer{display:flex;align-items:center;gap:12px;padding:9px 14px;border-top:1px solid var(--ui-line);background:var(--ui-surface-sunken);font-size:10px;color:var(--ui-text-tertiary)}.task-footer>span{margin-right:auto;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.task-footer button{flex:none;padding:0;border:0;background:none;color:var(--ui-text-secondary);font:inherit;font-size:10px;cursor:pointer}.task-footer button:hover{color:var(--ui-ink)}.task-collapsed{margin:0;padding:10px 14px;color:var(--ui-text-secondary);font-size:11px;line-height:1.6;overflow-wrap:anywhere}
button:focus-visible{outline:2px solid var(--ui-primary);outline-offset:2px}button:disabled{opacity:.45;cursor:default}.spin{animation:task-spin 1s linear infinite}@keyframes task-spin{to{transform:rotate(360deg)}}@media(prefers-reduced-motion:reduce){.spin{animation:none}.chevron,.task-track>i{transition:none}}@media(max-width:430px){.task-head{padding:10px 12px}.task-steps{padding-left:12px;padding-right:12px}.task-footer{padding:9px 12px;gap:9px}.task-footer>span{font-size:9px}}
</style>
