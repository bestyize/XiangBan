<script setup lang="ts">
import { ref } from 'vue'
import ProductDownloads from '@/components/ProductDownloads.vue'

const latest = ref<{ version: string; publishedAt: string } | null>(null)

function formatDate(iso?: string): string {
  if (!iso) return ''
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return ''
  return d.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' })
}
</script>

<template>
  <main class="dl-page">
    <div class="wrap">
      <!-- Single Unified Hero -->
      <section class="dl-hero">
        <div>
          <span class="kicker"><span class="kicker-dot" /> DOWNLOAD XIAOBAN</span>
          <h1>把小伴带到<br><em>你的设备上。</em></h1>
          <p class="dl-hero-sub">
            已为你匹配当前系统的推荐安装包，也可以直接选择其他平台版本。桌面端与手机端均为官方原包分发，支持应用内更新检测。
          </p>
        </div>
        <div v-if="latest" class="ver-badge-box">
          <span class="kicker-dot" aria-hidden="true" />
          <span>当前最新 <strong>v{{ latest.version }}</strong> · {{ formatDate(latest.publishedAt) }} 发布</span>
        </div>
      </section>

      <!-- Standalone Recommended Spotlight + All Platforms + Announcements -->
      <ProductDownloads standalone @loaded="latest = $event" />

      <!-- Structured Before You Start Guide -->
      <section class="guide-section" aria-labelledby="dl-guide-title">
        <div class="guide-head">
          <h2 id="dl-guide-title">安装与使用前了解</h2>
          <p>各平台特性、首次安装提示与 AI 模型说明</p>
        </div>

        <div class="guide-grid">
          <article class="guide-card">
            <span class="guide-card-ico" aria-hidden="true">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9"><rect x="2" y="3" width="20" height="14" rx="2" /><path d="M8 21h8" /><path d="M12 17v4" /></svg>
            </span>
            <h3>不同设备，体验各有侧重</h3>
            <p>桌面端聚焦深度工作流与个性桌面，手机端聚焦随时对话、社区互动与跨端互联。</p>
            <ul class="guide-list">
              <li><strong>Windows / macOS</strong>：完整 AI 任务工作区、壁纸特效、桌宠与插件</li>
              <li><strong>Android</strong>：原生社区、小伴 AI 语音/文字对话与手机桌面互联</li>
            </ul>
          </article>

          <article class="guide-card">
            <span class="guide-card-ico" aria-hidden="true">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9"><path d="M12 2l2.4 7.4H22l-6.2 4.5 2.4 7.4L12 16.8 5.8 21.3l2.4-7.4L2 9.4h7.6z" /></svg>
            </span>
            <h3>关于小伴 AI 与模型服务</h3>
            <p>灵活适配个人习惯，既可开箱即用，也支持完全自管模型接口。</p>
            <ul class="guide-list">
              <li>使用小伴 AI 需登录相伴账号并有可用额度</li>
              <li>也可按应用内引导配置第三方或本地模型服务</li>
            </ul>
          </article>

          <article class="guide-card">
            <span class="guide-card-ico" aria-hidden="true">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" /></svg>
            </span>
            <h3>首次安装与社区支持</h3>
            <p>Windows 默认安装至当前用户目录，无需管理员权限；macOS 首次打开如遇安全提示，右键选择「打开」即可。</p>
            <div class="guide-links">
              <RouterLink to="/guide" class="text-link">玩法指南 ↗</RouterLink>
              <a href="https://chat.xiangban.xyz/" target="_blank" rel="noopener noreferrer" class="text-link">小伴社区 ↗</a>
              <a href="https://chat.xiangban.xyz/contact" target="_blank" rel="noopener noreferrer" class="text-link">联系我们 ↗</a>
            </div>
          </article>
        </div>

        <div class="req-strip">
          <div class="req-items">
            <span><strong>Windows</strong>：Win 10/11 64 位 · 需 WebView2</span>
            <span><strong>macOS</strong>：macOS 12+ · Apple 芯片</span>
            <span><strong>Android</strong>：Android 10 及以上 · arm64</span>
          </div>
          <span class="req-note">官方原包分发 · 支持应用内更新检测</span>
        </div>
      </section>

      <footer class="site-footer">
        <div class="footer-brand">
          <strong>小伴桌面</strong>
          <p>你的 AI 工作与创作助手</p>
        </div>
        <div class="footer-links">
          <RouterLink to="/features">功能介绍</RouterLink>
          <RouterLink to="/download">下载安装</RouterLink>
          <RouterLink to="/guide">玩法指南</RouterLink>
          <RouterLink to="/images">壁纸</RouterLink>
          <a href="https://chat.xiangban.xyz/contact">联系我们</a>
          <a href="https://chat.xiangban.xyz/" target="_blank" rel="noopener noreferrer">社区 ↗</a>
        </div>
      </footer>
    </div>
  </main>
</template>

<style scoped>
.dl-page {
  --bg: #fafaf9;
  --surface: #ffffff;
  --surface-soft: #f4f3f0;
  --ink: #141416;
  --ink-2: #52525b;
  --ink-3: #71717a;
  --brand: #18181b;
  --line: rgba(24, 24, 27, 0.08);
  background: var(--bg);
  color: var(--ink);
  min-height: 100vh;
}

.wrap {
  width: min(1220px, calc(100% - 64px));
  margin: 0 auto;
}

.kicker {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  letter-spacing: 0.14em;
  font-weight: 700;
  color: var(--ink-2);
  text-transform: uppercase;
}

.kicker-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--brand);
  box-shadow: 0 0 0 4px rgba(24, 24, 27, 0.12);
}

.dl-hero {
  padding: 60px 0 36px;
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 32px;
}

.dl-hero h1 {
  font-size: clamp(40px, 4.8vw, 62px);
  line-height: 1.15;
  letter-spacing: -0.045em;
  font-weight: 700;
  margin: 18px 0 14px;
}

.dl-hero h1 em {
  font-style: normal;
  color: #52525b;
}

.dl-hero-sub {
  font-size: 15.5px;
  color: var(--ink-2);
  max-width: 540px;
  line-height: 1.8;
}

.ver-badge-box {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  padding: 10px 16px;
  border-radius: 14px;
  background: var(--surface);
  border: 1px solid var(--line);
  box-shadow: 0 2px 10px rgba(15, 15, 20, 0.04);
  font-size: 13px;
  color: var(--ink-2);
  flex-shrink: 0;
}

.ver-badge-box strong {
  color: var(--brand);
  font-weight: 700;
}

.guide-section {
  padding: 52px 0 68px;
}

.guide-head {
  margin-bottom: 22px;
}

.guide-head h2 {
  font-size: 22px;
  font-weight: 700;
  letter-spacing: -0.02em;
}

.guide-head p {
  font-size: 13.5px;
  color: var(--ink-3);
  margin-top: 4px;
}

.guide-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}

.guide-card {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 18px;
  padding: 24px;
  display: flex;
  flex-direction: column;
}

.guide-card-ico {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  background: var(--surface-soft);
  color: var(--brand);
  display: grid;
  place-items: center;
  margin-bottom: 16px;
}

.guide-card h3 {
  font-size: 16px;
  font-weight: 700;
  margin-bottom: 10px;
}

.guide-card p {
  font-size: 13.5px;
  color: var(--ink-2);
  line-height: 1.75;
}

.guide-list {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 7px;
  font-size: 13px;
  color: var(--ink-2);
  list-style: none;
}

.guide-list li {
  display: flex;
  align-items: flex-start;
  gap: 8px;
}

.guide-list li::before {
  content: '';
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: #52525b;
  margin-top: 8px;
  flex-shrink: 0;
}

.guide-links {
  margin-top: auto;
  padding-top: 16px;
  display: flex;
  flex-wrap: wrap;
  gap: 14px;
  font-size: 13.5px;
}

.text-link {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-weight: 650;
  color: var(--brand);
}
.text-link:hover {
  text-decoration: underline;
  text-underline-offset: 4px;
}

.req-strip {
  margin-top: 16px;
  border-radius: 14px;
  background: var(--surface-soft);
  border: 1px solid var(--line);
  padding: 16px 22px;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  font-size: 12.5px;
  color: var(--ink-2);
}

.req-items {
  display: flex;
  flex-wrap: wrap;
  gap: 18px;
}

.req-items strong {
  color: var(--ink);
  font-weight: 650;
}

.req-note {
  color: var(--ink-3);
}

.site-footer {
  border-top: 1px solid var(--line);
  padding: 36px 0 calc(var(--tabbar-h) + var(--safe-bottom) + 40px);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
}

@media (min-width: 769px) {
  .site-footer {
    padding-bottom: 64px;
  }
}

.footer-brand strong {
  font-size: 18px;
  font-weight: 700;
  display: block;
}

.footer-brand p {
  font-size: 12.5px;
  color: var(--ink-3);
  margin-top: 4px;
}

.footer-links {
  display: flex;
  flex-wrap: wrap;
  gap: 24px;
  font-size: 13.5px;
  color: var(--ink-2);
}

.footer-links a:hover {
  color: var(--brand);
  text-decoration: underline;
}

@media (max-width: 960px) {
  .guide-grid { grid-template-columns: 1fr; }
}

@media (max-width: 768px) {
  .wrap { width: calc(100% - 32px); }
  .dl-hero { flex-direction: column; align-items: flex-start; padding: 32px 0 24px; gap: 16px; }
  .site-footer { flex-direction: column; align-items: flex-start; }
}
</style>
