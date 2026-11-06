<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { builtinGame, isBuiltinGameMessage } from '@/lib/builtinCreation'
import { fitGameViewport } from '@/lib/gameViewport'
const props = defineProps<{ gameId: string; autostart?: boolean }>()
const emit = defineEmits<{ close: [] }>()
const game = computed(() => builtinGame(props.gameId)), frame = ref<HTMLIFrameElement | null>(null), dialog = ref<HTMLDialogElement | null>(null), nonce = ref(''), generation = ref(0), error = ref(''), loading = ref(false), ready = ref(false), connected = ref(false)
const surface=ref<HTMLElement>(), fullscreen=ref(false), fullscreenAvailable=ref(false), started=ref(false)
const stage=ref<HTMLElement>(), stageSize=ref({width:0,height:0})
const desktopInput=window.matchMedia('(hover: hover) and (pointer: fine)'), desktop=ref(desktopInput.matches)
const frameSize=computed(() => fitGameViewport(stageSize.value.width,stageSize.value.height,game.value?.orientation || 'landscape',desktop.value))
const frameStyle=computed(() => stageSize.value.width && stageSize.value.height ? {width:`${frameSize.value.width}px`,height:`${frameSize.value.height}px`} : {width:'100%',height:'100%'})
let stageObserver:ResizeObserver | undefined
let previousOverflow='', disposed=false, orientationLocked=false
const orientation=screen.orientation as ScreenOrientation & {lock?:(value:string)=>Promise<void>}
function fullscreenChanged() { fullscreen.value=!!document.fullscreenElement }
async function enterFullscreen() {
  const element=surface.value
  if (!element?.requestFullscreen || !document.fullscreenEnabled) return
  try {
    if (!document.fullscreenElement) await element.requestFullscreen()
    if (disposed) { if (document.fullscreenElement === element) await document.exitFullscreen(); return }
    if (!desktop.value && orientation?.lock) {
      try { await orientation.lock(game.value?.orientation === 'landscape' ? 'landscape' : 'portrait'); orientationLocked=true; if (disposed) orientation.unlock() } catch { /* Rotation remains available when the browser declines a lock. */ }
    }
  } catch { /* Keep the game usable and offer another user-gesture attempt. */ }
}
function updateInput() { desktop.value=desktopInput.matches }
let sequence = 0
let handshakeTimer: ReturnType<typeof setTimeout> | undefined
let previousFocus: HTMLElement | null = null
let manifestController:AbortController | undefined
async function restart() {
  clearTimeout(handshakeTimer)
  manifestController?.abort()
  const controller=new AbortController()
  manifestController=controller
  const manifestTimeout=setTimeout(() => controller.abort(),15000)
  const current = ++sequence; ready.value = false; connected.value = false; error.value = ''; loading.value = true
  try {
    if (!game.value) throw new Error('找不到这个内置游戏')
    const response = await fetch(`/builtin-games/${game.value.id}/manifest.json`, {credentials:'omit',signal:controller.signal})
    if (!response.ok) throw new Error('游戏资源不可用')
    const manifest = await response.json()
    if (manifest.id !== game.value.id || manifest.entry !== 'index.html' || manifest.runtimeProtocol !== 1) throw new Error('游戏资源不完整')
    if (current !== sequence) return
    nonce.value = crypto.randomUUID(); generation.value++; ready.value = true
    handshakeTimer = setTimeout(() => { if (current === sequence && !connected.value) error.value = '游戏未响应，请重试' }, 15000)
  } catch (e) { if (current === sequence) error.value = controller.signal.aborted ? '游戏加载超时，请重试' : e instanceof Error ? e.message : '游戏加载失败' }
  finally { clearTimeout(manifestTimeout); if (current === sequence) loading.value = false }
}
watch(() => props.gameId, restart, { immediate:true })
onMounted(() => {
  desktopInput.addEventListener('change',updateInput)
  fullscreenAvailable.value=!!surface.value?.requestFullscreen && document.fullscreenEnabled
  document.addEventListener('fullscreenchange',fullscreenChanged)
  stageObserver=new ResizeObserver(([entry]) => { if (entry) stageSize.value={width:entry.contentRect.width,height:entry.contentRect.height} })
  if (stage.value) stageObserver.observe(stage.value)
  // 从卡片点击直接进入：弹层立刻打开并显示加载状态，全屏请求沿用这次点击的用户激活。
  if (props.autostart) open()
})
// Resources and the handshake finish inside the preview. Keep fullscreen in
// this direct click handler so waiting for the network cannot lose activation.
function open() {
  if (!dialog.value) return false
  previousFocus=document.activeElement instanceof HTMLElement ? document.activeElement : null
  previousOverflow=document.documentElement.style.overflow
  document.documentElement.style.overflow='hidden'
  started.value=true
  dialog.value.showModal()
  if (!desktop.value) void enterFullscreen()
  return true
}
function start() { return connected.value && !error.value && open() }
defineExpose({start,restart,readyToPlay:connected,error})
const url = computed(() => game.value && ready.value ? `/builtin-games/${game.value.id}/index.html?hostNonce=${encodeURIComponent(nonce.value)}` : '')
function initialize() { frame.value?.contentWindow?.postMessage({channel:'xiaban.game-host',protocol:1,type:'host.init',nonce:nonce.value,save:null,safeArea:{top:0,right:0,bottom:0,left:0}}, '*') }
function receive(event:MessageEvent) {
  if (!isBuiltinGameMessage(event.data, event.origin, event.source, frame.value?.contentWindow, nonce.value)) return
  const d = event.data
  if (d.type === 'game.ready') { connected.value = true; error.value = ''; clearTimeout(handshakeTimer) }
  if (d.type === 'game.exit') emit('close')
}
window.addEventListener('message',receive)
onBeforeUnmount(() => { disposed=true; document.removeEventListener('fullscreenchange',fullscreenChanged); if (orientationLocked) orientation?.unlock(); if (document.fullscreenElement && surface.value?.contains(document.fullscreenElement)) void document.exitFullscreen().catch(() => {}); stageObserver?.disconnect(); desktopInput.removeEventListener('change',updateInput); if (started.value) document.documentElement.style.overflow=previousOverflow; sequence++; manifestController?.abort(); clearTimeout(handshakeTimer); window.removeEventListener('message',receive); dialog.value?.close(); previousFocus?.focus() })
</script>
<template>
  <dialog ref="dialog" class="player" :aria-hidden="!started" :aria-label="game ? `${game.name}试玩` : '游戏试玩'" @cancel.prevent="emit('close')">
    <div ref="surface" class="player-surface" :class="{mobile:!desktop}">
    <div v-if="started && (loading || !connected || error)" class="player-status"><p v-if="error" role="alert">{{ error }} <button class="btn" type="button" @click="restart">重试</button></p><p v-else role="status">游戏加载中…</p></div>
    <button v-if="!connected || error" class="player-exit btn" type="button" @click="emit('close')">退出试玩</button>
    <button v-if="!desktop && connected && !fullscreen && fullscreenAvailable" class="fullscreen-button" type="button" @click="enterFullscreen">全屏</button>
    <div ref="stage" class="game-stage"><iframe v-if="game && ready" :style="frameStyle" :key="generation" ref="frame" :src="url" :title="`${game.name}游戏区域`" sandbox="allow-scripts allow-orientation-lock" allow="fullscreen" allowfullscreen referrerpolicy="no-referrer" @load="initialize" /></div>
    </div>
  </dialog>
</template>
<style scoped>
.player{position:fixed;inset:0;width:100%;max-width:none;height:100dvh;max-height:none;margin:0;padding:0 0 env(safe-area-inset-bottom);border:0;background:var(--surface);color:var(--ink);overflow:hidden}
.player{display:flex;flex-direction:column;visibility:hidden;pointer-events:none}.player[open]{visibility:visible;pointer-events:auto}.player-surface{position:relative;display:flex;flex-direction:column;flex:1;min-height:0;width:100%;background:var(--surface);color:var(--ink)}.player-surface:fullscreen{width:100%;height:100%;padding:0}.player:has(.mobile){padding-bottom:0}.fullscreen-button,.player-exit{position:absolute;z-index:2;top:max(8px,env(safe-area-inset-top));right:max(8px,env(safe-area-inset-right));min-height:40px;padding:8px 14px;border:1px solid var(--line);border-radius:20px;background:var(--surface);color:var(--ink);font-size:13px}.player-exit{top:auto;bottom:max(8px,env(safe-area-inset-bottom))}.player::backdrop{background:rgba(0,0,0,.5)}
.player-status{position:absolute;inset:0;z-index:1;display:grid;place-items:center;background:var(--surface-2)}.player p{padding:12px;color:var(--ink-2)}
.game-stage{display:flex;align-items:center;justify-content:center;flex:1;min-height:0;min-width:0;overflow:hidden;background:var(--surface-2)}
.player iframe{display:block;flex:none;border:0;background:var(--surface-2)}
</style>
