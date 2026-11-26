<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { getAiCatalogue, saveAiCatalogue } from '@/admin/api'
import { aiTypeLabel, newAiProvider, type AiCatalogue, type AiProvider, type AiProviderType } from '@/admin/ai-types'
import AiProviderDialog from '@/components/admin/AiProviderDialog.vue'
import AdminMoreMenu, { type MoreMenuItem } from '@/components/admin/AdminMoreMenu.vue'
import ToggleSwitch from '@/components/admin/ToggleSwitch.vue'
import Icon from '@/components/Icon.vue'
import { toast } from '@/composables/toast'

const clone = <T,>(value: T): T => JSON.parse(JSON.stringify(value))
const message = (e: unknown) => (e instanceof Error ? e.message : String(e))

const config = ref<AiCatalogue | null>(null)
const loading = ref(true)
const busy = ref(false)
const error = ref('')
const filterType = ref<'all' | AiProviderType>('all')

const editing = ref<{ provider: AiProvider; originalId: string | null } | null>(null)
const dialogError = ref('')

const entryOn = computed(() => config.value?.xiaobanAi?.enabled ?? true)
const typeCls = { third: '', free: 'is-ok', paid: 'is-warn' } as const

const filteredProviders = computed(() => {
  const list = config.value?.providers ?? []
  return filterType.value === 'all' ? list : list.filter((p) => p.type === filterType.value)
})

async function load() {
  loading.value = true
  try {
    config.value = await getAiCatalogue()
    error.value = ''
  } catch (e) {
    error.value = message(e)
  } finally {
    loading.value = false
  }
}

onMounted(load)

async function publish(next: AiCatalogue): Promise<string> {
  busy.value = true
  try {
    config.value = await saveAiCatalogue(next)
    return ''
  } catch (e) {
    return message(e)
  } finally {
    busy.value = false
  }
}

async function change(next: AiCatalogue, done: string) {
  const refused = await publish(next)
  error.value = refused
  if (!refused) toast(done)
}

function openEditor(provider?: AiProvider) {
  dialogError.value = ''
  editing.value = provider
    ? { provider: clone(provider), originalId: provider.id }
    : { provider: newAiProvider(), originalId: null }
}

async function saveProvider(provider: AiProvider) {
  if (!config.value || !editing.value) return
  const next = clone(config.value)
  const { originalId } = editing.value
  if (next.providers.some((v) => v.id === provider.id && v.id !== originalId)) {
    dialogError.value = `ID「${provider.id}」已经被另一个供应商使用`
    return
  }
  const at = next.providers.findIndex((v) => v.id === originalId)
  if (at < 0) next.providers.push(provider)
  else next.providers[at] = provider
  dialogError.value = await publish(next)
  if (!dialogError.value) {
    editing.value = null
    error.value = ''
    toast(originalId ? '已保存并生效' : '已添加，客户端下次刷新后可见')
  }
}

function toggle(provider: AiProvider) {
  if (!config.value) return
  const next = clone(config.value)
  const target = next.providers.find((v) => v.id === provider.id)
  if (!target) return
  target.enabled = !provider.enabled
  void change(next, target.enabled ? `已启用 ${provider.provider}` : `已停用 ${provider.provider}`)
}

function setEntry() {
  if (!config.value) return
  const next = clone(config.value)
  next.xiaobanAi = { enabled: !entryOn.value }
  void change(next, next.xiaobanAi.enabled ? '已开启“小伴 AI”入口' : '已关闭“小伴 AI”入口')
}

function remove(provider: AiProvider) {
  if (!config.value) return
  const next = clone(config.value)
  next.providers = next.providers.filter((v) => v.id !== provider.id)
  void change(next, `已删除 ${provider.provider}`)
}

const enabledModels = (p: AiProvider) => p.models.filter((m) => m.enabled).length
const credential = (p: AiProvider) =>
  p.type === 'third' ? '用户自备' : p.type === 'paid' ? `${p.userAccess.length} 人已授权` : p.hasKey ? '已配置' : '免密网关'

function menuFor(p: AiProvider): MoreMenuItem[] {
  return [
    { label: '编辑与模型', icon: 'edit', action: () => openEditor(p) },
    { sep: true },
    {
      label: '删除供应商',
      icon: 'trash',
      danger: true,
      confirmText: '确认删除？',
      action: () => remove(p),
    },
  ]
}
</script>

<template>
  <div class="xb-page">
    <header class="xb-page-head">
      <div class="xb-tabs-lg">
        <button
          v-for="tab in [
            { v: 'all', l: '全部服务商' },
            { v: 'third', l: '第三方' },
            { v: 'free', l: '小伴免费' },
            { v: 'paid', l: '小伴付费' },
          ]"
          :key="tab.v"
          type="button"
          :class="{ 'is-active': filterType === tab.v }"
          @click="filterType = tab.v as 'all' | AiProviderType"
        >
          {{ tab.l }}
        </button>
      </div>
      <div class="xb-page-actions">
        <button
          type="button"
          class="xb-btn-ghost"
          :title="config ? `当前修订 #${config.revision}` : '刷新'"
          :disabled="busy || loading"
          @click="load"
        >
          <Icon name="refresh" :size="14" /><span>刷新</span>
        </button>
        <button type="button" class="xb-btn-primary" :disabled="busy || !config" @click="openEditor()">
          <Icon name="plus" :size="14" /><span>新增供应商</span>
        </button>
      </div>
    </header>

    <div v-if="error" class="xb-alert is-danger" role="alert">
      <div class="xb-alert-l"><Icon name="alert" :size="15" /><span>{{ error }}</span></div>
      <button type="button" class="xb-btn-ghost is-sm" @click="load">重试</button>
    </div>

    <div class="xb-group">
      <div class="xb-row" title="桌面端卡片列表排第一的官方服务入口，关闭后客户端下次刷新将隐藏">
        <div class="xb-row-main">
          <span class="xb-pill is-accent"><Icon name="sparkles" :size="12" /><span>小伴 AI 官方入口</span></span>
          <span class="xb-pill" :class="entryOn ? 'is-ok' : 'is-warn'">
            <i class="dot" />{{ entryOn ? '已开启' : '已隐藏' }}
          </span>
        </div>
        <ToggleSwitch
          :model-value="entryOn"
          label="在桌面端显示小伴 AI 入口"
          :disabled="busy || !config"
          @update:model-value="setEntry"
        />
      </div>
    </div>

    <div v-if="loading && !config" class="xb-group xb-empty-box">正在加载…</div>
    <div v-else-if="!filteredProviders.length" class="xb-group xb-empty-box">
      <Icon name="sparkles" :size="26" />
      <strong>暂无服务商</strong>
      <button type="button" class="xb-btn-soft is-sm" :disabled="busy" @click="openEditor()">新增供应商</button>
    </div>

    <div v-else class="xb-group">
      <div
        v-for="p in filteredProviders"
        :key="p.id"
        class="xb-row"
        :class="{ 'is-off': !p.enabled }"
        :title="`${p.description || p.provider} · ${p.endPoint}`"
      >
        <div style="flex: 1; min-width: 200px">
          <div class="xb-row-title">
            <span>{{ p.provider }}</span>
            <span class="xb-pill" :class="typeCls[p.type]">{{ aiTypeLabel(p.type) }}</span>
            <span v-if="p.defaultModel" class="xb-mono def-model">{{ p.defaultModel }}</span>
          </div>
          <div class="xb-row-meta">
            <span>{{ p.protocol }}</span>
            <span>·</span>
            <span class="xb-num">{{ enabledModels(p) }}/{{ p.models.length }} 模型启用</span>
            <span>·</span>
            <span>{{ credential(p) }}</span>
          </div>
        </div>

        <div class="xb-row-actions">
          <ToggleSwitch
            :model-value="p.enabled"
            :label="`${p.enabled ? '停用' : '启用'} ${p.provider}`"
            :disabled="busy"
            @update:model-value="toggle(p)"
          />
          <button type="button" class="xb-btn-ghost is-sm" :disabled="busy" @click="openEditor(p)">配置</button>
          <AdminMoreMenu :items="menuFor(p)" :disabled="busy" />
        </div>
      </div>
    </div>

    <AiProviderDialog
      v-if="editing"
      :key="editing.originalId ?? '__new'"
      :provider="editing.provider"
      :is-new="editing.originalId === null"
      :busy="busy"
      :error="dialogError"
      @save="saveProvider"
      @close="editing = null"
    />
  </div>
</template>

<style scoped>
.is-off { opacity: 0.6; }
.def-model {
  font-size: 12px;
  font-weight: 400;
  color: var(--ui-text-tertiary);
}
</style>
