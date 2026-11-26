<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import Icon from '@/components/Icon.vue'
import AdminSelect, { type SelectOption } from '@/components/admin/AdminSelect.vue'
import TagChipEditor from '@/components/admin/TagChipEditor.vue'
import { createCustomWallpaper, uploadFile } from '@/admin/api'
import { toast } from '@/composables/toast'
import { useCatalog } from '@/stores/catalog'

const router = useRouter()
const catalog = useCatalog()

const mediaType = ref<'image' | 'video'>('image')
const title = ref('')
const description = ref('')
const category = ref('')
const tags = ref<string[]>([])
const status = ref<'published' | 'pending'>('published')

const previewUrl = ref('')
const regularUrl = ref('')
const videoUrl = ref('')
const fileName = ref('')
const width = ref<number | undefined>(undefined)
const height = ref<number | undefined>(undefined)
const orientation = ref<'landscape' | 'portrait' | 'squarish'>('landscape')
const previewFrame = ref<'desktop' | 'phone'>('desktop')

const dragging = ref(false)
const uploadingMedia = ref(false)
const uploadingPoster = ref(false)
const submitting = ref(false)

const categoryOptions = computed<SelectOption[]>(() => [
  { value: '', label: '未分类' },
  ...catalog.categories.map((c) => ({ value: c.id, label: c.name })),
])

const orientationOptions: SelectOption[] = [
  { value: 'landscape', label: '横屏 (PC)', icon: 'monitor' },
  { value: 'portrait', label: '竖屏 (手机)', icon: 'phone' },
  { value: 'squarish', label: '方形', icon: 'grid' },
]

onMounted(() => {
  if (catalog.categories.length === 0) void catalog.load()
})

function resetMedia() {
  previewUrl.value = ''
  regularUrl.value = ''
  videoUrl.value = ''
  fileName.value = ''
  width.value = undefined
  height.value = undefined
}

async function processImageFile(file: File) {
  uploadingMedia.value = true
  try {
    const res = await uploadFile(file)
    regularUrl.value = res.url
    previewUrl.value = res.url
    fileName.value = file.name
    if (!title.value) title.value = file.name.replace(/\.[^/.]+$/, '')

    const img = new Image()
    img.src = res.url
    img.onload = () => {
      width.value = img.naturalWidth
      height.value = img.naturalHeight
      const ratio = img.naturalWidth / img.naturalHeight
      orientation.value = ratio > 1.2 ? 'landscape' : ratio < 0.85 ? 'portrait' : 'squarish'
    }
    toast('图片上传成功')
  } catch (err: unknown) {
    toast(err instanceof Error ? err.message : '图片上传失败')
  } finally {
    uploadingMedia.value = false
  }
}

async function processVideoFile(file: File) {
  uploadingMedia.value = true
  try {
    const res = await uploadFile(file)
    videoUrl.value = res.url
    regularUrl.value = res.url
    fileName.value = file.name
    if (!title.value) title.value = file.name.replace(/\.[^/.]+$/, '')

    const vid = document.createElement('video')
    vid.src = res.url
    vid.onloadedmetadata = () => {
      width.value = vid.videoWidth
      height.value = vid.videoHeight
      const ratio = vid.videoWidth / vid.videoHeight
      orientation.value = ratio > 1.2 ? 'landscape' : ratio < 0.85 ? 'portrait' : 'squarish'
    }
    toast('视频上传成功')
  } catch (err: unknown) {
    toast(err instanceof Error ? err.message : '视频上传失败')
  } finally {
    uploadingMedia.value = false
  }
}

function onMediaInput(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return
  if (mediaType.value === 'image') void processImageFile(file)
  else void processVideoFile(file)
}

function onDrop(e: DragEvent) {
  dragging.value = false
  const file = e.dataTransfer?.files?.[0]
  if (!file) return
  if (mediaType.value === 'image') void processImageFile(file)
  else void processVideoFile(file)
}

async function onPosterInput(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return
  uploadingPoster.value = true
  try {
    const res = await uploadFile(file)
    previewUrl.value = res.url
    toast('视频封面已上传')
  } catch (err: unknown) {
    toast(err instanceof Error ? err.message : '封面上传失败')
  } finally {
    uploadingPoster.value = false
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
  const poster = previewUrl.value || regularUrl.value
  submitting.value = true
  try {
    await createCustomWallpaper({
      title: title.value.trim(),
      description: description.value.trim() || undefined,
      media_type: mediaType.value,
      preview_url: poster,
      regular_url: regularUrl.value,
      video_url: mediaType.value === 'video' ? videoUrl.value : undefined,
      category: category.value || undefined,
      tags: tags.value,
      orientation: orientation.value,
      width: width.value,
      height: height.value,
      status: status.value,
    })
    toast('已创建并发布到壁纸图库')
    router.push('/admin/wallpapers')
  } catch (err: unknown) {
    toast(err instanceof Error ? err.message : '上架失败')
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <div class="xb-page is-narrow">
    <header class="xb-page-head">
      <div class="xb-tabs-lg">
        <button
          type="button"
          :class="{ 'is-active': mediaType === 'image' }"
          @click="mediaType = 'image'; resetMedia()"
        >
          静态图片
        </button>
        <button
          type="button"
          :class="{ 'is-active': mediaType === 'video' }"
          @click="mediaType = 'video'; resetMedia()"
        >
          动态视频
        </button>
      </div>
      <div class="xb-page-actions">
        <RouterLink to="/admin/wallpapers" class="xb-btn-ghost">取消</RouterLink>
        <button
          type="button"
          class="xb-btn-primary"
          :disabled="submitting || uploadingMedia || uploadingPoster"
          @click="submit"
        >
          {{ submitting ? '发布中…' : '上架发布' }}
        </button>
      </div>
    </header>

    <div class="xb-split-2">
      <div class="col-stack">
        <div v-if="regularUrl" class="xb-group xb-group-pad preview-card">
          <div class="preview-top">
            <span class="xb-mono fname" :title="fileName">{{ fileName || '已上传素材' }}</span>
            <div class="xb-seg">
              <button type="button" :class="{ 'is-active': previewFrame === 'desktop' }" @click="previewFrame = 'desktop'">
                桌面
              </button>
              <button type="button" :class="{ 'is-active': previewFrame === 'phone' }" @click="previewFrame = 'phone'">
                手机
              </button>
            </div>
          </div>

          <div class="device-frame" :class="{ 'is-phone': previewFrame === 'phone' }">
            <video
              v-if="mediaType === 'video'"
              :src="videoUrl"
              controls
              autoplay
              muted
              loop
              playsinline
            />
            <img v-else :src="regularUrl" alt="预览" />
          </div>

          <div class="preview-foot">
            <span class="xb-num res">{{ width && height ? `${width} × ${height}` : '自动探测中…' }}</span>
            <label class="xb-btn-ghost is-sm file-trigger">
              <span>更换素材</span>
              <input
                type="file"
                :accept="mediaType === 'image' ? 'image/*' : 'video/*'"
                hidden
                @change="onMediaInput"
              />
            </label>
          </div>
        </div>

        <label
          v-else
          class="drop-zone"
          :class="{ 'is-drag': dragging }"
          @dragover.prevent="dragging = true"
          @dragleave="dragging = false"
          @drop.prevent="onDrop"
        >
          <input
            type="file"
            :accept="mediaType === 'image' ? 'image/*' : 'video/*'"
            hidden
            @change="onMediaInput"
          />
          <span class="drop-ico"><Icon :name="mediaType === 'video' ? 'play' : 'upload'" :size="26" /></span>
          <strong>
            {{ uploadingMedia ? '上传中…' : `点击或拖拽上传${mediaType === 'video' ? '动态视频' : '高清图片'}` }}
          </strong>
          <span class="drop-sub">
            {{ mediaType === 'video' ? 'MP4 / WebM，建议无缝循环' : 'JPG / PNG / WebP，推荐 4K 以上' }}
          </span>
        </label>

        <div v-if="mediaType === 'video'" class="xb-group">
          <div class="xb-row" title="不上传时由后端 FFmpeg 自动截取第 1 帧作为封面">
            <div class="poster-info">
              <span style="font-weight: 550">视频封面海报</span>
              <span v-if="previewUrl && previewUrl !== videoUrl" class="xb-pill is-ok">已上传</span>
            </div>
            <label class="xb-btn-soft is-sm file-trigger">
              <span>{{ uploadingPoster ? '上传中…' : previewUrl && previewUrl !== videoUrl ? '更换封面' : '上传封面' }}</span>
              <input type="file" accept="image/*" hidden @change="onPosterInput" />
            </label>
          </div>
        </div>
      </div>

      <div class="xb-group xb-group-pad form-col">
        <label class="xb-field">
          <span class="xb-field-label">标题</span>
          <input v-model="title" type="text" class="xb-input" placeholder="例如：冰岛极光湖面 4K" />
        </label>

        <div class="xb-split-eq">
          <div class="xb-field">
            <span class="xb-field-label">分类</span>
            <AdminSelect v-model="category" :options="categoryOptions" full />
          </div>
          <div class="xb-field">
            <span class="xb-field-label">画幅</span>
            <AdminSelect
              :model-value="orientation"
              :options="orientationOptions"
              full
              @update:model-value="orientation = $event as 'landscape' | 'portrait' | 'squarish'"
            />
          </div>
        </div>

        <div class="xb-field">
          <span class="xb-field-label">标签</span>
          <TagChipEditor v-model="tags" />
        </div>

        <label class="xb-field">
          <span class="xb-field-label">备注</span>
          <textarea v-model="description" class="xb-textarea" rows="3" placeholder="可选：作者署名或拍摄参数" />
        </label>

        <div class="xb-field">
          <span class="xb-field-label">上架状态</span>
          <div class="xb-seg" style="align-self: flex-start">
            <button type="button" :class="{ 'is-active': status === 'published' }" @click="status = 'published'">
              立即上架
            </button>
            <button type="button" :class="{ 'is-active': status === 'pending' }" @click="status = 'pending'">
              存入待审核
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.col-stack {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.form-col {
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.drop-zone {
  border-radius: 12px;
  background: var(--ui-surface-sunken);
  min-height: 240px;
  padding: 28px 24px;
  border: 1.5px dashed var(--ui-border-strong);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  text-align: center;
  cursor: pointer;
  transition: background 0.14s, border-color 0.14s;
}
.drop-zone:hover,
.drop-zone.is-drag {
  background: var(--ui-soft);
  border-color: var(--ui-btn);
}
.drop-ico { color: var(--ui-ink); }
.drop-sub { font-size: 12px; color: var(--ui-text-tertiary); }

.preview-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.preview-top,
.preview-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}
.fname {
  font-size: 12px;
  color: var(--ui-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.res { font-size: 12px; color: var(--ui-text-tertiary); }
.device-frame {
  width: 100%;
  aspect-ratio: 16 / 9;
  border-radius: 10px;
  overflow: hidden;
  background: #111;
}
.device-frame.is-phone {
  width: 164px;
  aspect-ratio: 9 / 18;
  margin: 0 auto;
  border-radius: 22px;
  border: 5px solid #1C1C1F;
}
.device-frame img,
.device-frame video {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.poster-info {
  display: flex;
  align-items: center;
  gap: 8px;
}
.file-trigger { cursor: pointer; }
</style>
