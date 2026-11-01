<script setup lang="ts">
import { computed, ref, onMounted, onBeforeUnmount, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import ModalDialog from '@/components/ModalDialog.vue'
import CreationPreviewContent from '@/components/creation/CreationPreviewContent.vue'
import ClientCreationPrompt from '@/components/creation/ClientCreationPrompt.vue'
import BuiltinGamePlayer from '@/components/creation/BuiltinGamePlayer.vue'
import { BUILTIN_GAMES } from '@/lib/builtinCreation'
import BuiltinEffectPreview from '@/components/creation/BuiltinEffectPreview.vue'
import BuiltinWidgetPreview from '@/components/creation/BuiltinWidgetPreview.vue'
import BuiltinPetPreview from '@/components/creation/BuiltinPetPreview.vue'
import { BUILTIN_EFFECTS } from '@/lib/builtinEffects'
import { BUILTIN_WIDGETS } from '@/lib/builtinWidgets'
import { BUILTIN_PETS } from '@/lib/builtinPets'
import { CREATION_IDEAS } from '@/lib/creationIdeas'
import { useHead } from '@/lib/head'

useHead(() => ({ title:'创造' }))
const route = useRoute(), router = useRouter()
const categories = [{key:'',label:'全部'},{key:'game',label:'游戏'},{key:'wallpaper',label:'壁纸'},{key:'widget',label:'小组件'},{key:'effect',label:'特效'},{key:'pet',label:'伙伴'}]
const category = computed(() => categories.some(c => c.key === route.query.kind) ? String(route.query.kind) : '')
const games = computed(() => !category.value || category.value === 'game' ? BUILTIN_GAMES : [])
const decorationSections = computed(() => [
  {kind:'wallpaper',title:'让桌面有点不一样',items:CREATION_IDEAS.filter(item => item.kind === 'wallpaper') as Preview[]},
  {kind:'widget',title:'把日常放在眼前',items:BUILTIN_WIDGETS as Preview[]},
  {kind:'effect',title:'给桌面一点氛围',items:BUILTIN_EFFECTS as Preview[]},
  {kind:'pet',title:'找一位桌面小伙伴',items:BUILTIN_PETS as Preview[]},
].filter(section => !category.value || section.kind === category.value))
const promptOpen = ref(false), playing = ref('')
type Preview = {id:string; name:string; cover?:string; description:string; kind:string}
const preview = ref<Preview | null>(null)
function label(kind:string) { return categories.find(c => c.key === kind)?.label || kind }
function select(kind:string) { void router.push({path:'/create',query:kind ? {kind} : {}}) }
const wide = ref(false), showcase = ref<HTMLElement>(), content = ref<HTMLElement>()
let contentObserver:ResizeObserver | undefined
onMounted(() => {
  contentObserver=new ResizeObserver(([entry]) => { wide.value=(entry?.contentRect.width || 0) >= 1100 })
  if (content.value) contentObserver.observe(content.value)
})
onBeforeUnmount(() => contentObserver?.disconnect())
const featured = computed<Preview>(() => preview.value || (category.value && category.value !== 'game' ? decorationSections.value[0]?.items[0] : null) || {...BUILTIN_GAMES[0]!,kind:'game'})
watch(category,() => { preview.value=null })
function choose(item:Preview) { preview.value=item; if (wide.value) showcase.value?.scrollIntoView({block:'start',behavior:window.matchMedia('(prefers-reduced-motion: reduce)').matches ? 'instant' : 'smooth'}) }
function play() { const item=wide.value ? featured.value : preview.value; if (!item) return; playing.value=item.id; if (!wide.value) preview.value=null }
</script>
<template>
  <div ref="content" class="creation-home">
    <header class="creation-intro"><div><p class="eyebrow">小伴 · 官方内置</p><h1>让桌面，多一点喜欢。</h1><p class="intro-note">游戏、风景和桌面小伙伴，在这里发现，也在这里体验。</p></div></header><h1 class="sr-only mobile-title">创造</h1>
    <nav class="creation-tabs" aria-label="创造栏目"><RouterLink to="/create" class="active" aria-current="page">发现</RouterLink><RouterLink to="/me/works">我的作品</RouterLink></nav>
    <button class="creation-prompt" type="button" @click="promptOpen = true">
      <AppIcon name="ai" :size="29" /><span><strong>你想创造什么？</strong><small>一个游戏，一片风景，一位小伙伴…</small></span><AppIcon name="arrow" :size="20" />
    </button>
    <nav class="creation-filters" aria-label="作品分类"><button v-for="item in categories" :key="item.key" type="button" :class="{active:category === item.key}" :aria-pressed="category === item.key" @click="select(item.key)">{{ item.label }}</button></nav>
    <section v-if="wide" ref="showcase" class="desktop-showcase" aria-label="作品预览"><CreationPreviewContent :item="featured" wide :active="!playing" @play="play" /></section>
    <section v-if="games.length" aria-labelledby="builtin-games-heading">
      <header class="creation-section"><h2 id="builtin-games-heading">给自己几分钟好玩</h2><span>{{ games.length }} 款</span></header>
      <div class="creation-grid"><button v-for="game in games" :key="game.id" class="creation-card" type="button" :aria-label="`查看${game.name}`" :class="{selected:wide && featured.id === game.id}" :aria-pressed="wide ? featured.id === game.id : undefined" @click="choose({...game,kind:'game'})">
        <div class="creation-cover" :class="game.id"><img :src="game.cover" :alt="`${game.name}封面`" loading="lazy" /></div><h3>{{ game.name }}</h3><div class="creation-meta"><span>游戏</span><span><svg viewBox="0 0 20 20" aria-hidden="true"><rect x="5" y="2" width="10" height="16" rx="2"/><path d="M9 15h2"/></svg>网页可玩</span></div>
      </button></div>
    </section>
    <section v-for="section in decorationSections" :key="section.kind" :aria-labelledby="`builtin-${section.kind}-heading`">
      <header class="creation-section"><h2 :id="`builtin-${section.kind}-heading`">{{ section.title }}</h2><span>{{ section.kind === 'wallpaper' ? '电脑创作灵感' : `${section.items.length} 款 · 小伴官方` }}</span></header>
      <div class="creation-grid"><button v-for="item in section.items" :key="item.id" class="creation-card" type="button" :aria-label="`预览${item.name}`" :class="{selected:wide && featured.id === item.id}" :aria-pressed="wide ? featured.id === item.id : undefined" @click="choose({...item})">
        <div class="creation-cover" :class="{pet:item.kind === 'pet'}">
          <BuiltinEffectPreview v-if="item.kind === 'effect'" :id="item.id" />
          <BuiltinWidgetPreview v-else-if="item.kind === 'widget'" :id="item.id" />
          <BuiltinPetPreview v-else-if="item.kind === 'pet'" :id="item.id" />
          <img v-else :src="item.cover" :alt="`${item.name}灵感图`" loading="lazy" />
        </div><h3>{{ item.name }}</h3><div class="creation-meta"><span>{{ label(item.kind) }}</span><span><svg viewBox="0 0 20 20" aria-hidden="true"><rect x="2" y="3" width="16" height="11" rx="1.5"/><path d="M10 14v3M6 17h8"/></svg>{{ item.kind === 'wallpaper' ? '创作灵感' : '官方内置' }}</span></div>
      </button></div>
    </section>
    <ModalDialog :open="!wide && preview !== null" :title="preview?.name || '作品预览'" :width="640" @close="preview = null"><CreationPreviewContent v-if="!wide && preview" :item="preview" @play="play" /></ModalDialog>
    <BuiltinGamePlayer v-if="playing" :game-id="playing" @close="playing = ''" />
    <ClientCreationPrompt v-model:show="promptOpen" />
  </div>
</template>
<style scoped>
.creation-intro{display:none}
.creation-home{max-width:1000px;margin:0 auto;padding:12px 8px 24px;min-width:0}.creation-tabs{display:flex;gap:26px;align-items:center;margin:0 0 20px}.creation-tabs a{position:relative;display:flex;align-items:center;min-height:48px;padding-bottom:12px;font-size:20px;color:var(--ink-3)}.creation-tabs .active{color:var(--ink);font-weight:650}.creation-tabs .active:after{content:'';position:absolute;bottom:0;left:0;width:20px;height:3px;border-radius:3px;background:var(--brand)}
.creation-prompt{width:100%;display:flex;align-items:center;text-align:left;gap:12px;padding:18px 16px;border:0;border-radius:18px;background:var(--surface);color:var(--brand)}.creation-prompt>span{flex:1;min-width:0}.creation-prompt strong{display:block;color:var(--ink);font-size:15px;font-weight:650}.creation-prompt small{display:block;color:var(--ink-3);font-size:12px;margin-top:5px;line-height:1.7}.creation-prompt>svg{flex-shrink:0}.creation-prompt:hover{box-shadow:0 3px 18px #00000008}
.creation-filters{display:flex;gap:6px;overflow-x:auto;scrollbar-width:none;margin-top:16px;padding:2px 0}.creation-filters button{flex:none;min-height:44px;padding:0 12px;border:0;border-radius:8px;background:none;color:var(--ink-3);font-size:14px}.creation-filters .active{color:var(--paper);background:var(--ink)}.creation-section{display:flex;align-items:center;justify-content:space-between;gap:12px;margin:22px 0 16px}.creation-section h2{font-size:17px;font-weight:650;margin:0}.creation-section>span{font-size:12px;color:var(--ink-3);white-space:nowrap}
.creation-grid{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:24px 18px}.creation-card{min-width:0;display:block;padding:0;text-align:left;background:none;border:0;color:var(--ink);border-radius:16px}.creation-cover{aspect-ratio:1.12;overflow:hidden;border-radius:16px;background:var(--surface);transition:transform .18s}.creation-cover img{display:block;width:100%;height:100%;object-fit:contain}.creation-cover.jade-gomoku{background:#211b13}.creation-cover.ink-blade{background:#e7e0cd}.creation-cover.pro-cube{background:#0a1020}.creation-cover.pet{background:var(--surface-2)}.creation-cover.pet img{object-fit:contain;padding:12px}.creation-card:hover .creation-cover{transform:translateY(-2px)}.creation-card h3{font-size:14px;line-height:1.5;font-weight:550;margin:10px 0 5px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.creation-meta{display:flex;justify-content:space-between;gap:4px;font-size:11px;color:var(--ink-3)}.creation-meta>span:last-child{display:flex;align-items:center;gap:3px}.creation-meta svg{width:12px;height:12px;fill:none;stroke:currentColor;stroke-width:1.5}
.sr-only{position:absolute;width:1px;height:1px;overflow:hidden;clip-path:inset(50%)}
@media(max-width:1023px){.creation-grid{grid-template-columns:repeat(3,minmax(0,1fr))}}@media(max-width:600px){.creation-grid{grid-template-columns:repeat(2,minmax(0,1fr));gap:22px 14px}.creation-tabs a{font-size:18px}.creation-section h2{font-size:16px}.creation-filters{gap:2px}.creation-filters button{padding:0 10px;font-size:13px}.creation-home{padding-top:8px}}
</style>

<style scoped>
@media(min-width:1100px){
.creation-home{max-width:1240px;padding:20px 0 40px;display:grid;grid-template-columns:minmax(0,1fr) auto;column-gap:24px}.creation-home>section,.creation-filters{grid-column:1/-1}.creation-intro{display:block;grid-column:1;grid-row:1;margin-bottom:18px}.mobile-title{display:none}.eyebrow{font-size:12px;color:var(--brand);letter-spacing:.08em;margin:0 0 10px;font-weight:600}.creation-intro h1{font-size:30px;letter-spacing:-.035em;margin:0 0 12px;font-weight:650;line-height:1.35}.intro-note{font-size:14px;color:var(--ink-3);margin:0}.creation-tabs{grid-column:2;grid-row:1;margin:0 0 18px;align-self:end;justify-self:end;gap:24px}.creation-tabs a{font-size:16px;min-height:44px;padding-bottom:10px}.creation-prompt{grid-column:2;grid-row:2;justify-self:end;margin-bottom:20px;width:auto;padding:10px 16px;gap:10px;border:1px solid var(--line);border-radius:12px;align-self:center}.creation-prompt strong{font-size:14px}.creation-prompt small{display:none}.creation-prompt>svg:first-child{width:20px;height:20px}.creation-filters{grid-column:1;grid-row:2;margin:0 0 20px;gap:6px;border:0;padding:0;align-self:center}.creation-filters button{font-size:14px;min-height:38px;padding:0 16px;border-radius:20px}.creation-filters button:not(.active):hover{background:var(--surface)}.desktop-showcase{scroll-margin-top:88px}.creation-section{margin:34px 0 18px}.creation-section h2{font-size:20px;letter-spacing:-.02em}.creation-section>span{font-size:13px}.creation-grid{grid-template-columns:repeat(4,minmax(0,1fr));gap:20px}.creation-card{background:var(--surface);padding:8px 8px 14px;border:1px solid var(--line);border-radius:18px;transition:border-color .18s,box-shadow .18s}.creation-card.selected{border-color:var(--brand);box-shadow:0 0 0 1px var(--brand)}.creation-card:hover{border-color:var(--line-strong);box-shadow:var(--shadow)}.creation-card.selected:hover{border-color:var(--brand)}.creation-card h3{font-size:16px;font-weight:600;margin:12px 6px 6px}.creation-meta{font-size:12px;margin:0 6px;gap:8px}.creation-cover{aspect-ratio:1.5;border-radius:12px}.creation-cover.pet{aspect-ratio:1.35}.creation-card:hover .creation-cover{transform:none}
}
</style>
