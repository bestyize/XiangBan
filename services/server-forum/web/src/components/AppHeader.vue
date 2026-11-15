<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useMeta } from '@/stores/meta'
import { useSession } from '@/stores/session'
import { useNotifications } from '@/stores/notifications'
import { areaOf, communitySections, isCommunityRoot, sectionOf, type Section } from '@/lib/sections'
import { useTheme, THEME_LABEL } from '@/lib/theme'
import BrandMark from './BrandMark.vue'
import AppIcon from './AppIcon.vue'
import CommunityTabs from './community/CommunityTabs.vue'
const route = useRoute(), router = useRouter(), session = useSession(), inbox = useNotifications()
const { preference, resolved, openAppearance } = useTheme()
const titles = { community: '社区', ai: '小伴 AI', create: '应用', me: '我的' }
const title = computed(() => titles[areaOf(route)])
const communityRoot = computed(() => isCommunityRoot(route))
const meta = useMeta()
const section = computed(() => sectionOf(route,meta.categories))
const destinations = ref(Object.fromEntries(communitySections.map(item => [item.key, item.path])) as Record<Section, string>)
const communityLinks = computed(() => communitySections.map(({ key, label }) => ({ key, label, to: destinations.value[key] })))
watch(() => route.fullPath, path => { if (communityRoot.value) destinations.value[section.value] = path }, {immediate:true})
const detail = computed(() => !communityRoot.value)
const searchOpen = ref(false), query = ref(''), searchInput = ref<HTMLInputElement>()
async function search() { searchOpen.value = !searchOpen.value; if (searchOpen.value) { await nextTick(); searchInput.value?.focus() } }
function submit() { if (query.value.trim()) router.push({ name: 'search', query: { q: query.value.trim() } }) }
function back() { if (window.history.state?.back) router.back(); else router.push(areaOf(route) === 'create' ? '/create' : areaOf(route) === 'me' ? '/me' : '/') }
watch(() => route.fullPath, () => { searchOpen.value = false })
</script>
<template>
  <header class="site-header" :class="{'community-header':communityRoot,'detail-header':detail}">
    <div class="header-inner">
      <button v-if="detail" class="header-icon" aria-label="返回上一页" @click="back"><AppIcon name="back" /></button>
      <RouterLink v-else class="header-brand" to="/" aria-label="超级小伴首页"><BrandMark /><span>超级小伴</span></RouterLink>
      <CommunityTabs v-if="communityRoot" class="community-navigation" :items="communityLinks" :active="section" label="社区栏目" primary />
      <span v-else class="header-context">{{ title }}</span>
      <div class="header-actions">
        <button class="header-icon" :aria-expanded="searchOpen" aria-label="搜索社区" @click="search"><AppIcon name="search" /><span class="desktop-search-label" aria-hidden="true">搜索社区内容</span></button>
        <button class="header-icon theme-switch" :title="`外观：${THEME_LABEL[preference]}`" :aria-label="`外观与配色，当前${THEME_LABEL[preference]}`" @click="openAppearance"><AppIcon :name="resolved === 'dark' ? 'moon' : 'sun'" :size="20" /></button>
        <RouterLink v-if="session.loggedIn" class="header-icon notification" to="/notifications" :aria-label="inbox.unread ? `通知，${inbox.unread}条未读` : '通知'"><AppIcon name="bell" /><i v-if="inbox.unread" /></RouterLink>
        <button v-else class="header-login" @click="session.loginOpen = true">登录</button>
      </div>
    </div>
    <form v-if="searchOpen" class="header-search" role="search" @submit.prevent="submit">
      <label class="sr-only" for="site-search">搜索帖子、作品和内容</label>
      <input id="site-search" ref="searchInput" v-model="query" type="search" placeholder="搜索社区内容" maxlength="100" enterkeyhint="search" @keydown.esc="searchOpen = false" />
      <button class="btn btn-primary" type="submit">搜索</button>
    </form>
  </header>
</template>
<style scoped>
.site-header{position:sticky;top:0;z-index:30;background:var(--surface)}
.detail-header{background:color-mix(in srgb,var(--paper) 94%,transparent);backdrop-filter:blur(16px)}
.header-inner{height:var(--header-h);max-width:var(--page-w);margin:auto;display:flex;align-items:center;gap:12px;padding:0 24px}
.header-brand{display:flex;align-items:center;gap:8px;font-size:17px;font-weight:750;letter-spacing:.02em}.header-brand :deep(svg){width:28px;height:28px}
.detail-header .header-inner { height: 64px; }
.community-header .header-inner { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); }
.community-header .header-brand { justify-self: start; }
.community-navigation { grid-column: 2; grid-row: 1; }
.header-context{font-size:16px;font-weight:650;color:var(--ink-2)}.header-brand+.header-context{display:none}
.header-actions{margin-left:auto;display:flex;align-items:center;gap:4px}
.community-header .header-actions { grid-column: 3; grid-row: 1; justify-self: end; margin-left: 0; }
.header-icon{border:0;background:none;width:44px;height:44px;display:grid;place-items:center;border-radius:50%;color:var(--ink-2)}.header-icon:hover{background:var(--surface)}
.header-login{background:var(--surface);border:0;border-radius:22px;padding:0 18px;height:40px;font-size:14px;font-weight:600}
.desktop-search-label{display:none}.notification{position:relative}.notification i{position:absolute;right:10px;top:9px;width:6px;height:6px;border-radius:50%;background:var(--brand)}
.header-search{display:flex;gap:8px;padding:0 16px 12px;max-width:760px;margin:auto}.header-search input{min-width:0;flex:1;border:0;border-radius:12px;background:var(--surface);height:44px;padding:0 14px;font-size:16px}
@media(min-width:1024px){.header-brand{display:none}.header-brand+.header-context{display:block}}
@media(max-width:767px){.header-inner{padding:0 12px}.header-brand{font-size:16px}.header-actions{gap:0}.header-icon{width:40px}.header-login{padding:0 14px}.theme-switch{display:none}}
</style>

<style scoped>
@media(max-width:767px){
  .detail-header{background:var(--surface);backdrop-filter:none}
  .community-header .header-inner{grid-template-columns:44px minmax(0,1fr) 44px;gap:0}
  .community-header .header-brand{width:44px}
  .community-header .header-brand span{display:none}
  .community-header .header-brand :deep(svg){width:26px;height:26px}
  .community-header .header-login,.community-header .notification{display:none}
  .detail-header .header-inner{height:48px}
  .detail-header .header-login{display:none}
}
</style>

<style scoped>
@media(min-width:1100px){
.detail-header{background:color-mix(in srgb,var(--surface) 90%,transparent);border-bottom:1px solid var(--line)}.header-inner{padding:0 32px}.header-context{font-size:14px;font-weight:500}.header-icon{width:38px;height:38px}.header-login{height:36px;font-size:13px;border:1px solid var(--line)}.header-actions{gap:8px}.header-search{padding-top:8px}
}
@media(min-width:1600px){.header-inner{padding-left:40px;padding-right:40px}}
</style>
