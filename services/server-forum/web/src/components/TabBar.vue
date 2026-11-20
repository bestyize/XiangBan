<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { discussionTab, discussionTabs, resourceTab, resourceTabs } from '@/lib/communityStructure'
import { homeTabOf, homeTabs } from '@/lib/homeNavigation'
import { postingLocation } from '@/lib/postingStructure'
import { areaOf, isCommunityRoot, type CommunityLink, type Section } from '@/lib/sections'
import { useTheme, THEME_LABEL } from '@/lib/theme'
import { useMeta } from '@/stores/meta'
import { useNotifications } from '@/stores/notifications'
import { usePackPublisher } from '@/stores/packPublisher'
import { useSession } from '@/stores/session'
import { useTopicComposer } from '@/stores/topicComposer'
import BrandMark from './BrandMark.vue'
import AppIcon from './AppIcon.vue'
import ModalDialog from './ModalDialog.vue'
import UserAvatar from './UserAvatar.vue'

const props = defineProps<{ section: Section; communityLinks: readonly CommunityLink[]; searchOpen: boolean; commentDock: boolean }>()
defineEmits<{ search: [] }>()
const meta = useMeta(), session = useSession(), inbox = useNotifications()
const composer = useTopicComposer(), publisher = usePackPublisher()
const route = useRoute(), { preference, resolved, openAppearance } = useTheme()
const publishing = ref(false)
const sectionCollapsed = ref(false)
const lastCommunity = ref('/')
const lastApps = ref('/create')
watch(() => route.fullPath, path => {
  if (isCommunityRoot(route)) lastCommunity.value = path
  if (route.name === 'app-works' || (route.name === 'create' && !route.query.game)) lastApps.value = path
}, { immediate: true })

const current = computed(() => areaOf(route))
const expandedSection = computed(() => current.value === 'community' && !sectionCollapsed.value ? props.section : null)
watch(() => route.name, () => { sectionCollapsed.value = false })
const communityIcons = { home: 'compass', discuss: 'message', resources: 'folder' }
const productLinks = computed(() => [
  { to: lastApps.value, key: 'create', label: '应用', icon: 'grid' },
  { to: '/ai', key: 'ai', label: '小伴 AI', icon: 'ai' },
  { to: '/me', key: 'me', label: '我的', icon: 'me' },
])
const findQuery = (key: string) => typeof route.query[key] === 'string' ? route.query[key] : ''
const rootPage = computed(() => isCommunityRoot(route))
const sectionTabs = computed(() => {
  if (props.section === 'home') return homeTabs
  if (props.section === 'discuss') return discussionTabs(meta.categories)
  return resourceTabs(meta.categories)
})
const activeTab = computed(() => {
  if (!rootPage.value) return ''
  if (props.section === 'home') return homeTabOf(route.query.tab)
  if (props.section === 'discuss') return discussionTab(meta.categories, findQuery('tab'), findQuery('filter'), findQuery('board'))
  return resourceTab(meta.categories, findQuery('tab'))
})
function tabLocation(key: string) {
  const tab = ['ideas', 'all', 'software'].includes(key) ? undefined : key
  const isFollowing = route.name === 'discuss' && [route.query.mode, route.query.sort, route.query.tab].includes('following')
  return { name: props.section, query: { tab, mode: isFollowing ? 'following' : undefined } }
}

function toggleCurrentSection(key: Section, event: MouseEvent) {
  if (!rootPage.value || key !== props.section || event.defaultPrevented || event.button !== 0 ||
    event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return
  event.preventDefault()
  sectionCollapsed.value = !sectionCollapsed.value
}

const searchKey = /mac|iphone|ipad/i.test(navigator.platform) ? '⌘K' : 'Ctrl K'
const notificationsLabel = computed(() => inbox.unread ? `通知，${inbox.unread}条未读` : '通知')

function publishTopic() {
  publishing.value = false
  composer.open(postingLocation(route.name, route.query, meta.categories, route.params.slug))
}
function publishWork() {
  publishing.value = false
  publisher.open()
}
</script>

<template>
  <nav class="desktop-rail" aria-label="主导航">
    <RouterLink class="rail-brand" to="/">
      <BrandMark /> <strong>超级小伴</strong> <span>社区</span>
    </RouterLink>
    <button class="rail-search" type="button" aria-label="搜索社区" aria-keyshortcuts="Meta+K Control+K"
      aria-haspopup="dialog" :aria-expanded="searchOpen" @click="$emit('search')">
      <AppIcon name="search" :size="16" /> <span>搜索</span> <kbd>{{ searchKey }}</kbd>
    </button>
    <button class="rail-publish" type="button" aria-haspopup="dialog" :aria-expanded="publishing" @click="publishing = true">
      <AppIcon name="plus" :size="17" /> 发布灵感
    </button>

    <div class="rail-scroll">
      <div class="rail-group">
        <div class="rail-label">社区</div>
        <template v-for="item in communityLinks" :key="item.key">
          <RouterLink :to="item.to" class="rail-link" :class="{ active: current === 'community' && item.key === section }"
            :aria-current="current === 'community' && item.key === section ? 'location' : undefined"
            :aria-expanded="rootPage && item.key === section ? expandedSection === item.key : undefined"
            :aria-controls="rootPage && item.key === section ? `rail-${item.key}-subtabs` : undefined"
            @click.capture="toggleCurrentSection(item.key, $event)">
            <span class="rail-icon"><AppIcon :name="communityIcons[item.key]" :size="16" /></span>
            {{ item.label }} <AppIcon class="rail-chevron" :class="{ expanded: expandedSection === item.key }" name="arrow" :size="13" />
          </RouterLink>
          <div v-if="current === 'community' && item.key === section" v-show="expandedSection === item.key"
            :id="`rail-${item.key}-subtabs`" class="rail-subtabs">
            <RouterLink v-for="tab in sectionTabs" :key="tab.key" :to="tabLocation(tab.key)" :replace="rootPage"
              :title="tab.label" :aria-current="activeTab === tab.key ? 'page' : undefined">{{ tab.label }}</RouterLink>
          </div>
        </template>
      </div>
    </div>
    <div class="rail-group rail-products">
      <div class="rail-label">产品空间</div>
      <RouterLink v-for="item in productLinks" :key="item.key" :to="item.to" class="rail-link"
        :class="{ active: current === item.key }" :aria-current="current === item.key ? 'location' : undefined">
        <span class="rail-icon"><AppIcon :name="item.icon" :size="16" /></span> {{ item.label }}
      </RouterLink>
    </div>

    <div class="rail-bottom">
      <div class="rail-account">
        <template v-if="session.user">
          <RouterLink to="/me" aria-label="我的账号"><UserAvatar :name="session.user.nickname" :src="session.user.avatar_url" :size="30" /></RouterLink>
          <div class="rail-account-copy">
            <RouterLink to="/me" class="rail-name">{{ session.user.nickname }}</RouterLink>
            <RouterLink to="/points" class="rail-points">{{ session.user.points }} 积分</RouterLink>
          </div>
        </template>
        <RouterLink v-else-if="session.loggedIn" to="/me" class="rail-sign-in"><AppIcon name="me" :size="16" /> 我的账号</RouterLink>
        <button v-else class="rail-sign-in" type="button" @click="session.requireLogin()"><AppIcon name="me" :size="16" /> 登录</button>
        <RouterLink class="rail-bell" to="/notifications" :aria-label="notificationsLabel">
          <AppIcon name="bell" :size="17" /><i v-if="inbox.unread" />
        </RouterLink>
      </div>
      <div class="rail-utils">
        <button type="button" :aria-label="`外观与配色，当前${THEME_LABEL[preference]}`" @click="openAppearance">
          <AppIcon :name="resolved === 'dark' ? 'sun' : 'moon'" :size="15" /> 外观
        </button>
        <RouterLink to="/ai" aria-label="下载客户端"><AppIcon name="download" :size="15" /> 客户端</RouterLink>
      </div>
    </div>
  </nav>

  <nav class="mobile-tabbar" :class="{ 'with-comment-dock': commentDock }" aria-label="主导航">
    <RouterLink to="/ai" class="mobile-link" :aria-current="current === 'ai' ? 'page' : undefined"><AppIcon name="ai" :size="19" />小伴AI</RouterLink>
    <RouterLink :to="lastApps" class="mobile-link" :aria-current="current === 'create' ? 'page' : undefined"><AppIcon name="grid" :size="19" />应用</RouterLink>
    <button class="mobile-publish" type="button" aria-label="发布" aria-haspopup="dialog" :aria-expanded="publishing" @click="publishing = true">
      <span><AppIcon name="plus" :size="19" /></span>
    </button>
    <RouterLink :to="lastCommunity" class="mobile-link" :aria-current="current === 'community' ? 'page' : undefined"><AppIcon name="compass" :size="19" />社区</RouterLink>
    <RouterLink to="/me" class="mobile-link" :aria-current="current === 'me' ? 'page' : undefined"><AppIcon name="me" :size="19" />我的</RouterLink>
  </nav>

  <ModalDialog :open="publishing" title="分享一点新鲜事" @close="publishing = false">
    <div class="publish-options">
      <button type="button" @click="publishTopic"><span class="option-icon"><AppIcon name="community" /></span><span><b>发布帖子</b><small>交流想法、分享经验或发起求助</small></span><AppIcon name="arrow" :size="18" /></button>
      <button type="button" @click="publishWork"><span class="option-icon"><AppIcon name="upload" /></span><span><b>分享已有作品</b><small>上传单个 HTML 或作品 ZIP</small></span><AppIcon name="arrow" :size="18" /></button>
    </div>
  </ModalDialog>
</template>

<style scoped>
.desktop-rail { display: none; }
.mobile-tabbar {
  position: fixed;
  z-index: 35;
  inset: auto 0 0;
  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
  align-items: center;
  height: var(--mobile-nav-offset);
  padding: 0 4px env(safe-area-inset-bottom);
  border-top: 1px solid var(--community-divider);
  background: var(--mobile-nav-surface);
  font-family: var(--community-font);
}
.mobile-link, .mobile-publish { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 2px; height: 48px; }
.mobile-link { color: var(--community-muted); font-size: 10.5px; line-height: 1.5; white-space: nowrap; }
.mobile-link[aria-current] { color: var(--brand); font-weight: 650; }
.mobile-publish { justify-self: center; min-width: 46px; padding: 0; border: 0; background: none; }
.mobile-publish span, .rail-publish { background: linear-gradient(135deg, var(--brand-fill), var(--brand-hover)); color: var(--brand-ink); box-shadow: 0 8px 18px -6px color-mix(in srgb, var(--brand-fill) 55%, transparent); }
.mobile-publish span { display: grid; place-items: center; width: 46px; height: 34px; border-radius: 14px; box-shadow: 0 6px 14px -4px color-mix(in srgb, var(--brand-fill) 55%, transparent); }
.mobile-publish:hover span, .rail-publish:hover { background: var(--brand-fill); }
.publish-options{display:grid;gap:8px}
.publish-options>button{display:flex;align-items:center;gap:12px;width:100%;text-align:left;padding:16px 12px;border:0;border-radius:12px;background:var(--surface-2)}
.publish-options>button:hover{background:var(--brand-soft)}
.publish-options>button>span:nth-child(2){flex:1}
.publish-options b{display:block;font-size:15px;font-weight:600}
.publish-options small{display:block;font-size:12px;color:var(--ink-3);margin-top:3px}
.option-icon{color:var(--brand)}

@media (max-width: 767px) {
  .mobile-tabbar.with-comment-dock { display: none; }
}

@media (min-width: 1024px) {
  .mobile-tabbar { display: none; }
  .desktop-rail {
    position: fixed;
    inset: 0 auto 0 0;
    z-index: 35;
    display: flex;
    flex-direction: column;
    gap: 12px;
    width: var(--desktop-rail-width);
    padding: 16px 14px 14px;
    border-right: 1px solid var(--line);
    background: var(--surface-glass);
    backdrop-filter: blur(18px) saturate(160%);
  }
  .rail-brand { display: flex; align-items: center; gap: 10px; padding: 4px 8px; white-space: nowrap; }
  .rail-brand strong { font-size: 16px; }
  .rail-brand :deep(svg) { width: 28px; height: 28px; flex: none; }
  .rail-brand span { margin-left: auto; padding: 2px 7px; border-radius: 999px; color: var(--brand); background: var(--brand-soft); font-size: 11px; }
  .rail-search, .rail-publish { flex: none; display: flex; align-items: center; gap: 8px; border-radius: 999px; }
  .rail-search { min-height: 36px; padding: 0 10px; border: 1px solid var(--line); background: var(--surface-2); color: var(--ink-2); font-size: 13px; }
  .rail-search kbd { margin-left: auto; padding: 0 5px; border-radius: 4px; background: var(--surface); color: var(--ink-3); font-family: inherit; font-size: 11px; }
  .rail-search:hover { border-color: var(--line-strong); }
  .rail-publish { justify-content: center; min-height: 40px; padding: 0 12px; border: 0; font-weight: 650; }
  .rail-scroll { flex: 1; min-height: 0; overflow: auto; overscroll-behavior-y: contain; padding: 2px; margin-inline: -2px; scrollbar-width: thin; }
  .rail-group { display: grid; gap: 2px; }
  .rail-products { flex: none; padding-top: 10px; border-top: 1px solid var(--line); }
  .rail-label { padding: 4px 10px; color: var(--ink-3); font-size: 11px; font-weight: 600; }
  .rail-link { display: flex; align-items: center; gap: 10px; min-height: 40px; padding: 0 10px; border-radius: 12px; color: var(--ink-2); font-size: 14px; }
  .rail-link:hover, .rail-subtabs a:hover { color: var(--ink); background: var(--surface-2); }
  .rail-link.active { color: var(--brand); background: var(--brand-soft); font-weight: 650; }
  .rail-icon { flex: none; display: grid; place-items: center; width: 25px; height: 25px; border-radius: 7px; background: var(--surface-2); }
  .active .rail-icon { background: color-mix(in srgb, var(--brand-fill) 14%, transparent); }
  .rail-chevron { margin-left: auto; }
  .rail-chevron.expanded { transform: rotate(90deg); }
  .rail-subtabs { display: grid; gap: 2px; padding-inline-start: 26px; margin: 2px 0 6px; }
  .rail-subtabs a { align-content: center; min-width: 0; min-height: 28px; overflow: hidden; padding: 0 10px; border-radius: 7px; color: var(--ink-3); font-size: 13px; text-overflow: ellipsis; white-space: nowrap; }
  .rail-subtabs a[aria-current] { color: var(--ink); font-weight: 700; }
  .rail-bottom { flex: none; display: grid; gap: 7px; padding-top: 10px; border-top: 1px solid var(--line); }
  .rail-account { display: flex; align-items: center; gap: 8px; }
  .rail-account > a:first-child { display: flex; }
  .rail-account-copy { flex: 1; min-width: 0; }
  .rail-name { display: block; overflow: hidden; font-weight: 650; text-overflow: ellipsis; white-space: nowrap; }
  .rail-points { display: block; color: var(--brand); font-size: 12px; }
  .rail-sign-in, .rail-bell, .rail-utils > * { display: flex; align-items: center; justify-content: center; gap: 6px; min-height: 34px; padding: 0 8px; border: 0; border-radius: 8px; background: none; color: var(--ink-2); font-size: 12px; }
  .rail-sign-in { flex: 1; justify-content: flex-start; font-size: 14px; }
  .rail-bell { position: relative; min-width: 34px; background: var(--surface-2); }
  .rail-bell i { position: absolute; top: 7px; right: 7px; width: 5px; height: 5px; border-radius: 50%; background: var(--brand-fill); }
  .rail-utils { display: grid; grid-template-columns: 1fr 1fr; gap: 4px; }
  .rail-sign-in:hover, .rail-utils > :hover { background: var(--surface-2); color: var(--ink); }
  .rail-bell:hover, .rail-name:hover, .rail-points:hover { color: var(--brand); }
}
@media (min-width: 1024px) and (max-height: 760px) {
  .desktop-rail { gap: 10px; }
  .rail-link { min-height: 36px; }
  .rail-subtabs a { min-height: 26px; }
}
</style>
