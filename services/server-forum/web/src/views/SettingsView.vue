<script setup lang="ts">
import { onMounted, ref } from 'vue'
import TopicList from '@/components/TopicList.vue'
import PagerNav from '@/components/PagerNav.vue'
import { api } from '@/api/forum'
import { useSession } from '@/stores/session'
import { useTopicList } from '@/lib/useList'
import { TRUST_LABEL } from '@/lib/format'
import { useHead } from '@/lib/head'
import AppearanceOptions from '@/components/AppearanceOptions.vue'
import AvatarEditor from '@/components/AvatarEditor.vue'
import UserAvatar from '@/components/UserAvatar.vue'
import { analyticsEnabled, setAnalyticsEnabled } from '@/lib/analytics'

useHead(() => ({ title: '收藏与设置' }))
const session = useSession()

const bookmarks = useTopicList(p => api.bookmarks(p), [() => session.user?.uid])

const statsOn = ref(analyticsEnabled())
function toggleStats() {
  statsOn.value = !statsOn.value
  setAnalyticsEnabled(statsOn.value)
}

const blocked = ref<{ uid: string; nickname: string }[]>([])
async function loadBlocks() {
  try {
    blocked.value = (await api.myBlocks()).items
  } catch {
    blocked.value = []
  }
}
async function unblock(uid: string) {
  await api.block(uid, false)
  await loadBlocks()
}
onMounted(loadBlocks)

const avatarOpen = ref(false)
const avatarPending = ref(false)
const avatarEditor = ref<InstanceType<typeof AvatarEditor> | null>(null)
</script>

<template>
  <div class="settings">
    <h1>我的设置</h1>
    <section class="block">
      <h2>积分与签到</h2>
      <div class="panel box">
        <p>当前积分 <strong>{{ session.user?.points ?? 0 }}</strong></p>
        <RouterLink to="/points" class="btn btn-sm">签到、邀请和积分明细</RouterLink>
      </div>
    </section>

    <section class="block" aria-labelledby="appearance-title">
      <h2 id="appearance-title">外观</h2>
      <AppearanceOptions />
    </section>

    <section class="block" aria-labelledby="bm-title">
      <h2 id="bm-title">我的收藏</h2>
      <TopicList :items="bookmarks.items.value" :loading="bookmarks.loading.value" empty="还没有收藏。看到有用的帖子，点帖子下方的收藏就能存到这里。" />
      <PagerNav :page="bookmarks.page.value" :page-size="bookmarks.pageSize.value" :total="bookmarks.total.value" @go="bookmarks.go" />
    </section>

    <section class="block" aria-labelledby="stats-title">
      <h2 id="stats-title">匿名统计</h2>
      <div class="panel box">
        <p class="muted">用来了解哪些内容有人看、有人用。只记匿名设备做过哪类操作的次数，不关联账号，不记录内容、搜索词和文件名。</p>
        <button type="button" class="btn btn-sm" role="switch" :aria-checked="statsOn" @click="toggleStats">{{ statsOn ? '已开启，点击关闭' : '已关闭，点击开启' }}</button>
      </div>
    </section>

    <section class="block" aria-labelledby="blocks-title">
      <h2 id="blocks-title">屏蔽的用户</h2>
      <div class="panel box">
        <p v-if="!blocked.length" class="muted">没有屏蔽任何人。在对方的主页可以屏蔽。</p>
        <ul v-else class="blocked-list">
          <li v-for="u in blocked" :key="u.uid">
            <RouterLink :to="`/u/${u.uid}`">{{ u.nickname }}</RouterLink>
            <button type="button" class="btn btn-sm" @click="unblock(u.uid)">取消屏蔽</button>
          </li>
        </ul>
      </div>
    </section>

    <section class="block" aria-labelledby="profile-title">
      <h2 id="profile-title">资料</h2>
      <div class="panel box">
        <div class="profile">
          <button type="button" class="avatar-btn" aria-label="更换头像" @click="avatarOpen = true">
            <UserAvatar :name="session.user?.nickname ?? ''" :src="session.user?.avatar_url" :size="64" />
            <span class="avatar-cover" aria-hidden="true">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M4 8h3l2-3h6l2 3h3v11H4z" /><circle cx="12" cy="13" r="3.5" /></svg>更换
            </span>
            <span class="avatar-badge" aria-hidden="true">
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 8h3l2-3h6l2 3h3v11H4z" /><circle cx="12" cy="13" r="3.5" /></svg>
            </span>
          </button>
          <div class="field">
            <span>用户名</span>
            <strong class="username">{{ session.user?.nickname }}</strong>
            <small>用户名是你的唯一账号，注册时设置，不能修改。头像属于相伴账号，在所有相伴产品里通用。</small>
          </div>
          <button type="button" class="btn btn-sm" @click="avatarOpen = true">更换头像</button>
        </div>
        <div v-if="avatarPending" class="avatar-pending">
          <span>新头像还没保存，裁剪位置已保留。</span>
          <button type="button" class="btn btn-sm btn-ghost" @click="avatarEditor?.discard()">放弃</button>
          <button type="button" class="btn btn-sm" @click="avatarOpen = true">继续编辑</button>
        </div>
      </div>
      <AvatarEditor
        ref="avatarEditor"
        v-model:open="avatarOpen"
        :name="session.user?.nickname ?? ''"
        :current="session.user?.avatar_url ?? null"
        @pending="avatarPending = $event"
      />
    </section>

    <section v-if="session.info" class="block" aria-labelledby="level-title">
      <h2 id="level-title">信任等级</h2>
      <div class="panel box levels">
        <p>
          你现在是 <strong>{{ TRUST_LABEL[session.info.user.trust_level] }}</strong>。今天最多可以发
          {{ session.info.limits.daily_topics }} 个帖子、{{ session.info.limits.daily_posts }} 条回复。
        </p>
        <ol>
          <li :class="{ on: session.info.user.trust_level === 0 }">
            <strong>新人</strong>可以浏览公开帖子、点赞和回复，也可以上传图片。在「创意 → 壁纸／游戏」可以发帖，外链在升级为成员后可用。
          </li>
          <li :class="{ on: session.info.user.trust_level === 1 }">
            <strong>成员</strong>：注册满 {{ session.info.limits.tl1_days }} 天、读过 {{ session.info.limits.tl1_reads }} 个帖子后自动升级。可以发帖、发链接和图片，能看「成员以上」可见的帖子，发帖时也能设成「成员以上」可见。
          </li>
          <li :class="{ on: session.info.user.trust_level === 2 }">
            <strong>老用户</strong>：长期分享靠谱内容的成员，由管理员根据贡献升级。能看和设置「老用户以上」可见的帖子。
          </li>
          <li :class="{ on: session.info.user.trust_level === 3 }">
            <strong>守护者</strong>：由管理员邀请，能看所有帖子，协助处理举报。
          </li>
        </ol>
        <p class="note">发帖时可以设置谁能看，最高只能设到你自己的等级。设了限制的帖子，游客和等级不够的人在列表、搜索和搜索引擎里都看不到。</p>
      </div>
    </section>

    <section class="block" aria-labelledby="account-title">
      <h2 id="account-title">账号</h2>
      <div class="panel box account">
        <p class="muted">需要注销账号或导出数据，请发邮件到 support@xiangban.xyz，我们会在 7 个工作日内处理。</p>
        <button type="button" class="btn" @click="session.logout()">退出登录</button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.settings {
  max-width: 780px;
  margin: 0 auto;
  background: var(--surface);
  padding: 18px 22px 8px;
}
h1 {
  margin: 0 0 18px;
  font-size: var(--fs-2xl);
  font-weight: 800;
}
.block {
  margin-bottom: 26px;
}
h2 {
  font-size: var(--fs-md);
  margin: 0 0 10px;
}
.box {
  padding: 16px 18px;
}
.row {
  display: flex;
  gap: 12px;
  align-items: flex-start;
}
.row .btn {
  margin-top: 27px;
}
.grow {
  flex: 1;
}
.levels p {
  margin: 0 0 10px;
  font-size: var(--fs-sm);
}
.levels ol {
  margin: 0;
  padding-left: 1.3em;
  display: grid;
  gap: 6px;
  font-size: var(--fs-sm);
  color: var(--ink-2);
}
.levels .note {
  margin: 12px 0 0;
  color: var(--ink-3);
  font-size: var(--fs-xs);
}
.levels li.on {
  color: var(--ink);
}
.levels li.on::marker {
  color: var(--valid);
  font-weight: 700;
}
.account {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}
.account p {
  margin: 0;
  font-size: var(--fs-sm);
}
@media (max-width: 767px) {
  .settings {
    margin: 0 -12px;
    padding: 14px 12px 8px;
  }
  h1 {
    font-size: var(--fs-xl);
  }
  .row,
  .account {
    flex-direction: column;
    align-items: stretch;
  }
  .row .btn {
    margin-top: 0;
  }
}
.profile {
  display: flex;
  align-items: center;
  gap: 18px;
}
.profile .field {
  flex: 1;
  min-width: 0;
}
.avatar-btn {
  position: relative;
  flex: none;
  padding: 0;
  border: 0;
  background: none;
  border-radius: 50%;
}
.avatar-btn:focus-visible {
  outline: 2px solid var(--brand);
  outline-offset: 3px;
}
.avatar-cover {
  position: absolute;
  inset: 0;
  display: grid;
  place-items: center;
  align-content: center;
  gap: 2px;
  border-radius: 50%;
  background: rgba(34, 34, 34, 0.46);
  color: #fff;
  font-size: var(--fs-xs);
  opacity: 0;
  transition: opacity 0.15s;
}
.avatar-btn:hover .avatar-cover,
.avatar-btn:focus-visible .avatar-cover {
  opacity: 1;
}
.avatar-badge {
  display: none;
  position: absolute;
  right: -3px;
  bottom: -3px;
  width: 24px;
  height: 24px;
  place-items: center;
  border-radius: 50%;
  background: var(--surface);
  border: 1px solid var(--line);
  color: var(--ink-2);
}
@media (hover: none) {
  .avatar-cover {
    display: none;
  }
  .avatar-badge {
    display: grid;
  }
}
.avatar-pending {
  margin-top: 10px;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px 8px 14px;
  border-radius: var(--r-sm);
  background: color-mix(in srgb, var(--brand-fill) 10%, var(--surface));
  color: var(--brand);
  font-size: 13px;
}
.avatar-pending span {
  margin-right: auto;
}
@media (max-width: 767px) {
  .profile {
    gap: 14px;
    align-items: flex-start;
  }
}
.blocked-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 8px;
}
.blocked-list li {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>
