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
        <span class="mpicker-name" :class="{ placeholder: !modelValue }">{{ modelValue || placeholder }}</span>
        <template v-if="modelValue && picked">
          <span class="tag outlined num">{{ picked.model_ratio }} / ×{{ picked.completion_ratio }}</span>
          <span v-if="picked.ratio_warning" class="tag outlined"><i class="dot warn"></i>默认倍率</span>
        </template>
        <span v-else-if="modelValue && models.length" class="tag outlined"><i class="dot danger"></i>未找到</span>
      </div>
      <span class="chev"><Icon name="chevron-down" :size="14" /></span>
    </button>

    <div v-if="open" class="mpicker-menu">
      <div class="mpicker-search">
        <Icon name="search" :size="14" />
        <input
          ref="searchEl"
          v-model="query"
          type="text"
          placeholder="搜索模型名…"
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
          :title="`${opt.model_name} · 输入 ${opt.model_ratio} / 输出 ×${opt.completion_ratio}`"
          @click="pick(opt.model_name)"
        >
          <span class="mn">{{ opt.model_name }}</span>
          <span v-if="isRecommended(opt.model_name)" class="tag"><i class="dot brand"></i>推荐</span>
          <span class="faint num ratio">{{ opt.model_ratio }}/×{{ opt.completion_ratio }}</span>
          <i v-if="opt.ratio_warning" class="dot warn" title="默认倍率未定价"></i>
          <span v-if="opt.model_name === modelValue" class="chk"><Icon name="check" :size="14" :stroke="2.3" /></span>
        </button>
        <div v-if="filtered.length === 0" class="empty-msg">无匹配模型</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.mpicker { position: relative; }
.mpicker-trigger {
  width: 100%;
  min-height: 40px;
  padding: 7px 12px;
  border-radius: var(--r-inner);
  border: 1px solid transparent;
  background: var(--subtle);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  text-align: left;
  transition: background 140ms, border-color 140ms, box-shadow 140ms;
}
.mpicker-trigger:hover { background: var(--hover); }
.mpicker.open .mpicker-trigger {
  background: var(--surface);
  border-color: var(--brand);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--brand) 16%, transparent);
}
.mpicker-main { display: flex; align-items: center; gap: 7px; min-width: 0; flex: 1; flex-wrap: wrap; }
.mpicker-name { font-family: var(--mono); font-size: 13px; font-weight: 650; color: var(--ink-1); overflow-wrap: anywhere; }
.mpicker-name.placeholder { font-family: var(--font); font-weight: 500; color: var(--ink-3); }
.chev { color: var(--ink-3); transition: transform 160ms var(--ease); flex: none; }
.mpicker.open .chev { transform: rotate(180deg); color: var(--brand-ink); }

.mpicker-menu {
  margin-top: 6px;
  border-radius: var(--r-md);
  border: 1px solid var(--line);
  background: var(--surface);
  box-shadow: var(--shadow-menu);
  overflow: hidden;
  animation: dropIn 140ms var(--ease);
}
.mpicker-search {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--line);
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
  color: var(--ink-1);
}
.mpicker-search input::placeholder { font-family: var(--font); color: var(--ink-4); }
.mpicker-list {
  max-height: 204px;
  overflow-y: auto;
  padding: 4px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.mpicker-opt {
  width: 100%;
  height: 34px;
  padding: 0 10px;
  border-radius: var(--r-inner);
  text-align: left;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: background 100ms;
}
.mpicker-opt:hover { background: var(--subtle); }
.mpicker-opt.active { background: var(--brand-soft); }
.mn { font-family: var(--mono); font-size: 12.5px; font-weight: 550; color: var(--ink-1); flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.mpicker-opt.active .mn { color: var(--brand-ink); font-weight: 600; }
.ratio { font-size: 11.5px; flex: none; }
.chk { color: var(--brand-ink); flex: none; }
.empty-msg { padding: 14px; text-align: center; font-size: 12.5px; color: var(--ink-3); }
@keyframes dropIn {
  from { opacity: 0; transform: translateY(-4px); }
}
</style>
