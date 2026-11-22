<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import { track } from '@/lib/analytics'
import type { ShareDeletion, SharePayload } from '@/lib/detailShare'
import { canvasToBlob, renderPoster, sharePapers, type SharePaper } from '@/lib/poster'
import { useTheme } from '@/lib/theme'
import { useNarrow } from '@/lib/viewport'
import { useToast } from '@/stores/toast'

const props = defineProps<{ payload: SharePayload; deletion?: ShareDeletion; triggerClass?: string }>()
const router = useRouter()
const toast = useToast()
const { resolved, accent } = useTheme()
const mobile = useNarrow()
const dialog = ref<HTMLDialogElement>()
const rail = ref<HTMLElement>()
const cardButton = ref<HTMLButtonElement>()
const deleteButton = ref<HTMLButtonElement>()
const cancelDeleteButton = ref<HTMLButtonElement>()
const backButton = ref<HTMLButtonElement>()
const open = ref(false)
const checking = ref(false)
const studio = ref(false)
const confirmDelete = ref(false)
const deleting = ref(false)
const paper = ref<SharePaper>('white')
const excerpt = ref(true)
const qr = ref(true)
const railEnd = ref(true)
const failedThumb = ref(false)
const image = ref<{ blob: Blob; url: string }>()
const imageError = ref(false)
const retry = ref(0)
const copied = ref('')
const message = ref('')
const error = ref(false)
const systemShare = typeof navigator.share === 'function'
let feedbackTimer: number | undefined
let previewTracked = false

const showCard = computed(() => !props.payload.linkOnly && (!mobile.value || studio.value))
const thumbnail = computed(() => props.payload.thumbnail || props.payload.cover)
const contextMeta = computed(() => props.payload.meta || props.payload.author || new URL(props.payload.url).hostname)
const recommendation = computed(() => [props.payload.title, props.payload.excerpt, props.payload.url].filter(Boolean).join('\n'))
const imageFile = computed(() => image.value && new File(
  [image.value.blob], (props.payload.title.replace(/[\\/:*?"<>|]/g, '_').slice(0, 80) || '分享卡片') + '.png', { type: 'image/png' },
))
const canSendImage = computed(() => !!imageFile.value && navigator.canShare?.({ files: [imageFile.value] }) === true)

function clearFeedback() {
  window.clearTimeout(feedbackTimer)
  copied.value = ''
  message.value = ''
  error.value = false
}

function feedback(text: string, key = '', failed = false) {
  if (!open.value) return
  clearFeedback()
  copied.value = key
  message.value = text
  error.value = failed
  if (!failed) feedbackTimer = window.setTimeout(clearFeedback, 2000)
}

async function show() {
  const { verify, url } = props.payload
  if (verify) {
    checking.value = true
    try { await verify() }
    catch (e) {
      if (dialog.value && props.payload.url === url) toast.error(e)
      return
    } finally { checking.value = false }
    if (!dialog.value || props.payload.url !== url) return
  }
  previewTracked = false
  paper.value = 'white'
  excerpt.value = true
  qr.value = true
  studio.value = false
  confirmDelete.value = false
  failedThumb.value = false
  clearFeedback()
  open.value = true
  await nextTick()
  if (open.value) dialog.value?.showModal()
  syncRail()
}

function close() {
  if (deleting.value) return
  open.value = false
  dialog.value?.close()
  clearFeedback()
}

function escape() {
  if (deleting.value) return
  if (confirmDelete.value) void showDelete(false)
  else if (mobile.value && studio.value) void showStudio(false)
  else close()
}

function backdrop(event: MouseEvent) {
  if (event.target !== dialog.value || !dialog.value) return
  const { left, right, top, bottom } = dialog.value.getBoundingClientRect()
  if (event.clientX < left || event.clientX >= right || event.clientY < top || event.clientY >= bottom) close()
}

async function showStudio(value: boolean) {
  studio.value = value
  clearFeedback()
  await nextTick()
  if (value) backButton.value?.focus()
  else cardButton.value?.focus()
}

async function showDelete(value: boolean) {
  confirmDelete.value = value
  clearFeedback()
  await nextTick()
  if (value) cancelDeleteButton.value?.focus()
  else deleteButton.value?.focus()
}

function syncRail() {
  const el = rail.value
  railEnd.value = !el || el.scrollLeft + el.clientWidth >= el.scrollWidth - 4
}

function wheelRail(event: WheelEvent) {
  if (!rail.value || Math.abs(event.deltaY) <= Math.abs(event.deltaX) || rail.value.scrollWidth <= rail.value.clientWidth) return
  event.preventDefault()
  rail.value.scrollLeft += event.deltaY
}

watch(rail, (el, _, cleanup) => {
  syncRail()
  if (!el) return
  const observer = new ResizeObserver(syncRail)
  observer.observe(el)
  cleanup(() => observer.disconnect())
}, { flush: 'post' })

watch(() => props.deletion, value => { if (!value) confirmDelete.value = false })
watch([open, showCard, () => props.payload, paper, accent, excerpt, qr, retry], async (_, __, cleanup) => {
  let active = true
  let url = ''
  cleanup(() => { active = false; if (url) URL.revokeObjectURL(url) })
  image.value = undefined
  imageError.value = false
  if (!open.value || !showCard.value) return
  try {
    const blob = await canvasToBlob(await renderPoster(props.payload, {
      paper: paper.value, accent: accent.value, excerpt: excerpt.value, qr: qr.value,
    }))
    if (!active) return
    url = URL.createObjectURL(blob)
    image.value = { blob, url }
    if (!previewTracked) {
      previewTracked = true
      track('share_poster')
    }
  } catch {
    if (active) imageError.value = true
  }
})

onBeforeUnmount(() => {
  clearFeedback()
  dialog.value?.close()
})

async function copy(text: string, key: string) {
  try {
    await navigator.clipboard.writeText(text)
    feedback(key === 'link' ? '链接已复制' : '已复制', key)
    if (key === 'link') track('share_copy_link', props.payload.metric)
  } catch {
    feedback(key === 'link' ? '复制失败，请手动复制地址栏中的链接' : '复制失败，请重试', '', true)
  }
}

async function shareNative(files?: File[]) {
  try {
    await navigator.share(files ? { files } : { title: props.payload.title, url: props.payload.url })
  } catch (e) {
    if (!(e instanceof Error && e.name === 'AbortError')) {
      feedback(files ? '无法发送图片，请先保存图片' : '无法打开系统分享，请复制链接', '', true)
    }
  }
}

async function copyImage() {
  if (!image.value) return
  try {
    await navigator.clipboard.write([new ClipboardItem({ 'image/png': image.value.blob })])
    feedback('已复制图片', 'image')
  } catch {
    feedback('无法复制图片，请使用保存图片', '', true)
  }
}

function saveImage() {
  if (!image.value || !imageFile.value) return
  const link = document.createElement('a')
  link.href = image.value.url
  link.download = imageFile.value.name
  link.click()
  feedback('已下载 PNG', 'save')
}

function extra() {
  const item = props.payload.extra
  if (!item) return
  if ('text' in item) void copy(item.text(), 'extra')
  else window.open(item.href, '_blank', 'noopener,noreferrer')
}

async function ask() {
  try {
    await navigator.clipboard.writeText(`帮我解读和总结这条内容：\n${recommendation.value}`)
  } catch {
    feedback('复制失败，请重试', '', true)
    return
  }
  close()
  toast.show('内容已复制，请粘贴到小伴 AI 对话')
  void router.push('/ai')
}

async function remove() {
  const action = props.deletion
  if (!action || deleting.value) return
  deleting.value = true
  try {
    await action.run()
    deleting.value = false
    close()
  } catch (e) {
    feedback(e instanceof Error ? e.message : '删除失败，请重试', '', true)
  } finally {
    deleting.value = false
  }
}

const quoteAction = { id: 'quote', label: '复制推荐语', icon: 'quote', description: '标题 + 一句话摘要 + 链接', hint: '复制', run: () => copy(recommendation.value, 'quote') }
const askAction = { id: 'ask', label: '问问小伴', icon: 'ai', description: '带入小伴 AI 对话解读或总结', hint: '对话', run: ask }
const extraAction = computed(() => {
  const item = props.payload.extra
  if (!item) return
  return { id: 'extra', label: item.label, icon: item.icon, description: item.description, hint: 'text' in item ? '复制' : '打开', run: extra }
})
const moreActions = computed(() => [quoteAction, ...(extraAction.value ? [extraAction.value] : []), askAction])
const railActions = computed(() => [
  ...(systemShare ? [{ id: 'system', label: '系统分享', icon: 'share-up', run: () => shareNative() }] : []),
  { id: 'link', label: '复制链接', icon: 'link', run: () => copy(props.payload.url, 'link') },
  ...(!props.payload.linkOnly ? [
    { id: 'card', label: '分享卡片', icon: 'share-card', run: () => showStudio(true) },
    askAction,
    ...(extraAction.value?.hint === '复制' ? [extraAction.value] : []),
    quoteAction,
  ] : []),
  ...(props.deletion ? [{ id: 'delete', label: props.deletion.confirmLabel ? '下架' : '删除', icon: 'trash', run: () => showDelete(true) }] : []),
])
function recordRailButton(id: string, element: unknown) {
  const button = element instanceof HTMLButtonElement ? element : undefined
  if (id === 'card') cardButton.value = button
  if (id === 'delete') deleteButton.value = button
}
</script>

<template>
  <button type="button" :class="triggerClass || 'share-trigger'" aria-label="分享" title="分享" aria-haspopup="dialog" :aria-disabled="checking" @click="!checking && show()">
    <AppIcon name="share-up" :size="18" />
    <span v-if="!triggerClass">分享</span>
  </button>
  <Teleport to="body">
    <dialog ref="dialog" class="detail-share"
      :class="{ 'is-dark': resolved === 'dark', 'is-mobile': mobile, 'is-studio': mobile && studio, 'is-link-only': payload.linkOnly }"
      :aria-label="mobile && studio ? '分享卡片' : '分享'"
      @cancel.prevent="escape" @click="backdrop">
      <template v-if="open">
        <header v-if="!mobile" class="pc-modal-head">
          <div class="pc-modal-title-wrap"><h2>分享</h2><span class="pc-modal-badge">{{ payload.badge }}</span></div>
          <button type="button" class="pc-modal-close" aria-label="关闭" @click="close"><AppIcon name="close" :size="17" /></button>
        </header>
        <header v-else-if="studio" class="card-top">
          <button ref="backButton" type="button" class="card-top-icon" aria-label="返回" @click="showStudio(false)"><AppIcon name="back" :size="20" /></button>
          <h2>分享卡片</h2>
          <button type="button" class="card-top-icon" aria-label="关闭" @click="close"><AppIcon name="close" :size="20" /></button>
        </header>

        <div :class="mobile ? (studio ? 'studio-body' : 'share-sheet') : 'pc-modal-body'">
          <div v-if="showCard" :class="mobile ? 'card-stage' : 'pc-card-stage'">
            <div class="pc-card-holder">
              <img v-if="image" :src="image.url" class="card-preview" :alt="'分享卡片：' + payload.title" draggable="false">
              <div v-else class="card-pending">
                <template v-if="imageError">图片生成失败 <button type="button" @click="retry++">重试</button></template>
                <span v-else role="status">生成中…</span>
              </div>
            </div>
            <div class="card-controls" aria-label="卡片样式设置">
              <div class="tone-dots">
                <button v-for="item in sharePapers" :key="item.id" type="button" class="tone-dot"
                  :style="{ background: item.swatch }" :aria-label="item.label" :title="item.label"
                  :aria-pressed="paper === item.id" @click="paper = item.id" />
              </div>
              <span class="cust-sep"></span>
              <button v-if="payload.excerpt" type="button" class="card-toggle" :aria-pressed="excerpt" @click="excerpt = !excerpt"><span></span>摘录</button>
              <button type="button" class="card-toggle" :aria-pressed="qr" @click="qr = !qr"><span></span>二维码</button>
            </div>
          </div>

          <div v-if="!mobile" class="pc-actions-col">
            <div class="pc-actions-top">
              <section>
                <h3 class="pc-sec-label">链接地址</h3>
                <div class="pc-link-box">
                  <input class="pc-link-text" aria-label="链接地址" :value="payload.url" readonly @focus="($event.target as HTMLInputElement).select()">
                  <button type="button" class="pc-link-copy" :class="{ done: copied === 'link' }" @click="copy(payload.url, 'link')">
                    <AppIcon :name="copied === 'link' ? 'check' : 'link'" :size="14" />{{ copied === 'link' ? '已复制' : '复制链接' }}
                  </button>
                </div>
              </section>
              <section v-if="!payload.linkOnly">
                <h3 class="pc-sec-label">分享卡片</h3>
                <div class="pc-export-grid">
                  <button type="button" class="pc-export-tile primary" :class="{ done: copied === 'image' }" :disabled="!image" @click="copyImage">
                    <span class="pc-et-ico"><AppIcon :name="copied === 'image' ? 'check' : 'image-copy'" :size="17" /></span>
                    <span><strong class="pc-et-title">{{ copied === 'image' ? '已复制图片' : '复制图片' }}</strong><span class="pc-et-sub">直接粘贴到微信 / 飞书</span></span>
                  </button>
                  <button type="button" class="pc-export-tile" :class="{ done: copied === 'save' }" :disabled="!image" @click="saveImage">
                    <span class="pc-et-ico"><AppIcon :name="copied === 'save' ? 'check' : 'download'" :size="17" /></span>
                    <span><strong class="pc-et-title">{{ copied === 'save' ? '已下载 PNG' : '保存图片' }}</strong><span class="pc-et-sub">导出高清卡片到本地</span></span>
                  </button>
                </div>
              </section>
              <section v-if="!payload.linkOnly">
                <h3 class="pc-sec-label">更多方式</h3>
                <div class="pc-more-list">
                  <button v-for="item in moreActions" :key="item.id" type="button" class="pc-more-row" @click="item.run">
                    <span class="pc-mr-left">
                      <span class="pc-mr-ico"><AppIcon :name="item.icon" :size="16" /></span>
                      <span class="pc-mr-txt"><strong>{{ item.label }}</strong><span>{{ item.description }}</span></span>
                    </span>
                    <span class="pc-mr-state" :class="{ done: copied === item.id }">{{ copied === item.id ? '已复制' : item.hint }}</span>
                  </button>
                </div>
              </section>
            </div>

            <div v-if="showCard || deletion" class="pc-author-foot">
              <div v-if="confirmDelete && deletion" class="pc-confirm-strip">
                <span>{{ deletion.confirm }}</span>
                <div class="pc-confirm-btns">
                  <button type="button" ref="cancelDeleteButton" class="pc-c-cancel" :disabled="deleting" @click="showDelete(false)">取消</button>
                  <button type="button" class="pc-c-del" :disabled="deleting" @click="remove">{{ deleting ? '处理中…' : deletion.confirmLabel || '确认删除' }}</button>
                </div>
              </div>
              <template v-else>
                <span class="pc-foot-note"><template v-if="showCard && qr"><AppIcon name="qr" :size="14" />卡片二维码支持微信扫码在手机继续看</template></span>
                <button v-if="deletion" ref="deleteButton" type="button" class="pc-danger-btn" @click="showDelete(true)"><AppIcon name="trash" :size="14" />{{ deletion.label }}</button>
              </template>
            </div>
          </div>

          <template v-else-if="!studio">
            <div class="sheet-handle"></div>
            <div class="ctx-card">
              <span class="ctx-thumb">
                <img v-if="thumbnail && !failedThumb" :src="thumbnail" alt="" @error="failedThumb = true">
                <AppIcon v-else :name="payload.icon || 'document'" :size="19" />
              </span>
              <div class="ctx-info">
                <div class="ctx-title">{{ payload.title }}</div>
                <div class="ctx-sub">{{ contextMeta }}</div>
              </div>
              <span class="ctx-badge">{{ payload.badge }}</span>
            </div>

            <div v-if="confirmDelete && deletion" class="confirm-box">
              <strong>{{ deletion.confirm }}</strong>
              <p>{{ deletion.description || '删除后内容与评论将无法恢复' }}</p>
              <div class="confirm-actions">
                <button type="button" ref="cancelDeleteButton" class="confirm-cancel" :disabled="deleting" @click="showDelete(false)">取消</button>
                <button type="button" class="confirm-delete" :disabled="deleting" @click="remove">{{ deleting ? '处理中…' : deletion.confirmLabel || '确认删除' }}</button>
              </div>
            </div>
            <div v-else class="rail-wrap" :class="{ 'at-end': railEnd }">
              <div ref="rail" class="action-rail" @scroll.passive="syncRail" @wheel="wheelRail">
                <button v-for="item in railActions" :key="item.id" :ref="el => recordRailButton(item.id, el)"
                  type="button" class="action-item" :class="{ copied: copied === item.id, danger: item.id === 'delete', primary: item.id === 'system' }" @click="item.run">
                  <span class="action-tile"><AppIcon :name="copied === item.id ? 'check' : item.icon" :size="22" /></span>
                  <span class="action-label">{{ copied === item.id ? '已复制' : item.label }}</span>
                </button>
              </div>
            </div>
            <div class="sheet-cancel-band"><button type="button" class="sheet-cancel" :disabled="deleting" @click="close">取消</button></div>
          </template>

          <div v-else class="card-actions">
            <button type="button" class="card-act" :disabled="!image" @click="saveImage">
              <span class="card-act-ico"><AppIcon :name="copied === 'save' ? 'check' : 'download'" :size="22" /></span><span>保存图片</span>
            </button>
            <button type="button" class="card-act primary" :disabled="!canSendImage" title="通过系统分享图片" @click="imageFile && shareNative([imageFile])">
              <span class="card-act-ico"><AppIcon name="share-up" :size="22" /></span><span>发送给好友</span>
            </button>
            <button type="button" class="card-act" :disabled="!image" @click="copyImage">
              <span class="card-act-ico"><AppIcon :name="copied === 'image' ? 'check' : 'image-copy'" :size="22" /></span><span>复制图片</span>
            </button>
          </div>
        </div>
        <div class="share-feedback" role="status" aria-live="polite">
          <span v-if="message" :class="{ 'share-error': error }">{{ message }}</span>
        </div>
      </template>
    </dialog>
  </Teleport>
</template>

<style scoped src="./detail-share-dialog.css"></style>
