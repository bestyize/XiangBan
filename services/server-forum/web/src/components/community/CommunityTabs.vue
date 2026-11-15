<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, useId, useTemplateRef, watch } from 'vue'
import { RouterLink } from 'vue-router'

interface TabItem { key: string; label: string; to?: string }

const props = defineProps<{
  items: readonly TabItem[]
  active: string
  label: string
  primary?: boolean
}>()
const emit = defineEmits<{ change: [key: string] }>()
const row = useTemplateRef<HTMLDivElement>('row')
const overflow = useTemplateRef<HTMLDivElement>('overflow')
const more = useTemplateRef<HTMLButtonElement>('more')
const menuId = useId()
const menuOpen = ref(false)
const visiblePrefix = ref(Infinity)
const overflowItems = computed(() => props.items.slice(visiblePrefix.value).filter(item => item.key !== props.active))
let resize: ResizeObserver | undefined
let focusWhenSelected: string | undefined

function focusActive() {
  row.value?.querySelector<HTMLElement>(':scope > .tab-item.on')?.focus({ preventScroll: true })
}

function closeMenu(restoreFocus = false) {
  menuOpen.value = false
  if (restoreFocus) more.value?.focus({ preventScroll: true })
}

function measure() {
  if (!row.value || !more.value) return
  const tabs = Array.from(row.value.querySelectorAll<HTMLElement>(':scope > .tab-item'))
  const widths = tabs.map(tab => tab.getBoundingClientRect().width)
  const gap = parseFloat(getComputedStyle(row.value).columnGap) || 0
  const available = row.value.getBoundingClientRect().width
  const allWidth = widths.reduce((total, width) => total + width, gap * Math.max(0, widths.length - 1))
  if (allWidth <= available) {
    const restoreFocus = overflow.value?.contains(document.activeElement)
    visiblePrefix.value = Infinity
    closeMenu()
    if (restoreFocus) void nextTick(focusActive)
    return
  }

  let space = available - more.value.getBoundingClientRect().width
  let count = 0
  while (count < widths.length && widths[count]! + gap <= space) {
    space -= widths[count]! + gap
    count++
  }

  // Keep the first tab and reserve room for the current tab if it would be folded.
  const activeIndex = props.items.findIndex(item => item.key === props.active)
  while (activeIndex >= count && count > 1 && widths[activeIndex]! + gap > space) {
    count--
    space += widths[count]! + gap
  }
  visiblePrefix.value = Math.max(1, count)
}

onMounted(() => {
  measure()
  if (row.value && typeof ResizeObserver !== 'undefined') {
    resize = new ResizeObserver(measure)
    resize.observe(row.value)
  } else {
    window.addEventListener('resize', measure)
  }
  void document.fonts?.ready.then(measure)
})
onUnmounted(() => {
  resize?.disconnect()
  window.removeEventListener('resize', measure)
})

watch([() => props.items, () => props.active], async ([, active]) => {
  measure()
  if (focusWhenSelected !== active) return
  focusWhenSelected = undefined
  await nextTick()
  focusActive()
}, { flush: 'post' })

watch(menuOpen, (open, _, onCleanup) => {
  if (!open) return
  const dismiss = (event: PointerEvent) => {
    if (event.target instanceof Node && !overflow.value?.contains(event.target)) closeMenu()
  }
  document.addEventListener('pointerdown', dismiss)
  onCleanup(() => document.removeEventListener('pointerdown', dismiss))
})

function choose(item: TabItem, event: MouseEvent, fromMenu = false) {
  if (item.to && (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey)) return
  if (fromMenu) focusWhenSelected = item.key
  closeMenu(fromMenu)
  if (!item.to) emit('change', item.key)
}

function onEscape(event: KeyboardEvent) {
  if (!menuOpen.value) return
  event.stopPropagation()
  closeMenu(true)
}
</script>

<template>
  <nav class="community-tabs" :class="{ 'primary-tabs': primary }" :aria-label="label" @keydown.esc="onEscape">
    <div ref="row" class="community-tab-items" :class="{ 'menu-open': menuOpen }">
      <component :is="item.to ? RouterLink : 'button'" v-for="(item, index) in items" :key="item.key"
        :to="item.to" :type="item.to ? undefined : 'button'" class="tab-item"
        :class="{ on: active === item.key, 'tab-measure': index >= visiblePrefix && active !== item.key }"
        :aria-current="active === item.key ? 'page' : undefined" :title="item.label" @click="choose(item, $event)">
        <span class="tab-label">{{ item.label }}</span>
      </component>
      <div ref="overflow" class="tab-overflow" :class="{ 'tab-measure': !overflowItems.length }">
        <button ref="more" class="tab-more" type="button" :aria-expanded="menuOpen" :aria-controls="menuId"
          @click="menuOpen = !menuOpen">更多<span class="tab-chevron" aria-hidden="true" /></button>
        <div v-show="menuOpen" :id="menuId" class="tab-menu" role="group" :aria-label="`更多${label}`">
          <component :is="item.to ? RouterLink : 'button'" v-for="item in overflowItems" :key="item.key"
            :to="item.to" :type="item.to ? undefined : 'button'" @click="choose(item, $event, true)">{{ item.label }}</component>
        </div>
      </div>
    </div>
    <div v-if="$slots.actions" class="community-tab-actions"><slot name="actions" /></div>
  </nav>
</template>

<style scoped>
.community-tabs {
  display: flex;
  gap: 6px;
  min-width: 0;
  margin-bottom: 20px;
}
.community-tab-items {
  position: relative;
  display: flex;
  flex: 1 1 auto;
  align-items: center;
  justify-content: var(--tab-justify, flex-start);
  gap: inherit;
  min-width: 0;
  overflow-x: auto;
  scrollbar-width: none;
}
.community-tab-items.menu-open { overflow: visible; }
.community-tab-actions { display: flex; flex: none; align-items: center; gap: 12px; }
.tab-item, .tab-more {
  position: relative;
  display: inline-flex;
  flex: none;
  align-items: center;
  min-height: 36px;
  padding: 6px 14px;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: var(--ink-2);
  font-size: 14px;
  white-space: nowrap;
  transition: color .15s, background-color .15s;
}
.tab-item { max-width: min(320px, calc(50% - 42px)); }
.tab-label { min-width: 0; overflow: hidden; text-overflow: ellipsis; }
.tab-item:hover:not(.on) { background: var(--surface-2); }
.tab-item.on { color: var(--brand); font-weight: 650; }
.tab-item:focus-visible, .tab-more:focus-visible { outline-offset: -2px; }
.tab-overflow { position: relative; flex: none; }
/* Folded items remain measurable outside the flex flow. */
.tab-item.tab-measure, .tab-overflow.tab-measure {
  position: absolute;
  inset: 0 auto auto 0;
  width: max-content;
  visibility: hidden;
  pointer-events: none;
}
.tab-more { gap: 7px; }
.tab-more:is(:hover, [aria-expanded="true"]) { color: var(--brand); }
.tab-chevron {
  width: 7px;
  height: 7px;
  border: solid currentColor;
  border-width: 0 1.5px 1.5px 0;
  transform: translateY(-2px) rotate(45deg);
}
.tab-menu {
  position: absolute;
  inset-inline-end: 0;
  top: calc(100% + 6px);
  z-index: 25;
  width: min(224px, 55vw);
  max-height: min(280px, 55vh);
  padding: 5px;
  overflow-y: auto;
  overscroll-behavior: contain;
  border: 1px solid var(--line);
  border-radius: 10px;
  background: var(--surface);
  box-shadow: 0 8px 24px #0002;
}
.tab-menu :is(button, a) {
  display: block;
  width: 100%;
  min-height: 36px;
  padding: 7px 10px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--ink);
  font-size: 14px;
  text-align: start;
  overflow-wrap: anywhere;
}
.tab-menu :is(button, a):hover { background: var(--surface-2); }
.primary-tabs {
  --tab-justify: safe center;
  gap: clamp(9px, 2.7vw, 32px);
  margin-bottom: 0;
}
.primary-tabs .tab-item, .primary-tabs .tab-more {
  min-height: var(--header-h);
  padding: 0;
  font-size: clamp(14px, 1.5vw, 15px);
}
.primary-tabs .tab-item:hover:not(.on) { background: transparent; color: var(--brand); }
.primary-tabs .tab-item.on::after {
  position: absolute;
  bottom: 4px;
  left: 50%;
  transform: translateX(-50%);
  width: 22px;
  height: 3px;
  border-radius: 2px;
  background: var(--brand);
  content: '';
}
</style>
