<script setup lang="ts">
import { computed, onMounted, reactive, ref, useTemplateRef, watch } from 'vue'
import ModalDialog from '@/components/ModalDialog.vue'
import MarkdownEditor from '@/components/MarkdownEditor.vue'
import AttachmentUploader from '@/components/AttachmentUploader.vue'
import { api } from '@/api/forum'
import type { Attachment, TopicInput } from '@/api/types'
import { useMeta } from '@/stores/meta'
import { useSession } from '@/stores/session'
import { useToast } from '@/stores/toast'
import { useTopicComposer } from '@/stores/topicComposer'
import { READ_LEVELS, TRUST_LABEL, tagLabel } from '@/lib/format'
import { postingSections, postingSelection, type PostingSectionKey } from '@/lib/postingStructure'
import { canonicalCategory, navigationSection } from '@/lib/communityStructure'
const meta = useMeta()
const session = useSession()
const toast = useToast()
const composer = useTopicComposer()

const editId = computed(() => composer.target?.editId)

const form = reactive({
  sectionKey: 'discuss' as PostingSectionKey,
  boardKey: '',
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
const originalCategoryId = ref<number>()
const originalReady = ref(false)
const originalError = ref('')
const editRetry = ref(0)
const selectionTouched = ref(false)
const attachments = ref<Attachment[]>([])
const attachmentUploading = ref(false)
const editor = useTemplateRef<InstanceType<typeof MarkdownEditor>>('bodyEditor')
const uploading = computed(() => attachmentUploading.value || !!editor.value?.uploading)
const tagDraft = ref('')
const busy = ref(false)
const error = ref('')

/** 当前登录用户的信任等级；没登录按 0。 */
const myLevel = computed(() => session.user?.trust_level ?? 0)

const destinations = computed(() => postingSections(meta.categories, myLevel.value, originalCategoryId.value))
const destination = computed(() => destinations.value.find(s => s.key === form.sectionKey))
const leaf = computed(() => destination.value?.boards.find(c => c.key === form.boardKey && !c.unavailableReason))
const noDestinations = computed(() => !destinations.value.some(section =>
  section.boards.some(board => !board.unavailableReason),
))
const parent = computed(() => meta.byId.get(leaf.value?.parent_id ?? 0))
const isCasual = computed(() => leaf.value?.allow_short_title ?? (leaf.value?.slug === 'casual' || parent.value?.slug === 'casual'))
const isHelp = computed(() => canonicalCategory(leaf.value?.slug ?? '') === 'resource-help')
const presetTags = computed(() => leaf.value?.preset_tags ?? [])
const topicTag = computed(() => leaf.value?.topicTag)
const tags = computed(() => topicTag.value ? [...new Set([topicTag.value, ...form.tags])] : form.tags)
const requestedSlug = computed(() => editId.value ? '' : composer.target?.category ?? '')
const selectionNote = computed(() => {
  if (selectionTouched.value || !requestedSlug.value) return ''
  const key = canonicalCategory(requestedSlug.value)
  const board = destination.value?.boards.find(candidate => candidate.key === key)
  if (board?.unavailableReason) return `${board.name}（${board.unavailableReason}）`
  return leaf.value ? '' : '请选择当前可发帖的板块'
})

onMounted(() => { void meta.load() })

watch(
  [destinations, requestedSlug],
  () => {
    if (editId.value || selectionTouched.value || !meta.categories.length) return
    Object.assign(form, postingSelection(destinations.value, requestedSlug.value, meta.categories, composer.target?.section))
  },
  { immediate: true },
)
watch(leaf, (current, previous) => {
  if (!current && previous && form.boardKey === previous.key) form.boardKey = ''
})

function selectSection() {
  selectionTouched.value = true
  Object.assign(form, postingSelection(destinations.value, '', meta.categories, form.sectionKey))
}

function snapshot() {
  return JSON.stringify({
    ...form, sectionKey: editId.value ? form.sectionKey : undefined, boardKey: editId.value ? form.boardKey : undefined,
    tagDraft: tagDraft.value, attachments: attachments.value.map(({ id, price }) => [id, price]),
  })
}
const originalDraft = ref(snapshot())

function requestClose() {
  if (busy.value || uploading.value) return
  if (snapshot() !== originalDraft.value && !window.confirm('当前内容还未保存，确定关闭吗？')) return
  composer.close()
}

watch(
  [editId, () => composer.ready, editRetry],
  async ([id, ready], _, onCleanup) => {
    if (!id || !ready || originalReady.value) return
    originalError.value = ''
    let current = true
    onCleanup(() => { current = false })
    try {
      const t = await api.topic(id)
      if (!current) return
      if (!t.viewer.can_edit) {
        originalError.value = '这个帖子不能编辑'
        return
      }
      await meta.load()
      if (!current) return
      const c = meta.byId.get(t.category.id)
      originalCategoryId.value = t.category.id
      form.sectionKey = c ? navigationSection(c) : 'discuss'
      form.boardKey = c?.slug ?? t.category.slug
      form.title = t.title
      form.body = t.body_md
      form.tags = [...t.tags]
      form.readLevel = t.min_read_level
      attachments.value = [...t.attachments]
      originalReadLevel.value = t.min_read_level
      if (t.help) form.bounty = t.help.bounty
      originalDraft.value = snapshot()
      originalReady.value = true
    } catch (e) {
      if (current) originalError.value = e instanceof Error ? e.message : '帖子加载失败'
    }
  },
  { immediate: true },
)

function togglePreset(tag: string) {
  if (tag === topicTag.value) return
  const i = form.tags.indexOf(tag)
  if (i >= 0) form.tags.splice(i, 1)
  else if (tags.value.length < 5) form.tags.push(tag)
}

function addTag() {
  const parts = tagDraft.value
    .split(/[,，\s]+/)
    .map(t => t.replace(/^#/, '').trim().toLowerCase().slice(0, 20))
    .filter(Boolean)
  for (const t of parts) {
    if (tags.value.length >= 5) break
    if (!tags.value.includes(t)) form.tags.push(t)
  }
  tagDraft.value = ''
}

function removeTag(tag: string) {
  form.tags = form.tags.filter(current => current !== tag)
}

function onTagKey(e: KeyboardEvent) {
  if (e.key === 'Enter' || e.key === ',' || e.key === '，') {
    e.preventDefault()
    addTag()
  } else if (e.key === 'Backspace' && !tagDraft.value) {
    const last = form.tags.filter(tag => tag !== topicTag.value).at(-1)
    if (last) removeTag(last)
  }
}

/** 阅读权限最高只能设到发帖人自己的信任等级。 */
function readLevelDisabled(level: number) {
  return level > myLevel.value && level !== originalReadLevel.value
}

const titleLen = computed(() => Array.from(form.title.trim()).length)

const titleHint = computed(() => isCasual.value ? '添加标题（可选）' : '一句话说清楚你想讨论什么')

async function submit() {
  if (busy.value || uploading.value || !composer.ready) return
  error.value = ''
  if (!leaf.value || meta.categoriesLoading || meta.categoriesError || (editId.value && !originalReady.value)) return (error.value = '请选择可发帖的板块')
  if (!isCasual.value && titleLen.value < 4) return (error.value = '标题至少 4 个字')
  if (attachments.value.some(a => !Number.isInteger(a.price) || a.price < 0 || a.price > 10)) return (error.value = '附件下载积分必须为 0～10 的整数')
  if (isHelp.value && !editId.value && (!Number.isInteger(form.bounty) || form.bounty < 0 || form.bounty > 1000)) return (error.value = '悬赏积分必须为 0～1000 的整数')
  addTag()
  if (tags.value.length > 5) return (error.value = '最多 5 个标签，请先移除一个')
  const target = composer.target
  const input: TopicInput = {
    category_id: leaf.value.id,
    title: form.title,
    body: form.body,
    tags: [...tags.value],
    min_read_level: form.readLevel,
    attachment_ids: attachments.value.map(a => a.id),
    attachment_prices: attachments.value.map(a => ({ id: a.id, price: a.price })),
  }
  if (isHelp.value && !editId.value) input.help = { bounty: form.bounty, days: form.helpDays }
  busy.value = true
  try {
    const res = target?.editId ? await api.updateTopic(target.editId, input) : await api.createTopic(input)
    if (composer.target !== target) return
    session.loadSession()
    toast.show(target?.editId ? '修改已保存' : '发布成功', 'info',
      target?.editId ? undefined : { label: '查看帖子', to: `/t/${res.id}` })
    composer.complete(res.id)
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
  <ModalDialog :open="composer.visible" :title="editId ? '编辑帖子' : '发帖'" :width="780" scrollable keep-on-backdrop @close="requestClose">
    <div v-if="!composer.ready" class="composer-state" role="status">
      <p>正在确认账号信息…</p>
      <button v-if="!session.restoring" type="button" class="btn btn-sm" @click="session.loadSession()">重新读取</button>
    </div>
    <div v-else-if="blocked" class="composer-state" role="status">
      <p>{{ blocked }}</p>
    </div>
    <div v-else-if="editId && !originalReady" class="composer-state" role="status">
      <p>{{ originalError || '正在读取帖子…' }}</p>
      <button v-if="originalError" type="button" class="btn btn-sm" @click="editRetry++">重试</button>
    </div>

    <form v-if="!blocked && (!editId || originalReady)" v-show="composer.ready" class="form" novalidate @submit.prevent="submit">
      <div class="form-fields" :inert="busy">
        <section class="section">
          <fieldset v-if="destinations.length" class="destinations" :disabled="busy || meta.categoriesLoading || !!meta.categoriesError">
            <label class="field">
              <span>栏目</span>
              <select v-model="form.sectionKey" class="select" @change="selectSection">
                <option v-for="section in destinations" :key="section.key" :value="section.key">{{ section.name }}</option>
              </select>
            </label>
            <label class="field">
              <span>板块</span>
              <select v-model="form.boardKey" class="select" @change="selectionTouched = true">
                <option value="" disabled>选择板块</option>
                <option v-for="board in destination?.boards" :key="board.key" :value="board.key" :disabled="!!board.unavailableReason">
                  {{ board.name }}{{ board.unavailableReason ? `（${board.unavailableReason}）` : '' }}
                </option>
              </select>
            </label>
          </fieldset>
          <p v-if="meta.categoriesLoading" class="leaf-note" role="status">正在加载板块</p>
          <div v-else-if="meta.categoriesError" class="leaf-note" role="status">
            {{ meta.categoriesError }}
            <button type="button" class="btn btn-sm" @click="meta.load(true)">重试</button>
          </div>
          <p v-else-if="selectionNote" class="leaf-note" role="status">{{ selectionNote }}</p>
          <p v-else-if="noDestinations" class="leaf-note" role="status">当前没有可发帖的板块</p>
          <p v-else-if="leaf?.description && !topicTag" class="leaf-note">{{ leaf.description }}</p>
        </section>

        <section v-if="isHelp" class="section">
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

        <section class="section">
          <label class="field">
            <span>标题</span>
            <input v-model="form.title" class="input title" maxlength="80" :placeholder="titleHint" />
            <small>{{ titleLen }} / 80</small>
          </label>
          <div class="field">
            <span>正文</span>
            <MarkdownEditor
              id="topic-body"
              ref="bodyEditor"
              v-model="form.body"
              :label="'正文'"
              :rows="9"
              :placeholder="'至少 10 个字'"
            />
          </div>
          <div class="field">
            <span>标签</span>
            <div class="tags-input">
              <span v-for="t in tags" :key="t" class="tag" :class="{ required: t === topicTag }">
                #{{ t === topicTag ? leaf?.name : tagLabel(t) }}
                <button v-if="t !== topicTag" type="button" :aria-label="`移除标签 ${t}`" @click="removeTag(t)">×</button>
              </span>
              <input
                v-model="tagDraft"
                :disabled="tags.length >= 5"
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
                :class="{ on: tags.includes(t) }"
                :aria-pressed="tags.includes(t)"
                :disabled="t === topicTag || (!tags.includes(t) && tags.length >= 5)"
                @click="togglePreset(t)"
              >
                {{ tagLabel(t) }}
              </button>
            </div>
            <small>最多 5 个，方便别人按标签找到这个帖子。</small>
          </div>
          <div v-if="(session.info?.limits.upload_max_bytes ?? 0) > 0" class="field">
            <span>附件</span>
            <AttachmentUploader v-model="attachments" :max-bytes="session.info!.limits.upload_max_bytes" :disabled="busy" @uploading="attachmentUploading = $event" />
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
      </div>

      <footer class="submit">
        <p v-if="error" class="error-text" role="alert">{{ error }}</p>
        <RouterLink v-else to="/rules" class="muted rules-link" target="_blank" rel="noopener">社区规则</RouterLink>
        <button type="button" class="btn btn-ghost" :disabled="busy || uploading" @click="requestClose">取消</button>
        <button type="submit" class="btn btn-primary big" :disabled="busy || uploading || !leaf || meta.categoriesLoading || !!meta.categoriesError">
          {{ busy ? '正在提交' : uploading ? '正在上传' : editId ? '保存修改' : '发布' }}
        </button>
      </footer>
    </form>
  </ModalDialog>
</template>

<style scoped>
.form {
  display: flex;
  flex-direction: column;
  min-height: 0;
}
.form-fields {
  min-height: 0;
  overflow-y: auto;
  overscroll-behavior: contain;
  display: grid;
  gap: 16px;
  padding: 18px 22px 22px;
}
.section {
  display: grid;
  gap: 14px;
  min-width: 0;
}
.section + .section {
  border-top: 1px solid var(--line);
  padding-top: 16px;
}
.section h2 { margin: 0; font-size: var(--fs-md); }
.destinations {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 2fr);
  gap: 12px;
}
fieldset { border: 0; margin: 0; padding: 0; min-width: 0; }
legend { margin-bottom: 6px; padding: 0; font-size: var(--fs-sm); font-weight: 600; color: var(--ink-2); }
.composer-state { padding: 16px 22px 22px; color: var(--ink-2); }
.composer-state p { margin: 0 0 8px; }
.leaf-note { margin: 0; color: var(--ink-3); font-size: var(--fs-sm); }
.title { font-size: var(--fs-lg); font-weight: 600; }
.presets { display: flex; flex-wrap: wrap; align-items: center; gap: 6px; }
.presets-label { margin-inline-end: 2px; font-size: var(--fs-xs); color: var(--ink-3); }
.preset {
  padding: 2px 10px;
  border: 1px solid var(--line);
  border-radius: 12px;
  background: var(--surface);
  color: var(--ink-2);
  font-size: var(--fs-xs);
  font-weight: 600;
}
.preset:hover:not(:disabled) { border-color: var(--line-strong); }
.preset.on { border-color: var(--brand); color: var(--brand); }
.preset:disabled { opacity: 0.5; cursor: not-allowed; }
.seg { display: flex; flex-wrap: wrap; gap: 6px; }
.seg label {
  position: relative;
  padding: 5px 12px;
  border: 1px solid var(--line);
  border-radius: var(--r-sm);
  font-size: var(--fs-sm);
  cursor: pointer;
}
.seg input { position: absolute; opacity: 0; pointer-events: none; }
.seg label:has(input:focus-visible) { outline: 2px solid var(--focus); outline-offset: 2px; }
.seg label.on { border-color: var(--brand); color: var(--brand); }
.seg label.off { opacity: 0.4; cursor: not-allowed; }
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
.tags-input:focus-within { border-color: var(--focus); }
.tags-input input { flex: 1; min-width: 160px; border: 0; outline: none; background: none; padding: 2px; }
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
.tag.required { padding-inline: 9px; color: var(--brand); }
.tag button { border: 0; background: none; width: 18px; height: 18px; border-radius: 50%; color: var(--ink-3); line-height: 1; }
.tag button:hover { background: var(--line); }
.submit {
  flex: none;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
  padding: 12px 22px;
  border-top: 1px solid var(--line);
}
.submit .error-text, .rules-link { margin: 0 auto 0 0; }
.rules-link { font-size: var(--fs-sm); }
.rules-link:hover { color: var(--brand); }
.big { height: 42px; padding-inline: 22px; }
@media (max-width: 767px) {
  .form-fields, .composer-state { padding-inline: 16px; }
  .submit { padding: 10px 16px calc(10px + env(safe-area-inset-bottom)); flex-wrap: wrap; }
  .submit .error-text { flex-basis: 100%; }
}
</style>
