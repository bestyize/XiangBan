<script setup lang="ts">
import { computed, onActivated, onBeforeUnmount, onMounted, ref } from 'vue';
import { NModal } from 'naive-ui';
import { Plug } from 'lucide-vue-next';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import ConnectorTokenForm from './ConnectorTokenForm.vue';
import { connectorAuth, connectorStatus, type ConnectorAuthorization, type InstalledConnector } from './connector-plugins.ts';

const props = defineProps<{ store: ControlStore; query?: string }>();
const items = ref<InstalledConnector[]>([]);
const selected = ref<InstalledConnector | null>(null);
const loading = ref(false), busy = ref(false), error = ref(''), detailError = ref(''), unbinding = ref(false);
const authorization = ref<{ flowId: string; authorizationUrl: string } | null>(null);
let alive = true;
let authTimer: ReturnType<typeof setTimeout> | undefined;
let polling = false;
const visible = computed(() => items.value.filter(c => `${c.name} ${c.nameEn ?? ''} ${c.description}`.toLowerCase().includes((props.query ?? '').trim().toLowerCase())));
async function refresh() {
  if (!props.store.desktop || loading.value) return;
  loading.value = true; error.value = '';
  try {
    const result = await invoke<InstalledConnector[]>('ai_connector_installed');
    if (!alive) return;
    items.value = result;
    if (selected.value) selected.value = result.find(c => c.id === selected.value!.id) ?? null;
  } catch (e) { if (alive) error.value = `无法读取已安装连接器：${String(e)}`; }
  finally { loading.value = false; }
}
function update(item: InstalledConnector) {
  items.value = items.value.map(c => c.id === item.id ? item : c);
  selected.value = item;
}
function open(item: InstalledConnector) { selected.value = item; detailError.value = ''; unbinding.value = false; }
async function action(command: string, args: Record<string, unknown>) {
  if (busy.value || !selected.value) return;
  busy.value = true; detailError.value = '';
  try { update(await invoke<InstalledConnector>(command, { id: selected.value.id, ...args })); }
  catch (e) { detailError.value = String(e); }
  finally { busy.value = false; }
}
async function unbind() {
  if (!selected.value || busy.value) return;
  if (!unbinding.value) { unbinding.value = true; return; }
  busy.value = true; detailError.value = '';
  try {
    await invoke('ai_connector_unbind', { id: selected.value.id });
    selected.value = null; unbinding.value = false; await refresh();
  } catch (e) { detailError.value = String(e); }
  finally { busy.value = false; }
}
async function useExample(text: string) {
  await props.store.chat.select(null);
  props.store.creationDraft = text;
  props.store.navigate('new-task');
  selected.value = null;
}
async function openAuthorizationPage() {
  if (!authorization.value) return;
  try { await invoke('open_download_url', { url: authorization.value.authorizationUrl }); }
  catch (e) { detailError.value = `无法打开授权页面：${String(e)}`; }
}
async function pollAuthorization() {
  clearTimeout(authTimer);
  const flow = authorization.value;
  if (!flow || !alive || polling) return;
  polling = true;
  try {
    const result = await invoke<{ status: string; error?: string }>('ai_store_connector_auth_status', { flowId: flow.flowId });
    if (!alive || authorization.value?.flowId !== flow.flowId) return;
    if (result.status === 'connected') { authorization.value = null; await refresh(); }
    else if (['error', 'cancelled'].includes(result.status)) { authorization.value = null; detailError.value = result.error || '授权已取消，可重新授权。'; }
    else authTimer = setTimeout(() => void pollAuthorization(), 1000);
  } catch (e) { if (alive && authorization.value?.flowId === flow.flowId) detailError.value = `检查授权状态失败：${String(e)}`; }
  finally { polling = false; }
}
async function reauthorize() {
  if (!selected.value || busy.value || authorization.value) return;
  busy.value = true; detailError.value = '';
  try {
    const result = await invoke<ConnectorAuthorization>('ai_connector_reauthorize', { id: selected.value.id });
    if (result.status === 'authorization_required' && result.flowId && result.authorizationUrl) {
      if (!alive) { await invoke('ai_store_connector_auth_cancel', { flowId: result.flowId }); return; }
      authorization.value = { flowId: result.flowId, authorizationUrl: result.authorizationUrl };
      await openAuthorizationPage();
      authTimer = setTimeout(() => void pollAuthorization(), 1000);
    } else if (result.status === 'connected' || result.connected) await refresh();
    else throw new Error('授权未返回成功状态，请重试。');
  } catch (e) { detailError.value = String(e); }
  finally { busy.value = false; }
}
async function cancelAuthorization() {
  if (!authorization.value || busy.value) return;
  busy.value = true;
  try {
    await invoke('ai_store_connector_auth_cancel', { flowId: authorization.value.flowId });
    authorization.value = null; clearTimeout(authTimer);
  } catch (e) { detailError.value = `取消授权失败：${String(e)}`; }
  finally { busy.value = false; }
}
function close() { if (!busy.value && !authorization.value) selected.value = null; }
onMounted(refresh);
onActivated(refresh);
onBeforeUnmount(() => {
  alive = false; clearTimeout(authTimer);
  if (authorization.value) void invoke('ai_store_connector_auth_cancel', { flowId: authorization.value.flowId }).catch(() => {});
});
defineExpose({ refresh, count: computed(() => items.value.length) });
</script>

<template>
  <section class="connector-center" aria-label="已安装的连接器">
    <div class="center-heading"><h2>已安装的连接器</h2><button class="xb-btn is-sm" :disabled="loading || !store.desktop" @click="refresh">刷新</button></div>
    <p v-if="!store.desktop">请在桌面应用中管理连接器。</p>
    <p v-if="error" role="alert">{{ error }} <button class="xb-btn is-sm" @click="refresh">重试</button></p>
    <p v-if="loading" role="status">正在读取连接器…</p>
    <p v-else-if="!visible.length && !error">暂无匹配的已安装连接器。</p>
    <div class="xc-grid">
      <article v-for="item in visible" :key="item.id" class="xc store-card connector-card">
        <div class="xc-top"><div class="xc-ic"><img v-if="item.iconUrl?.startsWith('data:image/')" :src="item.iconUrl" alt="" /><Plug v-else :size="24" /></div><div class="xc-main"><h3>{{ item.name }}</h3><p class="xc-desc">{{ item.description }}</p></div></div>
        <div class="xc-foot"><span class="state" :class="item.status"><i />{{ item.unsupported ? '暂不支持' : connectorStatus[item.status] }}</span><span class="xc-meta">v{{ item.version }} · {{ connectorAuth[item.auth.type] }} · {{ item.tools.length }} 个工具</span><button class="xc-pill" @click="open(item)">管理</button></div>
      </article>
    </div>
    <NModal :show="!!selected" preset="card" :title="selected?.name" class="store-dialog" :mask-closable="!busy && !authorization" :closable="!busy && !authorization" style="width:640px;max-width:calc(100vw - 36px)" @update:show="!$event && close()">
      <template v-if="selected">
        <p>{{ selected.description }}</p><p>v{{ selected.version }} · {{ selected.source === 'local' ? '本地安装' : '小伴市场' }} · {{ connectorAuth[selected.auth.type] }} · {{ connectorStatus[selected.status] }}</p>
        <p v-if="selected.unsupported" role="alert">暂不支持：{{ selected.unsupported }}</p>
        <p v-if="selected.error" role="alert">{{ selected.error }}</p><p v-if="detailError" role="alert">{{ detailError }}</p>
        <template v-if="authorization"><p role="status">请在浏览器完成授权，完成后会自动更新连接状态。</p><button class="xb-btn" :disabled="busy" @click="openAuthorizationPage">打开授权页面</button><button class="xb-btn" :disabled="busy" @click="pollAuthorization">检查授权状态</button><button class="xb-btn" :disabled="busy" @click="cancelAuthorization">取消授权</button></template>
        <template v-else>
          <div class="center-actions"><button class="xb-btn" :disabled="busy || !!selected.unsupported || selected.status === 'connecting'" @click="action('ai_connector_set_enabled', { enabled: !selected.enabled })">{{ selected.enabled ? '停用' : '启用' }}</button><button class="xb-btn" :disabled="busy" @click="unbind">{{ unbinding ? '确认断开并清除凭据' : '断开并清除凭据' }}</button><button v-if="selected.auth.type === 'oauth'" class="xb-btn" :disabled="busy || !!selected.unsupported" @click="reauthorize">重新授权</button></div>
          <p v-if="unbinding">确认断开此服务并删除已保存的凭据？再次连接需要重新配置。</p>
          <ConnectorTokenForm v-if="!selected.unsupported && selected.auth.type !== 'oauth' && selected.auth.type !== 'cli' && (selected.auth.schema || selected.auth.type === 'none')" :key="selected.id" :schema="selected.auth.schema" :configured="selected.auth.configured" :busy="busy" @connect="action('ai_connector_configure', { values: $event, confirmed: true })" />
          <p v-else-if="!selected.unsupported && ['token', 'env'].includes(selected.auth.type)">此连接器缺少配置表单，请更新插件后重试。</p>
          <p v-if="selected.auth.type === 'cli'">请先按服务商说明完成命令行登录，再启用连接器。</p>
        </template>
        <h4>工具（{{ selected.tools.length }}）</h4><ul><li v-for="tool in selected.tools" :key="tool.name"><b>{{ tool.name }}</b> {{ tool.description }}</li></ul>
        <h4>示例提问</h4><div class="examples"><button v-for="example in selected.examples" :key="example" class="xb-btn" :disabled="busy || !!authorization" @click="useExample(example)">{{ example }}</button></div>
        <h4>随包技能（{{ selected.skills.length }}）</h4><ul><li v-for="skill in selected.skills" :key="skill">{{ skill }}</li></ul>
      </template>
    </NModal>
  </section>
</template>

<style scoped>
.connector-center{margin-bottom:24px}.center-heading,.center-actions{display:flex;align-items:center;gap:10px;flex-wrap:wrap}.center-heading{justify-content:space-between;margin-bottom:12px}h2,h3{margin:0;font-size:14px}.state{display:inline-flex;align-items:center;gap:5px;font-size:12px;color:var(--ui-text-secondary)}.state i{width:7px;height:7px;border-radius:50%;background:var(--ui-text-tertiary)}.state.connected i{background:var(--ui-success-text)}.state.error i,.state.needs_auth i{background:var(--ui-warning)}.state.connecting i{background:var(--ui-ink)}.examples{display:flex;gap:8px;flex-wrap:wrap}li,p{font-size:13px;line-height:1.7;overflow-wrap:anywhere}.xc-foot{flex-wrap:wrap}.xc-ic img{width:100%;height:100%;object-fit:contain}
</style>
