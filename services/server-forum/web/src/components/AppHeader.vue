<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSession } from '@/stores/session'
import UserAvatar from './UserAvatar.vue'
import BrandMark from './BrandMark.vue'
import { useTheme, THEME_LABEL } from '@/lib/theme'

const session = useSession()
const { preference, resolved, cycleTheme } = useTheme()
const route = useRoute()
const router = useRouter()

const q = ref(typeof route.query.q === 'string' ? route.query.q : '')
const searchOpen = ref(false)
const menuOpen = ref(false)

watch(
  () => route.fullPath,
  () => {
    menuOpen.value = false
    searchOpen.value = false
    if (route.name !== 'search') q.value = ''
  },
)

function submit() {
  const text = q.value.trim()
  if (!text) return
  router.push({ name: 'search', query: { q: text } })
}

function compose() {
  if (!session.requireLogin()) return
  router.push({ name: 'new' })
}

async function logout() {
  menuOpen.value = false
  await session.logout()
}
</script>

<template>
  <header class="header">
    <div class="inner">
      <RouterLink to="/" class="brand" aria-label="超级小伴首页">
        <BrandMark />
        <span class="wordmark">超级小伴</span>
      </RouterLink>

      <nav class="nav" aria-label="主导航">
        <RouterLink to="/" :class="{ on: route.name === 'home' }">首页</RouterLink>
        <RouterLink to="/resources" :class="{ on: route.name === 'resources' || route.name === 'pack' }">资源</RouterLink>
        <RouterLink to="/weekly" :class="{ on: route.name === 'weekly' || route.name === 'weekly-issue' }">周刊</RouterLink>
        <RouterLink to="/projects" :class="{ on: route.name === 'projects' || route.name === 'project' }">项目</RouterLink>
      </nav>

      <form class="search" :class="{ open: searchOpen }" role="search" @submit.prevent="submit">
        <label class="sr-only" for="site-search">搜索</label>
        <svg viewBox="0 0 20 20" aria-hidden="true"><circle cx="9" cy="9" r="6" /><path d="M13.5 13.5 17 17" /></svg>
        <input id="site-search" v-model="q" type="search" placeholder="搜索帖子" maxlength="50" enterkeyhint="search" />
      </form>

      <div class="actions">
        <button class="theme-toggle" type="button" aria-label="切换日间或夜间模式" title="点击切换：自动 → 日间 → 夜间 → 自动" @click="cycleTheme">
          <svg v-if="resolved === 'dark'" viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="3.5" /><path d="M12 2v2m0 16v2M4.93 4.93l1.42 1.42m11.3 11.3 1.42 1.42M2 12h2m16 0h2M4.93 19.07l1.42-1.42m11.3-11.3 1.42-1.42" /></svg>
          <svg v-else viewBox="0 0 24 24" aria-hidden="true"><path d="M20.3 15.4A8.4 8.4 0 0 1 8.6 3.7 8.5 8.5 0 1 0 20.3 15.4Z" /></svg>
          <span class="theme-label">外观 · {{ THEME_LABEL[preference] }}</span>
        </button>
        <button class="icon-btn only-mobile" type="button" aria-label="搜索" @click="searchOpen = !searchOpen">
          <svg viewBox="0 0 20 20" aria-hidden="true"><circle cx="9" cy="9" r="6" /><path d="M13.5 13.5 17 17" /></svg>
        </button>
        <button class="btn btn-primary compose only-desktop" type="button" @click="compose">发帖</button>

        <template v-if="session.loggedIn">
          <RouterLink to="/points" class="points-entry" aria-label="积分与签到">积分 {{ session.user?.points ?? '…' }}</RouterLink>
          <div class="me">
            <button class="me-btn" type="button" :aria-expanded="menuOpen" aria-haspopup="menu" @click="menuOpen = !menuOpen">
              <UserAvatar :name="session.user?.nickname ?? '我'" :src="session.user?.avatar_url" :size="32" />
              <span class="sr-only">我的菜单</span>
            </button>
            <div v-if="menuOpen" class="menu panel" role="menu" @keydown.esc="menuOpen = false">
              <div class="menu-head">
                <strong>{{ session.user?.nickname ?? '加载中' }}</strong>
              </div>
              <RouterLink v-if="session.user" role="menuitem" :to="`/u/${session.user.uid}`">我的主页</RouterLink>
              <RouterLink role="menuitem" to="/settings">收藏与设置</RouterLink>
              <RouterLink role="menuitem" to="/points">积分、签到与邀请</RouterLink>
              <button role="menuitem" type="button" @click="logout">退出登录</button>
            </div>
          </div>
        </template>
        <button v-else class="btn" type="button" @click="session.loginOpen = true">登录</button>
      </div>
    </div>
  </header>
  <div v-if="menuOpen" class="scrim" @click="menuOpen = false" />
</template>

<style scoped>
.points-entry { white-space: nowrap; color: var(--ink-2); font-size: var(--fs-xs); }
.points-entry:hover { color: var(--brand); }
.header {
  position: sticky;
  top: 0;
  z-index: 30;
  background: color-mix(in srgb, var(--surface) 94%, transparent);
  backdrop-filter: saturate(1.4) blur(10px);
  -webkit-backdrop-filter: saturate(1.4) blur(10px);
  border-bottom: 1px solid var(--line);
}
.inner {
  max-width: var(--page-w);
  margin: 0 auto;
  height: var(--header-h);
  padding: 0 20px;
  display: flex;
  align-items: center;
  gap: 22px;
}
.brand {
  display: flex;
  align-items: center;
  gap: 9px;
  flex: none;
}
.wordmark {
  font-size: 18px;
  font-weight: 800;
  letter-spacing: 0.04em;
}
.nav {
  display: flex;
  gap: 4px;
}
.nav a {
  padding: 6px 10px;
  border-radius: var(--r-sm);
  color: var(--ink-2);
  font-weight: 600;
  font-size: var(--fs-md);
}
.nav a:hover {
  color: var(--ink);
  background: var(--surface-2);
}
.nav a.on {
  color: var(--ink);
  box-shadow: inset 0 -2px 0 var(--brand);
  border-radius: 0;
}
.search {
  flex: 1;
  max-width: 380px;
  margin-left: auto;
  position: relative;
}
.search svg {
  position: absolute;
  left: 11px;
  top: 50%;
  width: 16px;
  height: 16px;
  transform: translateY(-50%);
  fill: none;
  stroke: var(--ink-3);
  stroke-width: 1.8;
  stroke-linecap: round;
}
.search input {
  width: 100%;
  height: 36px;
  border: 1px solid var(--line);
  background: var(--surface);
  border-radius: 18px;
  padding: 0 14px 0 34px;
  font-size: var(--fs-sm);
}
.search input:focus {
  outline: none;
  border-color: var(--focus);
}
.actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: none;
}
.icon-btn {
  width: 36px;
  height: 36px;
  display: grid;
  place-items: center;
  border: 0;
  background: transparent;
  border-radius: 50%;
}
.icon-btn svg {
  width: 19px;
  height: 19px;
  fill: none;
  stroke: var(--ink);
  stroke-width: 1.8;
  stroke-linecap: round;
}
.me {
  position: relative;
}
.me-btn {
  border: 0;
  padding: 0;
  background: none;
  border-radius: 50%;
  display: flex;
}
.menu {
  position: absolute;
  right: 0;
  top: calc(100% + 8px);
  min-width: 180px;
  padding: 6px;
  z-index: 40;
  box-shadow: var(--shadow);
  display: grid;
}
.menu-head {
  padding: 8px 10px 10px;
  border-bottom: 1px solid var(--line);
  margin-bottom: 4px;
  font-size: var(--fs-sm);
}
.menu a,
.menu button {
  text-align: left;
  padding: 8px 10px;
  border-radius: var(--r-sm);
  border: 0;
  background: none;
  font-size: var(--fs-sm);
}
.menu a:hover,
.menu button:hover {
  background: var(--surface-2);
}
.scrim {
  position: fixed;
  inset: 0;
  z-index: 25;
}
.only-mobile {
  display: none;
}
@media (max-width: 767px) {
  .inner {
    padding: 0 12px;
    gap: 10px;
    height: 52px;
  }
  .nav,
  .only-desktop {
    display: none;
  }
  .only-mobile {
    display: grid;
  }
  .search {
    display: none;
    position: absolute;
    left: 0;
    right: 0;
    top: 52px;
    max-width: none;
    padding: 8px 12px 10px;
    background: var(--surface);
    border-bottom: 1px solid var(--line);
  }
  .search.open {
    display: block;
  }
  .search svg {
    left: 23px;
  }
  .actions {
    margin-left: auto;
    gap: 6px;
  }
}
.theme-toggle { display: inline-flex; align-items: center; justify-content: center; gap: 5px; min-width: 38px; height: 36px; padding: 0 8px; border: 1px solid var(--line); border-radius: 999px; background: var(--surface); color: var(--ink-2); transition: background-color .15s, border-color .15s, color .15s; }
.theme-toggle:hover { border-color: var(--brand); color: var(--brand); background: var(--surface-2); }
.theme-toggle svg { width: 18px; height: 18px; flex: none; fill: none; stroke: currentColor; stroke-width: 1.7; stroke-linecap: round; stroke-linejoin: round; }
.theme-label { font-size: var(--fs-xs); font-weight: 650; }
@media (max-width: 767px) { .theme-toggle { width: 36px; min-width: 36px; padding: 0; } .theme-label { display: none; } }
</style>
