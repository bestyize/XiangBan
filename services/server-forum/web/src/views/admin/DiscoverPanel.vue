<script setup lang="ts">
// 手机端社区的发现页内容：头图、日报、开源热榜、模型榜、追踪、导航、资料导读和 AI 工具。
// 内容形状在服务端校验，这里用 JSON 编辑；草稿不对外，标成示例的条目手机端会显示「示例数据」。
import { onMounted, reactive, ref, watch } from 'vue'
import { adminApi, type DiscoverRow } from '@/api/admin'
import { perform } from './shared'

const KINDS = [
  { key: 'banner', label: '首页头图', sample: '{"tag":"本周原创","title":"标题","byline":"小伴编辑部 · 8 分钟读完","link":"topic:12"}' },
  { key: 'news', label: 'AI 日报', sample: '{"title":"标题","source":"来源","url":"https://example.com/a","cover_url":""}' },
  { key: 'repo_issue', label: '开源热榜（按期）', sample: '{"issue_no":1,"week_start":"10.23","week_end":"10.29","total_gain":86420,"wow_pct":12,"daily_gain":[1,2,3,4,5,6,7],"ranks":[{"full_name":"owner/repo","description":"简介","language":"Python","language_color":"#3572A5","star_total":12400,"star_gain":3214,"spark":[1,2,3,4,5,6,7]}]}' },
  { key: 'vendor', label: '模型厂商', sample: '{"name":"厂商名","brand_color":"#E3203C","tint":"#FFE4E8"}' },
  { key: 'model', label: '模型榜', sample: '{"name":"模型名","vendor":"厂商 key","iq":70,"code":68,"math":66,"agent":69,"price":3.4,"speed":90,"latency":0.9,"context":"200K","open":false,"reasoning":true,"cn":false}' },
  { key: 'quota', label: 'Codex 额度重置', sample: '{"progress_pct":68,"next_reset_at":"2026-11-01T10:00:00Z","recent_intervals":[5,5,4.5,5,5.5,5,5]}' },
  { key: 'launch', label: '模型上新', sample: '{"title":"标题","tags":["API","编程"]}' },
  { key: 'promo', label: '限时免费', sample: '{"text":"说明","started_at":"2026-10-28T00:00:00Z","ends_at":"2026-11-04T00:00:00Z"}' },
  { key: 'nav', label: '导航', sample: '{"group":"海外","name":"名称","vendor":"厂商","url":"https://example.com","brand_color":"#D97757","tint":"#FBE9E1"}' },
  { key: 'doc', label: '资料导读', sample: '{"product":"Claude","title":"标题","subtitle":"说明","label":"guide","url":"https://example.com"}' },
  { key: 'tool', label: 'AI 工具', sample: '{"name":"名称","developer":"开发者","type_label":"桌面应用","license":"open","platforms":["Windows","macOS"],"description":"简介","homepage_url":"https://github.com/owner/repo","discussion_tag":"tag","releases":[{"version":"v1.0.0","date":"10-20","items":[{"kind":"new","text":"内容"}]}],"artifacts":[{"os":"windows","desc":".msi 安装包","size":12582912,"sha256":"","mirror_url":"","github_url":"https://github.com/owner/repo/releases"}]}' },
] as const

const kind = ref<string>(KINDS[0].key)
const rows = ref<DiscoverRow[]>([])
const editing = ref<string | null>(null)
const busy = ref(false)
const form = reactive({ key: '', sort_order: 0, status: 'published' as 'draft' | 'published', sample: false, published_at: '', json: '' })

async function load() {
  rows.value = (await adminApi.discoverItems(kind.value)).items
}

function edit(r?: DiscoverRow) {
  editing.value = r?.key ?? ''
  form.key = r?.key ?? ''
  form.sort_order = r?.sort_order ?? 0
  form.status = r?.status ?? 'published'
  form.sample = !!r?.data.sample
  form.published_at = ''
  const data = { ...(r?.data ?? JSON.parse(KINDS.find(k => k.key === kind.value)!.sample)) }
  delete data.sample
  form.json = JSON.stringify(data, null, 2)
}

async function save() {
  let data: Record<string, unknown>
  try {
    data = JSON.parse(form.json)
  } catch {
    window.alert('内容不是合法的 JSON')
    return
  }
  busy.value = true
  const ok = await perform(
    () =>
      adminApi.putDiscover(kind.value, form.key.trim(), {
        data,
        sort_order: form.sort_order,
        status: form.status,
        sample: form.sample,
        ...(form.published_at ? { published_at: new Date(form.published_at).toISOString() } : {}),
      }),
    '已保存',
  )
  busy.value = false
  if (ok) {
    editing.value = null
    await load()
  }
}

async function remove(r: DiscoverRow) {
  if (!window.confirm(`删除 ${r.key}？`)) return
  if (await perform(() => adminApi.deleteDiscover(kind.value, r.key), '已删除')) await load()
}

function title(r: DiscoverRow) {
  const d = r.data
  return String(d.title ?? d.name ?? d.text ?? (d.issue_no ? `第 ${d.issue_no} 期` : ''))
}

watch(kind, () => {
  editing.value = null
  void load()
})
onMounted(load)
</script>

<template>
  <div class="adm-toolbar">
    <select v-model="kind" class="select">
      <option v-for="k in KINDS" :key="k.key" :value="k.key">{{ k.label }}</option>
    </select>
    <button class="btn btn-primary" @click="edit()">新增</button>
  </div>

  <form v-if="editing !== null" class="panel adm-card adm-form" @submit.prevent="save">
    <div class="adm-form-grid">
      <label class="field">
        <span>key</span>
        <input v-model="form.key" class="input" required pattern="[a-z0-9_\-]{1,60}" :disabled="!!editing" />
        <small>小写字母、数字、连字符；榜单期数、工具 id 等用它区分</small>
      </label>
      <label class="field">
        <span>状态</span>
        <select v-model="form.status" class="select" style="width: 100%">
          <option value="published">发布</option>
          <option value="draft">草稿</option>
        </select>
      </label>
      <label class="field"><span>排序（小在前）</span><input v-model.number="form.sort_order" class="input" type="number" /></label>
      <label class="field">
        <span>发布时间</span>
        <input v-model="form.published_at" class="input" type="datetime-local" />
        <small>日报、上新按它倒序；不填保持原样</small>
      </label>
      <label class="field"><span><input v-model="form.sample" type="checkbox" /> 示例数据（手机端会标出）</span></label>
      <label class="field wide">
        <span>内容（JSON）</span>
        <textarea v-model="form.json" class="input adm-mono" rows="14" spellcheck="false" />
      </label>
    </div>
    <div class="adm-form-actions">
      <button class="btn btn-primary" type="submit" :disabled="busy">保存</button>
      <button class="btn btn-ghost" type="button" @click="editing = null">取消</button>
    </div>
  </form>

  <div class="panel adm-card">
    <p v-if="!rows.length" class="adm-empty">还没有内容</p>
    <table v-else class="adm-table">
      <thead><tr><th>key</th><th>标题</th><th>排序</th><th>状态</th><th>操作</th></tr></thead>
      <tbody>
        <tr v-for="r in rows" :key="r.key">
          <td data-label="key" class="adm-mono">{{ r.key }}</td>
          <td data-label="标题">{{ title(r) }}<span v-if="r.data.sample"> · 示例</span></td>
          <td data-label="排序">{{ r.sort_order }}</td>
          <td data-label="状态">{{ r.status === 'published' ? '已发布' : '草稿' }}</td>
          <td data-label="操作" class="adm-actions">
            <button class="btn btn-sm" @click="edit(r)">编辑</button>
            <button class="btn btn-sm" @click="remove(r)">删除</button>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
