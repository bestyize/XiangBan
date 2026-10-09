<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import Icon from './Icon.vue'

const route = useRoute()
const tab = computed(() => route.meta.tab as string)

const links = [
  { to: '/images', tab: 'home', label: '图片' },
  { to: '/videos', tab: 'videos', label: '视频' },
]
</script>

<template>
  <header class="topbar">
    <div class="inner">
      <RouterLink to="/" class="brand" aria-label="小伴桌面首页" :aria-current="tab === 'desktop' ? 'page' : undefined"><img src="/xiaoban.svg" alt=""><span>小伴桌面</span></RouterLink>

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
      </nav>

      <div class="actions">
        <RouterLink to="/search" class="search" aria-label="搜索">
          <Icon name="search" :size="18" />
          <span>搜索壁纸</span>
        </RouterLink>
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
}
</style>
