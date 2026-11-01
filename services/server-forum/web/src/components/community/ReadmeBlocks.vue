<script setup lang="ts">
import type { ReadmeBlock } from '@/api/discover'
defineProps<{ blocks:ReadmeBlock[] }>()
const level = (l:number) => (l <= 2 ? 'h3' : l === 3 ? 'h4' : 'h5')
</script>
<template><div class="readme">
  <template v-for="(b, i) in blocks" :key="i">
    <component :is="level(b.l)" v-if="b.t === 'h'" class="rh" :class="`l${Math.min(b.l, 4)}`">{{ b.x }}</component>
    <p v-else-if="b.t === 'p'" class="rp">{{ b.x }}</p>
    <blockquote v-else-if="b.t === 'quote'" class="rq">{{ b.x }}</blockquote>
    <div v-else-if="b.t === 'li'" class="rli" :style="{ paddingLeft:`${b.d * 18}px` }"><span class="mark" aria-hidden="true">{{ b.n != null ? `${b.n}.` : '•' }}</span><span>{{ b.x }}</span></div>
    <pre v-else-if="b.t === 'code'" class="rc"><code>{{ b.x }}</code></pre>
    <div v-else-if="b.t === 'table'" class="rt"><table><tbody><tr v-for="(row, r) in b.rows" :key="r"><component :is="r === 0 ? 'th' : 'td'" v-for="(cell, c) in row" :key="c">{{ cell }}</component></tr></tbody></table></div>
    <hr v-else class="rx" />
  </template>
</div></template>
<style scoped>
.readme { font-size:14px; line-height:1.75; overflow-wrap:anywhere; color:var(--ink); }
.rh { margin:18px 0 6px; font-weight:700; line-height:1.4; } .rh.l1,.rh.l2 { font-size:16px; } .rh.l3 { font-size:15px; } .rh.l4 { font-size:14px; }
.rp { margin:0 0 10px; } .rq { margin:0 0 10px; padding:2px 0 2px 12px; border-left:3px solid var(--line-strong); color:var(--ink-2); }
.rli { display:flex; gap:8px; margin:0 0 4px; } .mark { flex:none; min-width:14px; color:var(--ink-3); }
.rc { margin:6px 0 12px; padding:10px 12px; border-radius:10px; background:#1f2328; color:#e6edf3; font:12px/1.7 ui-monospace,Menlo,monospace; overflow-x:auto; white-space:pre; }
.rc code { font:inherit; }
.rt { margin:6px 0 12px; overflow-x:auto; } table { border-collapse:collapse; font-size:13px; } th,td { border:1px solid var(--line); padding:6px 10px; text-align:left; vertical-align:top; } th { background:var(--surface-2); font-weight:600; }
.rx { border:0; border-top:1px solid var(--line); margin:16px 0; }
</style>
