<script setup lang="ts">
import { onActivated, ref } from "vue";
import Icon from "@/components/Icon.vue";
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
onActivated(load);
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
        ><span class="support">Windows / macOS · 无须注册账户</span>
      </div>
      <div class="room" aria-label="小伴桌面示意插画">
        <div class="window"><span>✦</span></div>
        <div class="companion"><i /><i /><b>⌣</b></div>
        <div class="desk" />
        <span class="room-caption">今天，也有小伴陪着你。</span>
      </div>
    </section>
    <section class="features">
      <article>
        <Icon name="heart" :size="30" />
        <h2>会回应的陪伴</h2>
        <p>望向你的鼠标，回应你的轻触。悬浮在桌面，或安静住进壁纸。</p>
      </article>
      <article>
        <Icon name="image" :size="30" />
        <h2>喜欢的风景</h2>
        <p>浏览本站图片与动态视频，下载后应用到桌面，也能导入自己的媒体。</p>
      </article>
      <article>
        <Icon name="device" :size="30" />
        <h2>轻盈的日常</h2>
        <p>下载的壁纸离线可用，支持开机启动、画面适配和视频播放速度调整。</p>
      </article>
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
        芯片。视频能否播放取决于系统解码支持。
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
  grid-template-columns: 1.1fr 1fr;
  gap: 60px;
  align-items: center;
}
.eyebrow {
  color: #eac7a0;
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
.room {
  height: 360px;
  position: relative;
  overflow: hidden;
  border: 1px solid #ffffff16;
  border-radius: 28px;
  background: linear-gradient(140deg, #302e47, #403840 65%, #675446);
  box-shadow: 0 20px 100px #e2ac7330;
}
.window {
  position: absolute;
  top: 38px;
  right: 38px;
  width: 106px;
  height: 136px;
  border: 8px solid #736357;
  border-radius: 60px 60px 0 0;
  background: #1b253d;
  box-shadow: 0 0 80px #dec79530;
}
.window span {
  position: absolute;
  right: 16px;
  top: 24px;
  color: #f5d6a4;
  font-size: 28px;
}
.desk {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 60px;
  height: 12px;
  background: #9d7a60;
  box-shadow: 0 20px 50px #0005;
}
.companion {
  position: absolute;
  z-index: 1;
  left: 25%;
  bottom: 72px;
  width: 110px;
  height: 120px;
  border-radius: 48% 48% 40% 40%;
  background: #ffedd1;
  box-shadow: 0 0 80px #ffd99b66;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 26px;
}
.companion i {
  width: 9px;
  height: 12px;
  border-radius: 50%;
  background: #665447;
}
.companion b {
  position: absolute;
  bottom: 28px;
  color: #a98065;
  font-size: 26px;
}
.room-caption {
  position: absolute;
  bottom: 22px;
  width: 100%;
  text-align: center;
  color: #efd9c4;
  font-size: 13px;
}
.features {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 30px;
  margin: 70px 0;
}
.features article,
.download-grid article,
.notices article {
  padding: 28px;
  border: 1px solid var(--line);
  border-radius: 20px;
  background: var(--surface);
}
.features svg {
  color: #eac7a0;
}
.features h2 {
  font-size: 19px;
  margin: 16px 0 10px;
}
.features p,
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
  color: #eac7a0;
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
  color: #ffb2a0;
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
  .room {
    height: 290px;
  }
  .features {
    grid-template-columns: 1fr;
    margin: 36px 0;
    gap: 14px;
  }
  .download-grid {
    grid-template-columns: 1fr;
  }
  .intro h1 {
    font-size: 48px;
  }
}
</style>
