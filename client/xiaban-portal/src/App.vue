<script setup lang="ts">
import { onBeforeUnmount, onMounted } from 'vue';
import PortalHome from './views/PortalHome.vue';
import DesktopDownloadView from './views/DesktopDownloadView.vue';
import MobileDownloadView from './views/MobileDownloadView.vue';
import { currentView, syncRouteFromLocation } from './lib/router';
import { toastText } from './lib/toast';

function onPopstate() {
  syncRouteFromLocation();
}

onMounted(() => {
  syncRouteFromLocation();
  window.addEventListener('popstate', onPopstate);
});

onBeforeUnmount(() => {
  window.removeEventListener('popstate', onPopstate);
});
</script>

<template>
  <DesktopDownloadView v-if="currentView === 'desktop'" />
  <MobileDownloadView v-else-if="currentView === 'mobile'" />
  <PortalHome v-else />

  <Transition name="xb-toast">
    <div v-if="toastText" class="xb-toast" role="status" aria-live="polite">
      {{ toastText }}
    </div>
  </Transition>
</template>

<style>
:root {
  --canvas: #faf8f5;
  --canvas-soft: #f2eee8;
  --surface: #ffffff;
  --ink: #18171a;
  --ink-2: #545058;
  --ink-3: #8a848e;

  --coral: #ee4f6d;
  --coral-2: #f9744b;
  --coral-grad: linear-gradient(135deg, #ff5b7d 0%, #f9744b 100%);
  --coral-soft: #fff1f3;

  --line: rgba(24, 23, 26, 0.08);
  --line-strong: rgba(24, 23, 26, 0.16);
  --shadow-sm: 0 2px 10px rgba(24, 20, 24, 0.04);
  --shadow-md: 0 14px 32px rgba(24, 20, 24, 0.09);
  --shadow-lg: 0 28px 64px rgba(24, 20, 24, 0.18);

  --font: 'PingFang SC', 'Hiragino Sans GB', 'Noto Sans SC', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  --mono: ui-monospace, 'SFMono-Regular', Menlo, Monaco, Consolas, monospace;
  --ease: cubic-bezier(0.22, 1, 0.36, 1);
}

*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
html {
  background: var(--canvas);
  color: var(--ink);
  font-family: var(--font);
  -webkit-font-smoothing: antialiased;
  scroll-behavior: smooth;
}
body {
  min-height: 100vh;
  background: var(--canvas);
  color: var(--ink);
  font-size: 14px;
  line-height: 1.5;
}
a { color: inherit; text-decoration: none; }
button { font: inherit; color: inherit; background: none; border: 0; cursor: pointer; }
img { display: block; max-width: 100%; }

.xb-toast {
  position: fixed;
  left: 50%;
  bottom: calc(28px + env(safe-area-inset-bottom, 0px));
  transform: translateX(-50%);
  z-index: 600;
  max-width: min(380px, calc(100vw - 32px));
  padding: 10px 18px;
  border-radius: 999px;
  background: rgba(24, 23, 28, 0.92);
  color: #ffffff;
  font-size: 13px;
  font-weight: 560;
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.28);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  pointer-events: none;
  text-align: center;
}
.xb-toast-enter-active,
.xb-toast-leave-active {
  transition: opacity 0.2s ease, transform 0.24s cubic-bezier(0.22, 1, 0.36, 1);
}
.xb-toast-enter-from,
.xb-toast-leave-to {
  opacity: 0;
  transform: translate(-50%, 10px);
}
</style>
