<script setup lang="ts">
import { ref } from 'vue'
import Icon from '@/components/Icon.vue'

const tags = defineModel<string[]>({ required: true })
const draft = ref('')
const inputEl = ref<HTMLInputElement>()

function commit() {
  const parts = draft.value
    .split(/[,，]/)
    .map((s) => s.trim())
    .filter(Boolean)
  if (parts.length) {
    const next = [...tags.value]
    for (const p of parts) {
      if (!next.includes(p)) next.push(p)
    }
    tags.value = next
  }
  draft.value = ''
}

function remove(idx: number) {
  const next = [...tags.value]
  next.splice(idx, 1)
  tags.value = next
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' || e.key === ',' || e.key === '，') {
    e.preventDefault()
    commit()
  } else if (e.key === 'Backspace' && !draft.value && tags.value.length) {
    remove(tags.value.length - 1)
  }
}
</script>

<template>
  <div class="tag-box" @click="inputEl?.focus()">
    <span v-for="(tag, idx) in tags" :key="tag" class="tag-chip">
      <span>{{ tag }}</span>
      <button type="button" :aria-label="`移除 ${tag}`" @click.stop="remove(idx)">
        <Icon name="close" :size="11" />
      </button>
    </span>
    <input
      ref="inputEl"
      v-model="draft"
      :placeholder="tags.length ? '继续输入…' : '输入后按回车添加'"
      @keydown="onKeydown"
      @blur="commit"
    />
  </div>
</template>

<style scoped>
.tag-box {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
  min-height: 36px;
  padding: 4px 8px;
  border-radius: 8px;
  background: var(--ui-control);
  cursor: text;
  transition: background 0.14s, box-shadow 0.14s;
}
.tag-box:focus-within {
  background: var(--ui-surface);
  box-shadow: 0 0 0 2px var(--ui-btn);
}
.tag-chip {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  height: 24px;
  padding: 0 6px 0 9px;
  border-radius: 6px;
  line-height: 1;
  background: var(--ui-surface);
  font-size: 12px;
  font-weight: 500;
  color: var(--ui-text-primary);
}
.tag-chip button {
  width: 15px;
  height: 15px;
  border-radius: 4px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--ui-text-tertiary);
}
.tag-chip button:hover {
  color: var(--ui-danger);
  background: var(--ui-danger-soft);
}
.tag-box input {
  flex: 1;
  min-width: 72px;
  height: 26px;
  border: 0;
  background: transparent;
  outline: none;
  font-size: 12.5px;
  color: var(--ui-text-primary);
}
.tag-box input::placeholder { color: var(--ui-text-tertiary); }
</style>
