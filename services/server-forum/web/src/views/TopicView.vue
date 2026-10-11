<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ForumShell from '@/components/ForumShell.vue'
import DealPanel from '@/components/DealPanel.vue'
import PostEntry from '@/components/PostEntry.vue'
import UserAvatar from '@/components/UserAvatar.vue'
import MarkdownEditor from '@/components/MarkdownEditor.vue'
import ReportDialog from '@/components/ReportDialog.vue'
import PagerNav from '@/components/PagerNav.vue'
import { api, ApiError } from '@/api/forum'
import type { PostItem, TopicDetail } from '@/api/types'
import { useSession } from '@/stores/session'
import { useMeta } from '@/stores/meta'
import { useToast } from '@/stores/toast'
import { TRUST_LABEL, formatDateTime, readLevelBadge, timeAgo } from '@/lib/format'
import { useHead } from '@/lib/head'

const route = useRoute()
const router = useRouter()
const session = useSession()
const meta = useMeta()
const toast = useToast()

const id = computed(() => Number(route.params.id))
const topic = ref<TopicDetail | null>(null)
const missing = ref(false)
/** 没有阅读权限时服务端返回的说明，比如「这个帖子仅「老用户」及以上等级可见」。 */
const forbidden = ref('')
const posts = ref<PostItem[]>([])
const postPage = ref(1)
const postTotal = ref(0)
const POST_PAGE = 50

const draft = ref('')
const replyTo = ref<PostItem | null>(null)
const sending = ref(false)
const editor = ref<InstanceType<typeof MarkdownEditor> | null>(null)
const reportTarget = ref<{ type: 'topic' | 'post'; id: number } | null>(null)

useHead(() => ({ title: topic.value?.title ?? '' }))

const parentCategory = computed(() => {
  const c = topic.value ? meta.byId.get(topic.value.category.id) : undefined
  return c?.parent_id ? meta.byId.get(c.parent_id) : undefined
})

async function loadTopic() {
  missing.value = false
  forbidden.value = ''
  try {
    topic.value = await api.topic(id.value)
  } catch (e) {
    if (e instanceof ApiError && e.status === 404) missing.value = true
    else if (e instanceof ApiError && e.status === 403) {
      // 例如退出登录后再看一个需要登录的帖子：把已经载入的内容也清掉。
      forbidden.value = e.message
      topic.value = null
      posts.value = []
    } else toast.error(e)
  }
}

async function loadPosts(page = 1) {
  try {
    const res = await api.posts(id.value, page)
    posts.value = res.items
    postPage.value = res.page
    postTotal.value = res.total
  } catch (e) {
    // 没有阅读权限的提示由帖子本身的请求负责展示，这里不重复弹。
    if (!(e instanceof ApiError && e.status === 403)) toast.error(e)
  }
}

watch(
  id,
  async () => {
    topic.value = null
    posts.value = []
    draft.value = ''
    replyTo.value = null
    // 链接带了楼层锚点时，先翻到那一页。
    const floor = Number(String(route.hash).replace('#floor-', ''))
    const page = floor > 0 ? Math.ceil(floor / POST_PAGE) : 1
    await Promise.all([loadTopic(), loadPosts(page)])
    if (route.hash) {
      await nextTick()
      document.querySelector(route.hash)?.scrollIntoView({ block: 'start' })
    }
  },
  { immediate: true },
)

// 登录状态变化后重新取一次：刷新"是否点过赞、能否编辑"，以及登录后能读到原来没权限的帖子。
watch(
  () => session.user?.uid,
  () => Promise.all([loadTopic(), loadPosts(1)]),
)

async function toggleLike() {
  if (!topic.value || !session.requireLogin()) return
  try {
    const res = await api.likeTopic(topic.value.id, !topic.value.viewer.liked)
    topic.value.viewer.liked = res.liked
    topic.value.like_count = res.like_count
  } catch (e) {
    toast.error(e)
  }
}

async function toggleBookmark() {
  if (!topic.value || !session.requireLogin()) return
  try {
    const res = await api.bookmark(topic.value.id, !topic.value.viewer.bookmarked)
    topic.value.viewer.bookmarked = res.bookmarked
    toast.show(res.bookmarked ? '已收藏，在"收藏与设置"里能找到' : '已取消收藏')
  } catch (e) {
    toast.error(e)
  }
}

async function share() {
  const url = `${location.origin}/t/${id.value}`
  try {
    if (navigator.share) await navigator.share({ title: topic.value?.title, url })
    else {
      await navigator.clipboard.writeText(url)
      toast.show('链接已复制')
    }
  } catch {
    // 用户取消分享
  }
}

async function removeTopic() {
  if (!topic.value || !confirm('删除这个帖子？删除后其他人将看不到它。')) return
  try {
    await api.deleteTopic(topic.value.id)
    toast.show('帖子已删除')
    router.replace('/')
  } catch (e) {
    toast.error(e)
  }
}

function startReply(post: PostItem | null) {
  if (!session.requireLogin()) return
  replyTo.value = post
  document.getElementById('reply-box')?.scrollIntoView({ behavior: 'smooth', block: 'center' })
  editor.value?.focus()
}

function report(target: { type: 'topic' | 'post'; id: number }) {
  if (!session.requireLogin()) return
  reportTarget.value = target
}

async function sendReply() {
  if (!topic.value || draft.value.trim().length < 2) return
  sending.value = true
  try {
    const res = await api.reply(topic.value.id, draft.value, replyTo.value?.id)
    draft.value = ''
    replyTo.value = null
    topic.value.reply_count += 1
    const lastPage = Math.ceil(res.floor / POST_PAGE)
    await loadPosts(lastPage)
    await nextTick()
    document.getElementById(`floor-${res.floor}`)?.scrollIntoView({ behavior: 'smooth', block: 'center' })
  } catch (e) {
    toast.error(e)
  } finally {
    sending.value = false
  }
}

function goPosts(page: number) {
  loadPosts(page)
  document.getElementById('replies')?.scrollIntoView({ behavior: 'smooth' })
}
</script>

<template>
  <ForumShell :active="topic?.category.slug" :rail="false">
    <div v-if="missing" class="panel missing">
      <h1>帖子不存在</h1>
      <p class="muted">可能已经被作者删除，或者因为违反社区规则被隐藏。</p>
      <RouterLink to="/" class="btn">回到首页</RouterLink>
    </div>

    <div v-else-if="forbidden" class="panel missing">
      <h1>没有权限查看</h1>
      <p class="muted">{{ forbidden }}</p>
      <button v-if="!session.loggedIn" type="button" class="btn btn-primary" @click="session.loginOpen = true">登录</button>
      <RouterLink v-else to="/" class="btn">回到首页</RouterLink>
    </div>

    <div v-else-if="!topic" class="panel loading" aria-busy="true">
      <div class="bar w60" />
      <div class="bar w40" />
      <div class="bar" />
      <div class="bar" />
    </div>

    <article v-else class="topic">
      <nav class="crumbs" aria-label="位置">
        <RouterLink v-if="parentCategory" :to="`/c/${parentCategory.slug}`">{{ parentCategory.name }}</RouterLink>
        <RouterLink :to="`/c/${topic.category.slug}`">{{ topic.category.name }}</RouterLink>
      </nav>
      <h1 class="title"><span v-if="topic.pinned" class="pin-badge">置顶</span>{{ topic.title }}</h1>
      <div class="byline">
        <RouterLink :to="`/u/${topic.author.uid}`" class="who">
          <UserAvatar :name="topic.author.nickname" :src="topic.author.avatar_url" :size="28" />
          <span class="name">{{ topic.author.nickname }}</span>
        </RouterLink>
        <span class="tl">{{ TRUST_LABEL[topic.author.trust_level] }}</span>
        <time :datetime="topic.created_at" :title="formatDateTime(topic.created_at)">{{ timeAgo(topic.created_at) }}发布</time>
        <span v-if="topic.edited_at">编辑于 {{ timeAgo(topic.edited_at) }}</span>
        <span>{{ topic.view_count }} 次浏览</span>
        <span v-if="topic.min_read_level > 0" class="read-lock">{{ readLevelBadge(topic.min_read_level) }}可见</span>
        <span v-if="topic.locked" class="locked">已锁定</span>
      </div>

      <DealPanel v-if="topic.deal" :topic="topic" />

      <div v-if="topic.body_html" class="body prose" v-html="topic.body_html" />

      <div v-if="topic.tags.length" class="tags">
        <RouterLink v-for="t in topic.tags" :key="t" :to="`/tag/${encodeURIComponent(t)}`" class="tag">#{{ t }}</RouterLink>
      </div>

      <div class="actions">
        <button type="button" class="btn" :class="{ liked: topic.viewer.liked }" :aria-pressed="topic.viewer.liked" @click="toggleLike">
          <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M8 13.5s-5-3-5-7a2.8 2.8 0 0 1 5-1.7A2.8 2.8 0 0 1 13 6.5c0 4-5 7-5 7z" /></svg>
          {{ topic.like_count ? `赞 ${topic.like_count}` : '赞' }}
        </button>
        <button type="button" class="btn" :aria-pressed="topic.viewer.bookmarked" @click="toggleBookmark">
          {{ topic.viewer.bookmarked ? '已收藏' : '收藏' }}
        </button>
        <button type="button" class="btn" @click="share">分享</button>
        <span class="spacer" />
        <template v-if="topic.viewer.can_edit">
          <RouterLink :to="`/t/${topic.id}/edit`" class="btn btn-ghost">编辑</RouterLink>
          <button type="button" class="btn btn-ghost btn-danger" @click="removeTopic">删除</button>
        </template>
        <button
          v-else-if="session.user?.uid !== topic.author.uid"
          type="button"
          class="btn btn-ghost"
          @click="report({ type: 'topic', id: topic.id })"
        >
          举报
        </button>
      </div>

      <section id="replies" class="replies" aria-labelledby="replies-title">
        <h2 id="replies-title">{{ topic.reply_count ? `${topic.reply_count} 条回复` : '还没有回复' }}</h2>
        <div v-if="posts.length" class="list">
          <PostEntry
            v-for="p in posts"
            :key="p.id"
            :post="p"
            :topic-author="topic.author.uid"
            :can-reply="!topic.locked"
            @reply="startReply"
            @report="post => report({ type: 'post', id: post.id })"
            @removed="topic.reply_count = Math.max(0, topic.reply_count - 1)"
          />
        </div>
        <PagerNav :page="postPage" :page-size="POST_PAGE" :total="postTotal" @go="goPosts" />
      </section>

      <section id="reply-box" class="reply-box" aria-label="回复">
        <p v-if="topic.locked" class="panel note">帖子已锁定，不能再回复。</p>
        <div v-else-if="!session.loggedIn" class="panel note">
          <span>登录后参与讨论，或者告诉大家这条羊毛还能不能领。</span>
          <button type="button" class="btn btn-primary btn-sm" @click="session.loginOpen = true">登录</button>
        </div>
        <form v-else class="compose" @submit.prevent="sendReply">
          <div v-if="replyTo" class="replying">
            回复 #{{ replyTo.floor }} {{ replyTo.author.nickname }}
            <button type="button" class="btn btn-ghost btn-sm" @click="replyTo = null">取消</button>
          </div>
          <MarkdownEditor
            id="reply-editor"
            ref="editor"
            v-model="draft"
            label="回复内容"
            :rows="5"
            placeholder="说点有用的：你领到了吗？用的什么方法？"
          />
          <div class="send">
            <span class="muted">{{ session.user?.trust_level === 0 ? `新人每天可以回复 ${session.info?.limits.daily_posts ?? 10} 次` : '' }}</span>
            <button type="submit" class="btn btn-primary" :disabled="sending || draft.trim().length < 2">
              {{ sending ? '正在发送' : '发表回复' }}
            </button>
          </div>
        </form>
      </section>
    </article>

    <ReportDialog :target="reportTarget" @close="reportTarget = null" />
  </ForumShell>
</template>

<style scoped>
.topic {
  max-width: 820px;
}
.crumbs {
  display: flex;
  gap: 8px;
  font-size: var(--fs-sm);
  color: var(--ink-3);
  margin-bottom: 6px;
}
.crumbs a + a::before {
  content: '/';
  margin-right: 8px;
  color: var(--line-strong);
}
.crumbs a:hover {
  color: var(--ink);
}
.title {
  margin: 0 0 10px;
  font-size: 26px;
  line-height: 1.35;
  font-weight: 800;
  overflow-wrap: anywhere;
}
.pin-badge {
  display: inline-block;
  margin-right: 8px;
  padding: 0 7px;
  border-radius: 4px;
  background: var(--brand);
  color: var(--brand-ink);
  font-size: var(--fs-sm);
  line-height: 22px;
  vertical-align: 3px;
}
.byline {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px 14px;
  font-size: var(--fs-xs);
  color: var(--ink-3);
  margin-bottom: 18px;
}
.who {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}
.name {
  font-size: var(--fs-sm);
  font-weight: 700;
  color: var(--ink);
}
.tl {
  border: 1px solid var(--line-strong);
  padding: 0 6px;
  border-radius: 4px;
  font-weight: 700;
  line-height: 18px;
  color: var(--ink-2);
}
.read-lock {
  border: 1px solid var(--line-strong);
  border-radius: 4px;
  padding: 0 6px;
  line-height: 18px;
  color: var(--ink-2);
  font-weight: 600;
}
.locked {
  color: var(--suspect);
  font-weight: 700;
}
.body {
  padding: 0;
}
.tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 12px;
}
.tag {
  font-size: var(--fs-xs);
  font-weight: 600;
  color: var(--ink-2);
  padding: 2px 9px;
  border-radius: 12px;
  background: var(--surface);
  border: 1px solid var(--line);
}
.tag:hover {
  border-color: var(--line-strong);
  color: var(--ink);
}
.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin: 16px 0 28px;
}
.actions svg {
  width: 15px;
  height: 15px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.5;
}
.actions .liked {
  color: var(--danger);
  border-color: color-mix(in srgb, var(--danger) 40%, var(--line));
}
.actions .liked svg {
  fill: currentColor;
}
.spacer {
  flex: 1;
}
.replies h2 {
  font-size: var(--fs-md);
  margin: 0 0 10px;
}
.list {
  border-top: 1px solid var(--line);
}
.list > :deep(*) {
  border-bottom: 1px solid var(--line);
}
.reply-box {
  margin-top: 18px;
  scroll-margin-top: 90px;
}
.note {
  padding: 16px 18px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  font-size: var(--fs-sm);
  color: var(--ink-2);
  margin: 0;
}
.compose {
  display: grid;
  gap: 10px;
}
.replying {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: var(--fs-sm);
  color: var(--ink-2);
}
.send {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  font-size: var(--fs-xs);
}
.missing {
  padding: 48px 24px;
  text-align: center;
}
.missing h1 {
  margin: 0 0 6px;
  font-size: var(--fs-xl);
}
.missing p {
  margin: 0 0 18px;
}
.loading {
  padding: 24px;
}
.loading .bar {
  height: 16px;
  border-radius: 4px;
  background: var(--surface-2);
  margin-bottom: 12px;
}
.w60 {
  width: 60%;
  height: 26px !important;
}
.w40 {
  width: 40%;
}
@media (max-width: 767px) {
  .title {
    font-size: 21px;
  }
  .note {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
