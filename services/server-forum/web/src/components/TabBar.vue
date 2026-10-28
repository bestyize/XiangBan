<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSession } from '@/stores/session'

const route = useRoute()
const router = useRouter()
const session = useSession()

const meTarget = computed(() => (session.user ? `/u/${session.user.uid}` : ''))

function compose() {
  if (!session.requireLogin()) return
  router.push({ name: 'new' })
}

function me() {
  if (!session.requireLogin()) return
  if (meTarget.value) router.push(meTarget.value)
}
</script>

<template>
  <nav class="tabbar" aria-label="底部导航">
    <RouterLink to="/" :class="{ on: route.name === 'home' }">
      <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 10.5 12 4l8 6.5V20h-5.5v-5.5h-5V20H4z" /></svg>
      <span>首页</span>
    </RouterLink>
    <RouterLink to="/resources" :class="{ on: route.name === 'resources' || route.name === 'pack' }">
      <svg viewBox="0 0 24 24" aria-hidden="true"><rect x="4" y="4" width="7" height="7" rx="1.5" /><rect x="13" y="4" width="7" height="7" rx="1.5" /><rect x="4" y="13" width="7" height="7" rx="1.5" /><rect x="13" y="13" width="7" height="7" rx="1.5" /></svg>
      <span>资源</span>
    </RouterLink>
    <button type="button" class="compose" aria-label="发帖" @click="compose">
      <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5v14M5 12h14" /></svg>
    </button>
    <RouterLink to="/weekly" :class="{ on: route.name === 'weekly' || route.name === 'weekly-issue' || route.name === 'projects' || route.name === 'project' }">
      <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 4h14v16H5zM9 9h6M9 13h6M9 17h3" /></svg>
      <span>周刊</span>
    </RouterLink>
    <button type="button" :class="{ on: route.name === 'user' || route.name === 'settings' }" @click="me">
      <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="8.5" r="3.8" /><path d="M4.5 20c1-3.6 4-5.5 7.5-5.5s6.5 1.9 7.5 5.5" /></svg>
      <span>我的</span>
    </button>
  </nav>
</template>

<style scoped>
.tabbar {
  display: none;
}
@media (max-width: 767px) {
  .tabbar {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 30;
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    align-items: center;
    height: calc(var(--tabbar-h) + env(safe-area-inset-bottom));
    padding-bottom: env(safe-area-inset-bottom);
    background: color-mix(in srgb, var(--surface) 94%, transparent);
    backdrop-filter: blur(10px);
    -webkit-backdrop-filter: blur(10px);
    border-top: 1px solid var(--line);
  }
  a,
  button {
    display: grid;
    justify-items: center;
    gap: 2px;
    border: 0;
    background: none;
    color: var(--ink-3);
    font-size: 11px;
    font-weight: 600;
    padding: 6px 0;
  }
  svg {
    width: 23px;
    height: 23px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.7;
    stroke-linejoin: round;
    stroke-linecap: round;
  }
  .on {
    color: var(--ink);
  }
  .compose {
    justify-self: center;
    width: 46px;
    height: 46px;
    border-radius: 14px;
    background: var(--brand);
    color: var(--brand-ink);
    place-items: center;
    padding: 0;
  }
  .compose svg {
    stroke-width: 2.4;
  }
}
</style>
