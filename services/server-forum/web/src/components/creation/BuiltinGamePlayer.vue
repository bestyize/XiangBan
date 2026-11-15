<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import AppRuntimeStatus from '@/components/creation/AppRuntimeStatus.vue'
import { builtinGame, isBuiltinGameMessage } from '@/lib/builtinCreation'
import { fitGameViewport } from '@/lib/gameViewport'
import { checkedMbtiResultLink, decodeMbtiPng } from '@/lib/mbtiExport'
const props = defineProps<{ gameId: string }>()
const emit = defineEmits<{ close: [] }>()
const game = computed(() => builtinGame(props.gameId)), frame = ref<HTMLIFrameElement | null>(null), nonce = ref(''), generation = ref(0), error = ref(''), ready = ref(false), connected = ref(false)
const message = ref('正在读取作品信息…')
const stage=ref<HTMLElement>(), stageSize=ref({width:0,height:0})
const desktopInput=window.matchMedia('(hover: hover) and (pointer: fine)'), desktop=ref(desktopInput.matches)
const frameSize=computed(() => fitGameViewport(stageSize.value.width,stageSize.value.height,game.value?.orientation || 'landscape',desktop.value && props.gameId !== 'mbti'))
const frameStyle=computed(() => stageSize.value.width && stageSize.value.height ? {width:`${frameSize.value.width}px`,height:`${frameSize.value.height}px`} : {width:'100%',height:'100%'})
const exportRequest=ref<{ action:'save'|'image'|'link'; file?:File; url?:string } | null>(null)
const exportError=ref('')
let exportSequence=0
let imageAssets = new Map<string, Promise<Blob>>()
async function provideMbtiAsset(d:Record<string,unknown>, target:Window) {
  if(props.gameId!=='mbti'||!connected.value||!Number.isSafeInteger(d.id)||
    (d.key!=='characters'&&d.key!=='observer'&&d.key!=='qr'))return
  const key=d.key, activeNonce=nonce.value
  let asset=imageAssets.get(key)
  if(!asset){
    const file=key==='characters'?'characters.png':key==='observer'?'observer-jade.png':'test-qr.png'
    asset=fetch(`/builtin-games/mbti/assets/${file}`,{credentials:'omit',redirect:'error'}).then(async response=>{
      if(!response.ok)throw new Error('素材不可用')
      const blob=await response.blob();if(blob.type!=='image/png'||blob.size>3*1024*1024)throw new Error('素材无效')
      return blob
    })
    imageAssets.set(key,asset)
    asset.catch(()=>imageAssets.delete(key))
  }
  try{const blob=await asset;if(!disposed&&nonce.value===activeNonce&&frame.value?.contentWindow===target)target.postMessage({channel:'xiaban.game-host',protocol:1,nonce:activeNonce,type:'mbti.asset',id:d.id,blob},'*')}catch{/* game shows a retry message */}
}
async function prepareExport(d:Record<string,unknown>) {
  if(props.gameId!=='mbti'||exportRequest.value)return
  const current=++exportSequence; exportError.value=''
  if(d.action==='link') {
    const url=checkedMbtiResultLink(d.url);if(url)exportRequest.value={action:'link',url}
    return
  }
  if(d.action!=='save'&&d.action!=='image')return
  try {
    const bytes=decodeMbtiPng(d.image);if(!bytes)return
    const file=new File([bytes],'小伴人格画像.png',{type:'image/png'})
    const valid=await validateImageDimensions(file)
    if(valid&&current===exportSequence)exportRequest.value={action:d.action,file}
  }catch{if(current===exportSequence)exportError.value='图片准备失败，请重新打开分享页。'}
}
async function validateImageDimensions(file:File):Promise<boolean>{
  if(typeof createImageBitmap==='function'){const bitmap=await createImageBitmap(file);const valid=bitmap.width===1080&&bitmap.height===1440;bitmap.close();return valid}
  const url=URL.createObjectURL(file)
  try{const image=new Image();image.src=url;await image.decode();return image.naturalWidth===1080&&image.naturalHeight===1440}finally{URL.revokeObjectURL(url)}
}
function closeExport(){exportSequence++;exportRequest.value=null;exportError.value=''}
function downloadExport(){const file=exportRequest.value?.file;if(!file)return;const url=URL.createObjectURL(file),a=document.createElement('a');a.href=url;a.download=file.name;document.body.append(a);a.click();a.remove();setTimeout(()=>URL.revokeObjectURL(url),30000);closeExport()}
async function shareExport(){
  const request=exportRequest.value;if(!request)return
  try {
    const payload:ShareData=request.action==='link'?{title:'我的小伴人格画像',url:request.url}:{title:'我的小伴人格画像',files:[request.file!]}
    if(!navigator.share || (request.action!=='link' && (!navigator.canShare || !navigator.canShare(payload)))){exportError.value=request.action==='link'?'当前浏览器无法直接分享，请复制链接。':'当前浏览器无法直接分享图片，请先保存图片。';return}
    await navigator.share(payload);closeExport()
  }catch(e){if(!(e instanceof DOMException&&e.name==='AbortError'))exportError.value='分享未完成，请重试。'}
}
async function copyResultLink(){if(!exportRequest.value?.url)return;try{await navigator.clipboard.writeText(exportRequest.value.url);closeExport()}catch{exportError.value='无法复制，请手动复制链接。'}}
let stageObserver:ResizeObserver | undefined
let disposed=false
function updateInput() { desktop.value=desktopInput.matches }
let sequence = 0
let manifestController:AbortController | undefined
async function restart() {
  closeExport(); imageAssets = new Map()
  manifestController?.abort()
  const controller=new AbortController()
  manifestController=controller
  const current = ++sequence
  ready.value = false
  connected.value = false
  error.value = ''
  message.value = '正在读取作品信息…'
  try {
    if (!game.value) throw new Error('找不到这个内置游戏')
    const response = await fetch(`/builtin-games/${game.value.id}/manifest.json`, {credentials:'omit',signal:controller.signal})
    if (!response.ok) throw new Error('游戏资源不可用')
    const manifest = await response.json()
    if (manifest.id !== game.value.id || manifest.entry !== 'index.html' || (manifest.runtimeProtocol !== 1 && manifest.runtimeProtocol !== 2)) throw new Error('游戏资源不完整')
    if (current !== sequence) return
    nonce.value = crypto.randomUUID(); generation.value++; ready.value = true
    message.value = '正在加载作品资源…'
  } catch (e) {
    if (current === sequence) error.value = e instanceof TypeError ? '游戏资源请求失败，请检查网络。' : e instanceof Error ? e.message : '游戏加载失败'
  }
}
watch(() => props.gameId, restart, { immediate:true })
onMounted(() => {
  desktopInput.addEventListener('change',updateInput)
  stageObserver=new ResizeObserver(([entry]) => { if (entry) stageSize.value={width:entry.contentRect.width,height:entry.contentRect.height} })
  if (stage.value) stageObserver.observe(stage.value)
})
const url = computed(() => game.value && ready.value ? `/builtin-games/${game.value.id}/index.html?hostNonce=${encodeURIComponent(nonce.value)}` : '')
function initialize() {
  if (!connected.value) message.value = '正在启动作品…'
  frame.value?.contentWindow?.postMessage({channel:'xiaban.game-host',protocol:1,type:'host.init',nonce:nonce.value,save:null,safeArea:{top:0,right:0,bottom:0,left:0}}, '*')
}
function receive(event:MessageEvent) {
  if (!isBuiltinGameMessage(event.data, event.origin, event.source, frame.value?.contentWindow, nonce.value)) return
  const d = event.data
  if (d.type === 'game.ready') { connected.value = true; error.value = '' }
  if (d.type === 'game.exit') emit('close')
  if (d.type === 'mbti.asset' && props.gameId === 'mbti' && frame.value?.contentWindow) void provideMbtiAsset(d,frame.value.contentWindow)
  if (d.type === 'mbti.export' && props.gameId === 'mbti') void prepareExport(d)
}
window.addEventListener('message',receive)
onBeforeUnmount(() => {
  closeExport()
  disposed = true
  stageObserver?.disconnect()
  desktopInput.removeEventListener('change', updateInput)
  sequence++
  manifestController?.abort()
  window.removeEventListener('message', receive)
})
</script>
<template>
  <section class="player" :aria-label="game?.name || '游戏试玩'">
    <AppRuntimeStatus v-if="!connected || error" class="player-status" :message="message" :error="error" @retry="restart" />
    <div ref="stage" class="game-stage"><iframe v-if="game && ready" :style="frameStyle" :key="generation" ref="frame" :src="url" :title="`${game.name}游戏区域`" sandbox="allow-scripts allow-orientation-lock" allow="fullscreen" allowfullscreen referrerpolicy="no-referrer" @load="initialize" /></div>
    <div v-if="exportRequest" class="export-overlay" role="dialog" aria-modal="true" aria-label="分享人格画像">
      <div class="export-panel"><h2>{{ exportRequest.action === 'link' ? '分享结果链接' : '分享人格海报' }}</h2>
        <p>{{ exportRequest.action === 'link' ? '朋友打开链接后可看到你选择分享的结果。' : '保存或分享这张图片，二维码可打开测评。' }}</p>
        <input v-if="exportRequest.url" :value="exportRequest.url" readonly aria-label="结果链接" @focus="($event.target as HTMLInputElement).select()">
        <p v-if="exportError" role="alert">{{ exportError }}</p>
        <div class="export-actions"><button v-if="exportRequest.file" class="btn" type="button" @click="downloadExport">保存图片</button><button v-if="exportRequest.action !== 'save'" class="btn btn-primary" type="button" @click="shareExport">打开分享面板</button><button v-if="exportRequest.url" class="btn" type="button" @click="copyResultLink">复制链接</button><button class="btn" type="button" @click="closeExport">取消</button></div>
      </div>
    </div>
  </section>
</template>
<style scoped>
.player{position:relative;display:flex;flex-direction:column;width:100%;height:100%;overflow:hidden;background:var(--surface);color:var(--ink)}
.export-overlay{position:absolute;inset:0;z-index:9;display:grid;place-items:center;padding:20px;background:#0007}.export-panel{width:min(100%,440px);padding:20px;border-radius:16px;background:var(--surface);box-shadow:0 8px 32px #0003}.export-panel h2{font-size:18px;margin:0}.export-panel p{padding:10px 0;font-size:13px;line-height:1.7}.export-panel input{width:100%;padding:10px;border:1px solid var(--line);border-radius:8px;background:var(--surface-2);color:var(--ink);font-size:12px}.export-actions{display:flex;flex-wrap:wrap;gap:8px}.export-actions button{min-height:44px}
.player p{padding:12px;color:var(--ink-2)}
.game-stage{display:flex;align-items:center;justify-content:center;flex:1;min-height:0;min-width:0;overflow:hidden;background:var(--surface-2)}
.player iframe{display:block;flex:none;border:0;background:var(--surface-2)}
</style>
