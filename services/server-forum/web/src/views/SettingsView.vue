<script setup lang="ts">
import TopicList from '@/components/TopicList.vue'
import PagerNav from '@/components/PagerNav.vue'
import { api } from '@/api/forum'
import { useSession } from '@/stores/session'
import { useTopicList } from '@/lib/useList'
import { TRUST_LABEL } from '@/lib/format'
import { useHead } from '@/lib/head'

useHead(() => ({ title: '收藏与设置' }))
const session = useSession()

const bookmarks = useTopicList(p => api.bookmarks(p), [() => session.user?.uid])
</script>

<template>
  <div class="settings">
    <h1>收藏与设置</h1>

    <section class="block" aria-labelledby="bm-title">
      <h2 id="bm-title">我的收藏</h2>
      <TopicList :items="bookmarks.items.value" :loading="bookmarks.loading.value" empty="还没有收藏。看到有用的羊毛，点帖子下方的收藏就能存到这里。" />
      <PagerNav :page="bookmarks.page.value" :page-size="bookmarks.pageSize.value" :total="bookmarks.total.value" @go="bookmarks.go" />
    </section>

    <section class="block" aria-labelledby="profile-title">
      <h2 id="profile-title">资料</h2>
      <div class="panel box">
        <div class="field">
          <span>用户名</span>
          <strong class="username">{{ session.user?.nickname }}</strong>
          <small>用户名是你的唯一账号，注册时设置，不能修改。头像属于相伴账号，在所有相伴产品里通用。</small>
        </div>
      </div>
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
            <strong>新人</strong>：可以浏览所有人可见的帖子、点赞和少量回复，链接和图片显示为文字，还不能发帖。
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
</style>
