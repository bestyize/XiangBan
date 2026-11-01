<script setup lang="ts">
import { ref, watch } from 'vue'
import ModalDialog from '@/components/ModalDialog.vue'
const props = defineProps<{ show: boolean; prompt?: string; kind?: string }>()
const emit = defineEmits<{ 'update:show': [value: boolean] }>()
const idea = ref(''), copied = ref('')
watch(() => props.show, open => { if (open) { if (props.prompt) idea.value = props.prompt; copied.value = '' } })
async function copy() {
  try { await navigator.clipboard.writeText(idea.value); copied.value = '想法已复制' }
  catch { copied.value = '复制失败，请选中文字手动复制' }
}
</script>
<template>
  <ModalDialog :open="show" title="在小伴客户端创作" @close="emit('update:show', false)">
    <div class="creation-prompt">
      <p>网页可以浏览和体验作品。创作{{ kind ? `或修改${kind}` : '' }}请使用客户端。</p>
      <label for="creation-idea">你的想法（可复制到客户端）</label>
      <textarea id="creation-idea" v-model="idea" class="input" rows="4" placeholder="写下你的创作想法" />
      <button v-if="idea" class="btn" type="button" @click="copy">复制想法</button><span role="status">{{ copied }}</span>
      <p>Windows / macOS 支持桌面创作；Android 目前支持游戏创作。iOS 暂不支持本地创作。</p>
      <a class="btn btn-primary" href="https://wallpaper.xiangban.xyz/" target="_blank" rel="noopener noreferrer">前往小伴官网下载</a>
    </div>
  </ModalDialog>
</template>
<style scoped>
.creation-prompt { display:grid; gap:12px; color:var(--ink-2); }.creation-prompt p { margin:0; }.creation-prompt textarea { width:100%; resize:vertical; }.creation-prompt .btn { min-height:44px; justify-self:start; }.creation-prompt span { font-size:var(--fs-sm); }
</style>
