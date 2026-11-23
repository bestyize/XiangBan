<script setup lang="ts">
import { MOBILE_SITE_QUERY, useNarrow } from '@/lib/viewport'
import { RouterLink } from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import UserAvatar from '@/components/UserAvatar.vue'
import type { ShareDeletion, SharePayload } from '@/lib/detailShare'
import DetailShareDialog from './DetailShareDialog.vue'
import DetailGallery from './DetailGallery.vue'
import '@/styles/topic-detail.css'

defineProps<{
  ready: boolean
  title?: string
  author?: string
  authorImage?: string | null
  authorTo?: string
  images?: string[]
  fallbackCover?: string
  badge?: string
  share?: SharePayload | null
  shareDeletion?: ShareDeletion
}>()
defineEmits<{ back: [] }>()
const mobile = useNarrow(MOBILE_SITE_QUERY)
</script>

<template>
  <div class="detail-page" :class="mobile ? 'xhs-h5-detail' : 'xhs-pc-wrap'">
    <div v-if="!mobile || !ready" class="xhs-pc-backbar">
      <button class="btn-outline" type="button" style="height:32px;font-size:12.5px" @click="$emit('back')">
        <AppIcon name="back" :size="14" /> 返回列表
      </button>
    </div>
    <slot v-if="!ready" name="state" />
    <article v-else :class="{ 'xhs-pc-card': !mobile }">
      <header :class="{ 'h5-header': mobile }">
        <div :class="mobile ? 'h5-detail-head' : 'xhs-author-bar'">
          <button v-if="mobile" class="h5-icon-btn" type="button" aria-label="返回" @click="$emit('back')">
            <AppIcon name="back" :size="19" />
          </button>
          <component :is="authorTo ? RouterLink : 'div'" class="xhs-head-author" :to="authorTo">
            <UserAvatar v-if="author" :name="author.replace(/^@/, '')" :src="authorImage"
              :size="mobile ? 30 : 34" :initial-size="mobile ? 12 : 13" />
            <strong>{{ author }}</strong>
          </component>
          <div class="detail-author-actions">
            <slot name="actions" />
            <DetailShareDialog v-if="share" :payload="share" :deletion="shareDeletion" trigger-class="h5-icon-btn" />
          </div>
        </div>
      </header>
      <DetailGallery v-if="images?.length" :images="images" :title="title" :badge="badge"
        :fallback-cover="fallbackCover" :mobile="mobile" />
      <div :class="[mobile ? 'xhs-h5-body' : 'xhs-pc-body', { 'has-dots': (images?.length ?? 0) > 1 }]">
        <div v-if="!mobile && $slots.cta" class="xhs-pc-head">
          <slot name="title"><h1 v-if="title" class="xhs-title">{{ title }}</h1></slot>
          <slot name="cta" />
        </div>
        <template v-else>
          <slot name="title"><h1 v-if="title" class="xhs-title">{{ title }}</h1></slot>
          <slot name="cta" />
        </template>
        <slot />
      </div>
      <slot name="comments" />
    </article>
    <slot name="dialogs" />
  </div>
</template>

<style scoped>
.detail-page { font: 14px/1.5 var(--community-font); }
.detail-author-actions { display: flex; align-items: center; gap: 8px; }
.h5-detail-head .detail-author-actions { display: contents; }
.xhs-head-author :deep(.avatar) { border: 0; background: var(--brand-fill); color: var(--brand-ink); }
@media (width < 960px) {
  .xhs-pc-backbar { padding: 8px; }
  .h5-header {
    position: sticky;
    top: 0;
    z-index: 30;
    padding-top: env(safe-area-inset-top);
    border-bottom: 1px solid transparent;
    background: var(--bg-surface);
  }
}
</style>
