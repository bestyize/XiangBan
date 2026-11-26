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
  | 'chevron-down'
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
  | 'minus'
  | 'trash'
  | 'check'
  | 'pulse'
  | 'upload'
  | 'sparkles'
  | 'book'
  | 'dashboard'
  | 'package'
  | 'megaphone'
  | 'more'
  | 'sun'
  | 'moon'
  | 'menu'
  | 'list'
  | 'copy'
  | 'alert'
  | 'edit'
  | 'eye'
  | 'eye-off'
  | 'bolt'
  | 'monitor'
  | 'phone'
  | 'calendar'
  | 'shield'
  | 'palette'
  | 'os-win'
  | 'os-apple'
  | 'os-android'

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
  'chevron-down': '<path d="m6 9 6 6 6-6"/>',
  external: '<path d="M14 5h5v5M19 5l-8 8M18 14v4a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h4"/>',
  volume: '<path d="M4 10v4h3.5l4.5 3.5v-11L7.5 10zM15.5 9a4 4 0 0 1 0 6M18 6.5a7.5 7.5 0 0 1 0 11"/>',
  'volume-off': '<path d="M4 10v4h3.5l4.5 3.5v-11L7.5 10zM16 9.5l4 5m0-5-4 5"/>',
  sliders: '<path d="M5 8h9m4 0h1M5 16h1m4 0h9"/><circle cx="16" cy="8" r="2"/><circle cx="8" cy="16" r="2"/>',
  refresh: '<path d="M19 12a7 7 0 1 1-2.3-5.2M19 4v4h-4"/>',
  key: '<circle cx="8" cy="15" r="3.5"/><path d="M10.5 12.5 19 4m-3 3 2.5 2.5M13.5 9.5 16 12"/>',
  network:
    '<circle cx="12" cy="5" r="2.5"/><circle cx="5" cy="19" r="2.5"/><circle cx="19" cy="19" r="2.5"/><path d="M12 7.5V12m0 0-5.3 4.8M12 12l5.3 4.8"/>',
  database:
    '<ellipse cx="12" cy="6" rx="7" ry="3"/><path d="M5 6v6c0 1.7 3.1 3 7 3s7-1.3 7-3V6M5 12v6c0 1.7 3.1 3 7 3s7-1.3 7-3v-6"/>',
  logout: '<path d="M14 4h4a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2h-4M10 16l-4-4 4-4M6 12h10"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  minus: '<path d="M5 12h14"/>',
  trash: '<path d="M5 7h14M10 7V5h4v2M7 7l1 12h8l1-12M10 11v5M14 11v5"/>',
  check: '<path d="m5 12.5 4.5 4.5L19 7.5"/>',
  pulse: '<path d="M3 12h4l2.5-6 4 12 2.5-6H21"/>',
  upload: '<path d="M12 15V4m0 0L8 8m4-4 4 4M5 19h14"/>',
  sparkles: '<path d="m12 3 1.9 5.8a2 2 0 0 0 1.3 1.3L21 12l-5.8 1.9a2 2 0 0 0-1.3 1.3L12 21l-1.9-5.8a2 2 0 0 0-1.3-1.3L3 12l5.8-1.9a2 2 0 0 0 1.3-1.3z"/>',
  book: '<path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1-2.5-2.5Z"/><path d="M6 6h10M6 10h10"/>',
  dashboard: '<rect x="3" y="3" width="7" height="9" rx="2"/><rect x="14" y="3" width="7" height="5" rx="2"/><rect x="14" y="12" width="7" height="9" rx="2"/><rect x="3" y="16" width="7" height="5" rx="2"/>',
  package: '<path d="m12 3 8 4.5v9L12 21l-8-4.5v-9L12 3Z"/><path d="M12 12l8-4.5"/><path d="M12 12v9"/><path d="M12 12 4 7.5"/>',
  megaphone: '<path d="m3 11 15-5v12L3 13v-2Z"/><path d="M11 14v4a2 2 0 0 1-2 2H8"/><path d="M18 8a4 4 0 0 1 0 8"/>',
  more: '<circle cx="5" cy="12" r="1.4" fill="currentColor" stroke="none"/><circle cx="12" cy="12" r="1.4" fill="currentColor" stroke="none"/><circle cx="19" cy="12" r="1.4" fill="currentColor" stroke="none"/>',
  sun: '<circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41"/>',
  moon: '<path d="M20.5 14.5A8.5 8.5 0 1 1 9.5 3.5a6.8 6.8 0 0 0 11 11Z"/>',
  menu: '<path d="M4 7h16M4 12h16M4 17h16"/>',
  list: '<path d="M8 6h13M8 12h13M8 18h13M3.5 6h.01M3.5 12h.01M3.5 18h.01"/>',
  copy: '<rect x="9" y="9" width="11" height="11" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>',
  alert: '<path d="M12 9v4"/><path d="M12 17h.01"/><path d="M10.3 3.9 2.6 17.4A2 2 0 0 0 4.3 20.4h15.4a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0Z"/>',
  edit: '<path d="M12 20h9"/><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z"/>',
  eye: '<path d="M2 12s3.6-7 10-7 10 7 10 7-3.6 7-10 7S2 12 2 12Z"/><circle cx="12" cy="12" r="3"/>',
  'eye-off': '<path d="M9.9 4.2A10.9 10.9 0 0 1 12 4c6.4 0 10 8 10 8a18.5 18.5 0 0 1-2.2 3.2"/><path d="M6.6 6.6A13.5 13.5 0 0 0 2 12s3.6 8 10 8a9.7 9.7 0 0 0 5.4-1.6"/><path d="m2 2 20 20"/>',
  bolt: '<polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>',
  monitor: '<rect x="2" y="3" width="20" height="14" rx="2"/><path d="M8 21h8M12 17v4"/>',
  phone: '<rect x="6" y="2" width="12" height="20" rx="3"/><path d="M11 18h2"/>',
  calendar: '<rect x="3" y="4" width="18" height="18" rx="2"/><path d="M16 2v4M8 2v4M3 10h18"/>',
  shield: '<path d="M12 3 5 6v6c0 4.5 3 7.7 7 9 4-1.3 7-4.5 7-9V6l-7-3Z"/>',
  palette: '<circle cx="13.5" cy="6.5" r=".7" fill="currentColor"/><circle cx="17.5" cy="10.5" r=".7" fill="currentColor"/><circle cx="8.5" cy="7.5" r=".7" fill="currentColor"/><circle cx="6.5" cy="12.5" r=".7" fill="currentColor"/><path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.9 0 1.6-.7 1.6-1.6 0-.4-.2-.8-.4-1.1-.3-.3-.4-.7-.4-1.1 0-.9.7-1.6 1.6-1.6H16c3.3 0 6-2.7 6-6 0-5-4.5-8.6-10-8.6Z"/>',
  'os-win': '<rect x="3" y="3" width="8" height="8" rx="1"/><rect x="13" y="3" width="8" height="8" rx="1"/><rect x="3" y="13" width="8" height="8" rx="1"/><rect x="13" y="13" width="8" height="8" rx="1"/>',
  'os-apple': '<path d="M12 20.94c1.5 0 2.75 1.06 4 1.06 3 0 6-8 6-12.22A4.91 4.91 0 0 0 17 5c-2.22 0-4 1.44-5 2-1-.56-2.78-2-5-2a4.9 4.9 0 0 0-5 4.78C2 14 5 22 8 22c1.25 0 2.5-1.06 4-1.06Z"/><path d="M10 2c1 .5 2 2 2 5"/>',
  'os-android': '<path d="M5 16v-3a7 7 0 0 1 14 0v3H5Z"/><path d="M9 6 7 3M15 6l2-3"/><circle cx="9.5" cy="11.5" r=".8" fill="currentColor" stroke="none"/><circle cx="14.5" cy="11.5" r=".8" fill="currentColor" stroke="none"/><path d="M5 18h14"/>',
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
    stroke-width="1.75"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
    v-html="markup"
  />
</template>
