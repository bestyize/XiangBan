<script setup lang="ts">
// 分享海报：弹窗里生成一张图，可保存或复制链接。手机上长按图片也能保存。
import { ref, watch } from 'vue'
import ModalDialog from './ModalDialog.vue'
import { canvasToBlob, renderPoster, type PosterInput } from '@/lib/poster'
import { track } from '@/lib/analytics'
import { useToast } from '@/stores/toast'

const props = defineProps<{ open: boolean; poster: PosterInput | null; name: string }>()
const emit = defineEmits<{ close: [] }>()
const toast = useToast()
const src = ref('')
const busy = ref(false)
let objectUrl = ''

function release() {
  if (objectUrl) URL.revokeObjectURL(objectUrl)
  objectUrl = ''
  src.value = ''
}

watch(
  () => [props.open, props.poster] as const,
  async ([open, poster]) => {
    release()
    if (!open || !poster) return
    busy.value = true
    try {
      const blob = await canvasToBlob(await renderPoster(poster))
      objectUrl = URL.createObjectURL(blob)
      src.value = objectUrl
      track('share_poster')
    } catch {
      toast.show('海报生成失败，请稍后重试')
    } finally {
      busy.value = false
    }
  },
  { immediate: true },
)

async function copyLink() {
  try {
    await navigator.clipboard.writeText(props.poster?.url ?? window.location.href)
    toast.show('链接已复制')
  } catch {
    toast.show('复制失败，请手动复制地址栏')
  }
}
</script>

<template>
  <ModalDialog :open="open" title="分享海报" :width="420" @close="emit('close')">
    <p v-if="busy" class="state">生成中…</p>
    <img v-else-if="src" :src="src" alt="分享海报" class="poster" />
    <div class="actions">
      <a v-if="src" class="btn btn-primary" :href="src" :download="`${name || '海报'}.png`">保存图片</a>
      <button class="btn" type="button" @click="copyLink">复制链接</button>
    </div>
    <p class="hint">手机上也可以长按图片保存。</p>
  </ModalDialog>
</template>

<style scoped>
.poster {
  display: block;
  width: 100%;
  border-radius: 8px;
  border: 1px solid var(--line);
}
.state {
  padding: 60px 0;
  text-align: center;
  color: var(--ink-3);
}
.actions {
  display: flex;
  gap: 8px;
  margin-top: 12px;
}
.hint {
  margin: 8px 0 0;
  font-size: var(--fs-xs);
  color: var(--ink-3);
}
</style>
