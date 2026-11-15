<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import AppIcon from '@/components/AppIcon.vue'

const props = defineProps<{
  title: string
  icon?: string | null
  compact?: boolean
  busy?: boolean
  closing?: boolean
  orientation?: 'portrait' | 'landscape'
}>()
const emit = defineEmits<{ reload: []; close: [] }>()
const workspace = ref<(HTMLElement & { webkitRequestFullscreen?: () => void | Promise<void> }) | null>(null)
const fullscreenDocument = document as Document & {
  webkitFullscreenEnabled?: boolean
  webkitFullscreenElement?: Element | null
  webkitExitFullscreen?: () => void | Promise<void>
}
const orientation = screen.orientation as ScreenOrientation & { lock?: (value: string) => Promise<void> }
const content = ref<HTMLElement | null>(null)
const fullscreenControl = ref<HTMLButtonElement | null>(null)
const fullscreen = ref(false)
const changingSize = ref(false)
const fullscreenError = ref('')
const failedIcon = ref('')
let orientationLocked = false
const fullscreenApi = computed(() => {
  if (document.fullscreenEnabled && workspace.value?.requestFullscreen) return 'standard'
  if (fullscreenDocument.webkitFullscreenEnabled && workspace.value?.webkitRequestFullscreen) return 'webkit'
  return null
})
const windowSizeAction = computed(() => fullscreen.value ? '退出全屏' : '进入全屏')

watch(fullscreen, active => {
  if (active) {
    content.value?.focus({ preventScroll: true })
  } else if (document.activeElement === content.value) {
    fullscreenControl.value?.focus({ preventScroll: true })
  }
}, { flush: 'post' })

function fullscreenElement() {
  return document.fullscreenElement || fullscreenDocument.webkitFullscreenElement || null
}

function releaseOrientation() {
  if (!orientationLocked) return
  orientationLocked = false
  orientation.unlock()
}

async function lockOrientation(element: HTMLElement) {
  if (!props.orientation || !orientation?.lock || window.matchMedia('(hover: hover) and (pointer: fine)').matches) return
  try {
    await orientation.lock(props.orientation)
    if (workspace.value === element && fullscreenElement() === element) orientationLocked = true
    else orientation.unlock()
  } catch { /* The user can still rotate the device. */ }
}

function syncFullscreen() {
  const element = workspace.value
  const active = element !== null && fullscreenElement() === element
  if (active === fullscreen.value) return
  fullscreen.value = active
  if (active && element) void lockOrientation(element)
  else releaseOrientation()
}

onMounted(() => {
  document.addEventListener('fullscreenchange', syncFullscreen)
  document.addEventListener('webkitfullscreenchange', syncFullscreen)
  syncFullscreen()
})
onBeforeUnmount(() => {
  document.removeEventListener('fullscreenchange', syncFullscreen)
  document.removeEventListener('webkitfullscreenchange', syncFullscreen)
  releaseOrientation()
})

async function toggleFullscreen() {
  const element = workspace.value
  if (!element || changingSize.value) return
  const exiting = fullscreenElement() === element
  if (!exiting && !fullscreenApi.value) return

  fullscreenError.value = ''
  changingSize.value = true
  try {
    if (exiting) {
      if (document.fullscreenElement === element) await document.exitFullscreen()
      else await fullscreenDocument.webkitExitFullscreen?.()
    } else if (fullscreenApi.value === 'standard') {
      await element.requestFullscreen()
    } else {
      await element.webkitRequestFullscreen?.()
    }
  } catch {
    fullscreenError.value = exiting ? '无法退出全屏，请重试。' : '无法进入全屏，请重试。'
  } finally {
    syncFullscreen()
    changingSize.value = false
  }
}

function handleEscape(event: KeyboardEvent) {
  if (event.defaultPrevented || event.repeat || changingSize.value) return
  const active = fullscreenElement()
  if (active) {
    if (active === workspace.value) void toggleFullscreen()
    return
  }
  if (fullscreen.value) return
  event.preventDefault()
  emit('close')
}
</script>

<template>
  <main ref="workspace" class="workspace" :class="{ fullscreen, compact }" @keydown.esc="handleEscape">
    <section class="app-window" :aria-label="title">
      <header class="titlebar">
        <img v-if="icon && failedIcon !== icon" :src="icon" alt="" class="app-icon"
          referrerpolicy="no-referrer" @error="failedIcon = icon || ''" />
        <span v-else class="app-icon fallback" aria-hidden="true"><AppIcon name="grid" :size="15" /></span>
        <h1 :title="title">{{ title }}</h1>
        <div class="window-actions">
          <button type="button" aria-label="重新加载" title="重新加载" :disabled="busy || closing" @click="emit('reload')">
            <AppIcon name="refresh" :size="18" />
          </button>
          <button v-if="fullscreenApi || fullscreen" ref="fullscreenControl" type="button" :aria-label="windowSizeAction" :title="windowSizeAction"
            :disabled="closing || changingSize" @click="toggleFullscreen">
            <AppIcon :name="fullscreen ? 'restore' : 'expand'" :size="17" />
          </button>
          <button type="button" class="close-action" aria-label="关闭应用" title="关闭应用"
            :disabled="closing" @click="emit('close')">
            <AppIcon name="close" :size="18" />
          </button>
        </div>
      </header>
      <div ref="content" class="window-content" tabindex="-1" :inert="closing">
        <slot />
        <p v-if="fullscreenError" class="fullscreen-feedback" role="status">{{ fullscreenError }}</p>
      </div>
    </section>
  </main>
</template>

<style scoped>
.workspace {
  display: grid;
  grid-template: minmax(0, 1fr) / minmax(0, 1fr);
  place-items: center;
  height: 100vh;
  height: 100dvh;
  padding: clamp(16px, 4vh, 40px) clamp(16px, 4vw, 56px);
  background: radial-gradient(ellipse at 50% 0, var(--brand-soft) 0, transparent 70%), var(--paper);
}
.app-window {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  width: min(100%, 1120px);
  height: min(100%, 900px);
  min-width: 0;
  overflow: hidden;
  border: 1px solid var(--line-strong);
  border-radius: 18px;
  background: var(--surface);
  box-shadow: var(--shadow), 0 18px 62px rgb(0 0 0 / 10%);
}
.compact .app-window { width: min(100%, 520px); }
.titlebar {
  display: flex;
  align-items: center;
  gap: 10px;
  min-height: 48px;
  padding: 0 6px 0 14px;
  border-bottom: 1px solid var(--line);
  background: var(--surface);
  user-select: none;
}
.app-icon {
  flex: none;
  display: block;
  width: 26px;
  height: 26px;
  border-radius: 7px;
  object-fit: cover;
  background: var(--surface-2);
}
.app-icon.fallback { display: grid; place-items: center; color: var(--brand); }
h1 {
  flex: 1;
  min-width: 0;
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 14px;
  font-weight: 650;
}
.window-actions { display: flex; flex: none; align-items: center; gap: 2px; }
.window-actions button {
  display: grid;
  place-items: center;
  width: 40px;
  height: 40px;
  padding: 0;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: var(--ink-2);
}
.window-actions button:not(:disabled):hover { background: var(--surface-2); color: var(--ink); }
.window-actions .close-action:not(:disabled):hover { color: var(--danger); }
.window-actions button:disabled { opacity: .45; }
.window-content { position: relative; min-width: 0; min-height: 0; overflow: hidden; }
.workspace.fullscreen { width: 100%; height: 100%; padding: 0; }
.fullscreen .app-window { width: 100%; height: 100%; border: 0; border-radius: 0; box-shadow: none; }
.fullscreen-feedback {
  position: absolute;
  z-index: 2;
  top: 8px;
  right: 8px;
  margin: 0;
  padding: 8px 12px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: var(--surface);
  color: var(--ink);
  font-size: 13px;
  pointer-events: none;
}
.fullscreen .app-window { grid-template-rows: minmax(0, 1fr); }
.fullscreen .titlebar { display: none; }
.fullscreen .window-content:focus { outline: none; }
@media (max-width: 640px) {
  .workspace { padding: 0; }
  .workspace .app-window { width: 100%; height: 100%; border: 0; border-radius: 0; box-shadow: none; }
  .titlebar {
    min-height: calc(48px + env(safe-area-inset-top));
    padding: env(safe-area-inset-top) max(4px, env(safe-area-inset-right)) 0 max(10px, env(safe-area-inset-left));
  }
  .window-actions button { width: 44px; height: 44px; }
  .window-content { padding-bottom: env(safe-area-inset-bottom); }
}
</style>
