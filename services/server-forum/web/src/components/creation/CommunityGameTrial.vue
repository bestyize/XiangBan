<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, shallowRef } from 'vue'
import AppRuntimeStatus from '@/components/creation/AppRuntimeStatus.vue'
import { contentApi, gameSandboxUrl, runtimeBase, type PackDetail } from '@/api/content'
import { appRuntimeChannel, appRuntimeEntry, canTrialPack } from '@/lib/appRuntime'
import { appRuntimeStorage } from '@/lib/appRuntimeStorage'
import { readGameZip, sandboxUrl, validateDownload } from '@/lib/creationTrial'
import { readIdeaPlayerDocument } from '@/lib/ideaPlayerDocument'
import { ideaPlayers, type IdeaPlayerId } from '@/lib/ideaPlayers'
const props = defineProps<{ pack?: PackDetail; version?: number; ideaId?: IdeaPlayerId }>()
const emit = defineEmits<{ exit: []; retry: [] }>()
const frame = ref<HTMLIFrameElement | null>(null)
const ready = ref(false), error = ref(''), host = ref('')
const message = ref('正在读取作品信息…')
const receivedBytes = ref<number>(), totalBytes = ref<number>()
const ideaDocument = ref<string>()
const files = shallowRef<Record<string, string> | null>(null), nonce = ref('')
const download = new AbortController()
let disposed = false
let exitTimer: ReturnType<typeof setTimeout> | undefined
let exitWait: Promise<void> | undefined
let resolveExit: (() => void) | undefined
const idea = computed(() => props.ideaId ? ideaPlayers[props.ideaId] : undefined)
const selected = computed(() => props.pack?.versions.find(v => v.version === props.version))
const usesV2 = computed(() => Boolean(idea.value) || selected.value?.manifest.runtimeProtocol === 2)
const sandbox = computed(() => {
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

async function start() {
  try {
    const { pack, version } = props
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
    if (usesV2.value) {
      if (!runtimeBase) throw new Error('网页试玩宿主尚未配置，请稍后再试')
      const url = appRuntimeEntry(runtimeBase, pack.id, version, manifest)
      if (url.origin === location.origin) throw new Error('作品需要独立来源')
      loadHost(url)
      return
    }
    if (!gameSandboxUrl) throw new Error('网页试玩宿主尚未配置，请稍后再试')
    const url = new URL(sandboxUrl(gameSandboxUrl, location.origin))
    url.searchParams.set('parentOrigin', location.origin)
    const info = await contentApi.download(pack.id, version, download.signal)
    if (disposed) return
    validateDownload(pack, info, version)
    const resource = new URL(info.url, location.origin)
    const devOrigin = import.meta.env.VITE_GAME_RESOURCE_ORIGIN as string | undefined
    const resourceUrl = import.meta.env.DEV && devOrigin && resource.origin === new URL(devOrigin).origin && resource.pathname.startsWith('/files/') ? resource.pathname + resource.search : resource.href
    message.value = '正在下载作品资源…'
    receivedBytes.value = 0
    totalBytes.value = info.size
    const response = await fetch(resourceUrl, { credentials: 'omit', referrerPolicy: 'no-referrer', redirect: 'error', signal: download.signal })
    if (!response.ok || !response.body) throw new Error('游戏下载失败')
    const reader = response.body.getReader()
    const bytes = new Uint8Array(info.size)
    let length = 0
    try {
      while (true) {
        const { done, value } = await reader.read()
        if (done) break
        if (length + value.length > bytes.length) throw new Error('游戏包超过声明大小')
        bytes.set(value, length)
        length += value.length
        receivedBytes.value = length
      }
    } finally {
      await reader.cancel().catch(() => {})
    }
    if (disposed) return
    receivedBytes.value = undefined
    totalBytes.value = undefined
    message.value = '正在校验作品资源…'
    const result = await readGameZip(bytes.subarray(0, length), info.size, info.sha256)
    if (disposed) return
    files.value = result
    nonce.value = crypto.randomUUID()
    url.searchParams.set('nonce', nonce.value)
    loadHost(url)
  } catch (e) {
    if (!disposed) error.value = e instanceof TypeError ? '资源请求失败，请检查网络。' : e instanceof Error ? e.message : '试玩加载失败'
  }
}
function receive(event:MessageEvent) {
  if (!host.value || !frame.value || event.source !== frame.value.contentWindow || !event.data) return
  const expectedOrigin = idea.value ? 'null' : new URL(host.value).origin
  if (event.origin !== expectedOrigin) return
  if (usesV2.value) {
    if (event.data.channel !== appRuntimeChannel) return
    if (event.data.type === 'ready') {
      ready.value = true
      error.value = ''
    }
    if (event.data.type === 'exit' && !exitWait) emit('exit')
    const storageOwner = props.ideaId === 'starry-nightrider' ? 'idea:starry-nightrider' : idea.value ? undefined : props.pack?.id
    if (event.data.type === 'storage' && storageOwner !== undefined && Number.isSafeInteger(event.data.id)) {
      const { id, operation, key, value } = event.data
      const reply = (result: Record<string, unknown>) => {
        if (disposed) return
        frame.value?.contentWindow?.postMessage({ channel: appRuntimeChannel, type: 'storage-result', id, ...result }, idea.value ? '*' : expectedOrigin)
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
  if (!usesV2.value || !frame.value?.contentWindow || !host.value) return
  frame.value.contentWindow.postMessage({ channel: appRuntimeChannel, type: 'hide' }, idea.value ? '*' : new URL(host.value).origin)
  exitWait = new Promise<void>(resolve => {
    resolveExit = resolve
    exitTimer = setTimeout(resolve, 500)
  })
  return exitWait
}
function resume() {
  if (!exitWait || !frame.value?.contentWindow || !host.value) return
  clearExitWait()
  frame.value.contentWindow.postMessage({ channel: appRuntimeChannel, type: 'show' }, idea.value ? '*' : new URL(host.value).origin)
}
onMounted(() => {
  window.addEventListener('message', receive)
  void start()
})
onBeforeUnmount(() => {
  disposed = true
  download.abort()
  clearExitWait()
  window.removeEventListener('message', receive)
})
defineExpose({ prepareExit, resume })
</script>
<template>
  <section class="trial">
    <iframe v-if="host" ref="frame" :src="ideaDocument ? undefined : host" :srcdoc="ideaDocument"
      :title="idea?.title ?? pack?.title" :sandbox="sandbox"
      referrerpolicy="no-referrer" @load="hostLoaded"></iframe>
    <AppRuntimeStatus v-if="!ready || error" class="window-status" :message="message" :error="error"
      :received-bytes="receivedBytes" :total-bytes="totalBytes" @retry="emit('retry')" />
  </section>
</template>
<style scoped>
.trial { position: relative; height: 100%; }
.trial iframe { display: block; width: 100%; height: 100%; border: 0; background: var(--surface-2); }
</style>
