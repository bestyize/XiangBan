<script setup lang="ts">
import { onActivated } from 'vue'
import { setAmbient } from '@/composables/ambient'
import { canInstall, isIOS, isStandalone, promptInstall } from '@/composables/install'
import { toast } from '@/composables/toast'
import { useAccount } from '@/stores/account'
import { useCatalog } from '@/stores/catalog'
import { providerName } from '@/utils'

const catalog = useCatalog()
const account = useAccount()

async function logout() {
  await account.logout()
  toast('已退出登录')
}

onActivated(() => {
  setAmbient(null)
  catalog.load()
})

async function install() {
  if (await promptInstall()) toast('已添加到主屏幕')
}
</script>

<template>
  <main class="page">
    <header class="heading">
      <h1 class="display">我的</h1>
    </header>

    <section class="section">
      <h2 class="display">相伴账号</h2>
      <template v-if="account.user">
        <p class="lead">
          {{ account.user.nickname }}<span v-if="account.email" class="muted-inline"> · {{ account.email }}</span>
        </p>
        <p class="hint">同一个账号可登录小伴桌面、小伴手机壁纸和相伴其他产品。</p>
        <button class="btn" type="button" @click="logout">退出登录</button>
      </template>
      <template v-else>
        <p class="lead">用邮箱验证码登录，新邮箱会自动注册。登录后小伴桌面与手机端共用同一个账号。</p>
        <button class="btn btn-primary" type="button" @click="account.openLogin()">登录 / 注册</button>
      </template>
    </section>

    <section v-if="!isStandalone && (canInstall || isIOS)" class="section">
      <h2 class="display">安装到手机</h2>
      <p class="lead">装到主屏幕后全屏运行，没有浏览器地址栏，用起来和 App 一样。</p>
      <button v-if="canInstall" class="btn btn-primary" type="button" @click="install">添加到主屏幕</button>
      <p v-else class="hint">在 Safari 里点底部的分享按钮，选择「添加到主屏幕」。</p>
    </section>

    <section class="section">
      <h2 class="display">数据源</h2>
      <ul class="rows">
        <li v-for="p in catalog.providers" :key="p.id" class="row">
          <div class="grow">
            <p class="name">{{ providerName(p.id) }}</p>
            <p class="desc">{{ p.description }}</p>
            <p v-if="!p.is_configured && p.id !== 'demo'" class="hint">
              到<RouterLink to="/admin/keys" class="link">管理后台</RouterLink>添加密钥后即可启用，不用重启服务。
            </p>
          </div>
          <span class="status" :class="{ ready: p.is_configured }">{{ p.is_configured ? '已就绪' : '未配置' }}</span>
        </li>
      </ul>
    </section>

    <section class="section">
      <h2 class="display">关于壁纸站</h2>
      <p class="lead">
        精选高品质 4K 超高清壁纸与动态视频壁纸，每一张均经过精心筛选与人工审核入库。
      </p>
      <div class="site-meta">
        <span class="badge-item">超清 4K</span>
        <span class="badge-item">动态视频</span>
        <span class="badge-item">全平台适配</span>
      </div>
    </section>

    <section class="section">
      <h2 class="display">站长管理</h2>
      <p class="lead">管理端控制台，支持供应商壁纸一键采集入库、内容审核与自定义资源上传上架。</p>
      <RouterLink to="/admin" class="btn btn-primary">进入管理后台</RouterLink>
    </section>
  </main>
</template>

<style scoped>
.heading {
  padding: 28px 0 8px;
}

.heading h1 {
  font-size: clamp(44px, 7vw, 76px);
}

.section {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 12px;
  max-width: 720px;
  padding: 28px 0;
  border-top: 1px solid var(--line);
}

.section h2 {
  font-size: 32px;
}

.lead {
  max-width: 52ch;
  color: var(--text-2);
}

.site-meta {
  display: flex;
  gap: 8px;
  margin-top: 6px;
}

.badge-item {
  font-size: 12px;
  padding: 4px 10px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.08);
  color: var(--text-2);
}

.muted-inline {
  color: var(--text-3);
}

.hint {
  max-width: 52ch;
  font-size: 13px;
  color: var(--text-3);
}

.link {
  color: var(--text-2);
  text-decoration: underline;
  text-underline-offset: 3px;
}

.link:hover {
  color: var(--text);
}

.rows {
  width: 100%;
  list-style: none;
}

.row {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  padding: 14px 0;
  border-bottom: 1px solid var(--line);
}

.row:last-child {
  border-bottom: 0;
}

.grow {
  flex: 1;
  min-width: 0;
}

.name {
  font-weight: 500;
}

.desc {
  font-size: 13px;
  color: var(--text-2);
}

.status {
  flex: none;
  padding: 3px 10px;
  border-radius: 999px;
  border: 1px solid var(--line);
  font-size: 12px;
  color: var(--text-3);
}

.status.ready {
  border-color: var(--line-strong);
  color: var(--text);
}
</style>
