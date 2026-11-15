<script setup lang="ts">
import { computed, ref } from 'vue'
import AppIcon from '@/components/AppIcon.vue'

defineProps<{ title: string; icon?: string | null; compact?: boolean; busy?: boolean; closing?: boolean }>()
const emit = defineEmits<{ reload: []; close: [] }>()
const maximized = ref(false)
const failedIcon = ref('')
const windowSizeAction = computed(() => maximized.value ? '还原窗口' : '最大化窗口')
</script>

<template>
  <main class="workspace" :class="{ maximized, compact }" @keydown.esc.prevent="emit('close')">
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
          <button type="button" class="size-action" :aria-label="windowSizeAction" :title="windowSizeAction"
            @click="maximized = !maximized">
            <AppIcon :name="maximized ? 'restore' : 'expand'" :size="17" />
          </button>
          <button type="button" class="close-action" aria-label="关闭应用" title="关闭应用"
            :disabled="closing" @click="emit('close')">
            <AppIcon name="close" :size="18" />
          </button>
        </div>
      </header>
      <div class="window-content" :inert="closing"><slot /></div>
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
.workspace.maximized { padding: 0; }
.maximized .app-window { width: 100%; height: 100%; border: 0; border-radius: 0; box-shadow: none; }
@media (max-width: 640px) {
  .workspace { padding: 0; }
  .workspace .app-window { width: 100%; height: 100%; border: 0; border-radius: 0; box-shadow: none; }
  .titlebar {
    min-height: calc(48px + env(safe-area-inset-top));
    padding: env(safe-area-inset-top) max(4px, env(safe-area-inset-right)) 0 max(10px, env(safe-area-inset-left));
  }
  .window-actions button { width: 44px; height: 44px; }
  .window-actions .size-action { display: none; }
  .window-content { padding-bottom: env(safe-area-inset-bottom); }
}
</style>
