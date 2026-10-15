<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { NAlert, NButton, NSelect, NSlider, NSwitch } from 'naive-ui';
import type { ControlStore } from '../control-types.ts';
import AsrStatus from './AsrStatus.vue';
const props = defineProps<{ store: ControlStore }>();
const speech = props.store.speech;
const asr = props.store.asr;
const disabled = computed(() => !props.store.desktop || !speech.ready);
const options = computed(() => {
  const voices = speech.voices.some(v => v.shortName === speech.settings.voice) ? speech.voices
    : [...speech.voices, { shortName: speech.settings.voice, friendlyName: speech.settings.voice, locale: '', gender: '' }];
  return [...voices].sort((a, b) => Number(!a.locale.startsWith('zh')) - Number(!b.locale.startsWith('zh')))
    .map(v => ({ value: v.shortName, label: v.friendlyName + (v.locale ? ' · ' + v.locale : '') }));
});
onMounted(() => { void speech.init(); void asr.init(); });
</script>
<template>
  <section class="surface voice-settings" data-testid="voice-settings">
    <div class="section-heading"><div><h2>小伴的声音</h2><p>Edge TTS 在线合成语音，需联网，无需填写语音服务 Key。</p></div></div>
    <NAlert v-if="!store.desktop" type="info" :show-icon="false">请在桌面应用中设置声音。</NAlert>
    <div class="setting-row"><div><b>语音识别</b><p>开启本地识别能力，录音需在聊天中手动点击麦克风。首次开启从魔搭下载约 229 MiB 模型，关闭后保留。</p></div>
      <NSwitch :value="asr.enabled" :disabled="!store.desktop || !asr.ready || asr.saving" :loading="asr.saving" data-testid="asr-settings-switch" @update:value="value => asr.setEnabled(value)" />
    </div>
    <AsrStatus :asr="asr" />
    <div class="setting-row"><div><b>自动朗读回复</b><p>开启后，当前聊天的完整回复文本会发送给 Edge 在线合成；切换会话或发送新消息会停止朗读。</p></div>
      <NSwitch :value="speech.settings.autoRead" :disabled="disabled" :loading="speech.saving" @update:value="value => speech.save({ autoRead: value })" />
    </div>
    <div class="setting-row"><div><b>音色</b><p>内置常用中文声音，也可以刷新完整音色列表。</p></div>
      <div class="voice-picker"><NSelect :value="speech.settings.voice" :options="options" filterable :disabled="disabled" @update:value="value => speech.save({ voice: value })" />
        <NButton :disabled="disabled" :loading="speech.loadingVoices" @click="speech.loadVoices()">刷新音色</NButton></div>
    </div>
    <div v-for="item in [{ key: 'rate' as const, label: '语速', min: -50, max: 100, suffix: '%' }, { key: 'volume' as const, label: '音量', min: 0, max: 100, suffix: '%' }, { key: 'pitch' as const, label: '音调', min: -50, max: 50, suffix: 'Hz' }]" :key="item.key" class="setting-row">
      <div><b>{{ item.label }}</b><p>{{ speech.settings[item.key] }}{{ item.suffix }}</p></div>
      <NSlider style="width:min(45%,280px)" :value="speech.settings[item.key]" :min="item.min" :max="item.max" :disabled="disabled" @update:value="value => speech.save({ [item.key]: value })" />
    </div>
    <div class="settings-actions">
      <NButton :disabled="disabled || speech.saving" :loading="speech.status === 'synthesizing' && speech.currentKey === 'voice-preview'" @click="speech.speak('你好，我是小伴。愿你的每一天，都有一点温暖的陪伴。', 'voice-preview')">试听声音</NButton>
      <NButton v-if="speech.status !== 'idle'" @click="speech.stop()">停止朗读</NButton>
    </div>
    <NAlert v-if="speech.error" type="warning" :show-icon="false">{{ speech.error }}</NAlert>
  </section>
</template>
<style scoped>
.voice-picker{display:flex;align-items:center;gap:10px;width:min(65%,470px)}.voice-picker :deep(.n-select){flex:1;min-width:0}@media(max-width:760px){.voice-picker{width:65%;flex-wrap:wrap}}
</style>

