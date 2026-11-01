<script setup lang="ts">
import { Check, LoaderCircle, Plug } from 'lucide-vue-next';
defineProps<{ name: string; stage: 'ready' | 'connecting' | 'authorizing' | 'connected'; iconUrl?: string | null; description?: string }>();
</script>
<template>
  <header class="connection-journey" :class="stage">
    <div class="connection-marks" aria-hidden="true">
      <span class="journey-app"><img src="/xiaoban.svg" alt="" /></span>
      <span class="journey-link"><Check v-if="stage === 'connected'" :size="19" /><LoaderCircle v-else-if="stage !== 'ready'" class="journey-spinner" :size="18" /><span v-else>···</span></span>
      <span class="journey-service"><img v-if="iconUrl?.startsWith('data:image/') || iconUrl?.startsWith('https://')" :src="iconUrl" alt="" referrerpolicy="no-referrer" /><Plug v-else :size="34" /></span>
    </div>
    <h2>{{ stage === 'connected' ? `${name}已连接` : stage === 'authorizing' ? `正在连接 ${name}` : stage === 'connecting' ? `正在连接 ${name}` : `连接 ${name}` }}</h2>
    <p v-if="description && stage === 'ready'" class="connection-description" :title="description">{{ description }}</p>
  </header>
</template>
<style scoped>
.connection-journey{text-align:center;padding:10px 0 0}.connection-marks{display:flex;justify-content:center;align-items:center;gap:18px;margin-bottom:22px}
.journey-app,.journey-service{display:grid;place-items:center;width:66px;height:66px;flex:none;border-radius:18px;overflow:hidden;color:var(--ui-ink)}.journey-app img{width:66px;height:66px;object-fit:contain}.journey-service img{width:62px;height:62px;object-fit:contain}.journey-service{background:var(--ui-surface)}
.journey-link{display:grid;place-items:center;width:28px;color:var(--ui-border-strong);letter-spacing:4px;font-size:26px}.connected .journey-link{color:var(--ui-success-text)}
h2{margin:0;color:var(--ui-text-primary);font-size:19px;font-weight:600;line-height:1.5}.connection-description{display:-webkit-box;-webkit-line-clamp:3;-webkit-box-orient:vertical;overflow:hidden;max-width:490px;margin:16px auto 0;color:var(--ui-text-secondary);font-size:13px;line-height:1.75}
.journey-spinner{animation:journey-spin 1.5s linear infinite}@keyframes journey-spin{to{transform:rotate(360deg)}}@media(prefers-reduced-motion:reduce){.journey-spinner{animation:none}}
</style>
