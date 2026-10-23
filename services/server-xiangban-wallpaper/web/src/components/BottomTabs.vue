<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import Icon, { type IconName } from './Icon.vue'

const route = useRoute()
const tab = computed(() => route.meta.tab as string)

const tabs: { to: string; tab: string; label: string; icon: IconName }[] = [
  { to: '/', tab: 'desktop', label: '小伴桌面', icon: 'device' },
  { to: '/guide', tab: 'guide', label: '玩法指南', icon: 'sparkles' },
  { to: '/plugins', tab: 'plugins', label: 'AI 插件', icon: 'grid' },
  { to: '/images', tab: 'home', label: '图片', icon: 'image' },
  { to: '/videos', tab: 'videos', label: '视频', icon: 'play' },
]
</script>

<template>
  <nav class="tabs" aria-label="主导航">
    <RouterLink
      v-for="t in tabs"
      :key="t.to"
      :to="t.to"
      :class="{ active: tab === t.tab }"
      :aria-current="tab === t.tab ? 'page' : undefined"
    >
      <span class="glyph"><img v-if="t.tab === 'desktop'" src="/app-icon.png" alt="" class="home-mark"><Icon v-else :name="t.icon" :size="24" /></span>
      <span class="label">{{ t.label }}</span>
    </RouterLink>
    <a href="https://chat.xiangban.xyz/c/xiaoban-wallpaper" target="_blank" rel="noopener noreferrer">
      <span class="glyph"><Icon name="network" :size="24" /></span>
      <span class="label">论坛</span>
    </a>
  </nav>
</template>

<style scoped>
.home-mark { width: 28px; height: 24px; object-fit: contain; }

.tabs {
  display: none;
}

@media (max-width: 768px) {
  .tabs {
    position: fixed;
    inset: auto 0 0;
    z-index: 40;
    display: grid;
    grid-auto-flow: column;
    grid-auto-columns: 1fr;
    padding-bottom: var(--safe-bottom);
    background: var(--tabs-bg);
    transition: background-color .45s ease, border-color .35s ease;
    -webkit-backdrop-filter: blur(22px) saturate(1.6);
    backdrop-filter: blur(22px) saturate(1.6);
    border-top: 1px solid var(--line);
  }

  a {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    height: var(--tabbar-h);
    color: var(--text-3);
    transition: color 0.2s;
  }

  a.active {
    color: var(--text);
  }

  a:active .glyph {
    transform: scale(0.88);
  }

  .glyph {
    position: relative;
    display: grid;
    transition: transform 0.15s var(--ease);
  }

  .label {
    font-size: 11px;
    line-height: 1;
  }
}
</style>

