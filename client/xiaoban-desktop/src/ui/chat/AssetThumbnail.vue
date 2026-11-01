<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { Image, ImageOff } from 'lucide-vue-next';
import { invoke } from '../../bridge.ts';
import type { ConversationAsset } from './assets.ts';
const props = defineProps<{ asset: ConversationAsset; session: string }>();
const host = ref<HTMLElement>();
const src = ref(''), failed = ref(false);
let observer: IntersectionObserver | undefined, epoch = 0, visible = false;
async function load() {
  const version = ++epoch; src.value = ''; failed.value = false;
  try {
    const asset = props.asset;
    const data = asset.generated
      ? await invoke<string>('chat_generated_image_read', { id: asset.id })
      : await invoke<string>('chat_asset_thumbnail', { sessionId: props.session, id: asset.id, relative: asset.relative ?? '' });
    if (version !== epoch) return;
    if (!/^data:image\/png;base64,/.test(data)) throw Error('无效缩略图');
    src.value = data;
  } catch { if (version === epoch) failed.value = true; }
}
watch(() => [props.asset.key, props.asset.updatedMs, props.session].join('|'), () => { if (visible) void load(); });
onMounted(() => {
  if (typeof IntersectionObserver === 'undefined') { visible = true; void load(); return; }
  observer = new IntersectionObserver(entries => { if (entries.some(e => e.isIntersecting)) { visible = true; observer?.disconnect(); void load(); } });
  if (host.value) observer.observe(host.value);
});
onBeforeUnmount(() => { epoch++; observer?.disconnect(); });
</script>
<template><span ref="host" class="asset-thumbnail"><img v-if="src" :src="src" :alt="asset.name" loading="lazy" @error="src=''; failed=true" /><ImageOff v-else-if="failed" :size="20" aria-label="缩略图不可用，仍可打开原文件" /><Image v-else :size="20" aria-hidden="true" /></span></template>
<style scoped>
.asset-thumbnail{display:grid;place-items:center;width:100%;aspect-ratio:1.2;overflow:hidden;border-radius:7px;background:var(--ui-control);color:var(--ui-text-tertiary)}img{display:block;width:100%;height:100%;object-fit:cover}
</style>
