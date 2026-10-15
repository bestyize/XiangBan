<script setup lang="ts">
import { computed } from 'vue';
import { NButton, NProgress } from 'naive-ui';
import type { AsrStore } from '../asr/store.ts';
const props = defineProps<{ asr: AsrStore; chat?: boolean }>();
const percent = computed(() => Math.min(100, Math.floor(props.asr.downloaded / Math.max(1, props.asr.total) * 100)));
</script>
<template>
  <div v-if="asr.enabled || asr.error" class="asr-status" role="status" aria-live="polite" data-testid="asr-status">
    <template v-if="asr.preparing">
      <span>{{ asr.phase === 'loading' ? '正在加载本地语音识别引擎，请稍后…' : '正在加载语音识别引擎，请稍后。从魔搭下载模型…' }}</span>
      <NProgress v-if="asr.phase === 'downloading'" type="line" :percentage="percent" :height="4" />
      <small v-if="asr.phase === 'downloading'">{{ (asr.downloaded / 1048576).toFixed(1) }} / {{ (asr.total / 1048576).toFixed(1) }} MiB</small>
    </template>
    <template v-else-if="asr.error"><span class="asr-error">{{ asr.error }}</span><NButton v-if="asr.enabled" size="tiny" :disabled="asr.saving" data-testid="asr-retry" @click="asr.retry()">重试</NButton></template>
    <span v-else-if="asr.recognizing">正在本地识别…</span>
    <span v-else-if="asr.listening">正在听你说话，停顿后转为文字，确认后发送。</span>
    <span v-else-if="asr.recordingRequested">正在准备麦克风，请稍后…</span>
    <span v-else-if="asr.enabled">{{ chat ? '点击麦克风开始本次语音输入。' : '语音识别已启用，录音需在聊天中点击麦克风；关闭录音会保留模型。' }}</span>
  </div>
</template>
<style scoped>
.asr-status{display:flex;flex-wrap:wrap;align-items:center;gap:8px;color:var(--ui-muted);font-size:12px;line-height:1.6}.asr-status :deep(.n-progress){flex-basis:100%;max-width:360px}.asr-error{color:var(--ui-text)}
</style>
