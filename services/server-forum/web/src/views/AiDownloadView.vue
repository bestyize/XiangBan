<script setup lang="ts">
import AppIcon from '@/components/AppIcon.vue'
import { ANDROID_DOWNLOAD_URL } from '@/lib/builtinCreation'
import { useHead } from '@/lib/head'
useHead(() => ({ title: '小伴 AI' }))
</script>

<template>
  <section class="ai-download" aria-labelledby="ai-title">
    <div class="ai-mark"><AppIcon name="ai" :size="29" /></div>
    <h1 id="ai-title">小伴 AI</h1>
    <p>下载小伴，在手机或电脑上开始对话与创作。</p>
    <div class="download-links">
      <a class="btn btn-primary" :href="ANDROID_DOWNLOAD_URL">
        <AppIcon name="download" :size="18" />下载手机版 <small>Android</small>
      </a>
      <a class="btn" href="https://wallpaper.xiangban.xyz/download#desktop-downloads">
        <AppIcon name="download" :size="18" />下载桌面版 <small>Windows / macOS</small>
      </a>
    </div>
  </section>
</template>

<style scoped>
.ai-download {
  display: grid;
  justify-items: center;
  align-content: center;
  min-height: min(68dvh, 620px);
  max-width: 560px;
  margin-inline: auto;
  padding: 32px 12px;
  text-align: center;
}
.ai-mark {
  display: grid;
  place-items: center;
  width: 62px;
  height: 62px;
  border-radius: 20px;
  background: var(--brand-soft);
  color: var(--brand);
}
h1 { margin: 16px 0 6px; font-size: 27px; }
p { margin: 0; color: var(--ink-2); font-size: 14px; }
.download-links { display: grid; gap: 10px; width: min(100%, 300px); margin-top: 28px; }
.download-links a { justify-content: flex-start; gap: 8px; min-height: 48px; padding: 8px 14px; }
.download-links small { margin-inline-start: auto; font-size: 12px; font-weight: 400; }
</style>
