<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import ForumShell from '@/components/ForumShell.vue'
import TopicList from '@/components/TopicList.vue'
import PagerNav from '@/components/PagerNav.vue'
import UserAvatar from '@/components/UserAvatar.vue'
import { api, ApiError } from '@/api/forum'
import type { Profile, UserReply } from '@/api/types'
import { useSession } from '@/stores/session'
import { useToast } from '@/stores/toast'
import { useTopicList } from '@/lib/useList'
import { TRUST_LABEL, formatDate, timeAgo } from '@/lib/format'
import { useHead } from '@/lib/head'

const route = useRoute()
const session = useSession()
const toast = useToast()
const uid = computed(() => String(route.params.uid))
const tab = ref<'topics' | 'replies'>('topics')

const profile = ref<Profile | null>(null)
const missing = ref(false)
const replies = ref<UserReply[]>([])

useHead(() => ({ title: profile.value?.nickname ?? '用户' }))

const isMe = computed(() => session.user?.uid === uid.value)

watch(
  uid,
  async u => {
    profile.value = null
    missing.value = false
    tab.value = 'topics'
    try {
      profile.value = await api.profile(u)
    } catch (e) {
      if (e instanceof ApiError && e.status === 404) missing.value = true
      else toast.error(e)
    }
  },
  { immediate: true },
)

const topics = useTopicList(p => api.userTopics(uid.value, p), [uid])

watch(tab, async t => {
  if (t === 'replies' && !replies.value.length) {
    try {
      replies.value = (await api.userPosts(uid.value)).items
    } catch (e) {
      toast.error(e)
    }
  }
})
watch(uid, () => (replies.value = []))
</script>

<template>
  <ForumShell :rail="false">
    <div v-if="missing" class="panel missing">
      <h1>找不到这个用户</h1>
      <p class="muted">登录过超级小伴的用户才有主页。</p>
    </div>
    <template v-else-if="profile">
      <header class="panel profile">
        <UserAvatar :name="profile.nickname" :src="profile.avatar_url" :size="64" />
        <div class="who">
          <h1>{{ profile.nickname }}</h1>
          <p class="muted">
            <span class="tl">{{ TRUST_LABEL[profile.trust_level] }}</span>
            {{ formatDate(profile.created_at) }}加入，{{ timeAgo(profile.last_seen_at) }}来过
            <span v-if="profile.banned" class="banned">已封禁</span>
          </p>
        </div>
        <dl class="stats">
          <div><dt>羊毛</dt><dd>{{ profile.deal_count }}</dd></div>
          <div><dt>帖子</dt><dd>{{ profile.topic_count }}</dd></div>
          <div><dt>回复</dt><dd>{{ profile.post_count }}</dd></div>
          <div><dt>积分</dt><dd>{{ profile.points }}</dd></div>
        </dl>
        <RouterLink v-if="isMe" to="/settings" class="btn btn-sm edit">收藏与设置</RouterLink>
      </header>

      <div class="tabs" role="tablist">
        <button type="button" role="tab" :aria-selected="tab === 'topics'" :class="{ on: tab === 'topics' }" @click="tab = 'topics'">帖子</button>
        <button type="button" role="tab" :aria-selected="tab === 'replies'" :class="{ on: tab === 'replies' }" @click="tab = 'replies'">回复</button>
      </div>

      <template v-if="tab === 'topics'">
        <TopicList :items="topics.items.value" :loading="topics.loading.value" empty="还没有发过帖子。" />
        <PagerNav :page="topics.page.value" :page-size="topics.pageSize.value" :total="topics.total.value" @go="topics.go" />
      </template>
      <div v-else class="panel replies">
        <RouterLink v-for="r in replies" :key="r.id" :to="`/t/${r.topic_id}#floor-${r.floor}`" class="reply">
          <span class="rt">{{ r.topic_title }}</span>
          <span class="rx">{{ r.excerpt }}</span>
          <span class="muted rd">{{ timeAgo(r.created_at) }}</span>
        </RouterLink>
        <p v-if="!replies.length" class="muted empty">还没有回复过。</p>
      </div>
    </template>
  </ForumShell>
</template>

<style scoped>
.profile {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  gap: 16px;
  align-items: center;
  padding: 18px 20px;
  margin-bottom: 16px;
  position: relative;
}
h1 {
  margin: 0;
  font-size: var(--fs-xl);
  font-weight: 800;
}
.who p {
  margin: 2px 0 0;
  font-size: var(--fs-xs);
}
.tl {
  display: inline-block;
  border: 1px solid var(--line-strong);
  padding: 0 6px;
  border-radius: 4px;
  font-weight: 700;
  color: var(--ink-2);
  margin-right: 6px;
}
.banned {
  color: var(--danger);
  font-weight: 700;
  margin-left: 6px;
}
.stats {
  display: flex;
  gap: 20px;
  margin: 0;
}
.stats div {
  text-align: center;
}
dt {
  font-size: var(--fs-xs);
  color: var(--ink-3);
}
dd {
  margin: 0;
  font-family: var(--font-display);
  font-size: 20px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}
.edit {
  position: absolute;
  top: 12px;
  right: 12px;
}
.tabs {
  display: flex;
  gap: 4px;
  margin-bottom: 10px;
  border-bottom: 1px solid var(--line);
}
.tabs button {
  border: 0;
  background: none;
  padding: 8px 12px;
  font-size: var(--fs-sm);
  font-weight: 600;
  color: var(--ink-3);
  margin-bottom: -1px;
  border-bottom: 2px solid transparent;
}
.tabs button.on {
  color: var(--ink);
  border-bottom-color: var(--ink);
}
.replies > * + * {
  border-top: 1px solid var(--line);
}
.reply {
  display: grid;
  gap: 2px;
  padding: 12px 16px;
}
.rt {
  font-weight: 600;
  font-size: var(--fs-sm);
}
.rx {
  font-size: var(--fs-sm);
  color: var(--ink-2);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.rd {
  font-size: var(--fs-xs);
}
.reply:hover .rt {
  text-decoration: underline;
}
.empty,
.missing {
  padding: 40px 20px;
  text-align: center;
}
@media (max-width: 767px) {
  .profile {
    grid-template-columns: auto minmax(0, 1fr);
    padding: 16px;
  }
  .stats {
    grid-column: 1 / -1;
    justify-content: space-around;
    padding-top: 12px;
    border-top: 1px solid var(--line);
  }
  .edit {
    position: static;
    grid-column: 1 / -1;
  }
}
</style>
