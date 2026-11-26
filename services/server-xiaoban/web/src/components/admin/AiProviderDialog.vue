<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import Icon from '@/components/Icon.vue'
import AdminSelect, { type SelectOption } from '@/components/admin/AdminSelect.vue'
import ToggleSwitch from '@/components/admin/ToggleSwitch.vue'
import { AI_PROTOCOLS, AI_TYPES, type AiModel, type AiProvider } from '@/admin/ai-types'

const props = defineProps<{ provider: AiProvider; isNew: boolean; busy: boolean; error: string }>()
const emit = defineEmits<{ save: [provider: AiProvider]; close: [] }>()

const clone = <T,>(value: T): T => JSON.parse(JSON.stringify(value))
const draft = ref<AiProvider>(clone(props.provider))
const initial = JSON.stringify(props.provider)
const dirty = computed(() => JSON.stringify(draft.value) !== initial)

const localError = ref('')
const discarding = ref(false)
const showKey = ref(false)
let bodyOverflow = ''

const EFFORT_LEVELS = ['low', 'medium', 'high', 'xhigh', 'max'] as const

const protocolOptions: SelectOption[] = AI_PROTOCOLS.map((p) => ({
  value: p,
  label: p,
  desc:
    p === 'OpenAI Responses'
      ? 'OpenAI 新一代 Responses 协议'
      : p === 'OpenAI Chat Completions'
        ? '通用 OpenAI Chat 兼容协议'
        : 'Anthropic Claude 原生消息协议',
}))

const effortModeOptions: SelectOption[] = [
  { value: 'auto', label: '自动识别', desc: '由客户端按模型名称自动推断' },
  { value: 'manual', label: '手动声明', desc: '显式勾选模型支持的思考档位' },
  { value: 'off', label: '不支持', desc: '不向用户展示思考等级切换' },
]

function onEsc(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    e.preventDefault()
    requestClose()
  }
}

onMounted(() => {
  bodyOverflow = document.body.style.overflow
  document.body.style.overflow = 'hidden'
  document.addEventListener('keydown', onEsc)
})
onBeforeUnmount(() => {
  document.body.style.overflow = bodyOverflow
  document.removeEventListener('keydown', onEsc)
})

const enabledModels = computed(() => draft.value.models.filter((m) => m.enabled && m.id.trim()))
const defaultModelOptions = computed<SelectOption[]>(() => [
  { value: '', label: enabledModels.value.length ? '请选择推荐模型' : '先启用一个模型' },
  ...enabledModels.value.map((m) => ({ value: m.id.trim(), label: m.name || m.id.trim(), desc: m.id.trim() })),
])

const sentFor = ref('')
watch(
  draft,
  () => {
    localError.value = ''
  },
  { deep: true },
)

const shownError = computed(() => {
  if (localError.value) return localError.value
  if (props.busy || !props.error) return ''
  return JSON.stringify(draft.value) === sentFor.value ? props.error : ''
})

const defaultMissing = computed(
  () => draft.value.enabled && !enabledModels.value.some((m) => m.id.trim() === draft.value.defaultModel),
)

watch(enabledModels, (models) => {
  if (!draft.value.defaultModel && models.length) draft.value.defaultModel = models[0].id.trim()
})

const isHttps = (value: string) => {
  try {
    const url = new URL(value.trim())
    return url.protocol === 'https:' && !url.username && !url.password && !url.search && !url.hash
  } catch {
    return false
  }
}
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i

function check(): string {
  const d = draft.value
  const id = d.id.trim()
  if (!/^[A-Za-z0-9-]{1,80}$/.test(id) || id === 'custom')
    return 'ID 只能由字母、数字和连字符组成（最长 80 位），且不能是 custom'
  if (!d.provider.trim()) return '请填写供应商名称'
  if (d.models.some((m) => !m.id.trim() || !m.name.trim())) return '每个模型都要填写调用 ID 和展示名称'
  const ids = d.models.map((m) => m.id.trim())
  if (new Set(ids).size !== ids.length) return '模型调用 ID 有重复'
  if (d.endPoint.trim() && !isHttps(d.endPoint)) return 'API 基础地址必须是 HTTPS 地址，且不带账号、参数或锚点'
  if (d.enabled) {
    if (!isHttps(d.endPoint)) return '启用的供应商必须填写有效的 HTTPS 基础地址'
    if (defaultMissing.value) return '启用的供应商需要选一个已启用的默认模型'
  }
  if (d.consoleUrl.trim() && !isHttps(d.consoleUrl)) return '获取 Key 的页面必须使用 HTTPS'
  if (d.type === 'paid') {
    const users = d.userAccess.filter((a) => a.uid.trim() || a.apikey)
    if (users.some((a) => !UUID.test(a.uid.trim()))) return '用户 UID 格式不对（应是 36 位带连字符的 UUID）'
    if (new Set(users.map((a) => a.uid.trim().toLowerCase())).size !== users.length) return '同一个用户不能授权两次'
    if (users.some((a) => !a.hasKey && !a.apikey)) return '请填写每位被授权用户的令牌'
  }
  return ''
}

function submit() {
  if (props.busy) return
  localError.value = check()
  if (localError.value) return
  discarding.value = false
  sentFor.value = JSON.stringify(draft.value)
  const d = clone(draft.value)
  d.id = d.id.trim()
  d.provider = d.provider.trim()
  d.endPoint = d.endPoint.trim()
  d.models = d.models.map((m) => ({ ...m, id: m.id.trim(), name: m.name.trim() }))
  d.userAccess = d.userAccess
    .filter((a) => a.uid.trim() || a.apikey)
    .map((a) => ({ ...a, uid: a.uid.trim() }))
  emit('save', d)
}

function requestClose() {
  if (props.busy) return
  if (dirty.value && !discarding.value) {
    discarding.value = true
    return
  }
  emit('close')
}

function setEndpoint(protocol: string, event: Event) {
  const value = (event.target as HTMLInputElement).value.trim()
  if (value) draft.value.endpoints[protocol] = value
  else delete draft.value.endpoints[protocol]
}

function addModel() {
  draft.value.models.push({ id: '', name: '', description: '', enabled: true })
}

function removeModel(index: number) {
  const [gone] = draft.value.models.splice(index, 1)
  if (gone && gone.id.trim() === draft.value.defaultModel) draft.value.defaultModel = ''
}

function effortModeOf(m: AiModel): string {
  return m.reasoningEfforts == null ? 'auto' : m.reasoningEfforts.length ? 'manual' : 'off'
}

function setEffortMode(m: AiModel, mode: string) {
  m.reasoningEfforts = mode === 'auto' ? null : mode === 'off' ? [] : ['low', 'high', 'max']
}

function toggleEffortLevel(m: AiModel, lv: string) {
  if (!m.reasoningEfforts) m.reasoningEfforts = []
  const i = m.reasoningEfforts.indexOf(lv)
  if (i >= 0) {
    if (m.reasoningEfforts.length > 1) m.reasoningEfforts.splice(i, 1)
  } else {
    m.reasoningEfforts.push(lv)
  }
}
</script>

<template>
  <Teleport to="body">
    <div class="xb-backdrop is-center" @click.self="requestClose">
      <form class="xb-dialog" role="dialog" aria-labelledby="ap-title" novalidate @submit.prevent="submit">
        <header class="xb-ov-head">
          <h2 id="ap-title">{{ isNew ? '新增模型供应商' : (draft.provider || draft.id) }}</h2>
          <button type="button" class="xb-icon-btn" aria-label="关闭" :disabled="busy" @click="requestClose">
            <Icon name="close" :size="18" />
          </button>
        </header>

        <div class="xb-ov-body">
          <div class="xb-split-eq">
            <label class="xb-field">
              <span class="xb-field-label">供应商名称</span>
              <input v-model="draft.provider" class="xb-input" maxlength="100" placeholder="DeepSeek" />
            </label>
            <label class="xb-field" :title="isNew ? '字母、数字与连字符，创建后不可修改' : '创建后不可修改'">
              <span class="xb-field-label">唯一 ID</span>
              <input
                v-model="draft.id"
                class="xb-input xb-mono"
                maxlength="80"
                :disabled="!isNew"
                placeholder="deepseek"
              />
            </label>
          </div>

          <div class="xb-split-eq">
            <div class="xb-field">
              <span class="xb-field-label">类型</span>
              <div class="xb-seg" role="group" aria-label="供应商类型" style="align-self: flex-start">
                <button
                  v-for="t in AI_TYPES"
                  :key="t.value"
                  type="button"
                  :class="{ 'is-active': draft.type === t.value }"
                  :title="t.hint"
                  :aria-description="t.hint"
                  @click="draft.type = t.value"
                >
                  {{ t.label }}
                </button>
              </div>
            </div>
            <div class="xb-field">
              <span class="xb-field-label">接口协议</span>
              <AdminSelect v-model="draft.protocol" :options="protocolOptions" full />
            </div>
          </div>

          <label class="xb-field">
            <span class="xb-field-label">API 基础地址</span>
            <input v-model="draft.endPoint" class="xb-input xb-mono" placeholder="https://api.example.com/v1" />
          </label>

          <label class="xb-field">
            <span class="xb-field-label">描述</span>
            <textarea
              v-model="draft.description"
              class="xb-textarea"
              rows="2"
              maxlength="1000"
              placeholder="一两句话说明此服务商特点"
            />
          </label>

          <div class="xb-group">
            <div class="xb-row" title="关闭后桌面端下次刷新将隐藏此服务商">
              <span style="font-weight: 550">启用此供应商</span>
              <ToggleSwitch v-model="draft.enabled" label="启用此供应商" />
            </div>
          </div>

          <template v-if="draft.type === 'free'">
            <label class="xb-field">
              <span class="xb-field-label">调用凭证</span>
              <div class="xb-affix has-r">
                <input
                  v-model="draft.apikey"
                  class="xb-input xb-mono"
                  :type="showKey ? 'text' : 'password'"
                  autocomplete="new-password"
                  :placeholder="draft.hasKey ? '已配置，留空保持原值' : '公共免密网关可留空'"
                />
                <button type="button" class="af-r" aria-label="显示或隐藏凭证" @click="showKey = !showKey">
                  <Icon :name="showKey ? 'eye-off' : 'eye'" :size="14" />
                </button>
              </div>
            </label>
            <div v-if="draft.hasKey" class="xb-group">
              <div class="xb-row" title="清除已保存凭证前需要先停用此供应商">
                <span style="font-size: 13px">清除已保存凭证</span>
                <ToggleSwitch
                  :model-value="Boolean(draft.clearKey)"
                  label="清除已保存凭证"
                  @update:model-value="draft.clearKey = $event"
                />
              </div>
            </div>
          </template>

          <template v-if="draft.type === 'third'">
            <div class="xb-split-eq">
              <label class="xb-field">
                <span class="xb-field-label">获取 Key 页面</span>
                <input v-model="draft.consoleUrl" class="xb-input xb-mono" placeholder="https://…" />
              </label>
              <label class="xb-field">
                <span class="xb-field-label">链接文字</span>
                <input v-model="draft.consoleText" class="xb-input" maxlength="40" placeholder="前往获取 API Key" />
              </label>
            </div>
            <details class="more-ep">
              <summary>按协议覆盖独立地址（可选）</summary>
              <div class="more-ep-list">
                <label v-for="p in AI_PROTOCOLS" :key="p" class="xb-field">
                  <span class="xb-field-label">{{ p }}</span>
                  <input
                    class="xb-input xb-mono"
                    :value="draft.endpoints[p] ?? ''"
                    placeholder="留空则使用基础地址"
                    @input="setEndpoint(p, $event)"
                  />
                </label>
              </div>
            </details>
          </template>

          <!-- Compact Model List -->
          <div class="xb-field">
            <div class="xb-field-label">
              <span>模型列表（{{ enabledModels.length }}/{{ draft.models.length }}）</span>
              <button type="button" class="xb-btn-ghost is-sm" @click="addModel">
                <Icon name="plus" :size="13" /><span>添加模型</span>
              </button>
            </div>

            <div v-if="!draft.models.length" class="empty-inline">暂无模型，点击右侧添加</div>

            <div v-else class="model-list">
              <div v-for="(m, i) in draft.models" :key="i" class="model-item">
                <div class="model-row">
                  <input v-model="m.id" class="xb-input xb-mono" placeholder="调用 ID" />
                  <input v-model="m.name" class="xb-input" placeholder="展示名称" />
                  <AdminSelect
                    :model-value="effortModeOf(m)"
                    :options="effortModeOptions"
                    title="思考等级策略"
                    sm
                    full
                    @update:model-value="setEffortMode(m, $event)"
                  />
                  <div class="model-end">
                    <ToggleSwitch v-model="m.enabled" :label="`启用 ${m.id || i + 1}`" />
                    <button
                      type="button"
                      class="xb-icon-btn is-sm is-danger"
                      :aria-label="`删除 ${m.id || i + 1}`"
                      @click="removeModel(i)"
                    >
                      <Icon name="trash" :size="14" />
                    </button>
                  </div>
                </div>
                <div class="model-row-sub">
                  <input v-model="m.description" class="xb-input is-sub" placeholder="描述（可选，悬停显示）" />
                  <div v-if="effortModeOf(m) === 'manual'" class="effort-bar">
                    <button
                      v-for="lv in EFFORT_LEVELS"
                      :key="lv"
                      type="button"
                      class="effort-chip"
                      :class="{ 'is-on': m.reasoningEfforts?.includes(lv) }"
                      @click="toggleEffortLevel(m, lv)"
                    >
                      {{ lv }}
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div class="xb-field" style="max-width: 320px">
            <span class="xb-field-label">推荐默认模型</span>
            <AdminSelect
              v-model="draft.defaultModel"
              :options="defaultModelOptions"
              :disabled="!enabledModels.length"
              full
            />
          </div>

          <template v-if="draft.type === 'paid'">
            <div class="xb-field">
              <div class="xb-field-label">
                <span>用户授权（{{ draft.userAccess.length }} 人）</span>
                <button
                  type="button"
                  class="xb-btn-ghost is-sm"
                  @click="draft.userAccess.push({ uid: '', apikey: '' })"
                >
                  <Icon name="plus" :size="13" /><span>添加用户</span>
                </button>
              </div>
              <div v-if="!draft.userAccess.length" class="empty-inline">暂无授权用户</div>
              <div v-else class="model-list">
                <div v-for="(a, i) in draft.userAccess" :key="i" class="access-row">
                  <input v-model="a.uid" class="xb-input xb-mono" placeholder="用户 UID (UUID)" />
                  <input
                    v-model="a.apikey"
                    class="xb-input xb-mono"
                    type="password"
                    autocomplete="new-password"
                    :placeholder="a.hasKey ? '留空保持原令牌' : '受限网关令牌'"
                  />
                  <button
                    type="button"
                    class="xb-icon-btn is-sm is-danger"
                    aria-label="撤销授权"
                    @click="draft.userAccess.splice(i, 1)"
                  >
                    <Icon name="trash" :size="14" />
                  </button>
                </div>
              </div>
            </div>
          </template>
        </div>

        <footer class="xb-ov-foot" style="justify-content: space-between">
          <span v-if="discarding" class="foot-msg">有未保存修改，确定放弃？</span>
          <span v-else-if="shownError" class="xb-error" role="alert">{{ shownError }}</span>
          <span v-else />
          <div class="foot-btns">
            <template v-if="discarding">
              <button type="button" class="xb-btn-ghost" @click="discarding = false">继续编辑</button>
              <button type="button" class="xb-btn-soft is-danger" @click="emit('close')">放弃修改</button>
            </template>
            <template v-else>
              <button type="button" class="xb-btn-ghost" :disabled="busy" @click="requestClose">取消</button>
              <button type="submit" class="xb-btn-primary" :disabled="busy">
                {{ busy ? '保存中…' : '保存并生效' }}
              </button>
            </template>
          </div>
        </footer>
      </form>
    </div>
  </Teleport>
</template>

<style scoped>
.more-ep {
  font-size: 12.5px;
  color: var(--ui-text-secondary);
}
.more-ep summary {
  cursor: pointer;
  width: fit-content;
}
.more-ep-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-top: 10px;
  padding: 12px;
  border-radius: 10px;
  background: var(--ui-surface-sunken);
}
.empty-inline {
  padding: 14px;
  border-radius: 10px;
  background: var(--ui-surface-sunken);
  text-align: center;
  color: var(--ui-text-tertiary);
  font-size: 12.5px;
}
.model-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.model-item {
  padding: 10px 12px;
  border-radius: 10px;
  background: var(--ui-surface-sunken);
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.model-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) 132px auto;
  gap: 8px;
  align-items: center;
}
.model-end {
  display: flex;
  align-items: center;
  gap: 4px;
}
.model-row-sub {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.is-sub {
  flex: 1;
  min-width: 180px;
  height: 30px;
  font-size: 12.5px;
}
.effort-bar {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-wrap: wrap;
}
.effort-chip {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: 24px;
  padding: 0 8px;
  border-radius: 6px;
  font-size: 11.5px;
  font-family: var(--ui-mono);
  line-height: 1;
  background: var(--ui-control);
  color: var(--ui-text-secondary);
}
.effort-chip.is-on {
  background: var(--ui-soft);
  color: var(--ui-ink);
  font-weight: 600;
}
.access-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) auto;
  gap: 8px;
  align-items: center;
  padding: 8px 10px;
  border-radius: 10px;
  background: var(--ui-surface-sunken);
}
.foot-msg {
  font-size: 13px;
  color: var(--ui-warning-text);
}
.foot-btns {
  display: flex;
  align-items: center;
  gap: 8px;
}
@media (max-width: 640px) {
  .model-row {
    grid-template-columns: 1fr;
  }
  .access-row {
    grid-template-columns: 1fr;
  }
}
</style>
