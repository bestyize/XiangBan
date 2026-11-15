<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, shallowRef } from 'vue'
import { contentApi, runtimeBase, type PackDetail } from '@/api/content'
import { appRuntimeChannel, appRuntimeEntry, canTrialPack } from '@/lib/appRuntime'
import { appRuntimeStorage } from '@/lib/appRuntimeStorage'
import { readGameZip, sandboxUrl, validateDownload } from '@/lib/creationTrial'
const props = defineProps<{ pack: PackDetail; version: number; autostart?: boolean; standalone?: boolean }>()
const emit = defineEmits<{ exit: []; retry: [] }>()
const frame = ref<HTMLIFrameElement | null>(null), active = ref(false), loading = ref(false), error = ref(''), files = shallowRef<Record<string,string> | null>(null), nonce = ref(''), host = ref('')
const ready = ref(false)
let sequence = 0
let hostTimer: ReturnType<typeof setTimeout> | undefined
let exitTimer: ReturnType<typeof setTimeout> | undefined
let exitWait: Promise<void> | undefined
let resolveExit: (() => void) | undefined
const usesV2 = computed(() => props.pack.versions.find(v => v.version === props.version)?.manifest.runtimeProtocol === 2)
const configured = computed(() => usesV2.value ? Boolean(runtimeBase) : Boolean(import.meta.env.VITE_GAME_SANDBOX_URL))
const eligible = computed(() => canTrialPack(props.pack.kind, props.pack.status, props.pack.trust_level,
  props.pack.versions.find(v => v.version === props.version)?.manifest.runtimeProtocol))
const isApp = computed(() => props.pack.kind === 'app')
async function start() {
  if (!eligible.value || !configured.value || loading.value || exitWait) return
  const current = ++sequence; loading.value = true; ready.value = false; error.value = ''; active.value = false; host.value = ''
  try {
    // The standalone window has just checked the published version before mounting this frame.
    const pack = props.standalone ? props.pack : await contentApi.pack(props.pack.id)
    if (current !== sequence) return
    const selected = pack.versions.find(v => v.version === props.version)
    if (!selected || !canTrialPack(pack.kind, pack.status, pack.trust_level, selected.manifest.runtimeProtocol)) throw new Error('作品暂不可试玩')
    if (usesV2.value) {
      if (!runtimeBase) throw new Error('网页试玩宿主尚未配置，请稍后再试')
      const url = appRuntimeEntry(runtimeBase, pack.id, selected.version, selected.manifest)
      if (url.origin === location.origin) throw new Error('作品需要独立来源')
      host.value = url.href
      active.value = true
      clearTimeout(hostTimer)
      hostTimer = setTimeout(() => { if (current === sequence) error.value = '作品未响应，请稍后再试' }, 10000)
      return
    }
    const configured = import.meta.env.VITE_GAME_SANDBOX_URL as string | undefined
    if (!configured) throw new Error('网页试玩宿主尚未配置，请稍后再试')
    const url = new URL(sandboxUrl(configured, location.origin)); url.searchParams.set('parentOrigin', location.origin)
    const info = await contentApi.download(props.pack.id, props.version)
    validateDownload(pack, info, props.version)
    const resource = new URL(info.url, location.origin)
    const devOrigin = import.meta.env.VITE_GAME_RESOURCE_ORIGIN as string | undefined
    const resourceUrl = import.meta.env.DEV && devOrigin && resource.origin === new URL(devOrigin).origin && resource.pathname.startsWith('/files/') ? resource.pathname + resource.search : resource.href
    const response = await fetch(resourceUrl, { credentials:'omit', referrerPolicy:'no-referrer', redirect:'error' })
    if (!response.ok || !response.body) throw new Error('游戏下载失败')
    const reader = response.body.getReader(), chunks:Uint8Array[] = []; let length=0
    try { while (true) { const {done,value}=await reader.read(); if(done) break; length+=value.length; if(length>info.size) throw new Error('游戏包超过声明大小'); chunks.push(value) } } finally { await reader.cancel().catch(() => {}) }
    const bytes = new Uint8Array(length); let offset=0; for(const chunk of chunks) { bytes.set(chunk,offset); offset+=chunk.length }
    const result = await readGameZip(bytes,info.size,info.sha256)
    if (current !== sequence) return
    files.value=result; nonce.value=crypto.randomUUID(); url.searchParams.set('nonce', nonce.value); active.value=true
    await nextTick(); if (current !== sequence) return; host.value=url.href
    clearTimeout(hostTimer); hostTimer=setTimeout(() => { if (current === sequence) error.value='试玩宿主未响应，请检查配置或重试' }, 10000)
  } catch (e) { if (current === sequence) error.value=e instanceof Error ? e.message : '试玩加载失败' }
  finally { if (current === sequence) loading.value=false }
}
function receive(event:MessageEvent) {
  if (!active.value || !host.value || !frame.value || event.source !== frame.value.contentWindow || event.origin !== new URL(host.value).origin || !event.data) return
  if (usesV2.value) {
    if (event.data.channel !== appRuntimeChannel) return
    if (event.data.type === 'ready') {
      clearTimeout(hostTimer)
      ready.value = true
      error.value = ''
    }
    if (event.data.type === 'exit') void stop()
    if (event.data.type === 'storage' && Number.isSafeInteger(event.data.id)) {
      const { id, operation, key, value } = event.data
      const current = sequence
      const reply = (result: Record<string, unknown>) => {
        if (current !== sequence) return
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
window.addEventListener('message', receive)
function reset() {
  clearTimeout(hostTimer)
  clearTimeout(exitTimer)
  exitTimer = undefined
  resolveExit?.()
  resolveExit = undefined
  exitWait = undefined
  sequence++
  active.value = false; loading.value = false; ready.value = false; files.value = null; host.value = ''
}
function prepareExit() {
  if (exitWait) return exitWait
  if (!usesV2.value || !frame.value?.contentWindow || !host.value) return
  clearTimeout(hostTimer)
  frame.value.contentWindow.postMessage({ channel: appRuntimeChannel, type: 'hide' }, new URL(host.value).origin)
  exitWait = new Promise<void>(resolve => {
    resolveExit = resolve
    exitTimer = setTimeout(resolve, 500)
  })
  return exitWait
}
async function stop() {
  if (exitWait) return
  const current = sequence
  const wait = prepareExit()
  if (wait) await wait
  if (current === sequence) finish()
}
function finish() {
  reset()
  emit('exit')
}
onMounted(() => { if (props.autostart) void start() })
onBeforeUnmount(() => { reset(); window.removeEventListener('message',receive) })
defineExpose({ prepareExit })
</script>
<template>
  <section v-if="eligible" class="trial" :class="{ standalone }">
    <p v-if="!configured">请在小伴客户端打开{{ isApp ? '此应用' : '此游戏' }}。</p>
    <template v-else-if="!standalone">
      <div v-if="!active"><button class="btn btn-primary" :disabled="loading" type="button" @click="start">{{ loading ? '正在打开…' : isApp ? '打开应用' : '试玩游戏' }}</button></div>
      <div v-else class="bar"><strong>{{ isApp ? pack.title : '游戏试玩' }}</strong><button class="btn" type="button" @click="start">{{ isApp ? '重新打开' : '重新开始' }}</button><button class="btn" type="button" @click="stop">{{ isApp ? '关闭应用' : '退出试玩' }}</button></div>
    </template>
    <iframe v-if="active" :key="host" ref="frame" :src="host || undefined" :title="pack.title" sandbox="allow-scripts allow-same-origin" referrerpolicy="no-referrer"></iframe>
    <div v-if="standalone && (!ready || error)" class="window-status">
      <div v-if="error" role="alert">{{ error }} <button class="btn" type="button" @click="emit('retry')">重试</button></div>
      <span v-else role="status">正在加载应用…</span>
    </div>
    <p v-else-if="error" role="alert">{{ error }}</p>
  </section>
</template>
<style scoped>
.trial { margin:16px 0; }.trial .btn { min-height:44px; }.bar { display:flex; gap:8px; align-items:center; flex-wrap:wrap; margin-bottom:8px; }.bar strong { margin-right:auto; }.trial iframe { width:100%; height:min(70vh,700px); min-height:420px; border:0; background:var(--surface-2); }.trial p { color:var(--danger); }
.trial.standalone { position: relative; height: 100%; margin: 0; }
.trial.standalone iframe { display: block; height: 100%; min-height: 0; }
.window-status { position: absolute; inset: 0; display: grid; place-items: center; padding: 20px; text-align: center; background: var(--surface); color: var(--ink-2); }
.window-status .btn { margin-inline-start: 8px; }
</style>
