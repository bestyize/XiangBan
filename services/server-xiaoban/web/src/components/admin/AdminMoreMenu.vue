<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import Icon, { type IconName } from '@/components/Icon.vue'

export interface MoreMenuItem {
  label?: string
  icon?: IconName
  desc?: string
  danger?: boolean
  confirmText?: string
  sep?: boolean
  disabled?: boolean
  action?: () => void
}

const props = defineProps<{ items: MoreMenuItem[]; disabled?: boolean }>()

const open = ref(false)
const armedIdx = ref<number | null>(null)
const trigger = ref<HTMLButtonElement>()
const menuStyle = ref<Record<string, string>>({})
let armTimer: ReturnType<typeof setTimeout> | undefined

function toggle() {
  if (props.disabled) return
  if (open.value) {
    close()
    return
  }
  const el = trigger.value
  if (el) {
    const rect = el.getBoundingClientRect()
    const w = 188
    const estH = Math.min(props.items.length * 35 + 14, 280)
    const openUp = rect.bottom + estH + 10 > window.innerHeight && rect.top > estH
    const left = Math.max(10, rect.right - w)
    const top = openUp ? Math.max(10, rect.top - estH - 6) : rect.bottom + 6
    menuStyle.value = { left: `${left}px`, top: `${top}px`, width: `${w}px` }
  }
  armedIdx.value = null
  open.value = true
}

function close() {
  open.value = false
  armedIdx.value = null
  clearTimeout(armTimer)
}

function onPick(item: MoreMenuItem, idx: number) {
  if (item.disabled || !item.action) return
  if (item.danger && armedIdx.value !== idx) {
    armedIdx.value = idx
    clearTimeout(armTimer)
    armTimer = setTimeout(() => { armedIdx.value = null }, 3000)
    return
  }
  const fn = item.action
  close()
  fn()
}

function onDocClick(e: MouseEvent) {
  if (!open.value) return
  const t = e.target as HTMLElement | null
  if (trigger.value?.contains(t)) return
  if (t?.closest('.xb-menu')) return
  close()
}

function onScroll(e: Event) {
  if (!open.value) return
  const t = e.target as HTMLElement | null
  if (t?.closest?.('.xb-menu')) return
  close()
}

function onKey(e: KeyboardEvent) {
  if (open.value && e.key === 'Escape') close()
}

onMounted(() => {
  document.addEventListener('click', onDocClick)
  window.addEventListener('scroll', onScroll, true)
  document.addEventListener('keydown', onKey)
})

onBeforeUnmount(() => {
  clearTimeout(armTimer)
  document.removeEventListener('click', onDocClick)
  window.removeEventListener('scroll', onScroll, true)
  document.removeEventListener('keydown', onKey)
})
</script>

<template>
  <button
    ref="trigger"
    type="button"
    class="xb-icon-btn is-sm"
    :disabled="disabled"
    aria-haspopup="menu"
    :aria-expanded="open"
    aria-label="更多操作"
    title="更多操作"
    @click.stop="toggle"
  >
    <Icon name="more" :size="15" />
  </button>

  <Teleport to="body">
    <div v-if="open" class="xb-menu-portal">
      <div class="xb-menu" role="menu" :style="menuStyle" @click.stop>
        <template v-for="(item, idx) in items" :key="idx">
          <div v-if="item.sep" class="xb-menu-sep" />
          <button
            v-else
            type="button"
            class="xb-menu-item"
            :class="{ 'is-danger': item.danger }"
            :disabled="item.disabled"
            role="menuitem"
            :title="item.desc || item.label"
            :aria-description="item.desc || undefined"
            @click="onPick(item, idx)"
          >
            <span v-if="item.icon" class="m-ico"><Icon :name="item.icon" :size="14" /></span>
            <span>{{ armedIdx === idx ? (item.confirmText || '再点一次确认') : item.label }}</span>
          </button>
        </template>
      </div>
    </div>
  </Teleport>
</template>
