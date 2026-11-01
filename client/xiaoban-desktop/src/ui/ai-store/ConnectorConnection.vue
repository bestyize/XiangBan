<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { invoke } from '../../bridge.ts';
import ConnectorTokenForm from './ConnectorTokenForm.vue';
import ConnectionJourney from './ConnectionJourney.vue';
import { type ConnectorAuthorization, type InstalledConnector } from './connector-plugins.ts';

const props = defineProps<{ connector: InstalledConnector; autoConnect?: boolean }>();
const emit = defineEmits<{ updated: [item: InstalledConnector]; connected: [item: InstalledConnector]; busy: [value: boolean]; screen: [value: string]; use: []; close: [] }>();
const busy = ref(false), error = ref('');
const authorization = ref<{ flowId: string; authorizationUrl: string; userCode?: string } | null>(null);
const copied = ref(false);
const manualToken = ref(false);
const configuring = ref(false);
const waitPane = ref<HTMLElement>();
const browserAuth = computed(() => props.connector.auth.browser === true && !manualToken.value);
const browserOpened = ref(false);
const baiduPersonal = computed(() => props.connector.auth.personalAuth === 'baidu-netdisk');
const baiduFlow = ref<string | null>(null);
const stage = computed(() => authorization.value || baiduFlow.value ? 'authorizing' : busy.value || props.connector.status === 'connecting' ? 'connecting' : props.connector.status === 'connected' ? 'connected' : 'ready');
watch(stage, async value => { if (value === 'authorizing') { await nextTick(); waitPane.value?.focus(); } });
const screen = computed(() => configuring.value ? 'configuring' : stage.value);
watch(screen, value => emit('screen', value), { immediate: true });
const baiduManual = ref(false);
const baiduAuthorizationUrl = 'https://openapi.baidu.com/oauth/2.0/authorize?client_id=QHOuRXiepJBMjtk0esLhrPoNlQyYd0mF&redirect_uri=oob&response_type=token&scope=basic%2Cnetdisk';
let alive = true, polling = false;
let timer: ReturnType<typeof setTimeout> | undefined;
const local = computed(() => props.connector.transport === 'stdio' && !browserAuth.value);
const missing = computed(() => !browserAuth.value && (props.connector.auth.schema?.fields.some(f => f.required && !props.connector.auth.configured.includes(f.key)) ?? false));
const configurable = computed(() => !props.connector.unsupported && (!['token', 'env'].includes(props.connector.auth.type) || !!props.connector.auth.schema));
function setBusy(value: boolean) { busy.value = value; emit('busy', value || !!authorization.value || !!baiduFlow.value); }
function update(item: InstalledConnector, notifyConnected = true) {
  if (item.status === 'connected' && notifyConnected) { configuring.value = false; baiduManual.value = false; manualToken.value = false; }
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
async function openBaiduAuthorization() {
  try { await invoke('open_download_url', { url: baiduAuthorizationUrl }); error.value = ''; }
  catch { if (alive) error.value = '无法打开百度授权页面，请重试。'; }
}
async function pollBaiduAuthorization() {
  clearTimeout(timer);
  const flowId = baiduFlow.value;
  if (!flowId || !alive || polling) return;
  polling = true;
  try {
    const result = await invoke<{ status: string; error?: string }>('ai_connector_baidu_status', { flowId });
    if (!alive || baiduFlow.value !== flowId) return;
    if (result.status === 'connected') {
      baiduFlow.value = null; emit('busy', false); error.value = '';
      const item = await refresh(true);
      if (item && item.status !== 'connected') error.value = item.error || '授权已完成，连接尚未就绪，请重试。';
    } else if (['error', 'cancelled'].includes(result.status)) {
      baiduFlow.value = null; emit('busy', false);
      error.value = result.error || '授权已取消，可重新连接。';
      await refresh();
    } else timer = setTimeout(() => void pollBaiduAuthorization(), 1000);
  } catch { if (alive && baiduFlow.value === flowId) error.value = '检查百度授权状态失败，请重试。'; }
  finally { polling = false; }
}
async function startBaiduAuthorization() {
  if (busy.value || baiduFlow.value) return;
  setBusy(true); error.value = '';
  try {
    const result = await invoke<{ flowId: string }>('ai_connector_baidu_start', { id: props.connector.id });
    if (!alive) { await invoke('ai_connector_baidu_cancel', { flowId: result.flowId }); return; }
    baiduFlow.value = result.flowId; emit('busy', true);
    timer = setTimeout(() => void pollBaiduAuthorization(), 1000);
  } catch (e) { if (alive) error.value = String(e); }
  finally { if (alive) { busy.value = false; emit('busy', !!baiduFlow.value); } }
}
async function cancelBaiduAuthorization() {
  const flowId = baiduFlow.value;
  if (!flowId || busy.value) return;
  setBusy(true);
  try {
    await invoke('ai_connector_baidu_cancel', { flowId });
    if (!alive || baiduFlow.value !== flowId) return;
    const result = await invoke<{ status: string; error?: string }>('ai_connector_baidu_status', { flowId });
    if (!alive || baiduFlow.value !== flowId) return;
    if (result.status === 'pending') { timer = setTimeout(() => void pollBaiduAuthorization(), 1000); return; }
    baiduFlow.value = null; clearTimeout(timer);
    error.value = result.status === 'connected' ? '' : result.error || '授权已取消。';
    await refresh(result.status === 'connected');
  } catch { if (alive) error.value = '取消授权失败，请重试。'; }
  finally { if (alive) setBusy(false); }
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
  if (baiduPersonal.value && values.BAIDU_NETDISK_ACCESS_TOKEN) {
    const input = values.BAIDU_NETDISK_ACCESS_TOKEN.trim();
    if (/^https?:\/\//i.test(input)) {
      try {
        const url = new URL(input);
        if (url.protocol !== 'https:' || !(url.hostname === 'baidu.com' || url.hostname.endsWith('.baidu.com'))) throw new Error();
        const token = url.searchParams.get('access_token') || new URLSearchParams(url.hash.slice(1)).get('access_token');
        if (!token) throw new Error();
        values = { ...values, BAIDU_NETDISK_ACCESS_TOKEN: token };
      } catch { error.value = '请粘贴百度授权后的完整地址，或只填写 Access Token。'; return; }
    } else values = { ...values, BAIDU_NETDISK_ACCESS_TOKEN: input };
  }
  setBusy(true); error.value = '';
  try {
    const result = await invoke<InstalledConnector | ConnectorAuthorization>('ai_connector_configure', { id: props.connector.id, values, confirmed: true,
      ...(props.connector.auth.browser ? { preferBrowser: browserAuth.value } : {}) });
    if (!alive) {
      if ('flowId' in result && result.flowId) await invoke('ai_store_connector_auth_cancel', { flowId: result.flowId });
      return;
    }
    if (result.status === 'authorization_required' && 'flowId' in result && result.flowId && result.authorizationUrl) {
      configuring.value = false;
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
    if (!alive || authorization.value?.flowId !== flow.flowId) return;
    authorization.value = null; clearTimeout(timer);
    const result = await invoke<{ status: string; error?: string }>('ai_store_connector_auth_status', { flowId: flow.flowId });
    if (!alive) return;
    error.value = result.status === 'connected' ? '' : result.error || '授权已取消，稍后可重新连接。';
    await refresh(result.status === 'connected');
  } catch (e) { if (alive) error.value = `取消授权失败：${String(e)}`; }
  finally { if (alive) setBusy(false); }
}
/** Match the reference flow: connect first, ask only for credentials this service actually requires. */
async function begin() {
  if (busy.value || authorization.value || baiduFlow.value || props.connector.status === 'connecting') return;
  if (baiduPersonal.value) { await startBaiduAuthorization(); return; }
  if (!browserAuth.value && missing.value) { configuring.value = true; return; }
  await connect();
}
function showManual() { manualToken.value = true; baiduManual.value = baiduPersonal.value; configuring.value = true; error.value = ''; }
defineExpose({ begin });
onMounted(() => {
  // Auto-connect is only requested by an explicit “安装并连接” action.
  if (props.autoConnect && (!local.value || baiduPersonal.value) && configurable.value && props.connector.auth.type !== 'cli') void begin();
});
onBeforeUnmount(() => {
  alive = false; clearTimeout(timer);
  if (authorization.value) void invoke('ai_store_connector_auth_cancel', { flowId: authorization.value.flowId }).catch(() => {});
  if (baiduFlow.value) void invoke('ai_connector_baidu_cancel', { flowId: baiduFlow.value }).catch(() => {});
});
</script>

<template>
  <section class="connector-connection" :class="{ 'is-configuring': configuring }" :aria-label="`${connector.name}连接`">
    <ConnectionJourney v-if="!configuring" :name="connector.name" :description="connector.description" :stage="stage" :icon-url="connector.iconUrl" />
    <p v-if="connector.unsupported" class="connection-error" role="alert">暂不支持：{{ connector.unsupported }}</p>
    <p v-if="error || (!authorization && connector.error)" class="connection-error" role="alert">{{ error || connector.error }}</p>
    <div v-if="authorization" ref="waitPane" class="authorization-panel" tabindex="-1" role="status" aria-live="polite">
      <template v-if="authorization.userCode">
        <p>{{ browserOpened ? '已在浏览器打开授权页，请输入下方验证码完成授权：' : '请打开授权页面，输入下方验证码完成授权：' }}</p>
        <div class="code-row"><strong class="user-code" data-testid="connector-user-code">{{ authorization.userCode }}</strong><button class="xb-btn" type="button" @click="copyUserCode">{{ copied ? '已复制' : '复制验证码' }}</button></div>
      </template>
      <p v-else>请在浏览器完成授权，完成后会自动更新连接状态。</p>
      <div class="connection-actions"><button class="store-primary" :disabled="busy" @click="openAuthorizationPage">打开授权页面</button></div>
      <div class="connection-secondary"><button class="xb-btn" :disabled="busy" @click="pollAuthorization">检查授权状态</button><button class="xb-btn" :disabled="busy" @click="cancelAuthorization">取消授权</button></div>
    </div>
    <div v-else-if="baiduFlow" ref="waitPane" class="authorization-panel" tabindex="-1" role="status" aria-live="polite">
      <p>请在百度授权窗口完成登录并确认权限，完成后会自动连接。</p>
      <div class="connection-secondary"><button class="xb-btn" :disabled="busy" @click="pollBaiduAuthorization">检查授权状态</button><button class="xb-btn" :disabled="busy" @click="cancelBaiduAuthorization">取消授权</button></div>
    </div>
    <template v-else-if="configuring">
      <p v-if="local" class="local-warning">连接时将在本机运行以下命令，需要本机已有对应运行环境：</p>
      <pre v-if="local && connector.command">{{ [connector.command, ...(connector.args ?? [])].join(' ') }}</pre>
      <button v-if="baiduManual" class="xb-btn" @click="openBaiduAuthorization">打开百度授权页面</button>
      <ConnectorTokenForm :schema="connector.auth.schema" :configured="connector.auth.configured" :busy="busy || connector.status === 'connecting'" :confirmation-required="false" cancelable :submit-label="local ? '运行并连接' : '保存并连接'" @connect="connect" @cancel="configuring = false; manualToken = false; baiduManual = false" />
    </template>
    <template v-else-if="configurable">
      <template v-if="connector.status === 'connected'">
        <p class="connection-success" role="status">已连接，可在任务中使用 · {{ connector.tools.length }} 个工具</p>
        <div class="connection-actions"><button class="store-primary" @click="connector.examples.length ? emit('use') : emit('close')">{{ connector.examples.length ? '去试试' : '完成' }}</button></div>
        <div class="connection-secondary"><button v-if="['oauth', 'auto', 'none'].includes(connector.auth.type) || browserAuth || baiduPersonal" class="xb-btn" :disabled="busy" @click="begin">重新连接</button><button v-if="connector.auth.schema?.fields.length && !browserAuth && !baiduPersonal" class="xb-btn" :disabled="busy" @click="showManual">重新配置</button></div>
      </template>
      <template v-else>
        <p v-if="local" class="local-warning">连接时将在本机运行以下命令，需要本机已有对应运行环境：</p>
        <pre v-if="local && connector.command">{{ [connector.command, ...(connector.args ?? [])].join(' ') }}</pre>
        <p v-if="connector.auth.type === 'cli'" class="local-warning">此服务需要专属命令行登录。请先按服务商说明完成登录；小伴暂不代办此登录流程。</p>
        <div class="connection-actions"><button class="store-primary" :disabled="busy || connector.status === 'connecting' || connector.auth.type === 'cli'" @click="begin">{{ busy || connector.status === 'connecting' ? '连接中…' : local ? '运行并连接' : '连接' }}</button></div>
      </template>
      <details v-if="(connector.auth.browser && connector.auth.schema?.fields.length) || baiduPersonal" class="connection-alternatives"><summary>其他连接方式</summary><button class="xb-btn" type="button" :disabled="busy" @click="showManual">{{ baiduPersonal ? '手动输入授权信息' : '使用 Token 连接' }}</button></details>
    </template>
    <p v-else-if="!connector.unsupported" class="connection-error" role="alert">此连接器缺少配置表单，请更新插件后重试。</p>
  </section>
</template>

<style scoped>
.connector-connection{margin:0;padding:12px 0 0}.connector-connection p{font-size:13px;line-height:1.75;overflow-wrap:anywhere}.connection-error{margin:18px 0 0;padding:10px 12px;border-radius:8px;background:var(--ui-control);color:var(--ui-danger)}
.connection-actions{display:flex;align-items:center;justify-content:center;gap:8px;margin-top:22px}.connection-actions .store-primary{min-width:76px;padding:0 18px}.connection-secondary{display:flex;justify-content:center;align-items:center;gap:8px;margin-top:10px}.connection-secondary .xb-btn{font-size:12px;height:28px;padding:0 8px}
.authorization-panel{text-align:center;outline:none;padding:0 0 4px}.authorization-panel:focus-visible{outline:2px solid var(--ui-btn);outline-offset:4px}.authorization-panel>p{margin:16px 0 0;color:var(--ui-text-secondary)}.code-row{display:flex;justify-content:center;align-items:center;gap:12px;margin:14px 0 0;flex-wrap:wrap}.user-code{font-size:25px;letter-spacing:3px;font-family:var(--ui-font-mono,monospace);color:var(--ui-text-primary)}
.connection-success{margin:16px 0 0;text-align:center;color:var(--ui-success-text)}.connection-alternatives{margin-top:18px;text-align:center;font-size:11px;color:var(--ui-text-tertiary)}.connection-alternatives summary{cursor:pointer}.connection-alternatives .xb-btn{margin:8px 0 0;font-size:12px}
.local-warning{margin:18px 0 0;color:var(--ui-text-secondary)}pre{white-space:pre-wrap;overflow-wrap:anywhere;font-size:12px;background:var(--ui-control);padding:12px;border-radius:8px}
.is-configuring{padding-top:0}
</style>
