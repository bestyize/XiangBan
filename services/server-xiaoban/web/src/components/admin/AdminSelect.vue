<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import Icon, { type IconName } from '@/components/Icon.vue'

export interface SelectOption {
  value: string
  label: string
  desc?: string
  hint?: string
  icon?: IconName
  dot?: 'ok' | 'warn' | 'danger' | 'info' | 'off'
}

const props = withDefaults(
  defineProps<{
    modelValue: string
    options: SelectOption[]
    prefix?: string
    placeholder?: string
    sm?: boolean
    full?: boolean
    disabled?: boolean
    title?: string
    width?: string
  }>(),
  {
    prefix: '',
    placeholder: '请选择',
    sm: false,
    full: false,
    disabled: false,
    title: '',
    width: '',
  },
)

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()

const open = ref(false)
const trigger = ref<HTMLButtonElement>()
const menuStyle = ref<Record<string, string>>({})

const current = computed(() => props.options.find((o) => o.value === props.modelValue))

function positionMenu() {
  const el = trigger.value
  if (!el) return
  const rect = el.getBoundingClientRect()
  const w = Math.max(Math.min(rect.width, 260), 176)
  const estH = Math.min(props.options.length * 35 + 14, 280)
  const openUp = rect.bottom + estH + 10 > window.innerHeight && rect.top > estH
  const left = Math.max(10, Math.min(rect.left, window.innerWidth - w - 12))
  const top = openUp ? Math.max(10, rect.top - estH - 6) : rect.bottom + 6
  menuStyle.value = { left: `${left}px`, top: `${top}px`, width: `${w}px` }
}

function toggle() {
  if (props.disabled) return
  if (open.value) {
    open.value = false
    return
  }
  positionMenu()
  open.value = true
}

function pick(val: string) {
  open.value = false
  if (val !== props.modelValue) emit('update:modelValue', val)
}

function onDocClick(e: MouseEvent) {
  if (!open.value) return
  const t = e.target as HTMLElement | null
  if (trigger.value?.contains(t)) return
  if (t?.closest('.xb-menu')) return
  open.value = false
}

function onScroll(e: Event) {
  if (!open.value) return
  const t = e.target as HTMLElement | null
  if (t?.closest?.('.xb-menu')) return
  open.value = false
}

function onKey(e: KeyboardEvent) {
  if (open.value && e.key === 'Escape') open.value = false
}

onMounted(() => {
  document.addEventListener('click', onDocClick)
  window.addEventListener('resize', () => (open.value = false))
  window.addEventListener('scroll', onScroll, true)
  document.addEventListener('keydown', onKey)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', onDocClick)
  window.removeEventListener('scroll', onScroll, true)
  document.removeEventListener('keydown', onKey)
})
</script>

<template>
  <button
    ref="trigger"
    type="button"
    class="xb-select-btn"
    :class="{ 'is-sm': sm, 'is-full': full, 'is-open': open }"
    :style="width ? { minWidth: width } : undefined"
    :disabled="disabled"
    aria-haspopup="listbox"
    :aria-expanded="open"
    :title="title || current?.desc || current?.label || placeholder"
    :aria-description="title || current?.desc || undefined"
    @click.stop="toggle"
  >
    <span class="xb-select-val">
      <i v-if="current?.dot" class="xb-dot-status" :class="current.dot" />
      <span v-if="prefix" class="pfx">{{ prefix }}</span>
      <span class="txt">{{ current?.label ?? placeholder }}</span>
    </span>
    <span class="xb-select-chev"><Icon name="chevron-down" :size="14" /></span>
  </button>

  <Teleport to="body">
    <div v-if="open" class="xb-menu-portal">
      <div class="xb-menu" role="listbox" :style="menuStyle" @click.stop>
        <button
          v-for="opt in options"
          :key="opt.value"
          type="button"
          class="xb-menu-item"
          :class="{ 'is-active': opt.value === modelValue }"
          role="option"
          :aria-selected="opt.value === modelValue"
          :title="opt.desc || opt.label"
          :aria-description="opt.desc || undefined"
          @click="pick(opt.value)"
        >
          <span v-if="opt.icon" class="m-ico"><Icon :name="opt.icon" :size="14" /></span>
          <i v-else-if="opt.dot" class="xb-dot-status" :class="opt.dot" />
          <span class="opt-txt">{{ opt.label }}</span>
          <span v-if="opt.value === modelValue" class="m-chk"><Icon name="check" :size="14" /></span>
          <span v-else-if="opt.hint" class="m-hint">{{ opt.hint }}</span>
        </button>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.xb-select-btn {
  height: 34px;
  padding: 0 10px 0 11px;
  border-radius: 8px;
  background: var(--ui-control);
  color: var(--ui-text-primary);
  display: inline-flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  font-size: 13px;
  line-height: 1;
  white-space: nowrap;
  transition: background 0.14s, box-shadow 0.14s;
}
.xb-select-btn.is-sm {
  height: 30px;
  font-size: 12.5px;
  padding: 0 9px 0 10px;
}
.xb-select-btn.is-full { width: 100%; }
.xb-select-btn:hover:not(:disabled) { background: var(--ui-control-hover); }
.xb-select-btn.is-open {
  background: var(--ui-surface);
  box-shadow: 0 0 0 2px var(--ui-btn);
}
.xb-select-btn:disabled { opacity: 0.55; cursor: not-allowed; }
.xb-select-val {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
  overflow: hidden;
}
.xb-select-val .pfx { color: var(--ui-text-tertiary); }
.xb-select-val .txt {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.xb-select-chev {
  color: var(--ui-text-tertiary);
  transition: transform 0.16s var(--ui-ease);
}
.xb-select-btn.is-open .xb-select-chev {
  transform: rotate(180deg);
  color: var(--ui-text-primary);
}
.opt-txt {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
