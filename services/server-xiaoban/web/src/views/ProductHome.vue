<script setup lang="ts">
import { nextTick, onActivated, watch } from 'vue'
import { useRoute } from 'vue-router'
import ProductDownloads from '@/components/ProductDownloads.vue'
const route = useRoute()
async function jumpToHash() {
  if (!route.hash) return
  await nextTick()
  let id = route.hash.slice(1)
  try { id = decodeURIComponent(id) } catch { /* Keep the original hash. */ }
  document.getElementById(id)?.scrollIntoView()
}
onActivated(jumpToHash)
watch(() => route.hash, jumpToHash)
</script>

<template>
  <main class="product-home">
    <section class="hero wrap">
      <div class="hero-copy">
        <span class="kicker"><span class="dot"></span> 你的 AI 工作与创作助手</span>
        <h1>让想法，<br><em>有了下一步。</em></h1>
        <p>从整理资料、处理任务，到设计和创作。<br class="desktop-only">把想法交给小伴，一起把它变成看得见的成果。</p>
        <div class="actions"><RouterLink class="button primary" to="/download">下载小伴 <span>↗</span></RouterLink><RouterLink class="button secondary" to="/features">探索功能 <span>→</span></RouterLink></div>
        <div class="platforms"><span>AVAILABLE FOR</span> Windows <i></i> macOS <i></i> Android</div>
      </div>
      <div class="hero-visual">
        <div class="visual-head"><div><span class="visual-mark">✳</span> 小伴桌面 <span class="visual-divider">/</span> 对话创作</div><span class="visual-dots">● ● ●</span></div>
        <img src="/desktop/current-home.webp" alt="小伴桌面新建任务界面，展示对话输入和创作入口" width="849" height="670">
        <div class="visual-caption"><span>小伴桌面 3.1.1 实机截图</span><span>从一句话，开始下一步</span></div>
      </div>
    </section>
    <div class="intro-line wrap"><span>AN ASSISTANT FOR WHAT'S NEXT</span><p>不止回答问题，<strong>也陪你往前走。</strong></p><span>01 — 03</span></div>
    <section class="story wrap" id="work">
      <div class="section-label"><span>01 / WORK</span><span>工作，一起推进</span></div>
      <div class="story-grid"><div class="story-copy"><h2>从问题出发，<br>一步步完成任务。</h2><p>查找资料、整理文件、规划步骤。用自然的对话告诉小伴你的目标，查看任务执行过程，并在需要时确认操作。</p><RouterLink to="/features" class="text-link">了解 AI 工作能力 <span>↗</span></RouterLink></div><div class="image-panel sage"><img src="/desktop/task-plan.webp" alt="小伴将咖啡店活动页面的创作任务拆解为多个执行步骤" width="849" height="440" loading="lazy"><div class="image-label">任务示例 · 规划并执行</div></div></div>
    </section>
    <section class="story wrap" id="create">
      <div class="section-label"><span>02 / CREATE</span><span>创意，逐渐成形</span></div>
      <div class="story-grid reverse"><div class="story-copy"><h2>有个想法？<br>试着把它做出来。</h2><p>和小伴一起探索设计、网页、游戏或桌面内容。通过对话不断调整，在作品区查看成果，让灵感有继续生长的空间。右图是一次真实任务生成的示例页面。</p><RouterLink to="/features#create" class="text-link">了解创作能力 <span>↗</span></RouterLink></div><div class="image-panel cream"><img src="/desktop/task-result.webp" alt="小伴根据示例需求生成的咖啡店活动页面" width="535" height="387" loading="lazy"><div class="image-label">示例作品 · 咖啡店活动页面</div></div></div>
    </section>
    <section class="story wrap" id="desktop">
      <div class="section-label"><span>03 / MAKE IT YOURS</span><span>桌面，也有自己的样子</span></div>
      <div class="story-grid"><div class="story-copy"><h2>让工作的地方，<br>更像你自己。</h2><p>换一张喜欢的壁纸，添加桌面组件与特效，也可以让桌面伙伴陪在身边。小伴把实用与个性，留在你每天使用的空间里。</p><RouterLink to="/desktop-experience" class="text-link">探索桌面体验 <span>↗</span></RouterLink></div><div class="image-panel blush"><img src="/desktop/wallpapers.webp" alt="小伴桌面的壁纸选择和预览界面" width="1280" height="800" loading="lazy"><div class="image-label">壁纸 · 组件 · 桌面伙伴 <span> / 历史版本界面</span></div></div></div>
    </section>
    <section class="closing wrap"><span class="kicker">A LITTLE MORE POSSIBLE</span><h2>下一件想做的事，<br>从这里开始。</h2><p>给想法一点空间，剩下的，一起探索。</p><RouterLink class="button primary" to="/download">下载小伴 <span>↗</span></RouterLink></section>
    <div class="wrap"><ProductDownloads /></div>
    <footer class="footer wrap"><div><strong>小伴</strong><p>你的 AI 工作与创作助手</p></div><div><RouterLink to="/features">功能介绍</RouterLink><RouterLink to="/guide">使用指南</RouterLink><RouterLink to="/images">壁纸</RouterLink><a href="https://chat.xiangban.xyz/" target="_blank" rel="noopener noreferrer">社区 ↗</a></div></footer>
  </main>
</template>

<style scoped>
.product-home{background:#f8faf7;color:#222b24;overflow:hidden}.wrap{width:min(1280px,calc(100% - 80px));margin:0 auto}.hero{min-height:660px;display:grid;grid-template-columns:.88fr 1.12fr;align-items:center;gap:5%;padding-top:55px;padding-bottom:80px}.kicker,.section-label,.intro-line>span{font-size:11px;letter-spacing:.15em;font-weight:750;color:#66816a}.kicker{display:inline-flex;align-items:center;gap:9px}.dot{width:8px;height:8px;background:#8eb89b;border-radius:50%;box-shadow:0 0 0 4px #e4eee4}.hero h1{font-size:clamp(49px,5.2vw,76px);line-height:1.18;letter-spacing:-.06em;margin:30px 0 25px;font-weight:700}.hero h1 em{font-style:normal;color:#668c6e}.hero-copy>p,.story-copy>p,.closing p{font-size:16px;line-height:1.95;color:#707b70}.actions{display:flex;gap:12px;margin-top:34px}.button{display:inline-flex;align-items:center;justify-content:center;gap:30px;min-height:50px;padding:0 21px;border-radius:7px;font-size:14px;font-weight:700}.primary{background:#284c36;color:#fff}.primary:hover{background:#3b6548}.secondary{border:1px solid #cdd8cd;background:#fff;color:#334a38}.button span,.text-link span{font-size:18px}.platforms{display:flex;align-items:center;gap:11px;font-size:12px;color:#68766a;margin-top:52px}.platforms span{font-size:9px;letter-spacing:.13em;color:#a0aba0;margin-right:5px}.platforms i{width:3px;height:3px;border-radius:50%;background:#a7b3a7}.hero-visual{padding:13px 13px 0;border:1px solid #d7e0d6;border-radius:13px;background:#fff;box-shadow:0 28px 70px #53695720;transform:rotate(1deg)}.visual-head,.visual-caption{display:flex;align-items:center;justify-content:space-between;font-size:11px;color:#7d887e}.visual-head{padding:2px 4px 12px;font-weight:650}.visual-mark{color:#668c6e;font-size:17px;margin-right:5px}.visual-divider{color:#d0d9d0;margin:0 6px}.visual-dots{letter-spacing:4px;color:#ced8cd;font-size:8px}.hero-visual img{width:100%;height:auto;border:1px solid #edf0ec;border-radius:5px}.visual-caption{padding:13px 4px;letter-spacing:.04em}.intro-line{border-top:1px solid #dfe7dd;border-bottom:1px solid #dfe7dd;display:flex;align-items:center;justify-content:space-between;gap:20px;padding:25px 0}.intro-line>span{font-size:9px;color:#9aa89b}.intro-line p{font-size:15px;color:#8a988b}.intro-line strong{color:#354b39}.story{padding-top:100px}.section-label{display:flex;justify-content:space-between;border-bottom:1px solid #dce5da;padding-bottom:16px}.section-label span:last-child{font-size:12px;letter-spacing:0;color:#9daaa0}.story-grid{display:grid;grid-template-columns:1fr 1.15fr;gap:8%;align-items:center;margin-top:44px}.story-grid.reverse{grid-template-columns:1.15fr 1fr}.story-grid.reverse .story-copy{order:2}.story-copy h2,.closing h2{font-size:clamp(30px,3.6vw,49px);line-height:1.35;letter-spacing:-.04em;margin-bottom:24px}.story-copy p{max-width:420px}.text-link{display:inline-flex;align-items:center;gap:25px;margin-top:32px;font-size:14px;font-weight:700;color:#416a4b}.text-link:hover{text-decoration:underline}.image-panel{padding:48px 40px 22px;border-radius:13px}.sage{background:#e7eee4}.cream{background:#f0eee5}.blush{background:#eeeae5}.image-panel img{width:100%;height:auto;border-radius:7px;box-shadow:0 16px 32px #344a3420}.image-label{margin-top:17px;color:#829080;font-size:11px;letter-spacing:.04em}.closing{text-align:center;margin-top:110px;padding:90px 20px;border-radius:16px;background:#e6eee3}.closing h2{margin:18px 0}.closing p{margin-bottom:27px}.footer{display:flex;justify-content:space-between;border-top:1px solid #dce5da;padding:38px 0 70px}.footer strong{font-size:20px}.footer p{font-size:12px;color:#849184;margin-top:5px}.footer>div:last-child{display:flex;gap:28px;font-size:13px;color:#617265}.footer a:hover{text-decoration:underline}@media(max-width:850px){.wrap{width:calc(100% - 40px)}.hero{grid-template-columns:1fr;min-height:0;gap:40px;padding-top:65px;padding-bottom:60px}.hero h1{font-size:clamp(44px,10vw,68px)}.hero-visual{transform:none}.intro-line>span{display:none}.story{padding-top:70px}.story-grid,.story-grid.reverse{grid-template-columns:1fr;gap:30px}.story-grid.reverse .story-copy{order:0}.image-panel{padding:25px 22px 18px}.closing{margin-top:70px;padding:65px 20px}.footer{gap:28px;flex-direction:column}.footer>div:last-child{flex-wrap:wrap;gap:20px}.platforms{margin-top:32px}}@media(max-width:420px){.wrap{width:calc(100% - 28px)}.hero h1{font-size:43px}.hero-copy>p{font-size:15px}.desktop-only{display:none}.actions{gap:8px}.button{gap:18px;padding:0 16px}}
</style>
