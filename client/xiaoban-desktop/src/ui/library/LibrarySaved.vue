<script setup lang="ts">
import { useConfirm } from './confirm.ts';
import { computed, ref } from 'vue';
import { Copy, CornerUpLeft, MessageCircle, Star } from 'lucide-vue-next';
import { formatWhen, SAVED_KINDS, searchText, type LibrarySaved, type LibraryStore, type SavedKind } from './library-store.ts';

const props = defineProps<{ library: LibraryStore; query: string }>();
const emit = defineEmits<{ open: [item: LibrarySaved]; ask: [item: LibrarySaved]; notify: [message: string, type?: 'success' | 'error' | 'info'] }>();
const confirm = useConfirm();

const FILTERS: ('全部' | SavedKind)[] = ['全部', 'answer', 'text', 'file', 'image', 'shot'];
const filter = ref<'全部' | SavedKind>('全部');
const items = computed(() => props.library.data.saved.filter(s => (filter.value === '全部' || s.kind === filter.value) && searchText(props.query, s.title, s.text, s.sessionTitle)));
defineExpose({ showAll: () => { filter.value = '全部'; } });

async function copy(item: LibrarySaved) {
  try { await navigator.clipboard.writeText(item.text); emit('notify', '已复制'); } catch { emit('notify', '复制失败，请重试', 'error'); }
}
function unsave(item: LibrarySaved) {
  confirm({ title: '取消收藏？', content: `「${item.title}」会从收藏里移除，原来的对话不受影响。`, positiveText: '取消收藏', negativeText: '保留', onPositiveClick: () => props.library.remove([item.id]) });
}
</script>

<template>
  <div class="saved" data-testid="library-saved">
    <div class="lib-toolbar">
      <div class="lib-chips" aria-label="按类型筛选">
        <button v-for="f in FILTERS" :key="f" type="button" :class="{ active: filter === f }" :aria-pressed="filter === f" @click="filter = f">{{ f === '全部' ? f : SAVED_KINDS[f] }}</button>
      </div>
      <span class="lib-muted right"><Star :size="13" />可以收藏回答，也可以手动添加内容</span>
    </div>
    <div class="lib-scroll grid-wrap">
      <div v-if="!library.data.saved.length" class="xb-empty">
        <span class="xb-empty-icon"><Star :size="24" /></span>
        <h3>还没有收藏</h3>
        <p>可以点击右上角“添加收藏”手动保存内容，也可以在回答下点“存到资料库”。</p>
      </div>
      <p v-else-if="!items.length" class="lib-muted nothing">没有找到相关收藏</p>
      <div v-else class="grid">
        <article v-for="item in items" :key="item.id" class="lib-card card" :data-testid="`library-saved-${item.id}`">
          <div class="head">
            <span class="lib-pill">{{ SAVED_KINDS[item.kind] }}</span><span class="lib-muted">{{ formatWhen(item.savedMs) }}</span>
            <button type="button" class="lib-icon-btn push" aria-label="复制" @click="copy(item)"><Copy :size="14" /></button>
            <button type="button" class="lib-icon-btn star" aria-label="取消收藏" @click="unsave(item)"><Star :size="15" fill="currentColor" /></button>
          </div>
          <b class="title">{{ item.title }}</b>
          <p class="text">{{ item.text }}</p>
          <div class="foot">
            <span class="lib-muted from">{{ item.kind === 'text' && !item.sessionId ? '手动创建 · 本机' : `来自「${item.sessionTitle || '一段对话'}」· ${item.source === 'assistant' ? '超级小伴' : '主窗口'}` }}</span>
            <button v-if="item.sessionId" type="button" class="lib-link" @click="emit('open', item)"><CornerUpLeft :size="13" />回到对话</button>
            <button type="button" class="lib-link muted" @click="emit('ask', item)"><MessageCircle :size="13" />接着问</button>
          </div>
        </article>
      </div>
    </div>
  </div>
</template>

<style scoped>
.saved{flex:1;min-height:0;display:flex;flex-direction:column;margin-top:14px}
.right{margin-left:auto;display:inline-flex;align-items:center;gap:6px}
.grid-wrap{margin:0 -28px;padding:14px 28px 24px}
.grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(320px,1fr));gap:14px;align-items:start}
.card{padding:14px 16px;display:flex;flex-direction:column;gap:10px;min-width:0}
.head{display:flex;align-items:center;gap:8px}
.push{margin-left:auto}
.star{color:var(--ui-btn)}
.title{font-size:14px;font-weight:600;line-height:1.45}
.text{margin:0;font-size:13px;line-height:1.7;color:var(--ui-text-secondary);white-space:pre-line;display:-webkit-box;-webkit-line-clamp:5;-webkit-box-orient:vertical;overflow:hidden}
.foot{display:flex;align-items:center;gap:12px;padding-top:10px;border-top:1px solid var(--ui-line);font-size:12px}
.from{flex:1;min-width:0;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.nothing{padding:64px 0;text-align:center;font-size:13px}
</style>
