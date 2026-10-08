<script setup lang="ts">
import { computed, nextTick, onDeactivated, ref, watch } from 'vue'
import { onClickOutside, onKeyStroke } from '@vueuse/core'
import Icon from './Icon.vue'
import { useCatalog } from '@/stores/catalog'
import type { Orientation } from '@/types'
import { providerName } from '@/utils'

const props = defineProps<{
  /** Show the category rail (home and category pages). */
  rail?: boolean
  activeCategory?: string
  media: 'image' | 'video'
}>()

const orientation = defineModel<Orientation>('orientation', { default: 'all' })
const provider = defineModel<string>('provider', { default: 'all' })

const catalog = useCatalog()
const open = ref(false)
const railEl = ref<HTMLElement>()
const filterEl = ref<HTMLElement>()
const buttonEl = ref<HTMLButtonElement>()

const ORIENTATIONS: { value: Orientation; label: string }[] = [
  { value: 'all', label: '全部画幅' },
  { value: 'landscape', label: '横屏' },
  { value: 'portrait', label: '竖屏' },
  { value: 'squarish', label: '方形' },
]

const sources = computed(() =>
  catalog.providers.filter((p) => p.is_configured && p.supported_media_types.includes(props.media)),
)

const filtered = computed(() => orientation.value !== 'all' || provider.value !== 'all')

function reset() {
  orientation.value = 'all'
  provider.value = 'all'
}

// The choices live in a popover under the button: it closes on an outside click, on Esc, and when
// the page is left, so it never stays open on top of the wallpapers.
onClickOutside(filterEl, () => (open.value = false))
onKeyStroke('Escape', () => {
  if (!open.value) return
  open.value = false
  buttonEl.value?.focus()
})
onDeactivated(() => (open.value = false))

// Scrolls only the rail itself; scrollIntoView would also move the whole page.
watch(
  () => [props.activeCategory, catalog.categories.length],
  async () => {
    await nextTick()
    const rail = railEl.value
    const chip = rail?.querySelector<HTMLElement>('.is-active')
    if (!rail || !chip) return
    rail.scrollTo({ left: chip.offsetLeft - (rail.clientWidth - chip.clientWidth) / 2, behavior: 'smooth' })
  },
  { immediate: true, flush: 'post' },
)
</script>

<template>
  <div class="feedbar">
    <div class="row">
      <nav v-if="rail" ref="railEl" class="rail hide-scrollbar" aria-label="分类">
        <RouterLink :to="media === 'video' ? '/videos' : '/'" class="chip" :class="{ 'is-active': !activeCategory }">
          {{ media === 'video' ? '全部' : '精选' }}
        </RouterLink>
        <RouterLink
          v-for="c in catalog.categories"
          :key="c.id"
          :to="media === 'video' ? `/videos/c/${c.id}` : `/c/${c.id}`"
          class="chip"
          :class="{ 'is-active': activeCategory === c.id }"
        >
          {{ c.name }}
        </RouterLink>
      </nav>
      <div v-else class="spacer" />

      <div ref="filterEl" class="filter">
        <button
          ref="buttonEl"
          class="filter-btn"
          type="button"
          :aria-expanded="open"
          aria-controls="feed-filters"
          aria-label="筛选"
          @click="open = !open"
        >
          <Icon name="sliders" :size="20" />
          <i v-if="filtered" class="dot" />
        </button>

        <Transition name="pop">
          <div v-if="open" id="feed-filters" class="popover" role="group" aria-label="筛选">
            <div class="group" role="group" aria-label="画幅">
              <button
                v-for="o in ORIENTATIONS"
                :key="o.value"
                class="chip"
                type="button"
                :aria-pressed="orientation === o.value"
                @click="orientation = o.value"
              >
                {{ o.label }}
              </button>
            </div>
            <div v-if="sources.length > 1" class="group" role="group" aria-label="来源">
              <button class="chip" type="button" :aria-pressed="provider === 'all'" @click="provider = 'all'">
                全部来源
              </button>
              <button
                v-for="p in sources"
                :key="p.id"
                class="chip"
                type="button"
                :aria-pressed="provider === p.id"
                @click="provider = p.id"
              >
                {{ providerName(p.id) }}
              </button>
            </div>
            <button v-if="filtered" class="reset" type="button" @click="reset">清除筛选</button>
          </div>
        </Transition>
      </div>
    </div>
  </div>
</template>

<style scoped>
.feedbar {
  position: sticky;
  top: calc(var(--header-h) + var(--safe-top));
  z-index: 30;
  margin: 0 calc(var(--gutter) * -1);
  padding: 10px var(--gutter);
  background: rgba(15, 15, 17, 0.8);
  -webkit-backdrop-filter: blur(18px) saturate(1.4);
  backdrop-filter: blur(18px) saturate(1.4);
}

@media (max-width: 768px) {
  .feedbar {
    top: var(--safe-top);
  }
}

.row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.rail {
  display: flex;
  flex: 1;
  gap: 8px;
  min-width: 0;
  overflow-x: auto;
  scroll-snap-type: x proximity;
}

.rail .chip {
  scroll-snap-align: center;
}

.spacer {
  flex: 1;
}

.filter {
  position: relative;
  flex: none;
}

.filter-btn {
  position: relative;
  display: grid;
  place-items: center;
  width: 40px;
  height: 40px;
  border-radius: 50%;
  border: 1px solid var(--line);
  color: var(--text-2);
}

.filter-btn[aria-expanded='true'] {
  color: var(--text);
  border-color: var(--line-strong);
}

.dot {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--text);
}

.popover {
  position: absolute;
  top: calc(100% + 10px);
  right: 0;
  z-index: 5;
  display: flex;
  flex-direction: column;
  gap: 14px;
  width: min(340px, calc(100vw - 2 * var(--gutter)));
  padding: 16px;
  border-radius: 16px;
  border: 1px solid var(--line-strong);
  background: var(--surface);
  box-shadow: 0 18px 50px rgba(0, 0, 0, 0.5);
  transform-origin: top right;
}

.group {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.reset {
  align-self: flex-start;
  color: var(--text-2);
  font-size: 13px;
  text-decoration: underline;
  text-decoration-color: var(--line-strong);
  text-underline-offset: 3px;
}

.reset:hover {
  color: var(--text);
}

.pop-enter-active,
.pop-leave-active {
  transition:
    opacity 0.15s,
    transform 0.25s var(--ease);
}

.pop-enter-from,
.pop-leave-to {
  opacity: 0;
  transform: translateY(-6px) scale(0.97);
}
</style>
