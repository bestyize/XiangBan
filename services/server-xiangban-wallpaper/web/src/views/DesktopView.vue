<script setup lang="ts">
import { nextTick, onActivated, ref } from "vue";
import { useRoute } from "vue-router";
import Icon from "@/components/Icon.vue";
import WindowsDesktopDemo from "@/components/WindowsDesktopDemo.vue";
import { getDesktopProduct, targets, type DesktopProduct } from "@/desktop";
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
        <p>
          {{
            product?.introduction ||
            "会回应的桌面小伙伴，搭配喜欢的图片与动态壁纸，让工作和休息多一点温暖。"
          }}
        </p>
        <a class="btn btn-primary" href="#desktop-downloads"
          ><Icon name="download" />下载小伴桌面</a
        ><span class="support">Windows / macOS / Android · 无须注册账户</span>
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
    <section class="showcase" aria-label="功能介绍">
      <article class="showcase-row">
        <div class="showcase-copy">
          <span class="eyebrow">桌面特效</span>
          <h2>雨雪花瓣，落在你的桌面上</h2>
          <p>
            下雨、下雪、萤火虫、花瓣飘落、枫叶飘落、流星划过、繁星，七种特效铺在壁纸之上。换壁纸时照常显示，密度、速度和大小都能调，鼠标点击照样穿过去，不挡你工作。
          </p>
        </div>
        <img
          class="shot"
          src="/desktop/effects.webp"
          width="1093"
          height="738"
          loading="lazy"
          decoding="async"
          alt="小伴桌面的桌面特效页：下雨、下雪、萤火虫、花瓣飘落、枫叶飘落等特效卡片，当前选中枫叶飘落"
        />
      </article>
      <article class="showcase-row reverse">
        <div class="showcase-copy">
          <span class="eyebrow">小伴桌宠</span>
          <h2>会回应你的桌面小伙伴</h2>
          <p>
            小伴会望向你的鼠标，点一下就换个表情回应你，拖一拖就能换个位置；一会儿不理它，它还会犯困、睡着。猫耳、兔耳、盆栽三款外壳随心换，可以悬浮在桌面上，也能安静住进壁纸里。
          </p>
        </div>
        <img
          class="shot"
          src="/desktop/companion.webp"
          width="1093"
          height="738"
          loading="lazy"
          decoding="async"
          alt="小伴桌面的桌面小伴页：左侧是笑眯眯的猫耳小伴互动预览，右侧可在猫耳、兔耳、盆栽三款外壳之间切换"
        />
      </article>
      <article class="showcase-row">
        <div class="showcase-copy">
          <span class="eyebrow">游戏空间</span>
          <h2>累了，就在桌面上玩一会儿</h2>
          <p>
            俄罗斯方块、2048、奶茶大师、小伴贪吃蛇、水墨剑客等七款小游戏，打开就能玩。全部离线可玩，不用注册账户，水墨剑客还会记住你的进度。
          </p>
        </div>
        <img
          class="shot"
          src="/desktop/games.webp"
          width="1093"
          height="738"
          loading="lazy"
          decoding="async"
          alt="小伴桌面的游戏空间页：俄罗斯方块、2048、消失高手、奶茶大师、小伴贪吃蛇、水墨剑客等本地游戏卡片"
        />
      </article>
      <p class="also">
        还有在线壁纸库、下班 / 休息 / 天气关怀提醒和开机自动启动。以上功能适用于 Windows 与 macOS
        版；Android 版用于浏览和下载壁纸。
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
              ><Icon name="download" />下载</a
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
  font-size: clamp(44px, 6vw, 76px);
  margin: 18px 0;
}
.intro p {
  color: var(--text-2);
  font-size: 18px;
  line-height: 1.9;
  margin-bottom: 28px;
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
  aspect-ratio: 1093 / 738;
  border-radius: 16px;
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
  .download-grid {
    grid-template-columns: 1fr;
  }
  .intro h1 {
    font-size: 48px;
  }
}
</style>
