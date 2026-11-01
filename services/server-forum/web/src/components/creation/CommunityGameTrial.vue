<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, shallowRef } from 'vue'
import { contentApi, type PackDetail } from '@/api/content'
import { readGameZip, sandboxUrl, validateDownload } from '@/lib/creationTrial'
const props = defineProps<{ pack: PackDetail; version: number }>()
const frame = ref<HTMLIFrameElement | null>(null), active = ref(false), loading = ref(false), error = ref(''), files = shallowRef<Record<string,string> | null>(null), nonce = ref(''), host = ref('')
let sequence = 0
let hostTimer: ReturnType<typeof setTimeout> | undefined
const configured = Boolean(import.meta.env.VITE_GAME_SANDBOX_URL)
const eligible = computed(() => props.pack.kind === 'game' && props.pack.status === 'published' && [0,1].includes(props.pack.trust_level))
async function start() {
  if (loading.value) return
  const current = ++sequence; loading.value = true; error.value = ''; active.value = false
  try {
    const configured = import.meta.env.VITE_GAME_SANDBOX_URL as string | undefined
    if (!configured) throw new Error('网页试玩宿主尚未配置，请稍后再试')
    const url = new URL(sandboxUrl(configured, location.origin)); url.searchParams.set('parentOrigin', location.origin)
    const pack = await contentApi.pack(props.pack.id), info = await contentApi.download(props.pack.id, props.version)
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
  if (event.data.type === 'xiaoban-game-host-ready' && event.data.nonce === nonce.value && files.value) frame.value.contentWindow?.postMessage({type:'xiaoban-game-init',nonce:nonce.value,files:files.value},new URL(host.value).origin)
  if (event.data.type === 'xiaoban-game-ready' && event.data.nonce === nonce.value) clearTimeout(hostTimer)
  if (event.data.type === 'xiaoban-game-error' && event.data.nonce === nonce.value) error.value=String(event.data.message || '游戏加载失败')
}
window.addEventListener('message', receive)
function stop() { clearTimeout(hostTimer); sequence++; active.value=false; loading.value=false; files.value=null; host.value='' }
onBeforeUnmount(() => { stop(); window.removeEventListener('message',receive) })
</script>
<template>
  <section v-if="eligible" class="trial">
    <p v-if="!configured">网页试玩暂不可用，可在客户端体验。</p>
    <div v-else-if="!active"><button class="btn btn-primary" :disabled="loading" type="button" @click="start">{{ loading ? '正在校验…' : '试玩游戏' }}</button></div>
    <div v-if="active" class="bar"><strong>游戏试玩</strong><button class="btn" type="button" @click="start">重新开始</button><button class="btn" type="button" @click="stop">退出试玩</button></div>
    <iframe v-show="active" ref="frame" :src="host || undefined" title="隔离游戏试玩宿主" sandbox="allow-scripts allow-same-origin" referrerpolicy="no-referrer"></iframe>
    <p v-if="error" role="alert">{{ error }}</p>
  </section>
</template>
<style scoped>.trial { margin:16px 0; }.trial .btn { min-height:44px; }.bar { display:flex; gap:8px; align-items:center; flex-wrap:wrap; margin-bottom:8px; }.bar strong { margin-right:auto; }.trial iframe { width:100%; height:min(70vh,700px); min-height:420px; border:0; background:var(--surface-2); }.trial p { color:var(--danger); }</style>
