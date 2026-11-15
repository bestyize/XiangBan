<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import { displayDate, webLink } from '@/components/community/contract'
import { useCommunityResource } from '@/components/community/resource'
import RequestState from '@/components/community/RequestState.vue'
import { useHead } from '@/lib/head'
import { loadFeaturedIdeas } from '@/lib/ideaCatalog'
import { ideaPlayerId, ideaRunLocation } from '@/lib/ideaPlayers'
import '@/components/community/community.css'

const route = useRoute()
const { data, loading, error, load } = useCommunityResource(loadFeaturedIdeas)
void load()
const work = computed(() => data.value?.find(item => item.kind === 'work' && item.id === route.params.slug))
const site = computed(() => webLink(work.value?.site))
const repository = computed(() => webLink(work.value?.repository))
const run = computed(() => {
  const id = ideaPlayerId(work.value?.player)
  return id ? ideaRunLocation(id) : undefined
})
useHead(() => ({ title: work.value?.title ?? '创意作品' }))
</script>

<template>
  <section class="community-page idea-work">
    <RouterLink :to="{ name: 'ideas' }" class="back"><AppIcon name="back" :size="16" /> 创意</RouterLink>
    <RequestState v-if="loading || error" :loading="loading" :error="error" @retry="load" />
    <article v-else-if="work" class="work">
      <img :src="webLink(work.cover)" alt="" width="1120" height="630" class="cover">
      <div class="copy">
        <h1>{{ work.title }}</h1>
        <p>{{ work.description }}</p>
        <div class="meta"><span>{{ work.author }}</span><time :datetime="work.date">收录 {{ displayDate(work.date) }}</time></div>
        <div class="actions">
          <RouterLink v-if="run" :to="run" class="btn btn-primary">在小伴中打开</RouterLink>
          <a v-if="site" :href="site" class="btn" :class="{ 'btn-primary': !run }">前往官网</a>
          <a v-if="repository" :href="repository" class="btn">查看源码</a>
        </div>
      </div>
    </article>
    <p v-else role="status">找不到这个作品。</p>
  </section>
</template>

<style scoped>
.idea-work { max-width: 1000px; padding: 0 8px 20px; }
.back { display: inline-flex; align-items: center; gap: 6px; min-height: 44px; color: var(--ink-2); font-size: 14px; }
.work { overflow: hidden; border: 1px solid var(--line); border-radius: 14px; background: var(--surface); }
.cover { display: block; width: 100%; height: auto; aspect-ratio: 16 / 9; object-fit: cover; background: var(--surface-2); }
.copy { padding: 16px; }
h1 { margin: 0; font-size: 22px; line-height: 1.35; }
.copy p { margin: 9px 0 14px; color: var(--ink-2); line-height: 1.6; }
.meta { display: flex; flex-wrap: wrap; gap: 6px 14px; margin-bottom: 16px; color: var(--ink-3); font-size: 12px; }
.actions { display: flex; flex-wrap: wrap; gap: 8px; }
@container page (min-width: 690px) {
  .work { display: grid; grid-template-columns: minmax(0, 1.6fr) minmax(0, 1fr); align-items: center; }
  .copy { padding: 22px; }
}
</style>
