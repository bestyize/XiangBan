<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { getAiCatalogue, saveAiCatalogue } from '@/admin/api'
import type { AiCatalogue, AiProvider } from '@/admin/ai-types'
const clone = <T,>(value: T): T => JSON.parse(JSON.stringify(value));
const config = ref<AiCatalogue | null>(null)
const draft = ref<AiProvider | null>(null)
const originalId = ref<string | null>(null)
const error = ref('')
const busy = ref(false)
const protocols = ['OpenAI Responses', 'OpenAI Chat Completions', 'Anthropic Messages']
const labels = { third: '第三方 · 用户填写 Key', free: '小伴免费', paid: '小伴付费' }
async function load() {
  busy.value = true
  try { config.value = await getAiCatalogue(); error.value = '' }
  catch (e) { error.value = String(e instanceof Error ? e.message : e) }
  finally { busy.value = false }
}
onMounted(load)
function edit(p?: AiProvider) {
  originalId.value = p?.id ?? null
  draft.value = p ? clone(p) : {
    id: '', provider: '', description: '', endPoint: '', protocol: 'OpenAI Responses', type: 'third',
    enabled: false, defaultModel: '', models: [], endpoints: {}, consoleUrl: '', consoleText: '', userAccess: [],
  }
}
async function publish(next: AiCatalogue) {
  busy.value = true
  try { config.value = await saveAiCatalogue(next); draft.value = null; error.value = '' }
  catch (e) { error.value = String(e instanceof Error ? e.message : e) }
  finally { busy.value = false }
}
async function save() {
  if (!config.value || !draft.value) return
  const next = clone(config.value), p = clone(draft.value)
  if (!p.id || !p.provider.trim()) { error.value = '请填写供应商 ID 和名称'; return }
  if (next.providers.some(v => v.id === p.id && v.id !== originalId.value)) { error.value = '供应商 ID 已存在'; return }
  const i = next.providers.findIndex(v => v.id === originalId.value)
  if (i < 0) next.providers.push(p); else next.providers[i] = p
  await publish(next)
}
async function toggle(p: AiProvider) {
  if (!config.value) return
  const next = clone(config.value)
  next.providers.find(v => v.id === p.id)!.enabled = !p.enabled
  await publish(next)
}
async function remove(p: AiProvider) {
  if (!config.value || !window.confirm('删除 ' + p.provider + '？客户端下次刷新后将无法再选择。')) return
  await publish({ ...clone(config.value), providers: config.value.providers.filter(v => v.id !== p.id) })
}
</script>
<template>
  <div class="a-page">
    <header class="a-head"><h1 class="display">模型供应商</h1><p>管理桌面端的第三方预设、免费和付费服务。保存后在客户端下次刷新时生效。</p></header>
    <p v-if="error" class="a-error" role="alert">{{ error }} <button type="button" class="a-link" :disabled="busy" @click="load">重新加载</button></p>
    <div class="toolbar"><button type="button" :disabled="busy" @click="edit()">新增供应商</button><button type="button" :disabled="busy" @click="load">刷新</button><span>修订 {{ config?.revision ?? '—' }}</span></div>
    <section v-for="p in config?.providers" :key="p.id" class="a-section">
      <div class="row"><div><h2>{{ p.provider }}</h2><p>{{ labels[p.type] }} · {{ p.enabled ? '已启用' : '已停用' }} · {{ p.models.length }} 个模型</p><p>{{ p.description }}</p><small>推荐模型：{{ p.defaultModel || '未设置' }}</small></div>
      <div class="toolbar"><button type="button" :disabled="busy" @click="edit(p)">编辑</button><button type="button" :disabled="busy" @click="toggle(p)">{{ p.enabled ? '停用' : '启用' }}</button><button type="button" :disabled="busy" @click="remove(p)">删除</button></div></div>
    </section>
    <form v-if="draft" class="editor a-section" @submit.prevent="save">
      <h2>{{ originalId ? '编辑供应商' : '新增供应商' }}</h2>
      <div class="fields">
        <label>ID<input v-model="draft.id" :disabled="originalId !== null" maxlength="80" required /></label>
        <label>名称<input v-model="draft.provider" required maxlength="100" /></label>
        <label>类型<select v-model="draft.type"><option value="third">第三方 third</option><option value="free">免费 free</option><option value="paid">付费 paid</option></select></label>
        <label>协议<select v-model="draft.protocol"><option v-for="p in protocols" :key="p">{{ p }}</option></select></label>
        <label class="wide">描述<textarea v-model="draft.description" maxlength="1000" /></label>
        <label class="wide">API 基础地址<input v-model="draft.endPoint" placeholder="https://gateway.example.com/v1" /></label>
        <label><input v-model="draft.enabled" type="checkbox" /> 启用供应商</label>
        <label>默认模型<select v-model="draft.defaultModel"><option value="">请选择</option><option v-for="m in draft.models.filter(m => m.enabled)" :key="m.id" :value="m.id">{{ m.name || m.id }}</option></select></label>
        <template v-if="draft.type === 'free'">
          <label class="wide">专用受限调用凭证<input v-model="draft.apikey" type="password" autocomplete="new-password" :placeholder="draft.hasKey ? '已配置，留空保持原值' : '填写用于免费服务的专用受限凭证'" /></label>
          <label><input v-model="draft.clearKey" type="checkbox" /> 清除已保存凭证（需停用供应商）</label>
        </template>
        <template v-if="draft.type === 'third'">
          <label class="wide">获取 Key 的页面<input v-model="draft.consoleUrl" placeholder="https://…" /></label>
          <label class="wide">链接文字<input v-model="draft.consoleText" /></label>
          <details class="wide"><summary>不同协议的基础地址</summary><label v-for="p in protocols" :key="p">{{ p }}<input :value="draft.endpoints[p] ?? ''" @input="($event.target as HTMLInputElement).value.trim() ? draft.endpoints[p] = ($event.target as HTMLInputElement).value.trim() : delete draft.endpoints[p]" /></label></details>
        </template>
      </div>
      <h3>模型列表</h3>
      <div v-for="(m, i) in draft.models" :key="i" class="model-row">
        <input v-model="m.id" placeholder="模型调用 ID" required /><input v-model="m.name" placeholder="展示名称" required />
        <input v-model="m.description" placeholder="描述（可选）" />
        <label><input v-model="m.enabled" type="checkbox" />启用</label><button type="button" @click="draft.models.splice(i, 1)">删除</button>
      </div>
      <button type="button" @click="draft.models.push({ id: '', name: '', description: '', enabled: true })">添加模型</button>
      <section v-if="draft.type === 'paid'"><h3>用户授权</h3><p>按用户 UID 下发该用户的网关令牌；计费和额度由网关校验，不提供共享上游 Key。</p>
        <div v-for="(a, i) in draft.userAccess" :key="i" class="model-row"><input v-model="a.uid" placeholder="用户 UID" required /><input v-model="a.apikey" type="password" autocomplete="new-password" :placeholder="a.hasKey ? '留空保持令牌' : '该用户的受限网关令牌'" /><button type="button" @click="draft.userAccess.splice(i, 1)">撤销授权</button></div>
        <button type="button" @click="draft.userAccess.push({ uid: '', apikey: '' })">添加用户授权</button>
      </section>
      <div class="toolbar"><button type="submit" :disabled="busy">{{ busy ? '保存中…' : '保存并生效' }}</button><button type="button" :disabled="busy" @click="draft = null">取消</button></div>
    </form>
  </div>
</template>
<style scoped>
.toolbar,.row{display:flex;align-items:center;gap:12px;flex-wrap:wrap}.toolbar{margin:16px 0}.row{justify-content:space-between}.fields{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:16px}.wide{grid-column:1/-1}label{display:grid;gap:6px}input:not([type=checkbox]),select,textarea{width:100%;padding:9px;border:1px solid var(--line);border-radius:8px;background:var(--surface);color:inherit}button{padding:8px 14px;border:1px solid var(--line);border-radius:8px;background:var(--surface);color:inherit;cursor:pointer}button:disabled{opacity:.5;cursor:default}.model-row{display:flex;gap:8px;align-items:center;margin:10px 0;flex-wrap:wrap}.model-row input:not([type=checkbox]){flex:1;min-width:150px}textarea{min-height:70px}section h3{margin-top:24px}@media(max-width:600px){.fields{grid-template-columns:1fr}}
</style>
