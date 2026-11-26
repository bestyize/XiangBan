<script setup lang="ts">
// 内容包管理：语义状态表 + 官方登记侧滑抽屉 + 居中确认/拒审弹层（彻底淘汰原生 prompt/confirm/alert）。
import { onMounted, reactive, ref } from 'vue'
import { adminApi, type AdminPack, type PackReviewAction } from '@/api/admin'
import { kindLabel, PACK_KINDS, trustLabel } from '@/lib/content'
import AdminDrawer from './AdminDrawer.vue'
import AdminIcon from './AdminIcon.vue'
import AdminModal from './AdminModal.vue'
import AdminPager from './AdminPager.vue'
import { fmtTime, perform, usePagedList } from './shared'

const STATUS: Record<string, string> = { pending: '待审', published: '已上架', rejected: '已拒绝', removed: '已下架' }

const status = ref('')
const kind = ref('')
const list = usePagedList<AdminPack>(
  (page, size) => adminApi.packs({ status: status.value, kind: kind.value, page, page_size: size }),
  30,
)

const removeTarget = ref<AdminPack | null>(null)
const rejectTarget = ref<AdminPack | null>(null)
const rejectNote = ref('')
const rejectError = ref('')

async function review(pack: AdminPack, action: PackReviewAction, note = '') {
  const ok = await perform(() => adminApi.reviewPack(pack.id, action, pack.version, note.trim()), '已处理')
  if (ok) await list.load()
}

async function confirmRemove() {
  if (!removeTarget.value) return
  const target = removeTarget.value
  removeTarget.value = null
  await review(target, 'remove')
}

async function confirmReject() {
  if (!rejectTarget.value) return
  if (!rejectNote.value.trim()) {
    rejectError.value = '请填写拒绝原因（会记录在操作日志中）'
    return
  }
  const target = rejectTarget.value
  const note = rejectNote.value.trim()
  rejectTarget.value = null
  rejectNote.value = ''
  rejectError.value = ''
  await review(target, 'reject', note)
}

async function setFeatured(pack: AdminPack) {
  const ok = await perform(() => adminApi.setPackFeatured(pack.id, !pack.featured), pack.featured ? '已取消精选' : '已设为精选')
  if (ok) await list.load()
}

// ───────── 官方登记抽屉 ─────────
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
const showDrawer = ref(false)
const busy = ref(false)
const manifestError = ref('')

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
  manifestError.value = ''
  let manifest: Record<string, unknown>
  try {
    manifest = JSON.parse(form.manifest)
  } catch {
    manifestError.value = 'manifest 不是有效的 JSON 格式'
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
    '官方内容包已登记',
  )
  busy.value = false
  if (ok) {
    showDrawer.value = false
    await list.search()
  }
}

function statusTone(st: string) {
  if (st === 'published') return 'valid'
  if (st === 'pending') return 'suspect'
  if (st === 'rejected') return 'danger'
  return 'expired'
}

function trustTone(level: number) {
  return level === 0 ? 'info' : level === 1 ? 'valid' : 'suspect'
}

onMounted(list.load)
</script>

<template>
  <div class="adm-toolbar is-spread">
    <div class="adm-toolbar" style="margin: 0">
      <div class="adm-seg" role="group" aria-label="状态">
        <button type="button" :class="{ 'is-on': status === '' }" @click="status = ''; list.search()">全部</button>
        <button
          v-for="(label, key) in STATUS"
          :key="key"
          type="button"
          :class="{ 'is-on': status === key }"
          @click="status = key; list.search()"
        >
          {{ label }}
        </button>
      </div>
      <select v-model="kind" class="select" aria-label="类型" @change="list.search()">
        <option value="">全部类型</option>
        <option v-for="k in PACK_KINDS" :key="k.key" :value="k.key">{{ k.label }}</option>
      </select>
    </div>
    <div class="adm-actions">
      <button class="btn btn-sm" type="button" :disabled="list.loading.value" @click="list.load()">
        <AdminIcon name="refresh" :size="13" /> 刷新
      </button>
      <button class="btn btn-sm btn-primary" type="button" @click="showDrawer = true">
        <AdminIcon name="plus" :size="13" /> 官方登记
      </button>
    </div>
  </div>

  <div class="panel adm-card">
    <p v-if="list.error.value" class="adm-state error-text">{{ list.error.value }}</p>
    <p v-else-if="list.loading.value && !list.items.value.length" class="adm-state muted">正在读取内容包…</p>
    <p v-else-if="!list.items.value.length" class="adm-empty">没有符合条件的内容包</p>
    <div v-else class="adm-table-scroll">
      <table class="adm-table">
        <thead>
          <tr>
            <th>作品与版本</th>
            <th>类型</th>
            <th>信任等级</th>
            <th>状态</th>
            <th class="num">使用</th>
            <th>更新</th>
            <th class="num">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="p in list.items.value" :key="p.id">
            <td data-label="作品" class="full">
              <div class="adm-title">
                <RouterLink :to="`/packs/${p.id}`" target="_blank">{{ p.title }}</RouterLink>
                <span v-if="p.featured" class="adm-tag brand" style="margin-left: 6px">创作者精选</span>
              </div>
              <div class="adm-sub">
                <span class="adm-mono">#{{ p.id }} · v{{ p.version }}</span>
                <span>作者 {{ p.author?.nickname ?? '官方' }}</span>
              </div>
            </td>
            <td data-label="类型" class="shrink">{{ kindLabel(p.kind) }}</td>
            <td data-label="信任等级" class="shrink">
              <span class="adm-tag" :class="trustTone(p.trust_level)">{{ trustLabel(p.trust_level) }}</span>
            </td>
            <td data-label="状态" class="shrink">
              <span class="adm-tag" :class="statusTone(p.status)">{{ STATUS[p.status] ?? p.status }}</span>
            </td>
            <td data-label="使用" class="num">{{ p.apply_count }}</td>
            <td data-label="更新" class="shrink adm-mono">{{ fmtTime(p.updated_at).slice(5) }}</td>
            <td data-label="操作" class="num shrink">
              <div class="adm-actions">
                <button
                  v-if="p.status === 'published'"
                  class="btn btn-sm"
                  :class="p.featured ? 'btn-soft' : ''"
                  type="button"
                  @click="setFeatured(p)"
                >
                  {{ p.featured ? '取消精选' : '设为精选' }}
                </button>
                <button v-if="p.trust_level > 1 && p.status === 'published'" class="btn btn-sm" type="button" @click="review(p, 'approve')">
                  通过审核
                </button>
                <button v-if="p.status === 'pending'" class="btn btn-sm btn-danger" type="button" @click="rejectTarget = p; rejectNote = ''; rejectError = ''">
                  拒绝
                </button>
                <button v-if="p.status === 'published'" class="btn btn-sm btn-ghost" type="button" @click="removeTarget = p">
                  下架
                </button>
                <button v-else class="btn btn-sm btn-soft" type="button" @click="review(p, 'restore')">恢复上架</button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <AdminPager :page="list.page.value" :page-size="list.pageSize" :total="list.total.value" @go="list.goto" />
  </div>

  <AdminDrawer :open="showDrawer" title="官方登记内容包（T0 / T1）" wide @close="showDrawer = false">
    <p class="adm-callout" style="margin-bottom: 16px">
      文件由运维先放进资源机（键形如 official/xxx.zip），此处录入元数据；客户端下载时会自动核对 SHA-256。
    </p>
    <form id="pack-register-form" @submit.prevent="register">
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
            <option :value="0">官方（T0）</option>
            <option :value="1">已审核第三方（T1）</option>
          </select>
        </label>
        <label class="field"><span>标题</span><input v-model="form.title" class="input" required maxlength="60" /></label>
        <label class="field"><span>标签</span><input v-model="form.tags" class="input" placeholder="逗号或空格分隔" /></label>
        <label class="field wide"><span>简介</span><input v-model="form.summary" class="input" maxlength="500" /></label>
        <label class="field wide"><span>文件键</span><input v-model="form.file_key" class="input adm-mono" required placeholder="official/aurora-1.0.0.zip" /></label>
        <label class="field wide"><span>SHA-256</span><input v-model="form.sha256" class="input adm-mono" required maxlength="64" /></label>
        <label class="field"><span>大小（字节）</span><input v-model.number="form.size" class="input" type="number" required min="1" /></label>
        <label class="field"><span>封面键</span><input v-model="form.cover_key" class="input adm-mono" placeholder="covers/…（可空）" /></label>
        <label class="field wide">
          <span>manifest.json</span>
          <textarea
            v-model="form.manifest"
            class="textarea adm-mono"
            rows="6"
            required
            spellcheck="false"
            placeholder='{"schemaVersion":1,"kind":"effect","id":"aurora","name":"极光","version":"1.0.0"}'
            @input="manifestError = ''"
          />
        </label>
        <label class="field wide">
          <span>官方镜像（每行：名称 https 地址）</span>
          <textarea v-model="form.mirrors" class="textarea adm-mono" rows="2" spellcheck="false" placeholder="modelscope https://modelscope.cn/…" />
        </label>
      </div>
      <p v-if="manifestError" class="error-text" role="alert" style="margin: 10px 0 0">{{ manifestError }}</p>
    </form>
    <template #footer>
      <button class="btn btn-ghost" type="button" @click="showDrawer = false">取消</button>
      <button class="btn btn-primary" type="submit" form="pack-register-form" :disabled="busy">{{ busy ? '登记中…' : '确认登记' }}</button>
    </template>
  </AdminDrawer>

  <AdminModal :open="!!removeTarget" title="确认下架内容包？" @close="removeTarget = null">
    <p v-if="removeTarget" style="margin: 0; color: var(--adm-ink-2)">
      确定下架内容包「<strong>{{ removeTarget.title }}</strong>」？下架后前台将不再可见，随时可点「恢复上架」。
    </p>
    <template #footer>
      <button class="btn btn-ghost" type="button" @click="removeTarget = null">取消</button>
      <button class="btn btn-danger-fill" type="button" @click="confirmRemove">确认下架</button>
    </template>
  </AdminModal>

  <AdminModal :open="!!rejectTarget" title="拒绝内容包上架" @close="rejectTarget = null">
    <label v-if="rejectTarget" class="field">
      <span>拒绝原因（必填，会记录在操作日志中）</span>
      <textarea v-model="rejectNote" class="textarea" rows="3" placeholder="说明拒绝理由…" @input="rejectError = ''" />
    </label>
    <p v-if="rejectError" class="error-text" role="alert" style="margin: 8px 0 0">{{ rejectError }}</p>
    <template #footer>
      <button class="btn btn-ghost" type="button" @click="rejectTarget = null">取消</button>
      <button class="btn btn-danger-fill" type="button" @click="confirmReject">确认拒绝</button>
    </template>
  </AdminModal>
</template>
