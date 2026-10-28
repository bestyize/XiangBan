<script setup lang="ts">
// 内容包：待审队列、全部内容包的上下架与审核，以及官方直接登记。
import { onMounted, reactive, ref } from 'vue'
import { adminApi, type AdminPack, type PackReviewAction } from '@/api/admin'
import { kindLabel, PACK_KINDS, trustLabel } from '@/lib/content'
import AdminPager from './AdminPager.vue'
import { fmtTime, perform, usePagedList } from './shared'

const STATUS: Record<string, string> = { pending: '待审', published: '已上架', rejected: '已拒绝', removed: '已下架' }

const status = ref('')
const kind = ref('')
const list = usePagedList<AdminPack>((page, size) => adminApi.packs({ status: status.value, kind: kind.value, page, page_size: size }), 30)

async function review(id: number, action: PackReviewAction, confirmText?: string) {
  if (confirmText && !window.confirm(confirmText)) return
  const note = action === 'reject' ? (window.prompt('拒绝原因（会记录在操作日志里）') ?? '') : ''
  if (action === 'reject' && !note.trim()) return
  const ok = await perform(() => adminApi.reviewPack(id, action, note.trim()), '已处理')
  if (ok) await list.load()
}

// ───────── 官方登记 ─────────
const form = reactive({
  kind: 'effect',
  title: '',
  summary: '',
  tags: '',
  manifest: '',
  file_key: '',
  sha256: '',
  size: 0,
  cover_key: '',
  mirrors: '',
  trust_level: 0 as 0 | 1,
})
const showForm = ref(false)
const busy = ref(false)

function parseMirrors(): { name: string; url: string }[] {
  return form.mirrors
    .split('\n')
    .map(l => l.trim())
    .filter(Boolean)
    .map(l => {
      const [name, ...rest] = l.split(/\s+/)
      return { name: name ?? '', url: rest.join('') }
    })
}

async function register() {
  let manifest: Record<string, unknown>
  try {
    manifest = JSON.parse(form.manifest)
  } catch {
    window.alert('manifest 不是有效的 JSON')
    return
  }
  busy.value = true
  const ok = await perform(
    () =>
      adminApi.createPack({
        kind: form.kind,
        title: form.title.trim(),
        summary: form.summary.trim(),
        tags: form.tags.split(/[,，、\s]+/).filter(Boolean),
        manifest,
        file_key: form.file_key.trim(),
        sha256: form.sha256.trim().toLowerCase(),
        size: Number(form.size),
        cover_key: form.cover_key.trim() || undefined,
        mirrors: parseMirrors(),
        trust_level: form.trust_level,
      }),
    '已登记',
  )
  busy.value = false
  if (ok) {
    showForm.value = false
    await list.search()
  }
}

onMounted(list.load)
</script>

<template>
  <div class="adm-toolbar">
    <select v-model="status" class="select" aria-label="状态" @change="list.search()">
      <option value="">全部状态</option>
      <option v-for="(label, key) in STATUS" :key="key" :value="key">{{ label }}</option>
    </select>
    <select v-model="kind" class="select" aria-label="类型" @change="list.search()">
      <option value="">全部类型</option>
      <option v-for="k in PACK_KINDS" :key="k.key" :value="k.key">{{ k.label }}</option>
    </select>
    <button class="btn" :disabled="list.loading.value" @click="list.load()">刷新</button>
    <button class="btn btn-primary" @click="showForm = !showForm">{{ showForm ? '收起' : '官方登记' }}</button>
  </div>

  <form v-if="showForm" class="panel adm-card adm-form" @submit.prevent="register">
    <p class="adm-sub">文件由运维先放进资源机（键形如 official/xxx.zip），这里只录入元数据；客户端会用这里的 sha256 校验下载。</p>
    <div class="adm-form-grid">
      <label class="field">
        <span>类型</span>
        <select v-model="form.kind" class="select" style="width: 100%">
          <option v-for="k in PACK_KINDS" :key="k.key" :value="k.key">{{ k.label }}</option>
        </select>
      </label>
      <label class="field">
        <span>信任等级</span>
        <select v-model.number="form.trust_level" class="select" style="width: 100%">
          <option :value="0">官方</option>
          <option :value="1">已审核（第三方）</option>
        </select>
      </label>
      <label class="field"><span>标题</span><input v-model="form.title" class="input" required maxlength="60" /></label>
      <label class="field"><span>标签</span><input v-model="form.tags" class="input" placeholder="逗号或空格分隔" /></label>
      <label class="field wide"><span>简介</span><input v-model="form.summary" class="input" maxlength="500" /></label>
      <label class="field wide"><span>文件键</span><input v-model="form.file_key" class="input" required placeholder="official/aurora-1.0.0.zip" /></label>
      <label class="field wide"><span>sha256</span><input v-model="form.sha256" class="input" required maxlength="64" /></label>
      <label class="field"><span>大小（字节）</span><input v-model.number="form.size" class="input" type="number" required min="1" /></label>
      <label class="field"><span>封面键</span><input v-model="form.cover_key" class="input" placeholder="covers/…（可空）" /></label>
      <label class="field wide">
        <span>manifest.json</span>
        <textarea v-model="form.manifest" class="input" rows="6" required spellcheck="false" placeholder='{"schemaVersion":1,"kind":"effect","id":"aurora","name":"极光","version":"1.0.0"}' />
      </label>
      <label class="field wide">
        <span>镜像（每行：名称 https 地址）</span>
        <textarea v-model="form.mirrors" class="input" rows="2" spellcheck="false" placeholder="modelscope https://modelscope.cn/…" />
      </label>
    </div>
    <div class="adm-form-actions">
      <button class="btn btn-primary" type="submit" :disabled="busy">登记</button>
    </div>
  </form>

  <div class="panel adm-card">
    <p v-if="list.error.value" class="adm-state error-text">{{ list.error.value }}</p>
    <p v-else-if="list.loading.value && !list.items.value.length" class="adm-state muted">加载中</p>
    <p v-else-if="!list.items.value.length" class="adm-empty">没有内容包</p>
    <table v-else class="adm-table">
      <thead>
        <tr><th>标题</th><th>类型</th><th>等级</th><th>状态</th><th>作者</th><th>使用</th><th>更新</th><th>操作</th></tr>
      </thead>
      <tbody>
        <tr v-for="p in list.items.value" :key="p.id">
          <td data-label="标题" class="adm-wrap"><RouterLink :to="`/packs/${p.id}`" target="_blank">{{ p.title }}</RouterLink> <small class="adm-mono">v{{ p.version }}</small></td>
          <td data-label="类型">{{ kindLabel(p.kind) }}</td>
          <td data-label="等级">{{ trustLabel(p.trust_level) }}</td>
          <td data-label="状态">{{ STATUS[p.status] ?? p.status }}</td>
          <td data-label="作者">{{ p.author?.nickname ?? '官方' }}</td>
          <td data-label="使用">{{ p.apply_count }}</td>
          <td data-label="更新" class="shrink">{{ fmtTime(p.updated_at) }}</td>
          <td data-label="操作" class="adm-actions">
            <button v-if="p.trust_level > 1 && p.status === 'published'" class="btn btn-sm" @click="review(p.id, 'approve')">通过审核</button>
            <button v-if="p.status === 'published'" class="btn btn-sm" @click="review(p.id, 'remove', '确定下架这个内容包？')">下架</button>
            <button v-else class="btn btn-sm" @click="review(p.id, 'restore')">恢复上架</button>
          </td>
        </tr>
      </tbody>
    </table>
    <AdminPager :page="list.page.value" :page-size="list.pageSize" :total="list.total.value" @go="list.goto" />
  </div>
</template>
