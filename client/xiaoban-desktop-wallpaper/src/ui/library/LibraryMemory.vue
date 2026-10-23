<script setup lang="ts">
import { useConfirm } from './confirm.ts';
import { computed, ref } from 'vue';
import { NButton, NInput, NModal, NSwitch } from 'naive-ui';
import { Flag, Lightbulb, NotebookPen, Pencil, ShieldCheck, Sparkles, Trash2 } from 'lucide-vue-next';
import { formatWhen, searchText, type Experience, type MemoryGoal, type MemoryNote, type LibraryStore } from './library-store.ts';

const props = defineProps<{ library: LibraryStore; query: string; desktop: boolean; sessionTitle: (id: string) => string }>();
const emit = defineEmits<{ openSession: [id: string] }>();
const confirm = useConfirm();

const memory = computed(() => props.library.memory);
const match = (...fields: string[]) => searchText(props.query, ...fields);
const candidates = computed(() => (memory.value?.candidates ?? []).filter(e => match(e.content, e.evidence)));
const notes = computed(() => (memory.value?.notes ?? []).filter(n => match(n.text, n.name)));
const approved = computed(() => (memory.value?.approved ?? []).filter(e => match(e.content)));
const goals = computed(() => (memory.value?.goals ?? []).filter(g => match(g.outcome, props.sessionTitle(g.sessionId))));
const GOAL_STATES: Record<string, string> = { draft: '草稿', running: '进行中', awaiting_verification: '等待验收', paused: '已暂停', interrupted: '已中断', needs_review: '需要你看看', completed: '已完成' };
const experienceKind = (e: Experience) => e.kind === 'failure' ? '踩过的坑' : '做事方法';

/* ── Writing and correcting ── */
type Editing = { mode: 'note'; note: MemoryNote | null } | { mode: 'experience'; experience: Experience; approveAfter: boolean };
const editing = ref<Editing | null>(null), text = ref('');
function editNote(note: MemoryNote | null) { editing.value = { mode: 'note', note }; text.value = note?.text ?? ''; }
function editExperience(experience: Experience, approveAfter = false) { editing.value = { mode: 'experience', experience, approveAfter }; text.value = experience.content; }
const limit = computed(() => editing.value?.mode === 'note' ? 2000 : 500);
async function saveEdit() {
  const job = editing.value;
  if (!job || !text.value.trim()) return;
  const ok = job.mode === 'note'
    ? await props.library.writeNote(job.note?.name ?? null, text.value)
    : await props.library.editExperience(job.experience.id, text.value) && (!job.approveAfter || await props.library.review(job.experience.id, true));
  if (ok) editing.value = null;
}
function forgetNote(note: MemoryNote) {
  confirm({ title: '忘掉这条记忆？', content: note.preview, positiveText: '忘掉', negativeText: '取消', onPositiveClick: () => props.library.deleteNote(note.name) });
}
function forgetExperience(experience: Experience) {
  confirm({ title: '忘掉这条经验？', content: experience.content.slice(0, 80), positiveText: '忘掉', negativeText: '取消', onPositiveClick: () => props.library.forgetExperience(experience.id) });
}
function clearAll() {
  confirm({ danger: true,
    title: '清空全部记忆？', content: '小伴记下的笔记和做事经验都会删除，不能恢复。对话里定下的目标会保留。', positiveText: '全部清空', negativeText: '取消',
    onPositiveClick: () => props.library.clearMemory(),
  });
}
const goalTitle = (goal: MemoryGoal) => props.sessionTitle(goal.sessionId) || '一段对话';
defineExpose({ write: () => editNote(null) });
</script>

<template>
  <div class="memory" data-testid="library-memory">
    <div class="columns">
      <div class="lib-scroll list">
        <p v-if="!memory" class="lib-muted nothing">{{ desktop ? '正在读取记忆…' : '请在小伴桌面客户端中查看记忆。' }}</p>
        <template v-else>
          <section v-if="candidates.length" class="pending" aria-label="等你确认" data-testid="memory-pending">
            <span class="lib-section">等你确认<span class="lib-pill accent">{{ candidates.length }}</span><span class="lib-muted plain">小伴从做过的事里总结的，确认后才会用到</span></span>
            <article v-for="e in candidates" :key="e.id" class="ask">
              <Sparkles :size="16" class="ask-icon" />
              <span class="ask-text"><b>{{ e.content }}</b><span class="lib-muted">{{ experienceKind(e) }} · {{ formatWhen(e.createdMs) }} · 依据：{{ e.evidence }}</span></span>
              <button type="button" class="lib-link muted" :data-testid="`memory-dismiss-${e.id}`" @click="library.review(e.id, false)">不用记</button>
              <button type="button" class="xb-btn is-sm" @click="editExperience(e, true)">改一下</button>
              <button type="button" class="xb-btn-primary is-sm" :data-testid="`memory-keep-${e.id}`" @click="library.review(e.id, true)">记住</button>
            </article>
          </section>

          <section class="group" aria-label="小伴的笔记">
            <span class="lib-section"><NotebookPen :size="14" />小伴的笔记<span class="lib-muted plain">{{ notes.length }}</span></span>
            <div v-for="note in notes" :key="note.name" class="item" :data-testid="`memory-note-${note.name}`">
              <span class="item-text"><span class="main-text">{{ note.text }}</span></span>
              <button type="button" class="lib-icon-btn" aria-label="修改这条记忆" @click="editNote(note)"><Pencil :size="14" /></button>
              <button type="button" class="lib-icon-btn" aria-label="忘掉这条记忆" @click="forgetNote(note)"><Trash2 :size="14" /></button>
            </div>
            <p v-if="!notes.length" class="lib-muted empty">{{ query.trim() ? '没有找到' : '还没有。小伴会在对话里记下值得长期记住的事，你也可以点「写一条」告诉它。' }}</p>
          </section>

          <section class="group" aria-label="做事经验">
            <span class="lib-section"><Lightbulb :size="14" />做事经验<span class="lib-muted plain">{{ approved.length }}</span></span>
            <div v-for="e in approved" :key="e.id" class="item">
              <span class="item-text"><span class="main-text">{{ e.content }}</span><span class="lib-muted">{{ experienceKind(e) }} · {{ formatWhen(e.createdMs) }}</span></span>
              <button type="button" class="lib-icon-btn" aria-label="修改这条经验" @click="editExperience(e)"><Pencil :size="14" /></button>
              <button type="button" class="lib-icon-btn" aria-label="忘掉这条经验" @click="forgetExperience(e)"><Trash2 :size="14" /></button>
            </div>
            <p v-if="!approved.length" class="lib-muted empty">还没有。完成任务后可以让小伴复盘，总结出的经验经你确认后放在这里。</p>
          </section>

          <section class="group" aria-label="长期目标">
            <span class="lib-section"><Flag :size="14" />长期目标<span class="lib-muted plain">{{ goals.length }}</span></span>
            <div v-for="g in goals" :key="g.sessionId" class="item">
              <span class="item-text"><span class="main-text">{{ g.outcome }}</span><span class="lib-muted">{{ GOAL_STATES[g.state] ?? g.state }} · 定于「{{ goalTitle(g) }}」 · {{ formatWhen(g.updatedAtMs) }}</span></span>
              <button type="button" class="lib-link" @click="emit('openSession', g.sessionId)">打开对话</button>
            </div>
            <p v-if="!goals.length" class="lib-muted empty">还没有。在对话里给小伴定一个目标，它会一直记着，直到完成。</p>
          </section>
        </template>
      </div>

      <aside class="side" aria-label="关于记忆">
        <div class="lib-card box">
          <div class="switch-row"><b>让小伴记住我</b><NSwitch :value="memory?.remembering ?? true" :disabled="!memory || library.busy" aria-label="让小伴记住我" data-testid="memory-switch" @update:value="library.setRemembering($event)" /></div>
          <span class="lib-muted">{{ memory?.remembering === false ? '已关闭：小伴不再记新的内容，已有的记忆也暂时不用。' : '小伴会记下你的偏好和做事习惯，用在以后的回答里。' }}</span>
        </div>
        <div class="lib-card box how">
          <span class="lib-section"><ShieldCheck :size="14" />记忆怎么用</span>
          <span>· 只保存在这台电脑上，不会上传</span>
          <span>· 小伴总结的经验，要你确认后才会用</span>
          <span>· 每一条都能改、能删；关掉开关就都不用了</span>
          <span>· 目标属于各自的对话，完成后自动归档</span>
        </div>
        <button type="button" class="lib-link danger clear" :disabled="!memory" data-testid="memory-clear" @click="clearAll">清空全部记忆…</button>
      </aside>
    </div>

    <NModal :show="!!editing" preset="card" :title="editing?.mode === 'note' ? (editing.note ? '修改记忆' : '写一条记忆') : '修改经验'" style="max-width: 480px; width: 92vw;" @update:show="value => { if (!value) editing = null; }">
      <form class="lib-form" @submit.prevent="saveEdit">
        <NInput v-model:value="text" type="textarea" :autosize="{ minRows: 3, maxRows: 8 }" :maxlength="limit" show-count :placeholder="editing?.mode === 'note' ? '比如：回答先给结论，再给理由' : ''" data-testid="memory-text" />
        <div class="lib-form-actions">
          <NButton @click="editing = null">取消</NButton>
          <NButton type="primary" attr-type="submit" :loading="library.busy" :disabled="!text.trim()" data-testid="memory-save">{{ editing?.mode === 'experience' && editing.approveAfter ? '改好并记住' : '保存' }}</NButton>
        </div>
      </form>
    </NModal>
  </div>
</template>

<style scoped>
.memory{flex:1;min-height:0;display:flex;flex-direction:column}
.columns{flex:1;min-height:0;display:flex;gap:20px}
.list{padding:16px 0 24px;display:flex;flex-direction:column;gap:18px}
.plain{font-weight:400}
.pending{display:flex;flex-direction:column;gap:10px}
.ask{display:flex;align-items:center;gap:12px;padding:12px 14px;border-radius:12px;background:var(--warn-bg,#FFFCF6);box-shadow:inset 0 0 0 1px #F1DDB8}
.ask-icon{flex:none;color:#B26B00}
.ask-text{flex:1;min-width:0;display:flex;flex-direction:column;gap:2px}
.ask-text b{font-size:13.5px;font-weight:500;color:var(--ui-text-primary)}
.ask-text .lib-muted{white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.group{display:flex;flex-direction:column;gap:2px}
.group .lib-section{padding:0 2px 6px}
.item{display:flex;align-items:center;gap:8px;padding:10px 12px;border-radius:10px;box-shadow:inset 0 -1px 0 var(--ui-line)}
.item-text{flex:1;min-width:0;display:flex;flex-direction:column;gap:2px}
.main-text{font-size:13.5px;color:var(--ui-text-primary);white-space:pre-line;display:-webkit-box;-webkit-line-clamp:3;-webkit-box-orient:vertical;overflow:hidden}
.empty{padding:6px 12px 2px;line-height:1.6}
.nothing{padding:64px 0;text-align:center;font-size:13px}
.side{width:280px;flex:none;padding:16px 0 18px;display:flex;flex-direction:column;gap:12px}
.box{padding:14px;display:flex;flex-direction:column;gap:10px}
.box:hover{box-shadow:0 0 0 1px var(--ui-line),0 1px 2px rgba(0,0,0,.03)}
.switch-row{display:flex;align-items:center;gap:10px}
.switch-row b{flex:1;font-size:13.5px;font-weight:600}
.box .lib-muted{line-height:1.65}
.how{gap:8px;font-size:12.5px;line-height:1.65;color:var(--ui-text-secondary)}
.clear{align-self:flex-start;margin-top:auto}
</style>
