<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { ArrowDown, ArrowUp, Check, ChevronDown, Clock3, CornerDownLeft, FileText, Image, Pause, Pencil, Play, Trash2, X } from 'lucide-vue-next';
import { invoke } from '../../bridge.ts';
import type { ChatStore } from '../chat/chat-store.ts';
import type { QueueEntry } from '../chat/types.ts';
const props = defineProps<{ chat: ChatStore; running: boolean }>();
const queue = computed(() => props.chat.queue);
const expanded = ref(false);
const editing = ref<string | null>(null);
const editText = ref('');
const removedImages = ref<number[]>([]);
const removedFiles = ref<string[]>([]);
const busy = ref(false);
const dragging = ref<string | null>(null);
const entries = computed(() => expanded.value ? queue.value?.items ?? [] : (queue.value?.items ?? []).slice(0, 2));
watch(() => props.chat.currentId, () => { editing.value = null; expanded.value = false; });
const previews = ref<Record<string, string>>({});
const previewKey = (e: QueueEntry) => `${props.chat.currentId}:${e.id}:${e.images}`;
watch(() => entries.value.filter(e => e.images).map(previewKey).join('|'), () => {
  const session = props.chat.currentId;
  const visible = entries.value.filter(e => e.images);
  previews.value = Object.fromEntries(Object.entries(previews.value).filter(([key]) => visible.some(e => previewKey(e) === key)));
  for (const entry of visible) {
    const key = previewKey(entry);
    if (previews.value[key] || !session) continue;
    void invoke<string | null>('chat_queue_preview', { sessionId: session, id: entry.id }).then(url => {
      if (props.chat.currentId === session && queue.value?.items.some(e => e.id === entry.id && e.images === entry.images) && typeof url === 'string' && url.startsWith('data:image/')) previews.value[key] = url;
    }).catch(() => {});
  }
}, { immediate: true });

async function begin(entry: QueueEntry) {
  if (busy.value) return;
  const session = props.chat.currentId;
  if (!session) return;
  busy.value = true;
  try {
    if (!await props.chat.updateQueue('pause', undefined, undefined, session) || props.chat.currentId !== session) return;
    const current = props.chat.queue?.items.find(e => e.id === entry.id && !e.dispatching);
    if (!current) return;
    editing.value = current.id; editText.value = current.text; removedImages.value = []; removedFiles.value = [];
  } finally { busy.value = false; }
}
async function act(action: Parameters<ChatStore['updateQueue']>[0], id?: string, options?: Parameters<ChatStore['updateQueue']>[2]) {
  if (busy.value) return;
  const session = props.chat.currentId;
  if (!session) return;
  busy.value = true;
  try { if (await props.chat.updateQueue(action, id, options, session)) { if (props.chat.currentId === session) editing.value = null; } }
  finally { busy.value = false; }
}
function save() { void act('edit', editing.value ?? undefined, { text: editText.value, removeImages: removedImages.value, removeFiles: removedFiles.value }); }
function move(id: string, before: string) {
  const ids = queue.value?.items.map(e => e.id) ?? [];
  const from = ids.indexOf(id), to = ids.indexOf(before);
  if (from < 0 || to < 0 || from === to) return;
  ids.splice(from, 1); ids.splice(to, 0, id);
  void act('reorder', undefined, { order: ids });
}
function step(id: string, delta: number) {
  const entries = queue.value?.items ?? [], index = entries.findIndex(e => e.id === id), other = entries[index + delta];
  if (other) move(id, other.id);
}
</script>
<template>
  <section v-if="queue?.items.length" class="message-queue" aria-label="待发送消息" data-testid="chat-message-queue">
    <div class="queue-head">
      <button class="queue-summary" type="button" :aria-expanded="expanded" @click="expanded = !expanded"><Clock3 :size="13" /><span>{{ queue.paused ? '队列已暂停' : '已排队' }} · {{ queue.items.length }} 条</span><ChevronDown :size="13" :class="{ expanded }" /></button>
      <span class="queue-hint">{{ queue.paused ? queue.reason : queue.items.some(e => e.mode === 'steer') ? '补充将在当前步骤结束后接收' : '当前任务结束后自动发送' }}</span>
      <button class="queue-action" type="button" :disabled="busy" :title="queue.paused ? '继续发送队列' : '暂停队列'" :aria-label="queue.paused ? '继续发送队列' : '暂停队列'" data-testid="chat-queue-toggle" @click="act(queue.paused ? 'resume' : 'pause')"><Play v-if="queue.paused" :size="13" /><Pause v-else :size="13" /></button>
    </div>
    <ol>
      <li v-for="(entry, index) in entries" :key="entry.id" :data-testid="`chat-queue-${entry.id}`" :draggable="!busy && !entry.dispatching && !editing" @dragstart="dragging = entry.id" @dragend="dragging = null" @dragover.prevent @drop.prevent="dragging && move(dragging, entry.id)">
        <template v-if="editing === entry.id">
          <div class="queue-edit">
            <textarea v-model="editText" aria-label="编辑排队消息" maxlength="30000" :disabled="busy" @keydown.ctrl.enter.prevent="save" @keydown.meta.enter.prevent="save" @keydown.esc.prevent="editing = null" />
            <div class="queue-files">
              <button v-for="n in entry.images" :key="`image-${n}`" type="button" :class="{ removed: removedImages.includes(n - 1) }" :disabled="busy" @click="removedImages.includes(n-1) ? removedImages = removedImages.filter(i => i !== n-1) : removedImages.push(n-1)"><Image :size="11" />图片 {{ n }}<X :size="10" /></button>
              <button v-for="file in entry.files.filter(f => !f.mime?.startsWith('image/'))" :key="file.id" type="button" :class="{ removed: removedFiles.includes(file.id ?? '') }" :disabled="busy" @click="removedFiles.includes(file.id ?? '') ? removedFiles = removedFiles.filter(id => id !== file.id) : removedFiles.push(file.id ?? '')"><FileText :size="11" />{{ file.name }}<X :size="10" /></button>
            </div>
          </div>
          <button type="button" class="queue-action" aria-label="保存排队消息" :disabled="busy" @click="save"><Check :size="13" /></button>
          <button type="button" class="queue-action" aria-label="取消编辑" :disabled="busy" @click="editing = null"><X :size="13" /></button>
        </template>
        <template v-else>
          <span class="queue-number">{{ index + 1 }}</span>
          <img v-if="previews[previewKey(entry)]" class="queue-thumb" :src="previews[previewKey(entry)]" alt="排队图片预览" />
          <div class="queue-content"><p :title="entry.text">{{ entry.text || '附件消息' }}</p><div class="queue-meta"><span v-if="entry.dispatching">正在发送…</span><span v-else-if="entry.mode === 'steer'">等待补充到当前任务</span><span v-if="entry.images"><Image :size="10" />{{ entry.images }} 张图片</span><span v-for="file in entry.files.filter(f => !f.mime?.startsWith('image/'))" :key="file.id"><FileText :size="10" />{{ file.name }}</span></div></div>
          <div class="queue-actions">
            <button type="button" class="queue-action" :disabled="busy || entry.dispatching || index === 0" aria-label="上移" @click="step(entry.id, -1)"><ArrowUp :size="12" /></button>
            <button type="button" class="queue-action" :disabled="busy || entry.dispatching || index === queue.items.length - 1" aria-label="下移" @click="step(entry.id, 1)"><ArrowDown :size="12" /></button>
            <button type="button" class="queue-action" :disabled="busy || entry.dispatching" aria-label="编辑排队消息" title="暂停队列并编辑" @click="begin(entry)"><Pencil :size="12" /></button>
            <button type="button" class="queue-action" :disabled="busy || entry.dispatching" :aria-label="running ? '停止并发送此条' : '立即发送此条'" :title="running ? '停止当前任务并发送此条，其余队列暂停' : '立即发送此条，其余队列暂停'" @click="act('send_now', entry.id)"><CornerDownLeft :size="13" /></button>
            <button type="button" class="queue-action" :disabled="busy || entry.dispatching" aria-label="删除排队消息" @click="act('remove', entry.id)"><Trash2 :size="12" /></button>
          </div>
        </template>
      </li>
    </ol>
    <button v-if="!expanded && queue.items.length > 2" type="button" class="queue-more" @click="expanded = true">展开其余 {{ queue.items.length - 2 }} 条</button>
  </section>
</template>
<style scoped>
.message-queue{margin:0 2px 8px;padding:7px 10px;background:var(--ui-surface-alt);border:1px solid var(--ui-border);border-radius:12px;font-size:12px;color:var(--ui-text-secondary)}
.queue-head{display:flex;align-items:center;gap:8px;min-height:22px}.queue-summary{display:flex;align-items:center;gap:5px;flex:none;border:0;background:none;color:inherit;font:inherit;padding:0;cursor:pointer}.queue-summary svg{transition:transform .15s}.queue-summary .expanded{transform:rotate(180deg)}.queue-hint{flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:11px;color:var(--ui-text-tertiary)}
ol{list-style:none;margin:3px 0 0;padding:0;max-height:230px;overflow:auto}li{display:flex;align-items:center;gap:7px;padding:6px 0;border-top:1px solid var(--ui-border)}li[draggable=true]{cursor:grab}.queue-thumb{flex:none;width:28px;height:28px;object-fit:cover;border-radius:5px;border:1px solid var(--ui-border)}.queue-number{width:14px;text-align:center;font-size:10px;color:var(--ui-text-tertiary)}.queue-content{flex:1;min-width:0}.queue-content p{margin:0;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;line-height:1.6;color:var(--ui-text-primary)}.queue-meta{display:flex;align-items:center;gap:6px;flex-wrap:wrap;font-size:10px;color:var(--ui-text-tertiary)}.queue-meta span{display:inline-flex;align-items:center;gap:3px;max-width:170px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.queue-actions{display:flex;gap:1px}.queue-action{display:inline-flex;align-items:center;justify-content:center;width:24px;height:24px;border:0;border-radius:6px;background:none;color:var(--ui-text-secondary);cursor:pointer;flex:none}.queue-action:hover:not(:disabled){background:var(--ui-surface-hover);color:var(--ui-text-primary)}.queue-action:disabled{opacity:.3;cursor:default}.queue-more{width:100%;padding:4px;border:0;background:none;color:var(--ui-text-tertiary);font:inherit;font-size:11px;cursor:pointer}.queue-edit{flex:1;min-width:0}.queue-edit textarea{box-sizing:border-box;width:100%;min-height:58px;resize:vertical;border:1px solid var(--ui-border);border-radius:6px;padding:6px;background:var(--ui-surface);color:var(--ui-text-primary);font:inherit;outline-color:var(--ui-btn)}.queue-files{display:flex;flex-wrap:wrap;gap:4px;margin-top:4px}.queue-files button{display:flex;align-items:center;gap:4px;max-width:180px;border:1px solid var(--ui-border);border-radius:5px;padding:3px 5px;background:none;color:inherit;font-size:10px;cursor:pointer}.queue-files .removed{opacity:.45;text-decoration:line-through}
@media(max-width:520px){.queue-hint{display:none}.queue-head{justify-content:space-between}.queue-actions{gap:0}.queue-action{width:22px}.message-queue{padding:6px 8px}}
</style>
