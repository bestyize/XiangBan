<script setup lang="ts">
import { computed, ref } from 'vue'
import type { IdeaCardItem } from '@/lib/ideaCards'
import { displayDate, webLink } from './contract'

const props = defineProps<{ card: IdeaCardItem }>()
const failedCover = ref<string>()
const cover = computed(() => {
  const url = webLink(props.card.cover)
  return url === failedCover.value ? undefined : url
})
</script>

<template>
  <RouterLink :to="card.to" class="idea-card">
    <div class="cover">
      <img v-if="cover" :src="cover" alt="" loading="lazy" decoding="async" @error="failedCover = cover">
      <span v-if="card.label" class="label">{{ card.label }}</span>
    </div>
    <div class="copy">
      <h2>{{ card.title }}</h2>
      <p v-if="card.description" class="description">{{ card.description }}</p>
      <footer v-if="card.author || card.date" class="meta">
        <span v-if="card.author" class="author">{{ card.author }}</span>
        <time v-if="card.date" :datetime="card.date">{{ card.dateLabel ? card.dateLabel + ' ' : '' }}{{ displayDate(card.date) }}</time>
      </footer>
    </div>
  </RouterLink>
</template>

<style scoped>
.idea-card {
  display: flex;
  flex-direction: column;
  min-width: 0;
  padding: 7px;
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 13px;
  color: var(--ink);
  transition: border-color .18s, box-shadow .18s, transform .18s;
}
@media (hover: hover) {
  .idea-card:hover {
    border-color: var(--line-strong);
    box-shadow: var(--shadow);
    transform: translateY(-2px);
  }
}
.cover {
  position: relative;
  flex: none;
  aspect-ratio: 16 / 9;
  overflow: hidden;
  border-radius: 8px;
  background: var(--surface-2);
}
.cover img { display: block; width: 100%; height: 100%; object-fit: cover; }
.label {
  position: absolute;
  inset: 8px 8px auto auto;
  max-width: calc(100% - 16px);
  overflow: hidden;
  padding: 3px 6px;
  border: 1px solid #ffffff36;
  border-radius: 5px;
  color: var(--brand-ink);
  background: var(--brand);
  font-size: 11px;
  font-weight: 600;
  line-height: 1.3;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.copy { display: flex; flex: 1; min-width: 0; flex-direction: column; padding: 12px 10px 4px; }
h2, .description {
  display: -webkit-box;
  overflow: hidden;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  overflow-wrap: anywhere;
}
h2 { margin: 0; font-size: 16px; font-weight: 650; line-height: 1.4; }
.description { margin: 5px 0 8px; color: var(--ink-2); font-size: 13px; line-height: 1.55; }
.meta {
  display: flex;
  justify-content: space-between;
  gap: 4px 8px;
  margin-top: auto;
  padding-top: 8px;
  border-top: 1px solid var(--line);
  color: var(--ink-3);
  font-size: 12px;
  line-height: 1.4;
}
h2 + .meta { margin-top: 8px; }
.author { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
time { white-space: nowrap; font-variant-numeric: tabular-nums; }
@container page (max-width: 530px) {
  .idea-card { padding: 5px; }
  .cover { border-radius: 6px; }
  .copy { padding: 9px 5px 4px; }
  h2 { font-size: 14px; }
  .description { font-size: 12px; }
  .meta { flex-direction: column; }
}
</style>
