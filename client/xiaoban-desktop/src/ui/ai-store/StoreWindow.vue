<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watchEffect } from 'vue';
import { darkTheme, NConfigProvider, NDialogProvider, NMessageProvider, zhCN } from 'naive-ui';
import { invoke, listen, type Settings } from '../../bridge.ts';
import { createControlStore } from '../control-store.ts';
import { accent, followAccent, followHost, isDark } from '../appearance.ts';
import { applyUiTokens } from '../design-tokens.ts';
import { configurationThemeFor } from '../theme.ts';
import WindowChrome from '../configuration/WindowChrome.vue';
import WindowResizeEdges from '../configuration/WindowResizeEdges.vue';
import AiStorePage from '../pages/AiStorePage.vue';
import RegistryCatalog from './RegistryCatalog.vue';
import ConnectorCenter from './ConnectorCenter.vue';
import MarketGroupDialog from './MarketGroupDialog.vue';
import MarketUpdateDialog from './MarketUpdateDialog.vue';
import type { Skill } from './catalog.ts';
import type { UpdateInfo } from './market.ts';
import type { StoreSelection } from './store-window.ts';
import { closeConfigurationWindow } from '../configuration/windows.ts';
import './store-cards.css';
let selection: StoreSelection | undefined;
try { selection = JSON.parse(new URLSearchParams(location.search).get('provider') || ''); } catch { /* Show a recoverable error. */ }
const error = ref(selection ? '' : '无法读取插件信息，请关闭窗口后重试。');
const toast = ref(''), update = ref<UpdateInfo | null>(null), installed = ref<Skill | null>(null);
const centerId = ref(''), updates = ref<Record<string, UpdateInfo>>({});
const connectorWindow = selection?.kind === 'connectors';
const store = createControlStore(message => { toast.value = String(message); }, { configuration: true });
const theme = computed(() => isDark.value ? darkTheme : null);
const overrides = computed(() => configurationThemeFor(isDark.value, accent.value));
watchEffect(() => { applyUiTokens(isDark.value, accent.value); document.documentElement.dataset.theme = isDark.value ? 'dark' : 'light'; });
document.documentElement.classList.add('configuration-frameless');
let disposed = false;
const offs: (() => void)[] = [];
function follow(settings: Settings) { followHost(settings.appearance); followAccent(settings.accent); }
async function finish(message: string) { store.notify(message); await closeConfigurationWindow(); }
function escape(event: KeyboardEvent) { if (event.key === 'Escape' && !event.defaultPrevented) { event.preventDefault(); void closeConfigurationWindow(); } }
async function showCenter(plugin: string) {
  try {
    const item = (await invoke<import('./connector-plugins.ts').InstalledConnector[]>('ai_connector_installed')).find(s => s.plugin === plugin);
    if (!item) throw new Error('无法读取连接器，请关闭窗口后刷新。');
    centerId.value = item.id;
  } catch (e) { error.value = String(e); }
}
onMounted(async () => {
  window.addEventListener('keydown', escape);
  try {
    let version = 0;
    const off = await listen<Settings>('settings-changed', settings => { version++; follow(settings); });
    if (disposed) { off(); return; } offs.push(off);
    const initial = version, settings = await invoke<Settings>('get_settings');
    if (disposed) return;
    if (initial === version) follow(settings);
    if (selection?.view === 'registry' && selection.source === 'market') {
      const response = await invoke<{ results: UpdateInfo[] }>('ai_market_updates');
      if (!disposed) updates.value = Object.fromEntries(response.results.map(s => [s.slug, s]));
    }
    if (selection?.view === 'update') {
      const [items, updates] = await Promise.all([invoke<Skill[]>('ai_store_installed'), invoke<{ results: UpdateInfo[] }>('ai_market_updates')]);
      if (disposed) return;
      installed.value = items.find(s => s.slug === selection!.item.slug) ?? null;
      update.value = updates.results.find(s => s.slug === selection!.item.slug) ?? null;
      if (!installed.value || !update.value) error.value = '此插件暂无可用更新。';
    }
  } catch (e) { if (!disposed) error.value = String(e); }
});
onBeforeUnmount(() => { disposed = true; window.removeEventListener('keydown', escape); offs.forEach(off => off()); store.dispose(); });
</script>
<template>
  <NConfigProvider :theme="theme" :theme-overrides="overrides" :locale="zhCN"><NMessageProvider><NDialogProvider>
    <main class="store-native-window" :class="{ 'is-connector': connectorWindow }">
      <WindowChrome :title="connectorWindow ? undefined : selection?.kind === 'experts' ? '专家' : '技能'" />
      <p v-if="error" class="store-window-error" role="alert">{{ error }}</p>
      <template v-if="selection">
        <ConnectorCenter v-if="centerId" :store="store" :detail-id="centerId" />
        <AiStorePage v-else-if="selection.view === 'skill'" :store="store" :detail="selection" />
        <RegistryCatalog v-else-if="selection.view === 'registry'" :store="store" :kind="selection.kind === 'experts' ? 'experts' : 'connectors'" query="" category="全部" :source="selection.source" :detail="selection.item" :updates="updates" @center="showCenter" />
        <ConnectorCenter v-else-if="selection.view === 'connector'" :store="store" :detail-id="selection.item.slug" />
        <MarketGroupDialog v-else-if="selection.view === 'group'" :item="selection.item as Skill" :kind="selection.kind === 'experts' ? 'experts' : 'skills'" @close="closeConfigurationWindow()" @done="finish" />
        <MarketUpdateDialog v-else-if="selection.view === 'update' && installed && update" :pkg="{ ...installed, sha256: installed.manifestSha256 ?? undefined, files: installed.files ?? undefined }" :info="update" @close="closeConfigurationWindow()" @done="finish" />
      </template>
      <p v-if="toast" class="store-window-toast" role="status">{{ toast }}</p>
      <WindowResizeEdges />
    </main>
  </NDialogProvider></NMessageProvider></NConfigProvider>
</template>
<style>
html,body,#app{height:100%;margin:0}html.configuration-frameless,html.configuration-frameless body,html.configuration-frameless #app{background:transparent}
.store-native-window{height:100dvh;display:flex;flex-direction:column;overflow:hidden;border-radius:12px;background:var(--ui-surface);color:var(--ui-text-primary);font-family:var(--ui-font-family)}
.store-native-window>.configuration-chrome{background:var(--ui-surface);border-bottom:1px solid var(--ui-line)}
.store-native-window.is-connector>.configuration-chrome{border-bottom:0;min-height:36px}
.store-native-window>.ai-store-page,.store-native-window>.registry,.store-native-window>.connector-center{display:flex;flex-direction:column;flex:1;min-height:0;margin:0;padding:0}
.store-window-error{margin:24px;overflow-wrap:anywhere;font-size:13px;color:var(--ui-danger)}
.store-window-toast{position:fixed;bottom:16px;left:24px;right:24px;margin:0;padding:12px 16px;background:var(--ui-control);border-radius:10px;font-size:13px}
html.configuration-expanded .store-native-window{border-radius:0}
</style>
