<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { getAiCatalogue, saveAiCatalogue } from '@/admin/api'
import { aiTypeLabel, newAiProvider, type AiCatalogue, type AiProvider } from '@/admin/ai-types'
import AiProviderDialog from '@/components/admin/AiProviderDialog.vue'
import ConfirmButton from '@/components/admin/ConfirmButton.vue'
import ToggleSwitch from '@/components/admin/ToggleSwitch.vue'
import Icon from '@/components/Icon.vue'
import { toast } from '@/composables/toast'

const clone = <T,>(value: T): T => JSON.parse(JSON.stringify(value))
const message = (e: unknown) => (e instanceof Error ? e.message : String(e))

const config = ref<AiCatalogue | null>(null)
const loading = ref(true)
const busy = ref(false)
const error = ref('')

// The provider being edited in the dialog. `originalId` is null for a new one.
const editing = ref<{ provider: AiProvider; originalId: string | null } | null>(null)
const dialogError = ref('')

const entryOn = computed(() => config.value?.xiaobanAi?.enabled ?? true)
const typeTone = { third: '', free: 'ok', paid: 'warn' } as const

async function load() {
  loading.value = true
  try { config.value = await getAiCatalogue(); error.value = '' }
  catch (e) { error.value = message(e) }
  finally { loading.value = false }
}
onMounted(load)

/** Saves the whole catalogue. Returns the reason when it was refused, so each caller shows it where the user is looking. */
async function publish(next: AiCatalogue): Promise<string> {
  busy.value = true
  try { config.value = await saveAiCatalogue(next); return '' }
  catch (e) { return message(e) }
  finally { busy.value = false }
}

/** Page-level changes (a switch, a delete) report to the banner at the top of the page. */
async function change(next: AiCatalogue, done: string) {
  const refused = await publish(next)
  error.value = refused
  if (!refused) toast(done)
}

function openEditor(provider?: AiProvider) {
  dialogError.value = ''
  editing.value = provider ? { provider: clone(provider), originalId: provider.id } : { provider: newAiProvider(), originalId: null }
}

async function saveProvider(provider: AiProvider) {
  if (!config.value || !editing.value) return
  const next = clone(config.value)
  const { originalId } = editing.value
  if (next.providers.some(v => v.id === provider.id && v.id !== originalId)) {
    dialogError.value = `ID「${provider.id}」已经被另一个供应商用了`
    return
  }
  const at = next.providers.findIndex(v => v.id === originalId)
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
  const target = next.providers.find(v => v.id === provider.id)
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
  next.providers = next.providers.filter(v => v.id !== provider.id)
  void change(next, `已删除 ${provider.provider}`)
}

const enabledModels = (p: AiProvider) => p.models.filter(m => m.enabled).length
const credential = (p: AiProvider) =>
  p.type === 'third' ? '用户自备' : p.type === 'paid' ? `${p.userAccess.length} 人已授权` : p.hasKey ? '已配置' : '免密网关'
</script>

<template>
  <div class="a-page">
    <header class="a-head">
      <h1 class="display">模型供应商</h1>
      <p>管理桌面端“设置 → AI 模型”里的服务商卡片。保存后，客户端在下次刷新时生效。</p>
    </header>

    <p v-if="error" class="a-error banner" role="alert">{{ error }} <button type="button" class="a-link" :disabled="busy || loading" @click="load">重新加载</button></p>

    <section class="entry" :class="{ off: !entryOn }" aria-labelledby="entry-title">
      <span class="entry-icon" aria-hidden="true"><Icon name="sparkles" :size="22" /></span>
      <div class="entry-main">
        <h2 id="entry-title">“小伴 AI”入口 <span class="a-tag" :class="entryOn ? 'ok' : 'warn'">{{ entryOn ? '已开启' : '已关闭' }}</span></h2>
        <p>桌面端卡片列表里排第一的官方服务（账号登录 + 额度，由 AI 网关提供），不在下面的列表里。关闭后，桌面端下次刷新时会隐藏它，正在用它聊天的用户会被提示改选其他服务商。</p>
      </div>
      <ToggleSwitch :model-value="entryOn" label="在桌面端显示“小伴 AI”入口" :disabled="busy || !config" @update:model-value="setEntry" />
    </section>

    <div class="bar">
      <h2 class="bar-title">服务商 <span v-if="config" class="n">{{ config.providers.length }}</span></h2>
      <div class="bar-actions">
        <span v-if="config" class="a-small">修订 {{ config.revision }}</span>
        <button type="button" class="a-btn small" :disabled="busy || loading" @click="load">刷新</button>
        <button type="button" class="a-btn small primary" :disabled="busy || !config" @click="openEditor()"><Icon name="plus" :size="16" />新增供应商</button>
      </div>
    </div>

    <p v-if="loading && !config" class="state" role="status">正在加载…</p>
    <div v-else-if="config && !config.providers.length" class="state">
      <p>还没有服务商。桌面端会只显示“小伴 AI”入口和用户自己添加的服务商。</p>
      <button type="button" class="a-btn" :disabled="busy" @click="openEditor()"><Icon name="plus" :size="16" />新增供应商</button>
    </div>

    <ul v-if="config?.providers.length" class="grid">
      <li v-for="p in config.providers" :key="p.id" class="card" :class="{ off: !p.enabled }">
        <div class="top">
          <div class="titles">
            <h3>{{ p.provider }}</h3>
            <span class="id">{{ p.id }}</span>
          </div>
          <ToggleSwitch :model-value="p.enabled" :label="`${p.enabled ? '停用' : '启用'} ${p.provider}`" :disabled="busy" @update:model-value="toggle(p)" />
        </div>
        <div class="tags">
          <span class="a-tag" :class="typeTone[p.type]">{{ aiTypeLabel(p.type) }}</span>
          <span class="a-tag">{{ p.enabled ? '已启用' : '已停用' }}</span>
        </div>
        <p class="desc">{{ p.description || '没有描述' }}</p>
        <dl class="facts">
          <div><dt>模型</dt><dd>{{ enabledModels(p) }}<small> / {{ p.models.length }}</small></dd></div>
          <div><dt>推荐</dt><dd class="mono" :title="p.defaultModel">{{ p.defaultModel || '未设置' }}</dd></div>
          <div><dt>凭证</dt><dd>{{ credential(p) }}</dd></div>
        </dl>
        <div class="actions">
          <button type="button" class="a-btn small" :disabled="busy" @click="openEditor(p)">编辑</button>
          <ConfirmButton label="删除" confirm-label="再点一次删除" :busy="busy" @confirm="remove(p)" />
        </div>
      </li>
    </ul>

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
.banner {
  margin-top: 16px;
}

.entry {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 16px;
  margin-top: 24px;
  padding: 18px 20px;
  border: 1px solid var(--line-strong);
  border-radius: 16px;
  background: linear-gradient(135deg, color-mix(in srgb, var(--heart) 11%, var(--surface)), var(--surface) 70%);
  transition: opacity 0.2s;
}

.entry.off {
  background: var(--surface);
}

.entry.off .entry-icon {
  color: var(--text-3);
}

.entry-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  border-radius: 12px;
  background: color-mix(in srgb, var(--heart) 16%, transparent);
  color: var(--heart);
}

.entry.off .entry-icon {
  background: var(--surface-2);
}

.entry-main h2 {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px 10px;
  font-size: 17px;
  font-weight: 600;
}

.entry-main p {
  margin-top: 4px;
  max-width: 62ch;
  color: var(--text-2);
  font-size: 13px;
  line-height: 1.6;
}

.bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 10px 16px;
  margin: 32px 0 14px;
}

.bar-title {
  font-size: 22px;
}

.n {
  margin-left: 4px;
  color: var(--text-3);
  font-size: 14px;
  font-weight: 400;
}

.bar-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.state {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 14px;
  padding: 28px 0;
  color: var(--text-3);
  border-top: 1px solid var(--line);
}

.grid {
  list-style: none;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 14px;
}

.card {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
  padding: 16px 18px 14px;
  border: 1px solid var(--line);
  border-radius: 16px;
  background: var(--surface);
  transition:
    border-color 0.2s,
    transform 0.2s var(--ease);
}

.card:hover {
  border-color: var(--line-strong);
}

.card.off .titles,
.card.off .desc,
.card.off .facts {
  opacity: 0.55;
}

.top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.titles {
  min-width: 0;
}

.titles h3 {
  overflow: hidden;
  font-size: 16px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.id {
  display: block;
  margin-top: 2px;
  overflow: hidden;
  color: var(--text-3);
  font-family: var(--mono);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.desc {
  display: -webkit-box;
  min-height: 3.2em;
  overflow: hidden;
  color: var(--text-2);
  font-size: 13px;
  line-height: 1.6;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  line-clamp: 2;
}

.facts {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  gap: 14px;
  padding: 10px 0;
  border-top: 1px solid var(--line);
  border-bottom: 1px solid var(--line);
}

.facts div {
  display: grid;
  gap: 2px;
  min-width: 0;
}

.facts dt {
  color: var(--text-3);
  font-size: 12px;
}

.facts dd {
  overflow: hidden;
  font-size: 13px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.facts dd small {
  color: var(--text-3);
}

.mono {
  font-family: var(--mono);
  font-size: 12px;
}

.actions {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: auto;
}

@media (max-width: 640px) {
  .entry {
    grid-template-columns: auto minmax(0, 1fr);
  }

  .entry > :last-child {
    grid-column: 2;
    justify-self: start;
  }

  .grid {
    grid-template-columns: 1fr;
  }
}
</style>
