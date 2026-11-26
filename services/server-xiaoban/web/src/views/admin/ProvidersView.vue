<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, reactive, ref } from 'vue'
import { useRoute } from 'vue-router'
import * as api from '@/admin/api'
import { formatDuration } from '@/admin/format'
import type { KeyInfo, ProviderConfig } from '@/admin/types'
import AdminMoreMenu, { type MoreMenuItem } from '@/components/admin/AdminMoreMenu.vue'
import HotlinkPolicy from '@/components/admin/HotlinkPolicy.vue'
import InlineEdit from '@/components/admin/InlineEdit.vue'
import ToggleSwitch from '@/components/admin/ToggleSwitch.vue'
import Icon from '@/components/Icon.vue'
import { toast } from '@/composables/toast'

const route = useRoute()

const GUIDE: Record<string, { name: string; docs: string; warn?: string }> = {
  unsplash: { name: 'Access Key', docs: 'https://unsplash.com/developers' },
  pixabay: {
    name: 'API Key',
    docs: 'https://pixabay.com/api/docs/',
    warn: 'Pixabay 图片链接 24 小时失效且禁止永久直链，图片默认保存本站副本。',
  },
  pexels: { name: 'API Key', docs: 'https://www.pexels.com/api/' },
  okslop: {
    name: 'API Key',
    docs: 'https://okslop.com/developers',
    warn: '免费层每日限 1000 次且仅支持图片，请勿批量抓取。',
  },
  bing: { name: 'API Key (可选)', docs: 'https://www.bing.com' },
}

const providers = ref<ProviderConfig[]>([])
const activeId = ref(typeof route.query.source === 'string' ? route.query.source : 'unsplash')
const loading = ref(true)
const loadError = ref<string | null>(null)
const draft = reactive({ label: '', key: '', busy: false, error: null as string | null })
const showKey = ref(false)

let refreshTimer: ReturnType<typeof setInterval> | undefined

const active = computed(() => providers.value.find((p) => p.id === activeId.value) ?? providers.value[0] ?? null)

async function load() {
  try {
    providers.value = await api.listProviders()
    if (providers.value.length && !providers.value.some((p) => p.id === activeId.value)) {
      activeId.value = providers.value[0].id
    }
    loadError.value = null
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : '加载失败'
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  void load()
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

async function addKey() {
  if (!active.value || draft.busy) return
  draft.busy = true
  draft.error = null
  try {
    await api.addKey(active.value.id, draft.label, draft.key)
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

function keyMenu(provider: ProviderConfig, key: KeyInfo): MoreMenuItem[] {
  return [
    {
      label: '删除密钥',
      icon: 'trash',
      danger: true,
      desc: '删除后自动清空搜索缓存',
      confirmText: '确认删除？',
      action: () => void removeKey(provider, key),
    },
  ]
}
</script>

<template>
  <div class="xb-page">
    <header class="xb-page-head">
      <h1 class="xb-page-title">图源与密钥</h1>
      <div v-if="active && GUIDE[active.id]" class="xb-page-actions">
        <a :href="GUIDE[active.id].docs" target="_blank" rel="noopener noreferrer" class="xb-btn-ghost">
          <Icon name="external" :size="13" /><span>获取 {{ GUIDE[active.id].name }}</span>
        </a>
      </div>
    </header>

    <div v-if="loadError" class="xb-alert is-danger" role="alert">
      <div class="xb-alert-l"><Icon name="alert" :size="15" /><span>{{ loadError }}</span></div>
      <button type="button" class="xb-btn-ghost is-sm" @click="load">重试</button>
    </div>

    <div v-if="loading && !providers.length" class="xb-group xb-empty-box">加载中…</div>

    <template v-else-if="active">
      <div class="src-tabs" role="tablist">
        <button
          v-for="s in providers"
          :key="s.id"
          type="button"
          class="src-tab"
          :class="{ 'is-active': s.id === active.id }"
          @click="activeId = s.id; draft.error = null"
        >
          <div>
            <div class="src-name">{{ s.name }}</div>
            <div class="src-sub">
              {{ s.keys.length ? `${s.keys.filter((k) => k.enabled).length}/${s.keys.length} 密钥` : '免密/未配' }}
            </div>
          </div>
          <i
            class="xb-dot-status"
            :class="s.keys.some((k) => k.cooldown_secs) ? 'warn' : s.keys.some((k) => k.enabled) || s.id === 'bing' ? 'ok' : 'off'"
          />
        </button>
      </div>

      <div v-if="GUIDE[active.id]?.warn" class="xb-alert">
        <div class="xb-alert-l">
          <Icon name="alert" :size="15" />
          <span>{{ GUIDE[active.id].warn }}</span>
        </div>
      </div>

      <div class="xb-group">
        <div class="xb-row" title="开启后访问此供应商接口将通过代理池转发">
          <span style="font-weight: 550">通过代理池访问</span>
          <ToggleSwitch
            :model-value="active.use_proxy"
            :label="`${active.name} 通过代理池访问`"
            @update:model-value="setProxy(active, $event)"
          />
        </div>
        <HotlinkPolicy :provider="active" @changed="load" />
      </div>

      <section>
        <div class="xb-sec-head"><h2 class="xb-sec-title">密钥池</h2></div>
        <div class="xb-group">
          <div v-if="!active.keys.length" class="xb-empty-box" style="padding: 26px">
            <span>暂无配置密钥</span>
          </div>

          <div v-for="key in active.keys" :key="key.id" class="xb-row">
            <div style="flex: 1; min-width: 180px">
              <div class="xb-row-title">
                <InlineEdit :value="key.label" label="名称" @save="renameKey(active, key, $event)" />
                <span v-if="!key.enabled" class="xb-pill">已停用</span>
                <span v-else-if="key.cooldown_secs" class="xb-pill is-warn">
                  <i class="dot" />冷却 {{ formatDuration(key.cooldown_secs) }}
                </span>
                <span v-else class="xb-pill is-ok"><i class="dot" />可用</span>
                <span v-if="key.last_status && key.last_status >= 400" class="xb-pill is-danger">
                  HTTP {{ key.last_status }}
                </span>
              </div>
              <div class="xb-row-meta">
                <span class="xb-mono">{{ key.preview }}</span>
                <span>·</span>
                <span class="xb-num">请求 {{ key.requests }} 次</span>
                <span>·</span>
                <span class="xb-num">失败 {{ key.failures }} 次</span>
              </div>
            </div>
            <div class="xb-row-actions">
              <ToggleSwitch
                :model-value="key.enabled"
                :label="key.enabled ? '停用密钥' : '启用密钥'"
                @update:model-value="toggleKey(active, key, $event)"
              />
              <AdminMoreMenu :items="keyMenu(active, key)" />
            </div>
          </div>

          <form class="xb-row add-row" @submit.prevent="addKey">
            <input
              v-model="draft.label"
              class="xb-input"
              style="flex: 1; min-width: 120px"
              placeholder="备注名称（如 主账号）"
              maxlength="40"
              required
            />
            <div class="xb-affix has-r" style="flex: 2; min-width: 180px">
              <input
                v-model="draft.key"
                class="xb-input xb-mono"
                :type="showKey ? 'text' : 'password'"
                autocomplete="off"
                spellcheck="false"
                :placeholder="`粘贴 ${GUIDE[active.id]?.name || 'API Key'}`"
                required
              />
              <button type="button" class="af-r" aria-label="显示或隐藏密钥" @click="showKey = !showKey">
                <Icon :name="showKey ? 'eye-off' : 'eye'" :size="14" />
              </button>
            </div>
            <button type="submit" class="xb-btn-primary" :disabled="draft.busy">
              {{ draft.busy ? '添加中…' : '添加密钥' }}
            </button>
            <p v-if="draft.error" class="xb-error err-line" role="alert">{{ draft.error }}</p>
          </form>
        </div>
      </section>
    </template>
  </div>
</template>

<style scoped>
.src-tabs {
  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
  gap: 8px;
}
.src-tab {
  padding: 11px 12px;
  border-radius: 10px;
  background: var(--ui-surface-sunken);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  text-align: left;
  transition: background 0.14s;
}
.src-tab:hover { background: var(--ui-control); }
.src-tab.is-active {
  background: var(--ui-soft);
  color: var(--ui-ink);
}
.src-name {
  font-size: 13px;
  font-weight: 600;
  line-height: 1.2;
}
.src-sub {
  font-size: 11.5px;
  color: var(--ui-text-tertiary);
  margin-top: 3px;
  line-height: 1;
}
.src-tab.is-active .src-sub {
  color: var(--ui-ink);
  opacity: 0.8;
}
.add-row { gap: 10px; }
.err-line { flex-basis: 100%; }

@media (max-width: 900px) { .src-tabs { grid-template-columns: repeat(3, minmax(0, 1fr)); } }
@media (max-width: 600px) { .src-tabs { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
</style>
