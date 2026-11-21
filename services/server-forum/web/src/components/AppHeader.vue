<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useNotifications } from '@/stores/notifications'
import { areaOf, isCommunityRoot, type CommunityLink, type Section } from '@/lib/sections'
import { useTheme, THEME_LABEL } from '@/lib/theme'
import BrandMark from './BrandMark.vue'
import AppIcon from './AppIcon.vue'
defineProps<{ section: Section; communityLinks: readonly CommunityLink[]; searchOpen: boolean }>()
defineEmits<{ search: [] }>()
const route = useRoute(), router = useRouter(), inbox = useNotifications()
const { preference, resolved, openAppearance } = useTheme()
const titles = { community: '社区', ai: '小伴 AI', create: '应用', me: '我的' }
const title = computed(() => titles[areaOf(route)])
const communityRoot = computed(() => isCommunityRoot(route))
const detail = computed(() => !communityRoot.value)
function back() { if (window.history.state?.back) router.back(); else router.push(areaOf(route) === 'create' ? '/create' : areaOf(route) === 'me' ? '/me' : '/') }
</script>
<template>
  <header class="site-header" :class="{'community-header':communityRoot,'detail-header':detail}">
    <div class="header-inner">
      <button v-if="detail" class="header-icon" type="button" aria-label="返回上一页" @click="back"><AppIcon name="back" :size="20" /></button>
      <RouterLink v-else class="header-brand" to="/" aria-label="超级小伴首页"><BrandMark /></RouterLink>
      <nav v-if="communityRoot" class="community-navigation" aria-label="社区栏目">
        <RouterLink v-for="item in communityLinks" :key="item.key" :to="item.to" class="primary-link"
          :class="{ 'is-active': section === item.key }" :aria-current="section === item.key ? 'page' : undefined">
          {{ item.label }}
        </RouterLink>
      </nav>
      <span v-else class="header-context">{{ title }}</span>
      <div class="header-actions">
        <button class="header-icon" type="button" :aria-expanded="searchOpen" aria-haspopup="dialog" aria-label="搜索社区" @click="$emit('search')"><AppIcon name="search" :size="communityRoot ? 17 : 18" /></button>
        <RouterLink class="header-icon notification" to="/notifications" :aria-label="inbox.unread ? `通知，${inbox.unread}条未读` : '通知'"><AppIcon name="bell" :size="communityRoot ? 17 : 18" /><i v-if="inbox.unread" /></RouterLink>
        <button v-if="detail" class="header-icon theme-switch" type="button" :title="`外观：${THEME_LABEL[preference]}`" :aria-label="`外观与配色，当前${THEME_LABEL[preference]}`" @click="openAppearance"><AppIcon :name="resolved === 'dark' ? 'moon' : 'sun'" :size="20" /></button>
      </div>
    </div>
  </header>
</template>
<style scoped>
.site-header {
  position: sticky;
  top: 0;
  z-index: 30;
  padding-top: env(safe-area-inset-top);
  background: var(--mobile-nav-surface);
}
.header-inner { display: flex; align-items: center; gap: 8px; height: var(--header-h); max-width: var(--page-w); margin: auto; padding: 0 8px; }
.community-header .header-inner {
  display: grid;
  grid-template-columns: 72px minmax(0, 1fr) 72px;
  gap: 8px;
  padding-inline: 12px;
}
.header-brand { display: inline-grid; place-items: center; width: 34px; height: 34px; justify-self: start; }
.header-brand :deep(svg) { width: 25px; height: 25px; }
.community-navigation { display: flex; justify-content: center; align-items: center; gap: 18px; }
.primary-link { flex: none; padding: 6px 2px; color: var(--community-muted); font-size: 15px; font-weight: 500; line-height: 1.5; white-space: nowrap; }
.primary-link.is-active { color: var(--ink); font-weight: 700; }
.header-context { font-size: 15px; font-weight: 650; }
.header-actions { display: flex; align-items: center; justify-content: flex-end; margin-left: auto; }
.community-header .header-actions { gap: 4px; }
.header-icon { position: relative; display: grid; place-items: center; width: 40px; height: 44px; padding: 0; border: 0; border-radius: 50%; background: none; color: var(--ink); }
.community-header .header-icon { width: 34px; height: 34px; }
.detail-header .header-icon:hover { color: var(--brand); }
.header-icon:active { background: var(--surface-2); }
.notification i { position: absolute; top: 9px; right: 9px; width: 6px; height: 6px; border-radius: 50%; background: var(--brand-fill); }
.community-header .notification i { top: 5px; right: 5px; box-shadow: 0 0 0 1.5px var(--surface); }
.detail-header { border-bottom: 1px solid var(--line); }
@media (max-width: 359px) {
  .community-navigation { gap: 10px; }
}
@media (width >= 960px) {
  .site-header { background: var(--surface-glass); backdrop-filter: blur(16px) saturate(150%); }
  .community-header, .header-actions { display: none; }
  .header-inner { padding-inline: 20px; }
}
</style>
