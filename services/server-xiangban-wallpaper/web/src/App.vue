<script setup lang="ts">
import { computed, watch } from 'vue'
import { useRoute } from 'vue-router'
import TopBar from './components/TopBar.vue'
import BottomTabs from './components/BottomTabs.vue'
import ToastHost from './components/ToastHost.vue'
import Viewer from './components/Viewer.vue'
import { ambientColor } from './composables/ambient'
import { useCatalog } from './stores/catalog'
import { useFavorites } from './stores/favorites'

const route = useRoute()
const catalog = useCatalog()
const favorites = useFavorites()

// The admin console has its own layout; the public site's navigation and viewer stay out of it.
watch(() => route.meta.tab, tab => {
  document.documentElement.dataset.siteTheme = tab === 'desktop' ? 'light' : 'dark'
}, { immediate: true })

const isAdmin = computed(() => route.meta.admin === true)

const glow = computed(() => ({ '--ambient': ambientColor.value ?? '#2b2b33' }))

let publicDataLoaded = false
watch(
  isAdmin,
  (admin) => {
    if (admin || publicDataLoaded) return
    publicDataLoaded = true
    catalog.load()
    favorites.load()
  },
  { immediate: true },
)
</script>

<template>
  <RouterView v-if="isAdmin" />
  <template v-else>
    <div class="ambient" :style="glow" aria-hidden="true" />
    <TopBar />
    <RouterView v-slot="{ Component }">
      <KeepAlive :max="6">
        <component :is="Component" :key="route.path" />
      </KeepAlive>
    </RouterView>
    <BottomTabs />
    <Viewer />
  </template>
  <ToastHost />
</template>

<style>
.ambient {
  position: fixed;
  inset: 0;
  z-index: -1;
  pointer-events: none;
  background:
    radial-gradient(70% 46% at 12% -4%, color-mix(in srgb, var(--ambient) 62%, transparent), transparent 72%),
    radial-gradient(54% 40% at 104% 22%, color-mix(in srgb, var(--ambient) 34%, transparent), transparent 74%);
  transition: --ambient 1.4s ease;
}
</style>
