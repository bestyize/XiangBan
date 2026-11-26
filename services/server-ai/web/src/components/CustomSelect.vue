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
    ariaLabel?: string;
  }>(),
  { placeholder: '请选择', size: 'md' },
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
      :class="size"
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
        @click="pick(opt.value)"
      >
        <span class="opt-txt">
          <span class="truncate">{{ opt.label }}</span>
          <small v-if="opt.desc" class="faint">{{ opt.desc }}</small>
        </span>
        <span class="chk"><Icon name="check" :size="13" :stroke="2.4" /></span>
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
  padding: 0 11px;
  border-radius: var(--r-sm);
  border: 1px solid var(--line-strong);
  background: var(--surface);
  color: var(--ink);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  font-size: 13px;
  text-align: left;
  box-shadow: var(--shadow-xs);
  transition: border-color 0.14s, box-shadow 0.14s, background 0.14s;
}
.cselect-btn.sm {
  height: 32px;
  font-size: 12.5px;
}
.cselect-btn:hover {
  border-color: var(--line-focus);
  background: var(--surface-hover);
}
.cselect.open .cselect-btn {
  border-color: var(--ink);
  box-shadow: var(--ring);
}
.cselect-chev {
  color: var(--ink-3);
  transition: transform 0.16s var(--ease);
  flex: none;
}
.cselect.open .cselect-chev {
  transform: rotate(180deg);
  color: var(--ink);
}
.cselect-menu {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  right: 0;
  min-width: 180px;
  z-index: 70;
  padding: 5px;
  border-radius: 12px;
  background: var(--surface);
  border: 1px solid var(--line-strong);
  box-shadow: var(--shadow-md);
  display: flex;
  flex-direction: column;
  gap: 2px;
  max-height: 240px;
  overflow-y: auto;
  animation: dropIn 0.14s var(--ease);
}
.cselect-opt {
  width: 100%;
  padding: 7px 10px;
  border-radius: 8px;
  text-align: left;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  font-size: 13px;
  color: var(--ink-2);
  transition: background 0.1s, color 0.1s;
}
.cselect-opt:hover {
  background: var(--canvas-soft);
  color: var(--ink);
}
.cselect-opt.active {
  background: var(--surface-muted);
  color: var(--ink);
  font-weight: 600;
}
.opt-txt {
  display: flex;
  flex-direction: column;
  min-width: 0;
}
.opt-txt small {
  font-size: 11px;
  font-weight: 450;
}
.chk {
  color: var(--coral);
  opacity: 0;
  flex: none;
}
.cselect-opt.active .chk {
  opacity: 1;
}
@keyframes dropIn {
  from { opacity: 0; transform: translateY(-4px); }
}
</style>
