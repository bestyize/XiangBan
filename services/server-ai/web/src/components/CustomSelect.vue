<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import Icon from './Icon.vue';

export interface SelectOption {
  value: string;
  label: string;
  desc?: string;
}

const props = withDefaults(
  defineProps<{
    modelValue: string;
    options: SelectOption[];
    placeholder?: string;
    size?: 'md' | 'sm';
    outlined?: boolean;
    ariaLabel?: string;
  }>(),
  { placeholder: '请选择', size: 'md', outlined: false },
);

const emit = defineEmits<{ 'update:modelValue': [value: string] }>();

const open = ref(false);
const root = ref<HTMLElement | null>(null);
const current = computed(() => props.options.find((o) => o.value === props.modelValue));

function pick(val: string) {
  emit('update:modelValue', val);
  open.value = false;
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
  <div ref="root" class="cselect" :class="{ open }" @keydown="onKeydown">
    <button
      type="button"
      class="cselect-btn"
      :class="[size, { outlined }]"
      :aria-label="ariaLabel"
      :aria-expanded="open"
      aria-haspopup="listbox"
      @click="open = !open"
    >
      <span class="truncate">{{ current?.label ?? placeholder }}</span>
      <span class="cselect-chev"><Icon name="chevron-down" :size="14" /></span>
    </button>

    <div v-if="open" class="cselect-menu" role="listbox">
      <button
        v-for="opt in options"
        :key="opt.value"
        type="button"
        role="option"
        class="cselect-opt"
        :class="{ active: opt.value === modelValue }"
        :aria-selected="opt.value === modelValue"
        :title="opt.desc || opt.label"
        :aria-description="opt.desc"
        @click="pick(opt.value)"
      >
        <span class="truncate">{{ opt.label }}</span>
        <span v-if="opt.value === modelValue" class="chk"><Icon name="check" :size="13" :stroke="2.4" /></span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.cselect {
  position: relative;
  display: inline-block;
  width: 100%;
}
.cselect-btn {
  width: 100%;
  height: 36px;
  padding: 0 12px;
  border-radius: var(--r-inner);
  border: 1px solid transparent;
  background: var(--subtle);
  color: var(--ink-1);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  font-size: 13px;
  text-align: left;
  transition: background 140ms, border-color 140ms, box-shadow 140ms;
}
.cselect-btn.sm {
  height: 32px;
  font-size: 12.5px;
}
.cselect-btn.outlined {
  background: var(--surface);
  border-color: var(--line);
}
.cselect-btn:hover {
  background: var(--hover);
}
.cselect-btn.outlined:hover {
  background: var(--subtle);
}
.cselect.open .cselect-btn {
  background: var(--surface);
  border-color: var(--brand);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--brand) 16%, transparent);
}
.cselect-chev {
  color: var(--ink-3);
  transition: transform 160ms var(--ease);
  flex: none;
}
.cselect.open .cselect-chev {
  transform: rotate(180deg);
  color: var(--brand-ink);
}
.cselect-menu {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  right: 0;
  min-width: 176px;
  z-index: 70;
  padding: 4px;
  border-radius: var(--r-md);
  background: var(--surface);
  border: 1px solid var(--line);
  box-shadow: var(--shadow-menu);
  display: flex;
  flex-direction: column;
  gap: 2px;
  max-height: 240px;
  overflow-y: auto;
  animation: dropIn 140ms var(--ease);
}
.cselect-opt {
  width: 100%;
  height: 32px;
  padding: 0 10px;
  border-radius: var(--r-inner);
  text-align: left;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  font-size: 13px;
  color: var(--ink-2);
  transition: background 100ms, color 100ms;
}
.cselect-opt:hover {
  background: var(--subtle);
  color: var(--ink-1);
}
.cselect-opt.active {
  background: var(--brand-soft);
  color: var(--brand-ink);
  font-weight: 600;
}
.chk {
  color: var(--brand-ink);
  flex: none;
}
@keyframes dropIn {
  from { opacity: 0; transform: translateY(-4px); }
}
</style>
