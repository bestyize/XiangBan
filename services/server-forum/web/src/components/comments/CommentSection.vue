<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import type { NewsComment } from '@/api/discover'
import { uploadImage } from '@/api/forum'
import { formatSize } from '@/lib/format'
import { useSession } from '@/stores/session'
import { useNarrow } from '@/lib/viewport'
import PagerNav from '@/components/PagerNav.vue'
import RequestState from '@/components/community/RequestState.vue'
import { useCommunityResource } from '@/components/community/resource'
import CommentRow from './CommentRow.vue'
import CommentEmpty from './CommentEmpty.vue'
import CommentDock from './CommentDock.vue'
import CommentSheet from './CommentSheet.vue'
import { commentImageId } from './images'
import type { CommentAdapter } from './types'

const props = defineProps<{ scope: string; adapter: CommentAdapter; column?: boolean }>()

const session = useSession()
const narrow = useNarrow()
const page = ref(1), body = ref(''), reply = ref<NewsComment | null>(null)
const busy = ref(false), actionError = ref('')
const uploading = ref(false), imageError = ref('')
const images = ref<{ id: string; url: string; name: string }[]>([])
const composerOpen = ref(false), editor = ref<HTMLTextAreaElement | null>(null), root = ref<HTMLElement | null>(null)
let context = 0

const { data, loading, error, load } = useCommunityResource(() => props.adapter.list(page.value))
watch([() => props.scope, () => session.user?.uid, () => session.loggedIn], () => {
  ++context; page.value = 1; body.value = ''; reply.value = null; busy.value = false; actionError.value = ''; composerOpen.value = false
  images.value = []; uploading.value = false; imageError.value = ''
  void load()
}, { immediate: true })
watch(page, load)
watch(narrow, () => { composerOpen.value = false })
onBeforeUnmount(() => { ++context })

const length = computed(() => Array.from(body.value.trim()).length)
const valid = computed(() => (length.value > 0 || images.value.length > 0) && length.value <= 2000)
const imageLimit = computed(() => session.info?.limits.comment_images ? session.info.limits.image_max_bytes : 0)
const names = computed(() => new Map((data.value?.items ?? []).map(item => [item.id, item.author.nickname])))
const total = computed(() => data.value?.total ?? null)
const title = computed(() => reply.value ? `回复 ${reply.value.author.nickname}` : '写评论')

async function openComposer(target: NewsComment | null = null) {
  if (!session.requireLogin()) return
  reply.value = target
  composerOpen.value = true
  await nextTick()
  editor.value?.focus()
}
async function act(work: () => Promise<unknown>, done?: () => void) {
  if (busy.value || !session.requireLogin()) return
  const mine = context
  busy.value = true; actionError.value = ''
  try {
    await work()
    if (mine !== context) return
    done?.()
    if (page.value !== 1) page.value = 1
    else await load()
  } catch (e) { if (mine === context) actionError.value = e instanceof Error ? e.message : '操作失败，请重试' }
  finally { if (mine === context) busy.value = false }
}
function submit() {
  if (!valid.value || uploading.value) return
  const text = body.value.trim(), parent = reply.value?.id
  const imageIds = images.value.map(image => image.id)
  void act(() => props.adapter.post(text, parent, imageIds.length ? imageIds : undefined), () => {
    body.value = ''; reply.value = null; composerOpen.value = false; images.value = []; imageError.value = ''
  })
}

async function selectImages(event: Event) {
  const input = event.target as HTMLInputElement
  const files = Array.from(input.files ?? [])
  input.value = ''
  if (!files.length || !imageLimit.value || uploading.value || busy.value || !session.requireLogin()) return

  const mine = context
  const max = imageLimit.value
  uploading.value = true
  imageError.value = ''
  try {
    for (const file of files) {
      if (mine !== context) return
      if (!['image/png', 'image/jpeg', 'image/gif', 'image/webp'].includes(file.type)) {
        imageError.value = '请选择 PNG、JPEG、GIF 或 WebP 图片。'
        continue
      }
      if (file.size > max) {
        imageError.value = `单张图片最多 ${formatSize(max)}。`
        continue
      }
      try {
        const image = await uploadImage(file)
        if (mine !== context) return
        if (commentImageId(image.url) !== image.id) throw new Error('图片上传结果无效，请重试。')
        if (!images.value.some(item => item.id === image.id)) images.value.push({ ...image, name: file.name })
      } catch (e) {
        if (mine === context) imageError.value = e instanceof Error ? e.message : '上传失败，请重新选图。'
      }
    }
  } finally {
    if (mine === context) uploading.value = false
  }
}
const toggleLike = (item: NewsComment) => void act(() => props.adapter.like(item.id, !item.liked))
function remove(item: NewsComment) {
  if (!window.confirm('删除这条评论？')) return
  void act(() => props.adapter.remove(item.id))
}
function jump() { root.value?.scrollIntoView({ behavior: 'smooth', block: 'start' }) }
defineExpose({ total, openComposer })
</script>

<template>
  <section id="comments" ref="root" class="comments" :class="{ column }">
    <h2>{{ data ? `共 ${data.total} 条评论` : '评论' }}</h2>
    <RequestState :loading="loading" :error="error" @retry="load" />
    <template v-if="data">
      <CommentRow
        v-for="item in data.items"
        :key="item.id"
        :author="item.author"
        :body="item.body"
        :images="item.images"
        :created-at="item.created_at"
        :reply-to-name="item.reply_to ? (names.get(item.reply_to) ?? `评论 #${item.reply_to}`) : undefined"
        :like-count="item.like_count"
        :liked="item.liked"
        :mine="item.mine"
        :busy="busy"
        :nested="!!item.reply_to"
        @like="toggleLike(item)"
        @reply="openComposer(item)"
        @remove="remove(item)"
      />
      <CommentEmpty v-if="!data.items.length" />
      <PagerNav :page="data.page" :page-size="data.page_size" :total="data.total" @go="page = $event" />
    </template>
    <p v-if="actionError && !composerOpen" class="action-error" role="alert">{{ actionError }}</p>

    <CommentSheet :open="composerOpen" :inline="!narrow" :title="title" @close="composerOpen = false">
      <form class="compose" @submit.prevent="submit">
        <template v-if="session.loggedIn">
          <p v-if="reply && !narrow" class="replying">回复 {{ reply.author.nickname }}<button type="button" @click="reply = null">取消</button></p>
          <label class="sr-only" for="comment-body">评论内容</label>
          <textarea id="comment-body" ref="editor" v-model="body" :disabled="busy" rows="3" placeholder="说点什么…" maxlength="4000" />
          <div v-if="images.length" class="draft-images">
            <div v-for="image in images" :key="image.id">
              <img :src="image.url" :alt="image.name">
              <button type="button" :disabled="busy || uploading" :aria-label="`移除${image.name}`" @click="images = images.filter(item => item.id !== image.id)">×</button>
            </div>
          </div>
          <p v-if="imageError || actionError" class="action-error" role="alert">{{ imageError || actionError }}</p>
          <div class="submit">
            <label v-if="imageLimit" class="pick-image" :class="{ disabled: busy || uploading }">
              {{ uploading ? '上传中…' : '添加图片' }}
              <input class="sr-only" type="file" accept="image/png,image/jpeg,image/gif,image/webp" multiple :disabled="busy || uploading" @change="selectImages">
            </label>
            <span>{{ length }} / 2000</span>
            <button type="submit" :disabled="!valid || busy || uploading">{{ busy ? '提交中…' : narrow ? '发送' : '发布评论' }}</button>
          </div>
        </template>
        <button v-else type="button" class="login" @click="session.requireLogin()">登录后评论</button>
      </form>
    </CommentSheet>

    <CommentDock v-if="narrow" :count="total" :hidden="composerOpen" @compose="openComposer(null)" @jump="jump">
      <slot name="actions" />
    </CommentDock>
  </section>
</template>

<style scoped>
.comments { padding: 20px 0 8px; scroll-margin-top: 72px; }
h2 { margin: 0 0 4px; font-size: 15px; font-weight: 600; }
.compose { display: grid; gap: 8px; padding-top: 8px; }
.replying { display: flex; align-items: center; gap: 8px; margin: 0; color: var(--ink-2); font-size: var(--fs-sm); }
.replying button { border: 0; background: none; color: var(--ink-3); font-size: 13px; }
textarea { width: 100%; box-sizing: border-box; resize: none; background: var(--surface-2); border: 0; border-radius: 12px; padding: 10px 12px; min-height: 88px; max-height: 30dvh; color: var(--ink); font-size: 16px; }
.submit { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
.submit span { margin-inline-end: auto; font-size: 12px; color: var(--ink-3); }
.pick-image { display: inline-flex; align-items: center; min-height: 40px; padding-inline: 10px; font-size: 14px; cursor: pointer; }
.pick-image:focus-within { outline: 2px solid var(--brand); border-radius: 8px; }
.pick-image.disabled { opacity: 0.5; cursor: default; }
.draft-images { display: flex; gap: 8px; overflow-x: auto; }
.draft-images > div { position: relative; flex: none; }
.draft-images img { display: block; width: 64px; height: 64px; object-fit: cover; border-radius: 8px; }
.draft-images button { position: absolute; top: 2px; right: 2px; width: 26px; height: 26px; border: 0; border-radius: 50%; background: #0009; color: #fff; }
.submit button, .login { min-height: 40px; min-width: 84px; padding: 0 20px; border: 0; border-radius: 20px; background: var(--brand-fill, var(--brand)); color: #fff; font-size: 14px; }
.submit button:disabled { opacity: 0.45; }
.login { background: color-mix(in srgb, var(--brand) 8%, var(--surface)); color: var(--brand); }
.action-error { margin: 0; color: var(--brand); font-size: 14px; }
@media (max-width: 767px) {
  /* 白底铺满屏幕宽度，抵消外层容器的左右留白（默认是 app-main 在手机社区页的 8px，页面自己有留白时用 --comments-bleed 覆盖）。 */
  .comments { margin: 8px calc(var(--comments-bleed, 8px) * -1) 0; padding: 16px 16px 24px; background: var(--surface); }
  .compose { padding-top: 0; }
}
@media (min-width: 768px) {
  .comments { margin-top: 24px; padding: 16px 24px 12px; border: 1px solid var(--line); border-radius: 14px; background: var(--surface); }
  .comments.column { max-width: 760px; margin-inline: auto; }
}
</style>
