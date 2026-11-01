<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watchEffect } from 'vue';
import { darkTheme, NConfigProvider, zhCN } from 'naive-ui';
import { FileText, RefreshCw, MessageCircle, ListChecks, ExternalLink, Download } from 'lucide-vue-next';
import { invoke, listen, type Settings } from '../../bridge.ts';
import { accent, followAccent, followHost, isDark } from '../appearance.ts';
import { applyUiTokens } from '../design-tokens.ts';
import { configurationThemeFor } from '../theme.ts';
import WindowChrome from '../configuration/WindowChrome.vue';
import WindowResizeEdges from '../configuration/WindowResizeEdges.vue';
import { closeConfigurationWindow } from '../configuration/windows.ts';
import { createCloudDownloader, type DownloadProgress } from './cloud-download.ts';
import type { LibraryDoc, LibraryLink } from './library-store.ts';
import type { CloudSnapshot } from './cloud-document.ts';
const id = new URLSearchParams(location.search).get('provider') || '';
const link = ref<LibraryLink | null>(null), content = ref<CloudSnapshot | null>(null);
const localDoc = ref<LibraryDoc | null>(null);
const downloader = createCloudDownloader(invoke, doc => { localDoc.value = doc; });
const busy = ref(false), preparing = ref(false), error = ref('');
const theme = computed(() => isDark.value ? darkTheme : null);
const overrides = computed(() => configurationThemeFor(isDark.value, accent.value));
const coverage = computed(() => content.value?.state.coverage === 'full' ? '正文 · 文字内容' : content.value?.state.coverage === 'partial' ? '摘要／部分内容' : '暂无可读正文');
watchEffect(() => { applyUiTokens(isDark.value, accent.value); document.documentElement.dataset.theme = isDark.value ? 'dark' : 'light'; });
document.documentElement.classList.add('configuration-frameless');
let disposed = false;
const offs: (() => void)[] = [];
function follow(settings: Settings) { followHost(settings.appearance); followAccent(settings.accent); }
function escape(e: KeyboardEvent) { if (e.key === 'Escape' && !e.defaultPrevented) { e.preventDefault(); void closeConfigurationWindow(); } }
async function read(refresh = false) {
  if (busy.value) return;
  busy.value = true; error.value = '';
  try { const result = await invoke<CloudSnapshot>('library_cloud_read', { id, refresh }); if (!disposed) content.value = result; }
  catch (e) { if (!disposed) error.value = String(e); }
  finally { if (!disposed) busy.value = false; }
}
async function openSource() { try { await invoke('library_cloud_open_source', { id }); } catch (e) { error.value = String(e); } }
async function ask(mode: 'ask' | 'summary') {
  if (preparing.value || !content.value?.text) return;
  preparing.value = true; error.value = '';
  try { await invoke('library_cloud_open_task', { id, mode }); await closeConfigurationWindow(); }
  catch (e) { if (!disposed) error.value = String(e); }
  finally { if (!disposed) preparing.value = false; }
}
onMounted(async () => {
  window.addEventListener('keydown', escape);
  try {
    let version = 0;
    const off = await listen<Settings>('settings-changed', s => { version++; follow(s); });
    if (disposed) { off(); return; } offs.push(off);
    const initial = version;
    const [settings, item] = await Promise.all([invoke<Settings>('get_settings'), invoke<{ link: LibraryLink; content: CloudSnapshot | null; localDoc?: LibraryDoc | null }>('library_cloud_item', { id })]);
    if (disposed) return;
    if (initial === version) follow(settings);
    link.value = item.link; content.value = item.content; localDoc.value = item.localDoc ?? null;
    const stopProgress = await listen<DownloadProgress>('library-cloud-download-progress', event => downloader.progress(event)); if (disposed) { stopProgress(); return; } offs.push(stopProgress);
    if (!item.content) await read();
  } catch (e) { if (!disposed) error.value = String(e); }
});
onBeforeUnmount(() => { disposed = true; window.removeEventListener('keydown', escape); offs.forEach(off => off()); });
</script>
<template>
  <NConfigProvider :theme="theme" :theme-overrides="overrides" :locale="zhCN">
    <main class="cloud-document-window">
      <WindowChrome title="云文档预览" />
      <header class="cloud-document-heading"><span class="cloud-document-icon"><FileText :size="24" /></span><div><h1>{{ link?.title || '云文档' }}</h1><p>{{ link?.platform === 'baidu' ? '百度网盘' : '腾讯文档' }}<template v-if="content"> · {{ new Date(content.state.updatedMs).toLocaleString('zh-CN') }} 保存</template></p></div><button v-if="link?.platform === 'baidu'" class="xb-btn" :disabled="!!downloader.id || !!localDoc" @click="downloader.download(id)"><Download :size="14" />{{ downloader.id ? downloader.label() : localDoc ? '原文件已保存' : '下载原文件' }}</button><button class="xb-btn" :disabled="!link" @click="openSource"><ExternalLink :size="14" />源站打开</button><button class="xb-btn" :disabled="busy" @click="read(true)"><RefreshCw :size="14" />{{ busy ? '读取中…' : '刷新内容' }}</button></header>
      <div v-if="content" class="cloud-document-scope"><b>{{ coverage }}</b><span>{{ content.state.note }}</span></div>
      <div v-if="downloader.error" class="cloud-document-error" role="alert">{{ downloader.error }}</div>
      <p v-if="downloader.message || downloader.id" class="cloud-document-download" role="status">{{ downloader.message || downloader.label() }}<button v-if="downloader.id" class="xb-btn" :disabled="downloader.cancelling" @click="downloader.cancel">取消下载</button></p>
      <div v-if="error" class="cloud-document-error" role="alert">{{ error }}<button class="xb-btn" :disabled="busy" @click="read(true)">重试读取</button></div>
      <div v-if="busy && !content" class="cloud-document-placeholder" role="status">正在读取云端正文，首次读取可能需要一些时间…</div>
      <pre v-else-if="content?.text" class="cloud-document-body" data-testid="cloud-document-body">{{ content.text }}</pre>
      <div v-else class="cloud-document-placeholder">{{ localDoc ? '原文件已保存到资料库的“我的文档”，可在那里查看解析状态并提问。' : content ? '该文件当前没有可用文字，可刷新内容或在源站查看。' : '还没有保存正文，连接账号后可重新读取。' }}</div>
      <footer class="cloud-document-actions"><span>{{ content?.text ? '正文快照保存于本机，可离线查看与提问' : '成功读取后，可保存内容并带入提问' }}</span><button class="xb-btn" :disabled="busy || preparing || !content?.text" @click="ask('summary')"><ListChecks :size="15" />准备总结</button><button class="xb-btn-primary" :disabled="busy || preparing || !content?.text" @click="ask('ask')"><MessageCircle :size="15" />{{ preparing ? '准备中…' : '带入提问' }}</button></footer>
      <WindowResizeEdges />
    </main>
  </NConfigProvider>
</template>
<style>
html,body,#app{height:100%;margin:0}html.configuration-frameless,html.configuration-frameless body,html.configuration-frameless #app{background:transparent}
.cloud-document-window{height:100dvh;display:flex;flex-direction:column;overflow:hidden;border-radius:12px;background:var(--ui-surface);color:var(--ui-text-primary);font-family:var(--ui-font-family)}
.cloud-document-window>.configuration-chrome{border-bottom:1px solid var(--ui-line)}
.cloud-document-heading{display:flex;align-items:center;gap:14px;padding:24px 28px 18px;flex:none}.cloud-document-icon{display:grid;place-items:center;width:46px;height:46px;border-radius:13px;background:var(--ui-control);color:var(--ui-accent)}.cloud-document-heading>div{flex:1;min-width:0}.cloud-document-heading h1{font-size:19px;line-height:1.4;margin:0;overflow-wrap:anywhere}.cloud-document-heading p{font-size:12px;color:var(--ui-text-secondary);margin:5px 0 0}
.cloud-document-scope{display:flex;gap:10px;align-items:baseline;flex-wrap:wrap;margin:0 28px 16px;padding:11px 14px;background:var(--ui-control);border-radius:9px;font-size:12px;color:var(--ui-text-secondary)}.cloud-document-scope b{color:var(--ui-text-primary)}
.cloud-document-body{flex:1;min-height:0;overflow:auto;white-space:pre-wrap;overflow-wrap:anywhere;margin:0;padding:8px 32px 32px;font:14px/1.85 var(--ui-font-family);user-select:text}.cloud-document-placeholder{flex:1;display:grid;place-items:center;padding:40px;color:var(--ui-text-secondary);font-size:13px;text-align:center}
.cloud-document-download{display:flex;align-items:center;justify-content:space-between;margin:0 28px 16px;font-size:13px;color:var(--ui-text-secondary)}.cloud-document-error{display:flex;align-items:center;gap:12px;margin:0 28px 16px;color:var(--ui-danger);font-size:13px}.cloud-document-error button{margin-left:auto;flex:none}
.cloud-document-actions{display:flex;align-items:center;gap:10px;padding:16px 28px;border-top:1px solid var(--ui-line);flex:none}.cloud-document-actions>span{flex:1;font-size:12px;color:var(--ui-text-secondary)}.cloud-document-actions button,.cloud-document-heading>button{display:inline-flex;align-items:center;gap:6px;white-space:nowrap}.cloud-document-actions button:disabled{opacity:.45;cursor:default}html.configuration-expanded .cloud-document-window{border-radius:0}
</style>
