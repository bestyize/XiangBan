<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import { NButton, NInput, NSwitch } from 'naive-ui';
import { ArrowRight, Check, ChevronDown, Clock3, MessageCircle, RefreshCw, SlidersHorizontal, Target, X } from 'lucide-vue-next';
import { invoke, listen } from '../../bridge.ts';
import type { SessionSummary } from './types.ts';

type GoalState = 'draft' | 'running' | 'awaiting_verification' | 'paused' | 'interrupted' | 'needs_review' | 'completed';
interface GoalContract {
  outcome: string; verification: string; constraints: string; boundaries: string; stopWhen: string;
  verificationCommand?: string; autoResume?: boolean;
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
  sessionTitle?: string;
}>();
const emit = defineEmits<{ (e: 'session-created', id: string): void; (e: 'changed'): void; (e: 'close'): void; (e: 'busy-change', value: boolean): void }>();
const goal = ref<GoalRecord | null>(null);
const outcome = ref('');
const verification = ref('');
const constraints = ref('');
const boundaries = ref('');
const stopWhen = ref('');
const verificationCommand = ref('');
const autoResume = ref(false);
const evidence = ref('');
const busy = ref(false);
const creatingNew = ref(false);
const editing = computed(() => !goal.value || goal.value.state === 'draft' || creatingNew.value);
const inputTheme = {
  borderRadius: '9px', heightMedium: '44px', fontSizeMedium: '12px',
  color: 'var(--ui-surface-sunken, #fafafa)', colorFocus: 'var(--ui-surface, #fff)',
  textColor: 'var(--ui-text-primary, #262629)', placeholderColor: 'var(--ui-text-tertiary, #a2a2a8)',
  border: '1px solid var(--ui-border, #e1e1e5)', borderHover: '1px solid var(--ui-border-strong, #ccc)',
  borderFocus: '1px solid var(--ui-btn, #bf5362)', boxShadowFocus: '0 0 0 3px var(--ui-soft, #fbf0f2)',
};
watch(busy, value => emit('busy-change', value), { flush: 'sync' });
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
      verificationCommand.value = value.contract.verificationCommand ?? '';
      autoResume.value = value.contract.autoResume ?? false;
    }
  } catch (e) { if (props.sessionId === id) error.value = String(e); }
}
watch(() => props.sessionId, () => { evidence.value = ''; creatingNew.value = false; void refresh(); }, { immediate: true });
watch(() => props.running, (running, wasRunning) => {
  if (wasRunning && !running) void refresh();
});
async function act<T>(work: () => Promise<T>, apply: (value: T) => void) {
  if (busy.value) return;
  busy.value = true;
  error.value = '';
  const session = props.sessionId;
  try {
    const result = await work();
    if (session !== null && session !== props.sessionId) return;
    apply(result); emit('changed');
  }
  catch (e) { if (session === props.sessionId) error.value = String(e); }
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
      verificationCommand: verificationCommand.value.trim(), autoResume: autoResume.value,
    } });
  }, record => { goal.value = record; creatingNew.value = false; });
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
function newGoal() {
  outcome.value = verification.value = constraints.value = boundaries.value = stopWhen.value = verificationCommand.value = evidence.value = '';
  autoResume.value = false;
  creatingNew.value = true;
}
</script>

<template>
  <section class="goal-dialog" data-testid="chat-goal-panel" role="dialog" aria-modal="true" aria-labelledby="goal-dialog-title" aria-describedby="goal-dialog-description" tabindex="-1">
    <header class="goal-header">
      <div class="goal-heading">
        <h2 id="goal-dialog-title"><Target :size="20" :stroke-width="1.7" />长期目标</h2>
        <p id="goal-dialog-description">{{ editing ? '设定目标与验收标准，在这段对话中持续推进。' : '查看当前进度，管理这段对话中的长期目标。' }}</p>
      </div>
      <button type="button" class="goal-icon-button" aria-label="关闭长期目标" title="关闭" :disabled="busy" data-testid="goal-close" @click="emit('close')"><X :size="17" /></button>
    </header>

    <div class="goal-body">
      <div class="goal-context"><MessageCircle :size="13" /><span>当前对话</span><i /><b>{{ sessionTitle || '新对话' }}</b><button v-if="sessionId" type="button" class="goal-refresh" aria-label="刷新目标" title="刷新目标" :disabled="busy" @click="refresh"><RefreshCw :size="13" /></button></div>

      <template v-if="editing">
        <label class="goal-field">
          <span>想完成什么</span>
          <NInput v-model:value="outcome" type="textarea" :autosize="{ minRows: 4, maxRows: 6 }" :theme-overrides="inputTheme" placeholder="描述你期待的最终结果" data-testid="goal-outcome" :readonly="goal?.state === 'draft'" />
        </label>
        <label class="goal-field">
          <span>怎样算完成<small>选填</small></span>
          <NInput v-model:value="verification" :theme-overrides="inputTheme" placeholder="写下你会如何检查结果" data-testid="goal-verification" :readonly="goal?.state === 'draft'" />
        </label>
        <details class="goal-options" data-testid="goal-options">
          <summary><SlidersHorizontal :size="14" />更多设置<small>执行范围、自动验收与恢复</small><ChevronDown :size="14" class="goal-chevron" /></summary>
          <div class="goal-options-content">
            <div class="goal-columns">
              <label class="goal-field"><span>允许范围</span><NInput v-model:value="boundaries" :theme-overrides="inputTheme" placeholder="例如：仅修改桌面客户端" :readonly="goal?.state === 'draft'" /></label>
              <label class="goal-field"><span>必须遵守</span><NInput v-model:value="constraints" :theme-overrides="inputTheme" placeholder="例如：保持现有接口兼容" :readonly="goal?.state === 'draft'" /></label>
            </div>
            <label class="goal-field"><span>何时停下来</span><NInput v-model:value="stopWhen" :theme-overrides="inputTheme" placeholder="例如：需要新密钥或人工决定时" :readonly="goal?.state === 'draft'" /></label>
            <label class="goal-field"><span>自动验收命令</span><NInput v-model:value="verificationCommand" :theme-overrides="inputTheme" placeholder="例如：npm test" data-testid="goal-verification-command" :readonly="goal?.state === 'draft'" /><small class="goal-hint">按当前权限执行；留空则由你确认完成。</small></label>
            <div class="goal-resume">
              <div><span>重新打开小伴后继续</span><p>仅恢复安全中断的目标，主动暂停不会自动继续。小伴关闭期间不执行。</p></div>
              <NSwitch v-model:value="autoResume" size="small" :disabled="goal?.state === 'draft'" data-testid="goal-auto-resume" aria-label="重新打开小伴后继续" />
            </div>
          </div>
        </details>
        <p v-if="goal?.state === 'draft'" class="goal-hint goal-draft-hint">目标已保存，开始后小伴会按以上约定执行。</p>
      </template>

      <div v-else-if="goal?.state === 'completed'" class="goal-completed" data-testid="goal-completed">
        <div class="goal-completed-icon"><Check :size="24" /></div>
        <h3>目标已确认完成</h3>
        <p>这次的目标与验收记录，会保留在当前对话中。</p>
        <span class="goal-state" data-state="completed" data-testid="goal-state">{{ stateLabel[goal.state] }}</span>
      </div>

      <template v-else-if="goal">
        <div class="goal-status-line">
          <span class="goal-state" :data-state="goal.state" data-testid="goal-state"><i />{{ stateLabel[goal.state] }}</span>
          <span v-if="goal.attempts" class="goal-attempts">第 {{ goal.attempts }} 次推进</span>
        </div>
        <h3 class="goal-summary">{{ goal.contract.outcome }}</h3>
        <dl class="goal-facts">
          <template v-if="goal.contract.verification"><dt>验收标准</dt><dd>{{ goal.contract.verification }}</dd></template>
          <template v-if="goal.contract.boundaries"><dt>执行范围</dt><dd>{{ goal.contract.boundaries }}</dd></template>
          <template v-if="goal.contract.constraints"><dt>执行约束</dt><dd>{{ goal.contract.constraints }}</dd></template>
          <template v-if="goal.contract.stopWhen"><dt>停止条件</dt><dd>{{ goal.contract.stopWhen }}</dd></template>
        </dl>
        <div class="goal-note"><Clock3 :size="14" /><p>{{ goal.note || (goal.state === 'running' ? '小伴正在处理这个目标。关闭弹窗后，任务会继续执行。' : goal.state === 'paused' ? '目标已暂停。点击继续执行，可从当前进度启动新一轮。' : goal.state === 'awaiting_verification' ? '这一轮已结束。请检查交付结果，再决定继续推进或确认完成。' : '请先核验中断现场，记录结果后再继续推进。') }}</p></div>
        <div v-if="['awaiting_verification', 'needs_review', 'interrupted'].includes(goal.state)" class="goal-evidence">
          <label class="goal-field"><span>{{ goal.state === 'awaiting_verification' ? '你的验收记录' : '中断核验记录' }}</span><NInput v-model:value="evidence" type="textarea" :theme-overrides="inputTheme" :autosize="{ minRows: 3, maxRows: 5 }" placeholder="记录你实际检查的测试结果、报告或交付文件。" data-testid="goal-evidence" /></label>
          <p class="goal-hint">{{ goal.state === 'awaiting_verification' ? '小伴的回答完成，不代表目标已经通过验收。' : '先确认已有操作的实际结果，避免重复执行。' }}</p>
        </div>
      </template>
      <p v-if="error" role="alert" class="goal-error" data-testid="goal-error">{{ error }}</p>
    </div>

    <footer class="goal-footer">
      <div class="goal-footer-meta"><MessageCircle :size="12" /><span>{{ goal ? '进度已保存在此对话' : '保存到当前对话' }}</span><button v-if="goal && !running && goal.state !== 'running'" type="button" class="goal-clear" :disabled="busy" @click="clear">清除目标</button></div>
      <div class="goal-actions">
        <template v-if="editing">
          <NButton :disabled="busy" @click="emit('close')">{{ goal?.state === 'draft' ? '关闭' : '取消' }}</NButton>
          <NButton v-if="goal?.state === 'draft'" type="primary" :loading="busy" :disabled="busy || !desktop || !configured" data-testid="goal-start" @click="start">开始执行<ArrowRight :size="13" /></NButton>
          <NButton v-else type="primary" :loading="busy" :disabled="busy || !desktop || !configured || !outcome.trim()" @click="save">保存目标<ArrowRight :size="13" /></NButton>
        </template>
        <template v-else-if="goal?.state === 'completed'">
          <NButton :disabled="busy" @click="newGoal">新建目标</NButton><NButton type="primary" @click="emit('close')">回到对话<ArrowRight :size="13" /></NButton>
        </template>
        <template v-else>
          <NButton v-if="goal?.state === 'running'" text :disabled="busy" @click="pause">暂停执行</NButton>
          <NButton v-if="goal?.state === 'paused' || goal?.state === 'awaiting_verification'" :text="goal?.state === 'awaiting_verification'" :type="goal?.state === 'paused' ? 'primary' : 'default'" :loading="busy" :disabled="busy || running || !configured" @click="start">{{ goal?.state === 'paused' ? '继续执行' : '继续推进' }}</NButton>
          <NButton v-if="goal?.state === 'awaiting_verification'" type="primary" :loading="busy" :disabled="busy || running || !evidence.trim()" data-testid="goal-finish" @click="finish">确认完成</NButton>
          <NButton v-else-if="goal?.state === 'needs_review' || goal?.state === 'interrupted'" type="primary" :loading="busy" :disabled="busy || !evidence.trim()" @click="acknowledge">已核验中断现场</NButton>
          <NButton v-else :type="goal?.state === 'running' ? 'primary' : 'default'" :disabled="busy" @click="emit('close')">回到对话<ArrowRight :size="13" /></NButton>
        </template>
      </div>
    </footer>
  </section>
</template>

<style scoped>
.goal-dialog{display:flex;flex-direction:column;width:min(568px,calc(100vw - 40px));max-height:calc(100dvh - 64px);background:var(--ui-surface,#fff);color:var(--ui-text-primary,#262629);border-radius:18px;box-shadow:var(--ui-shadow-dialog,0 32px 100px -18px #19151d38,0 8px 28px -12px #19151d24,0 0 0 1px #00000007);overflow:hidden;outline:none}
:global(.n-modal-container:has(.goal-dialog) .n-modal-mask){backdrop-filter:blur(4px)}
.goal-header{display:flex;align-items:flex-start;gap:12px;padding:28px 30px 20px;flex:none}.goal-heading{flex:1;min-width:0}.goal-heading h2{font-size:20px;line-height:1.4;letter-spacing:-.5px;font-weight:620;margin:0;display:flex;align-items:center;gap:9px}.goal-heading h2>svg{color:var(--ui-ink,#bf5362)}.goal-heading p{font-size:12px;line-height:1.6;color:var(--ui-text-secondary,#737378);margin:7px 0 0}
.goal-icon-button{display:grid;place-items:center;width:28px;height:28px;margin:-2px -7px 0 0;padding:0;border:0;border-radius:7px;background:none;color:var(--ui-text-tertiary,#a2a2a8);cursor:pointer;flex:none}.goal-icon-button:hover:not(:disabled){color:var(--ui-text-primary,#262629);background:var(--ui-surface-alt,#fafafa)}.goal-icon-button:disabled{opacity:.4;cursor:not-allowed}.goal-icon-button:focus-visible,.goal-refresh:focus-visible,.goal-clear:focus-visible,.goal-options summary:focus-visible{outline:2px solid var(--ui-btn,#bf5362);outline-offset:4px}
.goal-body{min-height:0;padding:0 30px 24px;overflow-y:auto;overscroll-behavior:contain;scrollbar-width:thin;scrollbar-color:var(--ui-border,#e1e1e5) transparent}.goal-context{display:flex;align-items:center;gap:6px;padding-bottom:23px;color:var(--ui-text-tertiary,#a2a2a8);font-size:11px;min-width:0}.goal-context>svg{flex:none}.goal-context>span{white-space:nowrap}.goal-context i{width:2px;height:2px;flex:none;border-radius:50%;background:currentColor;margin:0 3px}.goal-context b{font-weight:400;color:var(--ui-text-secondary,#737378);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}.goal-refresh{display:grid;place-items:center;flex:none;width:22px;height:22px;margin:-3px -3px -3px auto;border:0;border-radius:5px;background:none;color:var(--ui-text-tertiary,#a2a2a8);cursor:pointer}.goal-refresh:hover:not(:disabled){background:var(--ui-control,#eee);color:var(--ui-text-primary,#262629)}
.goal-field{display:flex;flex-direction:column;gap:8px;margin:0 0 19px;min-width:0}.goal-field>span{display:flex;align-items:center;justify-content:space-between;font-size:12px;line-height:1.6;font-weight:560}.goal-field>span>small{font-size:11px;font-weight:400;color:var(--ui-text-tertiary,#a2a2a8)}.goal-field :deep(.n-input__textarea-el){min-height:104px;line-height:1.8;padding-block:10px}.goal-field :deep(.n-input__input-el){line-height:1.8}.goal-hint{font-size:11px;line-height:1.7;color:var(--ui-text-tertiary,#a2a2a8);margin:8px 0 0}.goal-draft-hint{margin-top:14px}.goal-columns{display:grid;grid-template-columns:1fr 1fr;gap:13px}
.goal-options{margin-top:25px;border-top:1px solid var(--ui-line,#eaeaec)}.goal-options summary{display:flex;align-items:center;gap:8px;padding:17px 0 1px;cursor:pointer;list-style:none;color:var(--ui-text-secondary,#737378);font-size:12px;line-height:1.6}.goal-options summary::-webkit-details-marker{display:none}.goal-options summary>svg{color:var(--ui-text-tertiary,#a2a2a8)}.goal-options summary small{font-weight:400;font-size:11px;color:var(--ui-text-tertiary,#a2a2a8);margin-left:2px}.goal-chevron{margin-left:auto;transition:transform .2s}.goal-options[open] .goal-chevron{transform:rotate(180deg)}.goal-options-content{padding-top:21px}.goal-options .goal-field{margin-bottom:17px}.goal-options .goal-hint{margin-top:0}.goal-resume{display:flex;align-items:center;justify-content:space-between;gap:22px;margin-top:21px;padding:17px 0 0;border-top:1px solid var(--ui-line,#eaeaec)}.goal-resume>div>span{font-size:12px;font-weight:500}.goal-resume p{font-size:11px;line-height:1.7;color:var(--ui-text-tertiary,#a2a2a8);margin:4px 0 0}.goal-resume :deep(.n-switch){flex:none}
.goal-status-line{display:flex;align-items:center;justify-content:space-between;gap:12px;margin:0 0 23px}.goal-state{display:inline-flex;align-items:center;gap:6px;padding:4px 9px;border-radius:6px;color:var(--ui-text-secondary,#737378);background:var(--ui-surface-alt,#fafafa);font-size:11px;line-height:1.6}.goal-state i{width:5px;height:5px;border-radius:50%;background:currentColor}.goal-state[data-state=running],.goal-state[data-state=completed]{color:var(--ui-success-text,#338365);background:color-mix(in srgb,var(--ui-success,#338365) 9%,var(--ui-surface,#fff))}.goal-state[data-state=awaiting_verification],.goal-state[data-state=needs_review],.goal-state[data-state=interrupted]{color:var(--ui-warning-text,#a87932);background:color-mix(in srgb,var(--ui-warning,#a87932) 9%,var(--ui-surface,#fff))}.goal-attempts{font-size:11px;color:var(--ui-text-tertiary,#a2a2a8)}.goal-summary{font-size:16px;font-weight:550;line-height:1.85;letter-spacing:-.15px;margin:0 0 25px;overflow-wrap:anywhere;white-space:pre-wrap}.goal-facts{display:grid;grid-template-columns:72px minmax(0,1fr);gap:13px 16px;font-size:12px;line-height:1.8;margin:0 0 23px}.goal-facts:empty{display:none}.goal-facts dt{color:var(--ui-text-tertiary,#a2a2a8)}.goal-facts dd{margin:0;color:var(--ui-text-secondary,#737378);overflow-wrap:anywhere;white-space:pre-wrap}.goal-note{display:flex;align-items:flex-start;gap:8px;padding:13px 15px;border-radius:9px;background:var(--ui-surface-sunken,#fafafa);color:var(--ui-text-secondary,#737378);font-size:11px;line-height:1.8}.goal-note>svg{flex:none;margin-top:3px;color:var(--ui-text-tertiary,#a2a2a8)}.goal-note p{margin:0;overflow-wrap:anywhere;white-space:pre-wrap}.goal-evidence{padding-top:23px;border-top:1px solid var(--ui-line,#eaeaec);margin-top:25px}.goal-evidence .goal-field{margin-bottom:0}.goal-error{margin:16px 0 0;font-size:12px;color:var(--ui-danger,#c33);line-height:1.7}.goal-completed{text-align:center;padding:18px 0 24px}.goal-completed-icon{width:45px;height:45px;margin:0 auto 17px;display:grid;place-items:center;border-radius:50%;color:var(--ui-success-text,#338365);background:color-mix(in srgb,var(--ui-success,#338365) 9%,var(--ui-surface,#fff))}.goal-completed h3{font-size:18px;font-weight:550;margin:0}.goal-completed p{font-size:12px;color:var(--ui-text-secondary,#737378);margin:9px 0 14px}
.goal-footer{padding:19px 30px;border-top:1px solid var(--ui-line,#eaeaec);display:flex;align-items:center;gap:9px;background:var(--ui-surface,#fff);flex:none}.goal-footer-meta{display:flex;align-items:center;flex:1;flex-wrap:wrap;gap:6px;font-size:11px;color:var(--ui-text-tertiary,#a2a2a8)}.goal-footer-meta>span{white-space:nowrap}.goal-clear{padding:0;border:0;background:none;font-size:10px;color:var(--ui-text-tertiary,#a2a2a8);cursor:pointer;text-decoration:underline;text-underline-offset:3px}.goal-clear:hover:not(:disabled){color:var(--ui-danger,#c33)}.goal-actions{display:flex;align-items:center;justify-content:flex-end;gap:9px;flex-wrap:wrap}.goal-actions :deep(.n-button){height:38px;padding:0 16px;border-radius:8px;font-size:12px;font-weight:500}.goal-actions :deep(.n-button--text-type){padding-inline:2px}.goal-actions :deep(.n-button__content){gap:9px}
@media(max-width:600px){.goal-dialog{width:calc(100vw - 28px);max-height:calc(100dvh - 40px)}.goal-header{padding:24px 24px 18px}.goal-body{padding-inline:24px}.goal-footer{padding:17px 24px}}
@media(max-width:440px){.goal-dialog{width:calc(100vw - 20px);max-height:calc(100dvh - 24px)}.goal-header{padding:21px 20px 17px}.goal-heading h2{font-size:19px}.goal-body{padding:0 20px 21px}.goal-footer{padding:16px 20px;flex-wrap:wrap}.goal-footer-meta{font-size:10px}.goal-actions :deep(.n-button){padding-inline:13px}.goal-columns{grid-template-columns:1fr;gap:0}.goal-facts{grid-template-columns:58px minmax(0,1fr);gap:12px}}
@media(prefers-reduced-motion:reduce){.goal-chevron{transition:none}}
</style>
