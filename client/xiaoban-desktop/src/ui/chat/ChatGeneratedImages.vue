<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { Download, ExternalLink, ImageOff, LoaderCircle } from 'lucide-vue-next';
import { invoke } from '../../bridge.ts';
import type { ToolCall } from './replies.ts';
import type { GeneratedImage } from './generated-images.ts';

const props = defineProps<{ calls: ToolCall[]; awaitingApproval?: boolean }>();
const calls = computed(() => props.calls.filter(call => call.name === 'image_generate'));
const images = computed(() => [...new Map(calls.value.flatMap(call => call.generatedImages ?? []).map(image => [image.id, image])).values()]);
const previews = ref<Record<string, string>>({});
const errors = ref<Record<string, string>>({});
const actionErrors = ref<Record<string, string>>({});
const busy = ref<Record<string, boolean>>({});
const saved = ref<Record<string, boolean>>({});
const loading = new Set<string>();
let disposed = false;
onBeforeUnmount(() => { disposed = true; });

async function load(image: GeneratedImage) {
  if (loading.has(image.id) || previews.value[image.id]) return;
  loading.add(image.id); delete errors.value[image.id];
  try {
    const data = await invoke<string>('chat_generated_image_read', { id: image.id });
    if (!/^data:image\/png;base64,/.test(data)) throw Error('图片预览无法识别');
    if (!disposed) previews.value[image.id] = data;
  } catch (error) { if (!disposed) errors.value[image.id] = String(error); }
  finally { loading.delete(image.id); }
}
watch(() => images.value.map(image => image.id).join(','), () => { for (const image of images.value) void load(image); }, { immediate: true });
function previewFailed(id: string) { delete previews.value[id]; errors.value[id] = '图片预览加载失败'; }
async function action(image: GeneratedImage, save: boolean) {
  if (busy.value[image.id]) return;
  busy.value[image.id] = true; delete actionErrors.value[image.id];
  try {
    const result = await invoke<boolean | void>(save ? 'chat_generated_image_save' : 'chat_generated_image_open', { id: image.id });
    if (!disposed && save && result === true) saved.value[image.id] = true;
  } catch (error) { if (!disposed) actionErrors.value[image.id] = String(error); }
  finally { if (!disposed) busy.value[image.id] = false; }
}
</script>

<template>
  <section v-if="calls.length" class="generated" data-testid="chat-generated-images" aria-label="生成的图片">
    <p v-for="call in calls.filter(item => item.status !== 'done')" :key="call.id" class="generation-state" :class="{ failed: call.status === 'error' }" :role="call.status === 'error' ? 'alert' : 'status'">
      <LoaderCircle v-if="call.status === 'running'" :size="15" class="spin" />
      <span>{{ call.status === 'running' ? awaitingApproval ? '等待生图确认…' : '正在生成图片…' : !call.output || ['已中断', '没有执行完就中断了'].includes(call.output) ? '图片生成已中断；结果可能已生成并计费，请勿自动重试。' : call.output.split('\n{')[0] }}</span>
    </p>
    <p v-for="call in calls.filter(item => item.status === 'done' && item.output.includes('部分'))" :key="`${call.id}-warning`" class="generation-state" role="status">部分图片生成、保存或入库未完成，请检查执行结果，避免重复生成。</p>
    <div v-if="images.length" class="image-grid">
      <figure v-for="image in images" :key="image.id" class="image-card" data-testid="generated-image-card">
        <button v-if="previews[image.id]" type="button" class="image-preview" :aria-label="`查看原图：${image.name}`" title="在系统图片查看器中查看原图" :disabled="busy[image.id]" @click="action(image, false)">
          <img :src="previews[image.id]" :alt="image.name" :width="image.width" :height="image.height" @error="previewFailed(image.id)" />
        </button>
        <div v-else class="image-placeholder"><ImageOff v-if="errors[image.id]" :size="24" /><LoaderCircle v-else :size="24" class="spin" /></div>
        <figcaption>
          <span class="image-size">{{ image.width }} × {{ image.height }}</span>
          <button type="button" class="image-action" :disabled="busy[image.id]" title="在系统图片查看器中查看原图" aria-description="在系统图片查看器中查看原图" @click="action(image, false)"><ExternalLink :size="13" />原图</button>
          <button type="button" class="image-action" :disabled="busy[image.id]" @click="action(image, true)"><Download :size="13" />{{ saved[image.id] ? '已保存' : '另存为' }}</button>
        </figcaption>
        <p v-if="errors[image.id] || actionErrors[image.id]" class="image-error" role="alert">{{ actionErrors[image.id] || errors[image.id] }}<button v-if="errors[image.id]" type="button" class="image-action" @click="load(image)">重新读取</button></p>
      </figure>
    </div>
  </section>
</template>

<style scoped>
.generated{margin:10px 0;min-width:0}.generation-state{display:flex;align-items:center;gap:8px;margin:8px 0;color:var(--ui-text-secondary);font-size:13px;overflow-wrap:anywhere}.generation-state.failed,.image-error{color:var(--ui-danger)}
.image-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(240px,100%),1fr));gap:12px;max-width:720px}.image-card{min-width:0;margin:0;border-radius:12px;overflow:hidden;background:var(--ui-control)}
.image-preview{display:block;width:100%;padding:0;border:0;background:transparent;cursor:zoom-in}.image-preview img{display:block;width:100%;height:auto;max-height:400px;object-fit:contain}.image-placeholder{display:grid;place-items:center;min-height:160px;color:var(--ui-text-tertiary)}
figcaption{display:flex;align-items:center;gap:8px;padding:8px 10px}.image-size{margin-right:auto;font-size:11px;color:var(--ui-text-tertiary);font-variant-numeric:tabular-nums}.image-action{display:inline-flex;align-items:center;gap:4px;padding:4px 5px;border:0;border-radius:5px;color:var(--ui-text-secondary);background:transparent;cursor:pointer;white-space:nowrap;font:inherit;font-size:12px}.image-action:hover{background:var(--ui-soft);color:var(--ui-ink)}button:focus-visible{outline:2px solid var(--ui-primary);outline-offset:-2px}button:disabled{opacity:.5;cursor:default}.image-error{margin:0;padding:0 10px 8px;font-size:12px;overflow-wrap:anywhere}.spin{animation:image-spin 1s linear infinite}@keyframes image-spin{to{transform:rotate(360deg)}}@media(prefers-reduced-motion:reduce){.spin{animation:none}}
</style>
