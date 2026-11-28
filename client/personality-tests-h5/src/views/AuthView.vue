<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import {
  AccountApiError,
  checkUsernameAvailable,
  getAuthMethods,
  loginWithCode,
  loginWithPassword,
  registerAccount,
  resetPassword,
  sendVerificationCode,
  validatePasswordLocal,
  validateUsernameLocal,
  type AuthSessionResponse,
  type SignupMethods
} from '../engine/accountApi';
import type { AuthRouteMode } from '../engine/shareCodec';

const props = defineProps<{
  initialMode?: AuthRouteMode;
  localStickerCount: number;
}>();

const emit = defineEmits<{
  back: [];
  authenticated: [session: AuthSessionResponse];
  toast: [msg: string];
}>();

const mode = ref<AuthRouteMode>(props.initialMode || 'code');
const methods = ref<SignupMethods>({ email: true, phone: false });

// 验证码登录态
const codeIdentifier = ref('');
const codeInput = ref('');
const codeSending = ref(false);
const codeSubmitting = ref(false);
const codeCooldown = ref(0);
let codeTimer: number | null = null;

// 密码登录态
const pwdIdentifier = ref('');
const pwdPassword = ref('');
const showPwd = ref(false);
const pwdSubmitting = ref(false);

// 新号建档态（15 分钟注册凭证）
const regToken = ref('');
const regVerifiedIdentifier = ref('');
const regUsername = ref('');
const regPassword = ref('');
const regSubmitting = ref(false);
const usernameCheckStatus = ref<'idle' | 'checking' | 'ok' | 'bad'>('idle');
const usernameCheckMsg = ref('');
let usernameTimer: number | null = null;

// 重置密码态
const resetIdentifier = ref('');
const resetCode = ref('');
const resetNewPassword = ref('');
const resetSending = ref(false);
const resetSubmitting = ref(false);
const resetCooldown = ref(0);
let resetTimer: number | null = null;

const errorMsg = ref('');

watch(
  () => props.initialMode,
  (m) => {
    if (m) mode.value = m;
  }
);

onMounted(async () => {
  methods.value = await getAuthMethods();
});

onBeforeUnmount(() => {
  if (codeTimer) window.clearInterval(codeTimer);
  if (resetTimer) window.clearInterval(resetTimer);
  if (usernameTimer) window.clearTimeout(usernameTimer);
});

const codeMethodLabel = computed(() => {
  if (methods.value.email && methods.value.phone) return '邮箱 / 手机号';
  if (methods.value.phone && !methods.value.email) return '手机号';
  return '邮箱地址';
});

const codeMethodPlaceholder = computed(() => {
  if (methods.value.email && methods.value.phone) return '输入邮箱或中国大陆手机号';
  if (methods.value.phone && !methods.value.email) return '输入中国大陆手机号';
  return 'name@example.com';
});

const codeMethodFoot = computed(() => {
  if (methods.value.email && methods.value.phone) return '当前开放邮箱 / 手机验证码';
  if (methods.value.phone && !methods.value.email) return '当前开放手机验证码';
  return '当前开放邮箱验证码';
});

const heroMeta = computed(() => {
  if (mode.value === 'pwd') {
    return {
      title: '密码登录',
      desc: '支持输入相伴用户名、已绑定邮箱或手机号加密码登录。'
    };
  }
  if (mode.value === 'register') {
    return {
      title: '完善你的唯一身份',
      desc:
        props.localStickerCount > 0
          ? `为新账号设置唯一用户名与密码，本机 ${props.localStickerCount} 枚贴纸将自动关联保留。`
          : '为新账号设置全局唯一的用户名与密码，跨小伴全端通用。'
    };
  }
  if (mode.value === 'reset') {
    return {
      title: '重置登录密码',
      desc: '通过已绑定邮箱或手机号接收验证码，重置后其他设备会自动下线。'
    };
  }
  return {
    title: '登录人格贩卖机',
    desc: '验证邮箱即可登录；未注册邮箱验证后一步设用户名与密码建号。'
  };
});

function switchMode(next: AuthRouteMode) {
  mode.value = next;
  errorMsg.value = '';
}

function startCountdown(kind: 'login' | 'reset', seconds: number) {
  const sec = Math.max(1, Math.round(seconds || 60));
  if (kind === 'login') {
    if (codeTimer) window.clearInterval(codeTimer);
    codeCooldown.value = sec;
    codeTimer = window.setInterval(() => {
      codeCooldown.value -= 1;
      if (codeCooldown.value <= 0) {
        if (codeTimer) window.clearInterval(codeTimer);
        codeTimer = null;
        codeCooldown.value = 0;
      }
    }, 1000);
  } else {
    if (resetTimer) window.clearInterval(resetTimer);
    resetCooldown.value = sec;
    resetTimer = window.setInterval(() => {
      resetCooldown.value -= 1;
      if (resetCooldown.value <= 0) {
        if (resetTimer) window.clearInterval(resetTimer);
        resetTimer = null;
        resetCooldown.value = 0;
      }
    }, 1000);
  }
}

async function handleSendCode() {
  if (codeSending.value || codeCooldown.value > 0) return;
  const ident = codeIdentifier.value.trim();
  if (!ident) {
    errorMsg.value = `请先输入${codeMethodLabel.value}`;
    return;
  }
  errorMsg.value = '';
  codeSending.value = true;
  try {
    const res = await sendVerificationCode(ident, 'login');
    startCountdown('login', res.cooldown_seconds || 60);
    emit('toast', '验证码已发送，请查收（5 分钟内有效）');
  } catch (e) {
    if (e instanceof AccountApiError) {
      errorMsg.value = e.message;
      if (e.retryAfter && e.retryAfter > 0) {
        startCountdown('login', e.retryAfter);
      }
    } else {
      errorMsg.value = '发送验证码失败，请稍后重试';
    }
  } finally {
    codeSending.value = false;
  }
}

async function handleCodeSubmit() {
  if (codeSubmitting.value) return;
  const ident = codeIdentifier.value.trim();
  const code = codeInput.value.trim();
  if (!ident) {
    errorMsg.value = `请输入${codeMethodLabel.value}`;
    return;
  }
  if (!code) {
    errorMsg.value = '请输入 6 位验证码';
    return;
  }
  errorMsg.value = '';
  codeSubmitting.value = true;
  try {
    const res = await loginWithCode(ident, code);
    if ('registration_required' in res && res.registration_required) {
      regToken.value = res.registration_token;
      regVerifiedIdentifier.value = ident;
      switchMode('register');
      emit('toast', '验证通过！请设置唯一用户名与密码完成建号');
      return;
    }
    emit('authenticated', res as AuthSessionResponse);
  } catch (e) {
    errorMsg.value = e instanceof AccountApiError ? e.message : '验证失败，请稍后重试';
  } finally {
    codeSubmitting.value = false;
  }
}

async function handlePwdSubmit() {
  if (pwdSubmitting.value) return;
  const ident = pwdIdentifier.value.trim();
  if (!ident || !pwdPassword.value) {
    errorMsg.value = '请输入账号和密码';
    return;
  }
  errorMsg.value = '';
  pwdSubmitting.value = true;
  try {
    const res = await loginWithPassword(ident, pwdPassword.value);
    emit('authenticated', res);
  } catch (e) {
    errorMsg.value = e instanceof AccountApiError ? e.message : '登录失败，请稍后重试';
  } finally {
    pwdSubmitting.value = false;
  }
}

watch(regUsername, (val) => {
  if (usernameTimer) window.clearTimeout(usernameTimer);
  const trimmed = val.trim();
  if (!trimmed) {
    usernameCheckStatus.value = 'idle';
    usernameCheckMsg.value = '';
    return;
  }
  const localErr = validateUsernameLocal(trimmed);
  if (localErr) {
    usernameCheckStatus.value = 'bad';
    usernameCheckMsg.value = localErr;
    return;
  }
  usernameCheckStatus.value = 'checking';
  usernameCheckMsg.value = '正在检查用户名是否可用…';
  usernameTimer = window.setTimeout(async () => {
    try {
      const check = await checkUsernameAvailable(trimmed);
      if (regUsername.value.trim() !== trimmed) return;
      if (check.available) {
        usernameCheckStatus.value = 'ok';
        usernameCheckMsg.value = '✓ 该用户名可以使用';
      } else {
        usernameCheckStatus.value = 'bad';
        usernameCheckMsg.value = check.message || '这个用户名已被占用，换一个吧';
      }
    } catch {
      usernameCheckStatus.value = 'idle';
      usernameCheckMsg.value = '';
    }
  }, 360);
});

async function handleRegisterSubmit() {
  if (regSubmitting.value) return;
  if (!regToken.value) {
    errorMsg.value = '请先在「验证码登录」验证邮箱，再完成建号';
    return;
  }
  const uErr = validateUsernameLocal(regUsername.value);
  if (uErr) {
    errorMsg.value = uErr;
    return;
  }
  const pErr = validatePasswordLocal(regPassword.value, regUsername.value);
  if (pErr) {
    errorMsg.value = pErr;
    return;
  }
  errorMsg.value = '';
  regSubmitting.value = true;
  try {
    const res = await registerAccount(regToken.value, regUsername.value.trim(), regPassword.value);
    emit('authenticated', res);
  } catch (e) {
    if (e instanceof AccountApiError) {
      errorMsg.value = e.message;
      if (e.code === 'registration_expired') {
        regToken.value = '';
      }
    } else {
      errorMsg.value = '建号失败，请稍后重试';
    }
  } finally {
    regSubmitting.value = false;
  }
}

async function handleSendResetCode() {
  if (resetSending.value || resetCooldown.value > 0) return;
  const ident = resetIdentifier.value.trim();
  if (!ident) {
    errorMsg.value = '请先输入已绑定的邮箱或手机号';
    return;
  }
  errorMsg.value = '';
  resetSending.value = true;
  try {
    const res = await sendVerificationCode(ident, 'reset_password');
    startCountdown('reset', res.cooldown_seconds || 60);
    emit('toast', '若该账号存在，重置验证码已发送');
  } catch (e) {
    if (e instanceof AccountApiError) {
      errorMsg.value = e.message;
      if (e.retryAfter && e.retryAfter > 0) startCountdown('reset', e.retryAfter);
    } else {
      errorMsg.value = '发送失败，请稍后重试';
    }
  } finally {
    resetSending.value = false;
  }
}

async function handleResetSubmit() {
  if (resetSubmitting.value) return;
  if (!resetIdentifier.value.trim() || !resetCode.value.trim()) {
    errorMsg.value = '请输入邮箱/手机号和验证码';
    return;
  }
  const pErr = validatePasswordLocal(resetNewPassword.value);
  if (pErr) {
    errorMsg.value = pErr;
    return;
  }
  errorMsg.value = '';
  resetSubmitting.value = true;
  try {
    await resetPassword(resetIdentifier.value.trim(), resetCode.value.trim(), resetNewPassword.value);
    pwdIdentifier.value = resetIdentifier.value.trim();
    resetCode.value = '';
    resetNewPassword.value = '';
    switchMode('pwd');
    emit('toast', '密码重置成功，请使用新密码登录');
  } catch (e) {
    errorMsg.value = e instanceof AccountApiError ? e.message : '重置失败，请稍后重试';
  } finally {
    resetSubmitting.value = false;
  }
}
</script>

<template>
  <div class="auth-page">
    <header class="topbar">
      <button class="icon-btn" aria-label="返回" @click="emit('back')">
        <svg viewBox="0 0 24 24"><path d="M15 18l-6-6 6-6" /></svg>
      </button>
      <span class="topbar-title">相伴统一账号</span>
      <span style="width: 36px" />
    </header>

    <div class="auth-pad">
      <!-- 黑胶潮玩抬头卡 -->
      <section class="auth-hero">
        <span class="sticker" style="background: var(--pop-lime)">ONE ACCOUNT · 全端通用</span>
        <h2>{{ heroMeta.title }}</h2>
        <p>{{ heroMeta.desc }}</p>
      </section>

      <!-- 三态切换条 -->
      <div class="state-seg" role="tablist" aria-label="登录与注册方式">
        <button
          role="tab"
          :aria-selected="mode === 'code'"
          :class="{ active: mode === 'code' }"
          @click="switchMode('code')"
        >
          验证码登录
        </button>
        <button
          role="tab"
          :aria-selected="mode === 'pwd' || mode === 'reset'"
          :class="{ active: mode === 'pwd' || mode === 'reset' }"
          @click="switchMode('pwd')"
        >
          密码登录
        </button>
        <button
          role="tab"
          :aria-selected="mode === 'register'"
          :class="{ active: mode === 'register' }"
          @click="switchMode('register')"
        >
          新号建档
        </button>
      </div>

      <!-- 错误提示条 -->
      <div v-if="errorMsg" class="auth-error" role="alert">
        {{ errorMsg }}
      </div>

      <!-- 面板 1：验证码登录 -->
      <form v-if="mode === 'code'" class="auth-card" @submit.prevent="handleCodeSubmit">
        <div>
          <label class="field-lbl" for="code-ident">{{ codeMethodLabel }}（按账号中心开放方式）</label>
          <input
            id="code-ident"
            v-model="codeIdentifier"
            class="field-inp"
            :type="methods.phone ? 'text' : 'email'"
            :placeholder="codeMethodPlaceholder"
            autocomplete="email"
          />
        </div>
        <div>
          <label class="field-lbl" for="code-val">6 位验证码</label>
          <div class="code-row">
            <input
              id="code-val"
              v-model="codeInput"
              class="field-inp mono"
              inputmode="numeric"
              maxlength="6"
              placeholder="输入 6 位验证码"
              autocomplete="one-time-code"
            />
            <button
              type="button"
              class="code-btn"
              :disabled="codeSending || codeCooldown > 0"
              @click="handleSendCode"
            >
              {{
                codeSending
                  ? '发送中…'
                  : codeCooldown > 0
                    ? `${codeCooldown}s 后重发`
                    : '获取验证码'
              }}
            </button>
          </div>
        </div>
        <p class="field-hint">
          已有账号直接登录；首次使用的邮箱验证后进入「设置用户名与密码」
        </p>
        <button type="submit" class="btn-pop submit-btn" :disabled="codeSubmitting">
          {{ codeSubmitting ? '正在验证…' : '验证并登录 →' }}
        </button>
        <div class="auth-foot-links">
          <span>{{ codeMethodFoot }}</span>
          <button type="button" class="link-btn" @click="switchMode('pwd')">改用密码登录</button>
        </div>
      </form>

      <!-- 面板 2：密码登录 -->
      <form v-else-if="mode === 'pwd'" class="auth-card" @submit.prevent="handlePwdSubmit">
        <div>
          <label class="field-lbl" for="pwd-ident">用户名 / 邮箱 / 手机号</label>
          <input
            id="pwd-ident"
            v-model="pwdIdentifier"
            class="field-inp"
            placeholder="输入相伴用户名、邮箱或手机号"
            autocomplete="username"
          />
        </div>
        <div>
          <label class="field-lbl" for="pwd-secret">登录密码</label>
          <div class="pwd-wrap">
            <input
              id="pwd-secret"
              v-model="pwdPassword"
              class="field-inp"
              :type="showPwd ? 'text' : 'password'"
              placeholder="输入 8-128 位密码"
              autocomplete="current-password"
            />
            <button type="button" class="pwd-eye" @click="showPwd = !showPwd">
              {{ showPwd ? '隐藏' : '显示' }}
            </button>
          </div>
        </div>
        <p class="field-hint">密码连续输错 5 次将临时锁定 15 分钟，可随时改用验证码登录。</p>
        <button type="submit" class="btn-pop submit-btn" :disabled="pwdSubmitting">
          {{ pwdSubmitting ? '登录中…' : '立即登录 →' }}
        </button>
        <div class="auth-foot-links">
          <button type="button" class="link-btn" @click="switchMode('reset')">忘记密码？</button>
          <button type="button" class="link-btn" @click="switchMode('code')">用验证码登录</button>
        </div>
      </form>

      <!-- 面板 3：新号建档 -->
      <form
        v-else-if="mode === 'register'"
        class="auth-card"
        @submit.prevent="handleRegisterSubmit"
      >
        <div v-if="regToken" class="verified-pill">
          ✓ 已验证：{{ regVerifiedIdentifier }}（请在 15 分钟内完成建号）
        </div>
        <div v-else class="need-verify-box">
          <b>新用户两步极速建号</b>
          <p>
            请先在「验证码登录」输入邮箱并完成 6 位验证码验证，通过后将自动回到这里设置唯一用户名与密码。
          </p>
          <button type="button" class="btn-pop submit-btn" @click="switchMode('code')">
            先去验证邮箱 →
          </button>
        </div>

        <template v-if="regToken">
          <div>
            <label class="field-lbl" for="reg-username">设置全局唯一用户名（注册后不可修改）</label>
            <input
              id="reg-username"
              v-model="regUsername"
              class="field-inp"
              maxlength="20"
              placeholder="汉字或字母开头，如：林小满"
              autocomplete="username"
            />
            <p
              v-if="usernameCheckMsg"
              class="field-hint"
              :class="{
                'hint-ok': usernameCheckStatus === 'ok',
                'hint-bad': usernameCheckStatus === 'bad'
              }"
            >
              {{ usernameCheckMsg }}
            </p>
            <p v-else class="field-hint">
              含中文至少 2 字，纯英文至少 3 字母，最多 20 字符（汉字算 2），不设额外昵称。
            </p>
          </div>

          <div>
            <label class="field-lbl" for="reg-pwd">设置登录密码</label>
            <input
              id="reg-pwd"
              v-model="regPassword"
              class="field-inp"
              type="password"
              placeholder="8-128 位字符，不能与用户名相同"
              autocomplete="new-password"
            />
          </div>

          <button type="submit" class="btn-pop submit-btn" :disabled="regSubmitting">
            {{ regSubmitting ? '正在建号…' : '完成注册并登录 →' }}
          </button>
        </template>
      </form>

      <!-- 面板 4：忘记密码（重置） -->
      <form v-else class="auth-card" @submit.prevent="handleResetSubmit">
        <div>
          <label class="field-lbl" for="reset-ident">已绑定的邮箱 / 手机号</label>
          <input
            id="reset-ident"
            v-model="resetIdentifier"
            class="field-inp"
            placeholder="输入账号绑定的邮箱或手机号"
          />
        </div>
        <div>
          <label class="field-lbl" for="reset-code">6 位重置验证码</label>
          <div class="code-row">
            <input
              id="reset-code"
              v-model="resetCode"
              class="field-inp mono"
              inputmode="numeric"
              maxlength="6"
              placeholder="输入验证码"
            />
            <button
              type="button"
              class="code-btn"
              :disabled="resetSending || resetCooldown > 0"
              @click="handleSendResetCode"
            >
              {{
                resetSending
                  ? '发送中…'
                  : resetCooldown > 0
                    ? `${resetCooldown}s 后重发`
                    : '获取重置码'
              }}
            </button>
          </div>
        </div>
        <div>
          <label class="field-lbl" for="reset-pwd">设置新密码</label>
          <input
            id="reset-pwd"
            v-model="resetNewPassword"
            class="field-inp"
            type="password"
            placeholder="8-128 位新密码"
            autocomplete="new-password"
          />
        </div>
        <button type="submit" class="btn-pop submit-btn" :disabled="resetSubmitting">
          {{ resetSubmitting ? '正在重置…' : '重置密码 →' }}
        </button>
        <div class="auth-foot-links">
          <span>重置后其他设备会自动下线</span>
          <button type="button" class="link-btn" @click="switchMode('pwd')">返回密码登录</button>
        </div>
      </form>

      <!-- 零门槛承诺提示卡 -->
      <div class="auth-callout">
        <template v-if="localStickerCount > 0">
          ✦ 游客模式随时可测，登录后自动将本机已集齐的 <b>{{ localStickerCount }} 枚贴纸</b> 关联至相伴账号，绝不丢失。
        </template>
        <template v-else>
          ✦ 游客模式随时可测，登录相伴统一账号即可在人格贩卖机与小伴社区（chat.xiangban.xyz）畅行通用。
        </template>
      </div>

      <p class="auth-legal">
        由相伴统一账号中心（account.xiangban.xyz）提供安全鉴权<br />
        登录即表示同意《用户协议》与《隐私说明》
      </p>
    </div>
  </div>
</template>

<style scoped>
.auth-page {
  min-height: 100%;
  display: flex;
  flex-direction: column;
}
.auth-pad {
  padding: 6px 16px 28px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.auth-hero {
  padding: 16px;
  border-radius: 20px;
  background: #18181b;
  color: #fffdf8;
  border: 2px solid #18181b;
  box-shadow: 4px 4px 0 var(--pop-lime);
  position: relative;
  overflow: hidden;
}
.auth-hero::after {
  content: '';
  position: absolute;
  right: -26px;
  top: -26px;
  width: 112px;
  height: 112px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(228, 248, 87, 0.34), transparent 70%);
  pointer-events: none;
}
.auth-hero h2 {
  font-size: 21px;
  font-weight: 950;
  margin-top: 8px;
  letter-spacing: -0.3px;
}
.auth-hero p {
  font-size: 12px;
  color: #cfcbd8;
  margin-top: 5px;
  line-height: 1.5;
}

.state-seg {
  display: flex;
  gap: 6px;
  background: var(--surface);
  padding: 4px;
  border-radius: 14px;
  border: 1.6px solid var(--border);
}
.state-seg button {
  flex: 1;
  height: 32px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 800;
  color: var(--muted);
}
.state-seg button.active {
  background: #18181b;
  color: var(--pop-lime);
}
:global(.dark) .state-seg button.active {
  background: var(--pop-lime);
  color: #18181b;
}

.auth-error {
  padding: 10px 13px;
  border-radius: 14px;
  background: #ffe3e6;
  color: #b42333;
  border: 1.6px solid #18181b;
  box-shadow: 2px 2px 0 #18181b;
  font-size: 12px;
  font-weight: 800;
}

.auth-card {
  padding: 16px;
  border-radius: 20px;
  background: var(--surface);
  border: 2px solid var(--border);
  box-shadow: var(--shadow-lg);
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.field-lbl {
  display: block;
  font-size: 11.5px;
  font-weight: 850;
  color: var(--ink-2);
  margin-bottom: 5px;
}
.field-inp {
  width: 100%;
  height: 44px;
  padding: 0 12px;
  border-radius: 13px;
  background: var(--sub);
  border: 1.8px solid var(--border);
  font-size: 14px;
  font-weight: 700;
  color: var(--ink);
}
.field-inp::placeholder {
  color: var(--muted);
  font-weight: 550;
}
.code-row {
  display: flex;
  gap: 8px;
}
.code-row .field-inp {
  flex: 1;
  min-width: 0;
}
.code-btn {
  flex-shrink: 0;
  padding: 0 12px;
  height: 44px;
  border-radius: 13px;
  background: var(--pop-lime);
  color: #18181b;
  border: 1.8px solid #18181b;
  box-shadow: 2.5px 2.5px 0 #18181b;
  font-size: 12px;
  font-weight: 900;
  white-space: nowrap;
}
.code-btn:active {
  transform: translate(1.5px, 1.5px);
  box-shadow: none;
}
.code-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

.pwd-wrap {
  position: relative;
}
.pwd-wrap .field-inp {
  padding-right: 54px;
}
.pwd-eye {
  position: absolute;
  right: 10px;
  top: 50%;
  transform: translateY(-50%);
  font-size: 11.5px;
  font-weight: 800;
  color: var(--muted);
  padding: 4px 6px;
}

.field-hint {
  font-size: 11px;
  color: var(--muted);
  line-height: 1.5;
  margin-top: 4px;
}
.hint-ok {
  color: #147a4b;
  font-weight: 800;
}
.hint-bad {
  color: var(--pop-red);
  font-weight: 800;
}

.submit-btn {
  width: 100%;
  height: 45px;
  font-size: 14.5px;
}
.submit-btn:disabled {
  opacity: 0.65;
  cursor: not-allowed;
}

.auth-foot-links {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 11.5px;
  color: var(--muted);
  font-weight: 700;
}
.link-btn {
  color: var(--ink);
  font-weight: 850;
  text-decoration: underline;
  font-size: 11.5px;
}

.verified-pill {
  padding: 8px 11px;
  border-radius: 12px;
  background: var(--pop-mint);
  color: #18181b;
  border: 1.5px solid #18181b;
  font-size: 11.5px;
  font-weight: 850;
}
.need-verify-box {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.need-verify-box b {
  font-size: 14px;
  font-weight: 900;
}
.need-verify-box p {
  font-size: 12px;
  color: var(--muted);
  line-height: 1.55;
}

.auth-callout {
  padding: 11px 13px;
  border-radius: 14px;
  background: var(--pop-mint);
  color: #18181b;
  border: 1.6px solid #18181b;
  box-shadow: 2.5px 2.5px 0 #18181b;
  font-size: 11.5px;
  font-weight: 800;
  line-height: 1.5;
}
.auth-legal {
  font-size: 10.5px;
  color: var(--muted);
  text-align: center;
  line-height: 1.6;
}
</style>
