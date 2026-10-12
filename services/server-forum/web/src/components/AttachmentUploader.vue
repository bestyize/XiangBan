<script setup lang="ts">
import { ref } from 'vue'
import { deleteAttachment, uploadAttachment } from '@/api/forum'
import type { Attachment } from '@/api/types'
import { formatSize } from '@/lib/format'
import { useToast } from '@/stores/toast'

const MAX_FILES = 5

const props = defineProps<{ modelValue: Attachment[]; maxBytes: number }>()
const emit = defineEmits<{ 'update:modelValue': [Attachment[]] }>()

const toast = useToast()
const input = ref<HTMLInputElement | null>(null)
const uploading = ref<{ key: number; name: string; ratio: number }[]>([])
/** 这次编辑里新传的附件。删掉时直接撤销上传；原来就挂在帖子上的，等保存时由服务端移除。 */
const fresh = new Set<string>()
let seq = 0

async function onPick(e: Event) {
  const el = e.target as HTMLInputElement
  const files = Array.from(el.files ?? [])
  el.value = ''
  let list = [...props.modelValue]
  for (const file of files) {
    if (list.length + uploading.value.length >= MAX_FILES) {
      toast.show(`一个帖子最多 ${MAX_FILES} 个附件`, 'error')
      break
    }
    if (file.size === 0) {
      toast.show(`「${file.name}」是空文件`, 'error')
      continue
    }
    if (file.size > props.maxBytes) {
      toast.show(`「${file.name}」有 ${formatSize(file.size)}，超过了你的上限 ${formatSize(props.maxBytes)}`, 'error')
      continue
    }
    const item = { key: ++seq, name: file.name, ratio: 0 }
    uploading.value.push(item)
    try {
      const att = await uploadAttachment(file, r => (item.ratio = r))
      fresh.add(att.id)
      list = [...list, att]
      emit('update:modelValue', list)
    } catch (err) {
      toast.error(err)
    } finally {
      uploading.value = uploading.value.filter(u => u.key !== item.key)
    }
  }
}

function remove(att: Attachment) {
  emit('update:modelValue', props.modelValue.filter(a => a.id !== att.id))
  if (fresh.delete(att.id)) deleteAttachment(att.id).catch(() => {})
}
</script>

<template>
  <div class="uploader">
    <ul v-if="modelValue.length || uploading.length" class="files">
      <li v-for="a in modelValue" :key="a.id" class="file">
        <span class="name" :title="a.name">{{ a.name }}</span>
        <span class="size">{{ formatSize(a.size) }}</span>
        <button type="button" class="rm" :aria-label="`移除附件 ${a.name}`" @click="remove(a)">×</button>
      </li>
      <li v-for="u in uploading" :key="u.key" class="file busy">
        <span class="name" :title="u.name">{{ u.name }}</span>
        <span class="size">{{ Math.round(u.ratio * 100) }}%</span>
        <span class="bar" :style="{ width: `${u.ratio * 100}%` }" aria-hidden="true" />
      </li>
    </ul>
    <div class="row">
      <input ref="input" class="picker" type="file" multiple tabindex="-1" aria-hidden="true" @change="onPick" />
      <button
        type="button"
        class="btn btn-sm"
        :disabled="modelValue.length + uploading.length >= MAX_FILES"
        @click="input?.click()"
      >
        添加附件
      </button>
      <small>最多 {{ MAX_FILES }} 个，每个不超过 {{ formatSize(maxBytes) }}。</small>
    </div>
  </div>
</template>

<style scoped>
.uploader {
  display: grid;
  gap: 8px;
}
.picker {
  display: none;
}
.row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 10px;
}
.files {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 6px;
}
.file {
  position: relative;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 8px 6px 12px;
  border: 1px solid var(--line);
  border-radius: var(--r-sm);
  background: var(--surface-2);
  font-size: var(--fs-sm);
  overflow: hidden;
}
.name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.size {
  color: var(--ink-3);
  font-size: var(--fs-xs);
  font-variant-numeric: tabular-nums;
}
.rm {
  border: 0;
  background: none;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  color: var(--ink-3);
  line-height: 1;
}
.rm:hover {
  background: var(--line);
}
.busy .bar {
  position: absolute;
  left: 0;
  bottom: 0;
  height: 2px;
  background: var(--focus);
  transition: width 0.15s;
}
</style>
