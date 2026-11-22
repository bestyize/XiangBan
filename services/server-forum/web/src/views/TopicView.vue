<script setup lang="ts">
import { categoryDestination, categoryLabel } from '@/lib/communityStructure'
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AttachmentList from '@/components/AttachmentList.vue'
import HelpPanel from '@/components/HelpPanel.vue'
import DetailLayout from '@/components/community/DetailLayout.vue'
import ReportDialog from '@/components/ReportDialog.vue'
import AppIcon from '@/components/AppIcon.vue'
import ActionSheet from '@/components/ActionSheet.vue'
import ThreadComments from '@/components/comments/ThreadComments.vue'
import DockButton from '@/components/comments/DockButton.vue'
import { useCommunityResource } from '@/components/community/resource'
import { webLink } from '@/components/community/contract'
import { api, ApiError } from '@/api/forum'
import type { PinScope, PostItem, TopicDetail } from '@/api/types'
import { useSession } from '@/stores/session'
import { useMeta } from '@/stores/meta'
import { useTopicComposer } from '@/stores/topicComposer'
import { useToast } from '@/stores/toast'
import { compactCount, formatDateTime, pinScopeLabel, readLevelBadge, tagLabel, timeAgo } from '@/lib/format'
import { useHead } from '@/lib/head'
import { inviteUrl } from '@/lib/invite'
import { copyFull, type SharePayload, type ShareDeletion } from '@/lib/detailShare'

const route = useRoute()
const router = useRouter()
const session = useSession()
const meta = useMeta()
const topicComposer = useTopicComposer()
const toast = useToast()

const id = computed(() => Number(route.params.id))
const topic = ref<TopicDetail | null>(null)
const commentRestriction = computed(() => {
  if (topic.value?.locked) return '帖子已锁定，不能再回复。'
  if (topic.value && session.user && !topic.value.viewer.can_reply) return '当前账号暂不能参与评论。'
})
const pinScopeDraft = ref<Exclude<PinScope, 'none'>>('site')
const pinBoard = computed(() => topic.value && meta.byId.get(topic.value.category.id))
const pinActionLabel = computed(() => {
  if (!topic.value?.pinned) return '置顶'
  return topic.value.pin_scope === pinScopeDraft.value ? '取消置顶' : '更新置顶范围'
})
const missing = ref(false)
const loadError = ref('')
/** 没有阅读权限时服务端返回的说明，比如「这个帖子仅「老用户」及以上等级可见」。 */
const forbidden = ref('')
const comments = ref<InstanceType<typeof ThreadComments> | null>(null)
const reportTarget = ref<{ type: 'topic'; id: number } | null>(null)
const moreOpen = ref(false)
function fromMore(action: () => unknown) { moreOpen.value = false; void action() }
function reportTopic() {
  if (topic.value && session.requireLogin()) reportTarget.value = { type: 'topic', id: topic.value.id }
}
const helpBusy = ref(false)
const helpOwner = computed(() => session.user?.uid === topic.value?.author.uid)
const helpExpired = ref(false)
watch(() => topic.value?.help, (help, _, cleanup) => {
  const refresh = () => { helpExpired.value = !!help && new Date(help.expires_at).getTime() <= Date.now() }
  refresh()
  if (help?.status === 'open' && !helpExpired.value) {
    const timer = window.setInterval(() => {
      refresh()
      if (helpExpired.value) window.clearInterval(timer)
    }, 1000)
    cleanup(() => window.clearInterval(timer))
  }
}, { immediate: true })
const helpOpen = computed(() => topic.value?.help?.status === 'open' && !helpExpired.value)

useHead(() => ({ title: topic.value?.title || topic.value?.excerpt || '社区分享' }))

watch(topic, current => {
  if (current && current.pin_scope !== 'none') pinScopeDraft.value = current.pin_scope
})

const authorId = computed(() => topic.value?.author.uid)
const authorState = useCommunityResource(async () => {
  const uid = authorId.value
  if (!uid || !session.user || session.user.uid === uid) return null
  return api.profile(uid)
})
watch([authorId, () => session.user?.uid], authorState.load, { immediate: true })
const followBusy = ref(false)
async function toggleFollow() {
  const uid = authorId.value, reader = session.user?.uid
  if (!uid || uid === reader || followBusy.value || !session.requireLogin() || !reader) return
  followBusy.value = true
  try {
    if (!authorState.data.value) await authorState.load()
    const author = authorState.data.value
    if (uid !== authorId.value || reader !== session.user?.uid) return
    if (!author) throw new Error(authorState.error.value || '暂时无法读取关注状态')
    if (author.blocked || (author.banned && !author.following)) return
    const result = await api.follow(uid, !author.following)
    if (authorState.data.value === author && reader === session.user?.uid) {
      authorState.data.value = { ...author, ...result }
    }
  } catch (error) {
    if (uid === authorId.value && reader === session.user?.uid) toast.error(error)
  } finally { followBusy.value = false }
}

const content = computed(() => {
  const body = topic.value?.body_html ?? ''
  const gallery = [...new Set((topic.value?.images ?? []).map(webLink).filter((url): url is string => !!url))]
  if (!gallery.length || !body) return { gallery, body }
  const template = document.createElement('template')
  template.innerHTML = body
  const absolute = (url: string) => new URL(url, location.origin).href
  const galleryUrls = new Set(gallery.map(absolute))
  for (const image of template.content.querySelectorAll<HTMLImageElement>('img[src]')) {
    const src = webLink(image.getAttribute('src'))
    if (!src || !galleryUrls.has(absolute(src))) continue
    const link = image.closest('a[href]')
    const destination = webLink(link?.getAttribute('href'))
    if (link && destination && absolute(destination) !== absolute(src) && !link.textContent?.trim()) {
      link.append(image.alt || '图片链接')
    }
    let parent = image.parentElement
    image.remove()
    while (parent?.matches('p, a') && !parent.children.length && !parent.textContent?.trim()) {
      const previous = parent
      parent = parent.parentElement
      previous.remove()
    }
  }
  return { gallery, body: template.innerHTML }
})
const disliked = ref(false)
function toggleDislike() {
  disliked.value = !disliked.value
  toast.show(disliked.value ? '已标为不喜欢' : '已取消标记')
}

function goBack() {
  if (window.history.state?.back) router.back()
  else void router.push(topic.value ? breadcrumbDestination(topic.value.category.slug) : '/discuss')
}

let topicRequest = 0
async function loadTopic() {
  const current = ++topicRequest, topicId = id.value
  missing.value = false
  forbidden.value = ''
  loadError.value = ''
  try {
    const result = await api.topic(topicId)
    if (current === topicRequest && topicId === id.value) topic.value = result
  } catch (e) {
    if (current !== topicRequest || topicId !== id.value) return
    topic.value = null
    if (e instanceof ApiError && e.status === 404) missing.value = true
    else if (e instanceof ApiError && e.status === 403) forbidden.value = e.message
    else loadError.value = e instanceof Error ? e.message : '加载失败'
  }
}

function editTopic() {
  if (!topic.value?.viewer.can_edit) return
  moreOpen.value = false
  topicComposer.open({ editId: topic.value.id })
}

watch(() => topicComposer.saved, saved => {
  if (saved?.id === id.value) void loadTopic()
})

watch(id, () => {
  topic.value = null
  disliked.value = false
  moreOpen.value = false
  reportTarget.value = null
  void loadTopic()
}, { immediate: true })

// 评论入口与通知中的楼层链接定位到详情页内的评论。
watch([() => topic.value?.id, () => route.hash, () => route.query.compose], ([topicId, hash, compose], previous) => {
  if (!topicId || topicId !== id.value || !comments.value) return
  if (!compose && previous[2] && topicId === previous[0] && hash === previous[1]) return
  const floorNumber = Number(/^#floor-([1-9]\d*)$/.exec(hash)?.[1])
  const floor = Number.isSafeInteger(floorNumber) ? floorNumber : undefined
  if (floor) void comments.value.goToFloor(floor)
  else if (hash === '#replies') comments.value.jumpToComments()
  if (compose) {
    void comments.value.startReply()
    const { compose: _compose, ...query } = route.query
    void router.replace({ query, hash })
  }
}, { flush: 'post' })

watch(() => session.user?.uid, loadTopic)

async function toggleLike() {
  const current = topic.value
  if (!current || !session.requireLogin()) return
  try {
    const res = await api.likeTopic(current.id, !current.viewer.liked)
    if (topic.value === current) {
      current.viewer.liked = res.liked
      current.like_count = res.like_count
    }
  } catch (e) {
    toast.error(e)
  }
}

async function toggleBookmark() {
  const current = topic.value
  if (!current || !session.requireLogin()) return
  try {
    const res = await api.bookmark(current.id, !current.viewer.bookmarked)
    if (topic.value !== current) return
    current.viewer.bookmarked = res.bookmarked
    toast.show(res.bookmarked ? '已收藏，在"收藏与设置"里能找到' : '已取消收藏')
  } catch (e) {
    toast.error(e)
  }
}

async function togglePin() {
  if (!topic.value) return
  const shouldPin = !topic.value.pinned || topic.value.pin_scope !== pinScopeDraft.value
  try {
    const res = await api.pinTopic(topic.value.id, shouldPin, pinScopeDraft.value)
    topic.value.pinned = res.pinned
    topic.value.pin_scope = res.pin_scope
    toast.show(res.pinned ? `已设置为${pinScopeLabel(res.pin_scope)}` : '已取消置顶')
  } catch (e) {
    toast.error(e)
  }
}

const shareUrl = computed(() =>
  session.user ? inviteUrl(session.user.uid, `/t/${id.value}`) : `${location.origin}/t/${id.value}`,
)

const share = computed<SharePayload | undefined>(() => {
  const post = topic.value
  if (!post || post.status !== 'normal') return
  const linkOnly = post.min_read_level > 0
  return {
    title: linkOnly ? '社区帖子' : post.title || post.excerpt || '分享生活',
    url: shareUrl.value,
    badge: '社区帖子',
    author: linkOnly ? undefined : post.author.nickname,
    meta: linkOnly ? '需要阅读权限' : [post.author.nickname, timeAgo(post.created_at), categoryLabel(post.category)].filter(Boolean).join(' · '),
    excerpt: linkOnly ? undefined : post.excerpt,
    extra: !linkOnly && post.body_html ? copyFull(post.title, post.body_html) : undefined,
    linkOnly,
  }
})

const shareDeletion = computed<ShareDeletion | undefined>(() => {
  const post = topic.value
  if (!post?.viewer.can_edit) return
  return {
    label: '删除帖子',
    confirm: '删除这条帖子？',
    async run() {
      await api.deleteTopic(post.id)
      toast.show('帖子已删除')
      if (topic.value?.id === post.id) await router.replace('/')
    },
  }
})

async function removeTopic() {
  const action = shareDeletion.value
  if (!action || !confirm('删除这个帖子？删除后其他人将看不到它。')) return
  try { await action.run() } catch (e) { toast.error(e) }
}

function changeReplyCount(delta: number) {
  if (topic.value) topic.value.reply_count = Math.max(0, topic.value.reply_count + delta)
}

async function acceptAnswer(post: PostItem) {
  if (!topic.value || helpBusy.value || !window.confirm(`采纳「${post.author.nickname}」的回答？悬赏将发放给对方，采纳后不能撤销。`)) return
  helpBusy.value = true
  try {
    await api.acceptHelp(topic.value.id, post.id)
    toast.show('已采纳，求助标记为已解决')
    await Promise.all([loadTopic(), session.loadSession()])
  } catch (e) { toast.error(e) }
  finally { helpBusy.value = false }
}

async function closeHelp() {
  if (!topic.value || helpBusy.value || !window.confirm('关闭这次求助？尚未发放的悬赏积分将退还。')) return
  helpBusy.value = true
  try {
    const result = await api.closeHelp(topic.value.id)
    toast.show(result.refunded ? `已退还 ${result.refunded} 积分` : '求助已结束')
    await Promise.all([loadTopic(), session.loadSession()])
  } catch (e) { toast.error(e) }
  finally { helpBusy.value = false }
}

function breadcrumbDestination(slug: string) {
  const category=meta.bySlug.get(slug)
  return category ? categoryDestination(category,meta.categories) : `/c/${slug}`
}

</script>

<template>
  <DetailLayout :key="id" :ready="!!topic" :title="topic?.title" :author="topic?.author.nickname"
    :author-image="topic?.author.avatar_url" :author-to="topic ? '/u/' + topic.author.uid : undefined"
    :images="content.gallery" :badge="topic ? categoryLabel(topic.category) : undefined"
    :share="share" :share-deletion="shareDeletion" @back="goBack">
    <template #state>
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
      <div v-else-if="loadError" class="panel missing" role="alert">
        <h1>帖子加载失败</h1><p class="muted">{{ loadError }}</p>
        <button type="button" class="btn" @click="loadTopic">重试</button>
      </div>
      <div v-else-if="!topic" class="panel loading" aria-busy="true">
        <div class="bar w60" /><div class="bar w40" /><div class="bar" /><div class="bar" />
      </div>
    </template>
    <template #actions>
      <button v-if="topic && session.user?.uid !== topic.author.uid && !authorState.data.value?.blocked &&
        (!authorState.data.value?.banned || authorState.data.value.following)"
        class="xhs-follow-btn" :class="{ 'is-followed': authorState.data.value?.following }"
        type="button" :aria-pressed="!!authorState.data.value?.following"
        :disabled="followBusy || authorState.loading.value || (session.loggedIn && !session.user)" @click="toggleFollow">
        {{ authorState.data.value?.following ? '已关注' : '关注' }}
      </button>
    </template>
    <template #title>
      <h1 v-if="topic && (topic.title || topic.pinned)" class="xhs-title">
        <span v-if="topic.pinned" class="topic-badge">{{ pinScopeLabel(topic.pin_scope) }}</span>{{ topic.title }}
      </h1>
    </template>
    <template v-if="topic" #default>
      <HelpPanel v-if="topic.help" :help="topic.help" :owner="helpOwner" :busy="helpBusy" :expired="helpExpired" @close="closeHelp" />
      <div v-if="content.body" class="prose xhs-prose" v-html="content.body" />
      <AttachmentList v-if="topic.attachments.length" :items="topic.attachments" :restricted="topic.min_read_level > 0" />
      <div class="xhs-hashtags">
        <RouterLink class="xhs-hashtag" :to="breadcrumbDestination(topic.category.slug)">#{{ categoryLabel(topic.category) }}</RouterLink>
        <RouterLink v-for="tag in topic.tags.filter(tag => tagLabel(tag) !== categoryLabel(topic!.category))" :key="tag"
          class="xhs-hashtag" :to="'/tag/' + encodeURIComponent(tag)">#{{ tagLabel(tag) }}</RouterLink>
      </div>
      <div class="xhs-meta-row">
        <div class="topic-meta-text">
          <time :datetime="topic.edited_at || topic.created_at"
            :title="formatDateTime(topic.edited_at || topic.created_at) + ' · ' + topic.view_count + ' 次浏览'">
            {{ topic.edited_at ? '编辑于' : '发布于' }} {{ timeAgo(topic.edited_at || topic.created_at) }} 社区
          </time>
          <span v-if="topic.min_read_level > 0" class="topic-state">{{ readLevelBadge(topic.min_read_level) }}可见</span>
          <span v-if="topic.locked" class="topic-state">已锁定</span>
        </div>
        <div class="topic-feedback">
          <button class="xhs-dislike-pill" type="button" :aria-pressed="disliked" @click="toggleDislike">不喜欢</button>
          <button class="h5-icon-btn topic-more" type="button" aria-label="更多操作" aria-haspopup="dialog" @click="moreOpen = true">
            <AppIcon name="more" :size="16" />
          </button>
        </div>
      </div>
    </template>
    <template v-if="topic" #comments>
      <ThreadComments ref="comments" :key="topic.id" detail :topic-id="topic.id" :count="topic.reply_count"
        :topic-author="topic.author.uid" :accepted-id="topic.help?.accepted_post_id"
        :locked-reason="commentRestriction" @count-change="changeReplyCount">
        <template #actions>
          <DockButton detail :class="{ 'is-liked': topic.viewer.liked }" label="赞" :text="compactCount(topic.like_count)" :active="topic.viewer.liked" @click="toggleLike">
            <AppIcon name="heart" :size="19" />
          </DockButton>
          <DockButton detail :class="{ 'is-saved': topic.viewer.bookmarked }" :label="topic.viewer.bookmarked ? '取消收藏' : '收藏'"
            :text="topic.viewer.bookmarked ? '已收藏' : '收藏'" :active="topic.viewer.bookmarked" @click="toggleBookmark">
            <AppIcon name="star" :size="19" />
          </DockButton>
        </template>
        <template #before-post="{ post }">
          <div v-if="topic.help?.accepted_post_id === post.id" class="accepted">✓ 已采纳的回答</div>
        </template>
        <template #after-post="{ post }">
          <div v-if="helpOwner && helpOpen && !post.blocked && post.status === 'normal' && post.author.uid !== topic.author.uid" class="accept-control">
            <button class="btn btn-sm" type="button" :disabled="helpBusy" @click="acceptAnswer(post)">采纳回答{{ topic.help?.bounty ? ' · 发放 ' + topic.help.bounty + ' 积分' : '' }}</button>
          </div>
        </template>
      </ThreadComments>
    </template>
    <template #dialogs>
      <ActionSheet v-if="topic" :open="moreOpen" label="更多操作" @close="moreOpen = false">
        <div class="topic-more-actions">
          <template v-if="topic.viewer.can_pin">
            <label class="more-row more-pin">
              <span>置顶范围</span>
              <select v-model="pinScopeDraft" aria-label="置顶范围">
                <option value="site">全站</option>
                <option v-if="pinBoard?.section === 'resources'" value="parent">一级板块</option>
                <option v-if="pinBoard?.section !== 'resources' || pinBoard.parent_id" value="category">当前板块</option>
              </select>
            </label>
            <button type="button" class="more-row" @click="fromMore(togglePin)">{{ pinActionLabel }}</button>
          </template>
          <template v-if="topic.viewer.can_edit">
            <button type="button" class="more-row" @click="editTopic">编辑</button>
            <button type="button" class="more-row danger" @click="fromMore(removeTopic)">删除</button>
          </template>
          <button v-else-if="session.user?.uid !== topic.author.uid" type="button" class="more-row" @click="fromMore(reportTopic)">举报</button>
          <button type="button" class="more-row more-cancel" @click="moreOpen = false">取消</button>
        </div>
      </ActionSheet>

      <ReportDialog :target="reportTarget" @close="reportTarget = null" />
    </template>
  </DetailLayout>
</template>

<style scoped>
.topic-badge { display: inline-block; margin-inline-end: 8px; padding: 0 7px; border-radius: 4px; background: var(--brand-fill); color: var(--brand-ink); font-size: 12px; line-height: 22px; vertical-align: 2px; }
.xhs-prose :deep(img) { max-width: 100%; height: auto; object-fit: contain; }
.xhs-prose :deep(pre), .xhs-prose :deep(table) { max-width: 100%; overflow: auto; }
.xhs-prose :deep(table) { display: block; }
.xhs-hashtag:hover { text-decoration: underline; }
.topic-meta-text { display: flex; flex-wrap: wrap; align-items: center; gap: 4px 8px; min-width: 0; }
.topic-state { padding: 0 5px; border: 1px solid var(--bd); border-radius: 5px; }
.topic-feedback { display: flex; align-items: center; }
.xhs-dislike-pill[aria-pressed="true"] { border-color: var(--bd-strong); background: var(--bg-subtle); }
.topic-more { width: 24px; height: 24px; color: var(--ink-3); }
.accepted { padding-top: 8px; color: var(--brand); font-size: 12px; font-weight: 600; }
.accept-control { display: flex; justify-content: flex-end; }
.missing { padding: 48px 24px; text-align: center; }
.missing h1 { margin: 0 0 6px; font-size: 22px; }
.missing p { margin: 0 0 18px; }
.loading { padding: 24px; }
.bar { height: 16px; margin-bottom: 12px; border-radius: 4px; background: var(--surface-2); }
.w60 { width: 60%; height: 26px; }
.w40 { width: 40%; }
.more-row { display: flex; align-items: center; justify-content: center; gap: 12px; width: 100%; min-height: 54px; padding: 0 16px; border: 0; border-bottom: 1px solid var(--community-divider); background: transparent; color: var(--ink); font-size: 16px; }
.more-row.danger { color: var(--danger); }
.more-pin { justify-content: space-between; color: var(--ink-2); }
.more-pin select { border: 0; background: transparent; text-align: right; }
.more-cancel { border-bottom: 0; color: var(--ink-2); }
@media (width >= 960px) {
  :global(.action-sheet:has(.topic-more-actions)) { inset: auto max(0px, calc((100% - 400px) / 2)) 24px; border-radius: 16px; }
}
</style>
