<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { NAlert, NSelect, NSlider, NSwitch } from 'naive-ui';
import type { ControlStore } from '../control-types.ts';
import AsrStatus from './AsrStatus.vue';
import { MICROPHONE_UNSUPPORTED } from '../asr/capture.ts';
const props = defineProps<{ store: ControlStore }>();
const speech = props.store.speech;
const asr = props.store.asr;
const microphoneHint = computed(() => asr.microphoneSupported === false ? MICROPHONE_UNSUPPORTED : asr.microphoneError);
const disabled = computed(() => !props.store.desktop || !speech.ready);
const options = computed(() => {
  const voices = speech.voices.some(v => v.shortName === speech.settings.voice) ? speech.voices
    : [...speech.voices, { shortName: speech.settings.voice, friendlyName: speech.settings.voice, locale: '', gender: '' }];
  return [...voices].sort((a, b) => Number(!a.locale.startsWith('zh')) - Number(!b.locale.startsWith('zh')))
    .map(v => ({ value: v.shortName, label: v.friendlyName + (v.locale ? ' · ' + v.locale : '') }));
});
const sliders = [{ key: 'rate' as const, label: '语速', min: -50, max: 100, suffix: '%' }, { key: 'volume' as const, label: '音量', min: 0, max: 100, suffix: '%' }, { key: 'pitch' as const, label: '音调', min: -50, max: 50, suffix: 'Hz' }];
onMounted(() => { void speech.init(); void asr.init(); });
</script>
<template>
  <section class="voice-settings" data-testid="voice-settings">
    <NAlert v-if="!store.desktop" type="info" :show-icon="false" style="margin-bottom:12px">请在桌面应用中设置声音。</NAlert>
    <div class="xb-group-title">语音识别</div>
    <div class="xb-group">
      <div class="xb-row"><div class="xb-row-text"><b>语音识别</b><span>开启本地识别能力，录音需在聊天中手动点击麦克风。首次开启从魔搭下载约 229 MiB 模型，关闭后保留。</span></div>
        <NSwitch :value="asr.enabled" :disabled="!store.desktop || !asr.ready || asr.saving" :loading="asr.saving" aria-label="语音识别" data-testid="asr-settings-switch" @update:value="value => asr.setEnabled(value)" /></div>
      <div v-if="store.desktop && microphoneHint" class="xb-row"><NAlert type="warning" :show-icon="false" style="flex:1" data-testid="microphone-status">{{ microphoneHint }} 请在系统声音设置中检查输入设备；连接或切换麦克风后可回到聊天重试。</NAlert></div>
      <div class="xb-row asr-row"><AsrStatus :asr="asr" /></div>
    </div>

    <div class="xb-group-title">朗读</div>
    <div class="xb-group">
      <div class="xb-row"><div class="xb-row-text"><b>自动朗读回复</b><span>开启后，当前聊天的完整回复文本会发送给 Edge 在线合成；切换会话或发送新消息会停止朗读。</span></div>
        <NSwitch :value="speech.settings.autoRead" :disabled="disabled" :loading="speech.saving" aria-label="自动朗读回复" @update:value="value => speech.save({ autoRead: value })" /></div>
      <div class="xb-row"><div class="xb-row-text"><b>音色</b><span>内置常用中文声音，也可以刷新完整音色列表。</span></div>
        <div class="voice-picker"><NSelect :value="speech.settings.voice" :options="options" filterable :disabled="disabled" @update:value="value => speech.save({ voice: value })" />
          <button type="button" class="xb-btn is-sm" :disabled="disabled || speech.loadingVoices" @click="speech.loadVoices()">{{ speech.loadingVoices ? '刷新中…' : '刷新音色' }}</button></div></div>
      <div v-for="item in sliders" :key="item.key" class="xb-row">
        <div class="xb-row-text"><b>{{ item.label }}</b><span>{{ speech.settings[item.key] }}{{ item.suffix }}</span></div>
        <NSlider class="voice-slider" :value="speech.settings[item.key]" :min="item.min" :max="item.max" :disabled="disabled" :aria-label="item.label" @update:value="value => speech.save({ [item.key]: value })" />
      </div>
      <div class="xb-row"><div class="xb-row-text"><b>试听</b><span>用当前音色、语速、音量和音调朗读一句话。</span></div>
        <button v-if="speech.status !== 'idle'" type="button" class="xb-btn is-sm" @click="speech.stop()">停止朗读</button>
        <button type="button" class="xb-btn is-sm" :disabled="disabled || speech.saving" @click="speech.speak('你好，我是小伴。愿你的每一天，都有一点温暖的陪伴。', 'voice-preview')">{{ speech.status === 'synthesizing' && speech.currentKey === 'voice-preview' ? '合成中…' : '试听声音' }}</button></div>
      <div v-if="speech.error" class="xb-row"><NAlert type="warning" :show-icon="false" style="flex:1">{{ speech.error }}</NAlert></div>
    </div>
  </section>
</template>
<style scoped>
.voice-picker{display:flex;align-items:center;gap:10px;width:min(65%,380px)}.voice-picker :deep(.n-select){flex:1;min-width:0}
.voice-slider{width:min(45%,240px);flex-shrink:0}
.asr-row:not(:has(*)){display:none}
</style>
