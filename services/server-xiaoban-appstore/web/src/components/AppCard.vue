<template>
  <div class="clean-card" @click="$emit('select', app)">
    <div class="card-top">
      <div class="icon-wrap">
        <img :src="getAppIconUrl(app)" :alt="app.name" class="app-icon" @error="onImgError" />
      </div>
      <div class="title-meta">
        <div class="name-line">
          <h3 class="app-name">{{ app.name }}</h3>
          <span v-if="app.version" class="version-tag">v{{ app.version }}</span>
        </div>
        <div class="sub-line">
          <span>{{ app.category_name || '精选' }}</span>
          <span class="dot">·</span>
          <span>{{ formatPlatforms(app.platforms) }}</span>
        </div>
      </div>
      <div class="pricing-pill">
        <span v-if="app.pricing_type === 'open_source'" class="badge badge-green">开源</span>
        <span v-else-if="app.pricing_type === 'free'" class="badge badge-blue">免费</span>
        <span v-else-if="app.pricing_type === 'freemium'" class="badge badge-gray">免费+内购</span>
        <span v-else class="badge badge-purple">买断</span>
      </div>
    </div>

    <!-- 纯粹克制的编辑短评 (2 行限制) -->
    <p class="card-comment">{{ app.comment || app.summary }}</p>

    <!-- 底栏极简操作区 -->
    <div class="card-bottom">
      <div class="trust-mark">
        <span v-if="app.github_stars && app.github_stars > 0" class="star-text">
          ★ {{ formatStars(app.github_stars) }}
        </span>
        <span v-else-if="hasBadge('safe')" class="safe-text">
          <ShieldCheck :size="13" /> 已实测认证
        </span>
        <span v-else class="referrer-text">
          {{ app.referrer || '精选收录' }}
        </span>
      </div>

      <button class="btn-get" @click.stop="$emit('select', app)">
        <Download :size="13" />
        <span>获取</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { AppItem } from '../types';
import { ShieldCheck, Download } from 'lucide-vue-next';
import { handleImageFallback, getAppIconUrl } from '../utils/icon';

const props = defineProps<{
  app: AppItem;
}>();

defineEmits<{
  (e: 'select', app: AppItem): void;
}>();

function hasBadge(badge: string) {
  return Array.isArray(props.app.badges) && props.app.badges.includes(badge);
}

function formatPlatforms(platforms: string[]) {
  if (!Array.isArray(platforms) || platforms.length === 0) return '全平台';
  const map: Record<string, string> = {
    mac: 'macOS',
    windows: 'Win',
    linux: 'Linux',
    web: 'Web',
    ios: 'iOS',
    android: 'Android',
  };
  return platforms.map((p) => map[p] || p).join(' / ');
}

function formatStars(stars: number) {
  if (stars >= 1000) return (stars / 1000).toFixed(1) + 'k';
  return stars.toString();
}

function onImgError(e: Event) {
  handleImageFallback(e, props.app.name, props.app.slug);
}
</script>

<style scoped>
.clean-card {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  padding: 18px 20px;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  gap: 12px;
  height: 100%;
}

.clean-card:hover {
  transform: translateY(-3px);
  border-color: #cbd5e1;
  box-shadow: 0 10px 25px -5px rgba(15, 23, 42, 0.08), 0 4px 6px -2px rgba(15, 23, 42, 0.02);
}

.card-top {
  display: flex;
  align-items: center;
  gap: 12px;
}

.icon-wrap {
  width: 44px;
  height: 44px;
  min-width: 44px;
  border-radius: 10px;
  overflow: hidden;
  border: 1px solid #f1f5f9;
  background: #f8fafc;
}

.app-icon {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.title-meta {
  flex: 1;
  min-width: 0;
}

.name-line {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 2px;
}

.app-name {
  font-size: 15px;
  font-weight: 600;
  color: #0f172a;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.version-tag {
  font-size: 11px;
  color: #94a3b8;
}

.sub-line {
  font-size: 12px;
  color: #64748b;
  display: flex;
  align-items: center;
  gap: 5px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.dot {
  color: #cbd5e1;
}

.pricing-pill {
  flex-shrink: 0;
}

.card-comment {
  font-size: 13px;
  line-height: 1.55;
  color: #475569;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  text-overflow: ellipsis;
  margin: 0;
}

.card-bottom {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: 10px;
  border-top: 1px solid #f8fafc;
}

.trust-mark {
  font-size: 12px;
  color: #94a3b8;
  display: flex;
  align-items: center;
  gap: 4px;
}

.safe-text {
  color: #059669;
  display: flex;
  align-items: center;
  gap: 3px;
  font-weight: 500;
}

.star-text {
  color: #475569;
  font-weight: 500;
}

.referrer-text {
  color: #94a3b8;
}

.btn-get {
  background: #f1f5f9;
  color: #2563eb;
  font-size: 12px;
  font-weight: 600;
  padding: 5px 14px;
  border-radius: 20px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  transition: all 0.15s ease;
}

.btn-get:hover {
  background: #2563eb;
  color: #ffffff;
}
</style>
