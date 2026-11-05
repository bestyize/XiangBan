<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { aiApi } from '@/api/ai'
import { UNKNOWN_IMAGE_RESULT, type ImageCapability } from '@/lib/aiImages'
import { readAiImages, storeAiImages, type StoredAiImage } from '@/lib/aiImageStorage'

const props = defineProps<{owner:string; capability?:ImageCapability; getToken:() => Promise<string | null>}>()
const emit = defineEmits<{generated:[]}>()
const prompt = ref(''), format = ref<'png'|'gif'>('png'), size = ref(''), count = ref(1), frames = ref(9), duration = ref(200)
const busy = ref(false), error = ref(''), notice = ref('')
const images = ref<(StoredAiImage & {url:string})[]>([])
const gifAvailable = computed(() => props.capability?.formats?.includes('gif') === true)
const allowed = computed(() => props.capability?.available === true)
let controller: AbortController | null = null, timer: ReturnType<typeof setTimeout> | undefined, disposed = false
function show(items: StoredAiImage[]) {
  for (const item of images.value) URL.revokeObjectURL(item.url)
  images.value = items.slice(0,10).map(item => ({...item, url:URL.createObjectURL(item.blob)}))
}
onMounted(async () => {
  try {const saved = await readAiImages(props.owner); if (!disposed && !images.value.length) show(saved)}
  catch {if (!disposed) notice.value = '浏览器无法保存图片历史，生成后请及时下载。'}
})
onBeforeUnmount(() => {disposed = true; controller?.abort(); clearTimeout(timer); for (const item of images.value) URL.revokeObjectURL(item.url)})
async function generate() {
  if (busy.value || !props.capability || !allowed.value) return
  busy.value = true; error.value = ''; notice.value = ''
  const abort = new AbortController(); controller = abort
  try {
    const token = await props.getToken()
    if (disposed || abort.signal.aborted) return
    if (!token) throw Error('请先登录小伴账号。')
    timer = setTimeout(() => abort.abort(), 240_000)
    const gif = format.value === 'gif'
    const requested = gif ? 1 : count.value
    const description = prompt.value.trim()
    const blobs = await aiApi.images(token, {prompt:description, n:requested, size:size.value || undefined, output_format:format.value, ...(gif ? {frame_count:frames.value, frame_duration_ms:duration.value} : {})}, props.capability, abort.signal)
    if (disposed) return
    const items = blobs.map(blob => ({id:crypto.randomUUID(), owner:props.owner, prompt:description, created:Date.now(), blob}))
    show([...items, ...images.value])
    if (items.length < requested) notice.value = `已收到 ${items.length}/${requested} 张，请核对结果后再生成。`
    try {await storeAiImages(props.owner, items)} catch {if (!disposed) notice.value = '图片已生成，但浏览器未能保存历史，请及时下载。'}
  } catch (cause) {if (!disposed) error.value = abort.signal.aborted ? `已停止等待。${UNKNOWN_IMAGE_RESULT}` : cause instanceof Error ? cause.message : UNKNOWN_IMAGE_RESULT}
  finally {clearTimeout(timer); if (!disposed) {busy.value = false; controller = null; emit('generated')}}
}
function filename(image: StoredAiImage) {return `小伴生图-${image.id}.${({'image/png':'png','image/jpeg':'jpg','image/webp':'webp','image/gif':'gif'} as Record<string,string>)[image.blob.type] || 'png'}`}
</script>

<template>
  <section class="image-generator" aria-label="图片生成">
    <h2>生成图片</h2>
    <p v-if="!allowed" class="hint">{{ capability?.requires_paid ? '图片生成需要付费额度，请在点数面板兑换。' : '当前尚未开放图片生成，请稍后刷新。' }}</p>
    <form @submit.prevent="generate">
      <label for="image-prompt">画面描述</label>
      <textarea id="image-prompt" v-model="prompt" :disabled="busy" rows="3" maxlength="8000" placeholder="描述画面；生成 GIF 时请说明动作和循环…" />
      <div class="image-options">
        <label>格式<select v-model="format" :disabled="busy"><option value="png">静态图片</option><option v-if="gifAvailable" value="gif">GIF 动图</option></select></label>
        <label v-if="capability?.sizes.length">尺寸<select v-model="size" :disabled="busy"><option value="">默认尺寸</option><option v-for="option in capability.sizes" :key="option" :value="option">{{ option }}</option></select></label>
        <label v-if="format === 'png'">数量<input v-model.number="count" type="number" min="1" max="4" :disabled="busy" /></label>
        <template v-else><label>帧数<input v-model.number="frames" type="number" min="2" max="16" :disabled="busy" /></label><label>每帧毫秒<input v-model.number="duration" type="number" min="20" max="2000" step="10" :disabled="busy" /></label></template>
      </div>
      <div class="generate-actions"><span class="hint">使用小伴 AI 额度</span><button v-if="busy" type="button" class="btn" @click="controller?.abort()">停止等待</button><button class="btn btn-primary" :disabled="busy || !allowed || !prompt.trim()">{{ busy ? '正在生成…' : '生成' }}</button></div>
    </form>
    <p v-if="error" role="alert" class="image-error">{{ error }}</p><p v-if="notice" role="status" class="hint">{{ notice }}</p>
    <div v-if="images.length" class="generated-images"><figure v-for="image in images" :key="image.id"><img :src="image.url" :alt="image.prompt" loading="lazy" @error="notice = '图片预览失败，原文件仍可保存；请先核对结果，不要重复生成。'" /><figcaption><span>{{ image.blob.type === 'image/gif' ? 'GIF 动图' : '图片' }}</span><a :href="image.url" :download="filename(image)">保存原图</a></figcaption></figure></div>
    <p v-if="images.length" class="hint">图片仅保存在当前浏览器，最多保留最近 10 张；重要作品请下载保存。</p>
  </section>
</template>

<style scoped>
.image-generator{padding:16px;background:var(--surface);border-radius:16px;margin:16px 0}.image-generator h2{font-size:17px;margin:0 0 12px}.image-generator label{display:flex;flex-direction:column;gap:4px;font-size:13px;color:var(--ink-2)}textarea{display:block;width:100%;resize:vertical;background:var(--surface-2);border:0;border-radius:10px;padding:12px;color:var(--ink);margin-top:6px}.image-options{display:flex;flex-wrap:wrap;gap:12px;margin:12px 0}.image-options select,.image-options input{min-height:40px;max-width:160px;border:0;background:var(--surface-2);border-radius:8px;padding:6px 10px}.image-options input{width:90px}.generate-actions{display:flex;gap:10px;align-items:center;justify-content:flex-end}.generate-actions>span{margin-right:auto}.hint{font-size:12px;color:var(--ink-3)}.image-error{color:var(--danger);font-size:13px}.generated-images{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(240px,100%),1fr));gap:12px;margin-top:16px}.generated-images figure{margin:0;min-width:0;border-radius:12px;overflow:hidden;background:var(--surface-2)}.generated-images img{display:block;width:100%;max-height:400px;object-fit:contain}.generated-images figcaption{display:flex;justify-content:space-between;padding:10px;font-size:12px}.generated-images a{color:var(--brand)}
</style>
