<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { getDesktopProduct, targets, type DesktopProduct } from '@/desktop'

const product = ref<DesktopProduct | null>(null)
const loading = ref(true)
const error = ref('')
async function load() {
  loading.value = true
  error.value = ''
  try { product.value = await getDesktopProduct() }
  catch { error.value = '下载信息暂时无法加载，请稍后重试。' }
  finally { loading.value = false }
}
onMounted(load)
</script>

<template>
  <section id="desktop-downloads" class="downloads" aria-labelledby="downloads-title">
    <div class="heading"><div><span class="eyebrow">GET XIAOBAN</span><h2 id="downloads-title">开始使用小伴</h2></div><p>选择适合你设备的版本，开始工作与创作。<br>各平台功能有所不同，请以客户端实际功能为准。</p></div>
    <p v-if="loading" class="status" role="status">正在加载下载信息…</p>
    <p v-if="error" class="status" role="alert">{{ error }} <button type="button" @click="load">重试 →</button></p>
    <div class="grid">
      <article v-for="target in targets.filter(t => t.id !== 'darwin-x86_64' || product?.downloads[t.id])" :key="target.id" class="download-card">
        <span class="platform">{{ target.id.startsWith('windows') ? 'WINDOWS' : target.id.startsWith('darwin') ? 'macOS' : 'ANDROID' }}</span>
        <h3>{{ target.label }}</h3>
        <template v-if="product?.downloads[target.id]">
          <p class="version">版本 {{ product.downloads[target.id].version }} · {{ new Date(product.downloads[target.id].publishedAt).toLocaleDateString('zh-CN') }} 发布</p>
          <a :href="product.downloads[target.id].url" class="download-link">下载安装包 <span aria-hidden="true">↗</span></a>
          <details><summary>更新说明</summary><pre>{{ product.downloads[target.id].notes || '体验优化。' }}</pre></details>
        </template>
        <p v-else class="version">{{ loading ? '正在获取版本信息…' : error ? '下载信息暂不可用' : '安装包发布后即可下载。' }}</p>
      </article>
    </div>
    <p class="note">Windows 需要 WebView2；macOS 当前面向 Apple 芯片；Android 需要 Android 10 及以上系统。AI 能力和桌面功能因平台而异。</p>
    <div v-if="product?.announcements.length" class="announcements"><h3>小伴公告</h3><article v-for="notice in product.announcements" :key="notice.id"><strong>{{ notice.title }}</strong><p>{{ notice.body }}</p></article></div>
  </section>
</template>

<style scoped>
.downloads{scroll-margin-top:90px;padding:88px 0;color:#242923}.heading{display:flex;justify-content:space-between;align-items:end;gap:32px;margin-bottom:32px}.eyebrow,.platform{font-size:11px;font-weight:750;letter-spacing:.14em;color:#54715c}.heading h2{font-size:clamp(32px,4vw,50px);letter-spacing:-.04em;line-height:1.2;margin-top:12px}.heading p,.note,.version{color:#747c72}.heading p{line-height:1.8}.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:12px}.download-card{min-height:190px;padding:25px;border:1px solid #dce3da;background:#fff;border-radius:15px}.download-card h3{font-size:17px;margin:20px 0 7px}.version{font-size:13px}.download-link{display:inline-flex;gap:24px;margin-top:20px;color:#315d40;font-weight:700}.download-link:hover{text-decoration:underline}.download-card details{margin-top:14px;font-size:12px;color:#747c72}.download-card summary{cursor:pointer}.download-card pre{white-space:pre-wrap;font:inherit;margin-top:8px}.note{font-size:13px;line-height:1.8;margin-top:20px}.status{margin-bottom:15px;color:#747c72}.status button{color:#315d40;text-decoration:underline}.announcements{margin-top:40px;border-top:1px solid #dce3da;padding-top:24px}.announcements h3{margin-bottom:15px}.announcements article{margin:12px 0}.announcements p{white-space:pre-wrap;color:#747c72}@media(max-width:700px){.downloads{padding:60px 0}.heading{display:block}.heading p{margin-top:16px}.grid{grid-template-columns:1fr}.download-card{min-height:0}}
</style>
