<script setup lang="ts">
import { ref } from 'vue'
import type { ReplyWorkRef } from '@/api/replyContent'
import type { Attachment } from '@/api/types'
import AttachmentUploader from './AttachmentUploader.vue'
const props = defineProps<{ workRefs: ReplyWorkRef[]; attachments: Attachment[]; maxBytes: number; disabled?: boolean }>()
const emit = defineEmits<{ 'update:workRefs':[ReplyWorkRef[]]; 'update:attachments':[Attachment[]]; uploading:[boolean] }>()
const link = ref(''), version = ref(''), error = ref('')
function add() {
  if (props.disabled) return
  error.value = ''
  try {
    const url = new URL(link.value.trim(), window.location.origin)
    const match = url.pathname.match(/^\/(packs|schemes)\/([1-9]\d*)\/?$/)
    if (url.origin !== window.location.origin || !match) throw new Error('请粘贴本站的作品或方案详情链接')
    const kind = match[1] === 'packs' ? 'pack' : 'scheme', id = Number(match[2])
    if (!Number.isSafeInteger(id)) throw new Error('内容编号无效')
    if (props.workRefs.some(r => r.kind === kind && r.id === id)) throw new Error('已引用此内容')
    if (props.workRefs.length >= 3) throw new Error('最多引用 3 项内容')
    const selectedVersion = version.value ? Number(version.value) : null
    if (selectedVersion !== null && (!Number.isSafeInteger(selectedVersion) || selectedVersion <= 0 || kind !== 'pack')) throw new Error('内容包版本须为正整数，方案无需版本号')
    emit('update:workRefs', [...props.workRefs, { kind, id, version: selectedVersion }]); link.value = ''; version.value = ''
  } catch (e) { error.value = e instanceof Error ? e.message : '链接无效' }
}
function remove(index: number) { emit('update:workRefs', props.workRefs.filter((_, i) => i !== index)) }
</script>
<template>
  <details class="extras"><summary>引用作品、方案或添加附件</summary><div class="extra-fields">
    <p class="hint">公开内容最多引用 3 项。内容包不填版本时固定为发表当时的版本；下架内容自动变为占位。</p>
    <ul v-if="workRefs.length" class="references"><li v-for="(item, i) in workRefs" :key="`${item.kind}:${item.id}`"><span>{{ item.kind === 'pack' ? '内容包' : '方案' }} #{{ item.id }} <template v-if="item.version">· v{{ item.version }}</template></span><button class="btn btn-sm" type="button" :disabled="disabled" @click="remove(i)">移除</button></li></ul>
    <div class="add-reference"><label class="field"><span>本站作品或方案链接</span><input v-model="link" class="input" :disabled="disabled || workRefs.length >= 3" placeholder="/packs/123 或 /schemes/123" @keydown.enter.prevent="add" /></label><label class="field version"><span>版本（可选）</span><input v-model="version" class="input" type="number" min="1" step="1" :disabled="disabled || workRefs.length >= 3" @keydown.enter.prevent="add" /></label><button class="btn" type="button" :disabled="disabled || !link.trim() || workRefs.length >= 3" @click="add">加入引用</button></div>
    <p v-if="error" class="error" role="alert">{{ error }}</p>
    <AttachmentUploader :model-value="attachments" :max-bytes="maxBytes" :disabled="disabled" @update:model-value="emit('update:attachments', $event)" @uploading="emit('uploading', $event)" />
  </div></details>
</template>
<style scoped>
.extras { margin:12px 0; }.extras summary { cursor:pointer; font-size:var(--fs-sm); }.extra-fields { display:grid; gap:12px; padding:12px 0; }.hint { margin:0; color:var(--ink-3); font-size:var(--fs-sm); }.references { margin:0; padding:0; list-style:none; display:grid; gap:6px; }.references li { display:flex; align-items:center; justify-content:space-between; gap:10px; }.add-reference { display:flex; align-items:flex-end; gap:8px; flex-wrap:wrap; }.add-reference .field { flex:1 1 200px; }.add-reference .version { flex:0 1 110px; }.error { color:var(--danger); margin:0; }
</style>
