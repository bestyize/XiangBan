<script setup lang="ts">
import { computed, ref } from 'vue';
import { ALL_TESTS, getTestById } from '../data';
import {
  AccountApiError,
  changePassword,
  formatPrimaryIdentity,
  validatePasswordLocal,
  type AccountUser
} from '../engine/accountApi';
import { findProfileByCode } from '../engine/scoring';
import type { QuizDraft, UserPrefs } from '../engine/storage';
import { triggerSystemShare } from '../engine/systemShare';
import type { EvaluatedResult, TestId } from '../engine/types';

const props = defineProps<{
  user: AccountUser | null;
  accessToken: string | null;
  results: Partial<Record<TestId, EvaluatedResult>>;
  drafts: QuizDraft[];
  prefs: UserPrefs;
  dark: boolean;
  compatCount: number;
}>();

const emit = defineEmits<{
  gotoAuth: [mode?: 'code' | 'pwd' | 'register' | 'reset'];
  resumeDraft: [testId: TestId];
  toggleTheme: [];
  updatePrefs: [partial: Partial<UserPrefs>];
  logout: [];
  toast: [msg: string];
}>();

const showSecurityPanel = ref(false);
const showAboutPanel = ref(false);
const currentPassword = ref('');
const newPassword = ref('');
const confirmPassword = ref('');
const pwdBusy = ref(false);
const pwdError = ref('');

const unlockedList = computed(() =>
  ALL_TESTS.filter((t) => Boolean(props.results[t.id])).map((t) => {
    const res = props.results[t.id]!;
    return { test: t, res, profile: findProfileByCode(t, res.profileCode) };
  })
);

const unlockedCount = computed(() => unlockedList.value.length);
const eggCount = computed(() => unlockedList.value.filter((item) => item.res.isHiddenEgg).length);
const heroEmoji = computed(() => unlockedList.value[0]?.profile.avatarEmoji || '🍊');

const primaryIdentity = computed(() => (props.user ? formatPrimaryIdentity(props.user) : ''));

const latestDraft = computed(() => {
  const first = props.drafts[0];
  if (!first) return null;
  const test = getTestById(first.testId);
  return {
    draft: first,
    test,
    answered: Object.keys(first.answers || {}).length,
    total: test.questions.length
  };
});

async function shareSite() {
  const shareUrl =
    typeof window !== 'undefined' ? window.location.href.split('#')[0] : 'https://people.xiaban.xyz/';
  const outcome = await triggerSystemShare({
    title: '人格贩卖机 VEND·ME · 0元全免费趣味与人格盲盒测评',
    text: '12 套潮玩风格趣味与人格测评，免登录随时开测、生成高清海报还能双人合盘！',
    url: shareUrl
  });
  if (outcome.message) {
    emit('toast', outcome.message);
  }
}

function handleSecurityClick() {
  if (!props.user) {
    emit('gotoAuth', 'code');
    return;
  }
  showSecurityPanel.value = !showSecurityPanel.value;
  pwdError.value = '';
}

async function submitChangePassword() {
  if (!props.user || !props.accessToken || pwdBusy.value) return;
  pwdError.value = '';

  if (props.user.has_password && !currentPassword.value) {
    pwdError.value = '请输入当前密码';
    return;
  }
  const err = validatePasswordLocal(newPassword.value, props.user.username);
  if (err) {
    pwdError.value = err;
    return;
  }
  if (newPassword.value !== confirmPassword.value) {
    pwdError.value = '两次输入的新密码不一致';
    return;
  }

  pwdBusy.value = true;
  try {
    await changePassword(
      props.accessToken,
      props.user.has_password ? currentPassword.value : undefined,
      newPassword.value
    );
    currentPassword.value = '';
    newPassword.value = '';
    confirmPassword.value = '';
    showSecurityPanel.value = false;
    emit('toast', '登录密码已更新');
  } catch (e) {
    pwdError.value = e instanceof AccountApiError ? e.message : '修改密码失败，请稍后重试';
  } finally {
    pwdBusy.value = false;
  }
}
</script>

<template>
  <div class="mine-page">
    <header class="topbar">
      <span class="topbar-title">我的</span>
      <button class="sticker" style="background: var(--pop-lime)" @click="shareSite">
        安利给好友 ✦
      </button>
    </header>

    <div class="mine-pad">
      <!-- 状态 A：已登录身份卡 -->
      <section v-if="user" class="mine-hero">
        <div class="mine-row">
          <div class="mine-ava">
            <img v-if="user.avatar_url" :src="user.avatar_url" :alt="user.username" />
            <span v-else>{{ heroEmoji }}</span>
          </div>
          <div class="mine-info">
            <div class="mine-name-line">
              <b>{{ user.username }}</b>
              <span class="sticker mono" style="background: #fffdf8">#{{ user.account_id }}</span>
              <span class="sticker" style="background: var(--pop-mint)">已登录</span>
            </div>
            <div class="mine-sub">{{ primaryIdentity }} · 相伴统一账号通用</div>
          </div>
        </div>
        <div class="mine-stats">
          <div class="ms-box">
            <b class="num">{{ unlockedCount }}/{{ ALL_TESTS.length }}</b>
            <small>已集贴纸</small>
          </div>
          <div class="ms-box">
            <b class="num">{{ eggCount }}</b>
            <small>隐藏彩蛋</small>
          </div>
          <div class="ms-box">
            <b class="num">{{ compatCount }}</b>
            <small>合盘互动</small>
          </div>
        </div>
      </section>

      <!-- 状态 B：未登录游客引导卡 -->
      <section v-else class="mine-hero guest">
        <div class="mine-row">
          <div class="mine-ava">🎭</div>
          <div class="mine-info">
            <div class="mine-name-line">
              <b>未登录游客</b>
              <span class="sticker" style="background: #fffdf8">仅存本机</span>
            </div>
            <div class="mine-sub">登录相伴统一账号，用同一身份管理测评贴纸与小伴社区</div>
          </div>
        </div>
        <div class="guest-merge-tip">
          <template v-if="unlockedCount > 0">
            ✦ 当前本机已点亮 <b>{{ unlockedCount }}/{{ ALL_TESTS.length }}</b> 枚贴纸，登录后自动关联至相伴账号，绝不丢失
          </template>
          <template v-else>
            ✦ 游客模式 0 门槛随时开测；登录相伴账号后可与小伴社区互通身份
          </template>
        </div>
        <button class="btn-pop guest-cta" @click="emit('gotoAuth', 'code')">
          登录 / 注册相伴账号 →
        </button>
      </section>

      <!-- 未答完的盲盒草稿续答条 -->
      <section v-if="latestDraft" class="draft-banner">
        <div class="draft-copy">
          <b>有 {{ drafts.length }} 份未答完的盲盒草稿</b>
          <small>{{ latestDraft.test.title }} · 已答 {{ latestDraft.answered }}/{{ latestDraft.total }} 题</small>
        </div>
        <button
          class="sticker draft-go"
          @click="emit('resumeDraft', latestDraft.draft.testId)"
        >
          继续答 ›
        </button>
      </section>

      <!-- 小伴生态双入口：小伴社区 + 小伴游戏厅 -->
      <section class="mine-portals" aria-label="小伴生态入口">
        <a
          class="mine-portal portal-comm"
          href="https://chat.xiangban.xyz"
          target="_blank"
          rel="noopener noreferrer"
        >
          <div class="portal-top">
            <div class="portal-ico" aria-hidden="true">🫧</div>
            <span class="portal-arrow">去逛逛 ↗</span>
          </div>
          <b class="portal-title">小伴社区</b>
          <span class="portal-desc">晒贴纸 · 找同频搭子</span>
        </a>

        <a
          class="mine-portal portal-game"
          href="https://game.xiaban.xyz"
          target="_blank"
          rel="noopener noreferrer"
        >
          <div class="portal-top">
            <div class="portal-ico" aria-hidden="true">🕹️</div>
            <span class="portal-arrow">开一局 ↗</span>
          </div>
          <b class="portal-title">小伴游戏厅</b>
          <span class="portal-desc">点开即玩休闲小游戏</span>
        </a>

        <a
          class="mine-app-dl"
          href="https://xiaban.xyz/app"
          target="_blank"
          rel="noopener noreferrer"
        >
          <div class="app-dl-left">
            <div class="portal-ico" aria-hidden="true">📱</div>
            <div class="app-dl-body">
              <div class="app-dl-title">
                <b>下载小伴 App</b>
                <span class="sticker" style="background: #fffdf8">手机 / 电脑</span>
              </div>
              <span class="portal-desc">社区、AI 伙伴与高清壁纸一站体验</span>
            </div>
          </div>
          <span class="app-dl-cta">去下载 ↗</span>
        </a>
      </section>

      <!-- 偏好设置 -->
      <section class="mine-list" aria-label="偏好设置">
        <button class="mine-item" @click="emit('toggleTheme')">
          <span>外观主题</span>
          <span class="sticker" style="background: var(--pop-cream)">
            {{ dark ? '暗夜潮玩' : '暖纸杂志' }}
          </span>
        </button>

        <button
          class="mine-item"
          @click="emit('updatePrefs', { autoAdvance: !prefs.autoAdvance })"
        >
          <span>选完自动切下一题</span>
          <span
            class="sticker"
            :style="{ background: prefs.autoAdvance ? 'var(--pop-mint)' : 'var(--pop-cream)' }"
          >
            {{ prefs.autoAdvance ? '已开启' : '手动切题' }}
          </span>
        </button>

        <button
          class="mine-item"
          @click="
            emit('updatePrefs', {
              defaultPosterRatio: prefs.defaultPosterRatio === '916' ? '34' : '916'
            })
          "
        >
          <span>默认海报比例</span>
          <span
            class="sticker"
            :style="{
              background: prefs.defaultPosterRatio === '916' ? 'var(--pop-blue)' : 'var(--pop-pink)'
            }"
          >
            {{ prefs.defaultPosterRatio === '916' ? '9:16 朋友圈' : '3:4 小红书' }}
          </span>
        </button>
      </section>

      <!-- 账号与档案管理 -->
      <section class="mine-list" aria-label="账号与关于">
        <button class="mine-item" @click="handleSecurityClick">
          <span>账号与安全（邮箱 / 密码）</span>
          <small v-if="user">{{ showSecurityPanel ? '收起 ⌃' : '管理密码 ›' }}</small>
          <small v-else>去登录 / 注册 ›</small>
        </button>

        <!-- 登录态展开的密码管理面板 -->
        <div v-if="user && showSecurityPanel" class="inline-panel">
          <div class="sec-meta">
            <span>唯一用户名：<b>{{ user.username }}</b>（注册后不可修改）</span>
            <span v-if="user.identities.length">
              已绑定：{{ user.identities.map((i) => i.masked).join(' / ') }}
            </span>
          </div>
          <div class="sec-form">
            <input
              v-if="user.has_password"
              v-model="currentPassword"
              type="password"
              class="pop-input"
              placeholder="当前密码"
              autocomplete="current-password"
            />
            <input
              v-model="newPassword"
              type="password"
              class="pop-input"
              placeholder="新密码（8-128 位，不能与用户名相同）"
              autocomplete="new-password"
            />
            <input
              v-model="confirmPassword"
              type="password"
              class="pop-input"
              placeholder="再次确认新密码"
              autocomplete="new-password"
              @keyup.enter="submitChangePassword"
            />
            <p v-if="pwdError" class="inline-err">{{ pwdError }}</p>
            <button
              class="btn-pop sec-save-btn"
              :disabled="pwdBusy"
              @click="submitChangePassword"
            >
              {{ pwdBusy ? '提交中…' : '保存新密码' }}
            </button>
          </div>
        </div>

        <button class="mine-item" @click="showAboutPanel = !showAboutPanel">
          <span>关于人格贩卖机</span>
          <small>0元全免费承诺</small>
        </button>

        <div v-if="showAboutPanel" class="inline-panel about-copy">
          <b>人格贩卖机 VEND·ME</b> 收录 12 套原创潮玩风趣味与人格测评，承诺
          0 元全免费、免登录即可完整作答与导出高清海报；登录接入相伴统一账号中心（account.xiangban.xyz），与小伴社区共用同一身份。
        </div>

        <button v-if="user" class="mine-item logout-item" @click="emit('logout')">
          <span>退出当前账号</span>
          <small class="logout-sub">本地缓存保留 ›</small>
        </button>
      </section>
    </div>
  </div>
</template>

<style scoped>
.mine-page {
  min-height: 100%;
  display: flex;
  flex-direction: column;
}
.mine-pad {
  padding: 6px 16px 26px;
  display: flex;
  flex-direction: column;
  gap: 11px;
}

.mine-hero {
  padding: 15px;
  border-radius: 20px;
  background: var(--pop-peach);
  color: #18181b;
  border: 2px solid #18181b;
  box-shadow: 4px 4px 0 #18181b;
  position: relative;
  overflow: hidden;
}
.mine-hero.guest {
  background: linear-gradient(135deg, #fff2c2 0%, #ffd89b 100%);
}
.mine-row {
  display: flex;
  align-items: center;
  gap: 12px;
}
.mine-ava {
  width: 54px;
  height: 54px;
  border-radius: 16px;
  background: #fffdf8;
  border: 2px solid #18181b;
  box-shadow: 2.5px 2.5px 0 #18181b;
  display: grid;
  place-items: center;
  font-size: 28px;
  transform: rotate(-4deg);
  flex-shrink: 0;
  overflow: hidden;
}
.mine-ava img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.mine-info {
  flex: 1;
  min-width: 0;
}
.mine-name-line {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px;
}
.mine-name-line b {
  font-size: 18px;
  font-weight: 950;
  word-break: break-word;
}
.mine-sub {
  font-size: 11.5px;
  font-weight: 700;
  opacity: 0.84;
  margin-top: 3px;
  line-height: 1.45;
  word-break: break-word;
}
.mine-stats {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
  margin-top: 13px;
}
.ms-box {
  background: #fffdf8;
  border: 1.5px solid #18181b;
  border-radius: 12px;
  padding: 8px 6px;
  text-align: center;
}
.ms-box b {
  display: block;
  font-size: 17px;
  font-weight: 950;
  line-height: 1.15;
}
.ms-box small {
  display: block;
  font-size: 10px;
  font-weight: 800;
  color: #6e6960;
  margin-top: 2px;
}
.guest-merge-tip {
  margin-top: 11px;
  padding: 8px 10px;
  border-radius: 12px;
  background: #fffdf8;
  border: 1.5px dashed #18181b;
  font-size: 11.5px;
  font-weight: 750;
  line-height: 1.45;
}
.guest-cta {
  width: 100%;
  height: 43px;
  margin-top: 11px;
  font-size: 14px;
}

.draft-banner {
  padding: 11px 13px;
  border-radius: 16px;
  background: var(--pop-lime);
  color: #18181b;
  border: 1.8px solid #18181b;
  box-shadow: 3px 3px 0 #18181b;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}
.draft-copy {
  min-width: 0;
}
.draft-copy b {
  font-size: 13px;
  font-weight: 900;
  display: block;
}
.draft-copy small {
  font-size: 11px;
  font-weight: 700;
  display: block;
  margin-top: 2px;
}
.draft-go {
  background: #18181b;
  color: var(--pop-lime);
  flex-shrink: 0;
}

.mine-portals {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 10px;
}
.mine-portal {
  display: flex;
  flex-direction: column;
  padding: 12px 12px 11px;
  border-radius: 18px;
  text-decoration: none;
  color: #18181b;
  border: 1.8px solid #18181b;
  box-shadow: 3px 3px 0 #18181b;
  transition: transform 0.12s ease, box-shadow 0.12s ease;
  min-width: 0;
}
.mine-portal:active {
  transform: translate(2px, 2px);
  box-shadow: 1px 1px 0 #18181b;
}
.portal-comm {
  background: linear-gradient(135deg, #ece4ff 0%, #dcf3ff 100%);
}
.portal-game {
  background: linear-gradient(135deg, #d9f9e6 0%, #fff3b0 100%);
}
.portal-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
}
.portal-ico {
  width: 34px;
  height: 34px;
  border-radius: 11px;
  background: #fffdf8;
  border: 1.6px solid #18181b;
  box-shadow: 2px 2px 0 #18181b;
  display: grid;
  place-items: center;
  font-size: 18px;
  flex-shrink: 0;
}
.portal-arrow {
  padding: 3px 8px;
  border-radius: 999px;
  background: #18181b;
  color: var(--pop-lime);
  font-size: 10.5px;
  font-weight: 900;
  white-space: nowrap;
}
.portal-title {
  margin-top: 9px;
  font-size: 14px;
  font-weight: 950;
  line-height: 1.2;
}
.portal-desc {
  margin-top: 3px;
  font-size: 11px;
  font-weight: 700;
  color: #3f3a4d;
  line-height: 1.3;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.mine-app-dl {
  grid-column: 1 / -1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 18px;
  text-decoration: none;
  color: #18181b;
  background: linear-gradient(135deg, #ffe4ec 0%, #ffead2 100%);
  border: 1.8px solid #18181b;
  box-shadow: 3px 3px 0 #18181b;
  transition: transform 0.12s ease, box-shadow 0.12s ease;
}
.mine-app-dl:active {
  transform: translate(2px, 2px);
  box-shadow: 1px 1px 0 #18181b;
}
.app-dl-left {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
  flex: 1;
}
.app-dl-body {
  min-width: 0;
  display: flex;
  flex-direction: column;
}
.app-dl-title {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}
.app-dl-title b {
  font-size: 13.5px;
  font-weight: 950;
  line-height: 1.2;
}
.app-dl-cta {
  flex-shrink: 0;
  padding: 5px 10px;
  border-radius: 999px;
  background: #18181b;
  color: var(--pop-lime);
  font-size: 11px;
  font-weight: 900;
  border: 1.5px solid #18181b;
}

.mine-list {
  border-radius: 18px;
  background: var(--surface);
  border: 1.8px solid var(--border);
  box-shadow: var(--shadow);
  overflow: hidden;
}
.mine-item {
  width: 100%;
  padding: 12px 14px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  border-bottom: 1.2px solid rgba(24, 24, 27, 0.09);
  font-size: 13px;
  font-weight: 800;
  text-align: left;
}
:global(.dark) .mine-item {
  border-bottom-color: rgba(255, 255, 255, 0.08);
}
.mine-item:last-child {
  border-bottom: 0;
}
.mine-item small {
  font-size: 11px;
  color: var(--muted);
  font-weight: 650;
  flex-shrink: 0;
}
.logout-item {
  color: var(--pop-red);
}
.logout-sub {
  color: var(--pop-red) !important;
}

.inline-panel {
  padding: 12px 14px 14px;
  background: var(--sub);
  border-bottom: 1.2px solid rgba(24, 24, 27, 0.09);
}
:global(.dark) .inline-panel {
  border-bottom-color: rgba(255, 255, 255, 0.08);
}
.sec-meta {
  display: flex;
  flex-direction: column;
  gap: 3px;
  font-size: 11.5px;
  color: var(--muted);
  margin-bottom: 10px;
}
.sec-form {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.pop-input {
  width: 100%;
  height: 40px;
  padding: 0 12px;
  border-radius: 12px;
  background: var(--surface);
  border: 1.6px solid var(--border);
  font-size: 13px;
  font-weight: 650;
}
.inline-err {
  font-size: 11.5px;
  font-weight: 750;
  color: var(--pop-red);
}
.sec-save-btn {
  height: 40px;
  font-size: 13px;
}
.about-copy {
  font-size: 12px;
  line-height: 1.6;
  color: var(--ink-2);
}
</style>
