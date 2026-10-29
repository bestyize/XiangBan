<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import Icon from '@/components/Icon.vue'
import { createCustomWallpaper, uploadFile } from '@/admin/api'
import { toast } from '@/composables/toast'
import { useCatalog } from '@/stores/catalog'

const router = useRouter()
const catalog = useCatalog()

const mediaType = ref<'image' | 'video'>('image')
const title = ref('')
const description = ref('')
const category = ref('')
const tagsText = ref('')
const status = ref('published')

// 资源链接
const previewUrl = ref('')
const regularUrl = ref('')
const videoUrl = ref('')
const width = ref<number | undefined>(undefined)
const height = ref<number | undefined>(undefined)
const orientation = ref<'landscape' | 'portrait' | 'squarish'>('landscape')

// 上传中状态
const uploadingImage = ref(false)
const uploadingVideo = ref(false)
const submitting = ref(false)

async function handleImageFile(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  uploadingImage.value = true
  try {
    const res = await uploadFile(file)
    regularUrl.value = res.url
    previewUrl.value = res.url
    if (!title.value) {
      title.value = file.name.replace(/\.[^/.]+$/, '')
    }

    // 探测图片真实尺寸与方向
    const img = new Image()
    img.src = res.url
    img.onload = () => {
      width.value = img.naturalWidth
      height.value = img.naturalHeight
      const ratio = img.naturalWidth / img.naturalHeight
      if (ratio > 1.2) orientation.value = 'landscape'
      else if (ratio < 0.85) orientation.value = 'portrait'
      else orientation.value = 'squarish'
    }

    toast('图片上传成功！')
  } catch (err: any) {
    toast(err?.message || '图片上传失败')
  } finally {
    uploadingImage.value = false
  }
}

async function handleVideoFile(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  uploadingVideo.value = true
  try {
    const res = await uploadFile(file)
    videoUrl.value = res.url
    regularUrl.value = res.url
    if (!title.value) {
      title.value = file.name.replace(/\.[^/.]+$/, '')
    }

    // 探测视频尺寸与方向
    const vid = document.createElement('video')
    vid.src = res.url
    vid.onloadedmetadata = () => {
      width.value = vid.videoWidth
      height.value = vid.videoHeight
      const ratio = vid.videoWidth / vid.videoHeight
      if (ratio > 1.2) orientation.value = 'landscape'
      else if (ratio < 0.85) orientation.value = 'portrait'
      else orientation.value = 'squarish'
    }

    toast('视频上传成功！建议同时上传一张视频海报/封面图')
  } catch (err: any) {
    toast(err?.message || '视频上传失败')
  } finally {
    uploadingVideo.value = false
  }
}

async function handlePosterFile(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  try {
    const res = await uploadFile(file)
    previewUrl.value = res.url
    toast('视频封面图上传成功！')
  } catch (err: any) {
    toast(err?.message || '封面图上传失败')
  }
}

async function submit() {
  if (!title.value.trim()) {
    toast('请输入壁纸标题')
    return
  }
  if (!regularUrl.value) {
    toast(mediaType.value === 'image' ? '请先上传壁纸图片' : '请先上传视频文件')
    return
  }
  if (mediaType.value === 'video' && !previewUrl.value) {
    previewUrl.value = regularUrl.value
  }

  submitting.value = true
  const tags = tagsText.value
    .split(/[,，]/)
    .map((s) => s.trim())
    .filter(Boolean)

  try {
    await createCustomWallpaper({
      title: title.value.trim(),
      description: description.value.trim() || undefined,
      media_type: mediaType.value,
      preview_url: previewUrl.value,
      regular_url: regularUrl.value,
      video_url: mediaType.value === 'video' ? videoUrl.value : undefined,
      category: category.value || undefined,
      tags,
      orientation: orientation.value,
      width: width.value,
      height: height.value,
      status: status.value,
    })

    toast('壁纸创建并上架成功！')
    router.push('/admin/wallpapers')
  } catch (err: any) {
    toast(err?.message || '上架失败')
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <div class="upload-view">
    <header class="head">
      <div class="title-wrap">
        <h1 class="a-h1">自定义壁纸 / 视频上架</h1>
        <p class="desc">直接上传本地原创或高质量壁纸、4K 视频，填入分类与标签并发布到前台。</p>
      </div>
    </header>

    <div class="form-container">
      <!-- 媒体类型切换 -->
      <div class="type-selector">
        <label
          class="type-card"
          :class="{ active: mediaType === 'image' }"
          @click="mediaType = 'image'"
        >
          <Icon name="image" :size="24" />
          <div class="type-text">
            <span class="type-title">静态图片壁纸</span>
            <span class="type-sub">支持 4K/8K JPG, PNG, WEBP</span>
          </div>
        </label>

        <label
          class="type-card"
          :class="{ active: mediaType === 'video' }"
          @click="mediaType = 'video'"
        >
          <Icon name="play" :size="24" />
          <div class="type-text">
            <span class="type-title">动态视频壁纸</span>
            <span class="type-sub">支持 MP4, WebM 高清与 4K 循环动态壁纸</span>
          </div>
        </label>
      </div>

      <!-- 文件上传卡片 -->
      <div class="upload-sections">
        <!-- 静态壁纸上传 -->
        <div v-if="mediaType === 'image'" class="drop-zone">
          <input
            type="file"
            accept="image/*"
            class="file-input"
            @change="handleImageFile"
          />
          <div v-if="regularUrl" class="preview-wrap">
            <img :src="regularUrl" alt="预览图" class="uploaded-img" />
            <span class="re-upload-tip">点击或拖拽可更换图片</span>
          </div>
          <div v-else class="upload-placeholder">
            <Icon name="upload" :size="36" />
            <span class="main-tip">{{ uploadingImage ? '正在上传图片...' : '点击或拖拽上传壁纸图片' }}</span>
            <span class="sub-tip">推荐 3840x2160 (4K) 或更高分辨率</span>
          </div>
        </div>

        <!-- 动态壁纸上传 -->
        <div v-else class="video-upload-grid">
          <!-- 视频主体 -->
          <div class="drop-zone">
            <input
              type="file"
              accept="video/*"
              class="file-input"
              @change="handleVideoFile"
            />
            <div v-if="videoUrl" class="preview-wrap">
              <video :src="videoUrl" controls autoplay muted loop class="uploaded-video" />
              <span class="re-upload-tip">点击可更换视频</span>
            </div>
            <div v-else class="upload-placeholder">
              <Icon name="play" :size="36" />
              <span class="main-tip">{{ uploadingVideo ? '正在上传视频...' : '点击上传动态壁纸视频 (MP4/WebM)' }}</span>
              <span class="sub-tip">建议无黑边、支持无缝循环的短视频</span>
            </div>
          </div>

          <!-- 视频封面图（可选） -->
          <div class="drop-zone mini">
            <input
              type="file"
              accept="image/*"
              class="file-input"
              @change="handlePosterFile"
            />
            <div v-if="previewUrl && previewUrl !== videoUrl" class="preview-wrap">
              <img :src="previewUrl" alt="封面图" class="uploaded-img" />
              <span class="re-upload-tip">点击更换封面</span>
            </div>
            <div v-else class="upload-placeholder">
              <Icon name="image" :size="24" />
              <span class="main-tip">上传视频海报图 (可选)</span>
              <span class="sub-tip">若不上传将使用视频首帧</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 元信息表单 -->
      <div class="meta-form">
        <div class="form-row">
          <div class="form-item flex-2">
            <label class="label">壁纸标题 *</label>
            <input
              v-model="title"
              type="text"
              placeholder="例如：赛博朋克雨夜 4K"
              class="a-input"
            />
          </div>

          <div class="form-item flex-1">
            <label class="label">所属分类</label>
            <select v-model="category" class="a-select">
              <option value="">未分类</option>
              <option v-for="cat in catalog.categories" :key="cat.id" :value="cat.id">
                {{ cat.name }}
              </option>
            </select>
          </div>
        </div>

        <div class="form-row">
          <div class="form-item flex-2">
            <label class="label">标签 (逗号分隔)</label>
            <input
              v-model="tagsText"
              type="text"
              placeholder="例如：城市, 雨夜, 霓虹, 4K"
              class="a-input"
            />
          </div>

          <div class="form-item flex-1">
            <label class="label">屏幕方向</label>
            <select v-model="orientation" class="a-select">
              <option value="landscape">横屏 (PC 电脑)</option>
              <option value="portrait">竖屏 (手机 / 平板)</option>
              <option value="squarish">方形</option>
            </select>
          </div>
        </div>

        <div class="form-row">
          <div class="form-item flex-1">
            <label class="label">分辨率（自动探测）</label>
            <div class="res-display">
              {{ width && height ? `${width} × ${height}` : '上传文件后自动识别' }}
            </div>
          </div>

          <div class="form-item flex-1">
            <label class="label">上架状态</label>
            <select v-model="status" class="a-select">
              <option value="published">立即对外上架 (Published)</option>
              <option value="pending">仅存入待审核库 (Pending)</option>
            </select>
          </div>
        </div>

        <div class="form-item">
          <label class="label">描述信息 (可选)</label>
          <textarea
            v-model="description"
            rows="3"
            placeholder="填写简要壁纸说明或作者来源版权信息..."
            class="a-input"
          />
        </div>

        <div class="form-actions">
          <button
            type="button"
            class="a-btn primary submit-btn"
            :disabled="submitting || uploadingImage || uploadingVideo"
            @click="submit"
          >
            {{ submitting ? '正在上架...' : '确认上架发布' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.upload-view {
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding: 32px 36px 60px;
  max-width: 1200px;
}

.head {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.desc {
  font-size: 14px;
  color: var(--muted);
}

.form-container {
  display: flex;
  flex-direction: column;
  gap: 28px;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid var(--line);
  border-radius: var(--r-card);
  padding: 28px;
}

.type-selector {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.type-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px 20px;
  border-radius: var(--r-card);
  border: 1px solid var(--line);
  background: rgba(0, 0, 0, 0.25);
  cursor: pointer;
  transition: all 0.2s;
}

.type-card:hover {
  background: rgba(255, 255, 255, 0.04);
}

.type-card.active {
  border-color: rgba(59, 130, 246, 0.6);
  background: rgba(59, 130, 246, 0.08);
  box-shadow: 0 0 20px rgba(59, 130, 246, 0.1);
}

.type-title {
  display: block;
  font-size: 15px;
  font-weight: 600;
  color: var(--ink);
}

.type-sub {
  display: block;
  font-size: 12px;
  color: var(--muted);
  margin-top: 2px;
}

.drop-zone {
  position: relative;
  min-height: 240px;
  border: 2px dashed rgba(255, 255, 255, 0.15);
  border-radius: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.2);
  overflow: hidden;
  cursor: pointer;
  transition: border-color 0.2s;
}

.drop-zone:hover {
  border-color: rgba(255, 255, 255, 0.35);
}

.drop-zone.mini {
  min-height: 140px;
}

.file-input {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  opacity: 0;
  cursor: pointer;
  z-index: 10;
}

.upload-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  color: var(--muted);
  pointer-events: none;
}

.main-tip {
  font-size: 15px;
  font-weight: 500;
  color: var(--ink);
}

.sub-tip {
  font-size: 12px;
  color: var(--muted);
}

.preview-wrap {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}

.uploaded-img {
  max-height: 280px;
  max-width: 100%;
  object-fit: contain;
  border-radius: 8px;
}

.uploaded-video {
  max-height: 280px;
  max-width: 100%;
  border-radius: 8px;
}

.re-upload-tip {
  margin-top: 8px;
  font-size: 12px;
  color: var(--muted);
}

.video-upload-grid {
  display: grid;
  grid-template-columns: 2fr 1fr;
  gap: 16px;
}

.meta-form {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.form-row {
  display: flex;
  gap: 16px;
}

.flex-1 {
  flex: 1;
}

.flex-2 {
  flex: 2;
}

.form-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.label {
  font-size: 13px;
  font-weight: 500;
  color: var(--ink);
}

.res-display {
  display: flex;
  align-items: center;
  height: 38px;
  padding: 0 12px;
  background: rgba(0, 0, 0, 0.25);
  border: 1px solid var(--line);
  border-radius: var(--r-input);
  font-size: 13px;
  color: var(--muted);
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 12px;
}

.submit-btn {
  min-width: 180px;
  padding: 12px 28px;
  font-size: 14px;
}
</style>
