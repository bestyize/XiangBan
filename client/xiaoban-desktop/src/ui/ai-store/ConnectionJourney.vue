<script setup lang="ts">
import { Check, LoaderCircle, Plug } from 'lucide-vue-next';
defineProps<{ name: string; stage: 'ready' | 'connecting' | 'authorizing' | 'connected'; iconUrl?: string | null; description?: string }>();
</script>
<template>
  <header class="connection-journey" :class="stage">
    <div class="connection-marks" aria-hidden="true">
      <span class="journey-app"><img src="/xiaoban.svg" alt="" /></span>
      <span class="journey-link"><span class="journey-node"><Check v-if="stage === 'connected'" :size="13" :stroke-width="2.8" /><LoaderCircle v-else-if="stage !== 'ready'" class="journey-spinner" :size="13" /><span v-else class="journey-dots">••</span></span></span>
      <span class="journey-service"><img v-if="iconUrl?.startsWith('data:image/') || iconUrl?.startsWith('https://')" :src="iconUrl" alt="" referrerpolicy="no-referrer" /><Plug v-else :size="28" :stroke-width="1.8" /></span>
    </div>
    <h2>{{ stage === 'connected' ? `${name}已连接` : stage === 'authorizing' ? `正在连接 ${name}` : stage === 'connecting' ? `正在连接 ${name}` : `连接 ${name}` }}</h2>
    <p v-if="description && stage === 'ready'" class="connection-description" :title="description">{{ description }}</p>
  </header>
</template>
<style scoped>
.connection-journey{text-align:center;padding:8px 0 0}
.connection-marks{display:flex;justify-content:center;align-items:center;gap:14px;margin-bottom:18px}
.journey-app,.journey-service{display:grid;place-items:center;width:60px;height:60px;flex:none;border-radius:16px;overflow:hidden;background:var(--ui-surface-sunken);box-shadow:inset 0 0 0 1px var(--ui-line),0 2px 6px rgba(0,0,0,.04);color:var(--ui-text-secondary)}
.journey-app img{width:60px;height:60px;object-fit:contain}
.journey-service img{width:38px;height:38px;object-fit:contain}
.journey-link{position:relative;display:flex;align-items:center;justify-content:center;width:54px;height:24px;color:var(--ui-text-tertiary)}
.journey-link::before{content:"";position:absolute;left:0;right:0;top:50%;border-top:1.5px dashed var(--ui-border-strong)}
.journey-node{position:relative;z-index:1;display:grid;place-items:center;width:24px;height:24px;border-radius:50%;background:var(--ui-surface);box-shadow:inset 0 0 0 1px var(--ui-line)}
.journey-dots{font-size:11px;letter-spacing:1px;line-height:1}
.connected .journey-link::before{border-top-style:solid;border-top-color:var(--ui-success)}
.connected .journey-node{background:color-mix(in srgb,var(--ui-success) 14%,var(--ui-surface));color:var(--ui-success-text);box-shadow:none}
h2{margin:0;color:var(--ui-text-primary);font-size:19px;font-weight:600;letter-spacing:-.015em;line-height:1.45}
.connection-description{display:-webkit-box;-webkit-line-clamp:3;-webkit-box-orient:vertical;overflow:hidden;max-width:440px;margin:10px auto 0;color:var(--ui-text-secondary);font-size:13px;line-height:1.65}
.journey-spinner{animation:journey-spin 1.3s linear infinite;color:var(--ui-text-primary)}
@keyframes journey-spin{to{transform:rotate(360deg)}}
@media(prefers-reduced-motion:reduce){.journey-spinner{animation:none}}
</style>
