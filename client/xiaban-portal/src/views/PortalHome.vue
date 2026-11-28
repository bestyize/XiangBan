<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import {
  BRAND_ICON_URL,
  COMMUNITY_URL,
  CONTACT_URL,
  DOWNLOAD_OPTIONS,
  FILTERS,
  FOOTER_LINKS,
  PRODUCTS,
  type ProductCategory,
  type ProductItem,
} from '../data/products';
import { isInternalPortalUrl, navigate, toInternalPath } from '../lib/router';

const activeFilter = ref<ProductCategory>('all');
const downloadOpen = ref(false);
const activeLightbox = ref<ProductItem | null>(null);
const activeShotIndex = ref(0);

const visibleProducts = computed(() =>
  activeFilter.value === 'all'
    ? PRODUCTS
    : PRODUCTS.filter((item) => item.category === activeFilter.value)
);

const activeShot = computed(() => {
  const item = activeLightbox.value;
  if (!item) return null;
  return item.shots[activeShotIndex.value] ?? item.shots[0] ?? null;
});

function openLightbox(item: ProductItem) {
  activeLightbox.value = item;
  activeShotIndex.value = 0;
}

function closeLightbox() {
  activeLightbox.value = null;
}

function openDownload() {
  downloadOpen.value = true;
}

function closeDownload() {
  downloadOpen.value = false;
}

function onPortalLinkClick(e: MouseEvent, href: string) {
  if (!isInternalPortalUrl(href)) return;
  if (e.metaKey || e.ctrlKey || e.shiftKey || e.altKey || e.button !== 0) return;
  e.preventDefault();
  closeLightbox();
  closeDownload();
  navigate(toInternalPath(href));
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    closeLightbox();
    closeDownload();
  }
}

onMounted(() => {
  document.title = '小伴 · 产品矩阵 | xiaban.xyz';
  window.addEventListener('keydown', onKeydown);
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown);
});
</script>

<template>
  <div class="page">
    <header class="topbar">
      <div class="topbar-in">
        <a href="#top" class="brand" aria-label="小伴产品矩阵首页">
          <img :src="BRAND_ICON_URL" alt="小伴图标" width="28" height="28" />
          <span>小伴</span>
          <span class="brand-domain">xiaban.xyz</span>
        </a>
        <div class="top-actions">
          <a class="top-link" :href="COMMUNITY_URL" target="_blank" rel="noopener noreferrer">社区</a>
          <a class="top-link" :href="CONTACT_URL" target="_blank" rel="noopener noreferrer">交流群</a>
          <button type="button" class="top-btn" @click="openDownload">下载客户端</button>
        </div>
      </div>
    </header>

    <main id="top" class="wrap">
      <section class="hero">
        <div class="hero-left">
          <div class="hero-kicker"><i aria-hidden="true"></i> 小伴产品矩阵 · {{ PRODUCTS.length }} 款独立产品</div>
          <h1>你的 AI 伙伴，与随手可玩的小世界</h1>
          <p>从桌面与手机里的个人 Agent，到免安装即玩的小游戏、趣味测评、社区与 4K 壁纸库。</p>
        </div>
        <div class="filter-bar" role="tablist" aria-label="产品分类">
          <button
            v-for="filter in FILTERS"
            :key="filter.key"
            type="button"
            role="tab"
            :aria-selected="activeFilter === filter.key"
            class="filter-pill"
            :class="{ active: activeFilter === filter.key }"
            @click="activeFilter = filter.key"
          >
            {{ filter.label }} <span>{{ filter.count }}</span>
          </button>
        </div>
      </section>

      <section class="grid" aria-label="小伴产品列表">
        <article v-for="(item, idx) in visibleProducts" :key="item.id" class="card">
          <div class="shot-wrap">
            <a
              class="shot-link"
              :href="item.url"
              :target="isInternalPortalUrl(item.url) ? undefined : '_blank'"
              :rel="isInternalPortalUrl(item.url) ? undefined : 'noopener noreferrer'"
              :aria-label="item.name"
              @click="onPortalLinkClick($event, item.url)"
            >
              <div v-if="item.previewKind === 'dual-phone'" class="shot-studio">
                <div class="phone-shot primary">
                  <img
                    :src="item.previewImages.primary"
                    :alt="item.previewImages.primaryAlt"
                    :loading="idx < 3 ? 'eager' : 'lazy'"
                    decoding="async"
                  />
                </div>
                <div v-if="item.previewImages.secondary" class="phone-shot secondary">
                  <img
                    :src="item.previewImages.secondary"
                    :alt="item.previewImages.secondaryAlt"
                    :loading="idx < 3 ? 'eager' : 'lazy'"
                    decoding="async"
                  />
                </div>
              </div>

              <div v-else-if="item.previewKind === 'game-split'" class="shot-game-split">
                <div class="phone-shot primary game-phone">
                  <img
                    :src="item.previewImages.primary"
                    :alt="item.previewImages.primaryAlt"
                    loading="lazy"
                    decoding="async"
                  />
                </div>
                <div class="game-side-preview">
                  <div v-if="item.previewImages.secondary" class="game-mini-shot">
                    <img
                      :src="item.previewImages.secondary"
                      :alt="item.previewImages.secondaryAlt"
                      loading="lazy"
                      decoding="async"
                    />
                  </div>
                  <div v-if="item.previewImages.tertiary" class="game-mini-shot">
                    <img
                      :src="item.previewImages.tertiary"
                      :alt="item.previewImages.tertiaryAlt"
                      loading="lazy"
                      decoding="async"
                    />
                  </div>
                </div>
              </div>

              <img
                v-else
                class="shot-full"
                :class="{ 'shot-center': item.previewKind === 'full-center' }"
                :src="item.previewImages.primary"
                :alt="item.previewImages.primaryAlt"
                :loading="idx < 3 ? 'eager' : 'lazy'"
                decoding="async"
              />
            </a>

            <span class="shot-badge">{{ item.badge }}</span>
            <button type="button" class="shot-zoom" @click.stop="openLightbox(item)">{{ item.lightboxButtonLabel ?? '看实图' }}</button>
          </div>

          <a
            class="card-body"
            :href="item.url"
            :target="isInternalPortalUrl(item.url) ? undefined : '_blank'"
            :rel="isInternalPortalUrl(item.url) ? undefined : 'noopener noreferrer'"
            @click="onPortalLinkClick($event, item.url)"
          >
            <div class="card-head">
              <div class="card-title">
                <h2>{{ item.name }}</h2>
                <span>{{ item.domainLabel }}</span>
              </div>
              <span class="card-arrow" aria-hidden="true">↗</span>
            </div>
            <p class="card-desc">{{ item.description }}</p>
          </a>
        </article>
      </section>

      <div class="eco-strip">
        <div class="eco-points">
          <span><i aria-hidden="true"></i>统一相伴账号</span>
          <span><i aria-hidden="true"></i>桌面 / 手机 / 网页共享小伴点</span>
          <span><i aria-hidden="true"></i>H5 免安装零广告</span>
        </div>
        <a :href="CONTACT_URL" target="_blank" rel="noopener noreferrer">加入用户交流群 ↗</a>
      </div>

      <footer class="footer">
        <span>© 小伴 · xiaban.xyz</span>
        <div class="footer-links">
          <a
            v-for="link in FOOTER_LINKS"
            :key="link.label"
            :href="link.url"
            :target="isInternalPortalUrl(link.url) ? undefined : '_blank'"
            :rel="isInternalPortalUrl(link.url) ? undefined : 'noopener noreferrer'"
            @click="onPortalLinkClick($event, link.url)"
          >
            {{ link.label }}
          </a>
        </div>
      </footer>
    </main>

    <!-- 实机图放大灯箱 -->
    <div
      v-if="activeLightbox && activeShot"
      class="modal-bg"
      role="dialog"
      aria-modal="true"
      :aria-label="activeLightbox.lightboxTitle"
      @click.self="closeLightbox"
    >
      <div class="modal lightbox">
        <div class="modal-head">
          <h3>{{ activeLightbox.lightboxTitle }}</h3>
          <button type="button" class="modal-close" aria-label="关闭" @click="closeLightbox">✕</button>
        </div>
        <div class="lb-stage">
          <img :src="activeShot.src" :alt="activeShot.label" />
        </div>
        <div class="lb-foot">
          <div class="lb-tabs">
            <button
              v-for="(shot, i) in activeLightbox.shots"
              :key="shot.label"
              type="button"
              class="lb-tab"
              :class="{ active: i === activeShotIndex }"
              @click="activeShotIndex = i"
            >
              {{ shot.label }}
            </button>
          </div>
          <a
            class="lb-open"
            :href="activeLightbox.url"
            :target="isInternalPortalUrl(activeLightbox.url) ? undefined : '_blank'"
            :rel="isInternalPortalUrl(activeLightbox.url) ? undefined : 'noopener noreferrer'"
            @click="onPortalLinkClick($event, activeLightbox.url)"
          >
            打开产品 ↗
          </a>
        </div>
      </div>
    </div>

    <!-- 快捷下载弹窗 -->
    <div
      v-if="downloadOpen"
      class="modal-bg"
      role="dialog"
      aria-modal="true"
      aria-label="下载小伴客户端"
      @click.self="closeDownload"
    >
      <div class="modal">
        <div class="modal-head">
          <h3>下载小伴客户端</h3>
          <button type="button" class="modal-close" aria-label="关闭" @click="closeDownload">✕</button>
        </div>
        <div class="dl-list">
          <a
            v-for="opt in DOWNLOAD_OPTIONS"
            :key="opt.title"
            class="dl-row"
            :href="opt.url"
            :target="isInternalPortalUrl(opt.url) ? undefined : '_blank'"
            :rel="isInternalPortalUrl(opt.url) ? undefined : 'noopener noreferrer'"
            @click="onPortalLinkClick($event, opt.url)"
          >
            <div>
              <strong>{{ opt.title }}</strong>
              <span>{{ opt.subtitle }}</span>
            </div>
            <span class="dl-go">下载 ↗</span>
          </a>
        </div>
        <p class="modal-note">
          iOS 开发中，手机浏览器可先体验
          <a :href="COMMUNITY_URL" target="_blank" rel="noopener noreferrer">小伴社区网页版</a>
        </p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.page { min-height: 100vh; background: var(--canvas); }

.topbar {
  position: sticky;
  top: 0;
  z-index: 60;
  background: rgba(250, 248, 245, 0.86);
  backdrop-filter: blur(18px) saturate(1.4);
  -webkit-backdrop-filter: blur(18px) saturate(1.4);
  border-bottom: 1px solid var(--line);
}
.topbar-in {
  max-width: 1120px;
  margin: 0 auto;
  padding: 0 24px;
  height: 56px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}
.brand {
  display: inline-flex;
  align-items: center;
  gap: 9px;
  font-size: 16.5px;
  font-weight: 740;
  letter-spacing: -0.02em;
}
.brand img {
  width: 28px;
  height: 28px;
  border-radius: 8px;
  box-shadow: 0 3px 10px rgba(242, 92, 114, 0.24);
}
.brand-domain {
  font-family: var(--mono);
  font-size: 11px;
  font-weight: 600;
  color: var(--ink-3);
  background: rgba(24, 23, 26, 0.05);
  padding: 2px 8px;
  border-radius: 999px;
}
.top-actions { display: flex; align-items: center; gap: 8px; }
.top-link {
  height: 32px;
  padding: 0 12px;
  border-radius: 999px;
  font-size: 13px;
  color: var(--ink-2);
  display: inline-flex;
  align-items: center;
  gap: 4px;
  transition: 0.16s;
}
.top-link:hover { color: var(--ink); background: rgba(24, 23, 26, 0.05); }
.top-btn {
  height: 32px;
  padding: 0 14px;
  border-radius: 999px;
  background: var(--ink);
  color: #fff;
  font-size: 12.5px;
  font-weight: 620;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  transition: background 0.16s, transform 0.16s;
}
.top-btn:hover { background: #2e2b33; transform: translateY(-1px); }

.wrap { width: min(1120px, calc(100% - 48px)); margin: 0 auto; }

.hero {
  padding: 32px 0 22px;
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 20px;
  flex-wrap: wrap;
}
.hero-left { max-width: 560px; }
.hero-kicker {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  font-size: 12px;
  font-weight: 620;
  color: var(--ink-3);
  margin-bottom: 8px;
}
.hero-kicker i {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--coral-grad);
  display: block;
}
.hero h1 {
  font-size: clamp(24px, 3vw, 34px);
  line-height: 1.2;
  letter-spacing: -0.035em;
  font-weight: 760;
  margin-bottom: 6px;
}
.hero p { font-size: 14px; color: var(--ink-2); line-height: 1.6; }

.filter-bar {
  display: inline-flex;
  padding: 3px;
  border-radius: 999px;
  background: var(--canvas-soft);
  border: 1px solid var(--line);
  gap: 2px;
}
.filter-pill {
  height: 30px;
  padding: 0 12px;
  border-radius: 999px;
  font-size: 12.5px;
  font-weight: 550;
  color: var(--ink-2);
  display: inline-flex;
  align-items: center;
  gap: 5px;
  transition: 0.18s var(--ease);
  white-space: nowrap;
}
.filter-pill:hover { color: var(--ink); }
.filter-pill.active {
  background: var(--surface);
  color: var(--ink);
  font-weight: 650;
  box-shadow: 0 2px 6px rgba(24, 20, 24, 0.08);
}
.filter-pill span { font-size: 11px; font-family: var(--mono); color: var(--ink-3); }
.filter-pill.active span { color: var(--coral); }

.grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 16px;
  padding-bottom: 28px;
}

.card {
  position: relative;
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 18px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow-sm);
  transition: transform 0.24s var(--ease), box-shadow 0.24s var(--ease), border-color 0.2s;
}
.card:hover {
  transform: translateY(-3px);
  box-shadow: var(--shadow-md);
  border-color: var(--line-strong);
}

.shot-wrap {
  position: relative;
  aspect-ratio: 16 / 10;
  background: var(--canvas-soft);
  overflow: hidden;
  border-bottom: 1px solid var(--line);
}
.shot-link { display: block; width: 100%; height: 100%; }

.shot-full {
  width: 100%;
  height: 100%;
  object-fit: cover;
  object-position: top center;
  transition: transform 0.4s var(--ease);
}
.shot-center { object-position: center; }
.card:hover .shot-full { transform: scale(1.03); }

.shot-studio {
  width: 100%;
  height: 100%;
  padding: 12px 14px 0;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  gap: 10px;
  background:
    radial-gradient(65% 80% at 50% 0%, rgba(255, 118, 134, 0.12), transparent 72%),
    #f1ece5;
}

.phone-shot {
  width: 46%;
  height: 95%;
  border-radius: 14px 14px 0 0;
  overflow: hidden;
  box-shadow: 0 10px 24px rgba(24, 20, 28, 0.18);
  border: 2.5px solid #1d1b20;
  border-bottom: 0;
  background: #fff;
  transition: transform 0.35s var(--ease);
  flex-shrink: 0;
}
.phone-shot.game-phone { width: 45%; }
.phone-shot img { width: 100%; height: 100%; object-fit: cover; object-position: top center; }
.phone-shot.secondary {
  width: 40%;
  height: 84%;
  opacity: 0.96;
  transform: translateY(4px);
}
.card:hover .phone-shot.primary { transform: translateY(-3px); }
.card:hover .phone-shot.secondary { transform: translateY(0); }

.shot-game-split {
  width: 100%;
  height: 100%;
  padding: 12px 14px 0;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  gap: 10px;
  background:
    radial-gradient(65% 80% at 75% 10%, rgba(251, 146, 60, 0.16), transparent 70%),
    #f4efe8;
}
.game-side-preview {
  width: 44%;
  align-self: center;
  display: flex;
  flex-direction: column;
  gap: 7px;
  padding-bottom: 10px;
}
.game-mini-shot {
  width: 100%;
  aspect-ratio: 16 / 10;
  border-radius: 9px;
  overflow: hidden;
  border: 1.5px solid rgba(255, 255, 255, 0.9);
  box-shadow: 0 6px 16px rgba(24, 20, 24, 0.14);
  transition: transform 0.3s var(--ease);
}
.game-mini-shot img { width: 100%; height: 100%; object-fit: cover; }
.card:hover .game-mini-shot:first-child { transform: translateY(-2px); }
.card:hover .game-mini-shot:last-child { transform: translateY(2px); }

.shot-badge {
  position: absolute;
  top: 10px;
  left: 10px;
  z-index: 4;
  padding: 3px 8px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 620;
  color: #fff;
  background: rgba(20, 19, 24, 0.62);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  letter-spacing: 0.01em;
}
.shot-zoom {
  position: absolute;
  top: 9px;
  right: 9px;
  z-index: 5;
  height: 26px;
  padding: 0 9px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  color: #fff;
  background: rgba(20, 19, 24, 0.56);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  opacity: 0;
  transform: translateY(-2px);
  transition: opacity 0.18s, transform 0.18s, background 0.16s;
}
.card:hover .shot-zoom,
.shot-zoom:focus-visible {
  opacity: 1;
  transform: translateY(0);
}
.shot-zoom:hover { background: rgba(20, 19, 24, 0.85); }

.card-body {
  padding: 13px 15px 15px;
  display: flex;
  flex-direction: column;
  gap: 5px;
  flex: 1;
}
.card-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
}
.card-title {
  display: inline-flex;
  align-items: baseline;
  gap: 6px;
  min-width: 0;
}
.card-title h2 {
  font-size: 15.5px;
  font-weight: 720;
  letter-spacing: -0.02em;
  white-space: nowrap;
}
.card-title span {
  font-family: var(--mono);
  font-size: 11px;
  color: var(--ink-3);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.card-arrow {
  font-size: 14px;
  color: var(--ink-3);
  flex-shrink: 0;
  transition: transform 0.2s var(--ease), color 0.2s;
}
.card:hover .card-arrow { color: var(--coral); transform: translate(2px, -2px); }
.card-desc {
  font-size: 12.8px;
  line-height: 1.55;
  color: var(--ink-2);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.eco-strip {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  flex-wrap: wrap;
  padding: 14px 18px;
  border-radius: 14px;
  background: var(--surface);
  border: 1px solid var(--line);
  margin-bottom: 24px;
  font-size: 12.5px;
  color: var(--ink-2);
}
.eco-points { display: flex; align-items: center; gap: 18px; flex-wrap: wrap; }
.eco-points span { display: inline-flex; align-items: center; gap: 6px; }
.eco-points i { width: 5px; height: 5px; border-radius: 50%; background: var(--coral); display: block; }
.eco-strip a { font-weight: 620; color: var(--ink); white-space: nowrap; }
.eco-strip a:hover { color: var(--coral); }

.footer {
  border-top: 1px solid var(--line);
  padding: 18px 0 34px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  flex-wrap: wrap;
  font-size: 12.5px;
  color: var(--ink-3);
}
.footer-links { display: flex; flex-wrap: wrap; gap: 8px 18px; }
.footer-links a:hover { color: var(--ink); }

.modal-bg {
  position: fixed;
  inset: 0;
  z-index: 300;
  background: rgba(18, 16, 22, 0.56);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}
.modal {
  width: min(440px, 100%);
  background: var(--surface);
  border-radius: 22px;
  padding: 22px;
  box-shadow: var(--shadow-lg);
  border: 1px solid var(--line);
}
.modal.lightbox { width: min(820px, 100%); padding: 18px; }
.modal-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 14px;
}
.modal-head h3 { font-size: 16px; font-weight: 720; }
.modal-close {
  width: 30px;
  height: 30px;
  border-radius: 50%;
  background: var(--canvas-soft);
  display: grid;
  place-items: center;
  font-size: 13px;
  color: var(--ink-2);
}
.modal-close:hover { color: var(--ink); background: #e7e1d8; }
.lb-stage {
  background: var(--canvas-soft);
  border-radius: 14px;
  overflow: hidden;
  border: 1px solid var(--line);
  display: flex;
  align-items: center;
  justify-content: center;
  max-height: 62vh;
}
.lb-stage img { max-height: 62vh; width: auto; max-width: 100%; object-fit: contain; }
.lb-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-top: 12px;
  flex-wrap: wrap;
}
.lb-tabs { display: flex; gap: 6px; flex-wrap: wrap; }
.lb-tab {
  height: 28px;
  padding: 0 11px;
  border-radius: 999px;
  font-size: 12px;
  background: var(--canvas-soft);
  color: var(--ink-2);
  font-weight: 550;
}
.lb-tab.active { background: var(--ink); color: #fff; }
.lb-open {
  height: 32px;
  padding: 0 14px;
  border-radius: 999px;
  background: var(--coral-grad);
  color: #fff;
  font-size: 12.5px;
  font-weight: 650;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.dl-list { display: flex; flex-direction: column; gap: 8px; margin-bottom: 12px; }
.dl-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 14px;
  border-radius: 14px;
  background: var(--canvas);
  border: 1px solid var(--line);
  transition: 0.16s;
}
.dl-row:hover { background: var(--coral-soft); border-color: rgba(238, 79, 109, 0.25); }
.dl-row strong { display: block; font-size: 14px; }
.dl-row span { font-size: 12px; color: var(--ink-3); }
.dl-go {
  font-size: 12px;
  font-weight: 650;
  background: #fff;
  border: 1px solid var(--line-strong);
  padding: 5px 11px;
  border-radius: 999px;
  white-space: nowrap;
}
.modal-note { font-size: 12px; color: var(--ink-3); text-align: center; }
.modal-note a { color: var(--ink); text-decoration: underline; }

@media (max-width: 920px) {
  .grid { grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 14px; }
  .wrap { width: min(1120px, calc(100% - 32px)); }
  .topbar-in { padding: 0 16px; }
}
@media (max-width: 560px) {
  .grid { grid-template-columns: 1fr; gap: 12px; }
  .brand-domain, .top-link { display: none; }
  .hero { padding: 22px 0 16px; }
  .shot-wrap { aspect-ratio: 16 / 9; }
  .shot-zoom { opacity: 1; transform: none; }
}
</style>
