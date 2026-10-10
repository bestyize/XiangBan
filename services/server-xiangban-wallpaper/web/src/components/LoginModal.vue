<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { AccountError } from '@/account/api'
import { toast } from '@/composables/toast'
import { useAccount } from '@/stores/account'
import Icon from './Icon.vue'

const account = useAccount()

const dialog = ref<HTMLDialogElement>()
const emailInput = ref<HTMLInputElement>()
const codeInput = ref<HTMLInputElement>()

const email = ref('')
const code = ref('')
const sending = ref(false)
const submitting = ref(false)
const codeSent = ref(false)
const error = ref('')
const countdown = ref(0)
let timer: ReturnType<typeof setInterval> | undefined

const emailValid = computed(() => /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email.value.trim()))
const codeValid = computed(() => /^\d{6}$/.test(code.value.trim()))
const sendLabel = computed(() => {
  if (sending.value) return '发送中…'
  if (countdown.value > 0) return `${countdown.value} 秒后重发`
  return codeSent.value ? '重新发送' : '获取验证码'
})

function startCountdown(seconds: number) {
  clearInterval(timer)
  countdown.value = seconds
  timer = setInterval(() => {
    countdown.value -= 1
    if (countdown.value <= 0) clearInterval(timer)
  }, 1000)
}

function describe(e: unknown) {
  return e instanceof Error ? e.message : '操作失败，请稍后再试'
}

async function sendCode() {
  if (!emailValid.value) {
    error.value = '请输入有效的邮箱地址'
    emailInput.value?.focus()
    return
  }
  if (sending.value || countdown.value > 0) return
  error.value = ''
  sending.value = true
  try {
    const res = await account.sendCode(email.value.trim())
    codeSent.value = true
    startCountdown(res.cooldown_seconds || 60)
    toast('验证码已发送，请查收邮件')
    await nextTick()
    codeInput.value?.focus()
  } catch (e) {
    if (e instanceof AccountError && e.retryAfter) startCountdown(e.retryAfter)
    error.value = describe(e)
  } finally {
    sending.value = false
  }
}

async function submit() {
  if (!emailValid.value) {
    error.value = '请输入有效的邮箱地址'
    emailInput.value?.focus()
    return
  }
  if (!codeValid.value) {
    error.value = '请输入邮件里的 6 位验证码'
    codeInput.value?.focus()
    return
  }
  error.value = ''
  submitting.value = true
  try {
    const { isNewUser, user } = await account.loginWithCode(email.value.trim(), code.value.trim())
    account.closeLogin()
    toast(isNewUser ? `注册成功，欢迎你，${user.nickname}` : `欢迎回来，${user.nickname}`)
  } catch (e) {
    error.value = describe(e)
    codeInput.value?.select()
  } finally {
    submitting.value = false
  }
}

function reset() {
  code.value = ''
  error.value = ''
  codeSent.value = false
  clearInterval(timer)
  countdown.value = 0
}

watch(
  () => account.loginOpen,
  async open => {
    const el = dialog.value
    if (!el) return
    if (open && !el.open) {
      reset()
      el.showModal()
      await nextTick()
      emailInput.value?.focus()
    } else if (!open && el.open) {
      el.close()
    }
  },
  { flush: 'post' },
)

/** 原生 dialog 被 Esc 或点击遮罩关闭时同步状态。 */
function onClose() {
  if (account.loginOpen) account.closeLogin()
}

function onBackdrop(event: MouseEvent) {
  if (event.target === dialog.value && !submitting.value) account.closeLogin()
}

onBeforeUnmount(() => clearInterval(timer))
</script>

<template>
  <dialog
    ref="dialog"
    class="login"
    aria-labelledby="login-title"
    aria-describedby="login-desc"
    @close="onClose"
    @click="onBackdrop"
  >
    <form class="panel" novalidate @submit.prevent="submit">
      <header class="head">
        <h2 id="login-title" class="display">登录 / 注册</h2>
        <button type="button" class="close" aria-label="关闭" :disabled="submitting" @click="account.closeLogin()">
          <Icon name="close" :size="20" />
        </button>
      </header>
      <p id="login-desc" class="desc">输入邮箱获取验证码。新邮箱验证后会自动注册相伴账号，小伴桌面与手机端通用。</p>

      <label class="field">
        <span>邮箱</span>
        <input
          ref="emailInput"
          v-model="email"
          class="input"
          type="email"
          inputmode="email"
          autocomplete="email"
          placeholder="name@example.com"
          :disabled="submitting"
          required
        >
      </label>

      <label class="field">
        <span>验证码</span>
        <div class="code-row">
          <input
            ref="codeInput"
            v-model="code"
            class="input"
            type="text"
            inputmode="numeric"
            autocomplete="one-time-code"
            maxlength="6"
            placeholder="6 位数字"
            :disabled="submitting"
            required
          >
          <button
            type="button"
            class="btn send"
            :disabled="sending || countdown > 0 || submitting"
            @click="sendCode"
          >
            {{ sendLabel }}
          </button>
        </div>
      </label>

      <p v-if="error" class="error" role="alert">{{ error }}</p>

      <button type="submit" class="btn btn-primary submit" :disabled="submitting">
        {{ submitting ? '登录中…' : '登录 / 注册' }}
      </button>
      <p class="hint">没收到邮件时，请检查垃圾邮件箱。</p>
    </form>
  </dialog>
</template>

<style scoped>
.login {
  width: min(420px, calc(100vw - 32px));
  max-height: calc(100dvh - 32px);
  margin: auto;
  padding: 0;
  border: 1px solid var(--line-strong);
  border-radius: 20px;
  background: var(--surface);
  color: var(--text);
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.35);
}

.login::backdrop {
  background: rgba(0, 0, 0, 0.55);
  -webkit-backdrop-filter: blur(4px);
  backdrop-filter: blur(4px);
}

.panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 24px;
}

.head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.head h2 {
  font-size: 30px;
  line-height: 1.1;
}

.close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 50%;
  color: var(--text-2);
}

.close:hover {
  background: var(--surface-2);
  color: var(--text);
}

.desc {
  margin-top: -6px;
  color: var(--text-2);
  font-size: 14px;
  line-height: 1.6;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.field > span {
  font-size: 13px;
  color: var(--text-2);
}

.input {
  width: 100%;
  min-width: 0;
  height: 44px;
  padding: 0 14px;
  border-radius: var(--r-ui);
  border: 1px solid var(--line-strong);
  background: var(--bg);
  color: var(--text);
  font-size: 15px;
}

.input:focus {
  outline: none;
  border-color: var(--text-2);
}

.code-row {
  display: flex;
  gap: 8px;
}

.send {
  flex-shrink: 0;
  min-width: 112px;
  padding: 0 14px;
  font-size: 14px;
}

.error {
  padding: 10px 12px;
  border-radius: var(--r-ui);
  background: color-mix(in srgb, var(--heart) 14%, transparent);
  color: var(--heart);
  font-size: 13px;
}

.submit {
  width: 100%;
  margin-top: 4px;
}

.hint {
  margin-top: -6px;
  text-align: center;
  font-size: 12px;
  color: var(--text-3);
}
</style>
