<template>
  <div class="modal-backdrop" @click="$emit('close')">
    <div class="modal-content" @click.stop>
      <!-- 弹窗顶栏 -->
      <div class="modal-top">
        <div class="app-main-meta">
          <div class="icon-box">
            <img :src="getAppIconUrl(app)" :alt="app.name" class="detail-icon" @error="onImgError" />
          </div>
          <div class="title-box">
            <div class="title-with-version">
              <h2>{{ app.name }}</h2>
              <span v-if="app.version" class="version-badge">v{{ app.version }}</span>
            </div>
            <p class="summary-text">{{ app.summary }}</p>
            <div class="badges-row">
              <span class="category-badge">{{ app.category_name || '精选' }}</span>
              <span v-for="plat in (app.platforms || [])" :key="plat" class="platform-badge">
                {{ formatPlatform(plat) }}
              </span>
              <span v-if="app.pricing_type === 'open_source'" class="badge badge-green">开源</span>
              <span v-else-if="app.pricing_type === 'free'" class="badge badge-blue">免费</span>
              <span v-else-if="app.pricing_type === 'freemium'" class="badge badge-gray">免费+内购</span>
              <span v-else class="badge badge-purple">买断</span>
            </div>
          </div>
        </div>
        <button class="btn-close" @click="$emit('close')">
          <X :size="18" />
        </button>
      </div>

      <!-- 弹窗主体 -->
      <div class="modal-body">
        <!-- 精选推荐理由（纯粹克制） -->
        <div class="editorial-box">
          <p class="editorial-text">“{{ app.comment || app.summary }}”</p>
          <div class="editorial-by">— 由 {{ app.referrer || '小伴精选编辑部' }} 推荐</div>
        </div>

        <!-- 核心：直接下载安装包（免跳官网） -->
        <div class="downloads-section">
          <div class="section-title">
            <Download :size="15" class="title-icon" />
            <span>直接下载安装包</span>
          </div>

          <div class="direct-downloads-list">
            <!-- macOS 安装包 -->
            <a
              v-if="app.mac_download_url"
              :href="app.mac_download_url"
              class="dl-platform-btn dl-mac"
              @click="onDownload"
            >
              <div class="dl-icon-area">🍏</div>
              <div class="dl-text-area">
                <span class="dl-title">直接下载 macOS 安装包</span>
                <span class="dl-desc">{{ app.file_size ? app.file_size + ' · ' : '' }}v{{ app.version || '最新' }} (.dmg / .zip 直链)</span>
              </div>
              <Download :size="16" class="dl-action-icon" />
            </a>

            <!-- Windows 安装包 -->
            <a
              v-if="app.win_download_url"
              :href="app.win_download_url"
              class="dl-platform-btn dl-win"
              @click="onDownload"
            >
              <div class="dl-icon-area">🪟</div>
              <div class="dl-text-area">
                <span class="dl-title">直接下载 Windows 安装包</span>
                <span class="dl-desc">{{ app.file_size ? app.file_size + ' · ' : '' }}v{{ app.version || '最新' }} (.exe / .msi 直链)</span>
              </div>
              <Download :size="16" class="dl-action-icon" />
            </a>

            <!-- Linux 安装包 -->
            <a
              v-if="app.linux_download_url"
              :href="app.linux_download_url"
              class="dl-platform-btn dl-linux"
              @click="onDownload"
            >
              <div class="dl-icon-area">🐧</div>
              <div class="dl-text-area">
                <span class="dl-title">直接下载 Linux 安装包</span>
                <span class="dl-desc">v{{ app.version || '最新' }} (.AppImage / .deb 直链)</span>
              </div>
              <Download :size="16" class="dl-action-icon" />
            </a>

            <!-- 若暂无具体平台直链，提供官方直达通道 -->
            <a
              v-if="!app.mac_download_url && !app.win_download_url && !app.linux_download_url"
              :href="app.download_url"
              target="_blank"
              class="dl-platform-btn dl-default"
              @click="onDownload"
            >
              <div class="dl-icon-area">📦</div>
              <div class="dl-text-area">
                <span class="dl-title">官方通道直接下载</span>
                <span class="dl-desc">v{{ app.version || '最新版' }}</span>
              </div>
              <Download :size="16" class="dl-action-icon" />
            </a>
          </div>
        </div>

        <!-- 极客终端命令 (若存在) -->
        <div class="cmd-row" v-if="app.brew_cask">
          <div class="cmd-box">
            <div class="cmd-code">
              <Terminal :size="14" class="terminal-icon" />
              <code>brew install --cask {{ app.brew_cask }}</code>
            </div>
            <button class="btn-copy" @click="copyText(`brew install --cask ${app.brew_cask}`)">
              <Copy :size="12" />
              <span>{{ copiedBrew ? '已复制' : '复制命令' }}</span>
            </button>
          </div>
        </div>

        <!-- 底部轻量辅助外链 -->
        <div class="secondary-links-row" v-if="app.official_url || app.github_url || app.appstore_url">
          <a v-if="app.official_url" :href="app.official_url" target="_blank" class="sec-link">
            <Globe :size="13" />
            <span>官网主页</span>
          </a>
          <a v-if="app.github_url" :href="app.github_url" target="_blank" class="sec-link">
            <Github :size="13" />
            <span>GitHub 源码</span>
            <span v-if="app.github_stars" class="stars-tag">★ {{ app.github_stars }}</span>
          </a>
          <a v-if="app.appstore_url" :href="app.appstore_url" target="_blank" class="sec-link">
            <AppWindow :size="13" />
            <span>App Store</span>
          </a>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import type { AppItem } from '../types';
import {
  X,
  Terminal,
  Copy,
  Download,
  Globe,
  Github,
  AppWindow,
} from 'lucide-vue-next';
import { api } from '../api';
import { handleImageFallback, getAppIconUrl } from '../utils/icon';

const props = defineProps<{
  app: AppItem;
}>();

defineEmits<{
  (e: 'close'): void;
}>();

const copiedBrew = ref(false);

function onImgError(e: Event) {
  handleImageFallback(e, props.app.name, props.app.slug);
}

function formatPlatform(plat: string) {
  const map: Record<string, string> = {
    mac: 'macOS',
    windows: 'Windows',
    linux: 'Linux',
    web: 'Web',
    ios: 'iOS',
    android: 'Android',
  };
  return map[plat] || plat;
}

async function copyText(text: string) {
  await navigator.clipboard.writeText(text);
  copiedBrew.value = true;
  setTimeout(() => (copiedBrew.value = false), 2000);
}

function onDownload() {
  api.trackDownload(props.app.slug);
}
</script>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.45);
  backdrop-filter: blur(4px);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.modal-content {
  background: #ffffff;
  width: 100%;
  max-width: 540px;
  max-height: 85vh;
  border-radius: 14px;
  box-shadow: 0 20px 40px rgba(15, 23, 42, 0.16);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  animation: modalEnter 0.18s ease-out;
}

@keyframes modalEnter {
  from {
    opacity: 0;
    transform: scale(0.98) translateY(6px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

.modal-top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  padding: 22px 24px 16px;
  border-bottom: 1px solid #f1f5f9;
}

.app-main-meta {
  display: flex;
  align-items: center;
  gap: 14px;
  flex: 1;
  min-width: 0;
}

.icon-box {
  width: 52px;
  height: 52px;
  min-width: 52px;
  border-radius: 12px;
  overflow: hidden;
  border: 1px solid #f1f5f9;
  background: #f8fafc;
}

.detail-icon {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.title-box {
  flex: 1;
  min-width: 0;
}

.title-with-version {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 3px;
}

.title-with-version h2 {
  font-size: 18px;
  font-weight: 700;
  color: #0f172a;
}

.version-badge {
  font-size: 11px;
  color: #64748b;
  background: #f1f5f9;
  padding: 1px 6px;
  border-radius: 4px;
  font-family: monospace;
}

.summary-text {
  font-size: 12px;
  color: #64748b;
  margin-bottom: 6px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.badges-row {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}

.category-badge {
  font-size: 11px;
  color: #475569;
  background: #f1f5f9;
  padding: 1px 6px;
  border-radius: 4px;
}

.platform-badge {
  font-size: 11px;
  color: #2563eb;
  background: #eff6ff;
  padding: 1px 6px;
  border-radius: 4px;
}

.btn-close {
  color: #94a3b8;
  padding: 4px;
  border-radius: 6px;
  transition: all 0.15s ease;
}

.btn-close:hover {
  background: #f1f5f9;
  color: #0f172a;
}

.modal-body {
  padding: 20px 24px 24px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.editorial-box {
  background: #f8fafc;
  border-left: 3px solid #2563eb;
  border-radius: 0 8px 8px 0;
  padding: 12px 14px;
}

.editorial-text {
  font-size: 13px;
  line-height: 1.55;
  color: #1e293b;
  margin-bottom: 6px;
}

.editorial-by {
  font-size: 11px;
  color: #2563eb;
  text-align: right;
  font-weight: 500;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 600;
  color: #334155;
  margin-bottom: 8px;
}

.title-icon {
  color: #2563eb;
}

.direct-downloads-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.dl-platform-btn {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 11px 16px;
  border-radius: 10px;
  background: #2563eb;
  color: #ffffff;
  transition: all 0.15s ease;
  box-shadow: 0 1px 3px rgba(37, 99, 235, 0.15);
}

.dl-platform-btn:hover {
  background: #1d4ed8;
  transform: translateY(-1px);
  box-shadow: 0 4px 10px rgba(37, 99, 235, 0.25);
}

.dl-icon-area {
  font-size: 20px;
  line-height: 1;
}

.dl-text-area {
  display: flex;
  flex-direction: column;
  flex: 1;
}

.dl-title {
  font-size: 13px;
  font-weight: 600;
}

.dl-desc {
  font-size: 11px;
  opacity: 0.85;
}

.dl-action-icon {
  opacity: 0.85;
}

.cmd-row {
  margin-top: -2px;
}

.cmd-box {
  background: #1e293b;
  color: #f8fafc;
  padding: 8px 12px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.cmd-code {
  display: flex;
  align-items: center;
  gap: 8px;
}

.terminal-icon {
  color: #94a3b8;
}

.cmd-box code {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 12px;
}

.btn-copy {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: #94a3b8;
  background: #334155;
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.15s ease;
  white-space: nowrap;
}

.btn-copy:hover {
  background: #475569;
  color: #ffffff;
}

.secondary-links-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding-top: 10px;
  border-top: 1px dashed #e2e8f0;
}

.sec-link {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  color: #64748b;
  font-size: 12px;
  transition: all 0.15s ease;
}

.sec-link:hover {
  color: #0f172a;
}

.stars-tag {
  font-size: 10px;
  color: #2563eb;
  background: #eff6ff;
  padding: 1px 4px;
  border-radius: 3px;
}
</style>
