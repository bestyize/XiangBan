<script setup lang="ts">
withDefaults(defineProps<{name:string;size?:number}>(),{size:20})
const paths:Record<string,string[]>={
  eye:['M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7-10-7-10-7Z','M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0'],
  comment:['M21 11.5a8.5 8.5 0 0 1-8.5 8.5H3l2-5a8.5 8.5 0 1 1 16-3.5Z'],
  layers:['m12 3 10 5-10 5L2 8Z','m2 12 10 5 10-5M2 16l10 5 10-5'],
  compass:['M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0','m16 8-3 5-5 3 3-5Z'],
  news:['M5 3h14v18H5Z','M8 7h8M8 11h8M8 15h5'],sparkle:['m12 3 2.5 6.5L21 12l-6.5 2.5L12 21l-2.5-6.5L3 12l6.5-2.5Z'],
  star:['m12 3 2.8 5.7 6.2.9-4.5 4.4 1.1 6.2-5.6-3-5.6 3 1.1-6.2L3 9.6l6.2-.9Z'],
  github:['M9 19c-5 1-5-2-7-2m14 4v-3.4c0-1 .1-1.5-.5-2 3.5-.4 7-1.7 7-7a5.5 5.5 0 0 0-1.5-3.8c.1-1-.1-2-.5-2.8 0 0-1.2-.4-4 1.5a13 13 0 0 0-7 0C6 1.6 4.8 2 4.8 2a6 6 0 0 0-.5 2.8A5.5 5.5 0 0 0 3 8.5c0 5.3 3.5 6.6 7 7-.5.5-.7 1.2-.7 2V21'],
  terminal:['m5 7 5 5-5 5','M13 17h6'],code:['m8 6-6 6 6 6m8-12 6 6-6 6m-2-15-4 18'],doc:['M6 3h8l4 4v14H6Z','M14 3v5h5M9 12h6M9 16h4'],
  mic:['M9 5a3 3 0 0 1 6 0v7a3 3 0 0 1-6 0Z','M6 10v2a6 6 0 0 0 12 0v-2M12 18v4M8 22h8'],bolt:['m13 2-9 12h7l-1 8 10-13h-7Z'],
  external:['M14 3h7v7m0-7L10 14','M10 3H4v17h17v-6'],arrow:['m9 5 7 7-7 7'],down:['m6 9 6 6 6-6'],bookmark:['M6 3h12v18l-6-4-6 4Z'],
  clock:['M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0','M12 7v5l3 2'],globe:['M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0','M3 12h18M12 3c5 5 5 13 0 18-5-5-5-13 0-18'],info:['M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0','M12 11v6M12 7h.01'],refresh:['M20 7v5h-5','M4 17v-5h5','M6 6a8 8 0 0 1 13 2M5 16a8 8 0 0 0 13 2'],tag:['M3 3h8l10 10-8 8L3 11Z','M7 7h.01'],
}
</script>
<template><svg :width="size" :height="size" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path v-for="(path,i) in paths[name]||paths.doc" :key="i" :d="path"/></svg></template>
