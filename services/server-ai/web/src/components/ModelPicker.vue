<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue';
import type { Capability, ModelOption } from '../types';
import Icon from './Icon.vue';

const props = withDefaults(
  defineProps<{
    modelValue: string;
    models: ModelOption[];
    recommend?: Capability | 'chat';
    placeholder?: string;
  }>(),
  { placeholder: '选择上游模型…' },
);

const emit = defineEmits<{ 'update:modelValue': [value: string] }>();

const open = ref(false);
const query = ref('');
const root = ref<HTMLElement | null>(null);
const searchEl = ref<HTMLInputElement | null>(null);

const picked = computed(() => props.models.find((m) => m.model_name === props.modelValue) ?? null);

function isRecommended(name: string): boolean {
  if (!props.recommend) return false;
  const n = name.toLowerCase();
  if (props.recommend === 'speech') return /tts|speech|audio-preview|voice|cosyvoice|minimax-speech/.test(n);
  if (props.recommend === 'image') return /image|dall|flux|imagen|banana|ideogram|midjourney/.test(n);
  if (props.recommend === 'transcribe') return /whisper|transcribe|asr|sensevoice|paraformer/.test(n);
  if (props.recommend === 'realtime') return /realtime/.test(n);
  return !/tts|whisper|transcribe|realtime|dall|flux|embedding|rerank|moderation/.test(n);
}

const filtered = computed(() => {
  const q = query.value.trim().toLowerCase();
  const base = props.models.filter((m) => !q || m.model_name.toLowerCase().includes(q));
  return [...base].sort((a, b) => Number(isRecommended(b.model_name)) - Number(isRecommended(a.model_name)));
});

async function toggle() {
  open.value = !open.value;
  if (open.value) {
    await nextTick();
    searchEl.value?.focus();
  }
}

function pick(name: string) {
  emit('update:modelValue', name);
  open.value = false;
  query.value = '';
}

function onDocClick(e: MouseEvent) {
  if (open.value && root.value && !root.value.contains(e.target as Node)) {
    open.value = false;
  }
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && open.value) {
    e.stopPropagation();
    open.value = false;
  }
}

onMounted(() => document.addEventListener('mousedown', onDocClick));
onBeforeUnmount(() => document.removeEventListener('mousedown', onDocClick));
</script>

<template>
  <div ref="root" class="mpicker" :class="{ open }" @keydown="onKeydown">
    <button type="button" class="mpicker-trigger" :aria-expanded="open" @click="toggle">
      <div class="mpicker-main">
        <div class="mpicker-name" :class="{ placeholder: !modelValue }">{{ modelValue || placeholder }}</div>
        <div v-if="modelValue" class="mpicker-meta">
          <template v-if="picked">
            <span class="tag num">输入 {{ picked.model_ratio }}</span>
            <span class="tag num">输出 ×{{ picked.completion_ratio }}</span>
            <span v-if="picked.ratio_warning" class="tag warn"><Icon name="alert" :size="11" />默认倍率预警</span>
          </template>
          <span v-else-if="models.length" class="tag danger"><Icon name="circle-alert" :size="11" />网关中未找到此模型</span>
        </div>
      </div>
      <span class="chev"><Icon name="chevron-down" :size="15" /></span>
    </button>

    <div v-if="open" class="mpicker-menu">
      <div class="mpicker-search">
        <Icon name="search" :size="14" />
        <input
          ref="searchEl"
          v-model="query"
          type="text"
          placeholder="输入关键词搜索模型（如 claude / deepseek / image）…"
          autocomplete="off"
          spellcheck="false"
        />
        <button v-if="query" type="button" class="btn icon sm ghost" aria-label="清空搜索" @click="query = ''">
          <Icon name="x" :size="12" />
        </button>
      </div>
      <div class="mpicker-list" role="listbox">
        <button
          v-for="opt in filtered"
          :key="opt.model_name"
          type="button"
          role="option"
          class="mpicker-opt"
          :class="{ active: opt.model_name === modelValue }"
          :aria-selected="opt.model_name === modelValue"
          @click="pick(opt.model_name)"
        >
          <span class="mn">{{ opt.model_name }}</span>
          <span v-if="isRecommended(opt.model_name)" class="tag coral">推荐</span>
          <span class="tag num">{{ opt.model_ratio }} / ×{{ opt.completion_ratio }}</span>
          <span v-if="opt.ratio_warning" class="tag warn">默认倍率</span>
          <span v-if="opt.model_name === modelValue" class="chk"><Icon name="check" :size="14" :stroke="2.5" /></span>
        </button>
        <div v-if="filtered.length === 0" class="empty-msg">没有匹配的模型</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.mpicker { position: relative; }
.mpicker-trigger {
  width: 100%;
  padding: 10px 12px;
  border-radius: 11px;
  border: 1px solid var(--line-strong);
  background: var(--surface);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  text-align: left;
  box-shadow: var(--shadow-xs);
  transition: border-color 0.14s, box-shadow 0.14s;
}
.mpicker-trigger:hover { border-color: var(--line-focus); }
.mpicker.open .mpicker-trigger { border-color: var(--ink); box-shadow: var(--ring); }
.mpicker-main { display: flex; flex-direction: column; gap: 4px; min-width: 0; flex: 1; }
.mpicker-name { font-family: var(--mono); font-size: 13px; font-weight: 650; color: var(--ink); overflow-wrap: anywhere; }
.mpicker-name.placeholder { font-family: var(--font); font-weight: 500; color: var(--ink-3); }
.mpicker-meta { display: flex; align-items: center; gap: 5px; flex-wrap: wrap; }
.chev { color: var(--ink-3); transition: transform 0.16s var(--ease); flex: none; }
.mpicker.open .chev { transform: rotate(180deg); color: var(--ink); }

.mpicker-menu {
  margin-top: 6px;
  border-radius: 13px;
  border: 1px solid var(--line-strong);
  background: var(--surface);
  box-shadow: var(--shadow-md);
  overflow: hidden;
  animation: dropIn 0.14s var(--ease);
}
.mpicker-search {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 11px;
  border-bottom: 1px solid var(--line);
  background: var(--surface-muted);
  color: var(--ink-3);
}
.mpicker-search input {
  flex: 1;
  min-width: 0;
  border: 0;
  background: none;
  outline: none;
  font-size: 12.5px;
  font-family: var(--mono);
  color: var(--ink);
}
.mpicker-search input::placeholder { font-family: var(--font); color: var(--ink-4); }
.mpicker-list {
  max-height: 210px;
  overflow-y: auto;
  padding: 5px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.mpicker-opt {
  width: 100%;
  padding: 8px 10px;
  border-radius: 9px;
  text-align: left;
  display: flex;
  align-items: center;
  gap: 7px;
  flex-wrap: wrap;
  transition: background 0.1s;
}
.mpicker-opt:hover { background: var(--canvas-soft); }
.mpicker-opt.active { background: var(--coral-soft); }
.mn { font-family: var(--mono); font-size: 12.5px; font-weight: 600; color: var(--ink); flex: 1; min-width: 130px; overflow-wrap: anywhere; }
.chk { color: var(--coral); flex: none; }
.empty-msg { padding: 14px; text-align: center; font-size: 12.5px; color: var(--ink-3); }
@keyframes dropIn {
  from { opacity: 0; transform: translateY(-4px); }
}
</style>
