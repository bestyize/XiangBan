<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import Icon from '@/components/Icon.vue'
import ToggleSwitch from '@/components/admin/ToggleSwitch.vue'
import { AI_PROTOCOLS, AI_TYPES, type AiProvider } from '@/admin/ai-types'

const props = defineProps<{ provider: AiProvider; isNew: boolean; busy: boolean; error: string }>()
const emit = defineEmits<{ save: [provider: AiProvider]; close: [] }>()

const clone = <T,>(value: T): T => JSON.parse(JSON.stringify(value))
const draft = ref<AiProvider>(clone(props.provider))
const initial = JSON.stringify(props.provider)
const dirty = computed(() => JSON.stringify(draft.value) !== initial)

const dialog = ref<HTMLDialogElement>()
const localError = ref('')
const discarding = ref(false)
let bodyOverflow = ''

onMounted(() => {
  bodyOverflow = document.body.style.overflow
  document.body.style.overflow = 'hidden'
  dialog.value?.showModal()
})
onBeforeUnmount(() => { document.body.style.overflow = bodyOverflow })

const typeHint = computed(() => AI_TYPES.find(t => t.value === draft.value.type)?.hint ?? '')
const enabledModels = computed(() => draft.value.models.filter(m => m.enabled && m.id.trim()))
// The server's error describes the draft that was sent. Once something is edited it no longer applies, so it goes away
// instead of staying on screen while the user fixes the very thing it complains about. This compares the draft with
// what was sent, not the error text, so the same refusal shows again when the same kind of mistake is sent again.
const sentFor = ref('')
watch(draft, () => { localError.value = '' }, { deep: true })
const shownError = computed(() => {
  if (localError.value) return localError.value
  if (props.busy || !props.error) return ''
  return JSON.stringify(draft.value) === sentFor.value ? props.error : ''
})
const defaultMissing = computed(() => draft.value.enabled && !enabledModels.value.some(m => m.id.trim() === draft.value.defaultModel))

// The first enabled model becomes the recommended one, so a new provider does not have to be told twice.
watch(enabledModels, models => {
  if (!draft.value.defaultModel && models.length) draft.value.defaultModel = models[0].id.trim()
})

const isHttps = (value: string) => {
  try {
    const url = new URL(value.trim())
    return url.protocol === 'https:' && !url.username && !url.password && !url.search && !url.hash
  } catch { return false }
}
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i

/** The same rules the server applies, so most mistakes are caught before the round trip. */
function check(): string {
  const d = draft.value
  const id = d.id.trim()
  if (!/^[A-Za-z0-9-]{1,80}$/.test(id) || id === 'custom') return 'ID 只能由字母、数字和连字符组成（最长 80 位），且不能是 custom'
  if (!d.provider.trim()) return '请填写供应商名称'
  if (d.models.some(m => !m.id.trim() || !m.name.trim())) return '每个模型都要填写调用 ID 和展示名称'
  const ids = d.models.map(m => m.id.trim())
  if (new Set(ids).size !== ids.length) return '模型调用 ID 有重复'
  if (d.endPoint.trim() && !isHttps(d.endPoint)) return 'API 基础地址必须是 HTTPS 地址，且不带账号、参数或锚点'
  if (d.enabled) {
    if (!isHttps(d.endPoint)) return '启用的供应商必须填写有效的 HTTPS 基础地址'
    if (defaultMissing.value) return '启用的供应商需要选一个已启用的默认模型'
  }
  if (d.consoleUrl.trim() && !isHttps(d.consoleUrl)) return '获取 Key 的页面必须使用 HTTPS'
  if (d.type === 'paid') {
    const users = d.userAccess.filter(a => a.uid.trim() || a.apikey)
    if (users.some(a => !UUID.test(a.uid.trim()))) return '用户 UID 格式不对（应是 36 位带连字符的 UUID）'
    if (new Set(users.map(a => a.uid.trim().toLowerCase())).size !== users.length) return '同一个用户不能授权两次'
    if (users.some(a => !a.hasKey && !a.apikey)) return '请填写每位被授权用户的令牌'
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
  d.models = d.models.map(m => ({ ...m, id: m.id.trim(), name: m.name.trim() }))
  d.userAccess = d.userAccess.filter(a => a.uid.trim() || a.apikey).map(a => ({ ...a, uid: a.uid.trim() }))
  emit('save', d)
}

/** Closing with unsaved changes asks first, in the footer, so nothing typed is lost to a stray click or Esc. */
function requestClose() {
  if (props.busy) return
  if (dirty.value && !discarding.value) { discarding.value = true; return }
  emit('close')
}
function onBackdrop(event: MouseEvent) {
  if (event.target === dialog.value) requestClose()
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
</script>

<template>
  <dialog ref="dialog" class="dlg" aria-labelledby="ap-title" @cancel.prevent="requestClose" @click="onBackdrop">
    <form class="panel" novalidate @submit.prevent="submit">
      <header class="head">
        <div>
          <h2 id="ap-title" class="display">{{ isNew ? '新增供应商' : '编辑供应商' }}</h2>
          <p class="sub">{{ isNew ? '填好后点“保存并生效”，客户端下次刷新就能看到。' : draft.provider || draft.id }}</p>
        </div>
        <button type="button" class="close" aria-label="关闭" :disabled="busy" @click="requestClose"><Icon name="close" :size="20" /></button>
      </header>

      <div class="body">
        <section class="sec" aria-labelledby="ap-basic">
          <h3 id="ap-basic">基本信息</h3>
          <div class="grid2">
            <label class="a-field"><span>名称</span><input v-model="draft.provider" class="a-input" maxlength="100" placeholder="例如：DeepSeek" /></label>
            <label class="a-field"><span>ID</span><input v-model="draft.id" class="a-input mono" maxlength="80" :disabled="!isNew" placeholder="deepseek" /><small class="a-hint">{{ isNew ? '字母、数字、连字符；创建后不能修改。' : '创建后不能修改。' }}</small></label>
          </div>
          <div class="a-field">
            <span>类型</span>
            <div class="a-segment seg" role="group" aria-label="供应商类型">
              <button v-for="t in AI_TYPES" :key="t.value" type="button" :aria-pressed="draft.type === t.value" @click="draft.type = t.value">{{ t.label }}</button>
            </div>
            <small class="a-hint">{{ typeHint }}</small>
          </div>
          <label class="a-field"><span>描述（显示在客户端卡片上）</span><textarea v-model="draft.description" class="a-input area" maxlength="1000" placeholder="一两句话说明这个服务适合做什么" /></label>
          <div class="switch-row">
            <div><b>启用这个供应商</b><small class="a-hint">关闭后客户端下次刷新就看不到它。</small></div>
            <ToggleSwitch v-model="draft.enabled" label="启用这个供应商" />
          </div>
        </section>

        <section class="sec" aria-labelledby="ap-link">
          <h3 id="ap-link">连接</h3>
          <div class="grid2">
            <label class="a-field wide"><span>API 基础地址</span><input v-model="draft.endPoint" class="a-input mono" placeholder="https://api.example.com/v1" /></label>
            <label class="a-field wide"><span>接口协议</span>
              <select v-model="draft.protocol" class="a-select"><option v-for="p in AI_PROTOCOLS" :key="p" :value="p">{{ p }}</option></select>
            </label>
          </div>
          <template v-if="draft.type === 'free'">
            <label class="a-field"><span>调用凭证</span>
              <input v-model="draft.apikey" class="a-input mono" type="password" autocomplete="new-password" :placeholder="draft.hasKey ? '已配置，留空保持原值' : '公共免密网关留空；有自备的受限 Key 就填在这里'" />
            </label>
            <label v-if="draft.hasKey" class="check"><input v-model="draft.clearKey" type="checkbox" /> 清除已保存的凭证（需要先停用这个供应商）</label>
          </template>
          <template v-if="draft.type === 'third'">
            <div class="grid2">
              <label class="a-field"><span>获取 Key 的页面</span><input v-model="draft.consoleUrl" class="a-input mono" placeholder="https://…" /></label>
              <label class="a-field"><span>链接文字</span><input v-model="draft.consoleText" class="a-input" maxlength="40" placeholder="前往获取 API Key" /></label>
            </div>
            <details class="more">
              <summary>不同协议用不同地址（可选）</summary>
              <label v-for="p in AI_PROTOCOLS" :key="p" class="a-field"><span>{{ p }}</span><input class="a-input mono" :value="draft.endpoints[p] ?? ''" placeholder="留空则用上面的基础地址" @input="setEndpoint(p, $event)" /></label>
            </details>
          </template>
        </section>

        <section class="sec" aria-labelledby="ap-models">
          <div class="sec-head">
            <h3 id="ap-models">模型 <span class="count">{{ enabledModels.length }}/{{ draft.models.length }} 个已启用</span></h3>
            <button type="button" class="a-btn small" @click="addModel"><Icon name="plus" :size="16" />添加模型</button>
          </div>
          <p v-if="!draft.models.length" class="empty">还没有模型。客户端会列出这里已启用的模型供用户选择。</p>
          <ul v-else class="models">
            <li v-for="(m, i) in draft.models" :key="i" class="model">
              <input v-model="m.id" class="a-input mono" placeholder="调用 ID" :aria-label="`第 ${i + 1} 个模型的调用 ID`" />
              <input v-model="m.name" class="a-input" placeholder="展示名称" :aria-label="`第 ${i + 1} 个模型的展示名称`" />
              <input v-model="m.description" class="a-input" placeholder="描述（可选）" :aria-label="`第 ${i + 1} 个模型的描述`" />
              <div class="model-end">
                <span class="on-label" aria-hidden="true">启用</span>
                <ToggleSwitch v-model="m.enabled" :label="`启用模型 ${m.id || i + 1}`" />
                <button type="button" class="icon-btn" :aria-label="`删除模型 ${m.id || i + 1}`" @click="removeModel(i)"><Icon name="trash" :size="18" /></button>
              </div>
            </li>
          </ul>
          <label class="a-field default"><span>推荐（默认）模型</span>
            <select v-model="draft.defaultModel" class="a-select" :disabled="!enabledModels.length">
              <option value="">{{ enabledModels.length ? '请选择' : '先添加并启用一个模型' }}</option>
              <option v-for="m in enabledModels" :key="m.id" :value="m.id.trim()">{{ m.name || m.id }}</option>
            </select>
            <small v-if="defaultMissing" class="a-hint warn">启用的供应商需要一个已启用的默认模型。</small>
            <small v-else class="a-hint">用户选“跟随推荐”时，客户端会随它自动更新。</small>
          </label>
        </section>

        <section v-if="draft.type === 'paid'" class="sec" aria-labelledby="ap-users">
          <div class="sec-head">
            <h3 id="ap-users">用户授权 <span class="count">{{ draft.userAccess.length }} 人</span></h3>
            <button type="button" class="a-btn small" @click="draft.userAccess.push({ uid: '', apikey: '' })"><Icon name="plus" :size="16" />添加用户</button>
          </div>
          <p class="a-hint">按用户 UID 下发该用户自己的网关令牌；计费和额度由网关校验，不提供共享的上游 Key。</p>
          <p v-if="!draft.userAccess.length" class="empty">还没有授权任何用户。</p>
          <ul v-else class="models">
            <li v-for="(a, i) in draft.userAccess" :key="i" class="access">
              <input v-model="a.uid" class="a-input mono" placeholder="用户 UID" :aria-label="`第 ${i + 1} 位用户的 UID`" />
              <input v-model="a.apikey" class="a-input mono" type="password" autocomplete="new-password" :placeholder="a.hasKey ? '留空保持原令牌' : '该用户的受限网关令牌'" :aria-label="`第 ${i + 1} 位用户的令牌`" />
              <button type="button" class="icon-btn" :aria-label="`撤销第 ${i + 1} 位用户的授权`" @click="draft.userAccess.splice(i, 1)"><Icon name="trash" :size="18" /></button>
            </li>
          </ul>
        </section>
      </div>

      <footer class="foot">
        <p v-if="discarding" class="msg" role="status">有还没保存的修改，确定要放弃吗？</p>
        <p v-else-if="shownError" class="a-error msg" role="alert">{{ shownError }}</p>
        <div class="foot-actions">
          <template v-if="discarding">
            <button type="button" class="a-btn" @click="discarding = false">继续编辑</button>
            <button type="button" class="a-btn danger" @click="emit('close')">放弃修改</button>
          </template>
          <template v-else>
            <button type="button" class="a-btn" :disabled="busy" @click="requestClose">取消</button>
            <button type="submit" class="a-btn primary" :disabled="busy">{{ busy ? '保存中…' : '保存并生效' }}</button>
          </template>
        </div>
      </footer>
    </form>
  </dialog>
</template>

<style scoped>
.dlg {
  width: min(780px, calc(100vw - 32px));
  max-height: calc(100dvh - 32px);
  margin: auto;
  padding: 0;
  border: 1px solid var(--line-strong);
  border-radius: 20px;
  background: var(--surface);
  color: var(--text);
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.35);
  overflow: hidden;
}

.dlg::backdrop {
  background: rgba(0, 0, 0, 0.55);
  -webkit-backdrop-filter: blur(4px);
  backdrop-filter: blur(4px);
}

.panel {
  display: flex;
  flex-direction: column;
  max-height: calc(100dvh - 32px);
}

.head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  padding: 20px 24px 16px;
  border-bottom: 1px solid var(--line);
}

.head h2 {
  font-size: 30px;
  line-height: 1.1;
}

.sub {
  margin-top: 4px;
  color: var(--text-3);
  font-size: 13px;
}

.close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: none;
  width: 36px;
  height: 36px;
  border-radius: 50%;
  color: var(--text-2);
}

.close:hover:not(:disabled) {
  background: var(--surface-2);
  color: var(--text);
}

.body {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 18px 24px;
  overflow-y: auto;
  overscroll-behavior: contain;
}

.sec {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 16px 18px 18px;
  border: 1px solid var(--line);
  border-radius: 14px;
  background: color-mix(in srgb, var(--surface-2) 38%, transparent);
}

.sec h3 {
  font-size: 15px;
  font-weight: 600;
}

.sec-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.count {
  margin-left: 6px;
  color: var(--text-3);
  font-size: 12px;
  font-weight: 400;
}

.grid2 {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

/* A field next to a taller one keeps its own height instead of spreading its rows to match. */
.a-field {
  min-width: 0;
  align-content: start;
}

.mono {
  font-family: var(--mono);
  font-size: 13px;
}

.area {
  height: auto;
  min-height: 84px;
  padding: 10px 12px;
  font: inherit;
  line-height: 1.55;
  resize: vertical;
}

/* An ID that can no longer be changed is still a value, not a hint: keep it readable, with a surface that says it is locked. */
.a-input:disabled {
  opacity: 1;
  background: var(--surface-2);
  color: var(--text-2);
  cursor: not-allowed;
}

.seg {
  justify-self: start;
}

.switch-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding-top: 4px;
}

.switch-row > div {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.check {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--text-2);
}

.more {
  display: flex;
  flex-direction: column;
  gap: 12px;
  color: var(--text-2);
  font-size: 13px;
}

.more summary {
  cursor: pointer;
  width: fit-content;
}

.more[open] summary {
  margin-bottom: 4px;
}

.models {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.model {
  display: grid;
  grid-template-columns: minmax(0, 1.1fr) minmax(0, 1fr) minmax(0, 1.4fr) auto;
  align-items: center;
  gap: 8px;
}

.access {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) auto;
  align-items: center;
  gap: 8px;
}

.model-end {
  display: flex;
  align-items: center;
  gap: 8px;
}

.on-label {
  color: var(--text-3);
  font-size: 12px;
}

.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 10px;
  color: var(--text-3);
}

.icon-btn:hover {
  background: color-mix(in srgb, var(--danger) 14%, transparent);
  color: var(--danger);
}

.empty {
  padding: 14px 0;
  color: var(--text-3);
  font-size: 13px;
}

.default {
  max-width: 360px;
}

.a-hint.warn {
  color: var(--warn);
}

.foot {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 10px 16px;
  padding: 14px 24px;
  border-top: 1px solid var(--line);
  background: var(--surface);
}

.msg {
  flex: 1 1 260px;
  min-width: 0;
  font-size: 13px;
  line-height: 1.5;
}

.foot-actions {
  display: flex;
  gap: 10px;
  margin-left: auto;
}

@media (max-width: 640px) {
  .grid2 {
    grid-template-columns: 1fr;
  }

  .model {
    grid-template-columns: 1fr;
    padding: 10px;
    border: 1px solid var(--line);
    border-radius: 12px;
  }

  .model-end {
    justify-content: flex-start;
  }

  .model-end .icon-btn {
    margin-left: auto;
  }

  .access {
    grid-template-columns: 1fr auto;
  }

  .access > :first-child,
  .access > :nth-child(2) {
    grid-column: 1 / -1;
  }

  .body,
  .head,
  .foot {
    padding-left: 16px;
    padding-right: 16px;
  }
}
</style>
