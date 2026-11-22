<script setup lang="ts">
import { ref, shallowRef, watch } from 'vue'
import { contentApi, type MyPack } from '@/api/content'
import AppIcon from '@/components/AppIcon.vue'
import { kindLabel } from '@/lib/content'
import { convertPublishHtml, inspectPackZip, isPublishHtml, type PreparedPack } from '@/lib/publishPack'

export type DraftWebWork =
  | {
      source: 'upload'
      prepared: PreparedPack
      originalName: string
      format: 'zip' | 'html' | 'htm'
      rightsAccepted: boolean
    }
  | {
      source: 'mine'
      id: number
      title: string
      kind: string
      version: number
    }

const props = defineProps<{
  open: boolean
  current: DraftWebWork | null
  disabled?: boolean
}>()

const emit = defineEmits<{
  close: []
  select: [DraftWebWork]
}>()

const tab = ref<'upload' | 'mine'>('upload')
const preparing = ref(false)
const statusText = ref('')
const error = ref('')
const dragging = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)
const myPacks = shallowRef<MyPack[] | null>(null)
const loadingMine = ref(false)
const mineError = ref('')
let pickToken = 0

watch(() => props.open, isOpen => {
  if (!isOpen) return
  error.value = ''
  statusText.value = ''
  tab.value = props.current?.source === 'mine' ? 'mine' : 'upload'
  if (tab.value === 'mine' && myPacks.value === null) void loadMine()
})

watch(tab, next => {
  error.value = ''
  if (next === 'mine' && myPacks.value === null) void loadMine()
})

async function loadMine(force = false) {
  if (loadingMine.value && !force) return
  loadingMine.value = true
  mineError.value = ''
  try {
    const res = await contentApi.myPacks()
    myPacks.value = res.items.filter(p => p.status !== 'removed')
  } catch (e) {
    mineError.value = e instanceof Error ? e.message : '读取我的作品失败'
  } finally {
    loadingMine.value = false
  }
}

async function handleFile(file: File) {
  if (props.disabled || preparing.value) return
  const token = ++pickToken
  error.value = ''
  const isHtml = isPublishHtml(file.name)
  const isZip = /\.zip$/i.test(file.name)
  if (!isHtml && !isZip) {
    error.value = '请选择 .zip（V2 容器）、.html 或 .htm 文件'
    return
  }
  const format: 'zip' | 'html' | 'htm' = isZip
    ? 'zip'
    : /\.htm$/i.test(file.name)
      ? 'htm'
      : 'html'
  preparing.value = true
  statusText.value = isHtml ? '正在转换为 V2 容器…' : '正在检查作品包…'
  try {
    const prepared = isHtml ? await convertPublishHtml(file) : await inspectPackZip(file)
    if (token !== pickToken || !props.open) return
    emit('select', {
      source: 'upload',
      prepared,
      originalName: file.name,
      format,
      rightsAccepted: false,
    })
    emit('close')
  } catch (e) {
    if (token === pickToken) {
      error.value = e instanceof Error ? e.message : '无法读取作品文件'
    }
  } finally {
    if (token === pickToken) {
      preparing.value = false
      statusText.value = ''
    }
  }
}

function onPick(e: Event) {
  const el = e.target as HTMLInputElement
  const file = el.files?.[0]
  el.value = ''
  if (file) void handleFile(file)
}

function onDrop(e: DragEvent) {
  dragging.value = false
  const file = e.dataTransfer?.files?.[0]
  if (file) void handleFile(file)
}

function selectMine(pack: MyPack) {
  if (props.disabled) return
  emit('select', {
    source: 'mine',
    id: pack.id,
    title: pack.title,
    kind: pack.kind,
    version: pack.version,
  })
  emit('close')
}
</script>

<template>
  <div v-if="open" class="wp-mask" @click.self="!preparing && emit('close')">
    <div class="wp-sheet" role="dialog" aria-label="添加作品">
      <div class="wp-handle" aria-hidden="true" />
      <div class="wp-head">
        <h3>添加作品</h3>
        <button type="button" class="wp-x" aria-label="关闭" :disabled="preparing" @click="emit('close')">
          <AppIcon name="close" :size="16" />
        </button>
      </div>
      <p class="wp-note">每篇帖子最多附带 1 个可运行作品（切换方式或重新选择将替换当前作品）</p>
      <div class="wp-tabs" role="tablist" aria-label="添加作品方式">
        <button type="button" role="tab" :aria-selected="tab === 'upload'" :class="{ on: tab === 'upload' }" @click="tab = 'upload'">上传成品</button>
        <button type="button" role="tab" :aria-selected="tab === 'mine'" :class="{ on: tab === 'mine' }" @click="tab = 'mine'">关联我的创作</button>
      </div>

      <div class="wp-scroll">
        <template v-if="tab === 'upload'">
          <input ref="fileInput" type="file" hidden accept=".zip,.html,.htm" @change="onPick" />
          <div
            class="wp-drop"
            :class="{ dragging, busy: preparing }"
            role="button"
            tabindex="0"
            @click="!preparing && fileInput?.click()"
            @keydown.enter.prevent="!preparing && fileInput?.click()"
            @dragover.prevent="dragging = true"
            @dragleave="dragging = false"
            @drop.prevent="onDrop"
          >
            <span class="wp-drop-ico"><AppIcon name="upload" :size="20" /></span>
            <strong>{{ preparing ? statusText : '选择或拖入 .zip / .html / .htm 成品' }}</strong>
            <small>ZIP 需包含 V2 manifest.json；单个 HTML / HTM 上传后自动封装清单与握手，转为 V2 容器</small>
            <div class="wp-pills">
              <span class="wp-pill">.zip V2 容器</span>
              <span class="wp-pill">.html → 转 V2</span>
              <span class="wp-pill">.htm → 转 V2</span>
            </div>
          </div>
          <p v-if="error" class="wp-err" role="alert">{{ error }}</p>
        </template>

        <template v-else>
          <p v-if="loadingMine" class="wp-state">正在读取你的作品…</p>
          <div v-else-if="mineError" class="wp-state">
            <span>{{ mineError }}</span>
            <button type="button" class="btn btn-sm" @click="loadMine(true)">重试</button>
          </div>
          <div v-else-if="!myPacks?.length" class="wp-state">
            <span>还没有可关联的已发布作品，可以在上一页直接上传成品。</span>
          </div>
          <div v-else class="wp-list">
            <button
              v-for="pack in myPacks"
              :key="pack.id"
              type="button"
              class="wp-row"
              :class="{ on: current?.source === 'mine' && current.id === pack.id }"
              @click="selectMine(pack)"
            >
              <span class="wp-row-ico"><AppIcon name="grid" :size="17" /></span>
              <span class="wp-row-copy">
                <strong>{{ pack.title }}</strong>
                <small>{{ kindLabel(pack.kind) }} · v{{ pack.version }} · V2 容器</small>
              </span>
              <svg v-if="current?.source === 'mine' && current.id === pack.id" class="wp-check" viewBox="0 0 24 24" aria-hidden="true">
                <path d="M5 13l4 4L19 7" />
              </svg>
            </button>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.wp-mask {
  position: absolute;
  inset: 0;
  z-index: 30;
  background: rgba(0, 0, 0, 0.36);
  display: grid;
  place-items: center;
  padding: 20px;
}
.wp-sheet {
  width: 100%;
  max-width: 500px;
  max-height: 86%;
  border-radius: 20px;
  background: var(--surface);
  padding: 18px 20px 20px;
  box-shadow: 0 24px 60px -18px rgba(0, 0, 0, 0.35);
  display: flex;
  flex-direction: column;
  min-height: 0;
}
.wp-handle {
  display: none;
}
.wp-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
  flex: none;
}
.wp-head h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 700;
}
.wp-x {
  width: 30px;
  height: 30px;
  border-radius: 50%;
  border: 0;
  background: var(--surface-2);
  color: var(--ink-2);
  display: grid;
  place-items: center;
}
.wp-x:hover:not(:disabled) {
  color: var(--ink);
}
.wp-note {
  margin: 0 0 10px;
  font-size: 12px;
  line-height: 17px;
  color: var(--ink-3);
  flex: none;
}
.wp-tabs {
  display: grid;
  grid-template-columns: 1fr 1fr;
  padding: 3px;
  border-radius: 12px;
  background: var(--surface-2);
  gap: 2px;
  margin-bottom: 12px;
  flex: none;
}
.wp-tabs button {
  height: 34px;
  border: 0;
  border-radius: 9px;
  background: transparent;
  font-size: 13px;
  font-weight: 550;
  color: var(--ink-2);
}
.wp-tabs button.on {
  background: var(--surface);
  color: var(--ink);
  font-weight: 650;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
}
.wp-scroll {
  overflow-y: auto;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.wp-drop {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 8px;
  padding: 20px 14px;
  border-radius: 16px;
  background: var(--surface-2);
  border: 1.5px dashed var(--line-strong);
  cursor: pointer;
}
.wp-drop:hover,
.wp-drop.dragging {
  border-color: var(--brand);
}
.wp-drop.busy {
  opacity: 0.72;
  cursor: progress;
}
.wp-drop-ico {
  width: 40px;
  height: 40px;
  border-radius: 12px;
  background: var(--surface);
  color: var(--brand);
  display: grid;
  place-items: center;
}
.wp-drop strong {
  font-size: 14px;
  font-weight: 650;
  color: var(--ink);
}
.wp-drop small {
  font-size: 12px;
  line-height: 18px;
  color: var(--ink-2);
  max-width: 340px;
}
.wp-pills {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 6px;
  margin-top: 2px;
}
.wp-pill {
  padding: 2px 8px;
  border-radius: 99px;
  background: var(--surface);
  border: 1px solid var(--line);
  font-size: 11px;
  font-weight: 600;
  color: var(--ink-2);
}
.wp-err {
  margin: 0;
  padding: 9px 12px;
  border-radius: 10px;
  background: color-mix(in srgb, var(--danger) 12%, transparent);
  color: var(--danger);
  font-size: 12.5px;
  font-weight: 550;
}
.wp-state {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin: 0;
  padding: 16px 4px;
  font-size: 13px;
  color: var(--ink-2);
}
.wp-list {
  display: flex;
  flex-direction: column;
}
.wp-row {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 11px 8px;
  text-align: left;
  border: 0;
  border-bottom: 1px solid var(--line);
  border-radius: 10px;
  background: transparent;
}
.wp-row:last-child {
  border-bottom: 0;
}
.wp-row:hover {
  background: var(--surface-2);
}
.wp-row-ico {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  background: var(--surface-2);
  color: var(--ink-2);
  display: grid;
  place-items: center;
  flex: none;
}
.wp-row.on .wp-row-ico {
  color: var(--brand);
}
.wp-row-copy {
  flex: 1;
  min-width: 0;
}
.wp-row-copy strong {
  display: block;
  font-size: 14px;
  font-weight: 600;
  color: var(--ink);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.wp-row.on .wp-row-copy strong {
  color: var(--brand);
}
.wp-row-copy small {
  display: block;
  font-size: 12px;
  color: var(--ink-3);
  margin-top: 2px;
}
.wp-check {
  width: 18px;
  height: 18px;
  stroke: var(--brand);
  fill: none;
  stroke-width: 2.2;
  stroke-linecap: round;
  stroke-linejoin: round;
  flex: none;
}
@media (max-width: 767px) {
  .wp-mask {
    padding: 0;
    place-items: end center;
  }
  .wp-sheet {
    max-width: 100%;
    max-height: 78%;
    border-radius: 20px 20px 0 0;
    padding: 8px 16px calc(18px + env(safe-area-inset-bottom));
  }
  .wp-handle {
    display: block;
    width: 36px;
    height: 4px;
    border-radius: 99px;
    background: var(--line-strong);
    margin: 0 auto 10px;
    flex: none;
  }
}
</style>
