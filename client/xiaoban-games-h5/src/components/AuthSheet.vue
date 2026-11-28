<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import {
  checkUsernameAvailable,
  completeRegistration,
  fetchAuthMethods,
  loginWithCode,
  loginWithPassword,
  resetPassword,
  sendVerifyCode,
  type AuthMethods,
  type AuthSession
} from '../lib/accountApi';
import UiIcon from './UiIcon.vue';

const props = defineProps<{
  savedCount: number;
}>();

const emit = defineEmits<{
  close: [];
  success: [session: AuthSession, message: string];
  toast: [msg: string];
}>();

type AuthMode = 'code' | 'pwd' | 'register' | 'reset';

const mode = ref<AuthMode>('code');
const methods = ref<AuthMethods>({ email: true, phone: false });
const identifier = ref('');
const code = ref('');
const password = ref('');
const username = ref('');
const registrationToken = ref('');
const showPwd = ref(false);
const agree = ref(true);
const shakeAgree = ref(false);
const loading = ref(false);
const sending = ref(false);
const cooldown = ref(0);
const errorText = ref('');
const usernameHint = ref('');
const usernameOk = ref(false);

let cooldownTimer: number | undefined;
let checkTimer: number | undefined;

const isSubFlow = computed(() => mode.value === 'register' || mode.value === 'reset');
const codePlaceholder = computed(() =>
  methods.value.phone ? '输入邮箱或手机号' : '输入邮箱地址'
);
const heading = computed(() => {
  if (mode.value === 'register') return '设置用户名与密码';
  if (mode.value === 'reset') return '重置账号密码';
  return '登录相伴账号';
});
const subtitle = computed(() => {
  if (mode.value === 'register') return '新邮箱已验证 · 完成后即刻建号';
  if (mode.value === 'reset') return '验证绑定邮箱或手机号后设置新密码';
  return '全端通用 · 跨小伴桌面、手机与社区';
});
const submitLabel = computed(() => {
  if (loading.value) return '请稍候…';
  if (mode.value === 'code') return '登录 / 注册';
  if (mode.value === 'pwd') return '立即登录';
  if (mode.value === 'register') return '完成注册并登录';
  return '重置密码并登录';
});

function switchMode(next: AuthMode) {
  mode.value = next;
  errorText.value = '';
  showPwd.value = false;
}

function startCooldown(sec = 60) {
  cooldown.value = sec;
  window.clearInterval(cooldownTimer);
  cooldownTimer = window.setInterval(() => {
    cooldown.value = Math.max(0, cooldown.value - 1);
    if (cooldown.value === 0) window.clearInterval(cooldownTimer);
  }, 1000);
}

async function handleSendCode() {
  if (cooldown.value > 0 || sending.value) return;
  const id = identifier.value.trim();
  if (!id) {
    errorText.value = `请先${codePlaceholder.value}`;
    return;
  }
  errorText.value = '';
  sending.value = true;
  try {
    await sendVerifyCode(id, mode.value === 'reset' ? 'reset_password' : 'login');
    startCooldown(60);
    emit('toast', '验证码已发送，请查收');
  } catch (e) {
    const err = e as Error & { retryAfter?: number };
    if (err.retryAfter) startCooldown(err.retryAfter);
    errorText.value = err.message || '发送失败，请稍后重试';
  } finally {
    sending.value = false;
  }
}

watch(username, (val) => {
  if (mode.value !== 'register') return;
  window.clearTimeout(checkTimer);
  const trimmed = val.trim();
  if (!trimmed) {
    usernameHint.value = '注册后不可修改，将作为全站昵称';
    usernameOk.value = false;
    return;
  }
  usernameHint.value = '正在检查用户名可用性…';
  usernameOk.value = false;
  checkTimer = window.setTimeout(async () => {
    try {
      const res = await checkUsernameAvailable(trimmed);
      usernameOk.value = res.available;
      usernameHint.value = res.message;
    } catch {
      usernameHint.value = '支持中英文、数字与下划线，注册后不可修改';
    }
  }, 360);
});

async function handleSubmit() {
  if (loading.value) return;
  errorText.value = '';

  if (mode.value !== 'reset' && !agree.value) {
    shakeAgree.value = true;
    errorText.value = '请先阅读并勾选《用户协议》与《隐私政策》';
    window.setTimeout(() => { shakeAgree.value = false; }, 320);
    return;
  }

  loading.value = true;
  try {
    if (mode.value === 'code') {
      if (!identifier.value.trim() || !code.value.trim()) {
        errorText.value = '请输入标识与 6 位验证码';
        return;
      }
      const res = await loginWithCode(identifier.value, code.value);
      if (res.registrationRequired && res.registrationToken) {
        registrationToken.value = res.registrationToken;
        switchMode('register');
        usernameHint.value = '注册后不可修改，将作为全站昵称';
        return;
      }
      if (res.session) {
        emit('success', res.session, `欢迎回来，${res.session.user.username}`);
      }
    } else if (mode.value === 'register') {
      if (!username.value.trim() || password.value.length < 8) {
        errorText.value = '请填写用户名并设置至少 8 位登录密码';
        return;
      }
      const session = await completeRegistration(registrationToken.value, username.value, password.value);
      emit('success', session, `欢迎加入相伴，${session.user.username}`);
    } else if (mode.value === 'pwd') {
      if (!identifier.value.trim() || !password.value) {
        errorText.value = '请输入账号和密码';
        return;
      }
      const session = await loginWithPassword(identifier.value, password.value);
      emit('success', session, `已登录相伴账号（${session.user.username}）`);
    } else {
      if (!identifier.value.trim() || !code.value.trim() || password.value.length < 8) {
        errorText.value = '请填写邮箱/手机、验证码与至少 8 位新密码';
        return;
      }
      await resetPassword(identifier.value, code.value, password.value);
      const session = await loginWithPassword(identifier.value, password.value);
      emit('success', session, '密码已重置并登录成功');
    }
  } catch (e) {
    errorText.value = (e as Error).message || '操作失败，请稍后重试';
  } finally {
    loading.value = false;
  }
}

onMounted(async () => {
  methods.value = await fetchAuthMethods();
});

onBeforeUnmount(() => {
  window.clearInterval(cooldownTimer);
  window.clearTimeout(checkTimer);
});
</script>

<template>
  <div class="scrim" role="dialog" aria-modal="true" aria-label="登录相伴账号" @click="emit('close')">
    <div class="sheet auth-sheet" @click.stop>
      <div class="sheet-handle" />

      <button
        v-if="isSubFlow"
        type="button"
        class="auth-back-link"
        @click="switchMode(mode === 'reset' ? 'pwd' : 'code')"
      >
        <UiIcon name="back" :size="15" />返回{{ mode === 'reset' ? '密码登录' : '验证码登录' }}
      </button>

      <div class="auth-top">
        <div class="auth-brand">
          <div class="xb-mark lg"><UiIcon name="brand" :size="18" /></div>
          <div>
            <div class="auth-heading">{{ heading }}</div>
            <div class="auth-sub">{{ subtitle }}</div>
          </div>
        </div>
        <button type="button" class="auth-close" aria-label="关闭" @click="emit('close')">
          <UiIcon name="close" :size="16" />
        </button>
      </div>

      <div v-if="!isSubFlow" class="auth-tabs" role="tablist">
        <button type="button" role="tab" :class="{ active: mode === 'code' }" @click="switchMode('code')">验证码登录</button>
        <button type="button" role="tab" :class="{ active: mode === 'pwd' }" @click="switchMode('pwd')">密码登录</button>
      </div>

      <form @submit.prevent="handleSubmit">
        <template v-if="mode === 'code'">
          <div v-if="props.savedCount > 0" class="auth-sync-badge">
            <i /><span>登录后保留当前浏览器的 <b>{{ props.savedCount }} 份本机存档</b>，不丢进度</span>
          </div>
          <div class="auth-field">
            <div class="auth-input-wrap">
              <input v-model="identifier" class="auth-input" type="text" autocomplete="username" :placeholder="codePlaceholder" />
            </div>
          </div>
          <div class="auth-field">
            <div class="auth-input-wrap">
              <input v-model="code" class="auth-input num" type="text" inputmode="numeric" maxlength="6" autocomplete="one-time-code" placeholder="6 位验证码" />
              <button type="button" class="auth-code-btn num" :disabled="cooldown > 0 || sending" @click="handleSendCode">
                {{ cooldown > 0 ? `${cooldown}s 后重发` : sending ? '发送中…' : '获取验证码' }}
              </button>
            </div>
            <div class="auth-field-hint">
              <span>未注册邮箱验证后将进入完善注册</span>
            </div>
          </div>
        </template>

        <template v-else-if="mode === 'pwd'">
          <div class="auth-field">
            <div class="auth-input-wrap">
              <input v-model="identifier" class="auth-input" type="text" autocomplete="username" placeholder="用户名 / 邮箱 / 手机号" />
            </div>
          </div>
          <div class="auth-field">
            <div class="auth-input-wrap">
              <input v-model="password" class="auth-input" :type="showPwd ? 'text' : 'password'" autocomplete="current-password" placeholder="输入密码" />
              <button type="button" class="auth-eye-btn" aria-label="显示或隐藏密码" @click="showPwd = !showPwd">
                <UiIcon :name="showPwd ? 'eye' : 'eyeOff'" :size="17" />
              </button>
            </div>
            <div class="auth-field-hint">
              <span>支持与小伴桌面 / App 同一账号密码</span>
              <button type="button" @click="switchMode('reset')">忘记密码？</button>
            </div>
          </div>
        </template>

        <template v-else-if="mode === 'register'">
          <div class="auth-verified-bar">
            <span class="auth-verified-left"><i /><span>{{ identifier }} · 已验证</span></span>
            <button type="button" class="auth-verified-change" @click="switchMode('code')">换个邮箱</button>
          </div>
          <div class="auth-field">
            <div class="auth-input-wrap">
              <input v-model="username" class="auth-input" type="text" maxlength="20" placeholder="设置用户名（将作为全站昵称）" />
            </div>
            <div class="auth-field-hint" :class="{ ok: usernameOk }">
              <span>{{ usernameOk ? '✓ ' : '' }}{{ usernameHint || '注册后不可修改，将作为全站昵称' }}</span>
            </div>
          </div>
          <div class="auth-field">
            <div class="auth-input-wrap">
              <input v-model="password" class="auth-input" :type="showPwd ? 'text' : 'password'" autocomplete="new-password" placeholder="设置登录密码（8–32 位）" />
              <button type="button" class="auth-eye-btn" aria-label="显示或隐藏密码" @click="showPwd = !showPwd">
                <UiIcon :name="showPwd ? 'eye' : 'eyeOff'" :size="17" />
              </button>
            </div>
          </div>
        </template>

        <template v-else>
          <div class="auth-field">
            <div class="auth-input-wrap">
              <input v-model="identifier" class="auth-input" type="text" placeholder="已绑定的邮箱或手机号" />
            </div>
          </div>
          <div class="auth-field">
            <div class="auth-input-wrap">
              <input v-model="code" class="auth-input num" type="text" inputmode="numeric" maxlength="6" placeholder="6 位验证码" />
              <button type="button" class="auth-code-btn num" :disabled="cooldown > 0 || sending" @click="handleSendCode">
                {{ cooldown > 0 ? `${cooldown}s 后重发` : sending ? '发送中…' : '获取验证码' }}
              </button>
            </div>
          </div>
          <div class="auth-field">
            <div class="auth-input-wrap">
              <input v-model="password" class="auth-input" :type="showPwd ? 'text' : 'password'" autocomplete="new-password" placeholder="设置新密码（8–32 位）" />
              <button type="button" class="auth-eye-btn" aria-label="显示或隐藏密码" @click="showPwd = !showPwd">
                <UiIcon :name="showPwd ? 'eye' : 'eyeOff'" :size="17" />
              </button>
            </div>
          </div>
        </template>

        <p v-if="errorText" class="auth-error" role="alert">{{ errorText }}</p>

        <button
          v-if="mode !== 'reset'"
          type="button"
          class="auth-agree"
          :class="{ shake: shakeAgree }"
          @click="agree = !agree; errorText = ''"
        >
          <span class="auth-check" :class="{ checked: agree }">
            <UiIcon v-if="agree" name="check" :size="11" />
          </span>
          <span>已阅读并同意 <b>《用户协议》</b> 与 <b>《隐私政策》</b></span>
        </button>
        <div v-else style="height: 10px" />

        <button type="submit" class="auth-submit" :disabled="loading">{{ submitLabel }}</button>
      </form>

      <div class="auth-foot">
        <span class="mono">account.xiangban.xyz</span>
        <button type="button" @click="emit('close')">暂不登录</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.auth-sheet { padding-inline: 20px; }
.xb-mark.lg { width: 34px; height: 34px; border-radius: 10px; }
.auth-top { display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; margin-bottom: 14px; }
.auth-brand { display: flex; align-items: center; gap: 10px; min-width: 0; }
.auth-close {
  width: 32px; height: 32px; border-radius: 50%; flex: none;
  background: var(--fill); color: var(--ink2); display: grid; place-items: center;
}
.auth-heading { font-size: 18px; font-weight: 750; letter-spacing: -0.015em; line-height: 1.25; }
.auth-sub { font-size: 12px; color: var(--ink2); margin-top: 3px; }
.auth-back-link {
  display: inline-flex; align-items: center; gap: 3px; font-size: 12.5px; font-weight: 600;
  color: var(--ink2); margin-bottom: 10px;
}

.auth-tabs {
  display: grid; grid-template-columns: 1fr 1fr; gap: 3px; padding: 3px;
  border-radius: 12px; background: var(--fill); margin-bottom: 14px;
}
.auth-tabs button {
  height: 34px; border-radius: 9px; font-size: 13px; font-weight: 550; color: var(--ink2);
}
.auth-tabs button.active {
  background: var(--surface); color: var(--ink); font-weight: 700;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.08);
}

.auth-sync-badge {
  display: flex; align-items: center; gap: 6px; padding: 7px 10px; border-radius: 10px;
  background: var(--fill); font-size: 11.5px; color: var(--ink2); margin-bottom: 12px;
}
.auth-sync-badge i { width: 6px; height: 6px; border-radius: 50%; background: var(--ok); flex: none; }

.auth-verified-bar {
  display: flex; align-items: center; justify-content: space-between; gap: 8px;
  padding: 9px 12px; border-radius: 12px; background: var(--fill); margin-bottom: 12px; font-size: 12.5px;
}
.auth-verified-left { display: flex; align-items: center; gap: 6px; min-width: 0; font-weight: 600; }
.auth-verified-left i { width: 6px; height: 6px; border-radius: 50%; background: var(--ok); flex: none; }
.auth-verified-change { font-size: 12px; color: var(--ink2); font-weight: 600; flex: none; }

.auth-field { margin-bottom: 10px; }
.auth-input-wrap {
  height: 46px; border-radius: 12px; background: var(--fill);
  display: flex; align-items: center; gap: 8px; padding: 0 10px 0 14px;
  border: 1.5px solid transparent; transition: border-color 0.15s, background 0.15s;
}
.auth-input-wrap:focus-within { border-color: var(--accent); background: var(--surface); }
.auth-input { flex: 1; min-width: 0; height: 100%; font-size: 15px; color: var(--ink); outline: none; }
.auth-input::placeholder { color: var(--ink3); }
.auth-code-btn {
  height: 31px; padding: 0 10px; border-radius: 8px; flex: none;
  font-size: 12.5px; font-weight: 650; color: var(--accent);
  background: var(--surface); border: 1px solid var(--hairline);
}
.auth-code-btn:disabled { color: var(--ink3); cursor: default; opacity: 0.75; }
.auth-eye-btn { width: 32px; height: 32px; border-radius: 8px; display: grid; place-items: center; color: var(--ink3); flex: none; }
.auth-field-hint {
  font-size: 11.5px; color: var(--ink3); margin: 5px 2px 0;
  display: flex; align-items: center; justify-content: space-between; gap: 8px;
}
.auth-field-hint.ok { color: var(--ok); }
.auth-field-hint button { font-size: 12px; font-weight: 600; color: var(--ink2); }

.auth-error {
  font-size: 12.5px; color: var(--danger); margin: 6px 2px 4px; line-height: 1.45;
}

.auth-agree {
  display: flex; align-items: flex-start; gap: 8px; margin: 12px 2px 14px;
  font-size: 12px; line-height: 1.45; color: var(--ink2); text-align: left; width: 100%;
}
.auth-check {
  width: 16px; height: 16px; border-radius: 5px; flex: none; margin-top: 1px;
  border: 1.5px solid var(--ink3); display: grid; place-items: center;
}
.auth-check.checked { background: var(--accent); border-color: var(--accent); color: #fff; }
.auth-agree b { color: var(--ink); font-weight: 600; }

.auth-submit {
  width: 100%; height: 48px; border-radius: 999px;
  background: var(--accent); color: var(--accent-ink); font-size: 15px; font-weight: 700;
}
.auth-submit:disabled { opacity: 0.65; cursor: default; }
.auth-submit:active { transform: scale(0.98); }

.auth-foot {
  margin-top: 12px; display: flex; align-items: center; justify-content: space-between;
  font-size: 12px; color: var(--ink3);
}
.auth-foot button { color: var(--ink2); font-weight: 550; padding: 4px 0; }
.shake { animation: shakeX 0.28s ease; }
@keyframes shakeX {
  0%, 100% { transform: translateX(0); }
  20%, 60% { transform: translateX(-4px); }
  40%, 80% { transform: translateX(4px); }
}
</style>
