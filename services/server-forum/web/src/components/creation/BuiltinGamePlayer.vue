<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { builtinGame, isBuiltinGameMessage } from '@/lib/builtinCreation'
const props = defineProps<{ gameId: string }>()
const emit = defineEmits<{ close: [] }>()
const game = computed(() => builtinGame(props.gameId)), frame = ref<HTMLIFrameElement | null>(null), dialog = ref<HTMLDialogElement | null>(null), nonce = ref(''), generation = ref(0), error = ref(''), loading = ref(false), ready = ref(false), connected = ref(false)
let sequence = 0
let handshakeTimer: ReturnType<typeof setTimeout> | undefined
let previousFocus: HTMLElement | null = null
async function restart() {
  clearTimeout(handshakeTimer)
  const current = ++sequence; ready.value = false; connected.value = false; error.value = ''; loading.value = true
  try {
    if (!game.value) throw new Error('找不到这个内置游戏')
    const response = await fetch(`/builtin-games/${game.value.id}/manifest.json`, {credentials:'omit'})
    if (!response.ok) throw new Error('游戏资源不可用')
    const manifest = await response.json()
    if (manifest.id !== game.value.id || manifest.entry !== 'index.html' || manifest.runtimeProtocol !== 1) throw new Error('游戏资源不完整')
    if (current !== sequence) return
    nonce.value = crypto.randomUUID(); generation.value++; ready.value = true
    handshakeTimer = setTimeout(() => { if (current === sequence && !connected.value) error.value = '游戏未响应，请重试' }, 15000)
  } catch (e) { if (current === sequence) error.value = e instanceof Error ? e.message : '游戏加载失败' }
  finally { if (current === sequence) loading.value = false }
}
watch(() => props.gameId, restart, { immediate:true })
onMounted(async () => { previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null; await nextTick(); dialog.value?.showModal() })
const url = computed(() => game.value && ready.value ? `/builtin-games/${game.value.id}/index.html?hostNonce=${encodeURIComponent(nonce.value)}` : '')
function initialize() { frame.value?.contentWindow?.postMessage({channel:'xiaban.game-host',protocol:1,type:'host.init',nonce:nonce.value,save:null,safeArea:{top:0,right:0,bottom:0,left:0}}, '*') }
function receive(event:MessageEvent) {
  if (!isBuiltinGameMessage(event.data, event.origin, event.source, frame.value?.contentWindow, nonce.value)) return
  const d = event.data
  if (d.type === 'game.ready') { connected.value = true; error.value = ''; clearTimeout(handshakeTimer) }
  if (d.type === 'game.exit') emit('close')
}
window.addEventListener('message',receive)
onBeforeUnmount(() => { sequence++; clearTimeout(handshakeTimer); window.removeEventListener('message',receive); dialog.value?.close(); previousFocus?.focus() })
</script>
<template>
  <dialog ref="dialog" class="player" :aria-label="game ? `${game.name}试玩` : '游戏试玩'" @cancel.prevent="emit('close')">
    <div class="bar"><strong>{{ game?.name ?? '游戏不可用' }}</strong><button v-if="game" class="btn" type="button" :disabled="loading" @click="restart">重新开始</button><button class="btn" type="button" @click="emit('close')">退出试玩</button></div>
    <p class="notice">本次试玩不保存进度。</p><p v-if="loading || (ready && !connected && !error)" role="status">游戏加载中…</p><p v-if="error" role="alert">{{ error }} <button class="btn" type="button" @click="restart">重试</button></p>
    <iframe v-if="game && ready" :key="generation" ref="frame" :src="url" :title="`${game.name}游戏区域`" sandbox="allow-scripts" referrerpolicy="no-referrer" @load="initialize" />
  </dialog>
</template>
<style scoped>.player { width:100vw; max-width:none; height:100dvh; max-height:none; margin:0; padding:0; border:0; background:var(--surface); color:var(--ink); }.player::backdrop { background:rgba(0,0,0,.5); }.bar { display:flex; align-items:center; gap:8px; padding:8px 12px; }.bar strong { margin-right:auto; }.bar .btn { min-height:44px; }.notice { margin:0; padding:0 12px 8px !important; font-size:12px; }.player iframe { display:block; width:100%; height:calc(100dvh - 88px); border:0; background:var(--surface-2); }.player p { padding:12px; color:var(--ink-2); }</style>
