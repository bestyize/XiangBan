<script setup lang="ts">
import { ref, watch } from 'vue';
import type { EvaluatedResult, ResultProfile, TestDefinition } from '../engine/types';
import { renderResultPosterDataUrl, type PosterRatio } from '../engine/posterCanvas';

const props = defineProps<{
  open: boolean;
  test: TestDefinition;
  result: EvaluatedResult;
  profile: ResultProfile;
}>();

const emit = defineEmits<{
  close: [];
  copyLink: [];
  systemShare: [imageDataUrl: string, filename: string];
  downloaded: [msg: string];
}>();

const ratio = ref<PosterRatio>('916');
const previewDataUrl = ref('');

function currentFilename() {
  return `vendme-${props.test.id}-${props.profile.code}-${ratio.value === '34' ? '3x4' : '9x16'}.png`;
}

function rebuildPreview() {
  if (!props.open) return;
  previewDataUrl.value = renderResultPosterDataUrl(
    props.test,
    props.result,
    props.profile,
    ratio.value
  );
}

watch(
  () => [props.open, props.test.id, props.profile.code, ratio.value],
  () => rebuildPreview(),
  { immediate: true }
);

function downloadPng() {
  if (!previewDataUrl.value) rebuildPreview();
  const a = document.createElement('a');
  a.download = currentFilename();
  a.href = previewDataUrl.value;
  a.click();
  emit(
    'downloaded',
    `已保存 ${ratio.value === '34' ? '1080×1440 小红书' : '1080×1920 朋友圈'} 高清海报`
  );
}

function handleSystemShare() {
  if (!previewDataUrl.value) rebuildPreview();
  emit('systemShare', previewDataUrl.value, currentFilename());
}
</script>

<template>
  <div v-if="open" class="sheet-backdrop" @click.self="emit('close')">
    <div class="bottom-sheet" role="dialog" aria-modal="true" aria-label="保存与分享高清海报">
      <div class="bs-handle" />
      <div class="bs-head">
        <h3>保存 / 系统分享</h3>
        <div class="ratio-seg">
          <button :class="{ active: ratio === '916' }" @click="ratio = '916'">9:16 朋友圈</button>
          <button :class="{ active: ratio === '34' }" @click="ratio = '34'">3:4 小红书</button>
        </div>
      </div>

      <div class="bs-stage">
        <div class="img-frame" :class="{ r34: ratio === '34' }">
          <img
            v-if="previewDataUrl"
            :src="previewDataUrl"
            :alt="`${test.title} - ${profile.title} 高清海报`"
          />
        </div>
        <p class="bs-tip">微信内长按图片可保存转发 · 手机浏览器可一键拉起系统分享</p>
      </div>

      <div class="bs-actions-wrap">
        <!-- 主操作：拉起系统原生分享（带高清海报图片 + 合盘链接） -->
        <button class="btn-pop sys-share-btn" @click="handleSystemShare">
          <svg viewBox="0 0 24 24" class="share-svg" aria-hidden="true">
            <path d="M12 3v12" />
            <path d="M7.5 7.5 12 3l4.5 4.5" />
            <path d="M5 13v5a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-5" />
          </svg>
          <span>系统分享海报 & 链接</span>
        </button>

        <!-- 次操作：下载 PNG / 复制合盘链接 -->
        <div class="bs-sub-actions">
          <button class="btn-pop-light sub-btn" @click="downloadPng">
            <svg viewBox="0 0 24 24" class="share-svg" aria-hidden="true">
              <path d="M12 3v12" />
              <path d="m7.5 11.5 4.5 4.5 4.5-4.5" />
              <path d="M5 19h14" />
            </svg>
            <span>下载 PNG</span>
          </button>
          <button class="btn-pop-light sub-btn" @click="emit('copyLink')">
            <svg viewBox="0 0 24 24" class="share-svg" aria-hidden="true">
              <path d="M10 13a5 5 0 0 0 7.5.5l2-2a5 5 0 0 0-7.1-7.1l-1.1 1.1" />
              <path d="M14 11a5 5 0 0 0-7.5-.5l-2 2a5 5 0 0 0 7.1 7.1l1.1-1.1" />
            </svg>
            <span>复制合盘链接</span>
          </button>
        </div>
      </div>

      <button class="bs-cancel" @click="emit('close')">取消</button>
    </div>
  </div>
</template>

<style scoped>
.sheet-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(24, 24, 27, 0.54);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: flex-end;
  justify-content: center;
  z-index: 40;
}
.bottom-sheet {
  width: 100%;
  max-width: 430px;
  max-height: 92dvh;
  background: var(--bg);
  border-radius: 24px 24px 0 0;
  border-top: 2.2px solid var(--border);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding-bottom: env(safe-area-inset-bottom, 0px);
}
@media (min-width: 560px) {
  .sheet-backdrop {
    position: absolute;
  }
  .bottom-sheet {
    max-width: none;
    max-height: 92%;
  }
}
.bs-handle {
  width: 40px;
  height: 4.5px;
  border-radius: 99px;
  background: var(--ink);
  opacity: 0.28;
  margin: 10px auto 4px;
  flex-shrink: 0;
}
.bs-head {
  padding: 6px 16px 10px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.bs-head h3 {
  font-size: 16px;
  font-weight: 900;
}
.ratio-seg {
  display: inline-flex;
  background: var(--surface);
  padding: 2px;
  border-radius: 99px;
  border: 1.5px solid var(--border);
}
.ratio-seg button {
  padding: 4px 10px;
  border-radius: 99px;
  font-size: 11px;
  color: var(--muted);
  font-weight: 750;
}
.ratio-seg button.active {
  background: #18181b;
  color: var(--pop-lime);
}
.bs-stage {
  padding: 4px 20px 10px;
  display: flex;
  flex-direction: column;
  align-items: center;
  overflow-y: auto;
}
.img-frame {
  width: 204px;
  border-radius: 16px;
  overflow: hidden;
  border: 2px solid #18181b;
  box-shadow: 5px 5px 0 #18181b;
  background: #fffdf8;
}
.img-frame.r34 {
  width: 230px;
}
.img-frame img {
  display: block;
  width: 100%;
  height: auto;
}
.bs-tip {
  font-size: 11px;
  color: var(--muted);
  margin-top: 9px;
  text-align: center;
  font-weight: 650;
}
.bs-actions-wrap {
  padding: 12px 16px 10px;
  background: var(--surface);
  border-top: 1.5px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: 9px;
}
.sys-share-btn {
  width: 100%;
  height: 46px;
  font-size: 14.5px;
}
.bs-sub-actions {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 9px;
}
.sub-btn {
  height: 40px;
  font-size: 13px;
  box-shadow: var(--shadow-sm);
}
.share-svg {
  width: 17px;
  height: 17px;
  fill: none;
  stroke: currentColor;
  stroke-width: 2.2;
  stroke-linecap: round;
  stroke-linejoin: round;
  flex-shrink: 0;
}
.bs-cancel {
  border-top: 1px solid rgba(24, 24, 27, 0.12);
  height: 44px;
  font-size: 13.5px;
  color: var(--muted);
  font-weight: 750;
  background: var(--surface);
}
</style>
