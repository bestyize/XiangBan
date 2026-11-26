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
  { presets: () => [], placeholder: '输入内容后回车添加…', emptyText: '尚未添加选项' },
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
      <span
        v-for="(item, idx) in model"
        :key="item"
        class="ed-chip"
        :class="{ def: idx === 0 }"
        :title="idx === 0 ? '默认选项' : '点击星标设为默认'"
      >
        <span>{{ item }}</span>
        <button v-if="idx > 0" type="button" class="ed-chip-btn" title="设为默认" @click="setDefault(item)">
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
      <button type="button" class="btn sm outline" :disabled="!draft.trim()" @click="addDraft">
        <Icon name="plus" :size="12" />添加
      </button>
    </div>

    <div v-if="presets.length" class="chip-presets">
      <span>预设</span>
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
  border-radius: var(--r-md);
  background: var(--subtle);
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.chip-list { display: flex; flex-wrap: wrap; gap: 6px; align-items: center; }
.ed-chip {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  height: 26px;
  padding: 0 7px 0 9px;
  border-radius: var(--r-pill);
  background: var(--surface);
  border: 1px solid var(--line);
  font-family: var(--mono);
  font-size: 11.5px;
  font-weight: 550;
  color: var(--ink-1);
}
.ed-chip.def {
  background: var(--brand);
  border-color: var(--brand);
  color: var(--brand-on);
}
.ed-chip-btn {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  opacity: 0.72;
  transition: opacity 120ms, background 120ms;
}
.ed-chip-btn:hover { opacity: 1; background: rgba(128, 128, 128, 0.22); }
.chip-add-row { display: flex; gap: 6px; }
.chip-add-row .input { height: 30px; font-size: 12px; background: var(--surface); }
.chip-presets {
  display: flex;
  align-items: center;
  gap: 5px;
  flex-wrap: wrap;
  font-size: 11.5px;
  color: var(--ink-3);
}
.preset-pill {
  height: 22px;
  padding: 0 8px;
  border-radius: var(--r-pill);
  background: var(--surface);
  border: 1px solid var(--line);
  font-family: var(--mono);
  font-size: 11px;
  color: var(--ink-2);
  transition: 120ms;
}
.preset-pill:hover { color: var(--ink-1); border-color: var(--line-strong); }
.preset-pill.active {
  background: var(--brand);
  border-color: var(--brand);
  color: var(--brand-on);
}
</style>
