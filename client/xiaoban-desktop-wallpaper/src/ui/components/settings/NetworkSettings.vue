<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue';
import { NAlert, NSelect } from 'naive-ui';
import { invoke, desktop } from '../../../bridge.ts';
import PhoneLinkCard from './PhoneLinkCard.vue';

type Mode = 'system' | 'direct' | 'manual';
interface NetworkConfig { mode: Mode; proxyUrl: string }
const options = [
  { label: '自动检测', value: 'system' },
  { label: '直接连接', value: 'direct' },
  { label: '手动代理', value: 'manual' },
];
const draft = reactive<NetworkConfig>({ mode: 'system', proxyUrl: '' });
const busy = ref(false), testing = ref(false), loading = ref(true);
const message = ref(''), failed = ref(false);
const target = ref('https://api.openai.com/v1/models');

function selectMode(value: string | number | null) {
  if (value === 'system' || value === 'direct' || value === 'manual') draft.mode = value;
}
onMounted(async () => {
  try {
    if (!desktop) return;
    const config = await invoke<NetworkConfig>('get_network_config');
    draft.mode = config.mode;
    draft.proxyUrl = config.proxyUrl;
  } catch (error) {
    failed.value = true;
    message.value = String(error);
  } finally { loading.value = false; }
});
async function save() {
  busy.value = true; message.value = '';
  try {
    const config = await invoke<NetworkConfig>('save_network_config', { mode: draft.mode, proxyUrl: draft.proxyUrl });
    draft.mode = config.mode; draft.proxyUrl = config.proxyUrl;
    failed.value = false;
    message.value = '网络设置已保存，后续新建的连接将使用此模式。';
  } catch (error) { failed.value = true; message.value = String(error); }
  finally { busy.value = false; }
}
async function test() {
  testing.value = true; message.value = '';
  try {
    const result = await invoke<string>('test_network_connection', { url: target.value });
    failed.value = false; message.value = result;
  } catch (error) { failed.value = true; message.value = String(error); }
  finally { testing.value = false; }
}
</script>
<template>
  <div class="network-settings" data-testid="settings-network">
    <PhoneLinkCard />
    <div class="xb-group-title">网络连接方式</div>
    <div class="xb-group">
      <div class="xb-row">
        <div class="xb-row-text"><b>代理模式</b><span>选择小伴内置网络请求的连接方式</span></div>
        <NSelect  :value="draft.mode" @update:value="selectMode" aria-label="代理模式" :options="options" style="width:160px" :disabled="loading || busy || !desktop" />
      </div>
      <div v-if="draft.mode === 'manual'" class="xb-row proxy-url-row">
        <div class="xb-row-text"><b>代理服务器</b><span>支持 HTTP、HTTPS；示例：<code>http://127.0.0.1:7890</code></span></div>
        <input v-model.trim="draft.proxyUrl" class="net-input" aria-label="代理服务器地址" placeholder="http://127.0.0.1:7890" autocomplete="off" spellcheck="false" :disabled="loading || busy || !desktop" />
        <small>暂不支持 SOCKS 或包含账号密码的代理地址。</small>
      </div>
      <div class="xb-row">
        <div class="xb-row-text"><b>应用更改</b><span>不会中断已经开始的对话；新连接立即使用修改后的配置</span></div>
        <button type="button" class="xb-btn is-sm" data-testid="network-save" :disabled="!desktop || loading || busy" @click="save">{{ busy ? '保存中…' : '保存设置' }}</button>
      </div>
    </div>
    <div class="xb-group-title">连接诊断</div>
    <div class="xb-group">
      <div class="xb-row proxy-url-row">
        <div class="xb-row-text"><b>测试访问地址</b><span>使用已保存的代理设置发送请求；HTTP 401 也表示连接成功</span></div>
        <input v-model.trim="target" class="net-input" aria-label="测试地址" autocomplete="off" spellcheck="false" placeholder="https://api.openai.com/v1/models" :disabled="testing || !desktop" />
        <button type="button" class="xb-btn is-sm" data-testid="network-test" :disabled="!desktop || testing || busy || loading" @click="test">{{ testing ? '检测中…' : '测试连接' }}</button>
      </div>
    </div>
    <NAlert v-if="message" :type="failed ? 'error' : 'success'" :show-icon="false" style="margin-top:12px">{{ message }}</NAlert>
    <NAlert v-if="!desktop" type="info" :show-icon="false" style="margin-top:12px">请在桌面应用中管理网络配置。</NAlert>
    <p class="network-notice">自动模式沿用 reqwest 检测到的环境代理；如需使用本机代理软件，请选择手动代理并填写 HTTP 监听端口。手动代理优先于系统代理；直连将显式禁用代理。此设置适用于接入的原生 HTTP 客户端，不会修改浏览器 WebView、Shell 命令或第三方程序的网络配置。</p>
  </div>
</template>
<style scoped>
.proxy-url-row{flex-wrap:wrap;align-items:flex-start;gap:9px}
.proxy-url-row .xb-row-text{width:100%;flex-basis:100%}
.net-input{width:100%;min-width:0;box-sizing:border-box;height:36px;padding:0 10px;border-radius:9px;background:var(--ui-surface-raised);border:1px solid var(--ui-line);color:var(--ui-text-primary);font:inherit;outline:none}
.net-input:focus{border-color:var(--ui-btn)}
.proxy-url-row small{color:var(--ui-text-tertiary);font-size:11.5px}
.network-notice{font-size:11.5px;line-height:1.7;color:var(--ui-text-tertiary);padding:8px 2px}
</style>