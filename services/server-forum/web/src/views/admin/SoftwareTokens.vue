<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { request } from '@/api/admin'
import { errorText, fmtTime } from './shared'

interface TokenInfo {
  id: string
  label: string
  hint: string
  last_used_at: string | null
}

const path = '/collection/software-tokens'
const tokens = ref<TokenInfo[]>([])
const label = ref('')
const token = ref('')
const busy = ref(false)
const error = ref('')

async function update(method: 'GET' | 'POST' | 'DELETE', id?: string) {
  busy.value = true
  error.value = ''
  try {
    if (method === 'POST') {
      await request(method, path, { label: label.value, token: token.value })
      label.value = ''
      token.value = ''
    } else if (method === 'DELETE') {
      await request(method, `${path}/${id}`)
    }
    tokens.value = await request<TokenInfo[]>('GET', path)
  } catch (err) {
    error.value = errorText(err)
  } finally {
    busy.value = false
  }
}

onMounted(() => void update('GET'))
</script>

<template>
  <details class="panel adm-card">
    <summary>开源软件 GitHub 令牌（{{ tokens.length }}）</summary>
    <p class="muted">采集任务依次使用令牌列表中的各个令牌。同一 GitHub 账号的令牌共享额度。</p>
    <form class="adm-actions" @submit.prevent="update('POST')">
      <input v-model="label" class="input" aria-label="令牌名称" placeholder="令牌名称" maxlength="80" required>
      <input v-model="token" class="input" type="password" aria-label="GitHub 令牌" placeholder="GitHub 令牌" autocomplete="new-password" maxlength="256" required>
      <button class="btn" :disabled="busy">验证并添加</button>
    </form>
    <p v-if="error" role="alert">{{ error }} <button class="btn btn-sm" :disabled="busy" @click="update('GET')">重新读取</button></p>
    <ul v-if="tokens.length">
      <li v-for="item in tokens" :key="item.id">
        {{ item.label }} · {{ item.hint }} · {{ item.last_used_at ? fmtTime(item.last_used_at) : '尚未使用' }}
        <button type="button" class="btn btn-sm" :disabled="busy" @click="update('DELETE', item.id)">移除</button>
      </li>
    </ul>
  </details>
</template>
