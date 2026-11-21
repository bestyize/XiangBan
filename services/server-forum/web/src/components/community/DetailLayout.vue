<script setup lang="ts">
import { MOBILE_SITE_QUERY, useNarrow } from '@/lib/viewport'
import { RouterLink } from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import UserAvatar from '@/components/UserAvatar.vue'
import { useToast } from '@/stores/toast'
import DetailGallery from './DetailGallery.vue'
import '@/styles/topic-detail.css'

const props = defineProps<{
  ready: boolean
  title?: string
  author?: string
  authorImage?: string | null
  authorTo?: string
  images?: string[]
  fallbackCover?: string
  badge?: string
  shareable?: boolean
  shareUrl?: string
}>()
defineEmits<{ back: [] }>()
const mobile = useNarrow(MOBILE_SITE_QUERY)
const toast = useToast()

async function share() {
  const url = props.shareUrl || location.href
  try {
    if (navigator.share) await navigator.share({ title: props.title, url })
    else {
      await navigator.clipboard.writeText(url)
      toast.show('链接已复制')
    }
  } catch (error) {
    if (!(error instanceof Error && error.name === 'AbortError')) toast.show('暂时无法分享，请复制地址栏中的链接')
  }
}
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
            <button v-if="shareable" type="button" class="h5-icon-btn" aria-label="分享" @click="share">
              <AppIcon name="share-up" :size="mobile ? 18 : 16" />
            </button>
          </div>
        </div>
      </header>
      <DetailGallery v-if="images?.length" :images="images" :title="title" :badge="badge"
        :fallback-cover="fallbackCover" :mobile="mobile" />
      <div :class="[mobile ? 'xhs-h5-body' : 'xhs-pc-body', { 'has-dots': (images?.length ?? 0) > 1 }]">
        <slot name="title"><h1 v-if="title" class="xhs-title">{{ title }}</h1></slot>
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
