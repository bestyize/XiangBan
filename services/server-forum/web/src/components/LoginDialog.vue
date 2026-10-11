<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import ModalDialog from './ModalDialog.vue'
import { accountApi, AccountError } from '@/api/account'
import { useSession } from '@/stores/session'
import { useToast } from '@/stores/toast'

const session = useSession()
const toast = useToast()

const identifier = ref('')
const code = ref('')
const step = ref<'id' | 'code'>('id')
const busy = ref(false)
const error = ref('')
const cooldown = ref(0)
let timer: ReturnType<typeof setInterval> | null = null

const validId = computed(() => {
  const v = identifier.value.trim()
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(v) || /^\+?\d{6,15}$/.test(v)
})

watch(
  () => session.loginOpen,
  open => {
    if (open) {
      error.value = ''
      code.value = ''
      step.value = 'id'
    }
  },
)

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
onBeforeUnmount(() => timer && clearInterval(timer))

async function sendCode() {
  if (!validId.value || busy.value || cooldown.value > 0) return
  busy.value = true
  error.value = ''
  try {
    const res = await accountApi.sendCode(identifier.value.trim())
    step.value = 'code'
    startCooldown(res.cooldown_seconds || 60)
  } catch (e) {
    error.value = e instanceof Error ? e.message : '验证码发送失败'
    if (e instanceof AccountError && e.retryAfter) startCooldown(e.retryAfter)
  } finally {
    busy.value = false
  }
}

async function login() {
  if (code.value.trim().length < 4 || busy.value) return
  busy.value = true
  error.value = ''
  try {
    const isNew = await session.loginWithCode(identifier.value.trim(), code.value.trim())
    session.loginOpen = false
    toast.show(isNew ? '欢迎来到超级小伴' : '已登录')
  } catch (e) {
    error.value = e instanceof Error ? e.message : '登录失败'
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <ModalDialog :open="session.loginOpen" title="登录超级小伴" :width="400" @close="session.loginOpen = false">
    <p class="lead">用相伴账号登录。没有账号的话，验证后会自动注册。</p>

    <form v-if="step === 'id'" class="form" @submit.prevent="sendCode">
      <label class="field">
        <span>邮箱或手机号</span>
        <input
          v-model="identifier"
          class="input"
          type="text"
          inputmode="email"
          autocomplete="username"
          placeholder="name@example.com"
          autofocus
        />
      </label>
      <p v-if="error" class="error-text" role="alert">{{ error }}</p>
      <button class="btn btn-primary wide" type="submit" :disabled="!validId || busy || cooldown > 0">
        {{ cooldown > 0 ? `${cooldown} 秒后可重新发送` : busy ? '正在发送' : '发送验证码' }}
      </button>
    </form>

    <form v-else class="form" @submit.prevent="login">
      <p class="sent">验证码已发到 <strong>{{ identifier }}</strong></p>
      <label class="field">
        <span>验证码</span>
        <input
          v-model="code"
          class="input code"
          type="text"
          inputmode="numeric"
          autocomplete="one-time-code"
          maxlength="8"
          placeholder="6 位数字"
          autofocus
        />
      </label>
      <p v-if="error" class="error-text" role="alert">{{ error }}</p>
      <button class="btn btn-primary wide" type="submit" :disabled="code.trim().length < 4 || busy">
        {{ busy ? '正在登录' : '登录' }}
      </button>
      <div class="row">
        <button type="button" class="link" @click="step = 'id'">换个账号</button>
        <button type="button" class="link" :disabled="cooldown > 0 || busy" @click="sendCode">
          {{ cooldown > 0 ? `${cooldown} 秒后重发` : '重新发送' }}
        </button>
      </div>
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
</style>
