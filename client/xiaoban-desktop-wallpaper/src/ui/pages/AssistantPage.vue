<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { NAlert, NButton, NModal, NProgress, NSelect, NSwitch } from 'naive-ui';
import { Play, RotateCcw } from 'lucide-vue-next';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';

interface Status {
  enabled: boolean; shortcut: string; shortcutLabel: string; shortcuts: { value: string; label: string }[];
  ball: boolean; open: 'voice' | 'type'; read: 'voice' | 'always' | 'never'; hintSeen: boolean; registered: boolean; error: string;
}
const props = withDefaults(defineProps<{ store: ControlStore; embedded?: boolean }>(), { embedded: false });
const status = ref<Status>({
  enabled: props.store.settings.assistantEnabled, shortcut: props.store.settings.assistantShortcut, shortcutLabel: props.store.settings.assistantShortcut,
  shortcuts: [], ball: props.store.settings.assistantBall, open: props.store.settings.assistantOpen, read: props.store.settings.assistantRead,
  hintSeen: true, registered: false, error: '',
});
const busy = ref('');
const isMacOS = computed(() => props.store.info?.platform === 'macos');
const repairConfirm = ref(false);
const repairBusy = ref(false);
const permissionResetDone = ref(false);
const asr = computed(() => props.store.asr);
const shortcutOptions = computed(() => status.value.shortcuts.map(item => ({ label: item.label, value: item.value })));
const openOptions = [{ label: '直接开始听', value: 'voice' }, { label: '先打字', value: 'type' }];
const readOptions = [{ label: '仅语音提问时', value: 'voice' }, { label: '每次回答都朗读', value: 'always' }, { label: '不朗读', value: 'never' }];
const percent = computed(() => Math.min(100, Math.floor(asr.value.downloaded / Math.max(1, asr.value.total) * 100)));

/** One line about the speech engine, in the words a person would use. */
const voiceLine = computed(() => {
  const voice = asr.value;
  if (voice.error) return voice.error;
  if (!voice.enabled) {
    if (voice.modelReady) return '开启后可以直接说话提问。语音在本机识别，模型已经准备好了。';
    return voice.warming ? '开启后可以直接说话提问。模型（约 229 MB）正在后台下载，完成后开启即可立即使用。' : '开启后可以直接说话提问。语音在本机识别，模型（约 229 MB）会在后台自动下载。';
  }
  if (voice.preparing) return voice.phase === 'downloading' ? '正在下载语音模型，下载期间仍可打字使用。' : '正在加载语音引擎…';
  if (voice.warming) return '正在后台准备语音模型，完成前也可以打字使用。';
  if (voice.engineReady) return '语音已就绪，呼出后可以直接说话。';
  if (!voice.modelReady) return '语音模型还没下载，稍后会在后台自动下载。';
  return '语音模型已下载，正在准备。';
});
const voiceBusy = computed(() => asr.value.preparing || asr.value.warming);
const autostartOff = computed(() => props.store.desktop && !props.store.settings.autostart);

async function read() {
  if (!props.store.desktop) return;
  try { status.value = await invoke<Status>('assistant_status'); } catch { /* the saved settings stand in */ }
}
async function change(patch: Record<string, unknown>) {
  busy.value = 'assistant';
  try { status.value = await invoke<Status>('assistant_configure', { patch }); await props.store.refreshSettings(); }
  catch (error) { props.store.notify(error, 'error'); await read(); }
  finally { busy.value = ''; }
}
async function tryNow() {
  try { await invoke('assistant_try'); } catch (error) { props.store.notify(error, 'error'); }
}
async function setVoice(enabled: boolean) {
  try { await asr.value.setEnabled(enabled); } catch (error) { props.store.notify(error, 'error'); }
}
function openVoiceSettings() {
  props.store.settingsTab = 'voice';
  props.store.navigate('settings');
}
async function openCaptureSettings() {
  try { await invoke('open_screen_capture_settings'); }
  catch (error) { props.store.notify(error, 'error'); }
}
async function resetCapturePermission() {
  if (repairBusy.value) return;
  repairBusy.value = true;
  try {
    await invoke('reset_screen_capture_permission');
    permissionResetDone.value = true;
    repairConfirm.value = false;
    props.store.notify('已清除小伴的截图权限。请从托盘完全退出并重新打开小伴，再次截图时重新授权。');
  } catch (error) { props.store.notify(error, 'error'); }
  finally { repairBusy.value = false; }
}
onMounted(() => { void read(); void asr.value.init(); });
</script>

<template>
  <section class="assistant-page" :class="{ 'xb-page': !embedded, 'is-embedded': embedded }" data-testid="assistant-page">
    <header v-if="!embedded" class="xb-page-head"><div><h1 class="xb-page-title">超级小伴</h1></div></header>

    <div class="xb-group hero" data-testid="assistant-hero">
      <div class="xb-row">
        <div class="xb-row-text"><b>启用超级小伴</b><span>{{ status.enabled ? '随时呼出，说话或打字都可以。' : '开启后，可通过快捷键或悬浮球在任何应用里呼出，截图提问、语音对话。' }}</span></div>
        <span v-if="status.enabled && status.registered" class="xb-ok" data-testid="assistant-registered">快捷键已生效</span>
        <button type="button" class="xb-btn is-sm" :disabled="!store.desktop || !status.enabled" data-testid="assistant-try" @click="tryNow"><Play :size="13" />试一下</button>
        <NSwitch :value="status.enabled" :disabled="!store.desktop || busy === 'assistant'" aria-label="启用超级小伴" data-testid="assistant-enabled" @update:value="change({ enabled: $event })" />
      </div>
    </div>

    <NAlert v-if="status.enabled && status.error" type="warning" :show-icon="false" class="note" data-testid="assistant-shortcut-error">{{ status.error }}</NAlert>
    <NAlert v-if="autostartOff" type="info" :show-icon="false" class="note" data-testid="assistant-autostart-note">
      目前没有开启开机自启。重启电脑后，需要先打开小伴，快捷键和悬浮球才会出现。
      <button type="button" class="xb-link" @click="store.settingsTab = 'general'; store.navigate('settings')">去打开</button>
    </NAlert>

    <div class="xb-group-title">呼出方式</div>
    <div class="xb-group">
      <div class="xb-row"><div class="xb-row-text"><b>呼出快捷键</b><span>短按呼出或收起；按住说话，松开发送。与其他应用冲突时可更换组合。</span></div>
        <kbd class="xb-kbd" aria-label="当前快捷键" data-testid="assistant-shortcut-label">{{ status.shortcutLabel }}</kbd>
        <NSelect class="select" :value="status.shortcut" :options="shortcutOptions" :disabled="!store.desktop || !status.enabled || busy === 'assistant'" aria-label="呼出快捷键" data-testid="assistant-shortcut" @update:value="change({ shortcut: $event })" /></div>
      <div class="xb-row"><div class="xb-row-text"><b>悬浮球</b><span>点击展开，按住拖动。关闭后用快捷键呼出。</span></div>
        <NSwitch :value="status.ball" :disabled="!store.desktop || !status.enabled || busy === 'assistant'" aria-label="屏幕边缘的悬浮球" data-testid="assistant-ball-switch" @update:value="change({ ball: $event })" /></div>
      <div class="xb-row"><div class="xb-row-text"><b>悬浮位置</b><span>将悬浮球与对话框恢复到初始位置。</span></div>
        <button type="button" class="xb-btn is-sm" :disabled="!store.desktop || !status.enabled || busy === 'assistant'" data-testid="assistant-reset" @click="change({ resetPosition: true })"><RotateCcw :size="14" />重置位置</button></div>
    </div>

    <div class="xb-group-title">语音对话</div>
    <div class="xb-group">
      <div class="xb-row"><div class="xb-row-text"><b>用语音提问</b><span data-testid="assistant-voice-line" :class="{ problem: asr.error }">{{ voiceLine }}</span><NProgress v-if="asr.preparing && asr.phase === 'downloading'" class="progress" type="line" :percentage="percent" :height="4" :show-indicator="false" /></div>
        <button v-if="asr.error && asr.enabled" type="button" class="xb-btn is-sm" :disabled="asr.saving || voiceBusy" data-testid="assistant-voice-retry" @click="asr.retry()">重试</button>
        <NSwitch :value="asr.enabled" :disabled="!store.desktop || asr.saving" aria-label="用语音提问" data-testid="assistant-voice" @update:value="setVoice" /></div>
      <div class="xb-row"><div class="xb-row-text"><b>呼出后</b><span>语音未就绪时，自动切换为打字。</span></div>
        <NSelect class="select" :value="status.open" :options="openOptions" :disabled="!store.desktop || busy === 'assistant'" aria-label="呼出后" data-testid="assistant-open" @update:value="change({ open: $event })" /></div>
      <div class="xb-row"><div class="xb-row-text"><b>朗读回答</b><span>使用“声音”设置里选的音色。<button type="button" class="xb-link" @click="openVoiceSettings">去选音色</button></span></div>
        <NSelect class="select" :value="status.read" :options="readOptions" :disabled="!store.desktop || busy === 'assistant'" aria-label="朗读回答" data-testid="assistant-read" @update:value="change({ read: $event })" /></div>
    </div>

    <template v-if="isMacOS">
      <div class="xb-group-title">屏幕截图权限</div>
      <div class="xb-group capture-settings" data-testid="assistant-capture-settings">
        <div class="xb-row">
          <div class="xb-row-text"><b>截图失败时</b><span>更新后若系统仍显示已授权、截图却失败，可重置小伴自己的屏幕录制授权记录，再重新授权。</span></div>
          <div class="capture-actions">
            <button type="button" class="xb-btn is-sm" :disabled="!store.desktop || repairBusy" data-testid="assistant-capture-open-settings" @click="openCaptureSettings">打开系统设置</button>
            <button type="button" class="xb-btn is-sm" :disabled="!store.desktop || repairBusy" data-testid="assistant-capture-reset-tcc" @click="repairConfirm = true"><RotateCcw :size="14" />修复截图权限</button>
          </div>
        </div>
        <div v-if="permissionResetDone" class="xb-row"><NAlert type="info" :show-icon="false" style="flex:1" data-testid="assistant-capture-reset-notice">
          已清除小伴的截图授权。请从托盘完全退出小伴，再从「应用程序」重新打开并允许系统授权。若仍失败，请在系统设置中手动移除旧的小伴记录。
        </NAlert></div>
      </div>
    </template>
    <NModal v-model:show="repairConfirm" preset="card" title="重置小伴的截图权限" :mask-closable="!repairBusy" :closable="!repairBusy" :style="{ width: '420px', maxWidth: 'calc(100vw - 32px)' }">
      <p class="repair-message">这会运行系统命令 <code>tccutil reset ScreenCapture xyz.xiaban.desktop-wallpaper</code>，只清除小伴的屏幕录制授权，不会更改其他应用的权限。</p>
      <p class="repair-message">清除后需要从托盘完全退出并重新打开小伴，再次截图并允许授权。此操作不能直接修复截图坐标或其他截图错误。</p>
      <div class="repair-actions">
        <NButton :disabled="repairBusy" data-testid="assistant-capture-reset-cancel" @click="repairConfirm = false">取消</NButton>
        <NButton type="primary" :loading="repairBusy" data-testid="assistant-capture-reset-confirm" @click="resetCapturePermission">确认清除</NButton>
      </div>
    </NModal>
  </section>
</template>

<style scoped>
.assistant-page{max-width:920px;margin:0 auto;color:var(--ui-text-primary)}
.assistant-page.is-embedded{max-width:none;margin:0;padding:0}
.note{margin-top:12px}
.xb-kbd{flex-shrink:0}
.select{width:180px;flex:0 0 auto}
.xb-row-text .problem{color:var(--ui-warning-text)}
.xb-row-text .xb-link{margin-left:6px;font-size:12px}
.progress{margin-top:8px;max-width:320px}
.capture-actions{display:flex;align-items:center;justify-content:flex-end;gap:8px;flex-wrap:wrap;flex-shrink:0}
.repair-message{font-size:13px;line-height:1.8;color:var(--ui-text-secondary);margin:0 0 12px}
.repair-message code{font-size:11px;word-break:break-all}
.repair-actions{display:flex;justify-content:flex-end;gap:8px;margin-top:20px}
</style>
