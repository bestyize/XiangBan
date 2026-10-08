<script setup lang="ts">
import '@/styles/admin.css'
import { onBeforeUnmount, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import Icon, { type IconName } from '@/components/Icon.vue'
import { setUnauthorizedHandler } from '@/admin/api'
import { useAdmin } from '@/stores/admin'

const route = useRoute()
const router = useRouter()
const admin = useAdmin()

const NAV: { to: string; label: string; icon: IconName }[] = [
  { to: '/admin/collector', label: '采集工作台', icon: 'download' },
  { to: '/admin/wallpapers', label: '壁纸图库', icon: 'image' },
  { to: '/admin/upload', label: '自定义上架', icon: 'upload' },
  { to: '/admin/desktop', label: '小伴桌面', icon: 'device' },
  { to: '/admin/keys', label: '供应商密钥', icon: 'key' },
  { to: '/admin/proxies', label: '代理池', icon: 'network' },
  { to: '/admin/cache', label: '搜索缓存', icon: 'database' },
  { to: '/admin/account', label: '账户', icon: 'user' },
]

const previousTitle = document.title

onMounted(() => {
  document.title = '管理后台 · Wallpaper Collect'
  // A session that expired while the page was open sends the user back to sign in.
  setUnauthorizedHandler(() => {
    admin.clear()
    router.replace({ path: '/admin/login', query: { redirect: route.fullPath } })
  })
})

onBeforeUnmount(() => {
  document.title = previousTitle
  setUnauthorizedHandler(null)
})

async function signOut() {
  await admin.logout()
  router.replace('/admin/login')
}
</script>

<template>
  <div class="admin shell">
    <aside class="side">
      <div class="brand">
        <RouterLink to="/admin/keys" class="display name" aria-label="管理后台首页">Wallpapers</RouterLink>
        <span class="role">管理后台</span>
      </div>

      <nav class="menu" aria-label="管理后台菜单">
        <RouterLink v-for="item in NAV" :key="item.to" :to="item.to" class="item">
          <Icon :name="item.icon" :size="20" />
          <span>{{ item.label }}</span>
        </RouterLink>
      </nav>

      <div class="foot">
        <RouterLink to="/" class="back">
          <Icon name="chevron-left" :size="16" />
          返回前台
        </RouterLink>
        <div class="who">
          <span class="user" :title="admin.user?.username">{{ admin.user?.username }}</span>
          <button class="signout" type="button" aria-label="退出登录" title="退出登录" @click="signOut">
            <Icon name="logout" :size="18" />
          </button>
        </div>
      </div>
    </aside>

    <main class="main">
      <div v-if="admin.user?.is_default_password" class="a-banner notice" role="alert">
        <span>还在使用默认密码，任何能访问这台服务器的人都能登录后台。</span>
        <RouterLink to="/admin/account">去修改密码</RouterLink>
      </div>
      <RouterView />
    </main>
  </div>
</template>

<style scoped>
.shell {
  display: grid;
  grid-template-columns: 248px minmax(0, 1fr);
  min-height: 100vh;
  min-height: 100dvh;
}

.side {
  position: sticky;
  top: 0;
  display: flex;
  height: 100vh;
  height: 100dvh;
  flex-direction: column;
  gap: 28px;
  padding: 26px 14px 18px;
  border-right: 1px solid var(--line);
  background: rgba(15, 15, 17, 0.72);
}

.brand {
  display: grid;
  gap: 2px;
  padding: 0 10px;
}

.name {
  font-size: 30px;
  line-height: 1;
}

.role {
  font-size: 13px;
  color: var(--text-3);
}

.menu {
  display: grid;
  gap: 2px;
}

.item {
  display: flex;
  align-items: center;
  gap: 12px;
  height: 42px;
  padding: 0 12px;
  border-radius: 10px;
  color: var(--text-2);
  transition:
    background 0.2s,
    color 0.2s;
}

.item:hover {
  color: var(--text);
  background: var(--surface);
}

.item.router-link-active {
  color: var(--text);
  background: var(--surface-2);
}

.foot {
  display: grid;
  gap: 10px;
  margin-top: auto;
}

.back {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 0 12px;
  color: var(--text-3);
  font-size: 13px;
}

.back:hover {
  color: var(--text);
}

.who {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 8px 8px 8px 12px;
  border-radius: 10px;
  border: 1px solid var(--line);
}

.user {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.signout {
  display: grid;
  flex: none;
  place-items: center;
  width: 32px;
  height: 32px;
  border-radius: 8px;
  color: var(--text-2);
}

.signout:hover {
  color: var(--text);
  background: var(--surface-2);
}

.main {
  min-width: 0;
}

.notice {
  margin: 20px 44px 0;
  max-width: 772px;
}

@media (max-width: 860px) {
  /* `1fr` alone would let the wide menu stretch the column past the screen. */
  .shell {
    grid-template-columns: minmax(0, 1fr);
  }

  .side {
    position: sticky;
    z-index: 10;
    height: auto;
    flex-direction: row;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px 12px;
    padding: calc(var(--safe-top) + 10px) var(--gutter) 8px;
    border-right: 0;
    border-bottom: 1px solid var(--line);
    background: rgba(15, 15, 17, 0.92);
    -webkit-backdrop-filter: blur(18px);
    backdrop-filter: blur(18px);
  }

  .brand {
    flex: 1;
    grid-auto-flow: column;
    align-items: baseline;
    justify-content: start;
    gap: 10px;
    padding: 0;
  }

  .name {
    font-size: 26px;
  }

  .foot {
    order: 2;
    margin: 0;
    grid-auto-flow: column;
    align-items: center;
  }

  .back {
    display: none;
  }

  .who {
    padding: 0 0 0 10px;
    border: 0;
  }

  .menu {
    order: 3;
    min-width: 0;
    flex-basis: 100%;
    grid-auto-flow: column;
    grid-auto-columns: max-content;
    overflow-x: auto;
    scrollbar-width: none;
  }

  .menu::-webkit-scrollbar {
    display: none;
  }

  .item {
    height: 38px;
    padding: 0 12px;
    white-space: nowrap;
  }

  .notice {
    margin: 16px var(--gutter) 0;
  }
}
</style>
