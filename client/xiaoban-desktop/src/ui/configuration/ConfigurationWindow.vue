<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watchEffect } from 'vue';
import { darkTheme, dateZhCN, NConfigProvider, NDialogProvider, NMessageProvider, zhCN } from 'naive-ui';
import { createControlStore } from '../control-store.ts';
import { GATEWAY_PROVIDER, invoke, listen, type AiConfigView, type AccountSession, type Settings, type AppInfo } from '../../bridge.ts';
import type { SettingsTab } from '../control-types.ts';
import SettingsDialog from '../components/SettingsDialog.vue';
import AiConfigCard from '../components/AiConfigCard.vue';
import AiGatewayCard from '../components/AiGatewayCard.vue';
import LoginModal from '../components/LoginModal.vue';
import { accent, followAccent, followHost, isDark, publishAccentTo, publishAppearanceTo } from '../appearance.ts';
import { applyUiTokens } from '../design-tokens.ts';
import { themeFor } from '../theme.ts';
import { closeConfigurationWindow } from './windows.ts';
const query = new URLSearchParams(location.search);
const role = query.get('kind') === 'model' ? 'model' : 'settings';
const provider = query.get('provider') ?? '__new__';
const toast = ref(''), ready = ref(false), error = ref(''), config = ref<AiConfigView | null>(null);
let timer: ReturnType<typeof setTimeout> | undefined, disposed = false;
const offs: (() => void)[] = [];
function notify(message: unknown) { toast.value = String(message); clearTimeout(timer); timer = setTimeout(() => toast.value = '', 5000); }
const store = createControlStore(notify, { configuration: true });
store.settingsOpen = true;
const tabs: SettingsTab[] = ['general', 'appearance', 'ai', 'assistant', 'voice', 'network', 'safety', 'about'];
function pickTab(value: unknown) { if (tabs.includes(value as SettingsTab)) store.settingsTab = value as SettingsTab; }
pickTab(query.get('tab'));
store.closeSettings = () => { void closeConfigurationWindow().catch(notify); };
store.openSettings = tab => { pickTab(tab); };
if (role === 'model') store.openLogin = () => { void invoke('configuration_window_login').then(closeConfigurationWindow).catch(notify); };
const naiveTheme = computed(() => isDark.value ? darkTheme : null);
const overrides = computed(() => themeFor(isDark.value, accent.value));
watchEffect(() => { applyUiTokens(isDark.value, accent.value); document.documentElement.dataset.theme = isDark.value ? 'dark' : 'light'; });
function follow(value: { appearance?: string; accent?: string }) { followHost(value.appearance); followAccent(value.accent); }
publishAppearanceTo(mode => { void invoke('set_appearance', { mode }).catch(notify); });
publishAccentTo(value => { void invoke('set_accent', { accent: value }).catch(notify); });
onMounted(async () => {
  try {
    let settingsVersion = 0, accountVersion = 0, configVersion = 0;
    const subscriptions = await Promise.all([
      listen<Settings>('settings-changed', value => { settingsVersion++; store.settings = value; follow(value); }),
      listen<AccountSession | null>('account-changed', value => { accountVersion++; store.account = value; }),
      listen<AiConfigView>('ai-config-changed', value => { configVersion++; config.value = value; }),
      listen<unknown>('configuration-tab', pickTab),
    ]);
    if (disposed) { subscriptions.forEach(off => off()); return; }
    offs.push(...subscriptions);
    const versions = [settingsVersion, accountVersion, configVersion];
    const [settings, info, account, ai] = await Promise.all([
      invoke<Settings>('get_settings'), invoke<AppInfo>('get_app_info'), invoke<AccountSession | null>('get_account_session'), invoke<AiConfigView>('get_ai_config'),
    ]);
    if (disposed) return;
    if (settingsVersion === versions[0]) { store.settings = settings; follow(settings); }
    store.info = info;
    if (accountVersion === versions[1]) store.account = account;
    if (configVersion === versions[2]) config.value = ai;
    if (role === 'settings') {
      const details = await Promise.allSettled([invoke<typeof store.startup>('get_startup_status'), invoke<typeof store.game>('get_game_status'), invoke<typeof store.website>('get_website_status')]);
      if (disposed) return;
      if (details[0].status === 'fulfilled') store.startup = details[0].value;
      if (details[1].status === 'fulfilled') store.game = details[1].value;
      if (details[2].status === 'fulfilled') store.website = details[2].value;
    }
    ready.value = true;
  } catch (e) { if (!disposed) error.value = String(e); }
});
onBeforeUnmount(() => { disposed = true; offs.forEach(off => off()); clearTimeout(timer); publishAppearanceTo(() => {}); publishAccentTo(() => {}); store.dispose(); });
</script>
<template>
  <NConfigProvider :theme="naiveTheme" :theme-overrides="overrides" :locale="zhCN" :date-locale="dateZhCN">
    <NMessageProvider><NDialogProvider>
      <main class="configuration-window" :class="`configuration-${role}`">
        <template v-if="ready">
          <SettingsDialog v-if="role === 'settings'" :store="store" detached @open-site="invoke('open_download_url', { url: 'https://wallpaper.xiangban.xyz' }).catch(notify)" />
          <AiGatewayCard v-else-if="provider === GATEWAY_PROVIDER" :store="store" :config="config" detached @config="config = $event" />
          <AiConfigCard v-else :store="store" detached :initial-provider="provider" />
          <LoginModal :store="store" />
        </template>
        <section v-else class="configuration-loading" role="status">
          <p>{{ error || '正在加载…' }}</p><button v-if="error" type="button" class="xb-btn" @click="closeConfigurationWindow()">关闭窗口</button>
        </section>
        <p v-if="toast" class="configuration-toast" role="status">{{ toast }}</p>
      </main>
    </NDialogProvider></NMessageProvider>
  </NConfigProvider>
</template>
<style>
html,body,#app{height:100%;margin:0}
.configuration-window{height:100dvh;width:100%;overflow:hidden;background:var(--ui-surface-sunken);color:var(--ui-text-primary);font-family:var(--ui-font-family)}
.configuration-loading{height:100%;display:flex;flex-direction:column;justify-content:center;align-items:center;font-size:13px}
.configuration-toast{position:fixed;bottom:16px;left:50%;transform:translateX(-50%);max-width:calc(100% - 48px);padding:10px 16px;background:var(--ui-surface-raised);color:var(--ui-text-primary);border:1px solid var(--ui-line);border-radius:10px;box-shadow:var(--ui-shadow-menu);font-size:13px;z-index:2000;overflow-wrap:anywhere}
</style>
