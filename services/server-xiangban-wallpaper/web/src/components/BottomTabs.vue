<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import Icon, { type IconName } from './Icon.vue'

const route = useRoute()
const tab = computed(() => route.meta.tab as string)

const tabs: { to: string; tab: string; label: string; icon: IconName }[] = [
  { to: '/', tab: 'home', label: '图片', icon: 'image' },
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
      <span class="glyph"><Icon :name="t.icon" :size="24" /></span>
      <span class="label">{{ t.label }}</span>
    </RouterLink>
  </nav>
</template>

<style scoped>
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
    background: rgba(15, 15, 17, 0.78);
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
