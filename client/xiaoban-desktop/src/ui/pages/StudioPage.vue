<script setup lang="ts">
/**
 * 装扮: the four kinds of desktop decoration under large page-top tabs.
 * The inner pages are the existing ones; an inner page can put its own buttons in the header with
 * `<Teleport defer to="#studio-actions">…</Teleport>` (or the shell can fill the `actions` slot).
 */
import { defineAsyncComponent } from 'vue';
import type { ControlStore, StudioTab } from '../control-types.ts';
const WallpaperPage = defineAsyncComponent(() => import('./WallpaperPage.vue'));
const EffectsPage = defineAsyncComponent(() => import('./EffectsPage.vue'));
const WidgetsPage = defineAsyncComponent(() => import('./WidgetsPage.vue'));
const CompanionPage = defineAsyncComponent(() => import('./CompanionPage.vue'));
defineProps<{ store: ControlStore; widgetSettingsRequest?: { id: string } | null; petSettingsRequest?: { id: string } | null }>();
const emit = defineEmits<{ widgetSettingsOpened: []; petSettingsOpened: [] }>();
const tabs: { id: StudioTab; label: string }[] = [
  { id: 'wallpapers', label: '壁纸' }, { id: 'effects', label: '特效' }, { id: 'widgets', label: '组件' }, { id: 'pets', label: '伙伴' },
];
</script>
<template>
  <section class="studio-page" data-testid="studio-page" aria-label="装扮">
    <header class="studio-head" data-tauri-drag-region>
      <div class="xb-tabs-lg" role="tablist" aria-label="装扮类型">
        <button v-for="tab in tabs" :id="`studio-tab-${tab.id}`" :key="tab.id" type="button" role="tab" :aria-selected="store.studioTab === tab.id" :data-testid="`studio-tab-${tab.id}`" @click="store.studioTab = tab.id">{{ tab.label }}</button>
      </div>
      <div id="studio-actions" class="xb-page-actions"><slot name="actions" :tab="store.studioTab" /></div>
    </header>
    <div class="studio-body" role="tabpanel" :aria-labelledby="`studio-tab-${store.studioTab}`">
      <KeepAlive>
        <WallpaperPage v-if="store.studioTab === 'wallpapers'" :store="store" />
        <EffectsPage v-else-if="store.studioTab === 'effects'" :store="store" />
        <WidgetsPage v-else-if="store.studioTab === 'widgets'" :store="store" :settings-request="widgetSettingsRequest" @settings-opened="emit('widgetSettingsOpened')" />
        <CompanionPage v-else :store="store" :settings-request="petSettingsRequest" @settings-opened="emit('petSettingsOpened')" />
      </KeepAlive>
    </div>
  </section>
</template>
<style scoped>
.studio-page{flex:1;height:100%;min-height:0;min-width:0;display:flex;flex-direction:column;overflow:hidden}
.studio-head{flex-shrink:0;position:relative;z-index:2;background:var(--ui-surface);display:flex;align-items:center;gap:22px;padding:22px 28px 0}
.studio-body{flex:1;min-height:0;min-width:0;display:flex;flex-direction:column;overflow:hidden}
.studio-body>:deep(*){flex:1;min-height:0}
</style>

