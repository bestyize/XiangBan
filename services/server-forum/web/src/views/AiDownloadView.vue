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
  max-width: 540px;
  margin: 20px auto;
  padding: 34px 22px;
  border-radius: 18px;
  background: var(--surface);
  box-shadow: var(--shadow-card);
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
h1 { margin: 14px 0 6px; font-size: 22px; }
p { margin: 0; color: var(--ink-2); font-size: 14px; }
.download-links { display: flex; flex-wrap: wrap; justify-content: center; gap: 10px; margin-top: 22px; }
.download-links a { gap: 7px; min-height: 44px; padding: 8px 13px; border-radius: 999px; }
.download-links small { font-size: 11px; font-weight: 400; }
</style>
