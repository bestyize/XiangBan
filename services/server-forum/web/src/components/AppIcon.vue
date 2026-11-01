<script setup lang="ts">
withDefaults(defineProps<{ name: string; size?: number }>(), { size: 22 })
const paths: Record<string, string[]> = {
  grid: ['M4 3h5a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1Z','M15 3h5a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1h-5a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1Z','M4 14h5a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1v-5a1 1 0 0 1 1-1Z','M15 14h5a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1h-5a1 1 0 0 1-1-1v-5a1 1 0 0 1 1-1Z'],
  chat: ['M4 4h16v13h-8l-5 4v-4H4Z'],
  ai: ['m12 3 2.5 6.5L21 12l-6.5 2.5L12 21l-2.5-6.5L3 12l6.5-2.5Z'],
  create: ['m15 4 5 5-10 10-6 1 1-6Z', 'm12 7 5 5', 'M5 3v4M3 5h4'],
  community: ['M5 4h14a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2h-7l-6 4v-4H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2Z', 'M7 9h10M7 13h6'],
  me: ['M16 8a4 4 0 1 1-8 0 4 4 0 0 1 8 0Z', 'M4 21v-2a8 8 0 0 1 16 0v2'],
  plus: ['M12 5v14M5 12h14'], search: ['M17 10a7 7 0 1 1-14 0 7 7 0 0 1 14 0Z', 'm15 15 6 6'],
  bell: ['M6 17v-6a6 6 0 0 1 12 0v6l2 2H4Z', 'M10 22h4'],
  arrow: ['m9 5 7 7-7 7'], back: ['m15 5-7 7 7 7'],
  bookmark: ['M6 3h12v18l-6-4-6 4Z'], folder: ['M3 6h6l2 3h10v11H3Z'],
  settings: ['M4 7h16M4 17h16', 'M8 4v6M16 14v6'],
  sun: ['M16 12a4 4 0 1 1-8 0 4 4 0 0 1 8 0Z', 'M12 2v2M12 20v2M2 12h2M20 12h2M5 5l1.5 1.5M17.5 17.5 19 19M5 19l1.5-1.5M17.5 6.5 19 5'],
  moon: ['M20.5 14.5A8.5 8.5 0 0 1 9.5 3.5a9 9 0 1 0 11 11Z'],
  download: ['M12 3v12m-5-5 5 5 5-5', 'M4 16v5h16v-5'],
  upload: ['M12 16V4m-5 5 5-5 5 5', 'M4 16v5h16v-5'],
  shield: ['m12 3 8 3v6c0 5-8 9-8 9s-8-4-8-9V6Z', 'm8 12 3 3 5-6'],
}
</script>
<template><svg :width="size" :height="size" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path v-for="(d,i) in paths[name] || paths.community" :key="i" :d="d" /></svg></template>
