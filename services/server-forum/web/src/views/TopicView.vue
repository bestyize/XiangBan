<script setup lang="ts">
import { categoryDestination } from '@/lib/communityStructure'
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ForumShell from '@/components/ForumShell.vue'
import AttachmentList from '@/components/AttachmentList.vue'
import PosterDialog from '@/components/PosterDialog.vue'
import HelpPanel from '@/components/HelpPanel.vue'
import UserAvatar from '@/components/UserAvatar.vue'
import ReportDialog from '@/components/ReportDialog.vue'
import AppIcon from '@/components/AppIcon.vue'
import ActionSheet from '@/components/ActionSheet.vue'
import ThreadComments from '@/components/comments/ThreadComments.vue'
import DockButton from '@/components/comments/DockButton.vue'
import { useNarrow } from '@/lib/viewport'
import { api, ApiError } from '@/api/forum'
import type { PinScope, PostItem, TopicDetail } from '@/api/types'
import { useSession } from '@/stores/session'
import { useMeta } from '@/stores/meta'
import { useTopicComposer } from '@/stores/topicComposer'
import { useToast } from '@/stores/toast'
import { TRUST_LABEL, formatDateTime, pinScopeLabel, readLevelBadge, tagLabel, timeAgo } from '@/lib/format'
import { useHead } from '@/lib/head'
import { inviteUrl } from '@/lib/invite'

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
const posterOpen = ref(false)
const posterInput = computed(() =>
  topic.value
    ? { tag: topic.value.category.name, title: topic.value.title, summary: topic.value.excerpt || topic.value.title, url: `${window.location.origin}/t/${topic.value.id}`, seed: topic.value.id }
    : null,
)
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
const isMobile = useNarrow()
const moreOpen = ref(false)
function fromMore(action: () => unknown) { moreOpen.value = false; void action() }
function openPoster() { posterOpen.value = true }
function reportTopic() {
  if (topic.value && session.requireLogin()) reportTarget.value = { type: 'topic', id: topic.value.id }
}
watch(isMobile, mobile => { if (!mobile) moreOpen.value = false })
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

const parentCategory = computed(() => {
  const c = topic.value ? meta.byId.get(topic.value.category.id) : undefined
  return c?.parent_id ? meta.byId.get(c.parent_id) : undefined
})

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

async function share() {
  const url = session.user ? inviteUrl(session.user.uid, `/t/${id.value}`) : `${location.origin}/t/${id.value}`
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
  <ForumShell class="topic-shell" :active="topic?.category.slug" :rail="false">
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

    <div v-else-if="loadError" class="panel missing" role="alert"><h1>帖子加载失败</h1><p class="muted">{{ loadError }}</p><button type="button" class="btn" @click="loadTopic">重试</button></div>

    <div v-else-if="!topic" class="panel loading" aria-busy="true">
      <div class="bar w60" />
      <div class="bar w40" />
      <div class="bar" />
      <div class="bar" />
    </div>

    <article v-else class="topic">
      <nav class="crumbs" aria-label="位置">
        <RouterLink v-if="parentCategory" :to="breadcrumbDestination(parentCategory.slug)">{{ parentCategory.name }}</RouterLink>
        <RouterLink :to="breadcrumbDestination(topic.category.slug)">{{ topic.category.name }}</RouterLink>
      </nav>
      <h1 v-if="topic.title || topic.pinned" class="title"><span v-if="topic.pinned" class="pin-badge">{{ pinScopeLabel(topic.pin_scope) }}</span>{{ topic.title }}</h1>
      <div class="byline">
        <div class="byline-top">
          <RouterLink :to="`/u/${topic.author.uid}`" class="who">
            <UserAvatar :name="topic.author.nickname" :src="topic.author.avatar_url" :size="28" />
            <span class="name">{{ topic.author.nickname }}</span>
          </RouterLink>
          <button type="button" class="more-btn" aria-label="更多操作" aria-haspopup="dialog" @click="moreOpen = true">
            <AppIcon name="more" :size="22" />
          </button>
        </div>
        <span class="tl">{{ TRUST_LABEL[topic.author.trust_level] }}</span>
        <time :datetime="topic.created_at" :title="formatDateTime(topic.created_at)">{{ timeAgo(topic.created_at) }}发布</time>
        <span v-if="topic.edited_at">编辑于 {{ timeAgo(topic.edited_at) }}</span>
        <span>{{ topic.view_count }} 次浏览</span>
        <span v-if="topic.min_read_level > 0" class="read-lock">{{ readLevelBadge(topic.min_read_level) }}可见</span>
        <span v-if="topic.locked" class="locked">已锁定</span>
      </div>

      <HelpPanel v-if="topic.help" :help="topic.help" :owner="helpOwner" :busy="helpBusy" :expired="helpExpired" @close="closeHelp" />

      <div v-if="topic.body_html" class="body prose" v-html="topic.body_html" />

      <AttachmentList v-if="topic.attachments.length" :items="topic.attachments" :restricted="topic.min_read_level > 0" />

      <div v-if="topic.tags.length" class="tags">
        <RouterLink v-for="t in topic.tags" :key="t" :to="`/tag/${encodeURIComponent(t)}`" class="tag">#{{ tagLabel(t) }}</RouterLink>
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
        <button type="button" class="btn" @click="posterOpen = true">海报</button>
        <span class="spacer" />
        <template v-if="topic.viewer.can_pin">
          <label class="pin-scope-control">
            <span>置顶范围</span>
            <select v-model="pinScopeDraft" class="select is-sm" aria-label="置顶范围">
              <option value="site">全站</option>
              <option v-if="pinBoard?.section === 'resources'" value="parent">一级板块</option>
              <option v-if="pinBoard?.section !== 'resources' || pinBoard.parent_id" value="category">当前板块</option>
            </select>
          </label>
          <button type="button" class="btn btn-ghost" :aria-pressed="topic.pinned" @click="togglePin">
            {{ pinActionLabel }}
          </button>
        </template>
        <template v-if="topic.viewer.can_edit">
          <button type="button" class="btn btn-ghost" @click="editTopic">编辑</button>
          <button type="button" class="btn btn-ghost btn-danger" @click="removeTopic">删除</button>
        </template>
        <button
          v-else-if="session.user?.uid !== topic.author.uid"
          type="button"
          class="btn btn-ghost"
          @click="reportTopic"
        >
          举报
        </button>
      </div>

      <ThreadComments
        ref="comments"
        :key="topic.id"
        :topic-id="topic.id"
        :count="topic.reply_count"
        :topic-author="topic.author.uid"
        :accepted-id="topic.help?.accepted_post_id"
        :locked-reason="commentRestriction"
        @count-change="changeReplyCount"
      >
        <template #actions="{ narrow }">
          <template v-if="narrow">
            <DockButton label="赞" :text="topic.like_count || '赞'" :active="topic.viewer.liked" @click="toggleLike"><AppIcon name="heart" :size="24" /></DockButton>
            <DockButton label="收藏" :text="topic.viewer.bookmarked ? '已收藏' : '收藏'" :active="topic.viewer.bookmarked" @click="toggleBookmark"><AppIcon name="bookmark" :size="24" /></DockButton>
          </template>
        </template>
        <template #before-post="{ post }">
          <div v-if="topic.help?.accepted_post_id === post.id" class="accepted">✓ 已采纳的回答</div>
        </template>
        <template #after-post="{ post }">
          <div v-if="helpOwner && helpOpen && !post.blocked && post.status === 'normal' && post.author.uid !== topic.author.uid" class="accept-control">
            <button class="btn btn-sm" type="button" :disabled="helpBusy" @click="acceptAnswer(post)">采纳回答{{ topic.help?.bounty ? ` · 发放 ${topic.help.bounty} 积分` : '' }}</button>
          </div>
        </template>
      </ThreadComments>

      <ActionSheet :open="isMobile && moreOpen" label="更多操作" @close="moreOpen = false">
        <button type="button" class="more-row" @click="fromMore(share)">分享</button>
        <button type="button" class="more-row" @click="fromMore(openPoster)">生成海报</button>
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
        <button type="button" class="more-cancel" @click="moreOpen = false">取消</button>
      </ActionSheet>

    </article>

    <ReportDialog :target="reportTarget" @close="reportTarget = null" />
  </ForumShell>
  <PosterDialog :open="posterOpen" :poster="posterInput" :name="topic?.title ?? ''" @close="posterOpen = false" />
</template>

<style scoped src="../styles/forum-post.css"></style>

<style scoped>
.accepted { color: var(--brand); font-size: var(--fs-sm); font-weight: 600; padding-top: 12px; }
.accept-control { display: flex; justify-content: flex-end; padding: 0 0 12px; }
.body { overflow-wrap:anywhere; }
.body :deep(img) { max-width:100%; height:auto; }
.body :deep(pre), .body :deep(table) { max-width:100%; overflow:auto; }
.body :deep(table) { display:block; }
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
.pin-badge {
  display: inline-block;
  margin-right: 8px;
  padding: 0 7px;
  border-radius: 4px;
  background: var(--brand-fill);
  color: var(--brand-ink);
  font-size: var(--fs-sm);
  line-height: 22px;
  vertical-align: 3px;
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
  border: 0;
  background: var(--surface-2);
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
.pin-scope-control {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--ink-3);
  font-size: var(--fs-xs);
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
/* 桌面端：包一层只为手机上把“更多”按钮放到作者行右侧，这里让它不影响原布局。 */
.byline-top {
  display: contents;
}
.more-btn {
  display: none;
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
  .byline-top { display:flex; align-items:center; justify-content:space-between; width:100%; }
  .more-btn { display:grid; place-items:center; width:44px; height:44px; margin:-8px -12px -8px 0; border:0; background:none; color:var(--ink-2); }
  .byline .tl { display:none; }
  /* 分享、海报、举报、编辑等收进“更多”菜单；赞和收藏在底部栏。 */
  .actions { display:none; }
  .tags { margin-bottom:4px; }
  .more-row {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    box-sizing: border-box;
    width: 100%;
    min-height: 54px;
    padding: 0 16px;
    border: 0;
    border-bottom: 1px solid var(--line);
    background: none;
    color: var(--ink);
    font-size: 16px;
  }
  .more-row.danger { color: var(--danger); }
  .more-pin { justify-content: space-between; color: var(--ink-2); }
  .more-pin select { border: 0; background: none; color: var(--ink); font-size: 16px; text-align: right; }
  .more-cancel {
    display: block;
    box-sizing: border-box;
    width: 100%;
    min-height: 54px;
    margin-top: 8px;
    border: 0;
    border-top: 8px solid var(--surface-2);
    background: var(--surface);
    color: var(--ink-2);
    font-size: 16px;
  }
}
</style>
