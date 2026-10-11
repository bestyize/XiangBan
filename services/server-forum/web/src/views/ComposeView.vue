<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import MarkdownEditor from '@/components/MarkdownEditor.vue'
import { api } from '@/api/forum'
import type { DealType, Region, TopicInput } from '@/api/types'
import { useMeta } from '@/stores/meta'
import { useSession } from '@/stores/session'
import { useToast } from '@/stores/toast'
import { CONDITIONS, DEAL_TYPE_LABEL } from '@/lib/format'
import { useHead } from '@/lib/head'

const route = useRoute()
const router = useRouter()
const meta = useMeta()
const session = useSession()
const toast = useToast()

const editId = computed(() => (route.name === 'edit' ? Number(route.params.id) : null))
useHead(() => ({ title: editId.value ? '编辑帖子' : '发帖' }))

const form = reactive({
  rootId: 0,
  categoryId: 0,
  title: '',
  body: '',
  tags: [] as string[],
  productId: 0,
  dealType: 'free_credit' as DealType,
  valueText: '',
  conditions: 0,
  region: 'any' as Region,
  linkUrl: '',
  expiry: 'unknown' as 'unknown' | 'date' | 'long',
  expiresDate: '',
})
const tagDraft = ref('')
const busy = ref(false)
const error = ref('')
const loadedKind = ref<'deal' | 'normal' | null>(null)

/** 用户能发帖的分类：去掉站务专用的，一级分类下必须选到二级。 */
const roots = computed(() =>
  meta.tree
    .map(n => ({ ...n, children: n.children.filter(c => !c.admin_only) }))
    .filter(n => n.children.length > 0 || !n.admin_only)
    .filter(n => !loadedKind.value || n.kind === loadedKind.value),
)
const root = computed(() => roots.value.find(r => r.id === form.rootId))
const leaves = computed(() => root.value?.children ?? [])
const isDeal = computed(() => root.value?.kind === 'deal')

// 从 ?category= 预选分类（可以是一级或二级）。
watch(
  () => [meta.categories.length, route.query.category] as const,
  () => {
    if (editId.value || form.rootId) return
    const slug = typeof route.query.category === 'string' ? route.query.category : ''
    const c = meta.bySlug.get(slug)
    if (!c) return
    const parent = meta.parentOf(c)
    form.rootId = parent?.id ?? 0
    if (c.parent_id) form.categoryId = c.id
    const product = meta.products.find(p => p.slug === route.query.product)
    if (product) form.productId = product.id
  },
  { immediate: true },
)

watch(
  () => form.rootId,
  () => {
    if (!leaves.value.some(l => l.id === form.categoryId)) {
      form.categoryId = leaves.value.length === 1 ? leaves.value[0].id : 0
    }
  },
)

// 编辑：载入原帖。
watch(
  editId,
  async id => {
    if (!id) return
    try {
      const t = await api.topic(id)
      if (!t.viewer.can_edit) {
        toast.show('这个帖子不能编辑', 'error')
        router.replace(`/t/${id}`)
        return
      }
      await meta.load()
      const c = meta.byId.get(t.category.id)
      form.rootId = c?.parent_id ?? c?.id ?? 0
      form.categoryId = t.category.id
      form.title = t.title
      form.body = t.body_md
      form.tags = [...t.tags]
      loadedKind.value = t.deal ? 'deal' : 'normal'
      if (t.deal) {
        form.productId = t.deal.product.id
        form.dealType = t.deal.deal_type
        form.valueText = t.deal.value_text
        form.conditions = t.deal.conditions
        form.region = t.deal.region
        form.linkUrl = t.deal.link_url
        form.expiry = t.deal.long_term ? 'long' : t.deal.expires_at ? 'date' : 'unknown'
        form.expiresDate = t.deal.expires_at ? t.deal.expires_at.slice(0, 10) : ''
      }
    } catch (e) {
      toast.error(e)
    }
  },
  { immediate: true },
)

function toggleCondition(bit: number) {
  form.conditions ^= bit
}

function addTag() {
  const parts = tagDraft.value
    .split(/[,，\s]+/)
    .map(t => t.replace(/^#/, '').trim().toLowerCase())
    .filter(Boolean)
  for (const t of parts) {
    if (form.tags.length >= 5) break
    if (!form.tags.includes(t)) form.tags.push(t.slice(0, 20))
  }
  tagDraft.value = ''
}

function onTagKey(e: KeyboardEvent) {
  if (e.key === 'Enter' || e.key === ',' || e.key === '，') {
    e.preventDefault()
    addTag()
  } else if (e.key === 'Backspace' && !tagDraft.value && form.tags.length) {
    form.tags.pop()
  }
}

const titleLen = computed(() => Array.from(form.title.trim()).length)

const titleHint = computed(() =>
  isDeal.value ? '写清楚产品和能领到什么，比如：Claude API 新用户送 $5 额度，无需信用卡' : '一句话说清楚你想讨论什么',
)

async function submit() {
  error.value = ''
  if (!form.categoryId) return (error.value = '请选择分类')
  if (titleLen.value < 4) return (error.value = '标题至少 4 个字')
  if (isDeal.value && !form.productId) return (error.value = '请选择关联产品')
  if (isDeal.value && form.expiry === 'date' && !form.expiresDate) return (error.value = '请选择截止日期，或改成"有效期未知"')
  addTag()
  const input: TopicInput = {
    category_id: form.categoryId,
    title: form.title,
    body: form.body,
    tags: form.tags,
  }
  if (isDeal.value) {
    input.deal = {
      product_id: form.productId,
      deal_type: form.dealType,
      value_text: form.valueText,
      conditions: form.conditions,
      region: form.region,
      link_url: form.linkUrl.trim(),
      // 截止日期按北京时间当天结束算。
      expires_at: form.expiry === 'date' ? new Date(`${form.expiresDate}T23:59:59+08:00`).toISOString() : null,
      long_term: form.expiry === 'long',
    }
  }
  busy.value = true
  try {
    const res = editId.value ? await api.updateTopic(editId.value, input) : await api.createTopic(input)
    meta.load(true)
    toast.show(editId.value ? '修改已保存' : '发布成功')
    router.replace(`/t/${res.id}`)
  } catch (e) {
    error.value = e instanceof Error ? e.message : '发布失败'
  } finally {
    busy.value = false
  }
}

const blocked = computed(() => {
  const u = session.user
  if (!session.info || editId.value) return ''
  if (u?.status === 'muted') return '你目前处于禁言中，暂时不能发帖。'
  if (session.info.limits.daily_topics <= 0) {
    return `新人暂时不能发帖。注册满 ${session.info.limits.tl1_days} 天并读过 ${session.info.limits.tl1_reads} 个帖子后会自动升级为成员，到时候就能发帖和投票了。现在可以先回复和点赞。`
  }
  return ''
})

const today = new Date().toISOString().slice(0, 10)
</script>

<template>
  <div class="compose-page">
    <header class="head">
      <h1>{{ editId ? '编辑帖子' : '发帖' }}</h1>
      <RouterLink to="/rules" class="muted rules-link">发帖前看看社区规则</RouterLink>
    </header>

    <div v-if="blocked" class="panel blocked">
      <p>{{ blocked }}</p>
      <RouterLink to="/" class="btn btn-sm">去逛逛</RouterLink>
    </div>

    <form v-else class="form" novalidate @submit.prevent="submit">
      <section class="panel section">
        <fieldset class="kinds">
          <legend>发到哪里</legend>
          <div class="root-opts">
            <label v-for="r in roots" :key="r.id" class="root" :class="{ on: form.rootId === r.id, deal: r.kind === 'deal' }">
              <input v-model="form.rootId" type="radio" name="root" :value="r.id" :disabled="!!editId && r.kind !== loadedKind" />
              <span class="rname">{{ r.name }}</span>
              <span class="rdesc">{{ r.description }}</span>
            </label>
          </div>
        </fieldset>
        <div v-if="leaves.length" class="leaves" role="radiogroup" aria-label="二级分类">
          <label v-for="l in leaves" :key="l.id" class="leaf" :class="{ on: form.categoryId === l.id }">
            <input v-model="form.categoryId" type="radio" name="leaf" :value="l.id" />
            {{ l.name }}
          </label>
        </div>
      </section>

      <section v-if="isDeal" class="panel section deal">
        <h2>羊毛信息</h2>
        <div class="grid">
          <label class="field">
            <span>关联产品</span>
            <select v-model.number="form.productId" class="select" required>
              <option :value="0" disabled>选择产品</option>
              <option v-for="p in meta.products" :key="p.id" :value="p.id">{{ p.name }}{{ p.vendor && p.vendor !== p.name ? `（${p.vendor}）` : '' }}</option>
            </select>
            <small>没有你要的产品就选"其他"，并在标题里写清楚。</small>
          </label>
          <label class="field">
            <span>羊毛类型</span>
            <select v-model="form.dealType" class="select">
              <option v-for="(label, key) in DEAL_TYPE_LABEL" :key="key" :value="key">{{ label }}</option>
            </select>
          </label>
          <label class="field">
            <span>能领到什么</span>
            <input v-model="form.valueText" class="input" maxlength="40" placeholder="$5 额度、首月 1 元、3 个月 Pro" />
            <small>会高亮显示在列表里，数字放在最前面更醒目。</small>
          </label>
          <label class="field">
            <span>官方活动页</span>
            <input v-model="form.linkUrl" class="input" type="url" inputmode="url" maxlength="500" placeholder="https://" />
            <small>不要带 ref、aff、invite 这类个人推广参数。</small>
          </label>
        </div>

        <fieldset class="field conds">
          <legend>领取条件（需要什么就勾什么）</legend>
          <div class="checks">
            <label v-for="c in CONDITIONS" :key="c.key" class="check" :class="{ on: form.conditions & c.bit }">
              <input type="checkbox" :checked="!!(form.conditions & c.bit)" @change="toggleCondition(c.bit)" />
              {{ c.label }}
            </label>
          </div>
        </fieldset>

        <div class="grid">
          <fieldset class="field">
            <legend>能在哪用</legend>
            <div class="seg">
              <label :class="{ on: form.region === 'any' }"><input v-model="form.region" type="radio" value="any" />不限</label>
              <label :class="{ on: form.region === 'cn' }"><input v-model="form.region" type="radio" value="cn" />国内</label>
              <label :class="{ on: form.region === 'intl' }"><input v-model="form.region" type="radio" value="intl" />仅海外</label>
            </div>
          </fieldset>
          <fieldset class="field">
            <legend>有效期</legend>
            <div class="seg">
              <label :class="{ on: form.expiry === 'unknown' }"><input v-model="form.expiry" type="radio" value="unknown" />不清楚</label>
              <label :class="{ on: form.expiry === 'date' }"><input v-model="form.expiry" type="radio" value="date" />截止到</label>
              <label :class="{ on: form.expiry === 'long' }"><input v-model="form.expiry" type="radio" value="long" />长期有效</label>
            </div>
            <input v-if="form.expiry === 'date'" v-model="form.expiresDate" class="input date" type="date" :min="today" aria-label="截止日期" />
          </fieldset>
        </div>
      </section>

      <section class="panel section">
        <label class="field">
          <span>标题</span>
          <input v-model="form.title" class="input title" maxlength="80" :placeholder="titleHint" />
          <small>{{ titleLen }} / 80</small>
        </label>
        <div class="field">
          <span>{{ isDeal ? '领取步骤和补充说明' : '正文' }}</span>
          <MarkdownEditor
            id="topic-body"
            v-model="form.body"
            :label="isDeal ? '领取步骤' : '正文'"
            :rows="12"
            :placeholder="isDeal ? '1. 打开活动页注册\n2. 在控制台领取\n3. 注意事项……' : '至少 10 个字'"
          />
        </div>
        <div class="field">
          <span>标签</span>
          <div class="tags-input">
            <span v-for="(t, i) in form.tags" :key="t" class="tag">
              #{{ t }}
              <button type="button" :aria-label="`移除标签 ${t}`" @click="form.tags.splice(i, 1)">×</button>
            </span>
            <input
              v-model="tagDraft"
              :disabled="form.tags.length >= 5"
              placeholder="claude、gpu、学生，回车添加"
              aria-label="添加标签"
              @keydown="onTagKey"
              @blur="addTag"
            />
          </div>
          <small>最多 5 个，产品名做标签能让帖子出现在产品页。</small>
        </div>
      </section>

      <div class="submit">
        <p v-if="error" class="error-text" role="alert">{{ error }}</p>
        <RouterLink :to="editId ? `/t/${editId}` : '/'" class="btn btn-ghost">取消</RouterLink>
        <button type="submit" class="btn btn-primary big" :disabled="busy">
          {{ busy ? '正在提交' : editId ? '保存修改' : '发布' }}
        </button>
      </div>
    </form>
  </div>
</template>

<style scoped>
.compose-page {
  max-width: 780px;
  margin: 0 auto;
}
.head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  margin-bottom: 14px;
}
h1 {
  margin: 0;
  font-size: var(--fs-2xl);
  font-weight: 800;
}
.rules-link {
  font-size: var(--fs-sm);
  text-decoration: underline;
  text-underline-offset: 3px;
}
.blocked {
  padding: 24px;
  display: grid;
  gap: 14px;
  justify-items: start;
}
.blocked p {
  margin: 0;
  line-height: 1.7;
  color: var(--ink-2);
}
.form {
  display: grid;
  gap: 14px;
}
.section {
  padding: 18px;
  display: grid;
  gap: 16px;
}
.section h2 {
  margin: 0;
  font-size: var(--fs-md);
}
fieldset {
  border: 0;
  margin: 0;
  padding: 0;
  min-width: 0;
}
legend {
  font-size: var(--fs-sm);
  font-weight: 600;
  color: var(--ink-2);
  padding: 0;
  margin-bottom: 8px;
}
.root-opts {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 8px;
}
.root {
  position: relative;
  display: grid;
  gap: 2px;
  padding: 10px 12px;
  border: 1px solid var(--line);
  border-radius: var(--r-sm);
  cursor: pointer;
}
.root input,
.leaf input,
.seg input,
.check input {
  position: absolute;
  opacity: 0;
  pointer-events: none;
}
.root:has(input:focus-visible),
.leaf:has(input:focus-visible),
.seg label:has(input:focus-visible),
.check:has(input:focus-visible) {
  outline: 2px solid var(--focus);
  outline-offset: 2px;
}
.root:has(input:disabled) {
  opacity: 0.4;
  cursor: not-allowed;
}
.root.on {
  border-color: var(--ink);
  box-shadow: inset 0 0 0 1px var(--ink);
}
.root.deal.on {
  background: color-mix(in srgb, var(--brand) 7%, var(--surface));
}
.rname {
  font-weight: 700;
  font-size: var(--fs-sm);
}
.rdesc {
  font-size: 11.5px;
  color: var(--ink-3);
  line-height: 1.4;
}
.leaves {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding-top: 14px;
  border-top: 1px solid var(--line);
}
.leaf {
  position: relative;
  padding: 4px 12px;
  border-radius: 14px;
  border: 1px solid var(--line);
  font-size: var(--fs-sm);
  cursor: pointer;
}
.leaf.on {
  background: var(--ink);
  border-color: var(--ink);
  color: var(--paper);
}
.deal {
  border-color: color-mix(in srgb, var(--brand) 35%, var(--line));
}
.grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px 16px;
}
.checks {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.check {
  position: relative;
  padding: 5px 11px;
  border: 1px solid var(--line);
  border-radius: var(--r-sm);
  font-size: var(--fs-sm);
  cursor: pointer;
}
.check.on {
  border-color: var(--suspect);
  background: var(--suspect-soft);
  color: var(--suspect);
  font-weight: 600;
}
.seg {
  display: inline-flex;
  border: 1px solid var(--line-strong);
  border-radius: var(--r-sm);
  overflow: hidden;
}
.seg label {
  position: relative;
  padding: 5px 12px;
  font-size: var(--fs-sm);
  cursor: pointer;
  border-left: 1px solid var(--line);
}
.seg label:first-child {
  border-left: 0;
}
.seg label.on {
  background: var(--ink);
  color: var(--paper);
}
.date {
  margin-top: 8px;
  max-width: 200px;
}
.title {
  font-size: var(--fs-lg);
  font-weight: 600;
}
.tags-input {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
  border: 1px solid var(--line-strong);
  border-radius: var(--r-sm);
  padding: 6px 8px;
  background: var(--surface);
}
.tags-input:focus-within {
  border-color: var(--focus);
}
.tags-input input {
  flex: 1;
  min-width: 160px;
  border: 0;
  outline: none;
  background: none;
  font-size: var(--fs-sm);
  padding: 2px;
}
.tag {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  padding: 1px 4px 1px 9px;
  border-radius: 12px;
  background: var(--surface-2);
  border: 1px solid var(--line);
  font-size: var(--fs-xs);
  font-weight: 600;
}
.tag button {
  border: 0;
  background: none;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  color: var(--ink-3);
  line-height: 1;
}
.tag button:hover {
  background: var(--line);
}
.submit {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
}
.submit .error-text {
  margin: 0 auto 0 0;
}
.big {
  height: 42px;
  padding: 0 26px;
  font-size: var(--fs-md);
}
@media (max-width: 767px) {
  h1 {
    font-size: var(--fs-xl);
  }
  .section {
    padding: 14px;
  }
  .root-opts {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  .grid {
    grid-template-columns: minmax(0, 1fr);
  }
  .seg {
    display: flex;
  }
  .seg label {
    flex: 1;
    text-align: center;
  }
  .submit {
    flex-wrap: wrap;
  }
  .submit .error-text {
    flex-basis: 100%;
  }
  .big {
    flex: 1;
  }
}
</style>
