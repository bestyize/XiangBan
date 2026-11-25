<script setup lang="ts">
import { ref } from 'vue'
import ProductDownloads from '@/components/ProductDownloads.vue'

const latest = ref<{ version: string; publishedAt: string } | null>(null)
const openFaq = ref<number | null>(0)

const faqs = [
  {
    q: 'macOS 首次打开提示“无法验证开发者”怎么办？',
    a: '目前 macOS 安装包使用固定自签名证书（尚未公证）。把应用拖入「应用程序」后，首次启动请在访达中右键点击「小伴桌面」图标 → 选择「打开」，确认一次后之后就可以正常双击启动。',
  },
  {
    q: 'Windows 安装需要管理员权限吗？支持控电脑与控手机吗？',
    a: '不需要管理员权限。Windows 64 位 NSIS 安装包默认装在当前用户目录，卸载干净无残留，支持 Windows 10 / 11（需 WebView2，Win11 自带）。内置原生 Computer Use 控屏引擎与 ADB 控手机工具，免额外配置 Node 环境即可按需启用。',
  },
  {
    q: 'Android 手机版不登录或没开电脑也能独立使用吗？',
    a: '完全可以。手机版内置原生端侧 Agent 与离线语音管线，即使不开电脑也能独立在手机上造 3D 游戏、发语音通话、用离线红外万能遥控器控电视空调、使用 104 应用「轻启」或免登录接入自有第三方模型。登录同一账号后，还可自动发现并远程遥控电脑端。',
  },
]

function toggleFaq(index: number) {
  openFaq.value = openFaq.value === index ? null : index
}

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
      <!-- Centered Iconic Hero -->
      <section class="dl-center-hero">
        <img src="/desktop/app-icon-new.webp" alt="小伴双星图标" class="app-icon-hero" width="68" height="68">
        <h1>把小伴装进<span class="grad-text">你的设备</span></h1>
        <p>
          电脑端负责深度工作、控电脑/控手机、画布直出 PPTX/Figma 与 3D 龙鱼活桌面；Android 手机端内置本地造 3D 游戏、端侧语音通话、远程审批电脑与万能红外遥控器。
        </p>
        <div v-if="latest" class="latest-chip">
          <span class="latest-dot" aria-hidden="true" />
          <span>当前最新 <strong>v{{ latest.version }}</strong> · {{ formatDate(latest.publishedAt) }} 发布</span>
        </div>
      </section>

      <!-- Interactive OS Switcher + Main Download Stage + All-Platform Quick Strip + Announcements -->
      <ProductDownloads standalone @loaded="latest = $event" />

      <!-- 3-Step First-Run Journey + Compact FAQ -->
      <section class="journey-sec" aria-labelledby="journey-title">
        <div class="sec-title-center">
          <span>GETTING STARTED</span>
          <h2 id="journey-title">装好之后，3 步释放双端全部能力</h2>
        </div>

        <div class="steps-3">
          <article class="step-card">
            <span class="step-num">01</span>
            <h3>登录即用，或免登录接自有模型</h3>
            <p>登录相伴统一账号即可在双端共享小伴 AI 额度；如果你有 OpenAI / Anthropic / DeepSeek 等第三方 API，在桌面独立设置窗或手机模型配置里直接添加，密钥仅存本机。</p>
          </article>
          <article class="step-card">
            <span class="step-num">02</span>
            <h3>电脑开启控屏、画布与活桌面</h3>
            <p>在工作区派发自主任务、按全局快捷键用超级小伴截图标注、在设计画布导出 .pptx / .fig，再把 3D 龙鱼特效、实时股票组件和 7 只桌宠请上屏幕。</p>
          </article>
          <article class="step-card">
            <span class="step-num">03</span>
            <h3>装上 Android 版，随身造作品与遥控电脑</h3>
            <p>出门用端侧离线语音通话聊灵感、在手机本地造 3D 游戏或遥控家里电视空调；电脑上跑着长任务时，还能在手机实时看进度并远程审批权限。</p>
          </article>
        </div>

        <div class="faq-list">
          <div
            v-for="(item, idx) in faqs"
            :key="item.q"
            class="faq-row"
            :class="{ open: openFaq === idx }"
          >
            <button type="button" class="faq-q" :aria-expanded="openFaq === idx" @click="toggleFaq(idx)">
              <span>{{ item.q }}</span>
              <svg class="faq-chevron" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="m6 9 6 6 6-6" /></svg>
            </button>
            <div v-show="openFaq === idx" class="faq-a">
              {{ item.a }}
            </div>
          </div>
        </div>
      </section>

      <footer class="site-footer">
        <div class="footer-brand">
          <strong>小伴桌面</strong>
          <p>端云一体、双端接力的个人 AI 伙伴 · wallpaper.xiangban.xyz</p>
        </div>
        <div class="footer-links">
          <RouterLink to="/features">功能介绍</RouterLink>
          <RouterLink to="/download">下载安装</RouterLink>
          <RouterLink to="/guide">玩法指南</RouterLink>
          <RouterLink to="/images">壁纸库</RouterLink>
          <a href="https://chat.xiangban.xyz/contact" target="_blank" rel="noopener noreferrer">联系我们</a>
          <a href="https://chat.xiangban.xyz/" target="_blank" rel="noopener noreferrer">超级小伴社区 ↗</a>
        </div>
      </footer>
    </div>
  </main>
</template>

<style scoped>
.dl-page {
  --canvas: #fcfaf7;
  --surface: #ffffff;
  --ink: #19171a;
  --ink-2: #524d54;
  --ink-3: #837d86;
  --coral: #ee4f6d;
  --coral-grad: linear-gradient(135deg, #ff5b7d 0%, #f9744b 100%);
  --line: rgba(28, 22, 28, 0.075);
  --shadow-sm: 0 2px 10px rgba(26, 18, 24, 0.04);
  --ease: cubic-bezier(0.22, 1, 0.36, 1);

  min-height: 100vh;
  color: var(--ink);
  background:
    radial-gradient(62% 520px at 50% -110px, rgba(255, 114, 134, 0.16), transparent 68%),
    radial-gradient(42% 380px at 84% 120px, rgba(249, 146, 86, 0.13), transparent 66%),
    radial-gradient(40% 340px at 14% 200px, rgba(124, 178, 255, 0.10), transparent 66%),
    var(--canvas);
}

.wrap {
  width: min(1180px, calc(100% - 64px));
  margin: 0 auto;
}

.grad-text {
  background: var(--coral-grad);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

.dl-center-hero {
  padding: 54px 0 26px;
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
}
.app-icon-hero {
  width: 68px;
  height: 68px;
  border-radius: 18px;
  box-shadow: 0 18px 40px rgba(249, 102, 96, 0.32), 0 0 0 1px rgba(255, 255, 255, 0.6) inset;
  margin-bottom: 20px;
}
.dl-center-hero h1 {
  font-size: clamp(36px, 4.6vw, 56px);
  line-height: 1.12;
  letter-spacing: -0.045em;
  font-weight: 760;
  color: var(--ink);
}
.dl-center-hero p {
  margin-top: 14px;
  font-size: 15.5px;
  color: var(--ink-2);
  max-width: 660px;
  line-height: 1.75;
}
.latest-chip {
  margin-top: 16px;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 6px 14px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.85);
  border: 1px solid rgba(242, 92, 114, 0.22);
  font-size: 12.5px;
  color: var(--ink-2);
}
.latest-chip strong {
  color: var(--ink);
  font-weight: 700;
}
.latest-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--coral-grad);
}

.journey-sec {
  padding: 60px 0 28px;
}
.sec-title-center {
  text-align: center;
  margin-bottom: 32px;
}
.sec-title-center span {
  font-size: 11.5px;
  font-weight: 750;
  letter-spacing: 0.14em;
  color: var(--coral);
  text-transform: uppercase;
}
.sec-title-center h2 {
  margin-top: 8px;
  font-size: clamp(24px, 3vw, 34px);
  letter-spacing: -0.03em;
  font-weight: 750;
}

.steps-3 {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}
.step-card {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 22px;
  padding: 26px;
  box-shadow: var(--shadow-sm);
}
.step-num {
  width: 36px;
  height: 36px;
  border-radius: 11px;
  display: grid;
  place-items: center;
  font-size: 13px;
  font-weight: 800;
  margin-bottom: 16px;
}
.step-card:nth-child(1) .step-num { background: #fff0f2; color: #e24666; }
.step-card:nth-child(2) .step-num { background: #fff5e5; color: #d97706; }
.step-card:nth-child(3) .step-num { background: #eaf9f0; color: #158f4d; }
.step-card h3 { font-size: 17px; font-weight: 700; margin-bottom: 8px; }
.step-card p { font-size: 13.5px; color: var(--ink-2); line-height: 1.72; }

.faq-list {
  margin-top: 26px;
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 22px;
  overflow: hidden;
}
.faq-row { border-bottom: 1px solid var(--line); }
.faq-row:last-child { border-bottom: 0; }
.faq-q {
  width: 100%;
  padding: 18px 24px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  font-size: 14.5px;
  font-weight: 650;
  text-align: left;
  color: var(--ink);
}
.faq-q:hover { background: #fdfbf9; }
.faq-a {
  padding: 0 24px 18px;
  font-size: 13.5px;
  color: var(--ink-2);
  line-height: 1.75;
}
.faq-chevron {
  color: var(--ink-3);
  transition: transform 0.2s var(--ease);
  flex-shrink: 0;
}
.faq-row.open .faq-chevron { transform: rotate(180deg); }

.site-footer {
  border-top: 1px solid var(--line);
  padding: 34px 0 calc(var(--tabbar-h, 58px) + var(--safe-bottom, 0px) + 38px);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
}
@media (min-width: 769px) {
  .site-footer { padding-bottom: 60px; }
}
.footer-brand strong { font-size: 17px; font-weight: 720; display: block; }
.footer-brand p { font-size: 12.5px; color: var(--ink-3); margin-top: 3px; }
.footer-links { display: flex; flex-wrap: wrap; gap: 22px; font-size: 13.5px; color: var(--ink-2); }
.footer-links a:hover { color: var(--coral); }

@media (max-width: 960px) {
  .steps-3 { grid-template-columns: 1fr; }
}
@media (max-width: 768px) {
  .wrap { width: calc(100% - 32px); }
  .dl-center-hero { padding: 30px 0 20px; }
  .app-icon-hero { width: 54px; height: 54px; border-radius: 15px; margin-bottom: 14px; }
  .dl-center-hero p { font-size: 14px; }
  .site-footer { flex-direction: column; align-items: flex-start; }
}
</style>
