<script setup lang="ts">
import { computed, onBeforeUnmount, reactive, ref, shallowRef, useTemplateRef, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { contentApi, type PackDetail } from '@/api/content'
import { ApiError } from '@/api/forum'
import AppIcon from '@/components/AppIcon.vue'
import ModalDialog from '@/components/ModalDialog.vue'
import { track } from '@/lib/analytics'
import { COVER_LIMIT, fmtSize, kindLabel, KIND_LIMIT } from '@/lib/content'
import { preparePublishCover } from '@/lib/publishCover'
import { canPublishHtml, convertPublishHtml, inspectPackZip, isPublishHtml, WORK_TAGS, type PreparedPack } from '@/lib/publishPack'
import { uploadFile } from '@/lib/upload'
import { usePackPublisher } from '@/stores/packPublisher'
import { useSession } from '@/stores/session'
import { useToast } from '@/stores/toast'

const router = useRouter()
const route = useRoute()
const publisher = usePackPublisher()
const session = useSession()
const toast = useToast()
const packId = computed(() => publisher.target?.packId)
const existing = shallowRef<PackDetail | null>(null)
const loadError = ref('')
const retry = ref(0)
const allowHtml = computed(() => canPublishHtml(existing.value))
const form = reactive({ title: '', summary: '', tags: [] as string[], changelog: '', rights: false })
let defaults = { title: '', summary: '' }
const workInput = useTemplateRef<HTMLInputElement>('workInput')
const coverInput = useTemplateRef<HTMLInputElement>('coverInput')
const original = shallowRef<File | null>(null)
const work = shallowRef<PreparedPack | null>(null)
const cover = shallowRef<File | null>(null)
const preparing = ref(false)
const fileError = ref('')
const busy = ref(false)
const step = ref('')
const progress = ref(0)
const error = ref('')
let selection = 0
let upload: AbortController | null = null

const fileStatus = computed(() => {
  if (preparing.value) return isPublishHtml(original.value?.name ?? '') ? '正在转换为 V2 作品…' : '正在检查 ZIP…'
  if (work.value) {
    const { converted, protocol, kind, file, desktopResource } = work.value
    const format = desktopResource ? 'V2 桌面作品' : converted ? '已转换为 V2 作品' : protocol === 2 ? 'V2 作品' : '内容包'
    return `${format} · ${kindLabel(kind)} · ${fmtSize(file.size)}`
  }
  if (original.value) return fmtSize(original.value.size)
  const zip = !existing.value || existing.value.desktop_resource || existing.value.versions[0]?.manifest.runtimeProtocol === 2
    ? 'V2 ZIP' : 'ZIP'
  const size = KIND_LIMIT[existing.value?.kind ?? 'app'] ?? KIND_LIMIT.app!
  return `${allowHtml.value ? 'HTML、HTM 或 ' : ''}${zip} · ZIP 最大 ${fmtSize(size)}`
})

watch([packId, () => publisher.ready, retry], async ([id, ready], _, onCleanup) => {
  if (!id || !ready || existing.value) return
  const controller = new AbortController()
  onCleanup(() => controller.abort())
  loadError.value = ''
  try {
    const detail = await contentApi.pack(id, false, controller.signal)
    if (controller.signal.aborted) return
    if (detail.author?.uid !== session.user?.uid || detail.status === 'removed') {
      loadError.value = '这个内容包无法发布新版本'
      return
    }
    existing.value = detail
  } catch (cause) {
    if (!controller.signal.aborted) loadError.value = cause instanceof Error ? cause.message : '读取内容包失败'
  }
}, { immediate: true })

watch(() => publisher.ready, ready => { if (!ready) upload?.abort() }, { flush: 'sync' })
onBeforeUnmount(() => { selection++; upload?.abort() })

async function pickWork(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file || busy.value) return
  const current = ++selection
  original.value = file
  work.value = null
  error.value = ''
  fileError.value = ''
  preparing.value = true
  try {
    let prepared: PreparedPack
    if (isPublishHtml(file.name)) prepared = await convertPublishHtml(file, existing.value)
    else if (/\.zip$/i.test(file.name)) prepared = await inspectPackZip(file, existing.value)
    else throw new Error('请选择 HTML 或 ZIP 文件')
    if (current !== selection) return
    work.value = prepared
    if (!packId.value) {
      const summary = prepared.converted ? '' : prepared.description
      if (form.title === defaults.title) form.title = prepared.name
      if (form.summary === defaults.summary) form.summary = summary
      defaults = { title: prepared.name, summary }
    }
  } catch (cause) {
    if (current === selection) fileError.value = cause instanceof Error ? cause.message : '无法读取作品文件'
  } finally {
    if (current === selection) preparing.value = false
  }
}

function pickCover(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (file && !busy.value) selectCover(file)
}

function selectCover(file: File | null) {
  cover.value = file
  error.value = ''
}

function toggleTag(tag: string) {
  form.tags = form.tags.includes(tag) ? form.tags.filter(selected => selected !== tag) : [...form.tags, tag]
}

async function submit() {
  if (busy.value || preparing.value || !publisher.ready || (packId.value && !existing.value)) return
  const prepared = work.value
  error.value = ''
  if (!prepared) { if (!fileError.value) error.value = '请选择要发布的作品文件'; return }
  if (!packId.value) {
    if (!form.title.trim()) { error.value = '请填写标题'; return }
    if (!form.rights) { error.value = '请确认你拥有这份内容的发布权利'; return }
  }
  const controller = new AbortController()
  const { signal } = controller
  upload = controller
  busy.value = true
  let posting = false
  try {
    const id = packId.value
    let coverFile: File | undefined
    if (!id && cover.value) {
      step.value = '处理封面'
      coverFile = await preparePublishCover(cover.value, signal)
    }
    step.value = '上传作品文件'
    progress.value = 0
    const fileKey = await uploadFile(prepared.kind, prepared.file, ratio => (progress.value = ratio), signal)
    let coverKey: string | undefined
    if (coverFile) {
      step.value = '上传封面'
      progress.value = 0
      coverKey = await uploadFile('cover', coverFile, ratio => (progress.value = ratio), signal)
    }
    signal.throwIfAborted()
    step.value = id ? '发布新版本' : '发布作品'
    posting = true
    const result = id
      ? await contentApi.addVersion(id, fileKey, form.changelog.trim(), signal)
      : await contentApi.publish({
          kind: prepared.kind,
          title: form.title.trim(),
          summary: form.summary.trim(),
          tags: [...form.tags],
          file_key: fileKey,
          cover_key: coverKey,
          rights_ack: form.rights,
        }, signal)
    if (signal.aborted) return
    if (!id) track('publish_done', prepared.kind)
    toast.show(id ? '新版本已发布，正在等待重新审核' : '作品已发布')
    upload = null
    publisher.complete(result.id)
    void router.push(`/packs/${result.id}`)
  } catch (cause) {
    if (signal.aborted && !publisher.target) return
    error.value = posting && (signal.aborted || (cause instanceof ApiError && cause.status === 0))
      ? '发布结果尚未确认，请先到「我的作品」核对。'
      : signal.aborted ? '登录状态已变化，请重新确认后再试'
        : cause instanceof Error ? cause.message : '发布失败，请稍后重试'
  } finally {
    busy.value = false
    upload = null
  }
}
</script>

<template>
  <ModalDialog :open="publisher.visible" :title="packId ? '发布新版本' : '发布内容包'" :width="640"
    scrollable keep-on-backdrop :close-disabled="busy" @close="publisher.close()">
    <div v-if="!publisher.ready" class="dialog-state" role="status">
      <p>正在确认账号信息…</p>
      <button v-if="!session.restoring" type="button" class="btn btn-sm" @click="session.loadSession()">重新读取</button>
    </div>
    <div v-else-if="packId && !existing" class="dialog-state" :role="loadError ? 'alert' : 'status'">
      <p>{{ loadError || '正在读取内容包…' }}</p>
      <button v-if="loadError" type="button" class="btn btn-sm" @click="retry++">重试</button>
    </div>

    <form v-if="!packId || existing" v-show="publisher.ready" class="publish" novalidate @submit.prevent="submit">
      <div class="form-body" :inert="busy">
        <p v-if="existing" class="version-note">{{ existing.title }} · 当前版本 v{{ existing.version }}</p>
        <div class="field">
          <span>作品文件</span>
          <input ref="workInput" type="file" hidden :accept="allowHtml ? '.html,.htm,.zip' : '.zip'" @change="pickWork" />
          <button type="button" class="file-picker" :title="original?.name" @click="workInput?.click()">
            <span class="file-icon"><AppIcon name="upload" /></span>
            <span class="file-info">
              <strong>{{ original?.name ?? '选择文件' }}</strong>
              <span role="status">{{ fileStatus }}</span>
            </span>
            <span v-if="original" class="change-file">更换</span>
          </button>
          <small v-if="!packId">支持应用、游戏、壁纸、特效、组件和伙伴 ZIP。单个 HTML 会自动转换为 V2 应用。</small>
          <small v-else-if="allowHtml">单个 HTML 会自动转换为 V2 作品。包含独立脚本、样式或素材时，请上传 V2 ZIP。</small>
          <small v-else>ZIP 根目录需要有清单，作品的类型和 ID 需与原版本一致。</small>
          <p v-if="fileError" class="error" role="alert">{{ fileError }}</p>
        </div>

        <template v-if="!packId">
          <label class="field">
            <span>标题</span>
            <input v-model="form.title" class="input" required maxlength="60" placeholder="给作品起个名字" />
          </label>
          <label class="field">
            <span>简介（选填）</span>
            <textarea v-model="form.summary" class="input" rows="3" maxlength="500" placeholder="介绍一下你的作品" />
          </label>
          <div class="field" role="group" aria-label="标签（选填）">
            <span>标签（选填）</span>
            <div class="tags">
              <button v-for="tag in WORK_TAGS" :key="tag" type="button" class="tag"
                :aria-pressed="form.tags.includes(tag)" @click="toggleTag(tag)">{{ tag }}</button>
            </div>
          </div>
          <div class="field">
            <span>封面（选填）</span>
            <input ref="coverInput" type="file" hidden accept=".png,.jpg,.jpeg,.webp,image/png,image/jpeg,image/webp" @change="pickCover" />
            <div class="cover-picker">
              <button type="button" class="btn btn-sm" @click="coverInput?.click()">选择封面</button>
              <span v-if="cover" :title="cover.name">{{ cover.name }}</span>
              <button v-if="cover" type="button" class="btn btn-sm btn-ghost" @click="selectCover(null)">移除封面</button>
            </div>
            <small>支持 PNG、JPG 和 WebP。上传为 WebP，文件不超过 {{ fmtSize(COVER_LIMIT) }}。</small>
          </div>
          <label class="rights">
            <input v-model="form.rights" type="checkbox" required />
            <span>我确认这是我的原创，或已获得授权，并同意按社区规则发布。</span>
          </label>
        </template>
        <label v-else class="field">
          <span>更新说明（选填）</span>
          <textarea v-model="form.changelog" class="input" rows="3" maxlength="500" />
          <small>新版本发布后会重新进入审核。</small>
        </label>
      </div>

      <footer class="actions">
        <p v-if="error" class="error footer-message" role="alert">{{ error }}</p>
        <div v-if="busy" class="footer-message upload-progress">
          <span role="status">{{ step }}</span>
          <progress v-if="step.startsWith('上传')" :value="progress" max="1" :aria-label="step" />
        </div>
        <RouterLink v-if="!packId && !busy && route.name !== 'scheme-publish'" class="scheme-link" to="/schemes/new">发布方案</RouterLink>
        <button type="button" class="btn btn-ghost cancel" :disabled="busy" @click="publisher.close()">取消</button>
        <button type="submit" class="btn btn-primary" :disabled="busy || preparing || !!fileError">
          {{ busy ? '发布中…' : packId ? '发布新版本' : '发布' }}
        </button>
      </footer>
    </form>
  </ModalDialog>
</template>

<style scoped>
.publish { display: flex; flex-direction: column; min-height: 0; }
.form-body { display: grid; gap: 16px; align-content: start; min-width: 0; min-height: 0; padding: 18px 22px 22px; overflow-y: auto; overscroll-behavior: contain; }
.dialog-state { padding: 16px 22px 22px; color: var(--ink-2); }
.dialog-state p { margin: 0 0 8px; }
.version-note { margin: 0; color: var(--ink-2); font-size: var(--fs-sm); overflow-wrap: anywhere; }
.file-picker { display: flex; gap: 12px; align-items: center; min-width: 0; width: 100%; padding: 12px; border: 1px dashed var(--line-strong); border-radius: 12px; background: var(--surface-2); text-align: left; }
.file-picker:hover:not(:disabled) { border-color: var(--brand); }
.file-icon { display: grid; place-items: center; width: 38px; height: 38px; flex: none; color: var(--brand); background: var(--brand-soft); border-radius: 9px; }
.file-info { display: grid; gap: 3px; min-width: 0; flex: 1; }
.file-info strong { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: var(--fs-sm); }
.file-info > span { font-size: var(--fs-xs); color: var(--ink-3); }
.change-file { color: var(--brand); font-size: var(--fs-xs); flex: none; }
.tags { display: flex; flex-wrap: wrap; gap: 8px; }
.tag { min-height: 34px; padding: 4px 13px; border: 1px solid var(--line); border-radius: 999px; background: var(--surface); color: var(--ink-2); font-size: var(--fs-sm); }
.tag:hover:not(:disabled) { border-color: var(--line-strong); }
.tag[aria-pressed='true'] { color: var(--brand); border-color: var(--brand); background: var(--brand-soft); }
.cover-picker { display: flex; align-items: center; gap: 8px; min-width: 0; font-size: var(--fs-sm); }
.cover-picker button { flex: none; }
.cover-picker span { overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }
.rights { display: flex; align-items: flex-start; gap: 8px; font-size: var(--fs-sm); }
.rights input { flex: none; accent-color: var(--brand); }
.actions { display: flex; flex: none; flex-wrap: wrap; gap: 8px; align-items: center; padding: 12px 22px; border-top: 1px solid var(--line); }
.scheme-link { margin-right: auto; color: var(--ink-2); font-size: var(--fs-sm); }
.scheme-link:hover { color: var(--brand); }
.cancel { margin-left: auto; }
.error { margin: 0; color: var(--danger); font-size: var(--fs-sm); overflow-wrap: anywhere; }
.footer-message { flex: 1 0 100%; }
.upload-progress { display: flex; align-items: center; gap: 10px; font-size: var(--fs-sm); color: var(--ink-2); }
.upload-progress progress { flex: 1; height: 8px; accent-color: var(--brand); }
@media (max-width: 767px) {
  .form-body { padding-inline: 16px; }
  .actions { padding-inline: 16px; padding-bottom: calc(12px + env(safe-area-inset-bottom)); }
}
</style>
