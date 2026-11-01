<script setup lang="ts">
import { useSession } from '@/stores/session'
import { useNotifications } from '@/stores/notifications'
import { useTheme, THEME_LABEL } from '@/lib/theme'
import { useHead } from '@/lib/head'
import AppIcon from '@/components/AppIcon.vue'
import UserAvatar from '@/components/UserAvatar.vue'
useHead(() => ({title:'我的'}))
const session = useSession(), inbox = useNotifications(), { preference, accentName, openAppearance } = useTheme()
const groups = [
  [{to:'/me/works',icon:'create',title:'我的作品',description:'查看已发布的作品与方案'}, {to:'/me/content-bookmarks',icon:'bookmark',title:'作品收藏',description:'留住喜欢的作品和桌面方案'}, {to:'/settings#bm-title',icon:'folder',title:'帖子收藏',description:'值得再读的讨论与经验'}],
  [{to:'/notifications',icon:'bell',title:'消息通知',description:'回复、提及与社区消息'}, {to:'/ai?panel=account',icon:'ai',title:'小伴 AI 与额度',description:'查看可用额度和使用模式'}, {to:'/points',icon:'plus',title:'社区积分',description:'签到、邀请与积分账本'}, {to:'/settings',icon:'settings',title:'账户与设置',description:'个人资料、隐私和屏蔽管理'}],
]
</script>
<template>
  <div class="me-page">
    <section class="profile-block">
      <UserAvatar :name="session.user?.nickname || '小伴'" :src="session.user?.avatar_url" :size="64" />
      <div v-if="session.loggedIn"><h1>{{ session.user?.nickname || (session.restoring ? '正在加载…' : '欢迎回来') }}</h1><RouterLink v-if="session.user" :to="`/u/${session.user.uid}`">查看个人主页 <span aria-hidden="true">›</span></RouterLink><p v-else>账号已登录，资料加载后显示</p></div>
      <div v-else><h1>让灵感有个落脚处</h1><p>登录后收藏、交流，也可以和小伴聊聊。</p><button class="btn btn-primary" @click="session.loginOpen = true">登录 / 注册</button></div>
    </section>
    <section v-for="(rows,i) in groups" :key="i" class="me-group" :aria-label="i === 0 ? '我的内容' : '账户与消息'">
      <RouterLink v-for="row in rows" :key="row.to" :to="row.to" class="me-row"><AppIcon :name="row.icon" /><span><b>{{ row.title }}</b><small>{{ row.description }}</small></span><span v-if="row.to === '/notifications' && inbox.unread" class="unread">{{ inbox.unread }}</span><AppIcon name="arrow" :size="17" /></RouterLink>
    </section>
    <section class="me-group" aria-label="更多">
      <button class="me-row" @click="openAppearance"><AppIcon name="sun" /><span><b>外观</b><small>{{ THEME_LABEL[preference] }} · {{ accentName }}</small></span><AppIcon name="arrow" :size="17" /></button>
      <a class="me-row" href="https://wallpaper.xiangban.xyz" target="_blank" rel="noopener noreferrer"><AppIcon name="download" /><span><b>下载小伴客户端</b><small>创造自己的作品，把小伴带到桌面</small></span><AppIcon name="arrow" :size="17" /></a>
      <RouterLink class="me-row" to="/privacy"><AppIcon name="shield" /><span><b>隐私与使用说明</b></span><AppIcon name="arrow" :size="17" /></RouterLink>
    </section>
    <button v-if="session.loggedIn" class="logout" @click="session.logout()">退出登录</button>
  </div>
</template>
<style scoped>
.me-page{max-width:760px;margin:0 auto}.profile-block{display:flex;gap:20px;align-items:center;padding:24px 12px 32px}.profile-block>div{min-width:0}.profile-block h1{font-size:21px;margin:0 0 4px;overflow-wrap:anywhere}.profile-block p,.profile-block a{font-size:13px;color:var(--ink-3);margin:0 0 12px}.profile-block .btn{margin-top:4px}.me-group{background:var(--surface);border-radius:16px;overflow:hidden;margin-bottom:16px}.me-row{display:flex;align-items:center;gap:14px;width:100%;padding:16px 20px;min-height:72px;border:0;border-bottom:1px solid var(--line);background:none;text-align:left}.me-row:last-child{border-bottom:0}.me-row>svg{color:var(--ink-2);flex:none}.me-row>span:first-of-type{flex:1}.me-row b{display:block;font-size:15px;font-weight:550}.me-row small{display:block;color:var(--ink-3);font-size:12px;margin-top:2px}.me-row:hover{background:var(--surface-2)}.unread{background:var(--brand-soft);color:var(--brand);padding:2px 8px;border-radius:12px;font-size:12px}.logout{width:100%;min-height:48px;border:0;background:var(--surface);color:var(--ink-2);border-radius:14px;font-size:14px}
@media(max-width:767px){.me-group{margin-left:-12px;margin-right:-12px;border-radius:0}.profile-block{padding:20px 4px 28px;gap:16px}.profile-block h1{font-size:19px}.me-row{padding:14px 20px}.logout{border-radius:0;margin-left:-12px;width:calc(100% + 24px)}}
</style>
