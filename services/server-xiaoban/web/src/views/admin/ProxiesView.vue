<script setup lang="ts">
import { onBeforeUnmount, onMounted, reactive, ref } from 'vue'
import * as api from '@/admin/api'
import { formatDuration } from '@/admin/format'
import type { ProxyInfo, ProxyProbe, ProxyStrategy } from '@/admin/types'
import ConfirmButton from '@/components/admin/ConfirmButton.vue'
import InlineEdit from '@/components/admin/InlineEdit.vue'
import ToggleSwitch from '@/components/admin/ToggleSwitch.vue'
import { toast } from '@/composables/toast'

const proxies = ref<ProxyInfo[]>([])
const strategy = ref<ProxyStrategy>('round_robin')
const loading = ref(true)
const loadError = ref<string | null>(null)

const draft = reactive({ name: '', url: '', busy: false, error: null as string | null })
const testing = ref<Record<number, boolean>>({})
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
  load()
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
    toast('代理已添加')
    await load()
  } catch (e) {
    draft.error = e instanceof Error ? e.message : '添加失败'
  } finally {
    draft.busy = false
  }
}

const setStrategy = (value: ProxyStrategy) => {
  strategy.value = value
  return perform(() => api.setProxyStrategy(value), '轮换方式已保存')
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
</script>

<template>
  <div class="a-page">
    <header class="a-head">
      <h1 class="display">代理池</h1>
      <p>
        访问供应商接口时可以经过这里的代理。每次请求从已启用的代理中选一个，连不上的代理会暂停 30 秒再试。
        要让某个供应商使用代理池，请在「供应商密钥」里打开它的「通过代理池访问」。
      </p>
    </header>

    <p v-if="loading" class="a-note">加载中…</p>
    <p v-else-if="loadError" class="a-error" role="alert">
      {{ loadError }}
      <button class="a-link" type="button" @click="load">重试</button>
    </p>

    <section class="a-section" aria-label="轮换方式">
      <div class="a-section-head">
        <div>
          <h2 class="display">轮换方式</h2>
          <p class="a-sub">{{ strategy === 'round_robin' ? '按顺序依次使用每个代理' : '每次随机挑一个代理' }}</p>
        </div>
        <div class="a-segment" role="group" aria-label="轮换方式">
          <button type="button" :aria-pressed="strategy === 'round_robin'" @click="setStrategy('round_robin')">轮询</button>
          <button type="button" :aria-pressed="strategy === 'random'" @click="setStrategy('random')">随机</button>
        </div>
      </div>
    </section>

    <section class="a-section" aria-label="代理列表">
      <div class="a-section-head">
        <div>
          <h2 class="display">代理</h2>
          <p class="a-sub">支持 http、https、socks5、socks5h</p>
        </div>
      </div>

      <ul v-if="proxies.length" class="a-rows">
        <li v-for="proxy in proxies" :key="proxy.id" class="a-row">
          <div class="a-row-main">
            <div class="a-row-title">
              <InlineEdit :value="proxy.name" label="名称" @save="rename(proxy, $event)" />
              <span v-if="!proxy.enabled" class="a-tag">已停用</span>
              <span v-else-if="proxy.cooldown_secs" class="a-tag warn">
                暂停中，{{ formatDuration(proxy.cooldown_secs) }}后恢复
              </span>
              <span v-if="proxy.has_auth" class="a-tag">含账号密码</span>
            </div>
            <div class="a-row-meta">
              <span class="a-mono">{{ proxy.url }}</span>
              <span>使用 {{ proxy.requests }} 次</span>
              <span>失败 {{ proxy.failures }} 次</span>
            </div>
            <p v-if="proxy.last_error" class="a-error detail">最近一次失败：{{ proxy.last_error }}</p>
          </div>
          <div class="a-row-actions">
            <button class="a-btn small" type="button" :disabled="testing[proxy.id]" @click="test(proxy)">
              {{ testing[proxy.id] ? '测试中…' : '测试连接' }}
            </button>
            <ToggleSwitch
              :model-value="proxy.enabled"
              :label="proxy.enabled ? '停用这个代理' : '启用这个代理'"
              @update:model-value="toggle(proxy, $event)"
            />
            <ConfirmButton label="删除" confirm-label="确认删除" @confirm="remove(proxy)" />
          </div>

          <ul v-if="results[proxy.id]" class="a-row-extra probes" aria-label="连接测试结果">
            <li v-for="probe in results[proxy.id]" :key="probe.target">
              <span class="a-tag" :class="probe.ok ? 'ok' : 'danger'">{{ probe.ok ? '可连接' : '连不上' }}</span>
              <span class="target">{{ probe.target }}</span>
              <span v-if="probe.ok" class="a-small">{{ probe.latency_ms }} ms，返回 {{ probe.status }}</span>
              <span v-else class="a-error">{{ probe.error }}</span>
            </li>
          </ul>
        </li>
      </ul>
      <p v-else class="a-empty">还没有代理。添加后，打开了「通过代理池访问」的供应商会经过它们。</p>

      <form class="a-form inline" @submit.prevent="addProxy">
        <label class="a-field">
          名称
          <input v-model="draft.name" class="a-input" placeholder="例如 香港节点" maxlength="40" required />
        </label>
        <label class="a-field">
          地址
          <input
            v-model="draft.url"
            class="a-input"
            type="text"
            autocomplete="off"
            spellcheck="false"
            placeholder="http://127.0.0.1:7890 或 socks5://用户:密码@主机:1080"
            required
          />
        </label>
        <button class="a-btn" type="submit" :disabled="draft.busy">{{ draft.busy ? '添加中…' : '添加代理' }}</button>
        <p v-if="draft.error" class="a-error err" role="alert">{{ draft.error }}</p>
      </form>
    </section>
  </div>
</template>

<style scoped>
.detail {
  margin-top: 4px;
  overflow-wrap: anywhere;
}

.probes {
  display: grid;
  gap: 6px;
  padding: 10px 0 2px;
  list-style: none;
}

.probes li {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px 10px;
}

.target {
  min-width: 5.5em;
  font-weight: 500;
}

.err {
  grid-column: 1 / -1;
}
</style>
