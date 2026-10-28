<script setup lang="ts">
// 周刊：每期标题、导语和条目（引用内容包或帖子，或一段文字）。草稿不对外，发布后才出现在前台。
import { onMounted, reactive, ref } from 'vue'
import { adminApi, type WeeklyInput } from '@/api/admin'
import { isoWeek } from '@/lib/content'
import { perform } from './shared'

type Row = WeeklyInput & { week: string }
type Item = WeeklyInput['items'][number]
const rows = ref<Row[]>([])
const editing = ref<string | null>(null)
const busy = ref(false)
const form = reactive({ week: '', title: '', summary: '', status: 'draft' as 'draft' | 'published', items: [] as Item[] })

async function load() {
  rows.value = (await adminApi.weeklyIssues()).items
}

function edit(r?: Row) {
  editing.value = r?.week ?? ''
  form.week = r?.week ?? isoWeek(new Date())
  form.title = r?.title ?? ''
  form.summary = r?.summary ?? ''
  form.status = r?.status ?? 'draft'
  form.items = (r?.items ?? []).map(i => ({ ...i }))
}

function addItem(type: Item['type']) {
  form.items.push({ type, note: '', ...(type === 'text' ? { title: '' } : { ref: undefined }) })
}

async function save() {
  for (const it of form.items) {
    if (it.type !== 'text' && !Number.isInteger(it.ref)) {
      window.alert('内容包和帖子条目需要填写编号')
      return
    }
  }
  busy.value = true
  const ok = await perform(
    () => adminApi.putWeekly(form.week.trim(), { title: form.title.trim(), summary: form.summary.trim(), status: form.status, items: form.items }),
    form.status === 'published' ? '已发布' : '已保存草稿',
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
    <button class="btn btn-primary" @click="edit()">写新一期</button>
  </div>

  <form v-if="editing !== null" class="panel adm-card adm-form" @submit.prevent="save">
    <div class="adm-form-grid">
      <label class="field">
        <span>周标识</span>
        <input v-model="form.week" class="input" required pattern="\d{4}-W\d{2}" :disabled="!!editing" />
        <small>形如 2026-W44，默认本周</small>
      </label>
      <label class="field">
        <span>状态</span>
        <select v-model="form.status" class="select" style="width: 100%">
          <option value="draft">草稿</option>
          <option value="published">发布</option>
        </select>
      </label>
      <label class="field wide"><span>标题</span><input v-model="form.title" class="input" required maxlength="60" /></label>
      <label class="field wide"><span>导语</span><textarea v-model="form.summary" class="input" rows="3" maxlength="500" /></label>
    </div>

    <h3 class="adm-section-title">条目（{{ form.items.length }}/30）</h3>
    <div v-for="(it, i) in form.items" :key="i" class="adm-form-grid" style="margin-bottom: 10px">
      <label class="field">
        <span>{{ it.type === 'pack' ? '内容包编号' : it.type === 'topic' ? '帖子编号' : '小标题' }}</span>
        <input v-if="it.type !== 'text'" v-model.number="it.ref" class="input" type="number" min="1" required />
        <input v-else v-model="it.title" class="input" maxlength="60" />
      </label>
      <label class="field"><span>推荐语</span><input v-model="it.note" class="input" maxlength="300" /></label>
      <div class="adm-actions">
        <button class="btn btn-sm" type="button" :disabled="i === 0" @click="form.items.splice(i - 1, 2, form.items[i]!, form.items[i - 1]!)">上移</button>
        <button class="btn btn-sm" type="button" @click="form.items.splice(i, 1)">删除</button>
      </div>
    </div>
    <div class="adm-actions">
      <button class="btn btn-sm" type="button" :disabled="form.items.length >= 30" @click="addItem('pack')">加内容包</button>
      <button class="btn btn-sm" type="button" :disabled="form.items.length >= 30" @click="addItem('topic')">加帖子</button>
      <button class="btn btn-sm" type="button" :disabled="form.items.length >= 30" @click="addItem('text')">加文字</button>
    </div>

    <div class="adm-form-actions">
      <button class="btn btn-primary" type="submit" :disabled="busy">{{ form.status === 'published' ? '发布' : '保存草稿' }}</button>
      <button class="btn btn-ghost" type="button" @click="editing = null">取消</button>
    </div>
  </form>

  <div class="panel adm-card">
    <p v-if="!rows.length" class="adm-empty">还没有周刊</p>
    <table v-else class="adm-table">
      <thead><tr><th>周</th><th>标题</th><th>条目</th><th>状态</th><th>操作</th></tr></thead>
      <tbody>
        <tr v-for="r in rows" :key="r.week">
          <td data-label="周" class="adm-mono">{{ r.week }}</td>
          <td data-label="标题">{{ r.title }}</td>
          <td data-label="条目">{{ r.items.length }}</td>
          <td data-label="状态">{{ r.status === 'published' ? '已发布' : '草稿' }}</td>
          <td data-label="操作" class="adm-actions">
            <button class="btn btn-sm" @click="edit(r)">编辑</button>
            <RouterLink v-if="r.status === 'published'" class="btn btn-sm" :to="`/weekly/${r.week}`" target="_blank">查看</RouterLink>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
