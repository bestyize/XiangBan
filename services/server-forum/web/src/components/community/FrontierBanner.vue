<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import type { NewsItem } from '@/api/discover'
import { carouselIndex, swipeStep } from '@/lib/frontierCarousel'
import { displayDate, webLink } from './contract'
import DiscoverGlyph from './DiscoverGlyph.vue'

const props=defineProps<{items:readonly NewsItem[]}>()
const active=ref(0), paused=ref(false), hovered=ref(false), focused=ref(false), reduced=ref(false)
const broken=ref<string[]>([])
let timer:ReturnType<typeof setInterval>|undefined, media:MediaQueryList|undefined
let startX=0,startY=0
// Keep these nodes stable: article assets are no-store, so rebuilding them on
// every slide would repeatedly download the same images.
const suggestions=computed(()=>props.items.slice(1,3))
watch(()=>props.items.map(item=>item.key),(keys,old)=>{
  const index=keys.indexOf(old?.[active.value] || '')
  active.value=index<0?0:index
})
function move(step:number,manual=true) {
  active.value=carouselIndex(active.value,step,props.items.length)
  if(manual) paused.value=true
}
function select(index:number) { active.value=index;paused.value=true }
function touchStart(event:TouchEvent) { startX=event.touches[0]?.clientX||0;startY=event.touches[0]?.clientY||0 }
function touchEnd(event:TouchEvent) {
  const touch=event.changedTouches[0]
  if(!touch)return
  const step=swipeStep(touch.clientX-startX,touch.clientY-startY)
  if(step){event.preventDefault();move(step)}
}
function focusOut(event:FocusEvent) { focused.value=!!event.relatedTarget && (event.currentTarget as HTMLElement).contains(event.relatedTarget as Node) }
function preference(event:MediaQueryListEvent) { reduced.value=event.matches }
onMounted(()=>{
  media=window.matchMedia('(prefers-reduced-motion: reduce)');reduced.value=media.matches
  media.addEventListener('change',preference)
  timer=setInterval(()=>{if(!paused.value&&!hovered.value&&!focused.value&&!reduced.value&&!document.hidden&&props.items.length>1)move(1,false)},6000)
})
onBeforeUnmount(()=>{clearInterval(timer);media?.removeEventListener('change',preference)})
</script>
<template>
  <div class="frontier-feature">
    <section class="focus-banner" aria-label="小伴前沿焦点推荐" aria-roledescription="轮播" @pointerenter="hovered=true" @pointerleave="hovered=false" @focusin="focused=true" @focusout="focusOut" @keydown.left.prevent="move(-1)" @keydown.right.prevent="move(1)">
      <div class="focus-viewport" @touchstart.passive="touchStart" @touchend="touchEnd">
        <RouterLink v-for="(item,index) in items" :key="item.key" :to="`/frontier/${encodeURIComponent(item.key)}`" class="focus-slide" :hidden="index!==active" :aria-label="`${index+1} / ${items.length}：${item.title}`">
          <div class="focus-image"><img v-if="webLink(item.cover_url)&&!broken.includes(item.key)" :src="webLink(item.cover_url)" alt="" :loading="index===0?'eager':'lazy'" decoding="async" draggable="false" @error="broken.push(item.key)"><DiscoverGlyph v-else name="news" :size="40"/></div>
          <div class="focus-caption"><h3>{{ item.title }}</h3><p><span v-if="item.source">{{ item.source }}</span><time v-if="item.source_published_at||item.published_at">{{ displayDate(item.source_published_at||item.published_at) }}</time><span v-if="item.sample">示例</span></p></div>
        </RouterLink>
      </div>
      <div v-if="items.length>1" class="focus-controls">
        <div class="focus-dots" aria-label="选择前沿推荐"><button v-for="(item,index) in items" :key="item.key" type="button" :class="{active:index===active}" :aria-label="`第 ${index+1} 条：${item.title}`" :aria-current="index===active?'true':undefined" @click="select(index)"><span/></button></div>
        <button v-if="!reduced" class="play-control" type="button" :aria-label="paused?'播放前沿轮播':'暂停前沿轮播'" @click="paused=!paused">{{ paused?'播放':'暂停' }}</button>
        <button type="button" class="direction" aria-label="上一条前沿" @click="move(-1)"><DiscoverGlyph name="arrow" :size="16" class="previous"/></button><button type="button" class="direction" aria-label="下一条前沿" @click="move(1)"><DiscoverGlyph name="arrow" :size="16"/></button>
      </div>
    </section>
    <div v-if="suggestions.length" class="frontier-suggestions"><RouterLink v-for="item in suggestions" :key="item.key" :to="`/frontier/${encodeURIComponent(item.key)}`" class="frontier-suggestion"><div class="suggestion-image"><img v-if="webLink(item.cover_url)&&!broken.includes(item.key)" :src="webLink(item.cover_url)" alt="" loading="lazy" decoding="async" @error="broken.push(item.key)"><DiscoverGlyph v-else name="news" :size="28"/></div><h3>{{ item.title }}</h3></RouterLink></div>
  </div>
</template>
<style scoped>
.frontier-feature{min-width:0}.focus-banner{min-width:0;border-radius:12px;overflow:hidden;border:1px solid var(--line);background:var(--surface-2);color:var(--ink);display:flex;flex-direction:column}.focus-viewport{min-width:0;touch-action:pan-y}.focus-slide{display:block;color:inherit}.focus-slide[hidden]{display:none}.focus-image,.suggestion-image{aspect-ratio:16/9;position:relative;overflow:hidden;display:grid;place-items:center;background:var(--surface-2);color:var(--ink-3)}.focus-image img,.suggestion-image img{width:100%;height:100%;object-fit:cover;display:block}.focus-caption{padding:12px 16px 2px;background:linear-gradient(var(--surface),var(--surface-2))}h3{font-size:14px;font-weight:400;line-height:1.6;margin:0;display:-webkit-box;-webkit-box-orient:vertical;-webkit-line-clamp:2;overflow:hidden;overflow-wrap:anywhere}.focus-caption h3{min-height:3.2em}.focus-caption p{display:flex;flex-wrap:wrap;gap:4px 10px;font-size:12px;line-height:1.5;color:var(--ink-3);margin:8px 0 0}.focus-controls{display:flex;align-items:center;gap:4px;padding:4px 10px 8px;margin-top:auto}.focus-controls button{border:0;color:inherit;background:none;cursor:pointer;min-height:36px;font-size:12px}.focus-controls button:focus-visible,.focus-slide:focus-visible,.frontier-suggestion:focus-visible{outline:2px solid var(--brand);outline-offset:-3px}.focus-dots{display:flex;align-items:center;margin-right:auto}.focus-dots button{display:grid;place-items:center;min-width:24px;padding:0 4px}.focus-dots span{width:6px;height:6px;border-radius:50%;background:var(--line-strong);transition:width .18s,background .18s}.focus-dots .active span{width:18px;border-radius:4px;background:var(--brand)}.direction{display:grid;place-items:center;width:36px;border-radius:6px}.direction:hover,.play-control:hover{background:var(--surface)}.previous{transform:rotate(180deg)}.play-control{padding:0 8px}.frontier-suggestions{display:none}.frontier-suggestion{min-width:0;color:var(--ink);border-radius:10px;overflow:hidden;display:block;background:var(--surface)}.frontier-suggestion h3{padding:8px 0;min-height:3.2em}.suggestion-image{border-radius:8px}
@media(min-width:1024px){@container page (min-width:960px){.frontier-feature{display:grid;grid-template-columns:minmax(0,2fr) minmax(0,1fr);gap:20px;align-items:stretch}.frontier-suggestions{display:grid;gap:12px;align-content:start}.focus-caption{padding-top:14px}.focus-controls{padding-bottom:10px}}}
@media(prefers-reduced-motion:reduce){.focus-dots span{transition:none}}
</style>
