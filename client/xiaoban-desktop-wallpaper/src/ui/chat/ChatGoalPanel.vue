<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue';
import { NButton, NInput } from 'naive-ui';
import { invoke, listen } from '../../bridge.ts';
import type { SessionSummary } from './types.ts';

type GoalState = 'draft' | 'running' | 'awaiting_verification' | 'paused' | 'interrupted' | 'needs_review' | 'completed';
interface GoalContract {
  outcome: string; verification: string; constraints: string; boundaries: string; stopWhen: string;
}
interface GoalRecord {
  sessionId: string; contract: GoalContract; state: GoalState; attempts: number;
  lastRunId: string | null; note: string;
}
const props = defineProps<{
  sessionId: string | null;
  running: boolean;
  configured: boolean;
  desktop: boolean;
  permissionMode: 'ask' | 'auto';
}>();
const emit = defineEmits<{ (e: 'session-created', id: string): void; (e: 'changed'): void }>();
const goal = ref<GoalRecord | null>(null);
const outcome = ref('');
const verification = ref('');
const constraints = ref('');
const boundaries = ref('');
const stopWhen = ref('');
const evidence = ref('');
const busy = ref(false);
let disposed = false;
let stopEvents: () => void = () => {};
onMounted(async () => {
  const stop = await listen<{ session: string; goal: GoalRecord | null }>('chat-goal-changed', payload => {
    if (!disposed && props.sessionId === payload.session) goal.value = payload.goal;
  });
  if (disposed) stop(); else stopEvents = stop;
});
onUnmounted(() => { disposed = true; stopEvents(); });
const error = ref('');
const stateLabel: Record<GoalState, string> = {
  draft: '待开始', running: '执行中', awaiting_verification: '等待验收',
  paused: '已暂停', interrupted: '异常中断', needs_review: '需人工核验', completed: '已完成',
};
async function refresh() {
  const id = props.sessionId;
  error.value = '';
  if (!id || !props.desktop) { goal.value = null; return; }
  try {
    const value = await invoke<GoalRecord | null>('chat_goal_get', { sessionId: id });
    if (props.sessionId !== id) return;
    goal.value = value;
    if (value) {
      outcome.value = value.contract.outcome;
      verification.value = value.contract.verification;
      constraints.value = value.contract.constraints;
      boundaries.value = value.contract.boundaries;
      stopWhen.value = value.contract.stopWhen;
    }
  } catch (e) { if (props.sessionId === id) error.value = String(e); }
}
watch(() => props.sessionId, () => { void refresh(); }, { immediate: true });
watch(() => props.running, (running, wasRunning) => {
  if (wasRunning && !running) void refresh();
});
async function act<T>(work: () => Promise<T>, apply: (value: T) => void) {
  if (busy.value) return;
  busy.value = true;
  error.value = '';
  try { apply(await work()); emit('changed'); }
  catch (e) { error.value = String(e); }
  finally { busy.value = false; }
}
async function save() {
  if (!outcome.value.trim()) { error.value = '请填写目标'; return; }
  await act(async () => {
    let id = props.sessionId;
    if (!id) {
      const session = await invoke<SessionSummary>('chat_new_session', { permissionMode: props.permissionMode });
      id = session.id;
      emit('session-created', id);
    }
    return invoke<GoalRecord>('chat_goal_set', { sessionId: id, contract: {
      outcome: outcome.value.trim(), verification: verification.value.trim(), constraints: constraints.value.trim(),
      boundaries: boundaries.value.trim(), stopWhen: stopWhen.value.trim(),
    } });
  }, record => { goal.value = record; });
}
async function start() {
  const id = props.sessionId;
  if (!id) return;
  await act(() => invoke<GoalRecord>('chat_goal_start', { sessionId: id }), value => { goal.value = value; });
}
async function pause() {
  const id = props.sessionId;
  if (!id) return;
  await act(() => invoke<GoalRecord>('chat_goal_pause', { sessionId: id }), value => { goal.value = value; });
}
async function acknowledge() {
  const id = props.sessionId;
  if (!id || !evidence.value.trim()) return;
  await act(() => invoke<GoalRecord>('chat_goal_acknowledge', { sessionId: id, evidence: evidence.value.trim() }), value => {
    goal.value = value; evidence.value = '';
  });
}
async function finish() {
  const id = props.sessionId;
  if (!id || !evidence.value.trim()) return;
  await act(() => invoke<GoalRecord>('chat_goal_finish', { sessionId: id, evidence: evidence.value.trim() }), value => {
    goal.value = value; evidence.value = '';
  });
}
async function clear() {
  const id = props.sessionId;
  if (!id) return;
  await act(() => invoke<void>('chat_goal_delete', { sessionId: id }), () => { goal.value = null; });
}
</script>

<template>
  <section class="goal-panel" data-testid="chat-goal-panel" aria-label="长期目标">
    <header class="goal-top">
      <div>
        <strong>长期目标</strong>
        <span v-if="goal" class="state" data-testid="goal-state">{{ stateLabel[goal.state] }}</span>
        <p>目标与验收条件会保存到此对话；当前版本每次继续都需要你明确启动。</p>
      </div>
      <NButton size="tiny" quaternary :disabled="busy || !sessionId" @click="refresh">刷新</NButton>
    </header>
    <template v-if="!goal || goal.state === 'draft' || goal.state === 'completed'">
      <NInput v-model:value="outcome" type="textarea" :autosize="{ minRows: 2, maxRows: 3 }" placeholder="目标：例如修复项目测试并交付可审核的代码" data-testid="goal-outcome" :disabled="!!goal && goal.state === 'draft'"/>
      <div class="two-cols">
        <NInput v-model:value="verification" placeholder="验收：例如 cargo test 全部通过" data-testid="goal-verification" :disabled="!!goal && goal.state === 'draft'"/>
        <NInput v-model:value="boundaries" placeholder="范围：允许修改的目录或模块" :disabled="!!goal && goal.state === 'draft'"/>
      </div>
      <div class="two-cols">
        <NInput v-model:value="constraints" placeholder="约束：不能破坏的功能" :disabled="!!goal && goal.state === 'draft'"/>
        <NInput v-model:value="stopWhen" placeholder="停止条件：例如需要新密钥时" :disabled="!!goal && goal.state === 'draft'"/>
      </div>
      <NButton v-if="!goal || goal.state === 'completed'" type="primary" size="small" :disabled="busy || !desktop || !configured || !outcome.trim()" @click="save">保存目标</NButton>
      <NButton v-if="goal?.state === 'draft'" type="primary" size="small" :disabled="busy || !desktop || !configured" data-testid="goal-start" @click="start">开始执行</NButton>
    </template>
    <template v-else>
      <p class="contract"><b>目标：</b>{{ goal.contract.outcome }}</p>
      <p v-if="goal.contract.verification" class="contract"><b>验收：</b>{{ goal.contract.verification }}</p>
    </template>
    <p v-if="goal?.note" class="note">{{ goal.note }}</p>
    <template v-if="goal && ['awaiting_verification', 'needs_review', 'interrupted'].includes(goal.state)">
      <NInput v-model:value="evidence" type="textarea" :autosize="{ minRows: 2, maxRows: 3 }"
        placeholder="写下你实际检查的测试结果、产物或中断后的现场状态" data-testid="goal-evidence"/>
    </template>
    <div v-if="goal" class="actions">
      <NButton v-if="goal.state === 'running'" size="small" :disabled="busy" @click="pause">暂停并停止当前执行</NButton>
      <NButton v-if="goal.state === 'paused' || goal.state === 'awaiting_verification'" size="small" :disabled="busy || running || !configured" @click="start">继续目标（新回合）</NButton>
      <NButton v-if="goal.state === 'awaiting_verification'" type="primary" size="small" :disabled="busy || running || !evidence.trim()" @click="finish">根据证据确认完成</NButton>
      <NButton v-if="goal.state === 'needs_review' || goal.state === 'interrupted'" size="small" :disabled="busy || !evidence.trim()" @click="acknowledge">已核验中断现场</NButton>
      <NButton v-if="!running && goal.state !== 'running'" quaternary size="small" :disabled="busy" @click="clear">清除目标</NButton>
    </div>
    <p v-if="error" role="alert" class="error" data-testid="goal-error">{{ error }}</p>
  </section>
</template>
<style scoped>
.goal-panel{display:flex;flex-direction:column;gap:10px;padding:16px 20px;border-bottom:1px solid var(--ui-line,#ddd);background:var(--ui-bg-secondary,transparent)}
.goal-top{display:flex;align-items:flex-start;justify-content:space-between;gap:14px}
.goal-top strong{font-size:14px}.goal-top p{margin:5px 0 0;color:var(--ui-text-tertiary,#888);font-size:12px}
.state{display:inline-block;margin-left:9px;font-size:11px;color:var(--ui-text-secondary,#666)}
.two-cols{display:grid;grid-template-columns:1fr 1fr;gap:10px}
.contract{font-size:12px;margin:0;overflow-wrap:anywhere;color:var(--ui-text-secondary,#666)}
.note{margin:0;font-size:12px;color:var(--ui-text-secondary,#666)}
.actions{display:flex;flex-wrap:wrap;gap:7px}.error{margin:0;color:var(--ui-error,#c33);font-size:12px}
@media(max-width:700px){.two-cols{grid-template-columns:1fr}}
</style>
