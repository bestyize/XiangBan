<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from "vue";
import { Mail, ShieldCheck, X, Loader2 } from "lucide-vue-next";
import type { ControlStore } from "../control-types.ts";

const props = defineProps<{
  store: ControlStore;
}>();

const email = ref("");
const code = ref("");
const sending = ref(false);
const submitting = ref(false);
const errorMessage = ref("");
const countdown = ref(0);
let timer: ReturnType<typeof setInterval> | null = null;

const emailInput = ref<HTMLInputElement>();
const codeInput = ref<HTMLInputElement>();

const isValidEmail = computed(() => {
  const trimmed = email.value.trim();
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(trimmed);
});

function clearCountdown() {
  if (timer) {
    clearInterval(timer);
    timer = null;
  }
  countdown.value = 0;
}

function close(force = false) {
  if (submitting.value && !force) return;
  errorMessage.value = "";
  props.store.closeLogin();
}

function handleClose() {
  close(false);
}

async function handleSendCode() {
  if (!isValidEmail.value || countdown.value > 0 || sending.value) return;
  errorMessage.value = "";
  sending.value = true;
  try {
    const res = await props.store.sendAccountCode(email.value.trim());
    countdown.value = res.cooldownSeconds || 60;
    timer = setInterval(() => {
      countdown.value--;
      if (countdown.value <= 0) {
        clearCountdown();
      }
    }, 1000);
    props.store.notify("验证码已发送，请查收邮件", "success");
    await nextTick();
    codeInput.value?.focus();
  } catch (error) {
    errorMessage.value = String(error);
  } finally {
    sending.value = false;
  }
}

async function handleLogin() {
  if (!isValidEmail.value) {
    errorMessage.value = "请输入有效的邮箱地址";
    emailInput.value?.focus();
    return;
  }
  if (!code.value.trim()) {
    errorMessage.value = "请输入验证码";
    codeInput.value?.focus();
    return;
  }

  errorMessage.value = "";
  submitting.value = true;
  try {
    const result = await props.store.loginAccount(email.value.trim(), code.value.trim());
    clearCountdown();
    close(true);
    if (result.isNewUser) {
      props.store.notify("欢迎新朋友！已为你自动创建账号并登录", "success");
    } else {
      props.store.notify(`欢迎回来，${result.session.user.nickname || "小伴伙伴"}！`, "success");
    }
  } catch (error) {
    errorMessage.value = String(error);
  } finally {
    submitting.value = false;
  }
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape" && props.store.showLoginModal) {
    close();
  }
}

window.addEventListener("keydown", onKeyDown);

watch(
  () => props.store.showLoginModal,
  async (show) => {
    if (show) {
      errorMessage.value = "";
      code.value = "";
      await nextTick();
      emailInput.value?.focus();
    } else {
      clearCountdown();
    }
  }
);

onBeforeUnmount(() => {
  clearCountdown();
  window.removeEventListener("keydown", onKeyDown);
});
</script>

<template>
  <Teleport to="body">
    <Transition name="login-fade">
      <div
        v-if="store.showLoginModal"
        class="login-overlay"
        role="dialog"
        aria-modal="true"
        aria-labelledby="login-modal-title"
        data-testid="login-modal-overlay"
        @click.self="handleClose"
      >
        <div class="login-card" data-testid="login-card">
          <button
            type="button"
            class="close-button"
            data-testid="login-close-btn"
            title="关闭 (Esc)"
            @click="handleClose"
          >
            <X :size="18" />
          </button>

          <div class="card-header">
            <div class="brand-avatar">
              <img src="/xiaoban.svg" alt="小伴" />
            </div>
            <h2 id="login-modal-title">登录小伴 / 快速注册</h2>
            <p class="subtitle">未注册邮箱验证后自动创建账号，已注册直接登录</p>
          </div>

          <form class="login-form" @submit.prevent="handleLogin">
            <div v-if="errorMessage" class="error-banner" role="alert" data-testid="login-error-msg">
              {{ errorMessage }}
            </div>

            <div class="input-group">
              <label for="login-email">邮箱地址</label>
              <div class="input-wrapper">
                <Mail :size="18" class="input-icon" />
                <input
                  id="login-email"
                  ref="emailInput"
                  v-model="email"
                  type="email"
                  data-testid="login-email-input"
                  placeholder="name@example.com"
                  autocomplete="email"
                  :disabled="submitting"
                  @keyup.enter="codeInput?.focus()"
                />
              </div>
            </div>

            <div class="input-group">
              <label for="login-code">验证码</label>
              <div class="code-wrapper">
                <div class="input-wrapper flex-1">
                  <ShieldCheck :size="18" class="input-icon" />
                  <input
                    id="login-code"
                    ref="codeInput"
                    v-model="code"
                    type="text"
                    data-testid="login-code-input"
                    placeholder="请输入 6 位验证码"
                    maxlength="8"
                    autocomplete="one-time-code"
                    :disabled="submitting"
                    @keyup.enter="handleLogin"
                  />
                </div>
                <button
                  type="button"
                  class="code-send-btn"
                  data-testid="send-code-btn"
                  :disabled="!isValidEmail || countdown > 0 || sending || submitting"
                  @click="handleSendCode"
                >
                  <Loader2 v-if="sending" :size="14" class="spin" />
                  <span v-else-if="countdown > 0">{{ countdown }}s 后重发</span>
                  <span v-else>获取验证码</span>
                </button>
              </div>
            </div>

            <button
              type="submit"
              class="login-submit-btn"
              data-testid="login-submit-btn"
              :disabled="submitting"
            >
              <Loader2 v-if="submitting" :size="18" class="spin" />
              <span v-else>进入小伴生活空间</span>
            </button>

            <div class="policy-note">
              登录即代表你已阅读并同意
              <span class="policy-link">《用户协议》</span>
              与
              <span class="policy-link">《隐私政策》</span>
            </div>
          </form>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.login-overlay {
  position: fixed;
  inset: 0;
  z-index: 3500;
  background: rgba(10, 10, 14, 0.65);
  backdrop-filter: blur(8px);
  display: grid;
  place-items: center;
  padding: 20px;
}

.login-card {
  position: relative;
  width: 100%;
  max-width: 410px;
  background: var(--ui-surface);
  border: 1px solid var(--ui-border);
  border-radius: 24px;
  box-shadow: 0 28px 70px rgba(0, 0, 0, 0.35), 0 2px 10px rgba(0, 0, 0, 0.1);
  padding: 32px 30px 26px;
  color: var(--ui-text-primary);
  user-select: none;
}

.close-button {
  position: absolute;
  top: 18px;
  right: 18px;
  width: 32px;
  height: 32px;
  display: grid;
  place-items: center;
  border: 0;
  background: transparent;
  color: var(--ui-text-tertiary);
  border-radius: 50%;
  cursor: pointer;
  transition: all 0.15s ease;
}

.close-button:hover {
  background: var(--ui-surface-alt);
  color: var(--ui-text-primary);
}

.card-header {
  text-align: center;
  margin-bottom: 24px;
}

.brand-avatar {
  width: 58px;
  height: 58px;
  margin: 0 auto 12px;
  display: grid;
  place-items: center;
  background: var(--ui-primary-soft);
  border-radius: 18px;
}

.brand-avatar img {
  width: 42px;
  height: 42px;
}

.card-header h2 {
  font-size: 20px;
  font-weight: 700;
  margin: 0 0 6px;
  letter-spacing: -0.01em;
}

.subtitle {
  font-size: 13px;
  color: var(--ui-text-tertiary);
  margin: 0;
  line-height: 1.5;
}

.login-form {
  display: grid;
  gap: 16px;
}

.error-banner {
  background: color-mix(in srgb, #ef4444 14%, var(--ui-surface));
  color: #ef4444;
  border: 1px solid color-mix(in srgb, #ef4444 30%, transparent);
  border-radius: 12px;
  padding: 10px 14px;
  font-size: 13px;
  line-height: 1.4;
}

.input-group {
  display: grid;
  gap: 6px;
}

.input-group label {
  font-size: 13px;
  font-weight: 600;
  color: var(--ui-text-secondary);
}

.input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  background: var(--ui-surface-alt);
  border: 1px solid var(--ui-border);
  border-radius: 14px;
  transition: border-color 0.15s, box-shadow 0.15s;
}

.input-wrapper:focus-within {
  border-color: var(--ui-primary);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--ui-primary) 18%, transparent);
}

.input-icon {
  margin-left: 12px;
  color: var(--ui-text-tertiary);
  flex-shrink: 0;
}

.input-wrapper input {
  width: 100%;
  border: 0;
  background: transparent;
  padding: 12px 14px 12px 10px;
  color: var(--ui-text-primary);
  font-size: 14px;
  outline: none;
}

.input-wrapper input::placeholder {
  color: var(--ui-text-tertiary);
}

.code-wrapper {
  display: flex;
  gap: 10px;
  align-items: center;
}

.flex-1 {
  flex: 1;
}

.code-send-btn {
  flex-shrink: 0;
  height: 44px;
  min-width: 104px;
  padding: 0 14px;
  border: 1px solid var(--ui-border);
  background: var(--ui-surface-alt);
  color: var(--ui-text-primary);
  border-radius: 14px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  display: grid;
  place-items: center;
  transition: all 0.15s;
}

.code-send-btn:hover:not(:disabled) {
  border-color: var(--ui-primary);
  color: var(--ui-primary);
}

.code-send-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.login-submit-btn {
  margin-top: 6px;
  width: 100%;
  height: 46px;
  border: 0;
  border-radius: 14px;
  background: var(--ui-primary);
  color: #ffffff;
  font-size: 15px;
  font-weight: 650;
  cursor: pointer;
  display: grid;
  place-items: center;
  transition: opacity 0.15s, transform 0.1s;
  box-shadow: 0 4px 16px color-mix(in srgb, var(--ui-primary) 35%, transparent);
}

.login-submit-btn:hover:not(:disabled) {
  opacity: 0.94;
}

.login-submit-btn:active:not(:disabled) {
  transform: scale(0.99);
}

.login-submit-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.policy-note {
  text-align: center;
  font-size: 11px;
  color: var(--ui-text-tertiary);
  margin-top: 4px;
  line-height: 1.5;
}

.policy-link {
  color: var(--ui-text-secondary);
  cursor: pointer;
}

.policy-link:hover {
  text-decoration: underline;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.login-fade-enter-active,
.login-fade-leave-active {
  transition: opacity 0.2s ease;
}

.login-fade-enter-from,
.login-fade-leave-to {
  opacity: 0;
}

.login-fade-enter-active .login-card {
  animation: cardIn 0.22s cubic-bezier(0.16, 1, 0.3, 1);
}

.login-fade-leave-active .login-card {
  animation: cardOut 0.18s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes cardIn {
  from {
    transform: scale(0.92) translateY(8px);
    opacity: 0;
  }
  to {
    transform: scale(1) translateY(0);
    opacity: 1;
  }
}

@keyframes cardOut {
  from {
    transform: scale(1) translateY(0);
    opacity: 1;
  }
  to {
    transform: scale(0.94) translateY(6px);
    opacity: 0;
  }
}
</style>
