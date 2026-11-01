<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSession } from '@/stores/session'
import { useNotifications } from '@/stores/notifications'
import { areaOf, sectionOf } from '@/lib/sections'
import { useTheme, THEME_LABEL } from '@/lib/theme'
import BrandMark from './BrandMark.vue'
import AppIcon from './AppIcon.vue'
const route = useRoute(), router = useRouter(), session = useSession(), inbox = useNotifications()
const { preference, resolved, openAppearance } = useTheme()
const titles = { community: '社区', ai: '小伴 AI', create: '创造', me: '我的' }
const title = computed(() => titles[areaOf(route)])
const roots = ['home','discuss','resources','ai','create','me']
const communityRoot = computed(() => ['home','discuss','resources'].includes(String(route.name)))
const section = computed(() => sectionOf(route))
const destinations = ref({ home:'/', discuss:'/discuss', resources:'/resources' })
watch(() => route.fullPath, path => { if (communityRoot.value) destinations.value[section.value] = path }, {immediate:true})
const detail = computed(() => !roots.includes(String(route.name)))
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
      <nav v-if="communityRoot" class="community-navigation" aria-label="社区栏目"><RouterLink v-for="item in [{key:'home' as const,label:'首页'},{key:'discuss' as const,label:'讨论'},{key:'resources' as const,label:'资源'}]" :key="item.key" :to="destinations[item.key]" :class="{on:section===item.key}" :aria-current="section===item.key?'page':undefined">{{ item.label }}</RouterLink></nav>
      <span v-else class="header-context">{{ title }}</span>
      <div class="header-actions">
        <button class="header-icon" :aria-expanded="searchOpen" aria-label="搜索社区" @click="search"><AppIcon name="search" /></button>
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
.site-header{position:sticky;top:0;z-index:30;background:color-mix(in srgb,var(--paper) 94%,transparent);backdrop-filter:blur(16px)}
.header-inner{height:64px;max-width:var(--page-w);margin:auto;display:flex;align-items:center;gap:12px;padding:0 24px}
.header-brand{display:flex;align-items:center;gap:8px;font-size:17px;font-weight:750;letter-spacing:.02em}.header-brand :deep(svg){width:28px;height:28px}
.community-navigation{display:flex;justify-content:center;align-items:stretch;gap:28px;min-width:0;flex:1;height:48px}.community-navigation a{display:flex;align-items:center;position:relative;font-size:16px;color:var(--ink-2);white-space:nowrap}.community-navigation a.on{color:var(--brand);font-weight:650}.community-navigation a.on:after{content:"";position:absolute;bottom:4px;left:50%;transform:translateX(-50%);height:3px;width:18px;border-radius:2px;background:var(--brand)}
.header-context{font-size:16px;font-weight:650;color:var(--ink-2)}.header-brand+.header-context{display:none}
.header-actions{margin-left:auto;display:flex;align-items:center;gap:4px}
.header-icon{border:0;background:none;width:44px;height:44px;display:grid;place-items:center;border-radius:50%;color:var(--ink-2)}.header-icon:hover{background:var(--surface)}
.header-login{background:var(--surface);border:0;border-radius:22px;padding:0 18px;height:40px;font-size:14px;font-weight:600}
.notification{position:relative}.notification i{position:absolute;right:10px;top:9px;width:6px;height:6px;border-radius:50%;background:var(--brand)}
.header-search{display:flex;gap:8px;padding:0 16px 12px;max-width:760px;margin:auto}.header-search input{min-width:0;flex:1;border:0;border-radius:12px;background:var(--surface);height:44px;padding:0 14px;font-size:16px}
@media(min-width:1024px){.header-brand{display:none}.header-brand+.header-context{display:block}}
@media(max-width:767px){.header-inner{height:56px;padding:0 12px}.header-brand{font-size:16px}.header-actions{gap:0}.header-icon{width:40px}.header-login{padding:0 14px}.theme-switch{display:none}}
</style>

<style scoped>
@media(max-width:767px){
  .community-header,.detail-header{background:var(--surface);backdrop-filter:none}
  .community-header .header-inner{height:48px;gap:0;padding:0 12px}
  .community-header .header-brand{width:44px;justify-content:flex-start;flex:none}
  .community-header .header-brand span{display:none}
  .community-header .header-brand :deep(svg){width:26px;height:26px}
  .community-header .header-actions{width:44px;justify-content:flex-end;flex:none;margin-left:0}
  .community-header .header-login,.community-header .notification{display:none}
  .community-navigation{gap:28px;height:48px}
  .detail-header .header-inner{height:48px}
  .detail-header .header-login{display:none}
}
</style>
