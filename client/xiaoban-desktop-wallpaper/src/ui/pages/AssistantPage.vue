<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { NAlert, NButton, NProgress, NSelect, NSwitch } from 'naive-ui';
import { AudioLines, Mic, Play, RotateCcw } from 'lucide-vue-next';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';

interface Status {
  enabled: boolean; shortcut: string; shortcutLabel: string; shortcuts: { value: string; label: string }[];
  ball: boolean; open: 'voice' | 'type'; read: 'voice' | 'always' | 'never'; hintSeen: boolean; registered: boolean; error: string;
}
const props = defineProps<{ store: ControlStore }>();
const status = ref<Status>({
  enabled: props.store.settings.assistantEnabled, shortcut: props.store.settings.assistantShortcut, shortcutLabel: props.store.settings.assistantShortcut,
  shortcuts: [], ball: props.store.settings.assistantBall, open: props.store.settings.assistantOpen, read: props.store.settings.assistantRead,
  hintSeen: true, registered: false, error: '',
});
const busy = ref('');
const asr = computed(() => props.store.asr);
const shortcutOptions = computed(() => status.value.shortcuts.map(item => ({ label: item.label, value: item.value })));
const openOptions = [{ label: '直接开始听', value: 'voice' }, { label: '先打字，需要时点光球再说', value: 'type' }];
const readOptions = [{ label: '只在我用语音提问时朗读', value: 'voice' }, { label: '每次回答都朗读', value: 'always' }, { label: '不朗读', value: 'never' }];
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
onMounted(() => { void read(); void asr.value.init(); });
</script>

<template>
  <section class="ui-page assistant-page" data-testid="assistant-page">
    <header class="page-header"><h1>悬浮助手</h1><span class="header-note">随时呼出，直接对话</span></header>

    <section class="surface hero" data-testid="assistant-hero">
      <div class="hero-copy">
        <h2>在任何应用里，按下快捷键就能向小伴提问</h2>
        <p>说一句话或打几个字，答案就浮在屏幕上，再按一下收起。需要长谈时，点“↗”把这一轮带到完整聊天。</p>
        <div class="hero-actions">
          <NButton type="primary" round :disabled="!store.desktop || !status.enabled" data-testid="assistant-try" @click="tryNow"><template #icon><Play :size="15" /></template>现在试一下</NButton>
          <span v-if="status.enabled && status.registered" class="chip ok" data-testid="assistant-registered">快捷键已生效</span>
        </div>
      </div>
      <div class="keycap" aria-label="当前快捷键" data-testid="assistant-shortcut-label">{{ status.shortcutLabel }}</div>
    </section>

    <NAlert v-if="status.enabled && status.error" type="warning" :show-icon="false" data-testid="assistant-shortcut-error">{{ status.error }}</NAlert>
    <NAlert v-if="autostartOff" type="info" :show-icon="false" data-testid="assistant-autostart-note">
      目前没有开启开机自启。重启电脑后，需要先打开小伴，快捷键和悬浮球才会出现。
      <NButton text type="primary" @click="store.settingsTab = 'general'; store.navigate('settings')">去打开</NButton>
    </NAlert>

    <section class="surface">
      <div class="section-heading"><div><h2>呼出方式</h2><p>怎么找到它、怎么叫出它</p></div><AudioLines :size="20" /></div>
      <div class="setting-row"><div><b>启用悬浮助手</b><p>关闭后，快捷键和悬浮球都会消失。</p></div><NSwitch :value="status.enabled" :disabled="!store.desktop || busy === 'assistant'" aria-label="启用悬浮助手" data-testid="assistant-enabled" @update:value="change({ enabled: $event })" /></div>
      <div class="setting-row"><div><b>屏幕边缘的悬浮球</b><p>常驻一个小球，点一下展开，按住可以拖到喜欢的位置。关闭后只能用快捷键呼出。</p></div><NSwitch :value="status.ball" :disabled="!store.desktop || !status.enabled || busy === 'assistant'" aria-label="屏幕边缘的悬浮球" data-testid="assistant-ball-switch" @update:value="change({ ball: $event })" /></div>
      <div class="setting-row"><div><b>呼出快捷键</b><p>和别的应用冲突时，换一个组合就行。</p></div><NSelect class="select" :value="status.shortcut" :options="shortcutOptions" :disabled="!store.desktop || !status.enabled || busy === 'assistant'" aria-label="呼出快捷键" data-testid="assistant-shortcut" @update:value="change({ shortcut: $event })" /></div>
      <div class="setting-row"><div><b>位置</b><p>把悬浮球和对话框放回默认的位置。</p></div><NButton round :disabled="!store.desktop || !status.enabled || busy === 'assistant'" data-testid="assistant-reset" @click="change({ resetPosition: true })"><template #icon><RotateCcw :size="14" /></template>放回默认位置</NButton></div>
    </section>

    <section class="surface">
      <div class="section-heading"><div><h2>语音对话</h2><p>呼出后直接说话，回答可以读给你听</p></div><Mic :size="20" /></div>
      <div class="setting-row"><div><b>用语音提问</b><p data-testid="assistant-voice-line" :class="{ problem: asr.error }">{{ voiceLine }}</p><NProgress v-if="asr.preparing && asr.phase === 'downloading'" class="progress" type="line" :percentage="percent" :height="4" :show-indicator="false" /></div><div class="voice-actions"><NButton v-if="asr.error && asr.enabled" size="small" round :disabled="asr.saving || voiceBusy" data-testid="assistant-voice-retry" @click="asr.retry()">重试</NButton><NSwitch :value="asr.enabled" :disabled="!store.desktop || asr.saving" aria-label="用语音提问" data-testid="assistant-voice" @update:value="setVoice" /></div></div>
      <div class="setting-row"><div><b>呼出后</b><p>语音关闭或还没准备好时，会自动改成打字。</p></div><NSelect class="select wide" :value="status.open" :options="openOptions" :disabled="!store.desktop || busy === 'assistant'" aria-label="呼出后" data-testid="assistant-open" @update:value="change({ open: $event })" /></div>
      <div class="setting-row"><div><b>朗读回答</b><p>使用“声音”设置里选的音色。<NButton text type="primary" size="small" @click="openVoiceSettings">去选音色</NButton></p></div><NSelect class="select wide" :value="status.read" :options="readOptions" :disabled="!store.desktop || busy === 'assistant'" aria-label="朗读回答" data-testid="assistant-read" @update:value="change({ read: $event })" /></div>
    </section>
  </section>
</template>

<style scoped>
.assistant-page{display:grid;gap:var(--ui-section-gap,24px);max-width:860px;margin:0 auto;align-content:start}
.header-note{font-size:12px;color:var(--ui-text-tertiary)}
.hero{display:flex;align-items:center;justify-content:space-between;gap:28px;padding:28px 32px}
.hero-copy{max-width:470px}
.hero h2{font-size:19px;line-height:1.5;margin:0 0 10px}
.hero p{font-size:13px;line-height:1.8;color:var(--ui-text-secondary);margin:0 0 18px}
.hero-actions{display:flex;align-items:center;gap:14px;flex-wrap:wrap}
.chip{font-size:12px;padding:3px 10px;border-radius:12px}
.chip.ok{color:var(--ui-accent);background:color-mix(in srgb,var(--ui-accent) 14%,transparent)}
.keycap{flex:none;min-width:150px;text-align:center;padding:22px 26px;border-radius:18px;font:600 28px/1 ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;letter-spacing:.06em;color:var(--ui-text-primary);background:var(--ui-surface-alt);border:1px solid var(--ui-border);border-bottom-width:4px}
.select{width:200px}
.select.wide{width:260px}
.setting-row p.problem{color:#e0556a}
.progress{margin-top:8px;max-width:320px}
.voice-actions{display:flex;align-items:center;gap:12px}
@media(max-width:760px){.hero{flex-direction:column;align-items:flex-start}.select,.select.wide{width:100%}}
</style>
