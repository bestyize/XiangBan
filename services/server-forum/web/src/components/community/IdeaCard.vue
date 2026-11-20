<script setup lang="ts">
import { computed, ref } from 'vue'
import AppIcon from '@/components/AppIcon.vue'
import type { IdeaCardItem } from '@/lib/ideaCards'
import { displayDate, webLink } from './contract'

const props = defineProps<{ card: IdeaCardItem; spotlight?: boolean }>()
const failedCovers = ref<string[]>([])
const cover = computed(() => [webLink(props.card.cover), webLink(props.card.fallbackCover)]
  .find(url => url && !failedCovers.value.includes(url)))
const shortDate = computed(() => props.card.date?.match(/^\d{4}-(\d{2}-\d{2})$/)?.[1]
  ?? displayDate(props.card.date).replace(/^\d{4}\//, '').replace('/', '-'))
const authorInitial = computed(() => Array.from(props.card.author?.replace(/^@/, '').trim() ?? '')[0]?.toUpperCase() ?? '?')
</script>

<template>
  <RouterLink :to="card.to" class="idea-card" :class="{ 'idea-spotlight': spotlight }">
    <div v-if="cover || spotlight" class="cover">
      <img v-if="cover" :key="cover" :src="cover" alt="" :loading="spotlight ? 'eager' : 'lazy'" :fetchpriority="spotlight ? 'high' : undefined"
        decoding="async" @error="cover && failedCovers.push(cover)">
      <span v-if="card.label && !spotlight" class="cover-label">{{ card.label }}</span>
    </div>
    <div v-if="spotlight" class="spotlight-overlay">
      <span class="spotlight-label"><span><span aria-hidden="true">✨</span> 本周主推<span v-if="card.label"> · {{ card.label }}</span></span></span>
      <div class="spotlight-copy">
        <h2>{{ card.title }}</h2>
        <p v-if="card.description">{{ card.description }}</p>
        <span class="spotlight-cta">立即体验 <AppIcon name="arrow" :size="13" /></span>
      </div>
    </div>
    <div v-else class="card-copy">
      <span v-if="card.label && !cover" class="text-label">{{ card.label }}</span>
      <h2 :title="card.title">{{ card.title }}</h2>
      <p v-if="!cover && card.description" class="description">{{ card.description }}</p>
      <footer v-if="card.author || card.date" class="card-meta">
        <span v-if="card.author" class="card-author">
          <span class="card-avatar" :style="{ backgroundColor: card.avatarColor }" aria-hidden="true">{{ authorInitial }}</span>
          <span>{{ card.author }}</span>
        </span>
        <time v-if="card.date" :datetime="card.date" :title="displayDate(card.date)">{{ shortDate }}</time>
      </footer>
    </div>
  </RouterLink>
</template>

<style scoped>
.idea-card {
  --idea-shadow: 0 6px 24px -8px rgb(15 23 42 / 7.5%), 0 1px 3px rgb(15 23 42 / 3%);
  --idea-hover-shadow: 0 16px 34px -10px rgb(15 23 42 / 14%);
  position: relative;
  display: flex;
  min-width: 0;
  flex-direction: column;
  overflow: hidden;
  border: 1px solid var(--community-divider);
  border-radius: 18px;
  background: var(--surface);
  color: var(--ink);
  font-family: var(--community-font);
  line-height: 1.5;
  box-shadow: var(--idea-shadow);
  transition: transform .2s, box-shadow .2s;
}
:global(html[data-theme="dark"]) .idea-card {
  --idea-shadow: 0 8px 26px -8px rgb(0 0 0 / 45%);
  --idea-hover-shadow: 0 16px 36px -8px rgb(0 0 0 / 65%);
}
.cover { position: relative; aspect-ratio: 3 / 2; overflow: hidden; background: var(--surface-2); }
.cover img { display: block; width: 100%; height: 100%; object-fit: cover; transition: transform .35s cubic-bezier(.2, .8, .2, 1); }
.cover-label { position: absolute; top: 9px; left: 9px; max-width: calc(100% - 18px); overflow: hidden; padding: 3px 9px; border-radius: 999px; background: rgb(17 24 39 / 56%); backdrop-filter: blur(10px); color: #fff; font-size: 10.5px; font-weight: 600; text-overflow: ellipsis; white-space: nowrap; }
.card-copy { display: flex; flex: 1; min-width: 0; flex-direction: column; gap: 8px; padding: 11px 13px 12px; }
.card-copy h2 { display: -webkit-box; overflow: hidden; margin: 0; font-size: 14px; font-weight: 650; line-height: 1.45; overflow-wrap: anywhere; -webkit-box-orient: vertical; -webkit-line-clamp: 2; }
.text-label { color: var(--brand); font-size: 11px; font-weight: 650; }
.description { display: -webkit-box; overflow: hidden; margin: 0; color: var(--ink-2); font-size: 13px; line-height: 1.5; overflow-wrap: anywhere; -webkit-box-orient: vertical; -webkit-line-clamp: 2; }
.card-meta { display: flex; align-items: center; justify-content: space-between; gap: 6px; margin-top: auto; }
.card-author { display: inline-flex; align-items: center; gap: 6px; min-width: 0; color: var(--ink-2); font-size: 11.5px; }
.card-author > span:last-child { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.card-avatar { display: inline-grid; place-items: center; flex: none; width: 20px; height: 20px; border-radius: 50%; background: var(--brand-fill); color: #fff; font-size: 10.5px; font-weight: 700; }
.card-meta time { flex: none; color: var(--community-muted); font-size: 11px; font-variant-numeric: tabular-nums; }
.idea-spotlight { border: 0; border-radius: 20px; background: #141821; color: #fff; }
.idea-spotlight .cover { height: 280px; aspect-ratio: auto; background: #141821; }
.idea-spotlight .cover img { transition-duration: .45s; }
.spotlight-overlay { position: absolute; inset: 0; display: flex; flex-direction: column; justify-content: space-between; padding: 16px 20px 18px; background: linear-gradient(180deg, rgb(10 14 22 / 8%) 0%, rgb(10 14 22 / 22%) 45%, rgb(10 14 22 / 86%) 100%); }
.spotlight-label { display: inline-flex; align-items: center; align-self: flex-start; gap: 5px; padding: 4px 11px; border: 1px solid rgb(255 255 255 / 26%); border-radius: 999px; background: rgb(255 255 255 / 20%); backdrop-filter: blur(12px); color: #fff; font-size: 11.5px; font-weight: 600; }
.spotlight-copy { min-width: 0; max-width: 100%; }
.spotlight-copy h2 { margin: 0 0 6px; font-size: 22px; font-weight: 700; line-height: 1.3; overflow-wrap: anywhere; }
.spotlight-copy p { max-width: 44ch; margin: 0 0 12px; color: rgb(255 255 255 / 84%); font-size: 13px; }
.spotlight-cta { display: inline-flex; align-items: center; gap: 5px; padding: 7px 14px; border-radius: 999px; background: #fff; color: #111827; font-size: 12.5px; font-weight: 700; }
@media (hover: hover) {
  .idea-card:not(.idea-spotlight):hover { transform: translateY(-3px); box-shadow: var(--idea-hover-shadow); }
  .idea-card:hover .cover img { transform: scale(1.04); }
  .idea-spotlight:hover .cover img { transform: scale(1.03); }
  .idea-card:hover .card-copy h2 { color: var(--brand); }
}
@media (max-width: 1023px) {
  .idea-card { border-radius: 12px; }
  .idea-spotlight { border-radius: 14px; }
  .idea-spotlight .cover { height: 184px; }
  .spotlight-copy h2 { margin-bottom: 4px; font-size: 17px; }
  .spotlight-copy p { display: -webkit-box; overflow: hidden; margin-bottom: 8px; font-size: 12px; -webkit-box-orient: vertical; -webkit-line-clamp: 1; }
  .spotlight-label { padding: 2px 7px; font-size: 9.5px; line-height: 1.35; }
  .spotlight-cta { padding: 5px 10px; font-size: 11px; }
  .cover-label { top: 6px; left: 6px; padding: 1.5px 6px; font-size: 9.5px; line-height: 1.35; }
  .card-copy { gap: 6px; padding: 9px 10px 10px; }
  .card-copy h2 { font-size: 13px; line-height: 1.4; }
}
</style>
