<script setup lang="ts">
import { ref } from 'vue';
import Icon from './Icon.vue';

const model = defineModel<string[]>({ required: true });

withDefaults(
  defineProps<{
    presets?: string[];
    placeholder?: string;
    emptyText?: string;
  }>(),
  { presets: () => [], placeholder: '输入内容后回车或点添加…', emptyText: '尚未添加选项，请从下方预设点选或输入添加' },
);

const draft = ref('');

function addDraft() {
  const parts = draft.value
    .split(/[\n,，\s]+/)
    .map((s) => s.trim())
    .filter(Boolean);
  if (!parts.length) return;
  const next = [...model.value];
  for (const p of parts) {
    if (!next.includes(p)) next.push(p);
  }
  model.value = next;
  draft.value = '';
}

function remove(item: string) {
  model.value = model.value.filter((x) => x !== item);
}

function setDefault(item: string) {
  model.value = [item, ...model.value.filter((x) => x !== item)];
}

function togglePreset(item: string) {
  if (model.value.includes(item)) remove(item);
  else model.value = [...model.value, item];
}
</script>

<template>
  <div class="chip-editor">
    <div class="chip-list">
      <span v-for="(item, idx) in model" :key="item" class="ed-chip" :class="{ def: idx === 0 }">
        <span>{{ item }}</span>
        <span v-if="idx === 0" class="def-tag">默认</span>
        <button v-else type="button" class="ed-chip-btn" title="设为默认" @click="setDefault(item)">
          <Icon name="sparkles" :size="11" />
        </button>
        <button type="button" class="ed-chip-btn" title="移除" @click="remove(item)">
          <Icon name="x" :size="11" />
        </button>
      </span>
      <span v-if="model.length === 0" class="hint">{{ emptyText }}</span>
    </div>

    <div class="chip-add-row">
      <input
        v-model="draft"
        type="text"
        class="input mono"
        :placeholder="placeholder"
        autocomplete="off"
        spellcheck="false"
        @keydown.enter.prevent="addDraft"
      />
      <button type="button" class="btn sm" :disabled="!draft.trim()" @click="addDraft">
        <Icon name="plus" :size="12" />添加
      </button>
    </div>

    <div v-if="presets.length" class="chip-presets">
      <span>常用预设：</span>
      <button
        v-for="p in presets"
        :key="p"
        type="button"
        class="preset-pill"
        :class="{ active: model.includes(p) }"
        @click="togglePreset(p)"
      >{{ p }}</button>
    </div>
  </div>
</template>

<style scoped>
.chip-editor {
  padding: 10px 12px;
  border-radius: 11px;
  border: 1px solid var(--line-strong);
  background: var(--surface);
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.chip-list { display: flex; flex-wrap: wrap; gap: 6px; align-items: center; }
.ed-chip {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  height: 27px;
  padding: 0 6px 0 9px;
  border-radius: 7px;
  background: var(--canvas-soft);
  border: 1px solid var(--line);
  font-family: var(--mono);
  font-size: 12px;
  font-weight: 550;
  color: var(--ink);
}
.ed-chip.def {
  background: var(--btn-primary-bg);
  border-color: var(--btn-primary-bg);
  color: var(--btn-primary-ink);
}
.def-tag { font-family: var(--font); font-size: 10px; opacity: 0.8; font-weight: 600; }
.ed-chip-btn {
  width: 18px;
  height: 18px;
  border-radius: 4px;
  display: grid;
  place-items: center;
  opacity: 0.65;
  transition: opacity 0.12s, background 0.12s;
}
.ed-chip-btn:hover { opacity: 1; background: rgba(128, 128, 128, 0.22); }
.chip-add-row { display: flex; gap: 6px; }
.chip-add-row .input { height: 30px; font-size: 12px; }
.chip-presets {
  display: flex;
  align-items: center;
  gap: 5px;
  flex-wrap: wrap;
  font-size: 11.5px;
  color: var(--ink-3);
}
.preset-pill {
  height: 23px;
  padding: 0 8px;
  border-radius: 6px;
  border: 1px dashed var(--line-strong);
  background: var(--surface-muted);
  font-family: var(--mono);
  font-size: 11px;
  color: var(--ink-2);
  transition: 0.12s;
}
.preset-pill:hover { border-color: var(--ink); color: var(--ink); }
.preset-pill.active {
  border-style: solid;
  border-color: var(--ink);
  background: var(--btn-primary-bg);
  color: var(--btn-primary-ink);
}
</style>
