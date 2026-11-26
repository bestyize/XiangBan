<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import * as api from '@/admin/api'
import Icon from '@/components/Icon.vue'
import { useAdmin } from '@/stores/admin'
import { toast } from '@/composables/toast'

const admin = useAdmin()

const form = reactive({
  currentPassword: '',
  username: admin.user?.username ?? '',
  newPassword: '',
  confirmPassword: '',
})
const showCur = ref(false)
const showNew = ref(false)
const showCfm = ref(false)
const saving = ref(false)
const error = ref<string | null>(null)

watch(
  () => admin.user?.username,
  (name) => {
    if (name && !form.username) form.username = name
  },
)

const usernameChanged = computed(() => form.username.trim() !== (admin.user?.username ?? ''))
const passwordChanged = computed(() => form.newPassword !== '')
const mismatch = computed(
  () => passwordChanged.value && form.confirmPassword !== '' && form.newPassword !== form.confirmPassword,
)
const canSave = computed(
  () =>
    !saving.value &&
    form.currentPassword !== '' &&
    (usernameChanged.value || passwordChanged.value) &&
    (!passwordChanged.value || form.newPassword === form.confirmPassword),
)

async function save() {
  if (!canSave.value) return
  saving.value = true
  error.value = null
  try {
    const user = await api.updateAccount({
      current_password: form.currentPassword,
      new_username: usernameChanged.value ? form.username.trim() : undefined,
      new_password: passwordChanged.value ? form.newPassword : undefined,
    })
    admin.user = user
    form.username = user.username
    form.currentPassword = ''
    form.newPassword = ''
    form.confirmPassword = ''
    toast(passwordChanged.value ? '已修改，其他设备登录已失效' : '用户名已修改')
  } catch (e) {
    error.value = e instanceof Error ? e.message : '保存失败'
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="xb-page is-narrow">
    <header class="xb-page-head">
      <h1 class="xb-page-title">管理员账户</h1>
    </header>

    <div v-if="admin.user?.is_default_password" class="xb-alert" role="alert">
      <div class="xb-alert-l">
        <Icon name="alert" :size="15" />
        <span>当前仍是默认密码 123456，请尽快修改为强密码。</span>
      </div>
    </div>

    <form class="xb-group xb-group-pad acc-form" @submit.prevent="save">
      <label class="xb-field">
        <span class="xb-field-label">当前密码</span>
        <div class="xb-affix has-r">
          <input
            v-model="form.currentPassword"
            class="xb-input"
            :type="showCur ? 'text' : 'password'"
            autocomplete="current-password"
            placeholder="验证当前登录密码"
            required
          />
          <button type="button" class="af-r" aria-label="显示或隐藏当前密码" @click="showCur = !showCur">
            <Icon :name="showCur ? 'eye-off' : 'eye'" :size="14" />
          </button>
        </div>
      </label>

      <label class="xb-field" title="3 到 32 位，只能使用字母、数字、下划线、点和短横线">
        <span class="xb-field-label">用户名</span>
        <input
          v-model="form.username"
          class="xb-input"
          autocomplete="username"
          autocapitalize="off"
          minlength="3"
          maxlength="32"
          required
        />
      </label>

      <label class="xb-field" title="至少 8 位，修改后其他设备会话自动失效">
        <span class="xb-field-label">新密码</span>
        <div class="xb-affix has-r">
          <input
            v-model="form.newPassword"
            class="xb-input"
            :type="showNew ? 'text' : 'password'"
            autocomplete="new-password"
            minlength="8"
            maxlength="128"
            placeholder="不修改密码请留空（至少 8 位）"
          />
          <button type="button" class="af-r" aria-label="显示或隐藏新密码" @click="showNew = !showNew">
            <Icon :name="showNew ? 'eye-off' : 'eye'" :size="14" />
          </button>
        </div>
      </label>

      <label v-if="passwordChanged" class="xb-field">
        <span class="xb-field-label">确认新密码</span>
        <div class="xb-affix has-r">
          <input
            v-model="form.confirmPassword"
            class="xb-input"
            :type="showCfm ? 'text' : 'password'"
            autocomplete="new-password"
            placeholder="再次输入新密码"
            required
          />
          <button type="button" class="af-r" aria-label="显示或隐藏确认密码" @click="showCfm = !showCfm">
            <Icon :name="showCfm ? 'eye-off' : 'eye'" :size="14" />
          </button>
        </div>
        <span v-if="mismatch" class="xb-error" role="alert">两次输入的密码不一致</span>
      </label>

      <p v-if="error" class="xb-error" role="alert">{{ error }}</p>

      <div class="acc-foot">
        <button class="xb-btn-primary" type="submit" :disabled="!canSave">
          {{ saving ? '保存中…' : '保存修改' }}
        </button>
      </div>
    </form>
  </div>
</template>

<style scoped>
.acc-form {
  display: flex;
  flex-direction: column;
  gap: 14px;
  max-width: 460px;
}
.acc-foot {
  display: flex;
  justify-content: flex-end;
  padding-top: 4px;
}
</style>
