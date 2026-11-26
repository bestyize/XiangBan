<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watchEffect } from 'vue';
import { darkTheme, NConfigProvider, NDialogProvider, NMessageProvider, zhCN } from 'naive-ui';
import { invoke, listen, type AppInfo, type Settings } from '../../bridge.ts';
import type { PetCatalog } from '../../pets/model.ts';
import { createControlStore } from '../control-store.ts';
import { accent, followAccent, followHost, isDark } from '../appearance.ts';
import { applyUiTokens } from '../design-tokens.ts';
import { configurationThemeFor } from '../theme.ts';
import { closeConfigurationWindow } from './windows.ts';
import WindowChrome from './WindowChrome.vue';
import WindowResizeEdges from './WindowResizeEdges.vue';
import PetSettingsDialog, { type PetSettingsPatch } from '../components/PetSettingsDialog.vue';
import PetProactivityPanel, { type EnvironmentContext, type GreetingSettings } from '../components/PetProactivityPanel.vue';
import ResourceShareActions from '../components/ResourceShareActions.vue';

const query = new URLSearchParams(location.search);
const kind = query.get('kind') === 'proactivity' ? 'proactivity' : 'pet';
const petId = query.get('provider') ?? '';
document.documentElement.classList.add('configuration-frameless');

const toast = ref(''), ready = ref(false), busy = ref(false), error = ref('');
const platform = ref('');
const catalog = ref<PetCatalog>({ themes: [], layout: { version: 1, instances: [] }, errors: [] });
const greeting = ref<GreetingSettings>({ enabled: false, frequency: 'natural', contextEnabled: false, calendarRegion: null });
const environment = ref<EnvironmentContext | null>(null);
const paused = ref(false);

let timer: ReturnType<typeof setTimeout> | undefined;
let disposed = false;
const offs: (() => void)[] = [];

function notify(message: unknown) {
  toast.value = String(message);
  clearTimeout(timer);
  timer = setTimeout(() => { toast.value = ''; }, 3600);
}

const store = createControlStore(notify, { configuration: true });
const naiveTheme = computed(() => (isDark.value ? darkTheme : null));
const overrides = computed(() => configurationThemeFor(isDark.value, accent.value));
watchEffect(() => {
  applyUiTokens(isDark.value, accent.value);
  document.documentElement.dataset.theme = isDark.value ? 'dark' : 'light';
});

const instance = computed(() => catalog.value.layout.instances.find(item => item.id === petId));
const currentTheme = computed(() => catalog.value.themes.find(item => item.id === instance.value?.themeId));

function follow(settings: Settings) {
  followHost(settings.appearance);
  followAccent(settings.accent);
  paused.value = !!settings.paused;
}

async function loadCatalog() {
  const data = await invoke<PetCatalog>('get_pet_catalog');
  if (!disposed) catalog.value = data;
}

async function savePet(patch: PetSettingsPatch | { x: number; y: number }, center = false) {
  if (!instance.value || busy.value) return;
  busy.value = true;
  try {
    catalog.value = await invoke<PetCatalog>('update_pet', { id: instance.value.id, patch });
    if (center) notify('已移到屏幕正中央');
    else await closeConfigurationWindow();
  } catch (e) { notify(e); }
  finally { busy.value = false; }
}

async function centerPet() {
  await savePet({ x: 0.5, y: 0.5 }, true);
}

async function removePet() {
  if (!instance.value || busy.value) return;
  busy.value = true;
  try {
    await invoke('remove_pet', { id: instance.value.id });
    await closeConfigurationWindow();
  } catch (e) { notify(e); }
  finally { busy.value = false; }
}

async function createFromPet() {
  try {
    await invoke('pet_open_creation', currentTheme.value ? { themeId: currentTheme.value.id, themeName: currentTheme.value.name } : {});
    await closeConfigurationWindow();
  } catch (e) { notify(e); }
}

async function changeGreeting(patch: Partial<GreetingSettings>) {
  if (!ready.value || busy.value) return;
  busy.value = true;
  try {
    greeting.value = await invoke<GreetingSettings>('configure_proactivity', { settings: { ...greeting.value, ...patch } });
    environment.value = null;
    notify('感知与陪伴设置已保存');
  } catch (e) { notify(e); }
  finally { busy.value = false; }
}

async function inspectEnvironment() {
  try { environment.value = await invoke<EnvironmentContext>('get_proactivity_context'); }
  catch (e) { notify(e); }
}

function onEscape(event: KeyboardEvent) {
  if (event.key === 'Escape' && !event.defaultPrevented && !busy.value) {
    event.preventDefault();
    void closeConfigurationWindow();
  }
}

onMounted(async () => {
  window.addEventListener('keydown', onEscape);
  try {
    let settingsVersion = 0;
    const offSettings = await listen<Settings>('settings-changed', s => { settingsVersion++; follow(s); });
    if (disposed) { offSettings(); return; }
    offs.push(offSettings);
    const initial = settingsVersion;
    const [settings, info] = await Promise.all([invoke<Settings>('get_settings'), invoke<AppInfo>('get_app_info')]);
    if (disposed) return;
    if (settingsVersion === initial) follow(settings);
    platform.value = info.platform;
    if (kind === 'pet') {
      await loadCatalog();
      if (!instance.value) error.value = '这只伙伴已不在桌面上。';
    } else {
      const result = await invoke<GreetingSettings>('get_proactivity_settings');
      if (!disposed && result) greeting.value = result;
    }
    ready.value = true;
  } catch (e) {
    if (!disposed) error.value = String(e);
  }
});

onBeforeUnmount(() => {
  disposed = true;
  window.removeEventListener('keydown', onEscape);
  offs.forEach(off => off());
  clearTimeout(timer);
  store.dispose();
});
</script>

<template>
  <NConfigProvider :theme="naiveTheme" :theme-overrides="overrides" :locale="zhCN">
    <NMessageProvider><NDialogProvider>
      <main class="pet-config-window">
        <template v-if="ready && !error">
          <PetSettingsDialog
            v-if="kind === 'pet' && instance"
            show
            detached
            :platform="platform"
            :instance="instance"
            :themes="catalog.themes"
            :busy="busy"
            :paused="paused"
            @save="savePet"
            @remove="removePet"
            @center="centerPet"
            @create="createFromPet"
            @close="closeConfigurationWindow()"
          >
            <template #extra>
              <div v-if="currentTheme?.source === 'custom'" class="pet-share-card" data-testid="pet-instance-share">
                <span class="pet-share-title" title="打包当前所属整套主题，不包含这只宠物的个人位置、昵称或心情">分享自定义主题「{{ currentTheme.name }}」</span>
                <ResourceShareActions :store="store" kind="pet" :id="instance.themeId" :disabled="busy" />
              </div>
            </template>
          </PetSettingsDialog>
          <PetProactivityPanel
            v-else-if="kind === 'proactivity'"
            show
            detached
            :platform="platform"
            :settings="greeting"
            :ready="ready"
            :busy="busy"
            :environment="environment"
            @change="changeGreeting"
            @inspect="inspectEnvironment"
            @close="closeConfigurationWindow()"
          />
        </template>
        <section v-else class="pet-config-empty" role="status">
          <WindowChrome :platform="platform" />
          <p>{{ error || '正在加载…' }}</p>
          <button v-if="error" type="button" class="xb-btn is-sm" @click="closeConfigurationWindow()">关闭窗口</button>
        </section>
        <p v-if="toast" class="pet-config-toast" role="status">{{ toast }}</p>
        <WindowResizeEdges />
      </main>
    </NDialogProvider></NMessageProvider>
  </NConfigProvider>
</template>

<style>
html,body,#app{height:100%;margin:0}
html.configuration-frameless,html.configuration-frameless body,html.configuration-frameless body.desktop-control,html.configuration-frameless #app{background:transparent!important}
.pet-config-window{height:100dvh;width:100%;overflow:hidden;border-radius:12px;background:var(--ui-configuration-canvas);box-shadow:inset 0 0 0 1px var(--ui-border);color:var(--ui-text-primary);font-family:var(--ui-font-family)}
html.configuration-expanded .pet-config-window{border-radius:0}
.pet-config-empty{height:100%;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:12px;font-size:13px;color:var(--ui-text-secondary)}
.pet-config-empty>.configuration-chrome{position:absolute;top:0;left:0;right:0}
.pet-share-card{background:var(--ui-configuration-group);border-radius:12px;padding:10px 14px;display:flex;align-items:center;justify-content:space-between;gap:10px;flex-wrap:wrap}
.pet-share-title{font-size:12.5px;font-weight:500;color:var(--ui-text-primary)}
.pet-config-toast{position:fixed;bottom:14px;left:50%;transform:translateX(-50%);max-width:calc(100% - 40px);padding:8px 14px;background:var(--ui-surface-raised);color:var(--ui-text-primary);border:1px solid var(--ui-line);border-radius:10px;box-shadow:var(--ui-shadow-menu);font-size:12.5px;z-index:2000}
</style>
