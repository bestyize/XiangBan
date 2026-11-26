<script setup lang="ts">
import { onBeforeUnmount, onMounted, reactive, ref } from 'vue'
import * as api from '@/admin/api'
import { formatDuration } from '@/admin/format'
import type { ProxyInfo, ProxyProbe, ProxyStrategy } from '@/admin/types'
import AdminMoreMenu, { type MoreMenuItem } from '@/components/admin/AdminMoreMenu.vue'
import InlineEdit from '@/components/admin/InlineEdit.vue'
import ToggleSwitch from '@/components/admin/ToggleSwitch.vue'
import Icon from '@/components/Icon.vue'
import { toast } from '@/composables/toast'

const proxies = ref<ProxyInfo[]>([])
const strategy = ref<ProxyStrategy>('round_robin')
const loading = ref(true)
const loadError = ref<string | null>(null)

const draft = reactive({ name: '', url: '', busy: false, error: null as string | null })
const testing = ref<Record<number, boolean>>({})
const testingAll = ref(false)
const results = ref<Record<number, ProxyProbe[]>>({})

let refreshTimer: ReturnType<typeof setInterval> | undefined

async function load() {
  try {
    const data = await api.listProxies()
    proxies.value = data.proxies
    strategy.value = data.strategy
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

async function addProxy() {
  if (draft.busy) return
  draft.busy = true
  draft.error = null
  try {
    await api.addProxy(draft.name, draft.url)
    draft.name = ''
    draft.url = ''
    toast('代理节点已添加')
    await load()
  } catch (e) {
    draft.error = e instanceof Error ? e.message : '添加失败'
  } finally {
    draft.busy = false
  }
}

const setStrategy = (value: ProxyStrategy) => {
  strategy.value = value
  return perform(() => api.setProxyStrategy(value), value === 'round_robin' ? '已切为顺序轮询' : '已切为随机挑选')
}

const toggle = (proxy: ProxyInfo, enabled: boolean) => perform(() => api.updateProxy(proxy.id, { enabled }))
const rename = (proxy: ProxyInfo, name: string) => perform(() => api.updateProxy(proxy.id, { name }), '名称已修改')
const remove = (proxy: ProxyInfo) => perform(() => api.deleteProxy(proxy.id), '代理已删除')

async function test(proxy: ProxyInfo) {
  testing.value = { ...testing.value, [proxy.id]: true }
  try {
    results.value = { ...results.value, [proxy.id]: await api.testProxy(proxy.id) }
  } catch (e) {
    toast(e instanceof Error ? e.message : '测试失败')
  } finally {
    testing.value = { ...testing.value, [proxy.id]: false }
  }
}

async function testAll() {
  if (testingAll.value || !proxies.value.length) return
  testingAll.value = true
  await Promise.allSettled(proxies.value.map((p) => test(p)))
  testingAll.value = false
  toast('全部节点测速完成')
}

function menuFor(proxy: ProxyInfo): MoreMenuItem[] {
  return [
    {
      label: '删除节点',
      icon: 'trash',
      danger: true,
      confirmText: '确认删除？',
      action: () => void remove(proxy),
    },
  ]
}
</script>

<template>
  <div class="xb-page is-narrow">
    <header class="xb-page-head">
      <h1 class="xb-page-title">代理池</h1>
      <div class="xb-page-actions">
        <div class="xb-seg" role="group" aria-label="轮换方式">
          <button type="button" :class="{ 'is-active': strategy === 'round_robin' }" @click="setStrategy('round_robin')">
            顺序轮询
          </button>
          <button type="button" :class="{ 'is-active': strategy === 'random' }" @click="setStrategy('random')">
            随机挑选
          </button>
        </div>
        <button type="button" class="xb-btn-primary" :disabled="testingAll || !proxies.length" @click="testAll">
          <Icon name="bolt" :size="14" /><span>{{ testingAll ? '测速中…' : '全部测速' }}</span>
        </button>
      </div>
    </header>

    <div v-if="loadError" class="xb-alert is-danger" role="alert">
      <div class="xb-alert-l"><Icon name="alert" :size="15" /><span>{{ loadError }}</span></div>
      <button type="button" class="xb-btn-ghost is-sm" @click="load">重试</button>
    </div>

    <div v-if="loading && !proxies.length" class="xb-group xb-empty-box">加载中…</div>

    <div v-else class="xb-group">
      <div v-if="!proxies.length" class="xb-empty-box" style="padding: 28px">
        <span>暂无代理节点</span>
      </div>

      <div v-for="proxy in proxies" :key="proxy.id" class="xb-row">
        <div style="flex: 1; min-width: 220px">
          <div class="xb-row-title">
            <InlineEdit :value="proxy.name" label="名称" @save="rename(proxy, $event)" />
            <span v-if="!proxy.enabled" class="xb-pill">已停用</span>
            <span v-else-if="proxy.cooldown_secs" class="xb-pill is-warn">
              暂停 {{ formatDuration(proxy.cooldown_secs) }}
            </span>
            <span v-else class="xb-pill is-ok"><i class="dot" />在线</span>
            <span v-if="proxy.has_auth" class="xb-pill">含鉴权</span>
          </div>
          <div class="xb-row-meta">
            <span class="xb-mono">{{ proxy.url }}</span>
            <span>·</span>
            <span class="xb-num">使用 {{ proxy.requests }} 次</span>
            <span>·</span>
            <span class="xb-num">失败 {{ proxy.failures }} 次</span>
          </div>
          <p v-if="proxy.last_error" class="xb-error err-msg">{{ proxy.last_error }}</p>
        </div>

        <div class="xb-row-actions">
          <button type="button" class="xb-btn-ghost is-sm" :disabled="testing[proxy.id]" @click="test(proxy)">
            {{ testing[proxy.id] ? '测速中…' : '测速' }}
          </button>
          <ToggleSwitch
            :model-value="proxy.enabled"
            :label="proxy.enabled ? '停用代理' : '启用代理'"
            @update:model-value="toggle(proxy, $event)"
          />
          <AdminMoreMenu :items="menuFor(proxy)" />
        </div>

        <div v-if="results[proxy.id]" class="probe-grid">
          <div
            v-for="probe in results[proxy.id]"
            :key="probe.target"
            class="probe-card"
            :title="probe.ok ? `HTTP ${probe.status}` : (probe.error || '连接失败')"
          >
            <span>{{ probe.target }}</span>
            <span class="xb-pill" :class="probe.ok ? 'is-ok' : 'is-danger'">
              {{ probe.ok ? `${probe.latency_ms}ms` : '失败' }}
            </span>
          </div>
        </div>
      </div>

      <form class="xb-row add-row" @submit.prevent="addProxy">
        <input
          v-model="draft.name"
          class="xb-input"
          style="flex: 1; min-width: 120px"
          placeholder="节点名称（如 香港节点）"
          maxlength="40"
          required
        />
        <input
          v-model="draft.url"
          class="xb-input xb-mono"
          style="flex: 2; min-width: 200px"
          type="text"
          autocomplete="off"
          spellcheck="false"
          placeholder="http:// 或 socks5h://user:pass@host:1080"
          required
        />
        <button type="submit" class="xb-btn-primary" :disabled="draft.busy">
          {{ draft.busy ? '添加中…' : '添加节点' }}
        </button>
        <p v-if="draft.error" class="xb-error err-line" role="alert">{{ draft.error }}</p>
      </form>
    </div>
  </div>
</template>

<style scoped>
.err-msg {
  margin-top: 4px;
  overflow-wrap: anywhere;
}
.probe-grid {
  width: 100%;
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 8px;
  padding-top: 8px;
}
.probe-card {
  padding: 8px 10px;
  border-radius: 8px;
  background: var(--ui-surface);
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
}
.add-row { gap: 10px; }
.err-line { flex-basis: 100%; }

@media (max-width: 640px) {
  .probe-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
}
</style>
