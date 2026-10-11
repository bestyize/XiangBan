<script setup lang="ts">
import { ref } from 'vue'
import { api } from '@/api/forum'

const model = defineModel<string>({ required: true })
const props = defineProps<{ placeholder?: string; rows?: number; label: string; id: string }>()

const mode = ref<'write' | 'preview'>('write')
const html = ref('')
const loading = ref(false)
const textarea = ref<HTMLTextAreaElement | null>(null)

async function showPreview() {
  mode.value = 'preview'
  if (!model.value.trim()) {
    html.value = ''
    return
  }
  loading.value = true
  try {
    html.value = (await api.preview(model.value)).html
  } catch (e) {
    html.value = `<p>${e instanceof Error ? e.message : '预览失败'}</p>`
  } finally {
    loading.value = false
  }
}

/** 在光标处包一段 Markdown 语法。 */
function wrap(before: string, after = '', fallback = '') {
  const el = textarea.value
  if (!el) return
  const { selectionStart: s, selectionEnd: e } = el
  const picked = model.value.slice(s, e) || fallback
  model.value = model.value.slice(0, s) + before + picked + after + model.value.slice(e)
  requestAnimationFrame(() => {
    el.focus()
    el.setSelectionRange(s + before.length, s + before.length + picked.length)
  })
}

function focus() {
  mode.value = 'write'
  requestAnimationFrame(() => textarea.value?.focus())
}

defineExpose({ focus })
</script>

<template>
  <div class="editor">
    <div class="bar">
      <div class="modes" role="tablist" :aria-label="props.label">
        <button type="button" role="tab" :aria-selected="mode === 'write'" :class="{ on: mode === 'write' }" @click="mode = 'write'">
          编辑
        </button>
        <button type="button" role="tab" :aria-selected="mode === 'preview'" :class="{ on: mode === 'preview' }" @click="showPreview">
          预览
        </button>
      </div>
      <div v-if="mode === 'write'" class="tools">
        <button type="button" title="加粗" @click="wrap('**', '**', '加粗')"><b>B</b></button>
        <button type="button" title="链接" @click="wrap('[', '](https://)', '链接文字')">链接</button>
        <button type="button" title="行内代码" @click="wrap('`', '`', 'code')">代码</button>
        <button type="button" title="代码块" @click="wrap('\n```bash\n', '\n```\n', '')">代码块</button>
        <button type="button" title="列表" @click="wrap('\n- ', '', '')">列表</button>
      </div>
    </div>
    <label :for="id" class="sr-only">{{ label }}</label>
    <textarea
      v-show="mode === 'write'"
      :id="id"
      ref="textarea"
      v-model="model"
      class="textarea"
      :rows="rows ?? 8"
      :placeholder="placeholder"
    />
    <div v-if="mode === 'preview'" class="preview prose" :aria-busy="loading">
      <p v-if="loading" class="muted">正在生成预览</p>
      <p v-else-if="!html" class="muted">还没有内容。</p>
      <div v-else v-html="html" />
    </div>
    <p class="hint">支持 Markdown。新人发的链接和图片会显示为纯文字，升级到成员后恢复。</p>
  </div>
</template>

<style scoped>
.editor {
  border: 1px solid var(--line-strong);
  border-radius: var(--r-sm);
  background: var(--surface);
  overflow: hidden;
}
.editor:focus-within {
  border-color: var(--focus);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--focus) 15%, transparent);
}
.bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
  padding: 4px 6px;
  border-bottom: 1px solid var(--line);
  background: var(--surface-2);
  flex-wrap: wrap;
}
.modes,
.tools {
  display: flex;
  gap: 2px;
}
.modes button,
.tools button {
  border: 0;
  background: none;
  padding: 3px 9px;
  border-radius: 5px;
  font-size: var(--fs-xs);
  font-weight: 600;
  color: var(--ink-3);
}
.modes button.on {
  background: var(--surface);
  color: var(--ink);
  box-shadow: 0 0 0 1px var(--line);
}
.tools button:hover {
  background: var(--surface);
  color: var(--ink);
}
.textarea {
  border: 0;
  border-radius: 0;
  display: block;
  min-height: 140px;
}
.textarea:focus {
  box-shadow: none;
}
.preview {
  padding: 12px 14px;
  min-height: 140px;
}
.hint {
  margin: 0;
  padding: 6px 12px;
  font-size: 11.5px;
  color: var(--ink-3);
  border-top: 1px solid var(--line);
  background: var(--surface-2);
}
</style>
