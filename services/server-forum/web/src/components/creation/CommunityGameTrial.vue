<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, shallowRef } from 'vue'
import { contentApi, runtimeBase, type PackDetail } from '@/api/content'
import { appRuntimeChannel, appRuntimeEntry } from '@/lib/appRuntime'
import { appRuntimeStorage } from '@/lib/appRuntimeStorage'
import { readGameZip, sandboxUrl, validateDownload } from '@/lib/creationTrial'
const props = defineProps<{ pack: PackDetail; version: number }>()
const frame = ref<HTMLIFrameElement | null>(null), active = ref(false), loading = ref(false), error = ref(''), files = shallowRef<Record<string,string> | null>(null), nonce = ref(''), host = ref('')
let sequence = 0
let hostTimer: ReturnType<typeof setTimeout> | undefined
let exitTimer: ReturnType<typeof setTimeout> | undefined
const usesV2 = computed(() => props.pack.versions.find(v => v.version === props.version)?.manifest.runtimeProtocol === 2)
const configured = computed(() => usesV2.value ? Boolean(runtimeBase) : Boolean(import.meta.env.VITE_GAME_SANDBOX_URL))
const eligible = computed(() => (props.pack.kind === 'game' || (usesV2.value && props.pack.kind === 'app')) &&
  props.pack.status === 'published' && [0,1].includes(props.pack.trust_level))
async function start() {
  if (loading.value || exitTimer) return
  const current = ++sequence; loading.value = true; error.value = ''; active.value = false; host.value = ''
  try {
    const pack = await contentApi.pack(props.pack.id)
    if (current !== sequence) return
    const selected = pack.versions.find(v => v.version === props.version)
    if (!selected || pack.status !== 'published' || ![0,1].includes(pack.trust_level)) throw new Error('作品暂不可试玩')
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
    if (event.data.type === 'ready') clearTimeout(hostTimer)
    if (event.data.type === 'exit') stop()
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
  if (event.data.type === 'xiaoban-game-ready' && event.data.nonce === nonce.value) clearTimeout(hostTimer)
  if (event.data.type === 'xiaoban-game-error' && event.data.nonce === nonce.value) error.value=String(event.data.message || '游戏加载失败')
}
window.addEventListener('message', receive)
function reset() {
  clearTimeout(hostTimer)
  clearTimeout(exitTimer)
  exitTimer = undefined
  sequence++
  active.value = false; loading.value = false; files.value = null; host.value = ''
}
function stop() {
  if (exitTimer) return
  if (usesV2.value && frame.value?.contentWindow && host.value) {
    clearTimeout(hostTimer)
    frame.value.contentWindow.postMessage({ channel: appRuntimeChannel, type: 'hide' }, new URL(host.value).origin)
    exitTimer = setTimeout(reset, 500)
  } else {
    reset()
  }
}
onBeforeUnmount(() => { reset(); window.removeEventListener('message',receive) })
</script>
<template>
  <section v-if="eligible" class="trial">
    <p v-if="!configured">网页试玩暂不可用，可在客户端体验。</p>
    <div v-else-if="!active"><button class="btn btn-primary" :disabled="loading" type="button" @click="start">{{ loading ? '正在校验…' : '试玩游戏' }}</button></div>
    <div v-if="active" class="bar"><strong>游戏试玩</strong><button class="btn" type="button" @click="start">重新开始</button><button class="btn" type="button" @click="stop">退出试玩</button></div>
    <iframe v-if="active" :key="host" ref="frame" :src="host || undefined" title="隔离游戏试玩宿主" sandbox="allow-scripts allow-same-origin" referrerpolicy="no-referrer"></iframe>
    <p v-if="error" role="alert">{{ error }}</p>
  </section>
</template>
<style scoped>.trial { margin:16px 0; }.trial .btn { min-height:44px; }.bar { display:flex; gap:8px; align-items:center; flex-wrap:wrap; margin-bottom:8px; }.bar strong { margin-right:auto; }.trial iframe { width:100%; height:min(70vh,700px); min-height:420px; border:0; background:var(--surface-2); }.trial p { color:var(--danger); }</style>
