<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { invoke } from '../../bridge.ts';
import ConnectorTokenForm from './ConnectorTokenForm.vue';
import { connectorStatus, type ConnectorAuthorization, type InstalledConnector } from './connector-plugins.ts';

const props = defineProps<{ connector: InstalledConnector; autoConnect?: boolean }>();
const emit = defineEmits<{ updated: [item: InstalledConnector]; connected: [item: InstalledConnector]; busy: [value: boolean] }>();
const busy = ref(false), error = ref('');
const authorization = ref<{ flowId: string; authorizationUrl: string; userCode?: string } | null>(null);
const copied = ref(false);
const manualToken = ref(false);
const browserAuth = computed(() => props.connector.auth.browser === true && !manualToken.value);
const browserOpened = ref(false);
let alive = true, polling = false;
let timer: ReturnType<typeof setTimeout> | undefined;
const local = computed(() => props.connector.transport === 'stdio' && !browserAuth.value);
const missing = computed(() => !browserAuth.value && (props.connector.auth.schema?.fields.some(f => f.required && !props.connector.auth.configured.includes(f.key)) ?? false));
const configurable = computed(() => !props.connector.unsupported && (!['token', 'env'].includes(props.connector.auth.type) || !!props.connector.auth.schema));
function setBusy(value: boolean) { busy.value = value; emit('busy', value || !!authorization.value); }
function update(item: InstalledConnector, notifyConnected = true) {
  emit('updated', item);
  if (notifyConnected && item.status === 'connected') emit('connected', item);
}
async function refresh(notifyConnected = false) {
  const items = await invoke<InstalledConnector[]>('ai_connector_installed');
  if (!alive) return;
  const item = items.find(c => c.id === props.connector.id);
  if (!item) throw new Error('连接器已被移除，请重新安装。');
  update(item, notifyConnected);
  return item;
}
async function copyUserCode() {
  if (!authorization.value?.userCode) return;
  try {
    await navigator.clipboard.writeText(authorization.value.userCode);
    copied.value = true;
    setTimeout(() => { copied.value = false; }, 2000);
  } catch { error.value = '复制失败，请手动复制上方验证码。'; }
}
async function openAuthorizationPage() {
  if (!authorization.value) return;
  try { await invoke('open_download_url', { url: authorization.value.authorizationUrl }); browserOpened.value = true; error.value = ''; }
  catch (e) { if (alive) error.value = `无法打开授权页面，请点击“打开授权页面”重试：${String(e)}`; }
}
async function pollAuthorization() {
  clearTimeout(timer);
  const flow = authorization.value;
  if (!flow || !alive || polling) return;
  polling = true;
  try {
    const result = await invoke<{ status: string; error?: string }>('ai_store_connector_auth_status', { flowId: flow.flowId });
    if (!alive || authorization.value?.flowId !== flow.flowId) return;
    if (result.status === 'connected') {
      error.value = '';
      authorization.value = null; emit('busy', false);
      const item = await refresh(true);
      if (item && item.status !== 'connected') error.value = item.error || '授权已完成，连接尚未就绪，请重试。';
    } else if (['error', 'cancelled'].includes(result.status)) {
      authorization.value = null; emit('busy', false);
      error.value = result.error || '授权已取消，可重新连接。';
      await refresh();
    } else timer = setTimeout(() => void pollAuthorization(), 1000);
  } catch (e) {
    if (alive && authorization.value?.flowId === flow.flowId) error.value = `检查授权状态失败，可点击“检查授权状态”重试：${String(e)}`;
  } finally { polling = false; }
}
async function connect(values: Record<string, string> = {}) {
  if (busy.value || authorization.value || !configurable.value) return;
  setBusy(true); error.value = '';
  try {
    const result = await invoke<InstalledConnector | ConnectorAuthorization>('ai_connector_configure', { id: props.connector.id, values, confirmed: true,
      ...(props.connector.auth.browser ? { preferBrowser: browserAuth.value } : {}) });
    if (!alive) {
      if ('flowId' in result && result.flowId) await invoke('ai_store_connector_auth_cancel', { flowId: result.flowId });
      return;
    }
    if (result.status === 'authorization_required' && 'flowId' in result && result.flowId && result.authorizationUrl) {
      authorization.value = { flowId: result.flowId, authorizationUrl: result.authorizationUrl, userCode: result.userCode };
      copied.value = false;
      browserOpened.value = false;
      await openAuthorizationPage();
      if (alive) timer = setTimeout(() => void pollAuthorization(), 1000);
    } else if ('id' in result) {
      update(result);
      if (result.status !== 'connected') error.value = result.error || '连接尚未成功，请重试。';
    } else throw new Error('连接未返回有效状态，请重试。');
  } catch (e) {
    if (alive) {
      error.value = String(e);
      // Credentials may have been saved before the remote connection failed.
      try { await refresh(); } catch { /* Preserve the actionable connection error. */ }
    }
  } finally { if (alive) setBusy(false); }
}
async function cancelAuthorization() {
  const flow = authorization.value;
  if (!flow || busy.value) return;
  setBusy(true);
  try {
    await invoke('ai_store_connector_auth_cancel', { flowId: flow.flowId });
    if (!alive) return;
    authorization.value = null; clearTimeout(timer);
    error.value = '授权已取消，稍后可重新连接。';
    await refresh();
  } catch (e) { if (alive) error.value = `取消授权失败：${String(e)}`; }
  finally { if (alive) setBusy(false); }
}
onMounted(() => {
  // Auto-connect is only requested by an explicit “安装并连接” action.
  if (props.autoConnect && !local.value && configurable.value && !missing.value && props.connector.auth.type !== 'cli') void connect();
});
onBeforeUnmount(() => {
  alive = false; clearTimeout(timer);
  if (authorization.value) void invoke('ai_store_connector_auth_cancel', { flowId: authorization.value.flowId }).catch(() => {});
});
</script>

<template>
  <section class="connector-connection" :aria-label="`${connector.name}连接`">
    <p v-if="connector.unsupported" role="alert">暂不支持：{{ connector.unsupported }}</p>
    <p v-if="error || (!authorization && connector.error)" role="alert">{{ error || connector.error }}</p>
    <template v-if="authorization">
      <div v-if="authorization.userCode" class="device-code-box" role="status">
        <p>{{ browserOpened ? '已在浏览器打开授权页，请输入下方验证码完成授权：' : '请打开授权页面，输入下方验证码完成授权：' }}</p>
        <div class="code-row">
          <strong class="user-code" data-testid="connector-user-code">{{ authorization.userCode }}</strong>
          <button class="xb-btn" type="button" @click="copyUserCode">{{ copied ? '已复制' : '复制验证码' }}</button>
        </div>
      </div>
      <p v-else role="status">请在浏览器完成授权，完成后会自动更新连接状态。</p>
      <button class="xb-btn" :disabled="busy" @click="openAuthorizationPage">打开授权页面</button>
      <button class="xb-btn" :disabled="busy" @click="pollAuthorization">检查授权状态</button>
      <button class="xb-btn" :disabled="busy" @click="cancelAuthorization">取消授权</button>
    </template>
    <template v-else-if="configurable">
      <p v-if="connector.status === 'connected'" role="status">已连接，可在任务中使用 · {{ connector.tools.length }} 个工具</p>
      <button v-if="connector.status === 'connected' && ['oauth', 'auto'].includes(connector.auth.type)" class="xb-btn" :disabled="busy" @click="connect()">重新连接</button>
      <template v-else>
        <p v-if="local">连接时将在本机运行以下命令，需要本机已有对应运行环境：</p>
        <pre v-if="local && connector.command">{{ [connector.command, ...(connector.args ?? [])].join(' ') }}</pre>
        <p v-if="connector.auth.type === 'cli'">此服务需要专属命令行登录。请先按服务商说明完成登录；小伴暂不代办此登录流程。</p>
        <p v-else-if="browserAuth || !connector.auth.schema?.fields.length">无需手动填写配置。需要登录时，会打开浏览器授权页面。</p>
      </template>
      <ConnectorTokenForm v-if="!browserAuth && connector.auth.schema?.fields.length" :schema="connector.auth.schema" :configured="connector.auth.configured" :busy="busy || connector.status === 'connecting'" :confirmation-required="false" :submit-label="connector.status === 'connected' ? '保存并重新连接' : local ? '运行并连接' : '连接'" @connect="connect" />
      <button v-else-if="connector.status !== 'connected'" class="store-primary" :disabled="busy || connector.status === 'connecting'" @click="connect()">{{ busy ? '正在连接…' : connector.status === 'connecting' ? connectorStatus.connecting : local ? '运行并连接' : connector.auth.type === 'oauth' ? '登录并连接' : '连接' }}</button>
      <button v-if="connector.auth.browser && connector.auth.schema?.fields.length" class="xb-btn" type="button" :disabled="busy || connector.status === 'connecting'" @click="manualToken = !manualToken; error = ''">{{ manualToken ? '使用网页登录' : '使用 Token 连接' }}</button>
    </template>
    <p v-else-if="!connector.unsupported">此连接器缺少配置表单，请更新插件后重试。</p>
  </section>
</template>

<style scoped>
.connector-connection{margin:12px 0}.connector-connection p{font-size:13px;line-height:1.7;overflow-wrap:anywhere}.connector-connection button{margin:4px 6px 4px 0}pre{white-space:pre-wrap;overflow-wrap:anywhere;font-size:12px;background:var(--ui-control);padding:10px;border-radius:8px}
.device-code-box{background:var(--ui-control);padding:10px 12px;border-radius:8px;margin-bottom:8px}
.code-row{display:flex;align-items:center;gap:10px;margin:6px 0}
.user-code{font-size:18px;letter-spacing:2px;font-family:monospace;background:rgba(0,0,0,0.15);padding:4px 10px;border-radius:6px}
</style>
