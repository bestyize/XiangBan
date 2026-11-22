<script setup lang="ts">
// 更换头像：宽屏用站内对话框，左侧裁剪、右侧预览；窄屏与手机 App 一致，底部选图弹层 + 全屏裁剪页。
// 设计稿：client/xiaoban-multiplatform/docs/design-specs/prototypes/avatar-editor.html
import { computed, nextTick, onBeforeUnmount, onMounted, ref, shallowRef, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ModalDialog from './ModalDialog.vue'
import UserAvatar from './UserAvatar.vue'
import { uploadImage } from '@/api/forum'
import { useSession } from '@/stores/session'
import { useToast } from '@/stores/toast'
import { useNarrow } from '@/lib/viewport'
import { AvatarCropper, checkAvatarFile, exportAvatar, initialCrop, isSmallCrop, loadAvatarSource, paintCrop, type CropState } from '@/lib/avatarCrop'

const props = defineProps<{ open: boolean; name: string; current: string | null }>()
const emit = defineEmits<{ 'update:open': [open: boolean]; pending: [pending: boolean] }>()

const session = useSession()
const toast = useToast()
const route = useRoute()
const router = useRouter()
const narrow = useNarrow()

type Status = 'idle' | 'cropping' | 'saving' | 'failed'
const status = ref<Status>('idle')
const view = ref<'pick' | 'edit'>('pick')
const src = shallowRef<HTMLCanvasElement | null>(null)
const crop = shallowRef<CropState | null>(null)
const error = ref('')
const pending = ref(false)
const zoom = ref(1)
const small = ref(false)
const dragOver = ref(false)
const canvasEl = ref<HTMLCanvasElement | null>(null)

const saving = computed(() => status.value === 'saving')
const failed = computed(() => status.value === 'failed')
const sheetVisible = computed(() => narrow.value && props.open && view.value === 'pick')
const cropPageVisible = computed(() => narrow.value && props.open && !!src.value)
const zoomTrack = computed(() => `${((zoom.value - 1) / 3) * 100}%`)
const failReason = computed(() => error.value.replace(/[。.]$/, ''))

/** 窄屏画面撑满宽度（最多 480），圆占 300/390。 */
const stageSize = () => (narrow.value ? Math.min(window.innerWidth, 480) : 360)
const diameterOf = (size: number) => (narrow.value ? Math.round((size * 300) / 390) : 300)

let cropper: AvatarCropper | null = null
let cropDiameter = 300
let srcId = 0
let uploaded: { signature: string; url: string } | null = null
let frame = 0

const SIZE_LABEL: Record<number, string> = { 96: '主页', 48: '卡片', 32: '评论', 24: '列表' }

const zoomBy = (factor: number) => cropper?.zoomTo(zoom.value * factor)
const setZoom = (event: Event) => cropper?.zoomTo(Number((event.target as HTMLInputElement).value))
const rotate = () => cropper?.rotate()
const resetCrop = () => cropper?.reset()

watch(pending, value => emit('pending', value))

watch(canvasEl, el => {
  cropper?.destroy()
  cropper = null
  if (!el) return
  const size = stageSize()
  cropper = new AvatarCropper(el, { size, diameter: diameterOf(size), onChange: onCropChange })
  const k = cropper.diameter / cropDiameter
  cropper.restore(src.value, crop.value ? { ...crop.value, tx: crop.value.tx * k, ty: crop.value.ty * k } : null)
  cropper.lock(saving.value)
})

function onCropChange() {
  if (!cropper) return
  if (cropper.src !== src.value) {
    src.value = cropper.src as HTMLCanvasElement | null
    srcId++
  }
  crop.value = cropper.state
  cropDiameter = cropper.diameter
  zoom.value = cropper.state?.zoom ?? 1
  small.value = !!cropper.state && isSmallCrop(cropper.state)
  cancelAnimationFrame(frame)
  frame = requestAnimationFrame(paintPreviews)
}

function paintPreviews() {
  const dpr = Math.min(window.devicePixelRatio || 1, 3)
  document.querySelectorAll<HTMLCanvasElement>('canvas[data-avatar-preview]').forEach(cv => {
    const size = Number(cv.dataset.avatarPreview)
    const px = Math.round(size * dpr)
    if (cv.width !== px) cv.width = cv.height = px
    cv.style.width = cv.style.height = `${size}px`
    const ctx = cv.getContext('2d')
    if (ctx) paintCrop(ctx, px, src.value, crop.value, cropDiameter)
  })
}

/** 同一张图、同一个位置已经上传过时，重试只重新写账号中心。 */
function signature() {
  const s = crop.value!
  return [srcId, s.iw, s.ih, s.zoom.toFixed(4), (s.tx / cropDiameter).toFixed(4), (s.ty / cropDiameter).toFixed(4)].join('|')
}

watch(() => props.open, async open => {
  if (!open) return
  pending.value = false
  view.value = src.value ? 'edit' : 'pick'
  await nextTick()
  paintPreviews()
})

async function useFile(file?: File | null) {
  if (!file || saving.value) return
  const reason = checkAvatarFile(file)
  if (reason) return toast.show(reason, 'error')
  try {
    const canvas = await loadAvatarSource(file)
    src.value = canvas
    srcId++
    crop.value = initialCrop(canvas.width, canvas.height)
    cropDiameter = cropper?.diameter ?? diameterOf(stageSize())
    uploaded = null
    error.value = ''
    pending.value = false
    if (status.value !== 'saving') status.value = 'cropping'
    view.value = 'edit'
    cropper?.setImage(canvas)
    await nextTick()
    paintPreviews()
  } catch (e) {
    toast.error(e)
  }
}

function onFileInput(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  void useFile(file)
}

function onDrop(event: DragEvent) {
  dragOver.value = false
  if (saving.value) return
  void useFile(event.dataTransfer?.files?.[0])
}

function clear() {
  src.value = null
  crop.value = null
  status.value = 'idle'
  error.value = ''
  pending.value = false
  uploaded = null
  view.value = 'pick'
  cropper?.setImage(null)
}

function close() {
  if (saving.value) return
  if (failed.value && src.value) pending.value = true
  else clear()
  emit('update:open', false)
}

/** 窄屏选图弹层的取消：从裁剪页来的回到裁剪页，否则整个关掉。 */
function cancelSheet() {
  if (saving.value) return
  if (src.value) view.value = 'edit'
  else close()
}

async function save() {
  if (saving.value || !src.value || !crop.value) return
  status.value = 'saving'
  error.value = ''
  cropper?.lock(true)
  try {
    const sig = signature()
    let url = uploaded?.signature === sig ? uploaded.url : ''
    if (!url) {
      const image = await uploadImage(await exportAvatar(src.value, crop.value, cropDiameter))
      url = new URL(image.url, window.location.origin).href
      uploaded = { signature: sig, url }
    }
    await session.setAvatar(url)
    clear()
    emit('update:open', false)
    toast.show('头像已更新')
  } catch (e) {
    status.value = 'failed'
    error.value = e instanceof Error && e.message ? e.message : '保存失败，请重试'
  } finally {
    cropper?.lock(false)
  }
}

// 窄屏裁剪页写一条浏览器历史：系统返回键和左上角 ‹ 都只关闭裁剪页。
let pushedHistory = false
watch(cropPageVisible, async show => {
  document.documentElement.classList.toggle('avatar-crop-open', show)
  if (show && !pushedHistory) {
    pushedHistory = true
    if (route.query.avatar !== 'crop') await router.push({ query: { ...route.query, avatar: 'crop' } })
  } else if (!show && pushedHistory) {
    pushedHistory = false
    if (route.query.avatar === 'crop') router.back()
  }
})
watch(() => route.query.avatar, (value, old) => {
  if (old !== 'crop' || value === 'crop' || !pushedHistory) return
  pushedHistory = false
  if (saving.value) {
    pushedHistory = true
    void router.push({ query: { ...route.query, avatar: 'crop' } })
    return
  }
  close()
})

onMounted(() => {
  if (route.query.avatar === 'crop') {
    const query = { ...route.query }
    delete query.avatar
    void router.replace({ query })
  }
})
onBeforeUnmount(() => {
  cropper?.destroy()
  cancelAnimationFrame(frame)
  document.documentElement.classList.remove('avatar-crop-open')
})

defineExpose({ discard: clear })
</script>

<template>
  <!-- 宽屏：居中弹窗 -->
  <ModalDialog v-if="!narrow" :open="open" title="更换头像" :width="820" scrollable :close-disabled="saving" @close="close">
    <div v-if="view === 'pick' || !src" class="pick">
      <label class="drop" :class="{ over: dragOver }" @dragover.prevent="dragOver = !saving" @dragleave="dragOver = false" @drop.prevent="onDrop">
        <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="3" y="4" width="18" height="16" rx="3" /><circle cx="9" cy="10" r="2" /><path d="M21 16l-5-5-9 9" /></svg>
        <b>把图片拖到这里</b>
        <span>或 <span class="link">选择本机图片</span></span>
        <small>JPG、PNG、WebP，10 MB 以内</small>
        <input class="file" type="file" accept="image/*" @change="onFileInput" />
      </label>
    </div>

    <div v-else class="edit" :class="{ busy: saving }">
      <div class="left">
        <div class="stage" :class="{ over: dragOver }" @dragover.prevent="dragOver = !saving" @dragleave="dragOver = false" @drop.prevent="onDrop">
          <canvas ref="canvasEl" tabindex="0" aria-label="裁剪区域：拖动调整位置，滚轮缩放，方向键微调" />
        </div>
        <div class="tools">
          <button type="button" class="ib" aria-label="缩小" :disabled="saving" @click="zoomBy(1 / 1.2)"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"><path d="M6 12h12" /></svg></button>
          <input class="range" type="range" min="1" max="4" step="0.01" :value="zoom" :style="{ '--pct': zoomTrack }" aria-label="缩放" :disabled="saving" @input="setZoom" />
          <button type="button" class="ib" aria-label="放大" :disabled="saving" @click="zoomBy(1.2)"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"><path d="M6 12h12M12 6v12" /></svg></button>
          <button type="button" class="ib" aria-label="旋转 90 度" title="旋转 90°" :disabled="saving" @click="rotate"><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M20 11a8 8 0 1 0-2.3 5.7" /><path d="M20 4v7h-7" /></svg></button>
          <button type="button" class="ib" aria-label="还原位置" title="还原" :disabled="saving" @click="resetCrop"><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M4 12a8 8 0 1 0 2.3-5.7" /><path d="M4 4v4h4" /></svg></button>
          <button type="button" class="btn btn-sm" :disabled="saving" @click="view = 'pick'">换一张</button>
        </div>
        <p class="warn">{{ small ? '图片较小，再放大可能会模糊。' : '' }}</p>
      </div>
      <div class="right">
        <div>
          <p class="cap">当前 → 新</p>
          <div class="cmp">
            <figure><UserAvatar :name="name" :src="current" :size="56" /><figcaption>{{ failed ? '当前 · 未改变' : '当前' }}</figcaption></figure>
            <svg class="arrow" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M5 12h14M13 6l6 6-6 6" /></svg>
            <figure class="new"><canvas class="pv" data-avatar-preview="56" /><figcaption>新头像</figcaption></figure>
          </div>
        </div>
        <div>
          <p class="cap">实际尺寸</p>
          <div class="sizes">
            <figure v-for="size in [96, 48, 32, 24]" :key="size"><canvas class="pv" :data-avatar-preview="size" /><figcaption>{{ SIZE_LABEL[size] }} {{ size }}</figcaption></figure>
          </div>
        </div>
        <div>
          <p class="cap">在帖子里</p>
          <div class="topic">
            <div class="t"><div class="ttl">你发的帖子在列表里这样显示</div><div class="by"><canvas class="pv" data-avatar-preview="20" /><b>{{ name }}</b>· 刚刚</div></div>
            <div class="t"><div class="cm"><canvas class="pv" data-avatar-preview="32" /><div><b>{{ name }}</b><div>评论里的头像是 32 像素。</div></div></div></div>
          </div>
        </div>
      </div>
    </div>

    <p v-if="failed && view === 'edit'" class="err" role="alert">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" aria-hidden="true"><circle cx="12" cy="12" r="9" /><path d="M12 7.5v5.5M12 16.5v.01" /></svg>
      保存失败：{{ failReason }}。当前头像没有改变，裁剪已保留，可以直接重试。
    </p>
    <footer class="foot">
      <span class="hint" :style="{ visibility: view === 'edit' && src ? 'visible' : 'hidden' }">拖动调整位置 · 滚轮或滑块缩放 · <kbd>←</kbd><kbd>→</kbd> 微调 · <kbd>+</kbd><kbd>−</kbd> 缩放</span>
      <button v-if="view === 'pick' && src" type="button" class="btn" @click="view = 'edit'">返回裁剪</button>
      <button type="button" class="btn" :disabled="saving" @click="close">取消</button>
      <button v-if="view === 'edit' && src" type="button" class="btn btn-primary" :disabled="saving" @click="save">
        <span v-if="saving" class="spin" aria-hidden="true" />{{ saving ? '保存中…' : failed ? '重试' : '保存头像' }}
      </button>
    </footer>
  </ModalDialog>

  <!-- 窄屏：与手机 App 一致 -->
  <Teleport v-else to="body">
    <section v-if="cropPageVisible" class="m-page" :class="{ busy: saving }" role="dialog" aria-modal="true" aria-label="裁剪头像">
      <header class="m-top">
        <button type="button" class="m-icon" aria-label="返回" :disabled="saving" @click="close"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M15 5l-7 7 7 7" /></svg></button>
        <h2>裁剪头像</h2><span />
      </header>
      <div class="m-body">
        <p v-if="failed" class="m-banner" role="alert">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" aria-hidden="true"><circle cx="12" cy="12" r="9" /><path d="M12 7.5v5.5M12 16.5v.01" /></svg>
          保存失败，{{ failReason }}。头像没有改动。
        </p>
        <div class="m-stage"><canvas ref="canvasEl" aria-label="裁剪区域：单指拖动，双指缩放" /></div>
        <div class="m-tools">
          <button type="button" class="m-icon" aria-label="旋转 90 度" :disabled="saving" @click="rotate"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M20 11a8 8 0 1 0-2.3 5.7" /><path d="M20 4v7h-7" /></svg></button>
          <svg class="m-zoom-ico" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" aria-hidden="true"><circle cx="11" cy="11" r="7" /><path d="M8 11h6M20 20l-4-4" /></svg>
          <input class="range" type="range" min="1" max="4" step="0.01" :value="zoom" :style="{ '--pct': zoomTrack }" aria-label="缩放" :disabled="saving" @input="setZoom" />
          <svg class="m-zoom-ico" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" aria-hidden="true"><circle cx="11" cy="11" r="7" /><path d="M8 11h6M11 8v6M20 20l-4-4" /></svg>
          <button type="button" class="m-icon" aria-label="还原位置" :disabled="saving" @click="resetCrop"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M4 12a8 8 0 1 0 2.3-5.7" /><path d="M4 4v4h4" /></svg></button>
        </div>
        <p class="m-hint" :class="{ warn: small }">{{ small ? '图片较小，再放大可能会模糊' : '单指拖动 · 双指缩放 · 双击放大' }}</p>
        <div class="m-preview">
          <div class="m-cmp">
            <figure><UserAvatar :name="name" :src="current" :size="48" /><figcaption>{{ failed ? '当前 · 未改变' : '当前' }}</figcaption></figure>
            <svg class="arrow" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M5 12h14M13 6l6 6-6 6" /></svg>
            <figure class="new"><canvas class="pv" data-avatar-preview="56" /><figcaption>新头像</figcaption></figure>
          </div>
          <div class="m-post">
            <div class="who"><canvas class="pv" data-avatar-preview="24" /><b>{{ name }}</b>· 刚刚</div>
            <p>在帖子和评论里，你的头像这样显示。</p>
          </div>
        </div>
      </div>
      <div class="m-actions">
        <button type="button" class="pill sec" :disabled="saving" @click="view = 'pick'">重新选择</button>
        <button type="button" class="pill pri" :disabled="saving" @click="save"><span v-if="saving" class="spin" aria-hidden="true" />{{ saving ? '保存中' : failed ? '重试' : '保存' }}</button>
      </div>
    </section>

    <div v-if="sheetVisible" class="m-sheet-bg" @click.self="cancelSheet">
      <div class="m-sheet" role="dialog" aria-modal="true" aria-label="更换头像">
        <div class="grab" />
        <h3>更换头像</h3>
        <label class="m-row">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="3" y="4" width="18" height="16" rx="3" /><circle cx="9" cy="10" r="2" /><path d="M21 16l-5-5-9 9" /></svg>
          从相册选择
          <input class="file" type="file" accept="image/*" @change="onFileInput" />
        </label>
        <button type="button" class="m-cancel" @click="cancelSheet">取消</button>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.file { position: absolute; width: 1px; height: 1px; opacity: 0; pointer-events: none; }
.pv { display: block; border-radius: 50%; background: var(--surface-2); border: 1px solid var(--line); }
.range { flex: 1; min-width: 0; height: 24px; margin: 0 4px; appearance: none; -webkit-appearance: none; background: transparent; }
.range::-webkit-slider-runnable-track { height: 4px; border-radius: 2px; background: linear-gradient(90deg, var(--brand-fill) var(--pct), var(--line) var(--pct)); }
.range::-moz-range-track { height: 4px; border-radius: 2px; background: linear-gradient(90deg, var(--brand-fill) var(--pct), var(--line) var(--pct)); }
.range::-webkit-slider-thumb { -webkit-appearance: none; width: 20px; height: 20px; margin-top: -8px; border-radius: 50%; background: #fff; border: 1px solid #dadada; box-shadow: 0 1px 4px rgba(0, 0, 0, 0.18); }
.range::-moz-range-thumb { width: 20px; height: 20px; border-radius: 50%; background: #fff; border: 1px solid #dadada; box-shadow: 0 1px 4px rgba(0, 0, 0, 0.18); }
.range:disabled { opacity: 0.5; }
.spin { width: 14px; height: 14px; border-radius: 50%; border: 2px solid currentColor; border-right-color: transparent; animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
@media (prefers-reduced-motion: reduce) { .spin { animation-duration: 2.4s; } }

/* 宽屏 */
.pick { padding: 22px; }
.drop { display: grid; place-items: center; align-content: center; gap: 10px; height: 300px; border: 1.5px dashed var(--line-strong); border-radius: var(--r-md); color: var(--ink-2); text-align: center; cursor: pointer; transition: border-color 0.15s, background 0.15s; }
.drop:hover, .drop.over { border-color: var(--brand-fill); background: color-mix(in srgb, var(--brand-fill) 5%, var(--surface)); }
.drop svg { color: var(--ink-3); }
.drop b { font-size: 16px; color: var(--ink); }
.drop .link { color: var(--brand); font-weight: 600; }
.drop small { color: var(--ink-3); font-size: var(--fs-xs); }
.edit { display: grid; grid-template-columns: 392px minmax(0, 1fr); }
.left { padding: 16px 16px 8px; border-right: 1px solid var(--line); }
.stage { position: relative; width: 360px; height: 360px; border-radius: var(--r-sm); overflow: hidden; background: var(--surface-2); }
.stage canvas { display: block; cursor: grab; }
.stage canvas:active { cursor: grabbing; }
.stage canvas:focus-visible { outline: 2px solid var(--brand-fill); outline-offset: -2px; }
.stage.over::after { content: '松开替换图片'; position: absolute; inset: 0; display: grid; place-items: center; background: color-mix(in srgb, var(--surface) 85%, transparent); color: var(--brand); font-weight: 600; border: 1.5px dashed var(--brand-fill); border-radius: var(--r-sm); }
.busy .left, .busy .stage { opacity: 0.7; }
.tools { display: flex; align-items: center; gap: 4px; margin-top: 10px; }
.tools .btn { margin-left: 6px; }
.ib { width: 32px; height: 32px; border: 0; border-radius: var(--r-sm); background: transparent; color: var(--ink-2); display: grid; place-items: center; }
.ib:hover:not(:disabled) { background: var(--surface-2); }
.ib:disabled { opacity: 0.4; }
.warn { margin: 6px 0 0; min-height: 18px; font-size: var(--fs-xs); color: #9a5b00; }
.right { padding: 16px 22px; display: grid; gap: 18px; align-content: start; }
.cap { margin: 0 0 8px; font-size: var(--fs-xs); font-weight: 600; color: var(--ink-3); }
.cmp, .sizes { display: flex; align-items: center; gap: 14px; }
.sizes { align-items: flex-end; gap: 18px; }
figure { margin: 0; display: grid; justify-items: center; gap: 4px; font-size: var(--fs-xs); color: var(--ink-3); }
figure.new { color: var(--ink); }
.arrow { color: var(--line-strong); margin-top: -18px; flex: none; }
.topic { border: 1px solid var(--line); border-radius: var(--r-sm); }
.topic .t { padding: 10px 12px; display: grid; gap: 4px; }
.topic .t + .t { border-top: 1px solid var(--line); }
.ttl { font-size: 14px; font-weight: 600; }
.by { display: flex; align-items: center; gap: 6px; font-size: var(--fs-xs); color: var(--ink-3); }
.by b { color: var(--ink-2); }
.cm { display: flex; gap: 8px; font-size: 13px; line-height: 1.5; }
.err { display: flex; align-items: center; gap: 8px; margin: 0 22px; padding: 9px 12px; border-radius: var(--r-sm); background: color-mix(in srgb, var(--brand-fill) 10%, var(--surface)); color: var(--brand); font-size: 13px; }
.foot { display: flex; align-items: center; gap: 10px; padding: 12px 22px 16px; }
.foot .hint { margin-right: auto; font-size: var(--fs-xs); color: var(--ink-3); }
.foot kbd { font-family: var(--font-mono); font-size: 11px; border: 1px solid var(--line-strong); border-bottom-width: 2px; border-radius: 4px; padding: 0 4px; color: var(--ink-2); background: var(--surface); }
.foot .btn-primary .spin { margin-right: 2px; }

/* 窄屏：与 App 一致 */
.m-page { position: fixed; inset: 0; z-index: 1000; display: flex; flex-direction: column; background: var(--surface); color: var(--ink); padding-top: env(safe-area-inset-top); }
.m-top { height: 52px; flex: none; display: grid; grid-template-columns: 48px 1fr 48px; align-items: center; padding: 0 4px; }
.m-top h2 { margin: 0; text-align: center; font-size: 18px; font-weight: 600; }
.m-icon { width: 48px; height: 48px; border: 0; border-radius: 12px; background: transparent; color: var(--ink); display: grid; place-items: center; }
.m-icon:disabled { opacity: 0.35; }
.m-icon:not(:disabled):active { background: var(--surface-2); }
.m-body { flex: 1; min-height: 0; overflow-y: auto; }
.m-banner { display: flex; align-items: center; gap: 8px; margin: 0 12px 8px; padding: 10px 12px; border-radius: 8px; background: color-mix(in srgb, var(--brand-fill) 8%, var(--surface)); color: var(--brand); font-size: 13px; line-height: 18px; }
.m-stage { display: flex; justify-content: center; background: var(--surface-2); }
.m-stage canvas { display: block; }
.busy .m-stage { opacity: 0.7; }
.m-tools { display: flex; align-items: center; gap: 2px; padding: 8px 8px 0; }
.m-zoom-ico { color: var(--ink-3); flex: none; }
.m-hint { margin: 0; padding: 2px 16px 10px; min-height: 28px; text-align: center; font-size: var(--fs-xs); color: var(--ink-3); }
.m-hint.warn { color: #b26a00; }
.m-preview { display: flex; align-items: center; gap: 16px; padding: 12px 16px; border-top: 1px solid var(--line); }
.m-cmp { display: flex; align-items: center; gap: 8px; flex: none; }
.m-post { flex: 1; min-width: 0; border-left: 1px solid var(--line); padding-left: 16px; }
.m-post .who { display: flex; align-items: center; gap: 6px; font-size: var(--fs-xs); color: var(--ink-3); }
.m-post .who b { color: var(--ink); font-size: 13px; }
.m-post p { margin: 4px 0 0; font-size: 13px; line-height: 18px; color: var(--ink-2); }
.m-actions { flex: none; display: flex; gap: 12px; padding: 12px 16px calc(16px + env(safe-area-inset-bottom)); }
.pill { flex: 1; height: 48px; border: 0; border-radius: 999px; font-size: 15px; font-weight: 600; display: inline-flex; align-items: center; justify-content: center; gap: 8px; }
.pill.sec { background: var(--surface-2); color: var(--ink-2); }
.pill.pri { background: color-mix(in srgb, var(--brand-fill) 10%, var(--surface)); color: var(--brand); }
.pill:disabled { opacity: 0.55; }
.busy .pill.pri:disabled { opacity: 1; }
.m-sheet-bg { position: fixed; inset: 0; z-index: 1001; background: rgba(0, 0, 0, 0.28); display: flex; align-items: flex-end; }
.m-sheet { width: 100%; background: var(--surface); color: var(--ink); border-radius: 16px 16px 0 0; box-shadow: 0 -4px 24px rgba(0, 0, 0, 0.08); padding: 8px 0 env(safe-area-inset-bottom); }
.grab { width: 36px; height: 4px; border-radius: 2px; background: var(--line-strong); margin: 0 auto 8px; }
.m-sheet h3 { margin: 0; padding: 8px 16px 8px; font-size: 18px; font-weight: 600; }
.m-row { position: relative; display: flex; align-items: center; gap: 12px; min-height: 52px; padding: 0 16px; font-size: 15px; cursor: pointer; }
.m-row svg { color: var(--ink-2); }
.m-row:active { background: var(--surface-2); }
.m-cancel { display: block; width: 100%; height: 52px; border: 0; border-top: 8px solid var(--surface-2); background: var(--surface); color: var(--ink-2); font-size: 15px; margin-top: 8px; }
</style>

<style>
html.avatar-crop-open, html.avatar-crop-open body { overflow: hidden; }
</style>
