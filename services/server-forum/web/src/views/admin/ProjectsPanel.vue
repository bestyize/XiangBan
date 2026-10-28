<script setup lang="ts">
// 项目：项目页的增改，正文用 Markdown，保存后由后端渲染并净化。
import { onMounted, reactive, ref } from 'vue'
import { adminApi, type ProjectInput } from '@/api/admin'
import { perform } from './shared'

type Row = ProjectInput & { slug: string }
const rows = ref<Row[]>([])
const editing = ref<string | null>(null)
const busy = ref(false)
const form = reactive({ slug: '', name: '', summary: '', body_md: '', links: '', status: 'published' as 'draft' | 'published', sort_order: 0 })

async function load() {
  rows.value = (await adminApi.projects()).items
}

function edit(r?: Row) {
  editing.value = r?.slug ?? ''
  form.slug = r?.slug ?? ''
  form.name = r?.name ?? ''
  form.summary = r?.summary ?? ''
  form.body_md = r?.body_md ?? ''
  form.links = (r?.links ?? []).map(l => `${l.title} ${l.url}`).join('\n')
  form.status = r?.status ?? 'published'
  form.sort_order = r?.sort_order ?? 0
}

async function save() {
  const links = form.links
    .split('\n')
    .map(l => l.trim())
    .filter(Boolean)
    .map(l => {
      const [title, ...rest] = l.split(/\s+/)
      return { title: title ?? '', url: rest.join('') }
    })
  busy.value = true
  const ok = await perform(
    () => adminApi.putProject(form.slug.trim(), { name: form.name.trim(), summary: form.summary.trim(), body_md: form.body_md, links, status: form.status, sort_order: Number(form.sort_order) || 0 }),
    '已保存',
  )
  busy.value = false
  if (ok) {
    editing.value = null
    await load()
  }
}

onMounted(load)
</script>

<template>
  <div class="adm-toolbar">
    <button class="btn btn-primary" @click="edit()">新建项目</button>
  </div>

  <form v-if="editing !== null" class="panel adm-card adm-form" @submit.prevent="save">
    <div class="adm-form-grid">
      <label class="field">
        <span>id</span>
        <input v-model="form.slug" class="input" required maxlength="64" :disabled="!!editing" placeholder="xiaoban" />
        <small>小写字母、数字、连字符、下划线；创建后不可改</small>
      </label>
      <label class="field"><span>名称</span><input v-model="form.name" class="input" required maxlength="60" /></label>
      <label class="field wide"><span>简介</span><input v-model="form.summary" class="input" maxlength="300" /></label>
      <label class="field">
        <span>状态</span>
        <select v-model="form.status" class="select" style="width: 100%">
          <option value="published">发布</option>
          <option value="draft">草稿</option>
        </select>
      </label>
      <label class="field"><span>排序值</span><input v-model.number="form.sort_order" class="input" type="number" /></label>
      <label class="field wide"><span>正文（Markdown）</span><textarea v-model="form.body_md" class="input" rows="10" /></label>
      <label class="field wide"><span>链接（每行：标题 地址）</span><textarea v-model="form.links" class="input" rows="3" spellcheck="false" /></label>
    </div>
    <div class="adm-form-actions">
      <button class="btn btn-primary" type="submit" :disabled="busy">保存</button>
      <button class="btn btn-ghost" type="button" @click="editing = null">取消</button>
    </div>
  </form>

  <div class="panel adm-card">
    <p v-if="!rows.length" class="adm-empty">还没有项目</p>
    <table v-else class="adm-table">
      <thead><tr><th>名称</th><th>id</th><th>状态</th><th>排序</th><th>操作</th></tr></thead>
      <tbody>
        <tr v-for="r in rows" :key="r.slug">
          <td data-label="名称">{{ r.name }}</td>
          <td data-label="id" class="adm-mono">{{ r.slug }}</td>
          <td data-label="状态">{{ r.status === 'published' ? '已发布' : '草稿' }}</td>
          <td data-label="排序">{{ r.sort_order }}</td>
          <td data-label="操作" class="adm-actions"><button class="btn btn-sm" @click="edit(r)">编辑</button></td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
