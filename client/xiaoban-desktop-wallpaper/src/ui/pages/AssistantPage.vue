<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { NAlert, NButton, NModal, NProgress, NSelect, NSwitch } from 'naive-ui';
import { AudioLines, Mic, Play, RotateCcw, ScanLine } from 'lucide-vue-next';
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
  <section class="assistant-page" :class="{ 'ui-page': !embedded, 'is-embedded': embedded }" data-testid="assistant-page">
    <header class="page-header"><div><h1>超级小伴</h1><p class="header-note">快捷呼出与语音偏好</p></div><NButton round size="small" :disabled="!store.desktop || !status.enabled" data-testid="assistant-try" @click="tryNow"><template #icon><Play :size="13" /></template>试一下</NButton></header>

    <section class="surface hero" data-testid="assistant-hero">
      <div class="assistant-mark" aria-hidden="true"><AudioLines :size="23" :stroke-width="1.6" /></div>
      <div class="hero-copy"><h2>启用超级小伴</h2><p>{{ status.enabled ? '随时呼出，说话或打字都可以。' : '开启后，可通过快捷键或悬浮球呼出。' }}</p></div>
      <div class="hero-meta"><kbd class="keycap" aria-label="当前快捷键" data-testid="assistant-shortcut-label">{{ status.shortcutLabel }}</kbd><span v-if="status.enabled && status.registered" class="chip ok" data-testid="assistant-registered"><i aria-hidden="true"></i>快捷键已生效</span></div>
      <NSwitch :value="status.enabled" :disabled="!store.desktop || busy === 'assistant'" aria-label="启用超级小伴" data-testid="assistant-enabled" @update:value="change({ enabled: $event })" />
    </section>

    <NAlert v-if="status.enabled && status.error" type="warning" :show-icon="false" data-testid="assistant-shortcut-error">{{ status.error }}</NAlert>
    <NAlert v-if="autostartOff" type="info" :show-icon="false" data-testid="assistant-autostart-note">
      目前没有开启开机自启。重启电脑后，需要先打开小伴，快捷键和悬浮球才会出现。
      <NButton text type="primary" @click="store.settingsTab = 'general'; store.navigate('settings')">去打开</NButton>
    </NAlert>

    <div class="settings-grid">
    <section class="surface settings-group">
      <div class="section-heading"><AudioLines :size="17" :stroke-width="1.7" /><h2>呼出方式</h2></div>
      <div class="setting-row"><div><b>悬浮球</b><p>点击展开，按住拖动。关闭后用快捷键呼出。</p></div><NSwitch :value="status.ball" :disabled="!store.desktop || !status.enabled || busy === 'assistant'" aria-label="屏幕边缘的悬浮球" data-testid="assistant-ball-switch" @update:value="change({ ball: $event })" /></div>
      <div class="setting-row"><div><b>呼出快捷键</b><p>与其他应用冲突时，可更换组合。</p></div><NSelect class="select" :value="status.shortcut" :options="shortcutOptions" :disabled="!store.desktop || !status.enabled || busy === 'assistant'" aria-label="呼出快捷键" data-testid="assistant-shortcut" @update:value="change({ shortcut: $event })" /></div>
      <div class="setting-row"><div><b>悬浮位置</b><p>将悬浮球与对话框恢复到初始位置。</p></div><NButton round :disabled="!store.desktop || !status.enabled || busy === 'assistant'" data-testid="assistant-reset" @click="change({ resetPosition: true })"><template #icon><RotateCcw :size="14" /></template>重置位置</NButton></div>
    </section>

    <section class="surface settings-group">
      <div class="section-heading"><Mic :size="17" :stroke-width="1.7" /><h2>语音对话</h2></div>
      <div class="setting-row"><div><b>用语音提问</b><p data-testid="assistant-voice-line" :class="{ problem: asr.error }">{{ voiceLine }}</p><NProgress v-if="asr.preparing && asr.phase === 'downloading'" class="progress" type="line" :percentage="percent" :height="4" :show-indicator="false" /></div><div class="voice-actions"><NButton v-if="asr.error && asr.enabled" size="small" round :disabled="asr.saving || voiceBusy" data-testid="assistant-voice-retry" @click="asr.retry()">重试</NButton><NSwitch :value="asr.enabled" :disabled="!store.desktop || asr.saving" aria-label="用语音提问" data-testid="assistant-voice" @update:value="setVoice" /></div></div>
      <div class="setting-row"><div><b>呼出后</b><p>语音未就绪时，自动切换为打字。</p></div><NSelect class="select wide" :value="status.open" :options="openOptions" :disabled="!store.desktop || busy === 'assistant'" aria-label="呼出后" data-testid="assistant-open" @update:value="change({ open: $event })" /></div>
      <div class="setting-row"><div><b>朗读回答</b><p>使用“声音”设置里选的音色。<NButton text type="primary" size="small" @click="openVoiceSettings">去选音色</NButton></p></div><NSelect class="select wide" :value="status.read" :options="readOptions" :disabled="!store.desktop || busy === 'assistant'" aria-label="朗读回答" data-testid="assistant-read" @update:value="change({ read: $event })" /></div>
    </section>

    <section v-if="isMacOS" class="surface settings-group capture-settings" data-testid="assistant-capture-settings">
      <div class="section-heading"><ScanLine :size="17" :stroke-width="1.7" /><h2>屏幕截图权限</h2></div>
      <div class="setting-row">
        <div><b>截图失败时</b><p>更新后若系统仍显示已授权、截图却失败，可重置小伴自己的屏幕录制授权记录，再重新授权。</p></div>
        <div class="capture-actions">
          <NButton size="small" round :disabled="!store.desktop || repairBusy" data-testid="assistant-capture-open-settings" @click="openCaptureSettings">打开系统设置</NButton>
          <NButton size="small" round :disabled="!store.desktop || repairBusy" data-testid="assistant-capture-reset-tcc" @click="repairConfirm = true"><template #icon><RotateCcw :size="14" /></template>修复截图权限</NButton>
        </div>
      </div>
      <NAlert v-if="permissionResetDone" type="info" :show-icon="false" data-testid="assistant-capture-reset-notice" class="capture-notice">
        已清除小伴的截图授权。请从托盘完全退出小伴，再从「应用程序」重新打开并允许系统授权。若仍失败，请在系统设置中手动移除旧的小伴记录。
      </NAlert>
    </section>
    </div>
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
.assistant-page{container-type:inline-size;display:grid;gap:16px;width:100%;max-width:920px;margin:0 auto;align-content:start;color:var(--ui-text-primary)}
.assistant-page.is-embedded{max-width:none;margin:0;padding:0}
.assistant-page .page-header{margin:0 0 2px;gap:16px;align-items:center}
.page-header h1{font-size:23px;letter-spacing:-.035em;font-weight:650}
.header-note{margin:5px 0 0;font-size:12px;line-height:1.5;color:var(--ui-text-secondary)}
.assistant-page .surface{border:1px solid color-mix(in srgb,var(--ui-border) 65%,transparent);border-radius:16px;box-shadow:0 2px 8px color-mix(in srgb,var(--ui-text-primary) 3%,transparent)}
.assistant-page .hero{display:flex;align-items:center;gap:14px;padding:20px 22px;background:linear-gradient(120deg,var(--ui-surface),color-mix(in srgb,var(--ui-surface-alt) 65%,var(--ui-surface)))}
.assistant-mark{display:grid;place-items:center;flex:none;width:44px;height:44px;border-radius:14px;background:var(--ui-surface);color:var(--ui-text-secondary);box-shadow:inset 0 0 0 1px var(--ui-border),0 2px 4px color-mix(in srgb,var(--ui-text-primary) 3%,transparent)}
.hero-copy{flex:1;min-width:0}
.hero h2{font-size:14px;line-height:1.5;font-weight:600;margin:0 0 4px}
.hero p{font-size:12px;line-height:1.6;color:var(--ui-text-secondary);margin:0}
.hero-meta{display:grid;justify-items:end;gap:7px;margin-right:6px;flex:none}
.chip{display:inline-flex;align-items:center;gap:5px;font-size:10px;line-height:1.4;color:var(--ui-text-secondary)}
.chip i{width:5px;height:5px;border-radius:50%;background:var(--ui-accent)}
.keycap{padding:7px 11px;border-radius:8px;font:500 12px/1 ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;white-space:nowrap;color:var(--ui-text-primary);background:var(--ui-surface);border:1px solid var(--ui-border);box-shadow:0 2px 0 var(--ui-border)}
.settings-grid{display:grid;gap:16px;align-items:start}
.settings-group{min-width:0;overflow:hidden;padding:0 20px}
.assistant-page .section-heading{justify-content:flex-start;gap:8px;margin:0;padding:16px 0 12px;border-bottom:1px solid var(--ui-border);color:var(--ui-text-secondary)}
.section-heading h2{font-size:13px;font-weight:600;margin:0;color:var(--ui-text-primary)}
.assistant-page .setting-row{gap:16px;min-height:0;padding:15px 0;border-bottom:1px solid color-mix(in srgb,var(--ui-border) 75%,transparent)}
.assistant-page .setting-row:last-child{border-bottom:0;padding-bottom:17px}
.setting-row>div:first-child{flex:1;min-width:0}
.setting-row b{font-size:13px;font-weight:500}
.setting-row p{font-size:11px;line-height:1.65;margin:4px 0 0;max-width:410px;color:var(--ui-text-secondary)}
.setting-row p.problem{color:var(--ui-primary-text)}
.select{width:180px;flex:0 0 auto}
.select.wide{width:180px}
.setting-row :deep(.n-button){font-size:12px;height:32px}
.setting-row :deep(.n-button--text-type){height:auto;font-size:11px;margin-left:4px}
.progress{margin-top:8px;max-width:320px}
.voice-actions{display:flex;align-items:center;gap:10px;flex:none}
.capture-settings{grid-column:1/-1}
.capture-actions{display:flex;align-items:center;justify-content:flex-end;gap:8px;flex-wrap:wrap}
.capture-notice{margin-bottom:16px}
.repair-message{font-size:13px;line-height:1.8;color:var(--ui-text-secondary);margin:0 0 12px}
.repair-message code{font-size:11px;word-break:break-all}
.repair-actions{display:flex;justify-content:flex-end;gap:8px;margin-top:20px}
@container(min-width:820px){.settings-grid{grid-template-columns:1fr 1fr}.setting-row{flex-wrap:wrap;column-gap:12px}.setting-row>.select{width:100%;margin-top:0}.setting-row:has(>.select){row-gap:10px}.setting-row:has(>.select)>div:first-child{flex-basis:100%}}
@container(max-width:520px){.assistant-page .hero{flex-wrap:wrap;padding:16px;gap:12px}.hero-copy{flex-basis:calc(100% - 60px)}.hero-meta{margin-left:56px;margin-right:auto;justify-items:start;display:flex;align-items:center;gap:10px}.settings-group{padding:0 16px}.setting-row{flex-wrap:wrap;row-gap:10px}.setting-row>.select{width:100%}.setting-row:has(>.select)>div:first-child{flex-basis:100%}}
</style>
