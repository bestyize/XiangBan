<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import Icon, { type IconName } from '@/components/Icon.vue'
import AdminMoreMenu, { type MoreMenuItem } from '@/components/admin/AdminMoreMenu.vue'
import { deleteAppRelease, listAppReleases, saveAppRelease, uploadAppPackage } from '@/admin/api'
import { toast } from '@/composables/toast'
import { targets, type DesktopPackage, type DesktopRelease } from '@/desktop'

let uploadController: AbortController | null = null
onBeforeUnmount(() => uploadController?.abort())

const releases = ref<DesktopRelease[]>([])
const editor = ref<DesktopRelease | null>(null)
const editingVersion = ref('')
const error = ref('')
const busy = ref(false)
const loading = ref(true)
const uploading = ref('')
const progress = ref(0)
const uploadedNames = ref<Record<string, string>>({})
const form = ref<HTMLFormElement | null>(null)

const managed = (url: string) => url.startsWith('/api/v1/desktop/downloads/')
const size = (bytes?: number) =>
  !bytes
    ? ''
    : bytes < 1024
      ? `${bytes} B`
      : bytes < 1024 * 1024
        ? `${(bytes / 1024).toFixed(1)} KiB`
        : `${(bytes / 1024 / 1024).toFixed(2)} MiB`

const shortLabel = (id: string) =>
  id === 'windows-x86_64'
    ? 'Windows x64'
    : id === 'darwin-aarch64'
      ? 'macOS ARM'
      : id === 'darwin-x86_64'
        ? 'macOS Intel'
        : id === 'android-arm64'
          ? 'Android APK'
          : id

const osIcon = (id: string): IconName =>
  id.startsWith('windows') ? 'os-win' : id.startsWith('android') ? 'os-android' : 'os-apple'

const acceptOf = (id: string) =>
  id === 'windows-x86_64' ? '.exe,.msi,.zip' : id.startsWith('android') ? '.apk,.zip' : '.dmg,.pkg,.zip'

async function load() {
  loading.value = true
  try {
    releases.value = await listAppReleases()
    error.value = ''
  } catch (e) {
    error.value = (e as Error).message
  } finally {
    loading.value = false
  }
}

onMounted(load)

function create() {
  editingVersion.value = ''
  uploadedNames.value = {}
  editor.value = {
    version: '',
    notes: '',
    publishedAt: new Date().toISOString(),
    published: false,
    platforms: {},
  }
  error.value = ''
}

function edit(release: DesktopRelease) {
  editingVersion.value = release.version
  uploadedNames.value = {}
  editor.value = JSON.parse(JSON.stringify(release))
  error.value = ''
}

function dateInput(value: string) {
  if (!value) return ''
  const d = new Date(value)
  return new Date(d.getTime() - d.getTimezoneOffset() * 60000).toISOString().slice(0, 16)
}

function setDate(event: Event) {
  const value = (event.target as HTMLInputElement).value
  if (editor.value) editor.value.publishedAt = value ? new Date(value).toISOString() : ''
}

function statusInfo(release: DesktopRelease) {
  if (!release.published) return { text: '草稿', cls: 'is-warn' }
  if (new Date(release.publishedAt) > new Date()) return { text: '待发布', cls: 'is-info' }
  return { text: '已上架', cls: 'is-ok' }
}

function setExternalUrl(target: string, event: Event) {
  if (!editor.value) return
  const url = (event.target as HTMLInputElement).value.trim()
  if (!url) {
    delete editor.value.platforms[target]
    return
  }
  const existing: DesktopPackage | undefined = editor.value.platforms[target]
  if (url !== existing?.url) {
    editor.value.platforms[target] = { url }
  }
}

function cancelUpload() {
  uploadController?.abort()
}

async function upload(target: string, event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file || !editor.value) return
  if (file.size > 512 * 1024 * 1024) {
    error.value = '安装包不能超过 512 MiB'
    input.value = ''
    return
  }
  const draft = editor.value
  uploadController = new AbortController()
  uploading.value = target
  progress.value = 0
  error.value = ''
  try {
    const result = await uploadAppPackage(
      target,
      file,
      (percent) => (progress.value = percent),
      uploadController.signal,
    )
    draft.platforms[target] = {
      url: result.url,
      sha256: result.sha256,
      bytes: result.bytes,
    }
    uploadedNames.value[target] = result.fileName
    toast(`${shortLabel(target)} 安装包已上传`)
  } catch (e) {
    error.value = (e as Error).message
  } finally {
    uploading.value = ''
    uploadController = null
    input.value = ''
  }
}

async function persist(published: boolean) {
  if (!editor.value || busy.value || uploading.value || !form.value?.reportValidity()) return
  const entries = Object.entries(editor.value.platforms).filter(([, pkg]) => !!pkg?.url)
  if (!entries.length) {
    error.value = '请至少为一个平台上传安装包或填写下载链接'
    return
  }
  editor.value.platforms = Object.fromEntries(entries)
  editor.value.version = editor.value.version.trim()
  busy.value = true
  error.value = ''
  try {
    const saved = await saveAppRelease({ ...editor.value, published })
    await load()
    editor.value = null
    toast(
      published
        ? new Date(saved.publishedAt) > new Date()
          ? '已保存，到发布时间自动上架'
          : `v${saved.version} 已上架并同步官网`
        : '草稿已保存',
    )
  } catch (e) {
    error.value = (e as Error).message
  } finally {
    busy.value = false
  }
}

async function toggle(release: DesktopRelease) {
  busy.value = true
  error.value = ''
  try {
    await saveAppRelease({ ...release, published: !release.published })
    await load()
    toast(release.published ? '版本已下架' : '版本已上架')
  } catch (e) {
    error.value = (e as Error).message
  } finally {
    busy.value = false
  }
}

async function remove(release: DesktopRelease) {
  busy.value = true
  error.value = ''
  try {
    await deleteAppRelease(release.version)
    await load()
    toast(`已删除 v${release.version}`)
  } catch (e) {
    error.value = (e as Error).message
  } finally {
    busy.value = false
  }
}

function copySha(sha: string) {
  navigator.clipboard?.writeText(sha).catch(() => {})
  toast('已复制 SHA-256')
}

function menuFor(release: DesktopRelease): MoreMenuItem[] {
  return [
    { label: '编辑与传包', icon: 'edit', action: () => edit(release) },
    { sep: true },
    {
      label: '删除版本',
      icon: 'trash',
      danger: true,
      desc: '会从官网与客户端更新检测中移除',
      confirmText: '确认删除版本？',
      action: () => void remove(release),
    },
  ]
}
</script>

<template>
  <div class="xb-page">
    <header class="xb-page-head">
      <h1 class="xb-page-title">App 版本发布</h1>
      <div class="xb-page-actions">
        <RouterLink to="/admin/desktop" class="xb-btn-ghost">官网与公告</RouterLink>
        <button type="button" class="xb-btn-primary" :disabled="busy || !!uploading" @click="create">
          <Icon name="plus" :size="14" /><span>新建版本</span>
        </button>
      </div>
    </header>

    <div v-if="error && !editor" class="xb-alert is-danger" role="alert">
      <div class="xb-alert-l"><Icon name="alert" :size="15" /><span>{{ error }}</span></div>
    </div>

    <div v-if="loading" class="xb-group xb-empty-box">正在加载版本…</div>
    <div v-else-if="!releases.length" class="xb-group xb-empty-box">
      <Icon name="package" :size="28" />
      <strong>暂无 App 版本</strong>
      <button type="button" class="xb-btn-primary is-sm" @click="create">新建版本</button>
    </div>

    <div v-else class="rel-list">
      <section v-for="release in releases" :key="release.version" class="xb-group xb-group-pad rel-card">
        <div class="rel-head">
          <div class="rel-meta">
            <strong class="xb-num ver">v{{ release.version }}</strong>
            <span class="xb-pill" :class="statusInfo(release).cls">
              <i class="dot" />{{ statusInfo(release).text }}
            </span>
            <span class="xb-pill">
              {{ release.version.split('+')[0]?.includes('-') ? '预发布' : '稳定版' }}
            </span>
            <span class="time">{{ new Date(release.publishedAt).toLocaleString('zh-CN') }}</span>
          </div>
          <div class="rel-acts">
            <button type="button" class="xb-btn-ghost is-sm" :disabled="busy" @click="edit(release)">编辑</button>
            <button type="button" class="xb-btn-soft is-sm" :disabled="busy" @click="toggle(release)">
              {{ release.published ? '下架' : '上架' }}
            </button>
            <AdminMoreMenu :items="menuFor(release)" :disabled="busy" />
          </div>
        </div>

        <div class="plat-grid">
          <div
            v-for="t in targets"
            :key="t.id"
            class="plat-slot"
            :class="{ 'is-missing': !release.platforms[t.id] }"
          >
            <div class="plat-top">
              <span class="plat-os"><Icon :name="osIcon(t.id)" :size="14" /><span>{{ shortLabel(t.id) }}</span></span>
              <span
                v-if="release.platforms[t.id]"
                class="xb-pill"
                :class="managed(release.platforms[t.id].url) ? 'is-ok' : 'is-info'"
              >
                {{ managed(release.platforms[t.id].url) ? '站内' : '外链' }}
              </span>
              <span v-else class="xb-pill">未配置</span>
            </div>

            <template v-if="release.platforms[t.id]">
              <div class="plat-meta xb-num" :title="release.platforms[t.id].url">
                {{ managed(release.platforms[t.id].url) ? '站内安装包' : '外部下载链接' }}
                {{ size(release.platforms[t.id].bytes) ? `· ${size(release.platforms[t.id].bytes)}` : '' }}
              </div>
              <div v-if="release.platforms[t.id].sha256" class="sha-chip">
                <span>SHA256 {{ release.platforms[t.id].sha256?.slice(0, 10) }}…</span>
                <button type="button" title="复制 SHA-256" @click="copySha(release.platforms[t.id].sha256!)">
                  <Icon name="copy" :size="12" />
                </button>
              </div>
            </template>
          </div>
        </div>

        <pre class="notes">{{ release.notes || '暂无更新说明' }}</pre>
      </section>
    </div>

    <!-- 20px Slide-over Release Editor Drawer -->
    <Teleport to="body">
      <div v-if="editor" class="xb-backdrop" @click.self="!uploading && (editor = null)">
        <form ref="form" class="xb-drawer is-wide" @submit.prevent="persist(true)">
          <header class="xb-ov-head">
            <h2>{{ editingVersion ? `编辑 v${editingVersion}` : '新建版本' }}</h2>
            <button type="button" class="xb-icon-btn" :disabled="!!uploading" aria-label="关闭" @click="editor = null">
              <Icon name="close" :size="18" />
            </button>
          </header>

          <div class="xb-ov-body">
            <div v-if="error" class="xb-alert is-danger" role="alert">
              <div class="xb-alert-l"><Icon name="alert" :size="15" /><span>{{ error }}</span></div>
            </div>

            <div class="xb-split-eq">
              <label class="xb-field">
                <span class="xb-field-label">版本号</span>
                <input
                  v-model="editor.version"
                  class="xb-input xb-mono"
                  placeholder="3.2.0"
                  required
                  :disabled="!!editingVersion"
                />
              </label>
              <label class="xb-field" title="未来的发布时间将自动定时上架">
                <span class="xb-field-label">发布时间</span>
                <input
                  type="datetime-local"
                  class="xb-input xb-num"
                  :value="dateInput(editor.publishedAt)"
                  required
                  @change="setDate"
                />
              </label>
            </div>

            <label class="xb-field">
              <span class="xb-field-label">更新说明</span>
              <textarea v-model="editor.notes" class="xb-textarea" rows="3" placeholder="告诉用户这次更新了什么" />
            </label>

            <div v-if="uploading" class="upload-bar" role="status">
              <div class="upload-bar-top">
                <span>{{ shortLabel(uploading) }} 上传中 {{ progress }}%</span>
                <button type="button" class="xb-btn-ghost is-sm is-danger" @click="cancelUpload">取消</button>
              </div>
              <div class="xb-meter"><i :style="{ width: `${progress}%` }" /></div>
            </div>

            <div class="xb-field">
              <span class="xb-field-label">四端安装包（单包 ≤ 512 MiB）</span>
              <div class="xb-split-eq">
                <div v-for="t in targets" :key="t.id" class="xb-group xb-group-pad slot-ed">
                  <div class="plat-top">
                    <span class="plat-os"><Icon :name="osIcon(t.id)" :size="14" /><span>{{ t.label }}</span></span>
                    <button
                      v-if="editor.platforms[t.id]"
                      type="button"
                      class="xb-btn-ghost is-sm"
                      @click="delete editor.platforms[t.id]"
                    >
                      移除
                    </button>
                  </div>

                  <div v-if="editor.platforms[t.id] && managed(editor.platforms[t.id].url)" class="ok-info">
                    ✓ {{ uploadedNames[t.id] || '站内安装包已就绪' }}
                    {{ size(editor.platforms[t.id].bytes) ? `· ${size(editor.platforms[t.id].bytes)}` : '' }}
                  </div>

                  <label class="xb-btn-soft is-sm file-btn">
                    <Icon name="upload" :size="12" />
                    <span>{{ editor.platforms[t.id] && managed(editor.platforms[t.id].url) ? '更换安装包' : '上传安装包' }}</span>
                    <input
                      type="file"
                      :accept="acceptOf(t.id)"
                      :disabled="busy || !!uploading"
                      hidden
                      @change="upload(t.id, $event)"
                    />
                  </label>

                  <details class="ext-det">
                    <summary>外部下载链接</summary>
                    <input
                      type="url"
                      class="xb-input xb-mono"
                      style="margin-top: 8px; height: 30px; font-size: 12px"
                      :value="editor.platforms[t.id] && !managed(editor.platforms[t.id].url) ? editor.platforms[t.id].url : ''"
                      :placeholder="t.id.startsWith('android') ? 'https://…/XiaoBan.apk' : 'https://…/XiaoBan.zip'"
                      @input="setExternalUrl(t.id, $event)"
                    />
                  </details>
                </div>
              </div>
            </div>
          </div>

          <footer class="xb-ov-foot" style="justify-content: space-between">
            <button type="button" class="xb-btn-ghost" :disabled="!!uploading" @click="editor = null">取消</button>
            <div style="display: flex; gap: 8px">
              <button
                type="button"
                class="xb-btn-soft"
                :disabled="busy || !!uploading"
                @click="persist(false)"
              >
                {{ editor.published ? '保存并下架' : '保存草稿' }}
              </button>
              <button type="submit" class="xb-btn-primary" :disabled="busy || !!uploading">
                {{ busy ? '保存中…' : '保存并上架' }}
              </button>
            </div>
          </footer>
        </form>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.rel-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.rel-card {
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.rel-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}
.rel-meta,
.rel-acts {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.ver { font-size: 17px; }
.time { font-size: 12px; color: var(--ui-text-tertiary); }

.plat-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 10px;
}
.plat-slot {
  padding: 12px 13px;
  border-radius: 10px;
  background: var(--ui-surface);
  display: flex;
  flex-direction: column;
  gap: 7px;
}
.plat-slot.is-missing {
  background: transparent;
  border: 1px dashed var(--ui-border-strong);
  opacity: 0.7;
}
.plat-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
  font-size: 12.5px;
  font-weight: 600;
  line-height: 1;
}
.plat-os {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}
.plat-meta {
  font-size: 11.5px;
  color: var(--ui-text-tertiary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.sha-chip {
  margin-top: auto;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
  height: 24px;
  padding: 0 7px;
  border-radius: 6px;
  background: var(--ui-surface-sunken);
  font-family: var(--ui-mono);
  font-size: 10.5px;
  color: var(--ui-text-tertiary);
  line-height: 1;
}
.sha-chip button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--ui-text-secondary);
}
.notes {
  margin: 0;
  white-space: pre-wrap;
  font-family: inherit;
  font-size: 12.5px;
  color: var(--ui-text-secondary);
  line-height: 1.6;
}
.upload-bar {
  padding: 12px 14px;
  border-radius: 10px;
  background: var(--ui-soft);
  color: var(--ui-ink);
  display: flex;
  flex-direction: column;
  gap: 8px;
  font-size: 12.5px;
}
.upload-bar-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.slot-ed {
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.ok-info {
  font-size: 12px;
  color: var(--ui-success-text);
  font-weight: 550;
}
.file-btn {
  align-self: flex-start;
  cursor: pointer;
}
.ext-det {
  font-size: 12px;
  color: var(--ui-text-tertiary);
}
.ext-det summary { cursor: pointer; width: fit-content; }

@media (max-width: 960px) { .plat-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
@media (max-width: 640px) { .plat-grid { grid-template-columns: 1fr; } }
</style>
