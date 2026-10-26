<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { NAlert } from 'naive-ui';
import ConnectorConnection from './ConnectorConnection.vue';
import { invoke } from '../../bridge.ts';
import { connectorAuth, type InstalledConnector, type MarketConnector } from './connector-plugins.ts';
import type { SkillReview } from './catalog.ts';

interface ConnectorReview {
  sha256: string;
  manifest: {
    version: string;
    connectorCount: number;
    servers: { name: string; transport: string; url?: string; command?: string; args?: string[]; envKeys: string[]; headerKeys: string[] }[];
    skills: string[];
    warnings?: string[];
    files?: SkillReview['manifest']['files'];
  };
  warnings?: string[];
  scripts: SkillReview['scripts'];
}
const props = defineProps<{ connector: MarketConnector; slug: string; namespace?: string; version?: string; installed: boolean; updateVersion?: string; localError: string }>();
const emit = defineEmits<{ installed: []; changed: []; center: []; connected: []; busy: [value: boolean] }>();
const review = ref<ConnectorReview | null>(null);
const loading = ref(false), busy = ref(false), error = ref(''), done = ref(false);
const services = ref<InstalledConnector[]>([]);
const connecting = ref<Record<string, boolean>>({});
const connectOnInstall = ref(true);
const installationFinished = ref(false);
const allConnected = computed(() => services.value.length > 0 && services.value.every(s => s.status === 'connected'));
function serviceBusy(id: string, value: boolean) { connecting.value = { ...connecting.value, [id]: value }; emit('busy', busy.value || Object.values(connecting.value).some(Boolean)); }
function updated(item: InstalledConnector) { services.value = services.value.map(s => s.id === item.id ? item : s); emit('changed'); }
async function loadServices() {
  try {
    const items = await invoke<InstalledConnector[]>('ai_connector_installed');
    if (alive) services.value = items.filter(s => s.plugin === props.slug && s.namespace === props.namespace);
  } catch (e) { if (alive) error.value = `已安装，但读取连接配置失败，可重试：${String(e)}`; }
}
let alive = true;
onBeforeUnmount(() => { alive = false; });
const warnings = computed(() => [...new Set([...(review.value?.manifest.warnings ?? []), ...(review.value?.warnings ?? [])])]);
const targetVersion = computed(() => props.updateVersion || props.version);
let reviewId = 0;
watch(targetVersion, () => { ++reviewId; review.value = null; loading.value = false; error.value = ''; });
const canInstall = computed(() => !props.localError && !!props.namespace && !!targetVersion.value && (!props.installed || !!props.updateVersion) && !done.value);
async function inspect() {
  if (!canInstall.value || loading.value || busy.value) return;
  loading.value = true; error.value = ''; review.value = null;
  const id = ++reviewId, version = targetVersion.value;
  try {
    const result = await invoke<ConnectorReview>('ai_market_inspect', { kind: 'connectors', namespace: props.namespace, slug: props.slug, version });
    if (alive && id === reviewId) {
      if (!result.sha256 || result.manifest.version !== version) throw new Error('审查版本或摘要不完整，请重新审查。');
      review.value = result;
    }
  } catch (e) { if (alive && id === reviewId) error.value = String(e); }
  finally { if (alive && id === reviewId) loading.value = false; }
}
async function install() {
  const r = review.value;
  if (!r || r.manifest.version !== targetVersion.value || busy.value || !canInstall.value) return;
  busy.value = true; emit('busy', true); error.value = '';
  try {
    await invoke('ai_market_install', { kind: 'connectors', namespace: props.namespace, slug: props.slug, version: r.manifest.version, sha256: r.sha256 });
    done.value = true; review.value = null;
    emit('installed');
    await loadServices();
    installationFinished.value = true;
  } catch (e) { error.value = String(e); }
  finally { busy.value = false; emit('busy', Object.values(connecting.value).some(Boolean)); }
}
</script>

<template>
  <div class="market-connector-detail">
    <div class="detail-tags"><span v-for="auth in connector.authTypes" :key="auth" class="store-tag">{{ connectorAuth[auth] }}</span><span v-if="version" class="store-tag">v{{ version }}</span><span v-if="installed || done" class="store-tag">已安装</span></div>
    <h4>示例提问</h4><ul><li v-for="example in (connector.examples.zh?.length ? connector.examples.zh : connector.examples.en)" :key="example">{{ example }}</li></ul>
    <h4>需要填写的令牌字段</h4>
    <ul v-if="connector.tokenFields.length"><li v-for="field in connector.tokenFields" :key="field.server + ':' + field.key">{{ field.label || field.key }} · {{ field.required ? '必填' : '选填' }}</li></ul><p v-else>无需填写令牌字段。</p>
    <h4>传输方式</h4><p>{{ connector.transports.join('、') }}</p>
    <p>被策略禁用的工具：{{ connector.disabledToolCount }} 个</p>
    <h4>随包技能 · {{ connector.skillCount }} 个</h4><ul><li v-for="name in connector.skills" :key="name">{{ name }}</li></ul>
    <NAlert v-if="localError" type="warning" :show-icon="false">{{ localError }}</NAlert>
    <NAlert v-if="error" type="error" :show-icon="false">{{ error }}</NAlert>
    <p v-if="loading" class="review-loading">正在读取固定版本的文件与脚本…</p>
    <template v-if="review">
      <h4>安装审查 · v{{ review.manifest.version }}</h4>
      <h4>MCP 服务 · {{ review.manifest.connectorCount }} 个</h4>
      <ul class="review-files"><li v-for="server in review.manifest.servers" :key="server.name">
        <div>{{ server.name }} · {{ server.transport }}</div>
        <p v-if="server.url">地址：<code>{{ server.url }}</code></p>
        <p v-if="server.command">将运行的命令：<code>{{ [server.command, ...(server.args ?? [])].join(' ') }}</code></p>
        <p>环境变量名：{{ server.envKeys?.join('、') || '无' }}</p><p>请求头名：{{ server.headerKeys?.join('、') || '无' }}</p>
      </li></ul>
      <NAlert v-for="warning in warnings" :key="warning" type="warning" :show-icon="false">{{ warning }}</NAlert>
      <h4>审查包内技能</h4><ul><li v-for="name in review.manifest.skills" :key="name">{{ name }}</li></ul>
      <h4>文件清单</h4><ul class="review-files"><li v-for="file in review.manifest.files" :key="file.path"><div><code>{{ file.path }}</code><span>{{ file.executableScript ? '脚本' : '' }}</span></div><small>SHA256 {{ file.sha256 }}</small></li></ul>
      <p class="version-lock">版本锁定 · 清单摘要 <code>{{ review.sha256 }}</code></p>
      <details v-for="script in review.scripts" :key="script.path" class="script-preview"><summary>{{ script.path }} · 脚本预览{{ script.previewTruncated ? '（已截断）' : '' }}</summary><pre class="review-content">{{ script.preview || '此脚本暂无文本预览，请在确认来源后再安装。' }}</pre></details>
      <p class="detail-note">确认后安装此版本，并在当前窗口完成配置和授权。远程服务将尝试连接；本地命令会展示运行入口，点击“运行并连接”后才会启动。文件校验不代表安全认证。</p>
      <label class="connect-option"><input v-model="connectOnInstall" type="checkbox" :disabled="busy" />安装后连接远程服务（需要登录时打开浏览器）</label>
    </template>
    <template v-if="done">
      <p role="status">{{ allConnected ? '已安装并连接，可以使用了' : services.length ? '已安装，在此完成连接即可使用' : '已安装' }}</p>
      <section v-for="service in services" :key="service.id"><h4 v-if="services.length > 1">{{ service.name }}</h4><ConnectorConnection v-if="installationFinished" :connector="service" :auto-connect="connectOnInstall" @updated="updated" @connected="allConnected && emit('connected')" @busy="serviceBusy(service.id, $event)" /></section>
      <button v-if="error && !services.length" class="xb-btn" @click="loadServices">重试读取连接配置</button>
    </template>
    <p v-else-if="!namespace || !targetVersion" class="detail-note">此连接器暂未提供可安装的版本或市场来源。</p>
    <div class="dialog-actions">
      <button v-if="installed || done" class="store-secondary" :disabled="busy" @click="emit('center')">去连接器中心</button>
      <button v-if="!done && (!installed || updateVersion)" class="store-primary" :disabled="busy || loading || !canInstall" @click="review ? install() : inspect()">{{ busy ? '正在安装…' : loading ? '正在审查…' : review ? connectOnInstall ? '安装并连接' : '安装' : updateVersion ? '更新' : '安装' }}</button>
    </div>
  </div>
</template>

<style scoped>
p,li{font-size:13px;line-height:1.7;overflow-wrap:anywhere}
</style>
