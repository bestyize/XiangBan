<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { AccountError } from '@/account/api'
import { toast } from '@/composables/toast'
import { useAccount } from '@/stores/account'
import Icon from './Icon.vue'

type Mode = 'password' | 'code' | 'reset'
type Step = 'identify' | 'verify' | 'setup'
type Availability = { kind: 'idle' | 'checking' | 'ok' } | { kind: 'taken' | 'invalid'; message: string }

const account = useAccount()

const dialog = ref<HTMLDialogElement>()
const identifierInput = ref<HTMLInputElement>()

const mode = ref<Mode>('password')
const step = ref<Step>('identify')
const identifier = ref('')
const password = ref('')
const code = ref('')
const username = ref('')
const showPassword = ref(false)
const sending = ref(false)
const submitting = ref(false)
const error = ref('')
const notice = ref('')
const countdown = ref(0)
const availability = ref<Availability>({ kind: 'idle' })
let registrationToken: string | null = null
let timer: ReturnType<typeof setInterval> | undefined
let checkTimer: ReturnType<typeof setTimeout> | undefined

const codeSent = computed(() => step.value === 'verify')
const codeValid = computed(() => /^\d{6}$/.test(code.value.trim()))
const sendLabel = computed(() => {
  if (sending.value) return '发送中…'
  if (countdown.value > 0) return `${countdown.value} 秒后重发`
  return codeSent.value ? '重新发送' : '获取验证码'
})

function usernameError(raw: string): string {
  const value = raw.trim()
  if (!value) return ''
  if (!/^[A-Za-z]/.test(value)) return '用户名需以字母开头'
  if (!/^[A-Za-z0-9_]+$/.test(value)) return '只能包含字母、数字和下划线'
  if (value.length < 4 || value.length > 20) return '需为 4-20 个字符'
  return ''
}

function passwordError(raw: string, name = ''): string {
  if (!raw) return ''
  if (raw.length < 8) return '密码至少 8 位'
  if (raw.length > 128) return '密码最多 128 位'
  if (name && raw.toLowerCase() === name.trim().toLowerCase()) return '密码不能和用户名相同'
  return ''
}

const passwordProblem = computed(() => passwordError(password.value, mode.value === 'reset' ? '' : username.value))
const canSubmit = computed(() => {
  if (mode.value === 'password') return !!identifier.value.trim() && !!password.value
  if (mode.value === 'reset') return codeSent.value && codeValid.value && !!password.value && !passwordProblem.value
  if (step.value === 'setup') {
    const k = availability.value.kind
    return !!username.value.trim() && k !== 'taken' && k !== 'invalid' && k !== 'checking' && !!password.value && !passwordProblem.value
  }
  return codeSent.value && codeValid.value
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

function switchMode(next: Mode) {
  mode.value = next
  step.value = 'identify'
  code.value = ''
  password.value = ''
  error.value = ''
  notice.value = ''
  registrationToken = null
  clearInterval(timer)
  countdown.value = 0
}

// 设置账号时边输入边查重，只是提示；真正的唯一性由服务端保证。
watch([username, step], () => {
  clearTimeout(checkTimer)
  if (step.value !== 'setup') return
  const name = username.value.trim()
  if (!name) { availability.value = { kind: 'idle' }; return }
  const local = usernameError(name)
  if (local) { availability.value = { kind: 'invalid', message: local }; return }
  availability.value = { kind: 'checking' }
  checkTimer = setTimeout(async () => {
    try {
      const res = await account.checkUsername(name)
      if (username.value.trim() === name) availability.value = res.available ? { kind: 'ok' } : { kind: 'taken', message: res.message ?? '这个用户名已被占用' }
    } catch {
      if (username.value.trim() === name) availability.value = { kind: 'idle' }
    }
  }, 350)
})

const hintText = computed(() => {
  const a = availability.value
  if (a.kind === 'checking') return '正在检查…'
  if (a.kind === 'ok') return '可以使用'
  return a.kind === 'taken' || a.kind === 'invalid' ? a.message : ''
})

async function sendCode() {
  if (!identifier.value.trim()) {
    error.value = '请输入邮箱或手机号'
    identifierInput.value?.focus()
    return
  }
  if (sending.value || countdown.value > 0) return
  error.value = ''
  sending.value = true
  try {
    const res = await account.sendCode(identifier.value.trim(), mode.value === 'reset' ? 'reset_password' : 'login')
    step.value = 'verify'
    startCountdown(res.cooldown_seconds || 60)
    toast('验证码已发送，请查收')
  } catch (e) {
    if (e instanceof AccountError && e.retryAfter) startCountdown(e.retryAfter)
    error.value = describe(e)
  } finally {
    sending.value = false
  }
}

function welcome(result: { isNewUser: boolean; user: { username: string } }) {
  account.closeLogin()
  toast(result.isNewUser ? `注册成功，欢迎你，${result.user.username}` : `欢迎回来，${result.user.username}`)
}

async function submit() {
  if (!canSubmit.value || submitting.value) return
  error.value = ''
  submitting.value = true
  try {
    if (mode.value === 'password') {
      welcome(await account.loginWithPassword(identifier.value.trim(), password.value))
    } else if (mode.value === 'reset') {
      await account.resetPassword(identifier.value.trim(), code.value.trim(), password.value)
      switchMode('password')
      notice.value = '密码已重置，请用新密码登录。'
    } else if (step.value === 'setup') {
      if (!registrationToken) throw new AccountError(401, '验证已过期，请重新获取验证码', undefined, 'registration_expired')
      welcome(await account.register(registrationToken, username.value.trim(), password.value))
    } else {
      const res = await account.loginWithCode(identifier.value.trim(), code.value.trim())
      if ('registrationToken' in res) {
        registrationToken = res.registrationToken
        step.value = 'setup'
        password.value = ''
      } else {
        welcome(res)
      }
    }
  } catch (e) {
    if (e instanceof AccountError && e.code === 'username_taken') availability.value = { kind: 'taken', message: e.message }
    if (e instanceof AccountError && (e.code === 'registration_expired' || e.code === 'identity_exists')) {
      registrationToken = null
      step.value = 'identify'
      code.value = ''
    }
    error.value = describe(e)
  } finally {
    submitting.value = false
  }
}

function reset() {
  mode.value = 'password'
  step.value = 'identify'
  code.value = ''
  password.value = ''
  username.value = ''
  error.value = ''
  notice.value = ''
  registrationToken = null
  availability.value = { kind: 'idle' }
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
      identifierInput.value?.focus()
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

onBeforeUnmount(() => {
  clearInterval(timer)
  clearTimeout(checkTimer)
})
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
      <p id="login-desc" class="desc">
        <template v-if="step === 'setup'">验证通过，设置你的账号。用户名是你的唯一账号，设置后不能修改。</template>
        <template v-else>已有账号用密码登录；新用户用邮箱验证码注册，再设置用户名和密码。小伴桌面与手机端通用。</template>
      </p>

      <div v-if="step !== 'setup'" class="tabs" role="tablist">
        <button type="button" role="tab" :aria-selected="mode !== 'code'" :class="{ on: mode !== 'code' }" @click="switchMode('password')">密码登录</button>
        <button type="button" role="tab" :aria-selected="mode === 'code'" :class="{ on: mode === 'code' }" @click="switchMode('code')">验证码登录 / 注册</button>
      </div>

      <label v-if="step === 'setup'" class="field">
        <span>用户名</span>
        <input
          v-model="username"
          class="input"
          type="text"
          autocomplete="username"
          autocapitalize="none"
          spellcheck="false"
          maxlength="20"
          placeholder="字母开头，4-20 位字母、数字或下划线"
          :disabled="submitting"
        >
        <small v-if="hintText" class="field-hint" :class="{ bad: availability.kind === 'taken' || availability.kind === 'invalid' }" role="status">{{ hintText }}</small>
      </label>
      <label v-else class="field">
        <span>{{ mode === 'password' ? '用户名 / 邮箱 / 手机号' : '邮箱或手机号' }}</span>
        <input
          ref="identifierInput"
          v-model="identifier"
          class="input"
          type="text"
          autocomplete="username"
          autocapitalize="none"
          spellcheck="false"
          :placeholder="mode === 'password' ? '用户名、邮箱或手机号' : 'name@example.com'"
          :disabled="submitting || codeSent"
        >
      </label>

      <label v-if="mode !== 'code' && (mode === 'password' || codeSent)" class="field">
        <span>{{ mode === 'reset' ? '新密码' : '密码' }}</span>
        <div class="code-row">
          <input
            v-model="password"
            class="input"
            :type="showPassword ? 'text' : 'password'"
            :autocomplete="mode === 'reset' ? 'new-password' : 'current-password'"
            placeholder="至少 8 位"
            :disabled="submitting"
          >
          <button type="button" class="btn send" @click="showPassword = !showPassword">{{ showPassword ? '隐藏' : '显示' }}</button>
        </div>
        <small v-if="mode === 'reset' && passwordProblem" class="field-hint bad">{{ passwordProblem }}</small>
      </label>

      <label v-if="step === 'setup'" class="field">
        <span>设置密码</span>
        <div class="code-row">
          <input
            v-model="password"
            class="input"
            :type="showPassword ? 'text' : 'password'"
            autocomplete="new-password"
            placeholder="至少 8 位"
            :disabled="submitting"
          >
          <button type="button" class="btn send" @click="showPassword = !showPassword">{{ showPassword ? '隐藏' : '显示' }}</button>
        </div>
        <small v-if="passwordProblem" class="field-hint bad">{{ passwordProblem }}</small>
      </label>

      <label v-if="mode !== 'password' && codeSent" class="field">
        <span>验证码</span>
        <input
          v-model="code"
          class="input"
          type="text"
          inputmode="numeric"
          autocomplete="one-time-code"
          maxlength="6"
          placeholder="6 位数字"
          :disabled="submitting"
        >
      </label>

      <p v-if="notice" class="notice" role="status">{{ notice }}</p>
      <p v-if="error" class="error" role="alert">{{ error }}</p>

      <div v-if="mode !== 'password' && step !== 'setup'" class="actions">
        <button type="button" class="btn" :disabled="sending || countdown > 0 || submitting || !identifier.trim()" @click="sendCode">
          {{ sendLabel }}
        </button>
      </div>

      <button v-if="mode === 'password' || codeSent || step === 'setup'" type="submit" class="btn btn-primary submit" :disabled="submitting || !canSubmit">
        {{ submitting ? '请稍候…' : mode === 'password' ? '登录' : mode === 'reset' ? '重置密码' : step === 'setup' ? '完成注册' : '验证并继续' }}
      </button>

      <button v-if="mode === 'password'" type="button" class="linkish" @click="switchMode('reset')">忘记密码？</button>
      <button v-if="mode === 'reset'" type="button" class="linkish" @click="switchMode('password')">返回密码登录</button>
      <p v-if="mode !== 'password' && step !== 'setup'" class="hint">没收到时，请检查垃圾邮件箱。</p>
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
.tabs {
  display: flex;
  gap: 6px;
  padding: 4px;
  border-radius: 999px;
  background: var(--surface-2);
}

.tabs button {
  flex: 1;
  height: 36px;
  border-radius: 999px;
  color: var(--text-2);
  font-size: 14px;
}

.tabs button.on {
  background: var(--text);
  color: var(--bg);
}

.field-hint {
  font-size: 12px;
  color: var(--text-2);
}

.field-hint.bad {
  color: var(--heart);
}

.notice {
  color: var(--text-2);
  font-size: 13px;
}

.actions {
  display: flex;
}

.linkish {
  align-self: flex-start;
  font-size: 13px;
  color: var(--text-2);
  text-decoration: underline;
}
</style>
