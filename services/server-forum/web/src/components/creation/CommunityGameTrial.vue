<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, shallowRef } from 'vue'
import AppRuntimeStatus from '@/components/creation/AppRuntimeStatus.vue'
import { contentApi, gameSandboxUrl, runtimeBase, type DownloadInfo, type PackDetail } from '@/api/content'
import { appRuntimeChannel, appRuntimeEntry, canTrialPack, validateRuntimeDownload } from '@/lib/appRuntime'
import { appRuntimeStorage } from '@/lib/appRuntimeStorage'
import { readGameZip, sandboxUrl, validateDownload } from '@/lib/creationTrial'
import { readIdeaPlayerDocument } from '@/lib/ideaPlayerDocument'
import { ideaPlayers, type IdeaPlayerId } from '@/lib/ideaPlayers'
import { readRuntimeArchive, type PreparedPack } from '@/lib/publishPack'
import { sha256Hex } from '@/lib/upload'
import archiveBootstrap from '../../../runtime-v2/archive.js?raw'
import webBridge from '../../../runtime-v2/bridge.js?raw'

const props = defineProps<{ pack?: PackDetail; version?: number; ideaId?: IdeaPlayerId; draft?: PreparedPack }>()
const emit = defineEmits<{ exit: []; retry: [] }>()
const frame = ref<HTMLIFrameElement | null>(null)
const ready = ref(false), error = ref(''), host = ref('')
const message = ref('正在读取作品信息…')
const receivedBytes = ref<number>(), totalBytes = ref<number>()
const ideaDocument = ref<string>()
const archiveDocument = ref<string>()
const archiveInstance = ref('')
const archivePointerLock = ref(false)
let archive: Awaited<ReturnType<typeof readRuntimeArchive>> | undefined
const files = shallowRef<Record<string, string> | null>(null), nonce = ref('')
const download = new AbortController()
let disposed = false
let exitTimer: ReturnType<typeof setTimeout> | undefined
let exitWait: Promise<void> | undefined
let resolveExit: (() => void) | undefined
const idea = computed(() => props.ideaId ? ideaPlayers[props.ideaId] : undefined)
const selected = computed(() => props.pack?.versions.find(v => v.version === props.version))
const usesV2 = computed(() => Boolean(props.draft || idea.value) || selected.value?.manifest.runtimeProtocol === 2)
const inlineDocument = computed(() => archiveDocument.value ?? ideaDocument.value)
const sandbox = computed(() => {
  if (archiveInstance.value) return archivePointerLock.value ? 'allow-scripts allow-pointer-lock' : 'allow-scripts'
  if (idea.value) return idea.value.pointerLock ? 'allow-scripts allow-pointer-lock' : 'allow-scripts'
  const input = selected.value?.manifest.input
  return usesV2.value && props.pack?.kind === 'game' && Array.isArray(input) && input.includes('mouse')
    ? 'allow-scripts allow-same-origin allow-pointer-lock'
    : 'allow-scripts allow-same-origin'
})
function loadHost(url: URL) {
  receivedBytes.value = undefined
  totalBytes.value = undefined
  message.value = ideaDocument.value ? '正在启动作品…' : '正在加载作品资源…'
  host.value = url.href
}

function hostLoaded() {
  if (!ready.value && usesV2.value) message.value = '正在启动作品…'
}

async function startArchive(file: File, kind: string) {
  message.value = '正在读取作品包…'
  const prepared = await readRuntimeArchive(file, kind, download.signal)
  if (disposed) return
  archive = prepared
  archivePointerLock.value = prepared.pointerLock
  archiveInstance.value = crypto.randomUUID()
  const policy = "default-src 'none'; script-src 'unsafe-inline' 'wasm-unsafe-eval' blob: data:; style-src 'unsafe-inline' blob: data:; img-src blob: data:; media-src blob: data:; font-src blob: data:; connect-src blob: data:; worker-src blob:; base-uri 'none'; form-action 'none'"
  archiveDocument.value = `<!doctype html><meta charset="utf-8"><meta name="referrer" content="no-referrer">
<meta http-equiv="Content-Security-Policy" content="${policy}">
<script data-parent-origin="${location.origin}" data-instance="${archiveInstance.value}" data-preview="${Boolean(props.draft)}">${archiveBootstrap}<` + '/script>'
}

async function downloadArchive(info: DownloadInfo) {
  const resource = new URL(info.url, location.origin)
  const devOrigin = import.meta.env.VITE_GAME_RESOURCE_ORIGIN as string | undefined
  const resourceUrl = import.meta.env.DEV && devOrigin && resource.origin === new URL(devOrigin).origin && resource.pathname.startsWith('/files/') ? resource.pathname + resource.search : resource.href
  message.value = '正在下载作品资源…'
  receivedBytes.value = 0
  totalBytes.value = info.size
  const response = await fetch(resourceUrl, { credentials: 'omit', referrerPolicy: 'no-referrer', redirect: 'error', signal: download.signal })
  if (!response.ok || !response.body) throw new Error('作品下载失败')
  const reader = response.body.getReader()
  const bytes = new Uint8Array(info.size)
  let length = 0
  try {
    while (true) {
      const { done, value } = await reader.read()
      if (done) break
      if (length + value.length > bytes.length) throw new Error('作品包超过声明大小')
      bytes.set(value, length)
      length += value.length
      receivedBytes.value = length
    }
  } finally {
    await reader.cancel().catch(() => {})
  }
  if (length !== bytes.length) throw new Error('作品包下载不完整')
  receivedBytes.value = undefined
  totalBytes.value = undefined
  message.value = '正在校验作品资源…'
  return bytes
}

async function start() {
  try {
    const { pack, version, draft } = props
    if (draft) {
      await startArchive(draft.file, draft.kind)
      return
    }
    if (idea.value) {
      const url = new URL(`/idea-players/v1/${idea.value.entry}`, import.meta.env.DEV ? location.origin : 'https://res.xiangban.xyz')
      if (idea.value.selfContained && window === window.top) {
        message.value = '正在下载作品资源…'
        ideaDocument.value = await readIdeaPlayerDocument(url, sandbox.value, download.signal, (received, total) => {
          receivedBytes.value = received
          totalBytes.value = total
        }, location.origin)
        if (disposed) return
      }
      loadHost(url)
      return
    }
    const manifest = selected.value?.manifest
    if (!pack || version === undefined || !manifest || !canTrialPack(pack.kind, pack.status, pack.trust_level, manifest.runtimeProtocol)) {
      throw new Error('作品暂不可试玩')
    }
    if (usesV2.value && runtimeBase) {
      const url = appRuntimeEntry(runtimeBase, pack.id, version, manifest)
      if (url.origin === location.origin) throw new Error('作品需要独立来源')
      loadHost(url)
      return
    }
    if (usesV2.value) {
      const info = await contentApi.download(pack.id, version, download.signal)
      validateRuntimeDownload(pack, info, version)
      const bytes = await downloadArchive(info)
      const file = new File([bytes], `${pack.id}.zip`, { type: 'application/zip' })
      if (await sha256Hex(file) !== info.sha256.toLowerCase()) throw new Error('作品包 SHA-256 校验失败')
      await startArchive(file, pack.kind)
      return
    }
    if (!gameSandboxUrl) throw new Error('网页试玩宿主尚未配置，请稍后再试')
    const url = new URL(sandboxUrl(gameSandboxUrl, location.origin))
    url.searchParams.set('parentOrigin', location.origin)
    const info = await contentApi.download(pack.id, version, download.signal)
    if (disposed) return
    validateDownload(pack, info, version)
    const bytes = await downloadArchive(info)
    const result = await readGameZip(bytes, info.size, info.sha256)
    if (disposed) return
    files.value = result
    nonce.value = crypto.randomUUID()
    url.searchParams.set('nonce', nonce.value)
    loadHost(url)
  } catch (e) {
    if (!disposed) error.value = e instanceof TypeError ? '资源请求失败，请检查网络。' : e instanceof Error ? e.message : '试玩加载失败'
  }
}
function frameOrigin() {
  if (idea.value || archiveInstance.value) return 'null'
  return new URL(host.value).origin
}
function postRuntime(type: string, fields?: Record<string, unknown>) {
  const origin = frameOrigin()
  frame.value?.contentWindow?.postMessage({ channel: appRuntimeChannel, instance: archiveInstance.value || undefined, type, ...fields }, origin === 'null' ? '*' : origin)
}
function receive(event: MessageEvent) {
  if ((!host.value && !archiveDocument.value) || !frame.value || event.source !== frame.value.contentWindow || !event.data) return
  const expectedOrigin = frameOrigin()
  if (event.origin !== expectedOrigin) return
  if (usesV2.value) {
    if (event.data.channel !== appRuntimeChannel || (archiveInstance.value && event.data.instance !== archiveInstance.value)) return
    if (event.data.type === 'archive-ready' && archive) {
      postRuntime('archive-init', { ...archive, bridge: webBridge })
      archive = undefined
      return
    }
    if (event.data.type === 'archive-error' && archiveInstance.value) error.value = String(event.data.message)
    if (event.data.type === 'ready' || (event.data.type === 'archive-loaded' && archiveInstance.value)) {
      ready.value = true
      error.value = ''
    }
    if (event.data.type === 'exit' && !exitWait) emit('exit')
    const storageOwner = props.ideaId === 'starry-nightrider' ? 'idea:starry-nightrider' : idea.value ? undefined : props.pack?.id
    if (event.data.type === 'storage' && storageOwner !== undefined && Number.isSafeInteger(event.data.id)) {
      const { id, operation, key, value } = event.data
      const reply = (result: Record<string, unknown>) => {
        if (disposed) return
        postRuntime('storage-result', { id, ...result })
      }
      void appRuntimeStorage(storageOwner, operation, key, value).then(
        value => reply({ value }),
        error => reply({ error: error?.name === 'QuotaExceededError' || error?.message === 'quota' ? 'quota' :
          error?.message === 'invalid' ? 'invalid' : 'unsupported' }),
      )
    }
    return
  }
  if (event.data.type === 'xiaoban-game-host-ready' && event.data.nonce === nonce.value && files.value) {
    message.value = '正在启动作品…'
    frame.value.contentWindow?.postMessage({ type: 'xiaoban-game-init', nonce: nonce.value, files: files.value }, expectedOrigin)
  }
  if (event.data.type === 'xiaoban-game-ready' && event.data.nonce === nonce.value) {
    ready.value = true
    error.value = ''
  }
  if (event.data.type === 'xiaoban-game-error' && event.data.nonce === nonce.value) error.value=String(event.data.message || '游戏加载失败')
}
function clearExitWait() {
  clearTimeout(exitTimer)
  exitTimer = undefined
  resolveExit?.()
  resolveExit = undefined
  exitWait = undefined
}
function prepareExit() {
  if (exitWait) return exitWait
  if (!usesV2.value || !frame.value?.contentWindow || (!host.value && !archiveDocument.value)) return
  postRuntime('hide')
  exitWait = new Promise<void>(resolve => {
    resolveExit = resolve
    exitTimer = setTimeout(resolve, 500)
  })
  return exitWait
}
function resume() {
  if (!exitWait || !frame.value?.contentWindow) return
  clearExitWait()
  postRuntime('show')
}
onMounted(() => {
  window.addEventListener('message', receive)
  void start()
})
onBeforeUnmount(() => {
  disposed = true
  download.abort()
  archive = undefined
  clearExitWait()
  window.removeEventListener('message', receive)
})
defineExpose({ prepareExit, resume })
</script>
<template>
  <section class="trial">
    <iframe v-if="host || inlineDocument" ref="frame" :src="inlineDocument ? undefined : host" :srcdoc="inlineDocument"
      :title="idea?.title ?? pack?.title ?? draft?.name" :sandbox="sandbox"
      referrerpolicy="no-referrer" @load="hostLoaded"></iframe>
    <AppRuntimeStatus v-if="!ready || error" class="window-status" :message="message" :error="error"
      :local="Boolean(draft)" :received-bytes="receivedBytes" :total-bytes="totalBytes" @retry="emit('retry')" />
  </section>
</template>
<style scoped>
.trial { position: relative; height: 100%; }
.trial iframe { display: block; width: 100%; height: 100%; border: 0; background: var(--surface-2); }
</style>
