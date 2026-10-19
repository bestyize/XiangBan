<script setup lang="ts">
import { computed, nextTick, onActivated, ref } from "vue";
import { useRoute } from "vue-router";
import Icon from "@/components/Icon.vue";
import WindowsDesktopDemo from "@/components/WindowsDesktopDemo.vue";
import { getDesktopProduct, targets, type DesktopProduct } from "@/desktop";
const moreCards = [
  {
    src: "/desktop/care.webp",
    eyebrow: "关怀提醒",
    title: "该下班了，该吃药了",
    body: "下班、休息、天气与自定义提醒，小伴用气泡轻轻告诉你。可以稍后十分钟再说，有免打扰时段，开着游戏时会自动延后。",
    alt: "小伴桌面的关怀提醒页：下班提醒、休息提醒与提醒偏好开关",
  },
  {
    src: "/desktop/games.webp",
    eyebrow: "游戏空间",
    title: "累了，玩一会儿",
    body: "俄罗斯方块、2048、消失高手、奶茶大师、小伴贪吃蛇、水墨剑客、暮色滑翔，七款小游戏离线可玩，水墨剑客会记住进度。也能让小伴按想法创作一款。",
    alt: "小伴桌面的游戏空间页：七款本地游戏卡片，标注竖屏、横屏与离线可玩",
  },
  {
    src: "/desktop/models.webp",
    eyebrow: "模型选择",
    title: "小伴 AI，或你熟悉的模型",
    body: "登录后直接用上小伴 AI，额度和兑换在应用内查看。也可以接入 DeepSeek、智谱、Kimi、MiniMax 预设或任意兼容端点，密钥只保存在你的电脑上。",
    alt: "小伴桌面的 AI 模型设置：小伴 AI、DeepSeek、智谱、Kimi、MiniMax 及添加自定义供应商",
  },
];
const defaultIntroduction =
  "随时唤起超级小伴，打字或开口提问，让它帮你查资料、整理文件、创作壁纸和小组件。再把喜欢的风景、日历和一盆小植物，留在每天都看见的桌面上。";
const introduction = computed(() => {
  const text = product.value?.introduction?.trim();
  return !text || text === "会回应的桌面小伙伴，搭配喜欢的图片与动态壁纸，让工作和休息多一点温暖。"
    ? defaultIntroduction
    : text;
});
const product = ref<DesktopProduct | null>(null);
const error = ref("");
const loading = ref(false);
async function load() {
  if (loading.value) return;
  loading.value = true;
  error.value = "";
  try {
    product.value = await getDesktopProduct();
  } catch (e) {
    error.value = (e as Error).message;
  } finally {
    loading.value = false;
  }
}
const route = useRoute();
// The page is rendered by script, so the browser's own jump to a "#section" in the address has already
// come and gone before the section exists. Links from outside (the phone app's update prompt) rely on it.
async function scrollToHash() {
  if (!route.hash) return;
  await nextTick();
  let id = route.hash.slice(1);
  try {
    id = decodeURIComponent(id);
  } catch {
    /* keep the raw text */
  }
  document.getElementById(id)?.scrollIntoView();
}
onActivated(() => {
  void load();
  void scrollToHash();
});
</script>
<template>
  <main class="desktop-page">
    <section class="hero">
      <div class="intro">
        <span class="eyebrow">陪伴，就在桌面上</span>
        <h1>小伴桌面</h1>
        <h2 class="hero-tagline">一句话，办点事。<br />一抬眼，好风景。</h2>
        <p class="hero-description">{{ introduction }}</p>
        <a class="btn btn-primary" href="#desktop-downloads"
          ><Icon name="download" />下载小伴桌面</a
        ><span class="support">桌面版：Windows / macOS · Android 版浏览与下载壁纸</span>
      </div>
      <WindowsDesktopDemo />
    </section>
    <section class="live" aria-labelledby="live-title">
      <div class="section-head">
        <span class="eyebrow">实机效果</span>
        <h2 id="live-title">动态视频壁纸，铺满整个桌面</h2>
        <p>
          循环播放的风景视频直接成为桌面背景，再叠上一层飘落的枫叶，小伴就待在右下角陪你。下面是
          macOS 上的真实录屏。
        </p>
      </div>
      <figure class="live-frame">
        <picture>
          <source
            media="(prefers-reduced-motion: reduce)"
            srcset="/desktop/live-desktop-still.webp"
          />
          <img
            src="/desktop/live-desktop.webp"
            width="1280"
            height="720"
            loading="lazy"
            decoding="async"
            alt="macOS 桌面实机录屏：夕阳下的海湾是动态视频壁纸，水面在流动，枫叶从上方飘落，右下角的小伴眯着眼睛笑"
          />
        </picture>
        <figcaption>macOS 实机录屏 · 动态视频壁纸 + 枫叶飘落特效 + 小伴</figcaption>
      </figure>
      <ul class="live-points">
        <li>
          <Icon name="play" :size="22" />
          <div>
            <strong>图片与视频都能用</strong>
            <span>浏览本站在线壁纸，或导入自己的图片与 MP4 视频</span>
          </div>
        </li>
        <li>
          <Icon name="download" :size="22" />
          <div>
            <strong>下载后离线可用</strong>
            <span>壁纸保存在本机，断网也照常播放</span>
          </div>
        </li>
        <li>
          <Icon name="sliders" :size="22" />
          <div>
            <strong>按你的屏幕调整</strong>
            <span>画面适配方式和视频播放速度都能改</span>
          </div>
        </li>
      </ul>
    </section>
    <section class="assistant-feature" aria-labelledby="assistant-title">
      <div class="section-head">
        <span class="eyebrow">超级小伴</span>
        <h2 id="assistant-title">不用切走手头的事，<br />小伴随叫随到。</h2>
        <p>正在看网页，还是埋头写东西？按下快捷键，或点一下屏幕边缘的悬浮球，轻巧的对话框就出来了。打字、说话都可以，对话记录留在这里，需要长聊时再进入完整聊天。</p>
      </div>
      <img class="shot" src="/desktop/assistant.webp" width="1280" height="800" loading="lazy" decoding="async"
        alt="小伴桌面的超级小伴设置：Alt+Space 呼出、屏幕边缘悬浮球、语音提问与回答朗读偏好" />
      <ul class="live-points">
        <li><Icon name="sparkles" :size="22" /><div><strong>快捷键，或点一下</strong><span>默认 Alt+Space；悬浮球常驻屏幕边缘，点开即用，按住还能拖到顺手的位置</span></div></li>
        <li><Icon name="play" :size="22" /><div><strong>想到什么，直接说</strong><span>语音在本机转成文字，回答按你的偏好朗读；语音还没准备好时照常打字</span></div></li>
        <li><Icon name="sliders" :size="22" /><div><strong>按你的习惯来</strong><span>快捷键、呼出方式和朗读偏好都能调整，也可以在任意应用里随时问一句</span></div></li>
      </ul>
    </section>
    <section class="showcase" aria-label="功能介绍">
      <article class="showcase-row">
        <div class="showcase-copy">
          <span class="eyebrow">对话创作</span>
          <h2>说一句话，桌面就变了</h2>
          <p>
            不只挑现成的，也能做你想要的。说一句“做一个春日风筝特效，放到桌面”，小伴会写好特效、保存并应用；动画壁纸、HTML 小组件和小游戏也能按你的想法创作。想换氛围、记一个日子，直接告诉它。
          </p>
          <ul class="ticks">
            <li>查网页、整理文件、运行命令，也能帮你处理日常任务</li>
            <li>支持任务规划、长期记忆与技能；默认权限下，写文件和运行命令前先确认</li>
            <li>作品可以导入、导出资源包，也能分享到论坛</li>
          </ul>
          <RouterLink to="/guide" class="guide-action-link">
            <span>查看 AI 玩法指南与示例咒语 →</span>
          </RouterLink>
        </div>
        <img
          class="shot"
          src="/desktop/chat.webp"
          width="1280"
          height="800"
          loading="lazy"
          decoding="async"
          alt="小伴桌面的聊天页：用户让小伴制作春日风筝桌面特效，小伴创建特效并切换到桌面，列出彩鸢、樱花花瓣、暖阳柔光等六层效果"
        />
      </article>
      <article class="showcase-row reverse">
        <div class="showcase-copy">
          <span class="eyebrow">玩法指南</span>
          <h2>不知道让它做什么？<br />这里有一叠现成的句子</h2>
          <p>
            聊天页的“玩法指南”按现场创作、桌面控制、生活关怀、助手与多模态、模型接入分好类，每个例子都能一键填进输入框，也能复制带走。打开完整手册还有更多提示词和进阶用法。
          </p>
          <RouterLink to="/guide" class="guide-action-link">
            <span>打开在线玩法手册 →</span>
          </RouterLink>
        </div>
        <img
          class="shot"
          src="/desktop/chatguide.webp"
          width="1280"
          height="800"
          loading="lazy"
          decoding="async"
          alt="小伴桌面的 AI 玩法指南弹窗：现场造挂件、下班倒计时便签、暗黑星轨 Canvas 壁纸、春日微风落樱特效等示例与一键尝试按钮"
        />
      </article>
      <article class="showcase-row reverse">
        <div class="showcase-copy">
          <span class="eyebrow">壁纸库</span>
          <h2>风景会动，桌面更有生命力</h2>
          <p>
            在线精选图片与循环视频，或导入自己的图片和 MP4。保存后断网也能用，画面适配和播放速度随你调。壁纸位于图标之下，想回到系统壁纸，一键恢复。
          </p>
        </div>
        <img
          class="shot"
          src="/desktop/wallpapers.webp"
          width="1280"
          height="800"
          loading="lazy"
          decoding="async"
          alt="小伴桌面的壁纸库：使用原生壁纸的入口，以及桌面小星球、一池心灯、暮海、极光之夜等动态壁纸和图片壁纸"
        />
      </article>
      <article class="showcase-row">
        <div class="showcase-copy">
          <span class="eyebrow">桌面特效</span>
          <h2>十二种天气与节庆，落在你的桌面上</h2>
          <p>
            夜色里升起的孔明灯、打旋的枫叶、横过夜空的银河、翩跹的蝴蝶，还有烟花、白雪、细雨、萤火、落樱、流星与星空。特效铺在壁纸之上，换壁纸照常显示，鼠标直接穿过，不挡你工作。密度、速度、大小都能调，也能导入特效包，或让小伴现写一个。
          </p>
        </div>
        <img
          class="shot"
          src="/desktop/effects.webp"
          width="1280"
          height="800"
          loading="lazy"
          decoding="async"
          alt="小伴桌面的桌面特效页：无特效、灯火阑珊、层林尽染、星汉灿烂、庄周梦蝶、火树银花、银装素裹、和风细雨等特效卡片"
        />
      </article>
      <article class="showcase-row reverse">
        <div class="showcase-copy">
          <span class="eyebrow">小伴桌宠</span>
          <h2>会回应你的桌面小伙伴</h2>
          <p>
            一盆会回应你的小植物，安静待在桌面一角。它会望向鼠标，用表情回应点击，久不理它也会犯困、睡着。绿叶或白色小雏菊，选一株喜欢的；可以悬浮陪在身边，也能融进壁纸里。
          </p>
        </div>
        <img
          class="shot"
          src="/desktop/companion.webp"
          width="1280"
          height="800"
          loading="lazy"
          decoding="async"
          alt="小伴桌面的桌面宠物页：表情盆栽互动预览，绿叶与白色小雏菊两种植物外观，以及浮动桌宠和融入壁纸两种陪伴方式"
        />
      </article>
      <article class="showcase-row">
        <div class="showcase-copy">
          <span class="eyebrow">桌面小组件</span>
          <h2>壁纸是风景，小组件是日子</h2>
          <p>
            时钟、天气、待办事项、今日日程、股票行情、电子木鱼、月历与纪念日，八种常用小物件，一抬眼就能看见。可以放多个、随意缩放，支持多显示器与“始终置顶”；按住顶部拖动，点“…”就能调整它的内容和外观，也能让小伴现写一个专属小组件。
          </p>
        </div>
        <img
          class="shot"
          src="/desktop/widgets.webp"
          width="1280"
          height="800"
          loading="lazy"
          decoding="async"
          alt="小伴桌面的桌面组件库：桌面时钟、城市天气、待办事项、今日日程、股票行情、电子木鱼、月历与纪念日"
        />
      </article>
      <div class="more">
        <article v-for="card in moreCards" :key="card.src">
          <img
            :src="card.src"
            width="1280"
            height="800"
            loading="lazy"
            decoding="async"
            :alt="card.alt"
          />
          <span class="eyebrow">{{ card.eyebrow }}</span>
          <h3>{{ card.title }}</h3>
          <p>{{ card.body }}</p>
        </article>
      </div>
      <p class="also">
        桌面功能适用于 Windows 与 macOS 版；Android 版用于浏览、下载壁纸并接收更新提醒。壁纸与本地组件无需登录；小伴 AI 需登录并有可用额度，自己的模型服务按服务商要求配置。语音识别在本机完成，首次使用需下载模型；回答朗读需要联网。
      </p>
    </section>
    <section id="desktop-downloads" class="downloads">
      <h2>把小伴带到桌面</h2>
      <p v-if="loading && !product" role="status">正在加载下载信息…</p>
      <div v-if="error" class="error" role="alert">
        {{ error }} <button class="btn" @click="load">重试</button>
      </div>
      <div class="download-grid">
        <article
          v-for="target in targets.filter(
            (t) => t.id !== 'darwin-x86_64' || product?.downloads[t.id],
          )"
          :key="target.id"
        >
          <h3>{{ target.label }}</h3>
          <template v-if="product?.downloads[target.id]"
            ><span class="version"
              >v{{ product.downloads[target.id].version }}</span
            >
            <p>
              {{
                new Date(
                  product.downloads[target.id].publishedAt,
                ).toLocaleDateString("zh-CN")
              }}
              发布
            </p>
            <a
              class="btn btn-primary"
              :href="product.downloads[target.id].url"
              rel="noopener noreferrer"
              ><Icon name="download" />{{
                target.id === 'android-arm64' ? '下载壁纸手机版' : '下载桌面版'
              }}</a
            >
            <details>
              <summary>更新说明</summary>
              <pre>{{
                product.downloads[target.id].notes || "体验优化。"
              }}</pre>
            </details></template
          >
          <p v-else>
            {{ error ? "下载信息暂不可用" : "安装包准备中，发布后即可下载。" }}
          </p>
        </article>
      </div>
      <p class="requirements">
        Windows 需要 WebView2，缺少时程序会引导准备。macOS 当前面向 Apple
        芯片。Android 支持 Android 10 及以上系统。视频能否播放取决于系统解码支持。
      </p>
      <div class="usage-notes">
        <article>
          <h3>日常留在本机</h3>
          <p>壁纸、组件内容与游戏进度保存在这台设备；AI 对话会把完成任务所需的内容发给你选择的模型服务。屏幕与应用控制需要单独开启。</p>
        </article>
        <article>
          <h3>权限由你掌握</h3>
          <p>默认模式下，写文件和运行命令前先确认；完全信任模式减少询问，高风险操作仍会停下来问。这不是系统沙箱，请只把任务交给信任的来源。</p>
        </article>
      </div>
    </section>
    <section class="community" aria-labelledby="community-title">
      <div>
        <h2 id="community-title">来超级小伴论坛聊聊</h2>
        <p>
          更新日志、使用教程、问题反馈和想要的功能，都可以在论坛的小伴壁纸板块留言，也能看看大家做的特效和桌面。
        </p>
      </div>
      <a
        class="btn btn-primary"
        href="https://chat.xiangban.xyz/c/xiaoban-wallpaper"
        target="_blank"
        rel="noopener noreferrer"
        ><Icon name="external" />进入论坛</a
      >
    </section>
    <section v-if="product?.announcements.length" class="notices">
      <h2>小伴公告</h2>
      <article v-for="notice in product.announcements" :key="notice.id">
        <h3>{{ notice.title }}</h3>
        <p>{{ notice.body }}</p>
      </article>
    </section>
  </main>
</template>
<style scoped>
.desktop-page {
  max-width: 1160px;
  margin: auto;
  padding: 70px var(--gutter) calc(80px + var(--tabbar-h));
}
.hero {
  display: grid;
  grid-template-columns: 0.85fr 1.35fr;
  gap: 48px;
  align-items: center;
}
.eyebrow {
  color: #ad7084;
  font-size: 14px;
  letter-spacing: 0.14em;
}
.intro h1 {
  font-size: clamp(40px, 5vw, 62px);
  margin: 18px 0 10px;
}
.hero-tagline {
  font-size: clamp(24px, 2.9vw, 36px);
  line-height: 1.4;
  letter-spacing: -0.03em;
  margin: 0 0 20px;
}
.hero-description,
.intro p {
  color: var(--text-2);
  font-size: 17px;
  line-height: 1.9;
  margin-bottom: 28px;
}
.assistant-feature {
  margin-top: 88px;
}
.assistant-feature > .shot {
  max-width: 1000px;
  margin: auto;
}
.assistant-feature .live-points {
  max-width: 1000px;
  margin-inline: auto;
}
.usage-notes {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 28px;
  margin: 28px 0 0;
  padding: 24px 0 0;
  border-top: 1px solid var(--line);
}
.usage-notes h3 {
  font-size: 17px;
  margin: 0 0 8px;
}
.usage-notes p {
  font-size: 14px;
  margin: 0;
}
.support {
  display: block;
  color: var(--text-3);
  margin-top: 16px;
  font-size: 13px;
}
.live {
  margin: 96px 0 0;
}
.section-head {
  max-width: 680px;
  margin: 0 auto 32px;
  text-align: center;
}
.section-head h2,
.showcase-copy h2 {
  font-size: clamp(26px, 3.2vw, 36px);
  line-height: 1.3;
  margin: 12px 0 14px;
}
.section-head p,
.showcase-copy p {
  color: var(--text-2);
  font-size: 16px;
  line-height: 1.9;
}
.live-frame {
  margin: 0;
}
.live-frame img,
.shot {
  display: block;
  width: 100%;
  height: auto;
  border: 1px solid var(--line);
  background: var(--surface-2);
  box-shadow: 0 24px 60px rgba(63, 44, 51, 0.16);
}
.live-frame img {
  aspect-ratio: 16 / 9;
  border-radius: 22px;
}
.live-frame figcaption {
  margin-top: 14px;
  color: var(--text-3);
  font-size: 13px;
  text-align: center;
}
.live-points {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 20px;
  margin: 36px 0 0;
  padding: 0;
  list-style: none;
}
.live-points li,
.download-grid article,
.notices article {
  padding: 28px;
  border: 1px solid var(--line);
  border-radius: 20px;
  background: var(--surface);
}
.live-points li {
  display: flex;
  gap: 14px;
  padding: 20px 22px;
}
.live-points svg {
  flex: none;
  color: #ad7084;
  margin-top: 2px;
}
.live-points strong {
  display: block;
  font-size: 16px;
  margin-bottom: 6px;
}
.live-points span {
  color: var(--text-2);
  font-size: 14px;
  line-height: 1.7;
}
.showcase {
  display: grid;
  gap: 88px;
  margin: 104px 0 88px;
}
.showcase-row {
  display: grid;
  grid-template-columns: 0.85fr 1.15fr;
  gap: 56px;
  align-items: center;
}
.showcase-row.reverse {
  grid-template-columns: 1.15fr 0.85fr;
}
.showcase-row.reverse .showcase-copy {
  order: 2;
}
.shot {
  aspect-ratio: 8 / 5;
  border-radius: 16px;
}
.ticks {
  margin: 18px 0 0;
  padding: 0;
  list-style: none;
  display: grid;
  gap: 8px;
}
.ticks li {
  position: relative;
  padding-left: 22px;
  color: var(--text-2);
  font-size: 15px;
  line-height: 1.7;
}
.ticks li::before {
  content: "";
  position: absolute;
  left: 2px;
  top: 0.7em;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #ad7084;
}
.guide-action-link {
  display: inline-block;
  margin-top: 20px;
  color: #d4a373;
  font-size: 15px;
  font-weight: 600;
  text-decoration: none;
  transition: opacity 0.2s;
}
.guide-action-link:hover {
  text-decoration: underline;
  opacity: 0.85;
}
.more {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 24px;
}
.more article {
  padding: 16px 16px 24px;
  border: 1px solid var(--line);
  border-radius: 20px;
  background: var(--surface);
}
.more img {
  display: block;
  width: 100%;
  height: auto;
  aspect-ratio: 8 / 5;
  border: 1px solid var(--line);
  border-radius: 12px;
  background: var(--surface-2);
  margin-bottom: 18px;
}
.more .eyebrow {
  padding: 0 6px;
}
.more h3 {
  font-size: 20px;
  margin: 8px 6px 10px;
}
.more p {
  margin: 0 6px;
  color: var(--text-2);
  font-size: 14px;
  line-height: 1.8;
}
.also {
  color: var(--text-3);
  font-size: 14px;
  line-height: 1.8;
  text-align: center;
}
.downloads p {
  line-height: 1.8;
  color: var(--text-2);
}
.downloads > h2,
.notices > h2 {
  font-size: 30px;
  margin-bottom: 24px;
}
.download-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 24px;
}
.download-grid h3 {
  font-size: 19px;
  margin-bottom: 12px;
}
.download-grid .btn {
  margin: 18px 0;
}
.version {
  color: #ad7084;
}
.requirements {
  margin-top: 20px;
  font-size: 13px;
}
.community {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  margin-top: 56px;
  padding: 28px 32px;
  border: 1px solid var(--line);
  border-radius: 20px;
  background: var(--surface);
}
.community h2 {
  font-size: 22px;
  margin-bottom: 8px;
}
.community p {
  color: var(--text-2);
  line-height: 1.8;
  margin: 0;
}
.community .btn {
  flex: none;
}
.notices {
  margin-top: 50px;
}
.notices article {
  margin-top: 16px;
}
.notices p,
pre {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  color: var(--text-2);
  font-family: inherit;
  line-height: 1.8;
  margin-top: 12px;
}
.error {
  color: #b04b50;
  margin-bottom: 20px;
}
summary {
  cursor: pointer;
  color: var(--text-2);
}
@media (max-width: 768px) {
  .desktop-page {
    padding-top: 30px;
  }
  .hero {
    grid-template-columns: 1fr;
    gap: 30px;
  }
  .live {
    margin-top: 56px;
  }
  .live-frame img {
    border-radius: 14px;
  }
  .live-points {
    grid-template-columns: 1fr;
    gap: 12px;
    margin-top: 24px;
  }
  .showcase {
    gap: 56px;
    margin: 64px 0 56px;
  }
  .showcase-row,
  .showcase-row.reverse {
    grid-template-columns: 1fr;
    gap: 22px;
  }
  .showcase-row.reverse .showcase-copy {
    order: 0;
  }
  .shot {
    border-radius: 12px;
  }
  .more {
    grid-template-columns: 1fr;
    gap: 16px;
  }
  .download-grid {
    grid-template-columns: 1fr;
  }
  .community {
    flex-direction: column;
    align-items: flex-start;
    padding: 22px;
  }
  .intro h1 {
    font-size: 44px;
  }
  .assistant-feature {
    margin-top: 56px;
  }
  .usage-notes {
    grid-template-columns: 1fr;
    gap: 20px;
  }
}
</style>
