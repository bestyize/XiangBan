<script setup lang="ts">
import { computed, nextTick, onScopeDispose, ref } from 'vue'
import { uploadImage } from '@/api/forum'
import AppIcon from '@/components/AppIcon.vue'
import { formatSize } from '@/lib/format'
import { useSession } from '@/stores/session'
import { commentImageId } from './images'

const props = defineProps<{
  limit: number
  markdown?: boolean
  replyTo?: string
  lockedReason?: string
  busy?: boolean
  uploadingExtras?: boolean
  error?: string
}>()
const emit = defineEmits<{ submit: [body: string, imageIds: string[]]; cancelReply: []; close: [] }>()

const emojis = [
  ['😊', '微笑'], ['😂', '笑哭'], ['👍', '点赞'], ['❤️', '爱心'], ['🙌', '举手'], ['🤔', '思考'], ['🎉', '庆祝'],
  ['😄', '大笑'], ['🥹', '感动'], ['🤣', '笑翻'], ['😍', '喜欢'], ['😅', '尴尬'], ['😭', '大哭'], ['😮', '惊讶'],
  ['🤩', '星星眼'], ['😎', '墨镜'], ['🫡', '敬礼'], ['👏', '鼓掌'], ['🙏', '感谢'], ['💪', '加油'], ['🙈', '捂眼'],
  ['🔥', '火'], ['✨', '闪光'], ['👀', '看看'], ['💡', '灵感'], ['🌿', '绿叶'], ['☕', '咖啡'], ['💯', '满分'],
]
const imageTypes = ['image/png', 'image/jpeg', 'image/gif', 'image/webp']
const session = useSession()
const editable = computed(() => session.loggedIn && !props.lockedReason)
const editor = ref<HTMLTextAreaElement | null>(null)
const picker = ref<HTMLInputElement | null>(null)
const loginButton = ref<HTMLButtonElement | null>(null)
const draft = ref('')
const images = ref<{ id: string; url: string; name: string }[]>([])
const uploading = ref(false)
const imageError = ref('')
const expanded = ref(false)
type Panel = 'emoji' | 'gallery' | 'more'
const panel = ref<Panel | null>(null)
let generation = 0

const imageLimit = computed(() =>
  props.markdown || session.info?.limits.comment_images ? (session.info?.limits.image_max_bytes ?? 0) : 0,
)
const body = computed(() => {
  if (!props.markdown) return draft.value.trim()
  const pictures = images.value.map(({ name, url }) => {
    const alt = name.replace(/\.[^.]*$/, '').replace(/[^\p{L}\p{N} _-]/gu, '') || '评论图片'
    return `![${alt}](${url})`
  })
  return [draft.value.trim(), ...pictures].filter(Boolean).join('\n\n')
})
const length = computed(() => Array.from(body.value).length)
const disabled = computed(() => props.busy || uploading.value || props.uploadingExtras)
const ready = computed(() =>
  !disabled.value && length.value <= props.limit && (length.value > 0 || images.value.length > 0),
)

function focus() {
  panel.value = null
  const target = editor.value ?? loginButton.value
  target?.focus({ preventScroll: true })
}

function togglePanel(next: Panel) {
  if (panel.value === next) {
    focus()
    return
  }
  editor.value?.blur()
  panel.value = next
}

function insert(text: string, mention = false) {
  const el = editor.value
  if (!el || props.busy) return
  const { selectionStart: start, selectionEnd: end } = el
  const value = mention && /[\p{L}\p{N}_]$/u.test(draft.value.slice(0, start)) ? ' @' : text
  draft.value = draft.value.slice(0, start) + value + draft.value.slice(end)
  if (mention) focus()
  void nextTick(() => el.setSelectionRange(start + value.length, start + value.length))
}

function reset() {
  ++generation
  draft.value = ''
  images.value = []
  imageError.value = ''
  uploading.value = false
  panel.value = null
  expanded.value = false
}
onScopeDispose(() => { ++generation })
defineExpose({ focus, reset })

async function addImages(files: File[]) {
  if (!files.length || disabled.value || !imageLimit.value) return
  const current = generation, max = imageLimit.value
  imageError.value = ''
  uploading.value = true
  try {
    for (const file of files) {
      if (current !== generation) return
      if (!imageTypes.includes(file.type)) {
        imageError.value = '请选择 PNG、JPEG、GIF 或 WebP 图片。'
        continue
      }
      if (file.size > max) {
        imageError.value = `单张图片最多 ${formatSize(max)}。`
        continue
      }
      try {
        const image = await uploadImage(file)
        if (current !== generation) return
        if (commentImageId(image.url) !== image.id) throw new Error('图片上传结果无效，请重试。')
        if (!images.value.some(item => item.id === image.id)) images.value.push({ ...image, name: file.name })
      } catch (e) {
        if (current === generation) imageError.value = e instanceof Error ? e.message : '上传失败，请重新选图。'
      }
    }
  } finally {
    if (current === generation) {
      uploading.value = false
      if (panel.value === 'gallery' && !imageError.value) panel.value = null
    }
  }
}

function pick(event: Event) {
  const input = event.target as HTMLInputElement
  const files = Array.from(input.files ?? [])
  input.value = ''
  void addImages(files)
}

function paste(event: ClipboardEvent) {
  const files = Array.from(event.clipboardData?.files ?? []).filter(file => file.type.startsWith('image/'))
  if (!files.length || !imageLimit.value) return
  event.preventDefault()
  void addImages(files)
}

function drop(event: DragEvent) {
  const files = Array.from(event.dataTransfer?.files ?? [])
  if (!files.length) return
  event.preventDefault()
  void addImages(files)
}

function submit() {
  if (ready.value) emit('submit', body.value, images.value.map(image => image.id))
}
</script>

<template>
  <div class="composer">
    <header class="composer-head">
      <h2>{{ replyTo ? `回复 ${replyTo}` : '写评论' }}</h2>
      <template v-if="editable">
        <button v-if="replyTo" class="cancel-reply" type="button" :disabled="busy" @click="emit('cancelReply')">取消回复</button>
        <button class="tool" type="button" :aria-label="expanded ? '收起编辑区' : '展开编辑区'" :aria-pressed="expanded" @click="expanded = !expanded">
          <AppIcon :name="expanded ? 'restore' : 'expand'" :size="20" />
        </button>
      </template>
      <button class="tool" type="button" aria-label="关闭评论编辑" @click="emit('close')">
        <AppIcon name="close" :size="21" />
      </button>
    </header>
    <form v-if="editable" @submit.prevent="submit">
      <textarea
        ref="editor"
        v-model="draft"
        autofocus
        aria-label="评论内容"
        placeholder="说点什么…"
        :rows="expanded ? 7 : 2"
        :maxlength="limit * 2"
        :readonly="busy"
        @focus="panel = null"
        @paste="paste"
        @dragover.prevent
        @drop="drop"
      />
      <div v-if="images.length" class="draft-images" aria-label="已添加的图片">
        <div v-for="image in images" :key="image.id">
          <img :src="image.url" :alt="image.name">
          <button type="button" :disabled="busy" :aria-label="`移除图片 ${image.name}`" @click="images = images.filter(item => item.id !== image.id)">
            <AppIcon name="close" :size="16" />
          </button>
        </div>
      </div>
      <p v-if="imageError || error" class="composer-error" role="alert">{{ imageError || error }}</p>
      <p v-if="expanded || length > limit" class="character-count" :class="{ 'composer-error': length > limit }">{{ length }} / {{ limit }}</p>
      <div class="toolbar">
        <button class="tool" type="button" :disabled="busy" :aria-label="panel === 'emoji' ? '切换到键盘' : '打开表情'" :aria-expanded="panel === 'emoji'" @click="togglePanel('emoji')">
          <AppIcon :name="panel === 'emoji' ? 'keyboard' : 'smile'" />
        </button>
        <button v-if="markdown" class="tool" type="button" :disabled="busy" aria-label="插入 @" @click="insert('@', true)">
          <AppIcon name="at" />
        </button>
        <button v-if="imageLimit" class="tool" type="button" :disabled="busy" :aria-label="panel === 'gallery' ? '切换到键盘' : '打开相册'" :aria-expanded="panel === 'gallery'" @click="togglePanel('gallery')">
          <AppIcon :name="panel === 'gallery' ? 'keyboard' : 'image'" />
        </button>
        <button v-if="$slots.more" class="tool" type="button" :disabled="busy" aria-label="更多评论选项" :aria-expanded="panel === 'more'" @click="togglePanel('more')">
          <AppIcon name="plus" />
        </button>
        <button class="publish" type="submit" :disabled="!ready">{{ busy ? '发布中…' : '发布' }}</button>
      </div>
      <div v-if="panel === 'emoji'" class="emoji-panel">
        <p>常用表情</p>
        <div class="emojis">
          <button v-for="[emoji, label] in emojis" :key="label" type="button" :disabled="busy" :aria-label="label" @click="insert(emoji)">{{ emoji }}</button>
        </div>
      </div>
      <input v-if="imageLimit" ref="picker" type="file" hidden :accept="imageTypes.join(',')" multiple @change="pick">
      <div v-if="panel === 'gallery'" class="gallery-panel">
        <button type="button" class="gallery-pick" :disabled="disabled" @click="picker?.click()">{{ uploading ? '上传中…' : '从相册选择' }}</button>
      </div>
      <div v-if="$slots.more" v-show="panel === 'more'" class="more-panel"><slot name="more" /></div>
      <span v-if="uploading" class="sr-only" role="status">正在上传图片</span>
    </form>
    <div v-else class="unavailable">
      <p v-if="lockedReason">{{ lockedReason }}</p>
      <button v-else ref="loginButton" type="button" autofocus class="publish" @click="session.requireLogin()">登录后评论</button>
    </div>
  </div>
</template>

<style scoped>
.composer { padding: 4px 16px 6px; }
.composer-head { display: flex; align-items: center; gap: 4px; margin-inline-end: -6px; }
.composer-head h2 {
  margin: 0 auto 0 0;
  overflow: hidden;
  color: var(--ink);
  font-size: 16px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.cancel-reply {
  flex: none;
  min-height: 36px;
  padding: 0 4px;
  border: 0;
  background: none;
  color: var(--ink-3);
  font-size: 12px;
}
textarea {
  display: block;
  width: 100%;
  max-height: 30dvh;
  border: 1px solid transparent;
  border-radius: 8px;
  padding: 10px 12px;
  resize: none;
  background: var(--surface-2);
  color: var(--ink);
  font-size: 16px;
  line-height: 1.45;
}
textarea:focus { border-color: var(--line-strong); outline-offset: -2px; }
textarea::placeholder { color: var(--ink-3); }
.toolbar { display: flex; align-items: center; gap: 2px; min-height: 54px; margin-inline-start: -6px; }
.tool {
  display: grid;
  flex: none;
  place-items: center;
  width: 44px;
  height: 44px;
  padding: 0;
  border: 0;
  border-radius: 10px;
  background: none;
  color: var(--ink-2);
}
.tool:hover, .tool[aria-expanded="true"] { background: var(--surface-2); }
.tool[aria-expanded="true"] { color: var(--brand); }
.publish {
  min-width: 68px;
  min-height: 44px;
  margin-inline-start: auto;
  padding: 0 16px;
  border: 0;
  border-radius: 22px;
  background: color-mix(in srgb, var(--brand) 13%, var(--surface));
  color: var(--brand);
  font-size: 14px;
  font-weight: 650;
  white-space: nowrap;
}
.publish:disabled { background: var(--surface-2); color: var(--ink-3); }
button:disabled { cursor: default; }
.draft-images { display: flex; gap: 8px; margin-top: 8px; padding: 2px; overflow-x: auto; }
.draft-images > div { position: relative; flex: none; }
.draft-images img { display: block; width: 64px; height: 64px; border-radius: 8px; object-fit: cover; }
.draft-images button {
  position: absolute;
  inset: 2px 2px auto auto;
  display: grid;
  place-items: center;
  width: 30px;
  height: 30px;
  padding: 0;
  border: 0;
  border-radius: 50%;
  background: #0009;
  color: #fff;
}
.composer-error, .character-count { margin: 4px 0 0; color: var(--ink-3); font-size: 12px; }
.character-count { text-align: end; }
.composer-error { color: var(--danger); }
.emoji-panel { padding: 6px 0; }
.emoji-panel p { margin: 0 0 4px; color: var(--ink-3); font-size: 12px; }
.emojis { display: grid; grid-template-columns: repeat(7, minmax(0, 1fr)); }
.emojis button { min-height: 39px; padding: 0; border: 0; border-radius: 8px; background: none; font-size: 22px; }
.emojis button:hover { background: var(--surface-2); }
.emoji-panel, .gallery-panel, .more-panel { border-top: 1px solid var(--line); }
.gallery-panel, .more-panel { padding: 10px 0 6px; }
.gallery-pick { width: 100%; min-height: 44px; border: 0; border-radius: 22px; background: var(--surface-2); }
.unavailable { display: grid; justify-items: center; padding: 10px 16px; color: var(--ink-3); }
.unavailable .publish { margin: 0; }
.unavailable p { margin: 0; }
</style>
