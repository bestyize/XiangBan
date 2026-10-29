<script setup lang="ts">
import { onBeforeUnmount, onMounted, reactive, ref } from 'vue'
import * as api from '@/admin/api'
import { formatDuration } from '@/admin/format'
import type { KeyInfo, ProviderConfig } from '@/admin/types'
import ConfirmButton from '@/components/admin/ConfirmButton.vue'
import HotlinkPolicy from '@/components/admin/HotlinkPolicy.vue'
import InlineEdit from '@/components/admin/InlineEdit.vue'
import ToggleSwitch from '@/components/admin/ToggleSwitch.vue'
import { toast } from '@/composables/toast'

const GUIDE: Record<string, { name: string; docs: string }> = {
  unsplash: { name: 'Access Key', docs: 'https://unsplash.com/developers' },
  pixabay: { name: 'API Key', docs: 'https://pixabay.com/api/docs/' },
  pexels: { name: 'API Key', docs: 'https://www.pexels.com/api/' },
  okslop: { name: 'API Key', docs: 'https://okslop.com/developers' },
  bing: { name: 'API Key (可选)', docs: 'https://www.bing.com' },
}

interface Draft {
  label: string
  key: string
  busy: boolean
  error: string | null
}

const providers = ref<ProviderConfig[]>([])
const loading = ref(true)
const loadError = ref<string | null>(null)
const drafts = reactive<Record<string, Draft>>({})

let refreshTimer: ReturnType<typeof setInterval> | undefined

async function load() {
  try {
    providers.value = await api.listProviders()
    for (const p of providers.value) {
      drafts[p.id] ??= { label: '', key: '', busy: false, error: null }
    }
    loadError.value = null
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : '加载失败'
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  load()
  // Resting keys count down and request counters move, so keep the numbers fresh.
  refreshTimer = setInterval(load, 15_000)
})

onBeforeUnmount(() => clearInterval(refreshTimer))

async function perform(action: () => Promise<unknown>, done?: string) {
  try {
    await action()
    if (done) toast(done)
  } catch (e) {
    toast(e instanceof Error ? e.message : '操作失败')
  }
  await load()
}

async function addKey(provider: ProviderConfig) {
  const draft = drafts[provider.id]
  if (draft.busy) return
  draft.busy = true
  draft.error = null
  try {
    await api.addKey(provider.id, draft.label, draft.key)
    draft.label = ''
    draft.key = ''
    toast('密钥已添加')
    await load()
  } catch (e) {
    draft.error = e instanceof Error ? e.message : '添加失败'
  } finally {
    draft.busy = false
  }
}

const toggleKey = (provider: ProviderConfig, key: KeyInfo, enabled: boolean) =>
  perform(() => api.updateKey(provider.id, key.id, { enabled }))

const renameKey = (provider: ProviderConfig, key: KeyInfo, label: string) =>
  perform(() => api.updateKey(provider.id, key.id, { label }), '名称已修改')

const removeKey = (provider: ProviderConfig, key: KeyInfo) =>
  perform(() => api.deleteKey(provider.id, key.id), '密钥已删除')

const setProxy = (provider: ProviderConfig, on: boolean) =>
  perform(() => api.setProviderProxy(provider.id, on), on ? '已通过代理池访问' : '已改为直连')

function enabledCount(provider: ProviderConfig) {
  return provider.keys.filter((k) => k.enabled).length
}
</script>

<template>
  <div class="a-page">
    <header class="a-head">
      <h1 class="display">供应商密钥</h1>
      <p>每个供应商可以添加多个密钥。请求会轮流使用已启用的密钥，某个密钥被限流或失效时会自动换下一个。</p>
    </header>

    <p v-if="loading" class="a-note">加载中…</p>
    <p v-else-if="loadError" class="a-error" role="alert">
      {{ loadError }}
      <button class="a-link" type="button" @click="load">重试</button>
    </p>

    <section v-for="provider in providers" :key="provider.id" class="a-section" :aria-label="provider.name">
      <div class="a-section-head">
        <div>
          <h2 class="display">{{ provider.name }}</h2>
          <p class="a-sub">
            <template v-if="provider.keys.length">
              {{ enabledCount(provider) }} 个密钥已启用，共 {{ provider.keys.length }} 个
            </template>
            <template v-else>未添加密钥，前台不会使用这个来源</template>
            ·
            <a :href="GUIDE[provider.id].docs" class="a-link" target="_blank" rel="noopener noreferrer">
              获取 {{ GUIDE[provider.id].name }}
            </a>
          </p>
        </div>
        <label class="proxy-toggle">
          <span>通过代理池访问</span>
          <ToggleSwitch
            :model-value="provider.use_proxy"
            :label="`${provider.name} 通过代理池访问`"
            @update:model-value="setProxy(provider, $event)"
          />
        </label>
      </div>

      <HotlinkPolicy :provider="provider" @changed="load" />

      <ul v-if="provider.keys.length" class="a-rows">
        <li v-for="key in provider.keys" :key="key.id" class="a-row">
          <div class="a-row-main">
            <div class="a-row-title">
              <InlineEdit :value="key.label" label="名称" @save="renameKey(provider, key, $event)" />
              <span v-if="!key.enabled" class="a-tag">已停用</span>
              <span v-else-if="key.cooldown_secs" class="a-tag warn">
                暂停中，{{ formatDuration(key.cooldown_secs) }}后恢复
              </span>
              <span v-else-if="key.last_status && key.last_status >= 400" class="a-tag danger">
                上次返回 {{ key.last_status }}
              </span>
            </div>
            <div class="a-row-meta">
              <span class="a-mono">{{ key.preview }}</span>
              <span>请求 {{ key.requests }} 次</span>
              <span>失败 {{ key.failures }} 次</span>
            </div>
          </div>
          <div class="a-row-actions">
            <ToggleSwitch
              :model-value="key.enabled"
              :label="key.enabled ? '停用这个密钥' : '启用这个密钥'"
              @update:model-value="toggleKey(provider, key, $event)"
            />
            <ConfirmButton label="删除" confirm-label="确认删除" @confirm="removeKey(provider, key)" />
          </div>
        </li>
      </ul>
      <p v-else class="a-empty">添加第一个密钥后，这个来源会出现在前台的筛选里。</p>

      <form class="a-form inline" @submit.prevent="addKey(provider)">
        <label class="a-field">
          名称
          <input v-model="drafts[provider.id].label" class="a-input" placeholder="例如 主账号" maxlength="40" required />
        </label>
        <label class="a-field">
          {{ GUIDE[provider.id].name }}
          <input
            v-model="drafts[provider.id].key"
            class="a-input"
            type="password"
            autocomplete="off"
            spellcheck="false"
            placeholder="粘贴密钥"
            required
          />
        </label>
        <button class="a-btn" type="submit" :disabled="drafts[provider.id].busy">
          {{ drafts[provider.id].busy ? '添加中…' : '添加密钥' }}
        </button>
        <p v-if="drafts[provider.id].error" class="a-error err" role="alert">{{ drafts[provider.id].error }}</p>
      </form>
    </section>
  </div>
</template>

<style scoped>
.proxy-toggle {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  color: var(--text-2);
  font-size: 13px;
}

.err {
  grid-column: 1 / -1;
}
</style>
