<script setup lang="ts">
import '@/styles/admin.css'
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import Icon from '@/components/Icon.vue'
import { adminTheme, applyAdminRootAttrs, toggleAdminTheme } from '@/admin/prefs'
import { useAdmin } from '@/stores/admin'

const route = useRoute()
const router = useRouter()
const admin = useAdmin()

const username = ref('')
const password = ref('')
const showPw = ref(false)
const busy = ref(false)
const error = ref<string | null>(null)

const prevTitle = document.title
onMounted(() => {
  applyAdminRootAttrs(true)
  document.title = '登录 · 小伴控制台'
})
onBeforeUnmount(() => {
  applyAdminRootAttrs(false)
  document.title = prevTitle
})

async function submit() {
  if (busy.value) return
  busy.value = true
  error.value = null
  try {
    await admin.login(username.value.trim(), password.value)
    const target = route.query.redirect
    router.replace(typeof target === 'string' && target.startsWith('/admin/') ? target : '/admin/overview')
  } catch (e) {
    error.value = e instanceof Error ? e.message : '登录失败，请稍后再试'
    password.value = ''
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <main class="xb-admin login-stage">
    <div class="login-card">
      <div class="brand">
        <span class="mark">
          <img src="/app-icon.png" alt="小伴" @error="($event.target as HTMLImageElement).style.display = 'none'" />
        </span>
        <div>
          <h1>小伴控制台</h1>
          <div class="host">wallpaper.xiangban.xyz</div>
        </div>
      </div>

      <div v-if="admin.unavailable" class="xb-alert" role="alert">
        <div class="xb-alert-l">
          <Icon name="alert" :size="15" />
          <span>{{ admin.unavailable }}</span>
        </div>
      </div>

      <form v-else class="form" @submit.prevent="submit">
        <label class="xb-field">
          <span class="xb-field-label">用户名</span>
          <input
            v-model="username"
            class="xb-input"
            name="username"
            autocomplete="username"
            autocapitalize="off"
            placeholder="admin"
            required
            autofocus
          />
        </label>

        <label class="xb-field">
          <span class="xb-field-label">密码</span>
          <div class="xb-affix has-r">
            <input
              v-model="password"
              class="xb-input"
              name="password"
              :type="showPw ? 'text' : 'password'"
              autocomplete="current-password"
              placeholder="输入管理员密码"
              required
            />
            <button type="button" class="af-r" aria-label="显示或隐藏密码" @click="showPw = !showPw">
              <Icon :name="showPw ? 'eye-off' : 'eye'" :size="14" />
            </button>
          </div>
        </label>

        <p v-if="error" class="xb-error" role="alert">{{ error }}</p>

        <button class="xb-btn-primary is-lg" type="submit" :disabled="busy" style="margin-top: 4px">
          {{ busy ? '登录中…' : '登录' }}
        </button>
      </form>

      <div class="foot">
        <RouterLink to="/">← 返回官网</RouterLink>
        <button type="button" @click="toggleAdminTheme">
          {{ adminTheme === 'light' ? '深色' : '浅色' }}
        </button>
      </div>
    </div>
  </main>
</template>

<style scoped>
.login-stage {
  min-height: 100vh;
  min-height: 100dvh;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  background: var(--ui-tint);
}
.login-card {
  width: min(368px, 100%);
  padding: 28px 26px 24px;
  border-radius: 24px;
  background: var(--ui-surface);
  box-shadow: var(--ui-shadow-dialog);
  display: flex;
  flex-direction: column;
  gap: 18px;
}
.brand {
  display: flex;
  align-items: center;
  gap: 11px;
}
.mark {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  flex: none;
  overflow: hidden;
  background: linear-gradient(150deg, var(--ui-btn-hi), var(--ui-btn));
}
.mark img { width: 100%; height: 100%; object-fit: cover; }
h1 {
  margin: 0;
  font-size: 18px;
  font-weight: 650;
  line-height: 1.2;
}
.host {
  font-size: 12px;
  color: var(--ui-text-tertiary);
  margin-top: 2px;
}
.form {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  color: var(--ui-text-tertiary);
}
.foot a:hover,
.foot button:hover {
  color: var(--ui-text-primary);
}
</style>
