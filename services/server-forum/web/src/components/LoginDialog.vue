<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import ModalDialog from './ModalDialog.vue'
import { accountApi, AccountError } from '@/api/account'
import { useSession } from '@/stores/session'
import { useToast } from '@/stores/toast'

type Mode = 'password' | 'code' | 'reset'
type Step = 'identify' | 'verify' | 'setup'
type Availability = { kind: 'idle' } | { kind: 'checking' } | { kind: 'ok' } | { kind: 'taken' | 'invalid'; message: string }

const session = useSession()
const toast = useToast()

const mode = ref<Mode>('password')
const step = ref<Step>('identify')
const identifier = ref('')
const password = ref('')
const code = ref('')
const username = ref('')
const showPassword = ref(false)
const busy = ref(false)
const error = ref('')
const notice = ref('')
const cooldown = ref(0)
const availability = ref<Availability>({ kind: 'idle' })
const methods = ref({ email: true, phone: false })
const emailOnly = computed(() => mode.value === 'code' && !methods.value.phone)
let registrationToken: string | null = null
let timer: ReturnType<typeof setInterval> | null = null
let checkTimer: ReturnType<typeof setTimeout> | null = null

const codeSent = computed(() => step.value === 'verify')
const codeValid = computed(() => /^\d{6}$/.test(code.value.trim()))
const validId = computed(() => {
  const v = identifier.value.trim()
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(v) || (!emailOnly.value && /^\+?\d{6,15}$/.test(v))
})

function usernameProblem(raw: string): string {
  const v = raw.trim()
  if (!v) return ''
  if (!/^[A-Za-z一-鿿]/.test(v)) return '用户名需以字母或汉字开头'
  if (!/^[A-Za-z0-9_一-鿿]+$/.test(v)) return '只能包含字母、数字、下划线和汉字'
  const han = (v.match(/[一-鿿]/g) ?? []).length
  const letters = (v.match(/[A-Za-z]/g) ?? []).length
  if (han > 0 ? han + letters < 2 : letters < 3) return '至少要有 2 个汉字或 3 个英文字母'
  const width = [...v].reduce((n, c) => n + (/[一-鿿]/.test(c) ? 2 : 1), 0)
  if (width > 20) return '最长 20 个字符，汉字算 2 个'
  return ''
}

function passwordProblemOf(raw: string, name = ''): string {
  if (!raw) return ''
  if (raw.length < 8) return '密码至少 8 位'
  if (raw.length > 128) return '密码最多 128 位'
  if (name && raw.toLowerCase() === name.trim().toLowerCase()) return '密码不能和用户名相同'
  return ''
}

const passwordProblem = computed(() => passwordProblemOf(password.value, mode.value === 'reset' ? '' : username.value))
const hint = computed(() => {
  const a = availability.value
  if (a.kind === 'checking') return '正在检查…'
  if (a.kind === 'ok') return '可以使用'
  return a.kind === 'taken' || a.kind === 'invalid' ? a.message : ''
})
const canSubmit = computed(() => {
  if (mode.value === 'password') return !!identifier.value.trim() && !!password.value
  if (mode.value === 'reset') return codeSent.value && codeValid.value && !!password.value && !passwordProblem.value
  if (step.value === 'setup') {
    const k = availability.value.kind
    return !!username.value.trim() && k === 'ok' && !!password.value && !passwordProblem.value
  }
  return codeSent.value && codeValid.value
})

function startCooldown(secs: number) {
  cooldown.value = secs
  if (timer) clearInterval(timer)
  timer = setInterval(() => {
    cooldown.value -= 1
    if (cooldown.value <= 0 && timer) {
      clearInterval(timer)
      timer = null
    }
  }, 1000)
}

function switchMode(next: Mode) {
  mode.value = next
  step.value = 'identify'
  code.value = ''
  password.value = ''
  error.value = ''
  notice.value = ''
  registrationToken = null
  if (timer) clearInterval(timer)
  timer = null
  cooldown.value = 0
}

function changeIdentifier() {
  step.value = 'identify'
  code.value = ''
  password.value = ''
  error.value = ''
  notice.value = ''
  if (timer) clearInterval(timer)
  timer = null
  cooldown.value = 0
}

watch(
  () => session.loginOpen,
  open => {
    if (!open) return
    void accountApi.signupMethods().then(value => { methods.value = value })
    switchMode('password')
    username.value = ''
    availability.value = { kind: 'idle' }
  },
)

// 设置账号时边输入边查重，只是提示；真正的唯一性由服务端保证。
watch([username, step], () => {
  if (checkTimer) clearTimeout(checkTimer)
  if (step.value !== 'setup') return
  const name = username.value.trim()
  if (!name) {
    availability.value = { kind: 'idle' }
    return
  }
  const local = usernameProblem(name)
  if (local) {
    availability.value = { kind: 'invalid', message: local }
    return
  }
  availability.value = { kind: 'checking' }
  checkTimer = setTimeout(async () => {
    try {
      const res = await accountApi.checkUsername(name)
      if (username.value.trim() === name) availability.value = res.available ? { kind: 'ok' } : { kind: 'taken', message: res.message ?? '这个用户名已被占用' }
    } catch {
      if (username.value.trim() === name) availability.value = { kind: 'idle' }
    }
  }, 350)
})

onBeforeUnmount(() => {
  if (timer) clearInterval(timer)
  if (checkTimer) clearTimeout(checkTimer)
})

async function sendCode() {
  if (!validId.value || busy.value || cooldown.value > 0) return
  busy.value = true
  error.value = ''
  try {
    const res = await accountApi.sendCode(identifier.value.trim(), mode.value === 'reset' ? 'reset_password' : 'login')
    step.value = 'verify'
    startCooldown(res.cooldown_seconds || 60)
  } catch (e) {
    error.value = e instanceof Error ? e.message : '验证码发送失败'
    if (e instanceof AccountError && e.retryAfter) startCooldown(e.retryAfter)
  } finally {
    busy.value = false
  }
}

function welcome(result: { isNewUser: boolean; username: string }) {
  session.loginOpen = false
  toast.show(result.isNewUser ? `欢迎来到超级小伴，${result.username}` : `已登录，${result.username}`)
}

async function submit() {
  if (!canSubmit.value || busy.value) return
  busy.value = true
  error.value = ''
  try {
    if (mode.value === 'password') {
      welcome(await session.loginWithPassword(identifier.value.trim(), password.value))
    } else if (mode.value === 'reset') {
      await accountApi.resetPassword(identifier.value.trim(), code.value.trim(), password.value)
      switchMode('password')
      notice.value = '密码已重置，请用新密码登录。'
    } else if (step.value === 'setup') {
      if (!registrationToken) throw new AccountError(401, '验证已过期，请重新获取验证码', undefined, 'registration_expired')
      welcome(await session.register(registrationToken, username.value.trim(), password.value))
    } else {
      const res = await session.loginWithCode(identifier.value.trim(), code.value.trim())
      if ('registrationToken' in res) {
        registrationToken = res.registrationToken
        password.value = ''
        step.value = 'setup'
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
    error.value = e instanceof Error ? e.message : '登录失败'
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <ModalDialog :open="session.loginOpen" title="登录超级小伴" :width="400" @close="session.loginOpen = false">
    <p class="lead">
      <template v-if="step === 'setup'">验证通过，设置你的账号。用户名是你的唯一账号，设置后不能修改。</template>
      <template v-else>已有相伴账号用密码登录；新用户用邮箱验证码注册，再设置用户名和密码。</template>
    </p>

    <p v-if="mode === 'reset' && step !== 'setup'" class="lead">重置密码：先验证邮箱或手机号，再设置新密码。</p>

    <div v-if="step !== 'setup'" class="tabs" role="tablist">
      <button type="button" role="tab" :aria-selected="mode !== 'code'" :class="{ on: mode !== 'code' }" @click="switchMode('password')">密码登录</button>
      <button type="button" role="tab" :aria-selected="mode === 'code'" :class="{ on: mode === 'code' }" @click="switchMode('code')">验证码登录 / 注册</button>
    </div>

    <form class="form" @submit.prevent="submit">
      <label v-if="step === 'setup'" class="field">
        <span>用户名</span>
        <input v-model="username" class="input" type="text" autocomplete="username" autocapitalize="none" spellcheck="false" maxlength="20" placeholder="中英文、数字、下划线，至少 2 个汉字或 3 个字母" />
        <small v-if="hint" class="hint" :class="{ bad: availability.kind === 'taken' || availability.kind === 'invalid' }" role="status">{{ hint }}</small>
      </label>
      <label v-else class="field">
        <span>{{ mode === 'password' ? '用户名 / 邮箱 / 手机号' : emailOnly ? '邮箱' : '邮箱或手机号' }}</span>
        <input
          v-model="identifier"
          class="input"
          type="text"
          autocomplete="username"
          autocapitalize="none"
          spellcheck="false"
          :disabled="codeSent"
          :placeholder="mode === 'password' ? '用户名、邮箱或手机号' : 'name@example.com'"
        />
        <small v-if="mode !== 'password' && !codeSent && identifier.trim() && !validId" class="hint">{{ emailOnly ? '暂未开放手机号注册，请填写完整的邮箱。' : '验证码只能发到完整的邮箱或手机号。' }}</small>
        <button v-if="codeSent" type="button" class="link start" :disabled="busy" @click="changeIdentifier">填错了？换一个{{ emailOnly ? '邮箱' : '邮箱 / 手机号' }}</button>
      </label>

      <label v-if="mode !== 'password' && codeSent" class="field">
        <span>验证码</span>
        <input v-model="code" class="input code" type="text" inputmode="numeric" autocomplete="one-time-code" maxlength="6" placeholder="6 位数字" />
      </label>

      <label v-if="mode === 'password' || step === 'setup' || (mode === 'reset' && codeSent)" class="field">
        <span>{{ mode === 'reset' ? '新密码' : step === 'setup' ? '设置密码' : '密码' }}</span>
        <span class="with-button">
          <input
            v-model="password"
            class="input"
            :type="showPassword ? 'text' : 'password'"
            :autocomplete="mode === 'password' ? 'current-password' : 'new-password'"
            placeholder="至少 8 位"
          />
          <button type="button" class="link" @click="showPassword = !showPassword">{{ showPassword ? '隐藏' : '显示' }}</button>
        </span>
        <small v-if="mode !== 'password' && passwordProblem" class="hint bad">{{ passwordProblem }}</small>
      </label>

      <p v-if="notice" class="sent" role="status">{{ notice }}</p>
      <p v-if="error" class="error-text" role="alert">{{ error }}</p>

      <button v-if="mode !== 'password' && step !== 'setup'" class="btn wide" type="button" :disabled="!validId || busy || cooldown > 0" @click="sendCode">
        {{ cooldown > 0 ? `${cooldown} 秒后可重新发送` : busy ? '正在发送' : codeSent ? '重新发送' : '发送验证码' }}
      </button>

      <button v-if="mode === 'password' || codeSent || step === 'setup'" class="btn btn-primary wide" type="submit" :disabled="!canSubmit || busy">
        {{ busy ? '请稍候' : mode === 'password' ? '登录' : mode === 'reset' ? '重置密码' : step === 'setup' ? '完成注册' : '验证并继续' }}
      </button>

      <button v-if="mode === 'password'" type="button" class="link" @click="switchMode('reset')">忘记密码？</button>
      <button v-if="mode === 'reset'" type="button" class="link" @click="switchMode('password')">返回密码登录</button>
    </form>

    <p class="legal">
      登录即表示同意<RouterLink to="/terms" @click="session.loginOpen = false">用户协议</RouterLink>和<RouterLink
        to="/privacy"
        @click="session.loginOpen = false"
        >隐私政策</RouterLink
      >。
    </p>
  </ModalDialog>
</template>

<style scoped>
.lead {
  margin: -6px 0 16px;
  color: var(--ink-2);
  font-size: var(--fs-sm);
}
.form {
  display: grid;
  gap: 14px;
}
.wide {
  width: 100%;
  height: 42px;
  font-size: var(--fs-md);
}
.wide.btn-primary {
  background: var(--brand);
  border-color: var(--brand);
  color: var(--brand-ink);
}
.wide.btn-primary:hover:not(:disabled) {
  background: color-mix(in srgb, var(--brand) 88%, #000);
  border-color: color-mix(in srgb, var(--brand) 88%, #000);
}
.code {
  font-family: var(--font-display);
  font-size: 22px;
  letter-spacing: 0.3em;
}
.sent {
  margin: 0;
  font-size: var(--fs-sm);
  color: var(--ink-2);
}
.row {
  display: flex;
  justify-content: space-between;
}
.link {
  border: 0;
  background: none;
  padding: 0;
  color: var(--ink-2);
  font-size: var(--fs-sm);
  text-decoration: underline;
  text-underline-offset: 3px;
}
.link.start {
  align-self: flex-start;
  justify-self: start;
}
.link:disabled {
  color: var(--ink-3);
  text-decoration: none;
  cursor: default;
}
.legal {
  margin: 18px 0 0;
  font-size: var(--fs-xs);
  color: var(--ink-3);
}
.legal a {
  color: var(--ink-2);
  text-decoration: underline;
  text-underline-offset: 2px;
}
.tabs {
  display: flex;
  gap: 6px;
  margin: 0 0 14px;
  padding: 4px;
  border-radius: 999px;
  background: var(--surface-2);
}
.tabs button {
  flex: 1;
  height: 34px;
  border: 0;
  border-radius: 999px;
  background: none;
  color: var(--ink-2);
  font-size: var(--fs-sm);
}
.tabs button.on {
  background: var(--brand);
  color: var(--brand-ink);
  font-weight: 600;
}
.with-button {
  display: flex;
  align-items: center;
  gap: 10px;
}
.with-button .input {
  flex: 1;
  min-width: 0;
}
.hint {
  color: var(--ink-2);
  font-size: var(--fs-xs);
}
.hint.bad {
  color: var(--danger);
}
</style>
