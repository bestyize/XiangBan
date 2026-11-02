<script setup lang="ts">
import { computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { api } from '@/api/forum'
import { useCommunityResource } from './resource'
import ResourceTopicRow from './ResourceTopicRow.vue'
import RequestState from './RequestState.vue'
import PagerNav from '@/components/PagerNav.vue'
const props = withDefaults(defineProps<{ category:string; title?:string; intro?:string }>(), {title:'图文分享',intro:''})
const route=useRoute(),router=useRouter()
const page=computed(()=>Math.max(1,Math.floor(Number(route.query.articles_page)||1)))
const feed=useCommunityResource(()=>api.topics({category:props.category,page:page.value,sort:'new'}))
watch([()=>props.category,page],feed.load,{immediate:true})
function go(value:number) { void router.replace({query:{...route.query,articles_page:value===1?undefined:value}}) }
</script>
<template>
  <section class="editorial-feed" :aria-label="title">
    <header><h2>{{ title }}</h2><span v-if="feed.data.value">{{ feed.data.value.total }} 篇</span></header>
    <p v-if="intro" class="intro">{{ intro }}</p>
    <RequestState :loading="feed.loading.value" :error="feed.error.value" @retry="feed.load" />
    <div v-if="feed.data.value?.items.length" class="articles"><ResourceTopicRow v-for="article in feed.data.value.items" :key="article.id" :topic="article" /></div>
    <p v-else-if="!feed.loading.value && !feed.error.value" class="intro">暂无图文分享。</p>
    <PagerNav v-if="feed.data.value" :page="feed.data.value.page" :page-size="feed.data.value.page_size" :total="feed.data.value.total" @go="go" />
  </section>
</template>
<style scoped>
.editorial-feed{padding:12px}.editorial-feed header{display:flex;align-items:baseline;gap:10px;margin-bottom:10px}.editorial-feed h2{font-size:16px;margin:0}.editorial-feed header span,.intro{font-size:12px;color:var(--ink-3)}.intro{margin:6px 0 12px;line-height:1.6}.articles{display:grid;gap:8px}@media(min-width:1100px){.articles{grid-template-columns:repeat(2,minmax(0,1fr));gap:12px}}
</style>
