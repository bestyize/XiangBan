<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from "vue";
import { KeyRound, Mail, ShieldCheck, UserRound, X, Loader2 } from "lucide-vue-next";
import { toAccountError } from "../../bridge.ts";
import type { ControlStore } from "../control-types.ts";

const props = defineProps<{
  store: ControlStore;
}>();

type Mode = "password" | "code" | "reset";
type Step = "identify" | "verify" | "setup";
type Availability = { kind: "idle" | "checking" | "ok" } | { kind: "taken" | "invalid"; message: string };

const mode = ref<Mode>("password");
const step = ref<Step>("identify");
const identifier = ref("");
const password = ref("");
const code = ref("");
const username = ref("");
const showPassword = ref(false);
const sending = ref(false);
const submitting = ref(false);
const errorMessage = ref("");
const noticeMessage = ref("");
const countdown = ref(0);
const availability = ref<Availability>({ kind: "idle" });
let registrationToken: string | null = null;
let timer: ReturnType<typeof setInterval> | null = null;
let checkTimer: ReturnType<typeof setTimeout> | null = null;

const identifierInput = ref<HTMLInputElement>();
const codeInput = ref<HTMLInputElement>();

const codeSent = computed(() => step.value === "verify");
const codeValid = computed(() => /^\d{6}$/.test(code.value.trim()));

function usernameProblem(raw: string): string {
  const value = raw.trim();
  if (!value) return "";
  if (!/^[A-Za-z]/.test(value)) return "用户名需以字母开头";
  if (!/^[A-Za-z0-9_]+$/.test(value)) return "只能包含字母、数字和下划线";
  if (value.length < 4 || value.length > 20) return "需为 4-20 个字符";
  return "";
}

function passwordProblemOf(raw: string, name = ""): string {
  if (!raw) return "";
  if (raw.length < 8) return "密码至少 8 位";
  if (raw.length > 128) return "密码最多 128 位";
  if (name && raw.toLowerCase() === name.trim().toLowerCase()) return "密码不能和用户名相同";
  return "";
}

const passwordProblem = computed(() => passwordProblemOf(password.value, mode.value === "reset" ? "" : username.value));
const hint = computed(() => {
  const a = availability.value;
  if (a.kind === "checking") return "正在检查…";
  if (a.kind === "ok") return "可以使用";
  return a.kind === "taken" || a.kind === "invalid" ? a.message : "";
});
const canSubmit = computed(() => {
  if (mode.value === "password") return !!identifier.value.trim() && !!password.value;
  if (mode.value === "reset") return codeSent.value && codeValid.value && !!password.value && !passwordProblem.value;
  if (step.value === "setup") {
    const kind = availability.value.kind;
    return !!username.value.trim() && !["taken", "invalid", "checking"].includes(kind) && !!password.value && !passwordProblem.value;
  }
  return codeSent.value && codeValid.value;
});

function clearCountdown() {
  if (timer) {
    clearInterval(timer);
    timer = null;
  }
  countdown.value = 0;
}

function switchMode(next: Mode) {
  mode.value = next;
  step.value = "identify";
  code.value = "";
  password.value = "";
  errorMessage.value = "";
  noticeMessage.value = "";
  registrationToken = null;
  clearCountdown();
}

function changeIdentifier() {
  step.value = "identify";
  code.value = "";
  password.value = "";
  errorMessage.value = "";
  noticeMessage.value = "";
  clearCountdown();
}

function close(force = false) {
  if (submitting.value && !force) return;
  errorMessage.value = "";
  props.store.closeLogin();
}

function handleClose() {
  close(false);
}

// While choosing a username, check it as the user types. It is only a hint: the server enforces uniqueness.
watch([username, step], () => {
  if (checkTimer) clearTimeout(checkTimer);
  if (step.value !== "setup") return;
  const name = username.value.trim();
  if (!name) {
    availability.value = { kind: "idle" };
    return;
  }
  const local = usernameProblem(name);
  if (local) {
    availability.value = { kind: "invalid", message: local };
    return;
  }
  availability.value = { kind: "checking" };
  checkTimer = setTimeout(async () => {
    try {
      const res = await props.store.checkAccountUsername(name);
      if (username.value.trim() === name) {
        availability.value = res.available ? { kind: "ok" } : { kind: "taken", message: res.message || "这个用户名已被占用" };
      }
    } catch {
      if (username.value.trim() === name) availability.value = { kind: "idle" };
    }
  }, 350);
});

async function handleSendCode() {
  if (!identifier.value.trim() || countdown.value > 0 || sending.value) return;
  errorMessage.value = "";
  sending.value = true;
  try {
    const res = await props.store.sendAccountCode(identifier.value.trim(), mode.value === "reset" ? "reset_password" : "login");
    step.value = "verify";
    countdown.value = res.cooldownSeconds || 60;
    timer = setInterval(() => {
      countdown.value--;
      if (countdown.value <= 0) clearCountdown();
    }, 1000);
    props.store.notify("验证码已发送，请查收", "success");
    await nextTick();
    codeInput.value?.focus();
  } catch (error) {
    errorMessage.value = toAccountError(error).message;
  } finally {
    sending.value = false;
  }
}

function welcome(result: { isNewUser: boolean; session: { user: { username: string } } }) {
  clearCountdown();
  close(true);
  const name = result.session.user.username;
  props.store.notify(result.isNewUser ? `注册成功，欢迎你，${name}` : `欢迎回来，${name}`, "success");
}

async function handleSubmit() {
  if (!canSubmit.value || submitting.value) return;
  errorMessage.value = "";
  submitting.value = true;
  try {
    if (mode.value === "password") {
      welcome(await props.store.loginAccountWithPassword(identifier.value.trim(), password.value));
    } else if (mode.value === "reset") {
      await props.store.resetAccountPassword(identifier.value.trim(), code.value.trim(), password.value);
      switchMode("password");
      noticeMessage.value = "密码已重置，请用新密码登录。";
    } else if (step.value === "setup") {
      if (!registrationToken) throw new Error("验证已过期，请重新获取验证码");
      welcome(await props.store.registerAccount(registrationToken, username.value.trim(), password.value));
    } else {
      const outcome = await props.store.loginAccountWithCode(identifier.value.trim(), code.value.trim());
      if (outcome.kind === "registration") {
        registrationToken = outcome.registrationToken;
        password.value = "";
        step.value = "setup";
      } else {
        welcome(outcome);
      }
    }
  } catch (error) {
    const failure = toAccountError(error);
    if (failure.code === "username_taken") availability.value = { kind: "taken", message: failure.message };
    if (failure.code === "registration_expired" || failure.code === "identity_exists") {
      registrationToken = null;
      step.value = "identify";
      code.value = "";
    }
    errorMessage.value = failure.message;
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
      mode.value = "password";
      step.value = "identify";
      errorMessage.value = "";
      noticeMessage.value = "";
      code.value = "";
      password.value = "";
      username.value = "";
      registrationToken = null;
      availability.value = { kind: "idle" };
      await nextTick();
      identifierInput.value?.focus();
    } else {
      clearCountdown();
    }
  }
);

onBeforeUnmount(() => {
  clearCountdown();
  if (checkTimer) clearTimeout(checkTimer);
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
            <p class="subtitle" data-testid="login-subtitle">
              <template v-if="step === 'setup'">验证通过，设置你的账号。用户名是你的唯一账号，设置后不能修改。</template>
              <template v-else>已有账号用密码登录；新用户用邮箱验证码注册，再设置用户名和密码</template>
            </p>
          </div>

          <form class="login-form" @submit.prevent="handleSubmit">
            <div v-if="step !== 'setup'" class="mode-tabs" role="tablist">
              <button type="button" role="tab" data-testid="tab-password" :aria-selected="mode !== 'code'" :class="{ on: mode !== 'code' }" @click="switchMode('password')">密码登录</button>
              <button type="button" role="tab" data-testid="tab-code" :aria-selected="mode === 'code'" :class="{ on: mode === 'code' }" @click="switchMode('code')">验证码登录 / 注册</button>
            </div>

            <p v-if="mode === 'reset' && step !== 'setup'" class="reset-lead" data-testid="reset-lead">重置密码：先验证邮箱或手机号，再设置新密码。</p>

            <div v-if="errorMessage" class="error-banner" role="alert" data-testid="login-error-msg">
              {{ errorMessage }}
            </div>
            <div v-if="noticeMessage" class="notice-banner" role="status" data-testid="login-notice">
              {{ noticeMessage }}
            </div>

            <div v-if="step === 'setup'" class="input-group">
              <label for="login-username">用户名</label>
              <div class="input-wrapper">
                <UserRound :size="18" class="input-icon" />
                <input
                  id="login-username"
                  v-model="username"
                  type="text"
                  data-testid="login-username-input"
                  placeholder="字母开头，4-20 位字母、数字或下划线"
                  maxlength="20"
                  autocomplete="username"
                  autocapitalize="none"
                  spellcheck="false"
                  :disabled="submitting"
                />
              </div>
              <small v-if="hint" class="field-hint" :class="{ bad: availability.kind === 'taken' || availability.kind === 'invalid' }" data-testid="username-hint">{{ hint }}</small>
            </div>
            <div v-else class="input-group">
              <label for="login-identifier">{{ mode === "password" ? "用户名 / 邮箱 / 手机号" : "邮箱或手机号" }}</label>
              <div class="input-wrapper">
                <Mail :size="18" class="input-icon" />
                <input
                  id="login-identifier"
                  ref="identifierInput"
                  v-model="identifier"
                  type="text"
                  data-testid="login-identifier-input"
                  :placeholder="mode === 'password' ? '用户名、邮箱或手机号' : 'name@example.com'"
                  autocomplete="username"
                  autocapitalize="none"
                  spellcheck="false"
                  :disabled="submitting || codeSent"
                />
              </div>
              <button v-if="codeSent" type="button" class="link-button" data-testid="change-identifier" :disabled="submitting" @click="changeIdentifier">
                填错了？换一个邮箱 / 手机号
              </button>
            </div>

            <div v-if="mode !== 'password' && codeSent" class="input-group">
              <label for="login-code">验证码</label>
              <div class="input-wrapper">
                <ShieldCheck :size="18" class="input-icon" />
                <input
                  id="login-code"
                  ref="codeInput"
                  v-model="code"
                  type="text"
                  inputmode="numeric"
                  data-testid="login-code-input"
                  placeholder="请输入 6 位验证码"
                  maxlength="6"
                  autocomplete="one-time-code"
                  :disabled="submitting"
                />
              </div>
            </div>

            <div v-if="mode === 'password' || step === 'setup' || (mode === 'reset' && codeSent)" class="input-group">
              <label for="login-password">{{ mode === "reset" ? "新密码" : step === "setup" ? "设置密码" : "密码" }}</label>
              <div class="code-wrapper">
                <div class="input-wrapper flex-1">
                  <KeyRound :size="18" class="input-icon" />
                  <input
                    id="login-password"
                    v-model="password"
                    :type="showPassword ? 'text' : 'password'"
                    data-testid="login-password-input"
                    placeholder="至少 8 位"
                    :autocomplete="mode === 'password' ? 'current-password' : 'new-password'"
                    :disabled="submitting"
                  />
                </div>
                <button type="button" class="code-send-btn" data-testid="toggle-password" @click="showPassword = !showPassword">
                  {{ showPassword ? "隐藏" : "显示" }}
                </button>
              </div>
              <small v-if="passwordProblem && mode !== 'password'" class="field-hint bad" data-testid="password-hint">{{ passwordProblem }}</small>
            </div>

            <button
              v-if="mode !== 'password' && step !== 'setup'"
              type="button"
              class="code-send-btn wide"
              data-testid="send-code-btn"
              :disabled="!identifier.trim() || countdown > 0 || sending || submitting"
              @click="handleSendCode"
            >
              <Loader2 v-if="sending" :size="14" class="spin" />
              <span v-else-if="countdown > 0">{{ countdown }}s 后重发</span>
              <span v-else>{{ codeSent ? "重新发送" : "获取验证码" }}</span>
            </button>

            <button
              v-if="mode === 'password' || codeSent || step === 'setup'"
              type="submit"
              class="login-submit-btn"
              data-testid="login-submit-btn"
              :disabled="submitting || !canSubmit"
            >
              <Loader2 v-if="submitting" :size="18" class="spin" />
              <span v-else>{{ mode === "password" ? "登录" : mode === "reset" ? "重置密码" : step === "setup" ? "完成注册" : "验证并继续" }}</span>
            </button>

            <button v-if="mode === 'password'" type="button" class="link-button" data-testid="forgot-password" @click="switchMode('reset')">忘记密码？</button>
            <button v-if="mode === 'reset'" type="button" class="link-button" data-testid="back-to-password" @click="switchMode('password')">返回密码登录</button>

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
.mode-tabs {
  display: flex;
  gap: 6px;
  padding: 4px;
  border-radius: 999px;
  background: var(--ui-primary-soft);
}

.mode-tabs button {
  flex: 1;
  height: 34px;
  border: 0;
  border-radius: 999px;
  background: transparent;
  color: var(--ui-text-secondary);
  font: inherit;
  font-size: 13px;
  cursor: pointer;
}

.mode-tabs button.on {
  background: var(--ui-primary);
  color: #fff;
}

.notice-banner {
  background: var(--ui-primary-soft);
  color: var(--ui-text-primary);
  border-radius: 12px;
  padding: 10px 14px;
  font-size: 13px;
  line-height: 1.4;
}

.field-hint {
  margin-top: 6px;
  font-size: 12px;
  color: var(--ui-text-tertiary);
}

.field-hint.bad {
  color: #ef4444;
}

.code-send-btn.wide {
  width: 100%;
  justify-content: center;
}

.reset-lead {
  margin: 0;
  color: var(--ui-text-secondary);
  font-size: 13px;
  line-height: 1.4;
}

.link-button {
  justify-self: start;
  border: 0;
  background: transparent;
  padding: 0;
  color: var(--ui-text-secondary);
  font: inherit;
  font-size: 13px;
  text-decoration: underline;
  cursor: pointer;
}
</style>
