<script setup lang="ts">
import { ref } from 'vue';
import { NAlert, NSelect, NSwitch } from 'naive-ui';
import { FolderOpen } from 'lucide-vue-next';
import { invoke } from '../../../bridge.ts';
import type { ControlStore } from '../../control-types.ts';
const props = defineProps<{ store: ControlStore }>();
const busy = ref('');
async function act(key: string, action: () => Promise<unknown>, success?: string) {
  busy.value = key;
  try { await action(); if (success) props.store.notify(success); } catch (error) { props.store.notify(error, 'error'); } finally { busy.value = ''; }
}
const fpsOptions = [{ label: '30 帧 · 节能', value: 30 }, { label: '60 帧 · 流畅', value: 60 }];
</script>
<template>
  <div class="settings-general" data-testid="settings-general">
    <div class="xb-group-title">启动</div>
    <div class="xb-group">
      <div class="xb-row"><div class="xb-row-text"><b>开机自动启动</b><span>{{ store.startup.available ? '登录系统后在后台启动，恢复上次的桌面，不自动弹出窗口' : '请在正式桌面应用中设置' }}</span></div>
        <NSwitch :value="store.settings.autostart" aria-label="开机自动启动" :disabled="!store.startup.available || !!busy" :loading="busy === 'startup'" @update:value="value => act('startup', () => store.setAutostart(value))" /></div>
      <div v-if="store.startup.error" class="xb-row"><NAlert type="warning" :show-icon="false" style="flex:1">{{ store.startup.error }}</NAlert></div>
    </div>
    <div class="xb-group-title">桌面动画</div>
    <div class="xb-group">
      <div class="xb-row"><div class="xb-row-text"><b>暂停桌面动画</b><span>暂停伙伴与动态壁纸，不影响提醒计时</span></div>
        <NSwitch :value="store.settings.paused" aria-label="暂停桌面动画" :disabled="!!busy" :loading="busy === 'pause'" @update:value="value => act('pause', () => store.updateAppearance({ paused: value }))" /></div>
      <div class="xb-row"><div class="xb-row-text"><b>动画帧率</b><span>30 帧更省资源，60 帧更流畅</span></div>
        <NSelect style="width:150px" :value="store.settings.fps" :options="fpsOptions" :disabled="!!busy" @update:value="value => act('fps', () => store.updateAppearance({ fps: value }))" /></div>
    </div>
    <div class="xb-group-title">本地数据</div>
    <div class="xb-group">
      <div class="xb-row"><div class="xb-row-text"><b>数据位置</b><span class="settings-path">{{ store.info.dataDirectory || '桌面应用中可查看实际保存位置' }}</span></div>
        <button type="button" class="xb-btn is-sm" :disabled="!store.desktop" @click="act('directory', () => invoke('open_data_directory'))"><FolderOpen :size="14" />打开目录</button></div>
    </div>
  </div>
</template>
<style scoped>
.settings-path{font-family:var(--ui-font-mono);font-size:11.5px!important;overflow-wrap:anywhere}
</style>
