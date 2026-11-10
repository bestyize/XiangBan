<script setup lang="ts">
import { computed, ref } from 'vue';
import { NAlert, NSwitch } from 'naive-ui';
import { RefreshCw, Download, Sprout } from 'lucide-vue-next';
import { invoke } from '../../../bridge.ts';
import type { ControlStore } from '../../control-types.ts';
const props = defineProps<{ store: ControlStore }>();
const emit = defineEmits<{ openSite: [] }>();
const busy = ref('');
const update = computed(() => props.store.website.update);
const installation = computed(() => props.store.website.installation);
const announcements = computed(() => props.store.website.announcements);
const platform = computed(() => props.store.info.platform === 'windows' ? 'Windows · 64 位' : props.store.info.platform === 'macos' ? 'macOS' : '浏览器预览');
async function act(key: string, action: () => Promise<unknown>) {
  busy.value = key;
  try { await action(); } catch (error) { props.store.notify(error, 'error'); } finally { busy.value = ''; }
}
function openUpdatePage() {
  const url = update.value?.downloadUrl;
  if (url) return act('download', () => invoke('open_download_url', { url }));
}
function startUpdate() {
  const release = update.value;
  if (!release?.available) return;
  if (release.inApp) return act('download', () => invoke('install_website_update', { version: release.version }));
  return openUpdatePage();
}
</script>
<template>
  <div class="settings-about" data-testid="settings-about">
    <div class="xb-group about-identity">
      <div class="xb-mark about-mark"><Sprout :size="32" /></div>
      <div class="xb-row-text"><b class="about-name">小伴桌面</b><span>版本 {{ store.info.version }} · {{ platform }}</span></div>
      <button v-if="update?.available" type="button" class="xb-btn-primary" data-testid="about-download" :disabled="!!busy || installation?.busy" @click="startUpdate"><Download :size="14" />{{ update.inApp ? '安装并重启' : `下载 v${update.version}` }}</button>
      <button type="button" class="xb-btn" data-testid="about-check" :disabled="!store.desktop || !!busy || installation?.busy" @click="act('check', () => store.checkWebsite())"><RefreshCw :size="14" :class="{ spinning: busy === 'check' }" />检查更新</button>
    </div>
    <div class="xb-group-title">更新</div>
    <div class="xb-group">
      <div class="xb-row"><div class="xb-row-text"><b>自动检查版本与公告</b><span>每天自动同步，安装更新由你确认</span></div>
        <NSwitch :value="store.settings.automaticUpdates" aria-label="自动检查版本与公告" :disabled="!store.desktop || !!busy" :loading="busy === 'automatic'" @update:value="value => act('automatic', () => store.setAutomaticUpdates(value))" /></div>
      <div v-if="update?.available" class="xb-row release"><div class="xb-row-text"><b>新版本 v{{ update.version }}</b><span v-if="update.publishedAt">{{ update.publishedAt }}</span><p>{{ update.notes }}</p></div></div>
      <div v-else-if="update" class="xb-row"><div class="xb-row-text"><span>已完成检查，当前是最新版本</span></div></div>
      <div v-if="installation?.busy" class="xb-row" role="status"><div class="xb-row-text"><span>{{ installation.percent === 100 ? '正在安装，小伴即将重启…' : `正在下载并校验更新${installation.percent == null ? '…' : ` · ${installation.percent}%`}` }}</span></div></div>
      <div v-if="installation?.error" class="xb-row">
        <NAlert type="warning" :show-icon="false" style="flex:1">{{ installation.error }}</NAlert>
        <button v-if="update?.downloadUrl" type="button" class="xb-btn" :disabled="!!busy" @click="openUpdatePage">打开下载页</button>
      </div>
      <div v-if="store.website.errors.length" class="xb-row"><NAlert type="warning" :show-icon="false" style="flex:1">{{ store.website.errors.join('；') }}</NAlert></div>
    </div>
    <div class="xb-group-title">公告</div>
    <div class="xb-group">
      <div v-if="!announcements.length" class="xb-row"><div class="xb-row-text"><span>{{ store.desktop ? '当前没有公告，可通过检查更新同步' : '公告会在桌面应用连接网站后展示' }}</span></div></div>
      <div v-for="notice in announcements" :key="notice.id" class="xb-row notice">
        <div class="xb-row-text"><b><i v-if="store.website.dismissed[notice.id] !== notice.revision" class="xb-dot"></i>{{ notice.title }}</b><span>{{ notice.body }}</span></div>
        <button v-if="store.website.dismissed[notice.id] !== notice.revision" type="button" class="xb-btn is-sm" @click="act('dismiss', async () => { await invoke('dismiss_website_message', { id: notice.id, revision: notice.revision }); await store.checkWebsite(); })">标为已读</button>
      </div>
    </div>
    <div class="about-links"><button type="button" class="xb-link" @click="emit('openSite')">小伴官网</button></div>
  </div>
</template>
<style scoped>
.about-identity{display:flex;align-items:center;gap:18px;padding:24px}
.about-mark{width:64px;height:64px;border-radius:18px}
.about-name{font-size:17px!important;font-weight:600!important}
.release p{margin:6px 0 0;white-space:pre-wrap;font-size:12.5px;line-height:1.7;color:var(--ui-text-secondary)}
.notice b{display:flex;align-items:center;gap:6px}
.notice span{white-space:pre-wrap}
.about-links{display:flex;gap:18px;justify-content:center;margin-top:20px;font-size:12.5px}
.spinning{animation:spin 1s linear infinite}
@keyframes spin{to{transform:rotate(360deg)}}
</style>
