<script setup lang="ts">
import StoreDialog from './StoreDialog.vue';
import { computed, nextTick, onActivated, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { Lightbulb, MessageCircle, Plug, Trash2 } from 'lucide-vue-next';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import ConnectorConnection from './ConnectorConnection.vue';
import { connectorAuth, connectorStatus, type InstalledConnector } from './connector-plugins.ts';
import { fitConnectorWindow, openStoreTask, openStoreWindow, storeWindowScene, useNativeStoreWindow } from './store-window.ts';

const props = defineProps<{ store: ControlStore; query?: string; detailId?: string }>();
const items = ref<InstalledConnector[]>([]);
const selected = ref<InstalledConnector | null>(null);
const loading = ref(false), busy = ref(false), error = ref(''), detailError = ref(''), unbinding = ref(false);
const connectionBusy = ref(false);
const connection = ref<InstanceType<typeof ConnectorConnection>>();
const connectionScreen = ref('ready'), pendingExample = ref<string | null>(null);
const removing = ref(false);
const relatedServices = computed(() => selected.value?.plugin ? items.value.filter(item => item.plugin === selected.value!.plugin) : []);
const detailBody = ref<HTMLElement>();
async function fit() { await nextTick(); fitConnectorWindow(detailBody.value); }
watch(() => [selected.value?.id, connectionScreen.value, removing.value], fit);
let alive = true;
let refreshTimer: ReturnType<typeof setTimeout> | undefined;
let pendingPlugin: string | undefined;
const visible = computed(() => items.value.filter(c => `${c.name} ${c.nameEn ?? ''} ${c.description}`.toLowerCase().includes((props.query ?? '').trim().toLowerCase())));
async function refresh() {
  if (!props.store.desktop || loading.value) return;
  loading.value = true; error.value = '';
  try {
    const listed = await invoke<InstalledConnector[]>('ai_connector_installed');
    if (!alive) return;
    const result = Array.isArray(listed) ? listed : [];
    items.value = result;
    clearTimeout(refreshTimer);
    if (result.some(c => c.status === 'connecting')) refreshTimer = setTimeout(() => void refresh(), 1500);
    if (selected.value) selected.value = result.find(c => c.id === selected.value!.id) ?? null;
    if (pendingPlugin) {
      const item = result.find(c => c.plugin === pendingPlugin);
      if (item) { pendingPlugin = undefined; open(item); }
    }
  } catch (e) { if (alive) error.value = `无法读取已安装连接器：${String(e)}`; }
  finally { loading.value = false; }
}
function update(item: InstalledConnector) {
  items.value = items.value.map(c => c.id === item.id ? item : c);
  selected.value = item;
}
function open(item: InstalledConnector) {
  if (useNativeStoreWindow()) { void openStoreWindow({ view: 'connector', kind: 'connectors', source: 'market', item: { slug: item.id, name: item.name, description: item.description } }).catch(props.store.notify); return; }
  selected.value = item; detailError.value = ''; unbinding.value = false; removing.value = false; pendingExample.value = null;
}
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
async function remove() {
  const item = selected.value;
  if (!item?.plugin || busy.value || connectionBusy.value) return;
  busy.value = true; detailError.value = '';
  try {
    await invoke('ai_store_uninstall', { slug: item.plugin });
    items.value = items.value.filter(service => service.plugin !== item.plugin);
    selected.value = null; removing.value = false; pendingExample.value = null;
    props.store.notify(`已移除${item.name}`);
  } catch (e) { detailError.value = `移除失败：${String(e)}`; }
  finally { busy.value = false; }
}
async function useExample(text: string) {
  if (storeWindowScene() && selected.value) { await openStoreTask({ kind: 'example', id: selected.value.id, example: selected.value.examples.indexOf(text) }).catch(props.store.notify); return; }
  await props.store.chat.select(null);
  props.store.creationDraft = text;
  props.store.navigate('new-task');
  selected.value = null;
}
async function tryExample(text: string) {
  if (selected.value?.status === 'connected') { await useExample(text); return; }
  pendingExample.value = text;
  await connection.value?.begin();
}
function connectionFinished() {
  if (pendingExample.value) { const text = pendingExample.value; pendingExample.value = null; void useExample(text); }
}
function close() { if (!busy.value) selected.value = null; }
onMounted(async () => {
  await refresh();
  if (props.detailId) {
    const item = items.value.find(item => item.id === props.detailId);
    if (item) open(item); else error.value = '此连接器已被移除，请关闭窗口后刷新。';
  }
});
onActivated(refresh);
onBeforeUnmount(() => {
  alive = false; clearTimeout(refreshTimer);
});
async function reveal(plugin: string) {
  pendingPlugin = plugin;
  await refresh();
}
defineExpose({ refresh, reveal, count: computed(() => items.value.length) });
</script>

<template>
  <section class="connector-center" aria-label="已安装的连接器">
    <template v-if="!detailId">
    <div class="center-heading"><h2>已安装的连接器</h2><button class="xb-btn is-sm" :disabled="loading || !store.desktop" @click="refresh">刷新</button></div>
    <p v-if="!store.desktop">请在桌面应用中管理连接器。</p>
    <p v-if="error" role="alert">{{ error }} <button class="xb-btn is-sm" @click="refresh">重试</button></p>
    <p v-if="loading" role="status">正在读取连接器…</p>
    <p v-else-if="!visible.length && !error">暂无匹配的已安装连接器。</p>
    <div class="xc-grid connector-grid">
      <article v-for="item in visible" :key="item.id" class="xc store-card connector-card" :title="item.description" :aria-description="item.description">
        <div class="xc-top"><div class="xc-ic"><img v-if="item.iconUrl?.startsWith('data:image/')" :src="item.iconUrl" alt="" /><Plug v-else :size="24" /></div><div class="xc-main"><div class="xc-title"><h3><button type="button" class="xc-title-button" @click="open(item)">{{ item.name }}</button></h3></div><span class="xc-kind">{{ connectorAuth[item.auth.type] }}</span></div></div>
        <div class="xc-foot"><span class="state" :class="item.status"><i />{{ item.unsupported ? '暂不支持' : connectorStatus[item.status] }}</span><span class="xc-meta">v{{ item.version }} · {{ item.tools.length }} 个工具</span><button class="xc-pill" @click="open(item)">管理</button></div>
      </article>
    </div>
    </template>
    <p v-if="detailId && error" role="alert">{{ error }}</p>
    <StoreDialog :show="!!selected" preset="card" :title="selected?.name" hide-title class="store-dialog connector-connect-dialog" :mask-closable="!busy" :closable="!busy" style="width:600px;max-width:calc(100vw - 36px)" @update:show="!$event && close()">
      <div v-if="selected" ref="detailBody" class="connector-connect-body">
        <p v-if="detailError" role="alert">{{ detailError }}</p>
        <section v-if="removing" class="connector-remove-confirm" aria-labelledby="connector-remove-title">
          <Trash2 :size="28" aria-hidden="true" />
          <h2 id="connector-remove-title">移除「{{ selected.name }}」？</h2>
          <p>将卸载插件及随包技能，断开服务并清除本机保存的凭据。之后可以重新安装。</p>
          <p v-if="relatedServices.length > 1" class="connector-remove-services" role="alert">此插件包含 {{ relatedServices.length }} 项服务，将一并移除：{{ relatedServices.map(item => item.name).join('、') }}。</p>
          <div class="center-actions"><button class="xb-btn" :disabled="busy" @click="removing = false; detailError = ''">取消</button><button class="store-primary connector-remove-primary" :disabled="busy" @click="remove">{{ busy ? '移除中…' : '确认移除' }}</button></div>
        </section>
        <template v-else>
        <ConnectorConnection :key="selected.id" ref="connection" :connector="selected" @updated="update" @connected="connectionFinished" @busy="connectionBusy = $event" @screen="connectionScreen = $event" @use="useExample(selected.examples[0])" @close="close" />
        <section v-if="selected.examples.length && ['ready', 'connected'].includes(connectionScreen)" class="connector-examples"><h4><Lightbulb :size="16" />试试这样用</h4><button v-for="example in selected.examples" :key="example" class="connector-example" :title="example" :disabled="busy || connectionBusy" @click="tryExample(example)"><span>“{{ example }}”</span><MessageCircle :size="15" /></button></section>
        <div v-show="['ready', 'connected'].includes(connectionScreen)" class="connector-management-row">
        <details class="connector-management" @toggle="fit"><summary>连接管理</summary>
        <p class="store-detail-meta">v{{ selected.version }} · {{ selected.source === 'local' ? '本地安装' : '小伴市场' }} · {{ connectorAuth[selected.auth.type] }} · {{ connectorStatus[selected.status] }}</p>
        <details v-if="selected.tools.length" class="connector-capabilities"><summary>工具（{{ selected.tools.length }}）</summary><ul class="store-tool-list"><li v-for="tool in selected.tools" :key="tool.name" :title="tool.description" :aria-description="tool.description"><b>{{ tool.name }}</b></li></ul></details>
        <details v-if="selected.skills.length" class="connector-capabilities"><summary>随包技能（{{ selected.skills.length }}）</summary><ul><li v-for="skill in selected.skills" :key="skill">{{ skill }}</li></ul></details>
        <div class="center-actions"><button v-if="selected.enabled" class="xb-btn" :disabled="busy || connectionBusy" @click="action('ai_connector_set_enabled', { enabled: false })">停用</button><button class="xb-btn" :disabled="busy || connectionBusy" @click="unbind">{{ unbinding ? '确认断开并清除凭据' : '断开并清除凭据' }}</button></div>
        <p v-if="unbinding">确认断开此服务并删除已保存的凭据？再次连接需要重新配置。</p>
        </details>
        <button v-if="selected.plugin" class="connector-remove" :disabled="busy || connectionBusy" @click="removing = true; detailError = ''"><Trash2 :size="13" aria-hidden="true" />移除</button>
        </div>
        </template>
      </div>
    </StoreDialog>
  </section>
</template>

<style scoped>
.connector-center{margin-bottom:24px}.center-heading,.center-actions{display:flex;align-items:center;gap:10px;flex-wrap:wrap}.center-heading{justify-content:space-between;margin-bottom:12px}h2,h3{margin:0;font-size:14px}.state{display:inline-flex;align-items:center;gap:5px;font-size:12px;color:var(--ui-text-secondary)}.state i{width:7px;height:7px;border-radius:50%;background:var(--ui-text-tertiary)}.state.connected i{background:var(--ui-success-text)}.state.error i,.state.needs_auth i{background:var(--ui-warning)}.state.connecting i{background:var(--ui-ink)}.examples{display:flex;gap:8px;flex-wrap:wrap}li,p{font-size:13px;line-height:1.7;overflow-wrap:anywhere}.xc-foot{flex-wrap:wrap}.xc-ic img{width:100%;height:100%;object-fit:contain}
.connector-management-row{display:flex;align-items:flex-start;gap:16px;margin-top:14px}.connector-management-row>.connector-management{flex:1;min-width:0;margin:0}.connector-remove{display:inline-flex;align-items:center;gap:5px;padding:0;border:0;background:transparent;color:var(--ui-text-secondary);font:inherit;font-size:12px;cursor:pointer;line-height:1.6;white-space:nowrap}.connector-remove:hover{color:var(--ui-danger)}.connector-remove:disabled{opacity:.5;cursor:default}.connector-remove-confirm{padding:24px 0 8px}.connector-remove-confirm>svg{color:var(--ui-danger);margin-bottom:16px}.connector-remove-confirm h2{font-size:20px}.connector-remove-confirm p{color:var(--ui-text-secondary);margin:16px 0 24px}.connector-remove-confirm .connector-remove-services{color:var(--ui-danger)}.connector-remove-confirm .center-actions{justify-content:flex-end}.connector-remove-confirm .connector-remove-primary:not(:disabled){background:var(--ui-danger);color:var(--ui-text-on-btn)}.connector-remove-confirm .connector-remove-primary:hover:not(:disabled){background:color-mix(in srgb,var(--ui-danger) 90%,var(--ui-text-primary))}
</style>
