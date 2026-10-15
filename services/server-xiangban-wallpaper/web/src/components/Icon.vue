<script setup lang="ts">
import { computed } from 'vue'

export type IconName =
  | 'compass'
  | 'image'
  | 'grid'
  | 'play'
  | 'heart'
  | 'heart-fill'
  | 'user'
  | 'search'
  | 'close'
  | 'download'
  | 'share'
  | 'device'
  | 'chevron-left'
  | 'chevron-right'
  | 'external'
  | 'volume'
  | 'volume-off'
  | 'sliders'
  | 'refresh'
  | 'key'
  | 'network'
  | 'database'
  | 'logout'
  | 'plus'
  | 'trash'
  | 'check'
  | 'pulse'
  | 'upload'
  | 'sparkles'
  | 'book'

const props = withDefaults(defineProps<{ name: IconName; size?: number }>(), { size: 22 })

const PATHS: Record<IconName, string> = {
  compass: '<circle cx="12" cy="12" r="9"/><path d="m15.5 8.5-2 5-5 2 2-5z"/>',
  image:
    '<rect x="3.5" y="5" width="17" height="14" rx="3"/><circle cx="9" cy="10.5" r="1.5"/><path d="m4.5 17 4.5-4.5 3 3 3-3.5 4.5 5"/>',
  grid: '<rect x="4" y="4" width="7" height="7" rx="1.5"/><rect x="13" y="4" width="7" height="7" rx="1.5"/><rect x="4" y="13" width="7" height="7" rx="1.5"/><rect x="13" y="13" width="7" height="7" rx="1.5"/>',
  play: '<rect x="3.5" y="5" width="17" height="14" rx="3"/><path d="m10.5 9.5 4 2.5-4 2.5z"/>',
  heart: '<path d="M12 20s-7.5-4.6-7.5-10.2A4.3 4.3 0 0 1 12 7.4a4.3 4.3 0 0 1 7.5 2.4C19.5 15.4 12 20 12 20z"/>',
  'heart-fill': '<path fill="currentColor" d="M12 20s-7.5-4.6-7.5-10.2A4.3 4.3 0 0 1 12 7.4a4.3 4.3 0 0 1 7.5 2.4C19.5 15.4 12 20 12 20z"/>',
  user: '<circle cx="12" cy="8.5" r="3.5"/><path d="M5 20c.8-3.6 3.6-5.5 7-5.5s6.2 1.9 7 5.5"/>',
  search: '<circle cx="11" cy="11" r="6.5"/><path d="m16 16 4 4"/>',
  close: '<path d="M6 6l12 12M18 6 6 18"/>',
  download: '<path d="M12 4v11m0 0-4-4m4 4 4-4M5 19h14"/>',
  share: '<path d="M12 15V4m0 0L8 8m4-4 4 4M6 12v6a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2v-6"/>',
  device: '<rect x="8" y="3" width="8" height="15" rx="2"/><path d="M4 21h16"/>',
  'chevron-left': '<path d="m14.5 6-6 6 6 6"/>',
  'chevron-right': '<path d="m9.5 6 6 6-6 6"/>',
  external: '<path d="M14 5h5v5M19 5l-8 8M18 14v4a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h4"/>',
  volume: '<path d="M4 10v4h3.5l4.5 3.5v-11L7.5 10zM15.5 9a4 4 0 0 1 0 6M18 6.5a7.5 7.5 0 0 1 0 11"/>',
  'volume-off': '<path d="M4 10v4h3.5l4.5 3.5v-11L7.5 10zM16 9.5l4 5m0-5-4 5"/>',
  sliders: '<path d="M5 8h9m4 0h1M5 16h1m4 0h9"/><circle cx="16" cy="8" r="2"/><circle cx="8" cy="16" r="2"/>',
  refresh: '<path d="M19 12a7 7 0 1 1-2.3-5.2M19 4v4h-4"/>',
  key: '<circle cx="8" cy="15" r="3.5"/><path d="M10.5 12.5 19 4m-3 3 2.5 2.5M13.5 9.5 16 12"/>',
  network:
    '<circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3c2.5 2.6 3.8 5.6 3.8 9s-1.3 6.4-3.8 9c-2.5-2.6-3.8-5.6-3.8-9S9.5 5.6 12 3z"/>',
  database:
    '<ellipse cx="12" cy="6" rx="7" ry="3"/><path d="M5 6v6c0 1.7 3.1 3 7 3s7-1.3 7-3V6M5 12v6c0 1.7 3.1 3 7 3s7-1.3 7-3v-6"/>',
  logout: '<path d="M14 4h4a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2h-4M10 16l-4-4 4-4M6 12h10"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  trash: '<path d="M5 7h14M10 7V5h4v2M7 7l1 12h8l1-12M10 11v5M14 11v5"/>',
  check: '<path d="m5 12.5 4.5 4.5L19 7.5"/>',
  pulse: '<path d="M3 12h4l2.5-6 4 12 2.5-6H21"/>',
  upload: '<path d="M12 15V4m0 0L8 8m4-4 4 4M5 19h14"/>',
  sparkles: '<path d="m12 3 1.9 5.8a2 2 0 0 0 1.3 1.3L21 12l-5.8 1.9a2 2 0 0 0-1.3 1.3L12 21l-1.9-5.8a2 2 0 0 0-1.3-1.3L3 12l5.8-1.9a2 2 0 0 0 1.3-1.3zM19 3v4M21 5h-4M5 19v2M6 20H4"/>',
  book: '<path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1-2.5-2.5Z"/><path d="M6 6h10M6 10h10"/>',
}

const markup = computed(() => PATHS[props.name])
</script>

<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="1.6"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
    v-html="markup"
  />
</template>
