<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useAccount } from '@/stores/account'
import Icon from './Icon.vue'

const route = useRoute()
const account = useAccount()
const tab = computed(() => route.meta.tab as string)

const FORUM_URL = 'https://chat.xiangban.xyz/c/xiaoban-wallpaper'
const links = [
  { to: '/images', tab: 'home', label: '图片' },
  { to: '/videos', tab: 'videos', label: '视频' },
]
</script>

<template>
  <header class="topbar">
    <div class="inner">
      <RouterLink to="/" class="brand" aria-label="小伴桌面首页" :aria-current="tab === 'desktop' ? 'page' : undefined"><img src="/app-icon.png" alt=""><span>小伴桌面</span></RouterLink>

      <nav class="links" aria-label="主导航">
        <RouterLink
          v-for="l in links"
          :key="l.to"
          :to="l.to"
          :class="{ active: tab === l.tab }"
          :aria-current="tab === l.tab ? 'page' : undefined"
        >
          {{ l.label }}
        </RouterLink>
        <a :href="FORUM_URL" target="_blank" rel="noopener noreferrer">超级小伴论坛<Icon name="external" :size="14" /></a>
      </nav>

      <div class="actions">
        <RouterLink to="/search" class="search" aria-label="搜索">
          <Icon name="search" :size="18" />
          <span>搜索壁纸</span>
        </RouterLink>
        <RouterLink
          v-if="account.user"
          to="/me"
          class="account"
          :aria-label="`我的账号：${account.user.username}`"
        >
          <img v-if="account.user.avatar_url" :src="account.user.avatar_url" alt="" class="avatar">
          <Icon v-else name="user" :size="18" />
          <span class="label">{{ account.user.username }}</span>
        </RouterLink>
        <button v-else type="button" class="account" aria-label="登录或注册" @click="account.openLogin()">
          <Icon name="user" :size="18" />
          <span class="label">登录 / 注册</span>
        </button>
      </div>
    </div>
  </header>
</template>

<style scoped>
.topbar {
  position: sticky;
  top: 0;
  z-index: 40;
  padding-top: var(--safe-top);
  background: var(--header-bg);
  transition: background-color .45s ease, border-color .35s ease;
  -webkit-backdrop-filter: blur(18px) saturate(1.4);
  backdrop-filter: blur(18px) saturate(1.4);
  border-bottom: 1px solid var(--line);
}

.inner {
  display: flex;
  align-items: center;
  gap: 28px;
  height: var(--header-h);
  max-width: 1760px;
  margin: 0 auto;
  padding: 0 var(--gutter);
}

.brand {
  display: inline-flex;
  align-items: center;
  gap: 9px;
  flex-shrink: 0;
  font-size: 22px;
  font-weight: 650;
  line-height: 1;
}

.brand img { width: 42px; height: 37px; object-fit: contain; }

.links {
  display: flex;
  gap: 4px;
  margin-right: auto;
}

.links a {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border-radius: 999px;
  color: var(--text-2);
  transition: color 0.2s;
}

.links a:hover,
.links a.active {
  color: var(--text);
}

.links a.active {
  background: var(--surface-2);
}

.actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.search {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  height: 38px;
  min-width: 220px;
  padding: 0 16px;
  border-radius: 999px;
  background: var(--surface);
  border: 1px solid var(--line);
  color: var(--text-3);
  transition: border-color 0.2s;
}

.search:hover {
  border-color: var(--line-strong);
}

.account {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  height: 38px;
  max-width: 180px;
  padding: 0 14px;
  border-radius: 999px;
  border: 1px solid var(--line);
  color: var(--text-2);
  white-space: nowrap;
  transition: border-color 0.2s, color 0.2s;
}

.account:hover {
  border-color: var(--line-strong);
  color: var(--text);
}

.account .label {
  overflow: hidden;
  text-overflow: ellipsis;
}

.avatar {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  object-fit: cover;
}

@media (max-width: 768px) {
  /* Scrolls away with the page so the category rail can take the sticky slot at the top. */
  .topbar {
    position: relative;
    background: none;
    border-bottom: 0;
    -webkit-backdrop-filter: none;
    backdrop-filter: none;
  }

  .inner {
    gap: 12px;
  }

  .brand {
    margin-right: auto;
    font-size: 21px;
  }

  .links {
    display: none;
  }

  .search {
    min-width: 0;
    width: 40px;
    height: 40px;
    padding: 0;
    justify-content: center;
    border-radius: 50%;
  }

  .search span {
    display: none;
  }

  .account {
    width: 40px;
    height: 40px;
    padding: 0;
    justify-content: center;
    border-radius: 50%;
    background: var(--surface);
  }

  .account .label {
    display: none;
  }
}
</style>
