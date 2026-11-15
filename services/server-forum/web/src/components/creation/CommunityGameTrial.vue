<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, shallowRef } from 'vue'
import { contentApi, gameSandboxUrl, runtimeBase, type PackDetail } from '@/api/content'
import { appRuntimeChannel, appRuntimeEntry, canTrialPack } from '@/lib/appRuntime'
import { appRuntimeStorage } from '@/lib/appRuntimeStorage'
import { readGameZip, sandboxUrl, validateDownload } from '@/lib/creationTrial'
const props = defineProps<{ pack: PackDetail; version: number }>()
const emit = defineEmits<{ exit: []; retry: [] }>()
const frame = ref<HTMLIFrameElement | null>(null)
const ready = ref(false), error = ref(''), host = ref('')
const files = shallowRef<Record<string, string> | null>(null), nonce = ref('')
const download = new AbortController()
let disposed = false
let hostTimer: ReturnType<typeof setTimeout> | undefined
let exitTimer: ReturnType<typeof setTimeout> | undefined
let exitWait: Promise<void> | undefined
let resolveExit: (() => void) | undefined
const selected = computed(() => props.pack.versions.find(v => v.version === props.version))
const usesV2 = computed(() => selected.value?.manifest.runtimeProtocol === 2)
async function start() {
  try {
    const { pack, version } = props
    const manifest = selected.value?.manifest
    if (!manifest || !canTrialPack(pack.kind, pack.status, pack.trust_level, manifest.runtimeProtocol)) throw new Error('作品暂不可试玩')
    if (usesV2.value) {
      if (!runtimeBase) throw new Error('网页试玩宿主尚未配置，请稍后再试')
      const url = appRuntimeEntry(runtimeBase, pack.id, version, manifest)
      if (url.origin === location.origin) throw new Error('作品需要独立来源')
      host.value = url.href
      hostTimer = setTimeout(() => { error.value = '作品未响应，请稍后再试' }, 10000)
      return
    }
    if (!gameSandboxUrl) throw new Error('网页试玩宿主尚未配置，请稍后再试')
    const url = new URL(sandboxUrl(gameSandboxUrl, location.origin))
    url.searchParams.set('parentOrigin', location.origin)
    const info = await contentApi.download(pack.id, version)
    if (disposed) return
    validateDownload(pack, info, version)
    const resource = new URL(info.url, location.origin)
    const devOrigin = import.meta.env.VITE_GAME_RESOURCE_ORIGIN as string | undefined
    const resourceUrl = import.meta.env.DEV && devOrigin && resource.origin === new URL(devOrigin).origin && resource.pathname.startsWith('/files/') ? resource.pathname + resource.search : resource.href
    const response = await fetch(resourceUrl, { credentials: 'omit', referrerPolicy: 'no-referrer', redirect: 'error', signal: download.signal })
    if (!response.ok || !response.body) throw new Error('游戏下载失败')
    const reader = response.body.getReader(), chunks:Uint8Array[] = []; let length=0
    try { while (true) { const {done,value}=await reader.read(); if(done) break; length+=value.length; if(length>info.size) throw new Error('游戏包超过声明大小'); chunks.push(value) } } finally { await reader.cancel().catch(() => {}) }
    const bytes = new Uint8Array(length); let offset=0; for(const chunk of chunks) { bytes.set(chunk,offset); offset+=chunk.length }
    const result = await readGameZip(bytes,info.size,info.sha256)
    if (disposed) return
    files.value = result
    nonce.value = crypto.randomUUID()
    url.searchParams.set('nonce', nonce.value)
    host.value = url.href
    hostTimer = setTimeout(() => { error.value = '试玩宿主未响应，请检查配置或重试' }, 10000)
  } catch (e) {
    if (!disposed) error.value = e instanceof Error ? e.message : '试玩加载失败'
  }
}
function receive(event:MessageEvent) {
  if (!host.value || !frame.value || event.source !== frame.value.contentWindow || event.origin !== new URL(host.value).origin || !event.data) return
  if (usesV2.value) {
    if (event.data.channel !== appRuntimeChannel) return
    if (event.data.type === 'ready') {
      clearTimeout(hostTimer)
      ready.value = true
      error.value = ''
    }
    if (event.data.type === 'exit' && !exitWait) emit('exit')
    if (event.data.type === 'storage' && Number.isSafeInteger(event.data.id)) {
      const { id, operation, key, value } = event.data
      const reply = (result: Record<string, unknown>) => {
        if (disposed) return
        frame.value?.contentWindow?.postMessage({ channel: appRuntimeChannel, type: 'storage-result', id, ...result }, event.origin)
      }
      void appRuntimeStorage(props.pack.id, operation, key, value).then(
        value => reply({ value }),
        error => reply({ error: error?.name === 'QuotaExceededError' || error?.message === 'quota' ? 'quota' :
          error?.message === 'invalid' ? 'invalid' : 'unsupported' }),
      )
    }
    return
  }
  if (event.data.type === 'xiaoban-game-host-ready' && event.data.nonce === nonce.value && files.value) frame.value.contentWindow?.postMessage({type:'xiaoban-game-init',nonce:nonce.value,files:files.value},new URL(host.value).origin)
  if (event.data.type === 'xiaoban-game-ready' && event.data.nonce === nonce.value) {
    clearTimeout(hostTimer)
    ready.value = true
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
  frame.value.contentWindow.postMessage({ channel: appRuntimeChannel, type: 'hide' }, new URL(host.value).origin)
  exitWait = new Promise<void>(resolve => {
    resolveExit = resolve
    exitTimer = setTimeout(resolve, 500)
  })
  return exitWait
}
function resume() {
  if (!exitWait || !frame.value?.contentWindow || !host.value) return
  clearExitWait()
  frame.value.contentWindow.postMessage({ channel: appRuntimeChannel, type: 'show' }, new URL(host.value).origin)
}
onMounted(() => {
  window.addEventListener('message', receive)
  void start()
})
onBeforeUnmount(() => {
  disposed = true
  download.abort()
  clearTimeout(hostTimer)
  clearExitWait()
  window.removeEventListener('message', receive)
})
defineExpose({ prepareExit, resume })
</script>
<template>
  <section class="trial">
    <iframe v-if="host" ref="frame" :src="host" :title="pack.title" sandbox="allow-scripts allow-same-origin" referrerpolicy="no-referrer"></iframe>
    <div v-if="!ready || error" class="window-status">
      <div v-if="error" role="alert">{{ error }} <button class="btn" type="button" @click="emit('retry')">重试</button></div>
      <span v-else role="status">正在加载作品…</span>
    </div>
  </section>
</template>
<style scoped>
.trial { position: relative; height: 100%; }
.trial iframe { display: block; width: 100%; height: 100%; border: 0; background: var(--surface-2); }
.window-status { position: absolute; inset: 0; display: grid; place-items: center; padding: 20px; text-align: center; background: var(--surface); color: var(--ink-2); }
.window-status .btn { min-height: 44px; margin-inline-start: 8px; }
</style>
