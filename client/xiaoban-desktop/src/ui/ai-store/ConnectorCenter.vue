<script setup lang="ts">
import { computed, onActivated, onBeforeUnmount, onMounted, ref } from 'vue';
import { NModal } from 'naive-ui';
import { Plug } from 'lucide-vue-next';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import ConnectorConnection from './ConnectorConnection.vue';
import { connectorAuth, connectorStatus, type InstalledConnector } from './connector-plugins.ts';

const props = defineProps<{ store: ControlStore; query?: string }>();
const items = ref<InstalledConnector[]>([]);
const selected = ref<InstalledConnector | null>(null);
const loading = ref(false), busy = ref(false), error = ref(''), detailError = ref(''), unbinding = ref(false);
const connectionBusy = ref(false);
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
function close() { if (!busy.value && !connectionBusy.value) selected.value = null; }
onMounted(refresh);
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
    <NModal :show="!!selected" preset="card" :title="selected?.name" class="store-dialog" :mask-closable="!busy && !connectionBusy" :closable="!busy && !connectionBusy" style="width:640px;max-width:calc(100vw - 36px)" @update:show="!$event && close()">
      <template v-if="selected">
        <p>{{ selected.description }}</p><p>v{{ selected.version }} · {{ selected.source === 'local' ? '本地安装' : '小伴市场' }} · {{ connectorAuth[selected.auth.type] }} · {{ connectorStatus[selected.status] }}</p>
        <p v-if="detailError" role="alert">{{ detailError }}</p>
        <div class="center-actions"><button v-if="selected.enabled" class="xb-btn" :disabled="busy || connectionBusy" @click="action('ai_connector_set_enabled', { enabled: false })">停用</button><button class="xb-btn" :disabled="busy || connectionBusy" @click="unbind">{{ unbinding ? '确认断开并清除凭据' : '断开并清除凭据' }}</button></div>
        <p v-if="unbinding">确认断开此服务并删除已保存的凭据？再次连接需要重新配置。</p>
        <ConnectorConnection :key="selected.id" :connector="selected" @updated="update" @busy="connectionBusy = $event" />
        <h4>工具（{{ selected.tools.length }}）</h4><ul><li v-for="tool in selected.tools" :key="tool.name"><b>{{ tool.name }}</b> {{ tool.description }}</li></ul>
        <h4>示例提问</h4><div class="examples"><button v-for="example in selected.examples" :key="example" class="xb-btn" :disabled="busy || connectionBusy || selected.status !== 'connected'" @click="useExample(example)">{{ example }}</button></div>
        <h4>随包技能（{{ selected.skills.length }}）</h4><ul><li v-for="skill in selected.skills" :key="skill">{{ skill }}</li></ul>
      </template>
    </NModal>
  </section>
</template>

<style scoped>
.connector-center{margin-bottom:24px}.center-heading,.center-actions{display:flex;align-items:center;gap:10px;flex-wrap:wrap}.center-heading{justify-content:space-between;margin-bottom:12px}h2,h3{margin:0;font-size:14px}.state{display:inline-flex;align-items:center;gap:5px;font-size:12px;color:var(--ui-text-secondary)}.state i{width:7px;height:7px;border-radius:50%;background:var(--ui-text-tertiary)}.state.connected i{background:var(--ui-success-text)}.state.error i,.state.needs_auth i{background:var(--ui-warning)}.state.connecting i{background:var(--ui-ink)}.examples{display:flex;gap:8px;flex-wrap:wrap}li,p{font-size:13px;line-height:1.7;overflow-wrap:anywhere}.xc-foot{flex-wrap:wrap}.xc-ic img{width:100%;height:100%;object-fit:contain}
</style>
