<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import MarkdownEditor from '@/components/MarkdownEditor.vue'
import AttachmentUploader from '@/components/AttachmentUploader.vue'
import { api } from '@/api/forum'
import type { Attachment, TopicInput } from '@/api/types'
import { useMeta } from '@/stores/meta'
import { useSession } from '@/stores/session'
import { useToast } from '@/stores/toast'
import { READ_LEVELS, TRUST_LABEL, requiredPostLevel, tagLabel } from '@/lib/format'
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
  /** 谁能读：0 所有人，1 成员以上，2 老用户以上，3 守护者 */
  readLevel: 0,
  bounty: 0,
  helpDays: 7,
})
/** 编辑时帖子原来的阅读权限；发帖人后来被降级时，原值仍然可以保留。 */
const originalReadLevel = ref<number | null>(null)
const attachments = ref<Attachment[]>([])
const tagDraft = ref('')
const busy = ref(false)
const error = ref('')

/** 当前登录用户的信任等级；没登录按 0。 */
const myLevel = computed(() => session.user?.trust_level ?? 0)

/**
 * 用户能发帖的分类：一级分类下必须选到二级；新发帖时去掉当前信任等级发不了的板块
 * （比如只有守护者能发的「小伴产品」和站务公告）。游客会先被要求登录，这里按最低等级算。
 * 编辑已有帖子时不过滤，免得作者被降级后没法改自己的帖子。
 */
const roots = computed(() =>
  meta.tree
    .map(n => ({
      ...n,
      children: n.children.filter(c => editId.value || requiredPostLevel(c) <= myLevel.value),
    }))
    .filter(n => editId.value || n.children.length > 0),
)
const root = computed(() => roots.value.find(r => r.id === form.rootId))
const leaves = computed(() => root.value?.children ?? [])
const leaf = computed(() => leaves.value.find(l => l.id === form.categoryId))
const isHelp = computed(() => leaf.value?.slug === 'share-help')
/** 选中的二级分类上的内置标签；已经加到帖子上的也留着，方便点第二次取消。 */
const presetTags = computed(() => leaf.value?.preset_tags ?? [])

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
      form.readLevel = t.min_read_level
      attachments.value = [...t.attachments]
      originalReadLevel.value = t.min_read_level
      if (t.help) form.bounty = t.help.bounty
    } catch (e) {
      toast.error(e)
    }
  },
  { immediate: true },
)

function togglePreset(tag: string) {
  const i = form.tags.indexOf(tag)
  if (i >= 0) form.tags.splice(i, 1)
  else if (form.tags.length < 5) form.tags.push(tag)
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

/** 阅读权限最高只能设到发帖人自己的信任等级。 */
function readLevelDisabled(level: number) {
  return level > myLevel.value && level !== originalReadLevel.value
}

const titleLen = computed(() => Array.from(form.title.trim()).length)

const titleHint = '一句话说清楚你想讨论什么'

async function submit() {
  error.value = ''
  if (!form.categoryId) return (error.value = '请选择分类')
  if (titleLen.value < 4) return (error.value = '标题至少 4 个字')
  if (attachments.value.some(a => !Number.isInteger(a.price) || a.price < 0 || a.price > 10)) return (error.value = '附件下载积分必须为 0～10 的整数')
  if (isHelp.value && !editId.value && (!Number.isInteger(form.bounty) || form.bounty < 0 || form.bounty > 1000)) return (error.value = '悬赏积分必须为 0～1000 的整数')
  addTag()
  const input: TopicInput = {
    category_id: form.categoryId,
    title: form.title,
    body: form.body,
    tags: form.tags,
    min_read_level: form.readLevel,
    attachment_ids: attachments.value.map(a => a.id),
    attachment_prices: attachments.value.map(a => ({ id: a.id, price: a.price })),
  }
  if (isHelp.value && !editId.value) input.help = { bounty: form.bounty, days: form.helpDays }
  busy.value = true
  try {
    const res = editId.value ? await api.updateTopic(editId.value, input) : await api.createTopic(input)
    meta.load(true)
    session.loadSession()
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
    return `新人暂时不能发帖。注册满 ${session.info.limits.tl1_days} 天并读过 ${session.info.limits.tl1_reads} 个帖子后会自动升级为成员，到时候就能发帖了。现在可以先回复和点赞。`
  }
  return ''
})

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
            <label v-for="r in roots" :key="r.id" class="root" :class="{ on: form.rootId === r.id }">
              <input v-model="form.rootId" type="radio" name="root" :value="r.id" />
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
        <p v-if="leaf?.description" class="leaf-note">{{ leaf.description }}</p>
      </section>

      <section v-if="isHelp" class="panel section">
        <h2>资源求助与悬赏</h2>
        <label class="field">
          <span>悬赏积分（0 为免费求助）</span>
          <input v-model.number="form.bounty" class="input" type="number" min="0" max="1000" step="1" :disabled="!!editId" />
          <small>{{ editId ? '已发布的悬赏金额保持不变，可在帖子页采纳或关闭求助。' : `当前可用 ${session.user?.points ?? 0} 分；发布时暂存悬赏，采纳后给回答者，关闭或到期退还。` }}</small>
        </label>
        <label v-if="!editId" class="field">
          <span>求助有效期</span>
          <select v-model.number="form.helpDays" class="select"><option :value="7">7 天</option><option :value="14">14 天</option><option :value="30">30 天</option></select>
        </label>
      </section>

      <section class="panel section">
        <label class="field">
          <span>标题</span>
          <input v-model="form.title" class="input title" maxlength="80" :placeholder="titleHint" />
          <small>{{ titleLen }} / 80</small>
        </label>
        <div class="field">
          <span>正文</span>
          <MarkdownEditor
            id="topic-body"
            v-model="form.body"
            :label="'正文'"
            :rows="12"
            :placeholder="'至少 10 个字'"
          />
        </div>
        <div class="field">
          <span>标签</span>
          <div class="tags-input">
            <span v-for="(t, i) in form.tags" :key="t" class="tag">
              #{{ tagLabel(t) }}
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
          <div v-if="presetTags.length" class="presets" role="group" aria-label="这个板块的常用标签">
            <span class="presets-label">常用标签</span>
            <button
              v-for="t in presetTags"
              :key="t"
              type="button"
              class="preset"
              :class="{ on: form.tags.includes(t) }"
              :aria-pressed="form.tags.includes(t)"
              :disabled="!form.tags.includes(t) && form.tags.length >= 5"
              @click="togglePreset(t)"
            >
              {{ tagLabel(t) }}
            </button>
          </div>
          <small>最多 5 个，方便别人按标签找到这个帖子。</small>
        </div>
        <div v-if="(session.info?.limits.upload_max_bytes ?? 0) > 0" class="field">
          <span>附件</span>
          <AttachmentUploader v-model="attachments" :max-bytes="session.info!.limits.upload_max_bytes" />
        </div>
        <fieldset class="field">
          <legend>谁能看</legend>
          <div class="seg">
            <label
              v-for="r in READ_LEVELS"
              :key="r.level"
              :class="{ on: form.readLevel === r.level, off: readLevelDisabled(r.level) }"
              :title="readLevelDisabled(r.level) ? `需要升到「${TRUST_LABEL[r.level]}」才能设置` : ''"
            >
              <input v-model="form.readLevel" type="radio" :value="r.level" :disabled="readLevelDisabled(r.level)" />{{ r.label }}
            </label>
          </div>
          <small v-if="form.readLevel > 0">
            游客和等级不够的用户看不到这个帖子，也不会出现在列表、搜索和搜索引擎里。
          </small>
          <small v-else>最高只能设到你自己的等级（你现在是「{{ TRUST_LABEL[myLevel] }}」）。</small>
        </fieldset>
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
.optional {
  margin-left: 8px;
  font-size: var(--fs-xs);
  font-weight: 400;
  color: var(--ink-3);
}
.grid.one {
  grid-template-columns: minmax(0, 1fr);
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
.leaf-note {
  margin: 0;
  font-size: var(--fs-xs);
  color: var(--ink-3);
}
.presets {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
}
.presets-label {
  font-size: var(--fs-xs);
  color: var(--ink-3);
  margin-right: 2px;
}
.preset {
  border: 1px solid var(--line);
  background: var(--surface);
  border-radius: 12px;
  padding: 2px 10px;
  font-size: var(--fs-xs);
  font-weight: 600;
  color: var(--ink-2);
}
.preset:hover:not(:disabled) {
  border-color: var(--line-strong);
}
.preset.on {
  background: var(--ink);
  border-color: var(--ink);
  color: var(--paper);
}
.preset:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.leaf.on {
  background: var(--ink);
  border-color: var(--ink);
  color: var(--paper);
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
.seg label.off {
  opacity: 0.4;
  cursor: not-allowed;
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
