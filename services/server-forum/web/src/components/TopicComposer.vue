<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref, shallowRef, useTemplateRef, watch } from 'vue'
import ModalDialog from '@/components/ModalDialog.vue'
import MarkdownEditor from '@/components/MarkdownEditor.vue'
import AttachmentUploader from '@/components/AttachmentUploader.vue'
import WorkAttachPicker, { type DraftWebWork } from '@/components/WorkAttachPicker.vue'
import CommunityGameTrial from '@/components/creation/CommunityGameTrial.vue'
import AppIcon from '@/components/AppIcon.vue'
import { api } from '@/api/forum'
import { contentApi } from '@/api/content'
import type { Attachment, TopicInput } from '@/api/types'
import { useMeta } from '@/stores/meta'
import { useSession } from '@/stores/session'
import { useToast } from '@/stores/toast'
import { useTopicComposer } from '@/stores/topicComposer'
import { formatSize, READ_LEVELS, TRUST_LABEL, tagLabel } from '@/lib/format'
import { kindLabel } from '@/lib/content'
import { canonicalCategory, navigationSection } from '@/lib/communityStructure'
import { postingSections, postingSelection, type PostingCategory, type PostingSectionKey } from '@/lib/postingStructure'
import { uploadFile } from '@/lib/upload'
import { readTopicDraft, topicDraftKey, writeTopicDraft, type TopicDraft, type UploadedTopicWork } from '@/lib/topicDraft'

const meta = useMeta()
const session = useSession()
const toast = useToast()
const composer = useTopicComposer()

const editId = computed(() => composer.target?.editId)
const mode = ref<'write' | 'preview'>('write')

const form = reactive({
  sectionKey: 'discuss' as PostingSectionKey,
  boardKey: '',
  title: '',
  body: '',
  tags: [] as string[],
  readLevel: 0,
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
const draftWork = shallowRef<DraftWebWork | UploadedTopicWork | null>(null)
const uploadedWork = shallowRef<{ file: File; key: string } | null>(null)
const topicRequestId = ref<string | null>(null)
const workPickerOpen = ref(false)
const previewWork = ref(false)
const previewWorkRev = ref(0)

const pollEnabled = ref(false)
const pollOptions = ref<string[]>(['', ''])
const pollDays = ref(7)

const boardOpen = ref(false)
const pickerSectionKey = ref<PostingSectionKey>('discuss')
const hoveredBoard = ref<PostingCategory | null>(null)
const visOpen = ref(false)
const confirmLeave = ref(false)

const titleEl = ref<HTMLTextAreaElement | null>(null)
const boardAnchor = ref<HTMLElement | null>(null)
const visAnchor = ref<HTMLElement | null>(null)
const editor = useTemplateRef<InstanceType<typeof MarkdownEditor>>('bodyEditor')
const attUploader = useTemplateRef<InstanceType<typeof AttachmentUploader>>('attUploader')
const uploading = computed(() => attachmentUploading.value || !!editor.value?.uploading)

const tagDraft = ref('')
const busy = ref(false)
const submitStep = ref('')
const error = ref('')

const myLevel = computed(() => session.user?.trust_level ?? 0)
const destinations = computed(() => postingSections(meta.categories, myLevel.value, originalCategoryId.value))
const destination = computed(() => destinations.value.find(s => s.key === form.sectionKey))
const leaf = computed(() => destination.value?.boards.find(c => c.key === form.boardKey && !c.unavailableReason))
const noDestinations = computed(() => !destinations.value.some(section =>
  section.boards.some(board => !board.unavailableReason),
))
const parent = computed(() => meta.byId.get(leaf.value?.parent_id ?? 0))
const isCasual = computed(() => leaf.value?.allow_short_title ?? (leaf.value?.slug === 'casual' || parent.value?.slug === 'casual'))
const presetTags = computed(() => leaf.value?.preset_tags ?? [])
const topicTag = computed(() => leaf.value?.topicTag)
const tags = computed(() => topicTag.value ? [...new Set([topicTag.value, ...form.tags])] : form.tags)
const requestedSlug = computed(() => editId.value ? '' : composer.target?.category ?? '')

const activePickerSection = computed(() =>
  destinations.value.find(s => s.key === pickerSectionKey.value) ?? destination.value ?? destinations.value[0],
)
const inspectedBoard = computed(() => {
  const sec = activePickerSection.value
  if (!sec) return undefined
  if (hoveredBoard.value && sec.boards.some(b => b.key === hoveredBoard.value?.key)) return hoveredBoard.value
  return sec.boards.find(b => b.key === form.boardKey) ?? sec.boards[0]
})

const selectionNote = computed(() => {
  if (selectionTouched.value || !requestedSlug.value) return ''
  const key = canonicalCategory(requestedSlug.value)
  const board = destination.value?.boards.find(candidate => candidate.key === key)
  if (board?.unavailableReason) return `${board.name}（${board.unavailableReason}）`
  return leaf.value ? '' : '请选择当前可发帖的板块'
})

function boardRuleLabel(b?: PostingCategory): string {
  if (!b) return ''
  if (b.unavailableReason) return b.unavailableReason
  if (b.topicTag) return `必带 #${b.topicTag}`
  const p = b.parent_id ? meta.byId.get(b.parent_id) : undefined
  if (b.allow_short_title ?? (b.slug === 'casual' || p?.slug === 'casual')) return '可不写标题'
  if (b.slug === 'resource-movies') return '需合法授权'
  return ''
}

function syncTitleHeight() {
  const el = titleEl.value
  if (!el) return
  el.style.height = 'auto'
  el.style.height = `${el.scrollHeight}px`
}

onMounted(() => {
  void meta.load()
  document.addEventListener('click', onDocumentClick)
})
onBeforeUnmount(() => {
  document.removeEventListener('click', onDocumentClick)
})

function onDocumentClick(e: MouseEvent) {
  const target = e.target as Node | null
  if (boardOpen.value && target && boardAnchor.value && !boardAnchor.value.contains(target)) {
    boardOpen.value = false
  }
  if (visOpen.value && target && visAnchor.value && !visAnchor.value.contains(target)) {
    visOpen.value = false
  }
}

watch(() => composer.visible, async visible => {
  if (!visible) return
  mode.value = 'write'
  boardOpen.value = false
  visOpen.value = false
  workPickerOpen.value = false
  previewWork.value = false
  confirmLeave.value = false
  await nextTick()
  editor.value?.showWrite()
  syncTitleHeight()
})

watch(() => [form.title, mode.value], () => {
  void nextTick(syncTitleHeight)
})

watch(
  [destinations, requestedSlug],
  () => {
    if (editId.value || selectionTouched.value || !meta.categories.length) return
    Object.assign(form, postingSelection(destinations.value, requestedSlug.value, meta.categories, composer.target?.section))
    pickerSectionKey.value = form.sectionKey
  },
  { immediate: true },
)

watch(leaf, (current, previous) => {
  if (!current && previous && form.boardKey === previous.key) form.boardKey = ''
})

function openBoardPicker() {
  if (busy.value || meta.categoriesLoading || !!meta.categoriesError) return
  visOpen.value = false
  hoveredBoard.value = null
  pickerSectionKey.value = destinations.value.some(s => s.key === form.sectionKey)
    ? form.sectionKey
    : (destinations.value[0]?.key ?? 'discuss')
  boardOpen.value = !boardOpen.value
}

function pickBoard(sectionKey: PostingSectionKey, board: PostingCategory) {
  if (board.unavailableReason) return
  selectionTouched.value = true
  form.sectionKey = sectionKey
  form.boardKey = board.key
  boardOpen.value = false
}

function switchMode(next: 'write' | 'preview') {
  mode.value = next
  if (next === 'write') editor.value?.showWrite()
  else editor.value?.showPreview()
}

function snapshot() {
  return JSON.stringify({
    ...form,
    sectionKey: editId.value ? form.sectionKey : undefined,
    boardKey: editId.value ? form.boardKey : undefined,
    tagDraft: tagDraft.value,
    attachments: attachments.value.map(({ id, price }) => [id, price]),
    work: draftWork.value?.source === 'mine'
      ? ['mine', draftWork.value.id]
      : draftWork.value?.source === 'upload'
        ? ['upload', draftWork.value.originalName, draftWork.value.prepared.file.size]
        : draftWork.value?.source === 'uploaded' ? ['uploaded', draftWork.value.key] : null,
    poll: !editId.value && pollEnabled.value ? { options: pollOptions.value, days: pollDays.value } : null,
  })
}
const originalDraft = ref(snapshot())
const draftKey = computed(() => composer.ready && session.user ? topicDraftKey(session.user.uid, editId.value) : null)
let restoredKey: string | null = null
let finishedDraft = false

function uploadedDraft(work: Extract<DraftWebWork, { source: 'upload' }>, key: string): UploadedTopicWork {
  const { prepared } = work
  return {
    source: 'uploaded', key, kind: prepared.kind,
    title: (prepared.name || Array.from(form.title.trim()).slice(0, 60).join('') || '社区作品').trim(),
    summary: (prepared.description || Array.from(form.body.trim()).slice(0, 500).join('')).trim(),
    originalName: work.originalName, format: work.format, size: prepared.file.size, rightsAccepted: work.rightsAccepted,
  }
}

function draftSnapshot(): TopicDraft {
  const work = draftWork.value
  const savedUpload = uploadedWork.value
  return {
    form: { ...form, tags: [...form.tags] }, tagInput: tagDraft.value, attachments: [...attachments.value],
    work: work?.source === 'upload'
      ? (savedUpload?.file === work.prepared.file ? uploadedDraft(work, savedUpload.key) : null)
      : work,
    poll: pollEnabled.value ? { options: [...pollOptions.value], days: pollDays.value } : null,
    requestId: topicRequestId.value,
  }
}

function saveLocalDraft() {
  const key = draftKey.value
  if (!key || restoredKey !== key || finishedDraft) return
  const draft = draftSnapshot()
  writeTopicDraft(key, snapshot() !== originalDraft.value || draft.requestId ? draft : null)
}

function clearLocalDraft() {
  finishedDraft = true
  if (restoredKey && restoredKey === draftKey.value) writeTopicDraft(restoredKey, null)
}

watch([draftKey, originalReady], ([key]) => {
  if (!key || restoredKey === key || (editId.value && !originalReady.value)) return
  const saved = readTopicDraft(key)
  if (saved) {
    Object.assign(form, saved.form)
    selectionTouched.value = !!saved.form.boardKey
    pickerSectionKey.value = form.sectionKey
    tagDraft.value = saved.tagInput ?? ''
    attachments.value = saved.attachments
    draftWork.value = saved.work
    pollEnabled.value = !!saved.poll
    pollOptions.value = saved.poll?.options ?? ['', '']
    pollDays.value = saved.poll?.days ?? 7
    topicRequestId.value = saved.requestId
  }
  restoredKey = key
}, { immediate: true })
watch(draftSnapshot, saveLocalDraft, { deep: true })
onMounted(() => window.addEventListener('pagehide', saveLocalDraft))
onBeforeUnmount(() => window.removeEventListener('pagehide', saveLocalDraft))

function requestClose() {
  if (busy.value || uploading.value) return
  if (previewWork.value) { previewWork.value = false; return }
  if (workPickerOpen.value) { workPickerOpen.value = false; return }
  if (boardOpen.value) { boardOpen.value = false; return }
  if (visOpen.value) { visOpen.value = false; return }
  if (confirmLeave.value) { confirmLeave.value = false; return }
  if (snapshot() !== originalDraft.value) {
    confirmLeave.value = true
    return
  }
  clearLocalDraft()
  composer.close()
}

function confirmDiscard() {
  confirmLeave.value = false
  clearLocalDraft()
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
      originalDraft.value = snapshot()
      originalReady.value = true
      await nextTick()
      syncTitleHeight()
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
function pickReadLevel(level: number) {
  if (readLevelDisabled(level)) return
  form.readLevel = level
  visOpen.value = false
}

const currentReadLevel = computed(() => READ_LEVELS.find(r => r.level === form.readLevel) ?? READ_LEVELS[0])

const titleLen = computed(() => Array.from(form.title.trim()).length)
const bodyLen = computed(() => Array.from(form.body.trim()).length)
const titleHint = computed(() => isCasual.value ? '添加标题（可选）' : '一句话说清楚你想讨论什么')

const cleanPollOptions = computed(() => pollOptions.value.map(o => o.trim()).filter(Boolean))
const pollError = computed(() => {
  if (editId.value || !pollEnabled.value) return ''
  const clean = cleanPollOptions.value
  if (clean.length < 2) return '投票至少需要 2 个有效选项'
  if (clean.some(o => Array.from(o).length > 40)) return '每个选项最多 40 个字'
  if (new Set(clean).size !== clean.length) return '投票选项不能重复'
  return ''
})

const workError = computed(() =>
  draftWork.value && draftWork.value.source !== 'mine' && !draftWork.value.rightsAccepted
    ? '请确认上传作品的原创或授权声明'
    : '',
)

const canSubmit = computed(() =>
  !busy.value && !uploading.value && !meta.categoriesLoading && !meta.categoriesError
  && !!leaf.value && (!editId.value || originalReady.value)
  && (isCasual.value ? titleLen.value <= 80 : titleLen.value >= 4 && titleLen.value <= 80)
  && bodyLen.value >= 10 && bodyLen.value <= 20_000
  && !pollError.value && !workError.value,
)

function addPollOption() {
  if (pollOptions.value.length < 6) pollOptions.value.push('')
}
function removePollOption(index: number) {
  if (pollOptions.value.length > 2) pollOptions.value.splice(index, 1)
}

function workBadgeText(work: DraftWebWork | UploadedTopicWork): string {
  if (work.source === 'mine') return '我的创作'
  if (work.format === 'html') return 'HTML → V2'
  if (work.format === 'htm') return 'HTM → V2'
  return 'V2 容器'
}
function workSubtitle(work: DraftWebWork | UploadedTopicWork): string {
  if (work.source === 'mine') return `${kindLabel(work.kind)} · v${work.version} · V2 容器`
  if (work.source === 'uploaded') return `${work.originalName} · V2 作品包 · ${formatSize(work.size)}`
  if (work.prepared.converted) return `${work.originalName} · 已自动转为 V2 容器 · ${formatSize(work.prepared.file.size)}`
  return `${work.originalName} · ${work.prepared.protocol === 2 ? 'V2 作品包' : kindLabel(work.prepared.kind)} · ${formatSize(work.prepared.file.size)}`
}

function changeWork(work: DraftWebWork | null) {
  draftWork.value = work
  uploadedWork.value = null
}

function workRightsChanged(event: Event) {
  const work = draftWork.value
  if (work && work.source !== 'mine') draftWork.value = { ...work, rightsAccepted: (event.target as HTMLInputElement).checked }
}

async function submit() {
  if (busy.value || uploading.value || !composer.ready) return
  error.value = ''
  if (!leaf.value || meta.categoriesLoading || meta.categoriesError || (editId.value && !originalReady.value)) {
    error.value = '请选择可发帖的板块'
    return
  }
  if (!isCasual.value && titleLen.value < 4) { error.value = '标题至少 4 个字'; return }
  if (titleLen.value > 80) { error.value = '标题最多 80 个字'; return }
  if (bodyLen.value < 10) { error.value = '正文至少 10 个字'; return }
  if (bodyLen.value > 20_000) { error.value = '正文最多 20,000 个字'; return }
  if (pollError.value) { error.value = pollError.value; return }
  if (workError.value) { error.value = workError.value; return }
  if (attachments.value.some(a => !Number.isInteger(a.price) || a.price < 0 || a.price > 10)) {
    error.value = '附件下载积分必须为 0～10 的整数'
    return
  }
  addTag()
  if (tags.value.length > 5) { error.value = '最多 5 个标签，请先移除一个'; return }

  const target = composer.target
  busy.value = true
  submitStep.value = ''
  try {
    let linkedWorkId: number | null = draftWork.value?.source === 'mine' ? draftWork.value.id : null
    let linkedWorkTitle = draftWork.value?.source === 'mine' ? draftWork.value.title : ''
    const selectedWork = draftWork.value
    if (selectedWork && selectedWork.source !== 'mine') {
      let work: UploadedTopicWork
      if (selectedWork.source === 'upload') {
        const { prepared } = selectedWork
        if (uploadedWork.value?.file !== prepared.file) {
          submitStep.value = '上传作品…'
          const key = await uploadFile(prepared.kind, prepared.file, () => {})
          if (composer.target !== target) return
          uploadedWork.value = { file: prepared.file, key }
        }
        work = uploadedDraft(selectedWork, uploadedWork.value.key)
      } else {
        work = selectedWork
      }
      saveLocalDraft()
      submitStep.value = '登记作品…'
      const published = await contentApi.publish({
        kind: work.kind,
        title: work.title,
        summary: work.summary,
        tags: [...tags.value],
        file_key: work.key,
        rights_ack: work.rightsAccepted,
      })
      if (composer.target !== target) return
      linkedWorkId = published.id
      linkedWorkTitle = work.title
      // 已发布即切换为已关联创作，防止后续失败重试时重复创建作品包。
      changeWork({ source: 'mine', id: published.id, title: work.title, kind: work.kind, version: 1 })
      saveLocalDraft()
    }

    let finalBody = form.body.trim()
    if (linkedWorkId && !finalBody.includes(`/packs/${linkedWorkId}`)) {
      const safeLabel = linkedWorkTitle.replace(/[\r\n[\]\\<>`*_!]/g, ' ').trim() || '查看作品'
      finalBody = `[${safeLabel}](${window.location.origin}/packs/${linkedWorkId})\n\n${finalBody}`
    }
    if (Array.from(finalBody).length > 20_000) {
      error.value = '加上作品链接后正文超过 20,000 字，请精简正文'
      return
    }

    submitStep.value = editId.value ? '保存中…' : '发布中…'
    const input: TopicInput = {
      category_id: leaf.value.id,
      title: form.title,
      body: finalBody,
      tags: [...tags.value],
      min_read_level: form.readLevel,
      attachment_ids: attachments.value.map(a => a.id),
      attachment_prices: attachments.value.map(a => ({ id: a.id, price: a.price })),
    }
    if (!editId.value && pollEnabled.value) {
      input.poll = {
        options: [...cleanPollOptions.value],
        days: pollDays.value > 0 ? pollDays.value : undefined,
      }
    }
    if (!target?.editId) input.request_id = topicRequestId.value ??= crypto.randomUUID()
    saveLocalDraft()
    const res = target?.editId ? await api.updateTopic(target.editId, input) : await api.createTopic(input)
    if (composer.target !== target) return
    session.loadSession()
    toast.show(target?.editId ? '修改已保存' : '发布成功', 'info',
      target?.editId ? undefined : { label: '查看帖子', to: `/t/${res.id}` })
    clearLocalDraft()
    composer.complete(res.id)
  } catch (e) {
    error.value = e instanceof Error ? e.message : '发布失败'
  } finally {
    busy.value = false
    submitStep.value = ''
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
  <ModalDialog
    :open="composer.visible"
    :title="editId ? '编辑帖子' : '发帖'"
    :width="820"
    scrollable
    keep-on-backdrop
    hide-header
    :close-disabled="busy || uploading"
    @close="requestClose"
  >
    <div class="composer-shell">
      <div v-if="!composer.ready" class="composer-state" role="status">
        <p>正在确认账号信息…</p>
        <button v-if="!session.restoring" type="button" class="btn btn-sm" @click="session.loadSession()">重新读取</button>
        <button type="button" class="btn btn-sm btn-ghost" @click="composer.close()">关闭</button>
      </div>
      <div v-else-if="blocked" class="composer-state" role="status">
        <p>{{ blocked }}</p>
        <button type="button" class="btn btn-sm" @click="composer.close()">知道了</button>
      </div>
      <div v-else-if="editId && !originalReady" class="composer-state" role="status">
        <p>{{ originalError || '正在读取帖子…' }}</p>
        <div class="state-actions">
          <button v-if="originalError" type="button" class="btn btn-sm" @click="editRetry++">重试</button>
          <button type="button" class="btn btn-sm btn-ghost" @click="composer.close()">关闭</button>
        </div>
      </div>

      <form v-if="!blocked && (!editId || originalReady)" v-show="composer.ready" class="form" novalidate @submit.prevent="submit">
        <header class="c-head">
          <div ref="boardAnchor" class="c-head-l">
            <button
              type="button"
              class="board-btn"
              :disabled="busy || meta.categoriesLoading || !!meta.categoriesError"
              @click="openBoardPicker"
            >
              <span class="dot" />
              <span>{{ leaf ? `${destination?.name ?? ''} · ${leaf.name}` : '选择板块' }}</span>
              <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 9l6 6 6-6" /></svg>
            </button>
            <span v-if="isCasual && leaf" class="rule-badge">标题可选</span>
            <span v-if="topicTag" class="rule-badge">必带 #{{ leaf?.name }}</span>

            <!-- PC Cascading Popover -->
            <div v-if="boardOpen" class="board-pop">
              <div class="pop-secs">
                <button
                  v-for="sec in destinations"
                  :key="sec.key"
                  type="button"
                  class="pop-sec"
                  :class="{ on: sec.key === activePickerSection?.key }"
                  @click="pickerSectionKey = sec.key"
                >
                  <span class="pop-sec-l">
                    <span>{{ sec.name }}</span>
                    <span class="pop-sec-cnt">{{ sec.boards.length }}</span>
                  </span>
                  <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M9 18l6-6-6-6" /></svg>
                </button>
              </div>
              <div v-if="activePickerSection" class="pop-right">
                <div class="pop-boards">
                  <button
                    v-for="b in activePickerSection.boards"
                    :key="b.key"
                    type="button"
                    class="pop-tile"
                    :class="{ on: b.key === form.boardKey }"
                    :disabled="!!b.unavailableReason"
                    @mouseenter="hoveredBoard = b"
                    @click="pickBoard(activePickerSection.key, b)"
                  >
                    <span>{{ b.name }}</span>
                    <i v-if="b.topicTag" class="pop-badge">#</i>
                  </button>
                </div>
                <div v-if="inspectedBoard" class="pop-foot">
                  <small>{{ inspectedBoard.name }}{{ inspectedBoard.description ? `：${inspectedBoard.description}` : '' }}</small>
                  <span v-if="boardRuleLabel(inspectedBoard)" class="rule-badge">{{ boardRuleLabel(inspectedBoard) }}</span>
                </div>
              </div>
            </div>
          </div>

          <div class="c-head-r">
            <div class="mode-seg" role="tablist" aria-label="编辑与预览切换">
              <button type="button" role="tab" :aria-selected="mode === 'write'" :class="{ on: mode === 'write' }" @click="switchMode('write')">编辑</button>
              <button type="button" role="tab" :aria-selected="mode === 'preview'" :class="{ on: mode === 'preview' }" @click="switchMode('preview')">预览</button>
            </div>
            <button type="button" class="c-close" aria-label="关闭" :disabled="busy || uploading" @click="requestClose">
              <AppIcon name="close" :size="16" />
            </button>
          </div>
        </header>

        <div v-show="mode === 'write'" class="c-tbar">
          <button type="button" class="tbtn" title="加粗" @click="editor?.wrap('**', '**', '加粗')">
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M7 5h6a3.5 3.5 0 0 1 0 7H7zM7 12h7a3.5 3.5 0 0 1 0 7H7z" /></svg>
            <span>加粗</span>
          </button>
          <button type="button" class="tbtn" title="链接" @click="editor?.wrap('[', '](https://)', '链接文字')">
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" /><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" /></svg>
            <span>链接</span>
          </button>
          <button type="button" class="tbtn" title="行内代码" @click="editor?.wrap('`', '`', 'code')">
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M16 18l6-6-6-6M8 6l-6 6 6 6" /></svg>
            <span>代码</span>
          </button>
          <button type="button" class="tbtn" title="代码块" @click="editor?.wrap('\n```bash\n', '\n```\n', '')">
            <svg viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="4" width="18" height="16" rx="2.5" /><path d="M9 10l-2 2 2 2M15 10l2 2-2 2M13 9l-2 6" /></svg>
            <span>代码块</span>
          </button>
          <button type="button" class="tbtn" title="列表" @click="editor?.wrap('\n- ', '', '')">
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M9 6h11M9 12h11M9 18h11M4 6h.01M4 12h.01M4 18h.01" /></svg>
            <span>列表</span>
          </button>
          <div class="tdiv" />
          <button type="button" class="tbtn" :class="{ on: !!editor?.uploading }" :disabled="busy" @click="editor?.pickImage()">
            <AppIcon name="image" :size="15" />
            <span>{{ editor?.uploading ? '上传中' : '插图' }}</span>
          </button>
          <button type="button" class="tbtn" :class="{ on: !!draftWork }" :disabled="busy" @click="workPickerOpen = true">
            <AppIcon name="grid" :size="15" />
            <span>作品{{ draftWork ? ' · 已添' : '' }}</span>
          </button>
          <button v-if="!editId" type="button" class="tbtn" :class="{ on: pollEnabled }" :disabled="busy" @click="pollEnabled = !pollEnabled">
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 20h16" /><rect x="6" y="11" width="3" height="6" rx="1" /><rect x="10.5" y="6" width="3" height="11" rx="1" /><rect x="15" y="13" width="3" height="4" rx="1" /></svg>
            <span>投票</span>
          </button>
          <button
            v-if="(session.info?.limits.upload_max_bytes ?? 0) > 0"
            type="button"
            class="tbtn"
            :class="{ on: attachments.length > 0 }"
            :disabled="busy || attachments.length >= 5"
            @click="attUploader?.pick()"
          >
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z" /><path d="M14 3v5h5" /></svg>
            <span>附件{{ attachments.length ? ` · ${attachments.length}` : '' }}</span>
          </button>
        </div>

        <div class="form-fields" :inert="busy">
          <div v-if="meta.categoriesLoading" class="leaf-note" role="status">正在加载板块…</div>
          <div v-else-if="meta.categoriesError" class="leaf-note" role="status">
            <span>{{ meta.categoriesError }}</span>
            <button type="button" class="btn btn-sm" @click="meta.load(true)">重试</button>
          </div>
          <p v-else-if="selectionNote" class="leaf-note" role="status">{{ selectionNote }}</p>
          <p v-else-if="noDestinations" class="leaf-note" role="status">当前没有可发帖的板块</p>

          <div v-show="mode === 'write'" class="title-row">
            <textarea
              ref="titleEl"
              v-model="form.title"
              rows="1"
              class="title-input"
              maxlength="80"
              :placeholder="titleHint"
              @keydown.enter.prevent
              @input="syncTitleHeight"
            />
            <span class="count" :class="{ bad: titleLen > 80 }">{{ titleLen }} / 80</span>
          </div>
          <div v-show="mode === 'write'" class="c-div" />

          <MarkdownEditor
            id="topic-body"
            ref="bodyEditor"
            v-model="form.body"
            label="正文"
            :rows="9"
            frameless
            placeholder="输入正文，支持 Markdown、粘贴或拖入图片（至少 10 字）…"
          />

          <div v-show="mode === 'write'" class="tags-box">
            <div class="tags-line">
              <span v-for="t in tags" :key="t" class="tag" :class="{ required: t === topicTag }">
                # {{ t === topicTag ? leaf?.name : tagLabel(t) }}
                <button v-if="t !== topicTag" type="button" :aria-label="`移除标签 ${t}`" @click="removeTag(t)">×</button>
              </span>
              <input
                v-model="tagDraft"
                class="tag-in"
                :disabled="tags.length >= 5"
                :placeholder="tags.length >= 5 ? '已达 5 个标签上限' : '添加话题标签，回车或逗号确认（最多 5 个）'"
                aria-label="添加标签"
                @keydown="onTagKey"
                @blur="addTag"
              />
            </div>
            <div v-if="presetTags.length" class="presets" role="group" aria-label="这个板块的常用标签">
              <span>常用标签</span>
              <button
                v-for="t in presetTags"
                :key="t"
                type="button"
                class="preset"
                :class="{ on: tags.includes(t) }"
                :disabled="t === topicTag || (!tags.includes(t) && tags.length >= 5)"
                @click="togglePreset(t)"
              >
                {{ tagLabel(t) }}
              </button>
            </div>
          </div>

          <section v-if="!editId && pollEnabled && mode === 'write'" class="fill-card">
            <div class="card-head">
              <span class="card-title">
                <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 20h16" /><rect x="6" y="11" width="3" height="6" rx="1" /><rect x="10.5" y="6" width="3" height="11" rx="1" /><rect x="15" y="13" width="3" height="4" rx="1" /></svg>
                发起投票
              </span>
              <button type="button" class="mini-x" aria-label="移除投票" @click="pollEnabled = false">
                <AppIcon name="close" :size="14" />
              </button>
            </div>
            <div class="poll-grid">
              <div v-for="(_, idx) in pollOptions" :key="idx" class="poll-opt">
                <input v-model="pollOptions[idx]" type="text" maxlength="40" :placeholder="`选项 ${idx + 1}`" />
                <button v-if="pollOptions.length > 2" type="button" class="mini-x" aria-label="删除选项" @click="removePollOption(idx)">
                  <AppIcon name="close" :size="13" />
                </button>
              </div>
            </div>
            <div class="poll-foot">
              <div class="poll-days">
                <button
                  v-for="day in [0, 1, 3, 7, 14, 30]"
                  :key="day"
                  type="button"
                  class="chip"
                  :class="{ on: pollDays === day }"
                  @click="pollDays = day"
                >
                  {{ day === 0 ? '不限时' : `${day} 天` }}
                </button>
              </div>
              <button v-if="pollOptions.length < 6" type="button" class="chip" @click="addPollOption">+ 添加选项</button>
            </div>
            <p class="card-note" :class="{ bad: !!pollError }">{{ pollError || '发布后不能修改投票选项（至少 2 项，每项最多 40 字）' }}</p>
          </section>

          <section v-if="draftWork" class="work-card">
            <div class="work-row">
              <span class="work-ico"><AppIcon name="grid" :size="18" /></span>
              <div class="work-copy">
                <div class="work-title-line">
                  <strong>{{ draftWork.source === 'upload' ? (draftWork.prepared.name || draftWork.originalName) : draftWork.title }}</strong>
                  <span class="work-badge">{{ workBadgeText(draftWork) }}</span>
                </div>
                <small>{{ workSubtitle(draftWork) }}</small>
              </div>
              <div class="work-acts">
                <button
                  v-if="draftWork.source === 'upload' && draftWork.prepared.protocol === 2"
                  type="button"
                  class="work-btn primary"
                  @click="previewWorkRev++; previewWork = true"
                >
                  <AppIcon name="play" :size="11" />
                  <span>试玩</span>
                </button>
                <button v-if="mode === 'write'" type="button" class="work-btn" @click="workPickerOpen = true">更换</button>
                <button v-if="mode === 'write'" type="button" class="mini-x" aria-label="移除作品" @click="changeWork(null)">
                  <AppIcon name="close" :size="14" />
                </button>
              </div>
            </div>
            <label v-if="draftWork.source !== 'mine' && mode === 'write'" class="work-rights" :class="{ need: !draftWork.rightsAccepted }">
              <input :checked="draftWork.rightsAccepted" type="checkbox" @change="workRightsChanged" />
              <span>我确认这是原创或已获授权，并同意按社区规则发布{{ draftWork.rightsAccepted ? '' : '（必选）' }}</span>
            </label>
          </section>

          <div v-if="(session.info?.limits.upload_max_bytes ?? 0) > 0" v-show="mode === 'write' && (attachments.length > 0 || attachmentUploading)">
            <AttachmentUploader
              ref="attUploader"
              v-model="attachments"
              :max-bytes="session.info!.limits.upload_max_bytes"
              :disabled="busy"
              frameless
              @uploading="attachmentUploading = $event"
            />
          </div>
        </div>

        <footer class="c-foot">
          <div class="foot-top">
            <div ref="visAnchor" class="vis-wrap">
              <button type="button" class="vis-btn" @click="visOpen = !visOpen; boardOpen = false">
                <svg viewBox="0 0 24 24" aria-hidden="true"><rect x="5" y="11" width="14" height="10" rx="2" /><path d="M8 11V7a4 4 0 0 1 8 0v4" /></svg>
                <span>{{ currentReadLevel.label }}可见</span>
                <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 9l6 6 6-6" /></svg>
              </button>
              <div v-if="visOpen" class="vis-pop">
                <button
                  v-for="r in READ_LEVELS"
                  :key="r.level"
                  type="button"
                  class="vis-opt"
                  :class="{ on: form.readLevel === r.level }"
                  :disabled="readLevelDisabled(r.level)"
                  @click="pickReadLevel(r.level)"
                >
                  <div>
                    <strong>{{ r.label }}</strong>
                    <small>{{ readLevelDisabled(r.level) ? `需升到「${TRUST_LABEL[r.level]}」` : r.level === 0 ? '所有人与搜索引擎均可浏览' : `${r.short}` }}</small>
                  </div>
                  <span v-if="form.readLevel === r.level" class="vis-cur">当前</span>
                </button>
              </div>
            </div>
            <span class="count" :class="{ bad: bodyLen > 20_000 }">{{ bodyLen }} / 20,000 字</span>
            <p v-if="error" class="foot-msg bad" role="alert">{{ error }}</p>
            <RouterLink v-else to="/rules" class="rules-link" target="_blank" rel="noopener">社区规则</RouterLink>
          </div>
          <div class="foot-acts">
            <button type="button" class="btn-ghost-pill" :disabled="busy || uploading" @click="requestClose">取消</button>
            <button
              type="submit"
              class="btn-brand-pill"
              :disabled="busy || uploading || !canSubmit"
              :title="workError || pollError || ''"
            >
              {{ busy ? (submitStep || '发布中…') : uploading ? '上传中…' : editId ? '保存修改' : '发布' }}
            </button>
          </div>
        </footer>
      </form>

      <!-- H5 Bottom Board Picker Drawer (< 768px) -->
      <div v-if="boardOpen" class="h5-board-mask" @click.self="boardOpen = false">
        <div class="h5-board-sheet" role="dialog" aria-label="选择发布板块">
          <div class="h5-handle" aria-hidden="true" />
          <div class="h5-sheet-head">
            <h3>选择发布板块</h3>
            <button type="button" class="mini-x" aria-label="关闭" @click="boardOpen = false">
              <AppIcon name="close" :size="15" />
            </button>
          </div>
          <div class="seg-track">
            <button
              v-for="sec in destinations"
              :key="sec.key"
              type="button"
              :class="{ on: sec.key === activePickerSection?.key }"
              @click="pickerSectionKey = sec.key"
            >
              {{ sec.name }}<span>{{ sec.boards.length }}</span>
            </button>
          </div>
          <div v-if="activePickerSection" class="h5-board-scroll">
            <div class="h5-board-grid">
              <button
                v-for="b in activePickerSection.boards"
                :key="b.key"
                type="button"
                class="h5-board-tile"
                :class="{ on: b.key === form.boardKey }"
                :disabled="!!b.unavailableReason"
                @click="pickBoard(activePickerSection.key, b)"
              >
                <span>{{ b.name }}</span>
                <i v-if="b.topicTag" class="pop-badge">#</i>
              </button>
            </div>
            <div v-if="inspectedBoard" class="h5-board-desc">
              <div>
                <strong>{{ inspectedBoard.name }}</strong>
                <small>{{ inspectedBoard.description || '选择该板块发布动态' }}</small>
              </div>
              <span v-if="boardRuleLabel(inspectedBoard)" class="rule-badge">{{ boardRuleLabel(inspectedBoard) }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Unified 2-Tab Work Picker -->
      <WorkAttachPicker
        :open="workPickerOpen"
        :current="draftWork?.source === 'uploaded' ? null : draftWork"
        :disabled="busy"
        @close="workPickerOpen = false"
        @select="changeWork"
      />

      <!-- Local V2 Trial Preview Overlay -->
      <div v-if="previewWork && draftWork?.source === 'upload'" class="trial-mask" @click.self="previewWork = false">
        <div class="trial-dialog" role="dialog" aria-label="试玩作品预览">
          <div class="trial-head">
            <strong>试玩预览 · {{ draftWork.prepared.name || draftWork.originalName }}</strong>
            <button type="button" class="btn btn-sm" @click="previewWork = false">返回编辑</button>
          </div>
          <div class="trial-stage">
            <CommunityGameTrial :key="previewWorkRev" :draft="draftWork.prepared" @retry="previewWorkRev++" @exit="previewWork = false" />
          </div>
        </div>
      </div>

      <!-- In-modal Unsaved Confirm Overlay -->
      <div v-if="confirmLeave" class="confirm-mask" @click.self="confirmLeave = false">
        <div class="confirm-box" role="alertdialog" aria-label="放弃未保存的内容？">
          <h3>放弃未保存的内容？</h3>
          <p>关闭后当前编辑的标题、正文及待发布作品不会保留。</p>
          <div class="confirm-acts">
            <button type="button" class="btn-ghost-pill" @click="confirmLeave = false">继续编辑</button>
            <button type="button" class="btn-brand-pill" @click="confirmDiscard">确认放弃</button>
          </div>
        </div>
      </div>
    </div>
  </ModalDialog>
</template>

<style scoped>
.composer-shell {
  position: relative;
  display: flex;
  flex-direction: column;
  min-height: 0;
  height: min(696px, calc(100dvh - 36px));
  background: var(--surface);
}
.composer-state {
  padding: 28px 24px;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 10px;
  color: var(--ink-2);
}
.composer-state p {
  margin: 0;
}
.state-actions {
  display: flex;
  gap: 8px;
}
.form {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}

/* Integrated Header */
.c-head {
  flex: none;
  height: 58px;
  padding: 0 18px 0 22px;
  border-bottom: 1px solid var(--line);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.c-head-l {
  position: relative;
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}
.c-head-r {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: none;
}
.board-btn {
  height: 34px;
  padding: 0 13px;
  border: 0;
  border-radius: 99px;
  background: var(--surface-2);
  color: var(--ink);
  display: inline-flex;
  align-items: center;
  gap: 7px;
  font-size: 13.5px;
  font-weight: 600;
  max-width: 100%;
}
.board-btn:hover:not(:disabled) {
  filter: brightness(0.97);
}
.board-btn .dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--brand);
  flex: none;
}
.board-btn span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.board-btn svg {
  width: 14px;
  height: 14px;
  stroke: var(--ink-3);
  stroke-width: 2;
  fill: none;
  flex: none;
}
.rule-badge {
  height: 22px;
  padding: 0 8px;
  border-radius: 99px;
  background: color-mix(in srgb, var(--brand) 12%, transparent);
  color: var(--brand);
  font-size: 11px;
  font-weight: 600;
  display: inline-flex;
  align-items: center;
  white-space: nowrap;
  flex: none;
}
.mode-seg {
  display: inline-flex;
  padding: 3px;
  border-radius: 99px;
  background: var(--surface-2);
  gap: 2px;
}
.mode-seg button {
  height: 26px;
  padding: 0 12px;
  border: 0;
  border-radius: 99px;
  background: transparent;
  font-size: 12.5px;
  font-weight: 550;
  color: var(--ink-2);
}
.mode-seg button.on {
  background: var(--surface);
  color: var(--ink);
  font-weight: 650;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}
.c-close {
  width: 34px;
  height: 34px;
  border: 0;
  border-radius: 50%;
  background: transparent;
  color: var(--ink-2);
  display: grid;
  place-items: center;
}
.c-close:hover:not(:disabled) {
  background: var(--surface-2);
  color: var(--ink);
}
.c-close:disabled {
  opacity: 0.45;
}

/* PC Cascading Board Popover */
.board-pop {
  position: absolute;
  top: calc(100% + 8px);
  left: 0;
  width: 520px;
  padding: 10px;
  z-index: 25;
  background: var(--surface);
  border-radius: 16px;
  border: 1px solid var(--line-strong);
  box-shadow: 0 18px 46px -10px rgba(0, 0, 0, 0.24);
  display: grid;
  grid-template-columns: 116px 1fr;
  gap: 10px;
  align-items: stretch;
}
.pop-secs {
  border-right: 1px solid var(--line);
  padding-right: 8px;
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.pop-sec {
  width: 100%;
  padding: 8px 9px;
  border: 0;
  border-radius: 10px;
  background: transparent;
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 13px;
  font-weight: 550;
  color: var(--ink-2);
}
.pop-sec:hover {
  background: var(--surface-2);
}
.pop-sec.on {
  background: color-mix(in srgb, var(--brand) 12%, transparent);
  color: var(--brand);
  font-weight: 650;
}
.pop-sec-l {
  display: inline-flex;
  align-items: center;
  gap: 5px;
}
.pop-sec-cnt {
  font-size: 11px;
  opacity: 0.7;
  font-weight: 500;
}
.pop-sec svg {
  width: 13px;
  height: 13px;
  stroke: currentColor;
  fill: none;
  stroke-width: 2;
}
.pop-right {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}
.pop-boards {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 6px;
  align-content: start;
}
.pop-tile {
  height: 36px;
  padding: 0 8px;
  border-radius: 9px;
  background: var(--surface-2);
  color: var(--ink);
  border: 1px solid transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  font-size: 12.5px;
  font-weight: 550;
  min-width: 0;
}
.pop-tile:hover:not(:disabled) {
  filter: brightness(0.97);
}
.pop-tile.on {
  background: color-mix(in srgb, var(--brand) 12%, transparent);
  color: var(--brand);
  border-color: color-mix(in srgb, var(--brand) 36%, transparent);
  font-weight: 650;
}
.pop-tile:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
.pop-tile span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.pop-badge {
  font-style: normal;
  font-size: 10px;
  line-height: 14px;
  padding: 0 4px;
  border-radius: 4px;
  background: var(--surface);
  color: var(--ink-3);
  font-weight: 600;
  flex: none;
}
.pop-foot {
  margin-top: auto;
  padding: 8px 10px;
  border-radius: 10px;
  background: var(--surface-2);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.pop-foot small {
  font-size: 11.5px;
  line-height: 16px;
  color: var(--ink-2);
  flex: 1;
  min-width: 0;
}

/* Single-Row Action Toolbar */
.c-tbar {
  flex: none;
  padding: 7px 22px;
  border-bottom: 1px solid var(--line);
  background: var(--paper);
  display: flex;
  align-items: center;
  gap: 4px;
  overflow-x: auto;
  scrollbar-width: none;
}
.c-tbar::-webkit-scrollbar {
  display: none;
}
.tbtn {
  height: 30px;
  padding: 0 9px;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: var(--ink-2);
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 12.5px;
  font-weight: 550;
  flex: none;
}
.tbtn:hover:not(:disabled) {
  background: var(--surface);
  color: var(--ink);
}
.tbtn.on {
  background: color-mix(in srgb, var(--brand) 12%, transparent);
  color: var(--brand);
  font-weight: 650;
}
.tbtn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
.tbtn svg {
  width: 15px;
  height: 15px;
  stroke: currentColor;
  fill: none;
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
}
.tdiv {
  width: 1px;
  height: 16px;
  background: var(--line-strong);
  margin: 0 4px;
  flex: none;
}

/* Writing Body */
.form-fields {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overscroll-behavior: contain;
  padding: 18px 24px 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.leaf-note {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
  color: var(--ink-3);
  font-size: 13px;
}
.title-row {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  flex: none;
}
.title-input {
  flex: 1;
  min-width: 0;
  width: 100%;
  border: 0;
  outline: none;
  background: transparent;
  padding: 0;
  resize: none;
  overflow: hidden;
  font-size: 22px;
  line-height: 30px;
  font-weight: 700;
  color: var(--ink);
}
.title-input::placeholder {
  color: var(--ink-3);
  font-weight: 600;
}
.count {
  font-size: 12px;
  color: var(--ink-3);
  font-variant-numeric: tabular-nums;
  flex: none;
}
.title-row .count {
  margin-top: 7px;
}
.count.bad {
  color: var(--danger);
  font-weight: 600;
}
.c-div {
  height: 1px;
  background: var(--line);
  flex: none;
}

/* Tags Bar */
.tags-box {
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex: none;
}
.tags-line {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  border-radius: 12px;
  background: var(--surface-2);
}
.tag {
  height: 26px;
  padding: 0 6px 0 9px;
  border-radius: 99px;
  background: var(--surface);
  color: var(--ink);
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  font-weight: 550;
  border: 1px solid var(--line);
}
.tag.required {
  padding-inline: 10px;
  background: color-mix(in srgb, var(--brand) 12%, transparent);
  color: var(--brand);
  border-color: transparent;
  font-weight: 600;
}
.tag button {
  width: 16px;
  height: 16px;
  border: 0;
  border-radius: 50%;
  background: transparent;
  color: var(--ink-3);
  line-height: 1;
  display: grid;
  place-items: center;
}
.tag button:hover {
  background: var(--line);
}
.tag-in {
  flex: 1;
  min-width: 140px;
  height: 26px;
  border: 0;
  outline: none;
  background: transparent;
  font-size: 13px;
  color: var(--ink);
}
.tag-in::placeholder {
  color: var(--ink-3);
}
.presets {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
}
.presets span {
  font-size: 12px;
  color: var(--ink-3);
  margin-right: 2px;
}
.preset {
  height: 26px;
  padding: 0 10px;
  border: 0;
  border-radius: 99px;
  background: var(--surface-2);
  color: var(--ink-2);
  font-size: 12px;
  font-weight: 550;
}
.preset:hover:not(:disabled) {
  filter: brightness(0.96);
}
.preset.on {
  background: color-mix(in srgb, var(--brand) 12%, transparent);
  color: var(--brand);
}
.preset:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

/* Poll & Work Cards */
.fill-card,
.work-card {
  background: var(--surface-2);
  border-radius: 16px;
  padding: 13px 15px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  flex: none;
}
.card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.card-title {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 13.5px;
  font-weight: 650;
  color: var(--ink);
}
.card-title svg {
  width: 15px;
  height: 15px;
  stroke: var(--brand);
  fill: none;
  stroke-width: 1.9;
}
.mini-x {
  width: 26px;
  height: 26px;
  border: 0;
  border-radius: 50%;
  background: transparent;
  color: var(--ink-3);
  display: grid;
  place-items: center;
  flex: none;
}
.mini-x:hover {
  background: var(--line);
  color: var(--ink);
}
.poll-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}
.poll-opt {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 38px;
  padding: 0 8px 0 12px;
  border-radius: 10px;
  background: var(--surface);
  border: 1px solid var(--line);
}
.poll-opt input {
  flex: 1;
  min-width: 0;
  border: 0;
  outline: none;
  background: transparent;
  font-size: 13.5px;
  color: var(--ink);
}
.poll-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 8px;
}
.poll-days {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.chip {
  height: 30px;
  padding: 0 11px;
  border-radius: 9px;
  background: var(--surface);
  border: 1px solid var(--line);
  color: var(--ink-2);
  font-size: 12.5px;
  font-weight: 550;
}
.chip.on {
  background: color-mix(in srgb, var(--brand) 12%, transparent);
  border-color: transparent;
  color: var(--brand);
  font-weight: 650;
}
.card-note {
  margin: 0;
  font-size: 12px;
  color: var(--ink-3);
}
.card-note.bad {
  color: var(--danger);
  font-weight: 550;
}

.work-row {
  display: flex;
  align-items: center;
  gap: 10px;
}
.work-ico {
  width: 34px;
  height: 34px;
  border-radius: 10px;
  background: var(--surface);
  color: var(--brand);
  display: grid;
  place-items: center;
  flex: none;
}
.work-copy {
  flex: 1;
  min-width: 0;
}
.work-title-line {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}
.work-title-line strong {
  font-size: 13.5px;
  font-weight: 650;
  color: var(--ink);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
}
.work-copy small {
  display: block;
  margin-top: 2px;
  font-size: 12px;
  color: var(--ink-2);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.work-badge {
  flex: none;
  padding: 1.5px 6px;
  border-radius: 5px;
  background: color-mix(in srgb, var(--brand) 12%, transparent);
  color: var(--brand);
  font-size: 10.5px;
  font-weight: 650;
  line-height: 14px;
}
.work-acts {
  display: flex;
  align-items: center;
  gap: 4px;
  flex: none;
}
.work-btn {
  height: 28px;
  padding: 0 9px;
  border-radius: 99px;
  background: var(--surface);
  border: 1px solid var(--line);
  color: var(--ink);
  font-size: 12px;
  font-weight: 600;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.work-btn.primary {
  color: var(--brand);
}
.work-rights {
  display: flex;
  align-items: center;
  gap: 7px;
  padding-top: 8px;
  border-top: 1px solid var(--line);
  font-size: 12px;
  color: var(--ink-2);
  cursor: pointer;
}
.work-rights input {
  width: 14px;
  height: 14px;
  flex: none;
  cursor: pointer;
}
.work-rights.need {
  color: var(--danger);
  font-weight: 550;
}

/* Integrated Footer */
.c-foot {
  flex: none;
  padding: 12px 22px;
  border-top: 1px solid var(--line);
  background: var(--surface);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.foot-top {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
  flex: 1;
}
.foot-acts {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: none;
}
.vis-wrap {
  position: relative;
  flex: none;
}
.vis-btn {
  height: 34px;
  padding: 0 12px;
  border: 0;
  border-radius: 99px;
  background: var(--surface-2);
  color: var(--ink-2);
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12.5px;
  font-weight: 550;
}
.vis-btn:hover {
  color: var(--ink);
}
.vis-btn svg {
  width: 14px;
  height: 14px;
  stroke: currentColor;
  fill: none;
  stroke-width: 1.9;
}
.vis-pop {
  position: absolute;
  bottom: calc(100% + 8px);
  left: 0;
  width: 248px;
  padding: 7px;
  z-index: 25;
  background: var(--surface);
  border-radius: 16px;
  border: 1px solid var(--line-strong);
  box-shadow: 0 18px 46px -10px rgba(0, 0, 0, 0.24);
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.vis-opt {
  padding: 8px 10px;
  border: 0;
  border-radius: 10px;
  background: transparent;
  text-align: left;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.vis-opt:hover:not(:disabled) {
  background: var(--surface-2);
}
.vis-opt.on {
  background: color-mix(in srgb, var(--brand) 12%, transparent);
}
.vis-opt:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
.vis-opt strong {
  display: block;
  font-size: 13px;
  font-weight: 600;
  color: var(--ink);
}
.vis-opt.on strong {
  color: var(--brand);
}
.vis-opt small {
  display: block;
  font-size: 11px;
  color: var(--ink-3);
}
.vis-cur {
  font-size: 11px;
  font-weight: 650;
  color: var(--brand);
  flex: none;
}
.foot-msg {
  margin: 0;
  font-size: 12.5px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.foot-msg.bad {
  color: var(--danger);
  font-weight: 550;
}
.rules-link {
  font-size: 12.5px;
  color: var(--ink-3);
  white-space: nowrap;
}
.rules-link:hover {
  color: var(--brand);
}
.btn-ghost-pill {
  height: 38px;
  padding: 0 16px;
  border: 0;
  border-radius: 99px;
  background: transparent;
  color: var(--ink-2);
  font-size: 13.5px;
  font-weight: 550;
}
.btn-ghost-pill:hover:not(:disabled) {
  background: var(--surface-2);
  color: var(--ink);
}
.btn-brand-pill {
  height: 38px;
  padding: 0 22px;
  border: 0;
  border-radius: 99px;
  background: var(--brand);
  color: var(--brand-ink);
  font-size: 13.5px;
  font-weight: 600;
}
.btn-brand-pill:disabled {
  background: var(--surface-2);
  color: var(--ink-3);
}

/* Overlays */
.h5-board-mask {
  display: none;
}
.confirm-mask,
.trial-mask {
  position: absolute;
  inset: 0;
  z-index: 35;
  background: rgba(0, 0, 0, 0.36);
  display: grid;
  place-items: center;
  padding: 18px;
}
.confirm-box {
  width: 100%;
  max-width: 340px;
  border-radius: 18px;
  background: var(--surface);
  padding: 20px;
  box-shadow: var(--shadow);
}
.confirm-box h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 700;
}
.confirm-box p {
  margin: 6px 0 0;
  font-size: 13px;
  color: var(--ink-2);
}
.confirm-acts {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 18px;
}
.trial-dialog {
  width: 100%;
  height: 100%;
  max-width: 780px;
  max-height: 640px;
  border-radius: 18px;
  background: var(--surface);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: var(--shadow);
}
.trial-head {
  flex: none;
  padding: 12px 16px;
  border-bottom: 1px solid var(--line);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}
.trial-stage {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

@media (max-width: 767px) {
  .composer-shell {
    height: min(92dvh, 100dvh - 12px);
  }
  .c-head {
    height: 52px;
    padding: 0 12px 0 14px;
  }
  .board-pop {
    display: none;
  }
  .c-tbar {
    padding: 6px 12px;
  }
  .tbtn {
    height: 34px;
    padding: 0 10px;
    border-radius: 10px;
    background: var(--surface-2);
    font-size: 13px;
  }
  .tdiv {
    display: none;
  }
  .form-fields {
    padding: 14px 16px 18px;
  }
  .title-input {
    font-size: 18px;
    line-height: 25px;
  }
  .tag-in {
    font-size: 16px;
  }
  .poll-grid {
    grid-template-columns: 1fr;
  }
  .poll-opt input {
    font-size: 16px;
  }
  .c-foot {
    padding: 10px 14px calc(12px + env(safe-area-inset-bottom));
    flex-direction: column;
    align-items: stretch;
    gap: 10px;
  }
  .foot-top {
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 8px;
  }
  .foot-msg {
    flex-basis: 100%;
  }
  .foot-acts {
    display: grid;
    grid-template-columns: 1fr 1.6fr;
    gap: 10px;
  }
  .foot-acts .btn-ghost-pill {
    background: var(--surface-2);
    height: 42px;
  }
  .foot-acts .btn-brand-pill {
    height: 42px;
  }

  .h5-board-mask {
    position: absolute;
    inset: 0;
    z-index: 30;
    background: rgba(0, 0, 0, 0.36);
    display: flex;
    align-items: flex-end;
  }
  .h5-board-sheet {
    width: 100%;
    max-height: 78%;
    background: var(--surface);
    border-radius: 20px 20px 0 0;
    padding: 8px 16px calc(18px + env(safe-area-inset-bottom));
    display: flex;
    flex-direction: column;
  }
  .h5-handle {
    width: 36px;
    height: 4px;
    border-radius: 99px;
    background: var(--line-strong);
    margin: 0 auto 10px;
    flex: none;
  }
  .h5-sheet-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 10px;
    flex: none;
  }
  .h5-sheet-head h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 700;
  }
  .seg-track {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    padding: 3px;
    border-radius: 12px;
    background: var(--surface-2);
    gap: 2px;
    margin-bottom: 10px;
    flex: none;
  }
  .seg-track button {
    height: 32px;
    border: 0;
    border-radius: 9px;
    background: transparent;
    font-size: 13px;
    font-weight: 550;
    color: var(--ink-2);
  }
  .seg-track button.on {
    background: var(--surface);
    color: var(--ink);
    font-weight: 650;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
  }
  .seg-track span {
    font-size: 11px;
    opacity: 0.7;
    margin-left: 3px;
  }
  .h5-board-scroll {
    overflow-y: auto;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .h5-board-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 8px;
  }
  .h5-board-tile {
    height: 40px;
    padding: 0 6px;
    border-radius: 10px;
    background: var(--surface-2);
    color: var(--ink);
    border: 1px solid transparent;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    font-size: 13px;
    font-weight: 550;
    min-width: 0;
  }
  .h5-board-tile.on {
    background: color-mix(in srgb, var(--brand) 12%, transparent);
    color: var(--brand);
    border-color: color-mix(in srgb, var(--brand) 36%, transparent);
    font-weight: 650;
  }
  .h5-board-tile:disabled {
    opacity: 0.45;
  }
  .h5-board-tile span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .h5-board-desc {
    padding: 10px 12px;
    border-radius: 12px;
    background: var(--surface-2);
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 8px;
  }
  .h5-board-desc strong {
    display: block;
    font-size: 13px;
    font-weight: 650;
  }
  .h5-board-desc small {
    display: block;
    margin-top: 2px;
    font-size: 12px;
    color: var(--ink-2);
  }
}
</style>
