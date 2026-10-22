<script setup lang="ts">
import { computed, onDeactivated, onMounted, onBeforeUnmount, ref, watch } from 'vue';
import { NAlert, NButton, NInput, NModal, NPopconfirm, NSelect, NSwitch, NTag } from 'naive-ui';
import { Bot, Cpu, ExternalLink, FolderOpen, Globe, Plus, RefreshCw, Settings, ShieldCheck, Sparkles, Trash2, Wrench } from 'lucide-vue-next';
import { GATEWAY_PROVIDER, invoke, listen, type AiConfigView, type AiProtocol, type AiReplyPreference, type AiReasoningEffort, type ProviderConfigView } from '../../bridge.ts';
import AiGatewayCard from './AiGatewayCard.vue';
import type { ControlStore } from '../control-types.ts';
import {
  aiProtocols,
  BUILTIN_PROVIDERS,
  endpointPreview,
  getProviderMeta as fallbackMeta,
  remoteProviderMeta,
  insecureRemote,
  normalizeBaseUrl,
  protocolOption,
  reasoningLevels,
  reasoningEfforts,
  preferenceOptions,
  providerEndpoint,
  validateAiForm,
  validateBaseUrl,
  validateKey,
  type ProviderMeta,
} from '../ai-config.ts';

const props = defineProps<{ store: ControlStore }>();


const blank: AiConfigView = {
  activeProvider: '',
  providers: [],
  protocol: 'responses',
  baseUrl: '',
  model: '',
  hasKey: false,
  keyHint: '',
  workspace: '',
  defaultWorkspace: '',
  hideToolCalls: false,
  openComputerUse: false,
};

const saved = ref<AiConfigView>({ ...blank });
const busy = ref('');
const loadError = ref('');

// Global preferences state
const workspace = ref('');
const hideToolCalls = ref(false);
const openComputerUse = ref(false);


// Modal state
const modalVisible = ref(false);
const editingId = ref<string>('deepseek');
const modalName = ref('');
const modalProtocol = ref<AiProtocol>('responses');
const modalBaseUrl = ref('');
const modalModel = ref('');
const modalApiKey = ref('');
const modalSelectionMode = ref<'manual' | 'recommended'>('manual');
const modalReplyPreference = ref<AiReplyPreference>('default');
const modalReasoningMode = ref<'auto' | 'manual' | 'off'>('auto');
const modalReasoningLevels = ref<AiReasoningEffort[]>(['low', 'high', 'max']);
watch([modalModel, modalProtocol], () => {
  if (modalVisible.value) { modalReasoningMode.value = 'auto'; modalReplyPreference.value = 'default'; }
}, { flush: 'sync' });
const modalHasKey = ref(false);
const modalKeyHint = ref('');
const confirmingClear = ref(false);
type DiscoveredModel = NonNullable<ProviderConfigView['models']>[number];
const fetchedModels = ref<DiscoveredModel[]>([]);
const fetchedFor = ref('');
watch([modalBaseUrl, modalProtocol], () => {
  if (modalVisible.value) {
    fetchedModels.value = [];
    fetchedFor.value = '';
  }
}, { flush: 'sync' });

const isEditingCustom = computed(() => {
  return getProviderSaved(editingId.value)?.isCustom ?? !catalogueProviders.value.some(p => p.id === editingId.value);
});

const catalogueProviders = computed<ProviderMeta[]>(() => {
  if (!saved.value.catalogueReceived) return BUILTIN_PROVIDERS.filter(p => p.id !== 'custom');
  return (saved.value.providers ?? []).filter(p => !p.isCustom && p.available !== false).map(remoteProviderMeta);
});
function getProviderMeta(id: string): ProviderMeta {
  const p = getProviderSaved(id);
  return p?.remote ? remoteProviderMeta(p) : fallbackMeta(id);
}
const isManaged = computed(() => ['free', 'paid'].includes(getProviderSaved(editingId.value)?.type ?? ''));
const currentMeta = computed<ProviderMeta>(() => getProviderMeta(editingId.value));
const modalLevels = computed(() => reasoningLevels(modalProtocol.value, modalModel.value,
  modalReasoningMode.value === 'off' ? [] : modalReasoningMode.value === 'manual' ? modalReasoningLevels.value
    : currentMeta.value.models?.find(m => m.id === modalModel.value)?.reasoningEfforts));
const canSetReplyPreference = computed(() => modalLevels.value.length > 0);
const replyPreferenceOptions = computed(() => preferenceOptions(modalLevels.value));
watch(replyPreferenceOptions, options => { if (!options.some(o => o.value === modalReplyPreference.value)) modalReplyPreference.value = 'default'; });
const selectedModelDescription = computed(() => currentMeta.value.models?.find(m => m.id === modalModel.value)?.description ?? '');
const modalOption = computed(() => protocolOption(modalProtocol.value));
const modalPreview = computed(() => validateBaseUrl(modalBaseUrl.value) ? '' : endpointPreview(modalProtocol.value, modalBaseUrl.value));

/** True when the address is one of this provider's own presets, so switching protocol may replace it. */
function isPresetAddress(id: string, url: string): boolean {
  const trimmed = normalizeBaseUrl(url);
  if (!trimmed) return true;
  const meta = getProviderMeta(id);
  return Object.values(meta.endpoints).some(endpoint => normalizeBaseUrl(endpoint ?? '') === trimmed);
}

/**
 * Each built-in vendor serves all three wire formats, but on a different path, so following the
 * protocol means following the address. A hand-edited address (a proxy, a gateway) is never
 * replaced; for a custom provider the address is simply the protocol's own example.
 */
function onProtocolChange(newProto: AiProtocol) {
  const previous = modalProtocol.value;
  modalProtocol.value = newProto;
  if (isEditingCustom.value) {
    const newOption = protocolOption(newProto);
    if (isPresetAddress(editingId.value, modalBaseUrl.value) || normalizeBaseUrl(modalBaseUrl.value) === normalizeBaseUrl(protocolOption(previous).placeholder)) {
      modalBaseUrl.value = newOption.placeholder;
    }
    return;
  }
  if (isPresetAddress(editingId.value, modalBaseUrl.value)) {
    modalBaseUrl.value = providerEndpoint(editingId.value, newProto);
  }
}

/** The protocol picker. Every built-in vendor serves all three formats, so all of them are listed;
 * a hand-typed address stays valid because the host is told the protocol, not the vendor. */
const protocolChoices = computed(() => aiProtocols.map(o => ({ label: o.label, value: o.value })));

const activeProviderMeta = computed(() => {
  const activeId = saved.value.activeProvider || 'deepseek';
  const custom = saved.value.providers?.find(p => p.id === activeId);
  if (custom && custom.name) {
    return { name: custom.name, desc: custom.baseUrl };
  }
  return getProviderMeta(activeId);
});

// The site can close the 小伴 AI entry; a settings file from before that existed has no flag, which means open.
const gatewayOffered = computed(() => saved.value.gatewayEnabled !== false);

const activeStatusText = computed(() => {
  const activeId = saved.value.activeProvider;
  if (activeId === GATEWAY_PROVIDER) {
    return gatewayOffered.value
      ? '当前生效：小伴 AI（点“小伴 AI”卡片的“配置”可查看额度、更换模式）'
      : '小伴 AI 入口已被关闭，请选择其他服务商';
  }
  if (!activeId || !isProviderFullyConfigured(activeId)) {
    return '尚未配置可用服务商，配置好一个后会自动启用';
  }
  return `当前生效：${activeProviderMeta.value.name} · ${saved.value.model || '未设模型'}${saved.value.hasKey ? ' · 已配置密钥' : ''}`;
});

// User's custom providers
const customProviders = computed<ProviderConfigView[]>(() => {
  return (saved.value.providers || []).filter(p => p.isCustom);
});

// Dropdown options for models in modal
const allModelOptions = computed(() => {
  const set = new Set<string>();
  fetchedModels.value.forEach(m => set.add(m.id));
  currentMeta.value.suggestedModels.forEach(m => set.add(m));
  return Array.from(set).map(m => ({
    label: fetchedModels.value.find(v => v.id === m)?.name
      || currentMeta.value.models?.find(v => v.id === m)?.name || m,
    value: m,
  }));
});

const modalFormError = computed(() => {
  if (isEditingCustom.value && !modalName.value.trim()) return '请填写自定义供应商名称';
  return validateAiForm({ baseUrl: modalBaseUrl.value, model: modalModel.value }) || validateKey(modalApiKey.value);
});

const modalKeyPlaceholder = computed(() => {
  if (modalHasKey.value) return `已保存 ${modalKeyHint.value}，留空表示不修改`;
  return '没有密钥（本地模型服务可以不填）';
});

const workspaceChanged = computed(() => workspace.value !== saved.value.workspace);
const shownWorkspace = computed(() => workspace.value || saved.value.defaultWorkspace || '桌面应用中可查看');
const prefsDirty = computed(() => {
  return (
    workspaceChanged.value ||
    hideToolCalls.value !== saved.value.hideToolCalls ||
    openComputerUse.value !== saved.value.openComputerUse
  );
});

function getProviderSaved(id: string): ProviderConfigView | undefined {
  return saved.value.providers?.find(p => p.id === id);
}

function isProviderActive(id: string): boolean {
  if (!saved.value.activeProvider) return false;
  return saved.value.activeProvider === id;
}

// 没有配置 API Key 的，不算已配置，显示为待配置；官方免费服务(free)或自定义本地服务允许免密
function isProviderFullyConfigured(id: string): boolean {
  const p = getProviderSaved(id);
  if (!p) return false;
  return p.available !== false && Boolean(p.baseUrl && p.model && (p.hasKey || p.isCustom || p.type === 'free'));
}

// 真正正在使用中的服务商：必须是 activeProvider 且已经配置了密钥（或是自定义服务）
function isProviderInUse(id: string): boolean {
  return isProviderActive(id) && isProviderFullyConfigured(id);
}

// 当前没有可用服务商：此时保存任何配置完整的一个，主机都会直接把它设为生效，用户无需再点启用。
const noActiveProvider = computed(() => {
  const id = saved.value.activeProvider;
  if (id === GATEWAY_PROVIDER) return !gatewayOffered.value;
  return !id || !isProviderFullyConfigured(id);
});

function fill(view: AiConfigView) {
  saved.value = view;
  workspace.value = view.workspace;
  hideToolCalls.value = view.hideToolCalls ?? false;
  openComputerUse.value = view.openComputerUse ?? false;
}

let unlisten: (() => void) | undefined;
onBeforeUnmount(() => unlisten?.());
onDeactivated(() => { modalApiKey.value = ''; modalVisible.value = false; });
async function refreshCatalogue() {
  if (!props.store.desktop || busy.value) return;
  busy.value = 'catalogue';
  try { fill(await invoke<AiConfigView>('refresh_ai_providers', { force: true })); loadError.value = ''; }
  catch (e) { loadError.value = String(e instanceof Error ? e.message : e); }
  finally { busy.value = ''; }
}
onMounted(async () => {
  if (!props.store.desktop) return;
  try {
    fill(await invoke<AiConfigView>('get_ai_config'));
    unlisten = await listen<AiConfigView>('ai-config-changed', fill);
    void invoke<AiConfigView>('refresh_ai_providers', { force: false }).then(fill).catch(() => {});
  } catch (error) {
    loadError.value = String(error instanceof Error ? error.message : error);
  }
});

function openConfigureModal(id: string) {
  editingId.value = id;
  const meta = getProviderMeta(id);
  const p = getProviderSaved(id);

  if (p) {
    modalName.value = p.name || meta.name;
    modalProtocol.value = p.protocol;
    modalBaseUrl.value = p.baseUrl;
    modalModel.value = p.model;
    modalHasKey.value = p.hasKey;
    modalKeyHint.value = p.keyHint;
  } else {
    modalName.value = meta.name;
    modalProtocol.value = meta.protocol;
    modalBaseUrl.value = providerEndpoint(id, meta.protocol);
    modalModel.value = meta.defaultModel;
    modalHasKey.value = false;
    modalKeyHint.value = '';
  }

  modalReasoningMode.value = p?.reasoningEfforts == null ? 'auto' : p.reasoningEfforts.length ? 'manual' : 'off';
  modalReasoningLevels.value = p?.reasoningEfforts?.length ? [...p.reasoningEfforts] : ['low', 'high', 'max'];
  modalSelectionMode.value = p?.selectionMode ?? 'manual';
  modalReplyPreference.value = p?.replyPreference ?? 'default';
  modalApiKey.value = '';
  confirmingClear.value = false;
  fetchedModels.value = [];
  fetchedFor.value = '';
  modalVisible.value = true;
}

function openCreateCustomModal() {
  modalReasoningMode.value = 'auto';
  modalReasoningLevels.value = ['low', 'high', 'max'];
  modalSelectionMode.value = 'manual';
  modalReplyPreference.value = 'default';
  const newId = `custom_${Date.now()}`;
  editingId.value = newId;
  modalName.value = '自定义供应商';
  modalProtocol.value = 'responses';
  modalBaseUrl.value = '';
  modalModel.value = '';
  modalApiKey.value = '';
  modalHasKey.value = false;
  modalKeyHint.value = '';
  confirmingClear.value = false;
  fetchedModels.value = [];
  fetchedFor.value = '';
  modalVisible.value = true;
}

async function activateProvider(id: string) {
  if (!props.store.desktop) return;
  const p = getProviderSaved(id);
  if (!p || p.available === false || !p.baseUrl || !p.model || (!p.hasKey && !p.isCustom && p.type !== 'free')) {
    props.store.notify('该服务商尚未完整配置（缺少密钥或地址），请先配置', 'info');
    openConfigureModal(id);
    return;
  }
  busy.value = `activate-${id}`;
  try {
    const updated = await invoke<AiConfigView>('switch_ai_provider', { providerId: id });
    fill(updated);
    props.store.notify(`已切换至 ${p.name || getProviderMeta(id).name}`, 'success');
  } catch (error) {
    props.store.notify(String(error instanceof Error ? error.message : error), 'error');
  } finally {
    busy.value = '';
  }
}

async function saveProvider(activate: boolean) {
  if (!props.store.desktop) return;
  if (modalFormError.value) {
    props.store.notify(modalFormError.value, 'error');
    return;
  }
  busy.value = 'save-provider';
  // 保存前是否已在生效；用于区分“本次保存让它生效”和“它本来就在用”。
  const wasInUse = isProviderInUse(editingId.value);
  try {
    const updated = await invoke<AiConfigView>('save_ai_provider', {
      providerId: editingId.value,
      name: modalName.value.trim() || null,
      protocol: modalProtocol.value,
      baseUrl: modalBaseUrl.value.trim(),
      model: modalModel.value.trim(),
      apiKey: modalApiKey.value.trim() || null,
      clearKey: false,
      activate,
      selectionMode: modalSelectionMode.value,
      ...(!isManaged.value ? { reasoningEfforts: modalReasoningMode.value === 'manual' ? modalReasoningLevels.value : modalReasoningMode.value === 'off' ? [] : null, clearReasoningEfforts: modalReasoningMode.value === 'auto' } : {}),
      replyPreference: canSetReplyPreference.value ? modalReplyPreference.value : 'default',
      discoveredModel: fetchedFor.value === `${modalProtocol.value}\n${modalBaseUrl.value.trim()}`
        ? fetchedModels.value.find(m => m.id === modalModel.value.trim()) ?? null
        : null,
    });
    fill(updated);
    modalVisible.value = false;
    const displayName = modalName.value.trim() || currentMeta.value.name;
    // 没有可用服务商时主机自动启用刚保存的这个，按实际结果提示，而不是按按钮字面。
    const becameActive = !wasInUse && updated.activeProvider === editingId.value;
    props.store.notify(activate || becameActive ? `已保存并启用 ${displayName}` : `${displayName} 配置已保存`, 'success');
  } catch (error) {
    props.store.notify(String(error instanceof Error ? error.message : error), 'error');
  } finally {
    busy.value = '';
  }
}

async function deleteCustomProvider(id: string) {
  if (!props.store.desktop) return;
  busy.value = `delete-${id}`;
  try {
    const updated = await invoke<AiConfigView>('delete_ai_provider', { providerId: id });
    fill(updated);
    props.store.notify('已删除自定义服务商', 'success');
  } catch (error) {
    props.store.notify(String(error instanceof Error ? error.message : error), 'error');
  } finally {
    busy.value = '';
  }
}

async function clearProviderKey() {
  if (!props.store.desktop) return;
  busy.value = 'clear-key';
  try {
    const updated = await invoke<AiConfigView>('save_ai_provider', {
      providerId: editingId.value,
      name: modalName.value.trim() || null,
      protocol: modalProtocol.value,
      baseUrl: modalBaseUrl.value.trim(),
      model: modalModel.value.trim(),
      apiKey: null,
      clearKey: true,
      activate: isProviderActive(editingId.value),
    });
    fill(updated);
    modalHasKey.value = false;
    modalKeyHint.value = '';
    modalApiKey.value = '';
    confirmingClear.value = false;
    props.store.notify('已删除保存的密钥', 'success');
  } catch (error) {
    props.store.notify(String(error instanceof Error ? error.message : error), 'error');
  } finally {
    busy.value = '';
  }
}

async function fetchModels() {
  if (!props.store.desktop) return;
  const cleanUrl = modalBaseUrl.value.trim();
  if (!cleanUrl) {
    props.store.notify('请先填写 API 地址', 'info');
    return;
  }
  const err = validateBaseUrl(cleanUrl);
  if (err) {
    props.store.notify(err, 'info');
    return;
  }
  busy.value = 'models';
  const requestedProtocol = modalProtocol.value;
  const requestedProvider = editingId.value;
  try {
    const list = await invoke<DiscoveredModel[]>('fetch_ai_models', {
      protocol: requestedProtocol,
      baseUrl: cleanUrl,
      apiKey: modalApiKey.value.trim() || null,
    });
    if (!modalVisible.value || editingId.value !== requestedProvider
        || modalBaseUrl.value.trim() !== cleanUrl || modalProtocol.value !== requestedProtocol) {
      return; // A stale async response must never attach capabilities to another endpoint.
    }
    if (list && list.length > 0) {
      fetchedModels.value = list;
      fetchedFor.value = `${requestedProtocol}\n${cleanUrl}`;
      props.store.notify(`成功获取到 ${list.length} 个模型`, 'success');
    } else {
      props.store.notify('未获取到可用模型', 'info');
    }
  } catch (error) {
    props.store.notify(String(error instanceof Error ? error.message : error), 'error');
  } finally {
    busy.value = '';
  }
}

async function savePreferences() {
  if (!props.store.desktop) return;
  busy.value = 'save-prefs';
  try {
    const updated = await invoke<AiConfigView>('save_ai_preferences', {
      workspace: workspaceChanged.value ? workspace.value : null,
      hideToolCalls: hideToolCalls.value,
      openComputerUse: openComputerUse.value,
    });
    fill(updated);
    props.store.notify('通用设置已保存', 'success');
  } catch (error) {
    props.store.notify(String(error instanceof Error ? error.message : error), 'error');
  } finally {
    busy.value = '';
  }
}

async function chooseFolder() {
  busy.value = 'folder';
  try {
    const chosen = await invoke<string | null>('pick_workspace');
    if (chosen) workspace.value = chosen;
  } catch (error) {
    props.store.notify(error, 'error');
  } finally {
    busy.value = '';
  }
}

function openExternal(url?: string) {
  if (!url) return;
  invoke('open_download_url', { url }).catch(() => {});
}

function getProviderIcon(id: string) {
  switch (id) {
    case 'deepseek': return Sparkles;
    case 'zhipu': return Cpu;
    case 'kimi': return Globe;
    case 'minimax': return Bot;
    default: return Wrench;
  }
}
</script>

<template>
  <section class="surface ai-config" data-testid="ai-config-card">
    <div class="section-heading">
      <div>
        <h2>AI 模型服务商</h2>
        <p data-testid="ai-status">{{ activeStatusText }}</p>
      </div>
      <NButton size="small" secondary :disabled="!store.desktop || !!busy" :loading="busy === 'catalogue'" @click="refreshCatalogue">刷新供应商</NButton>
    </div>

    <NAlert v-if="!store.desktop" type="info" :show-icon="false">请在桌面应用中配置模型服务。</NAlert>
    <NAlert v-if="loadError" type="warning" :show-icon="false" data-testid="ai-load-error">
      {{ loadError }}。可以稍后刷新，已有配置会保留。
    </NAlert>

    <!-- Providers Grid Cards -->
    <div class="providers-section">
      <div class="providers-grid">
        <!-- 小伴 AI is its own kind of provider: account login and credit instead of an address and key -->
        <AiGatewayCard v-if="gatewayOffered" :store="store" :config="saved" @config="fill" />

        <!-- 4 Built-in templates -->
        <article
          v-for="p in catalogueProviders"
          :key="p.id"
          class="provider-card"
          :class="{ active: isProviderActive(p.id) }"
          :data-testid="`ai-provider-card-${p.id}`"
        >
          <div class="card-top">
            <div class="provider-badge">
              <component :is="getProviderIcon(p.id)" :size="20" />
              <div class="provider-title">
                <h3>{{ p.name }}</h3>
                <span class="provider-sub">{{ p.type === 'free' ? '免费服务' : p.type === 'paid' ? '付费服务' : '第三方供应商' }}</span>
              </div>
            </div>
            <div class="status-tags">
              <NTag v-if="isProviderInUse(p.id)" type="success" size="small" :bordered="false">当前使用</NTag>
              <NTag v-else-if="isProviderFullyConfigured(p.id)" type="info" size="small" :bordered="false">已配置</NTag>
              <NTag v-else type="warning" size="small" :bordered="false">待配置</NTag>
            </div>
          </div>

          <p class="provider-desc">{{ p.desc }}</p>

          <div class="provider-meta-row">
            <span class="meta-label">生效模型：</span>
            <code class="meta-model">{{ getProviderSaved(p.id)?.model || p.defaultModel || '暂未设置' }}</code>
          </div>

          <div class="card-actions">
            <NButton
              size="small"
              secondary
              :disabled="!store.desktop || !!busy"
              :data-testid="`ai-provider-config-${p.id}`"
              @click="openConfigureModal(p.id)"
            >
              <template #icon><Settings :size="13" /></template>
              配置
            </NButton>
            <NButton
              v-if="!isProviderInUse(p.id)"
              size="small"
              type="primary"
              secondary
              :loading="busy === `activate-${p.id}`"
              :disabled="!store.desktop || !!busy"
              :data-testid="`ai-provider-activate-${p.id}`"
              @click="activateProvider(p.id)"
            >
              启用此服务商
            </NButton>
            <NButton v-else size="small" disabled quaternary type="success">
              正在使用中
            </NButton>
          </div>
        </article>

        <!-- User configured custom providers -->
        <article
          v-for="cp in customProviders"
          :key="cp.id"
          class="provider-card"
          :class="{ active: isProviderInUse(cp.id) }"
          :data-testid="`ai-provider-card-${cp.id}`"
        >
          <div class="card-top">
            <div class="provider-badge">
              <component :is="getProviderIcon(cp.id)" :size="20" />
              <div class="provider-title">
                <h3>{{ cp.name || '自定义供应商' }}</h3>
                <span class="provider-sub">自定义端点</span>
              </div>
            </div>
            <div class="status-tags">
              <NTag v-if="isProviderInUse(cp.id)" type="success" size="small" :bordered="false">当前使用</NTag>
              <NTag v-else-if="isProviderFullyConfigured(cp.id)" type="info" size="small" :bordered="false">已配置</NTag>
              <NTag v-else type="warning" size="small" :bordered="false">待配置</NTag>
            </div>
          </div>

          <p class="provider-desc">{{ cp.baseUrl || '尚未填写 API 端点地址' }}</p>

          <div class="provider-meta-row">
            <span class="meta-label">生效模型：</span>
            <code class="meta-model">{{ cp.model || '暂未设置' }}</code>
          </div>

          <div class="card-actions">
            <NButton
              size="small"
              secondary
              :disabled="!store.desktop || !!busy"
              :data-testid="`ai-provider-config-${cp.id}`"
              @click="openConfigureModal(cp.id)"
            >
              <template #icon><Settings :size="13" /></template>
              配置
            </NButton>
            <NButton
              v-if="!isProviderInUse(cp.id)"
              size="small"
              type="primary"
              secondary
              :loading="busy === `activate-${cp.id}`"
              :disabled="!store.desktop || !!busy"
              :data-testid="`ai-provider-activate-${cp.id}`"
              @click="activateProvider(cp.id)"
            >
              启用
            </NButton>
            <NButton v-else size="small" disabled quaternary type="success">
              正在使用中
            </NButton>
            <NPopconfirm
              v-if="!isProviderActive(cp.id)"
              @positive-click="deleteCustomProvider(cp.id)"
            >
              <template #trigger>
                <NButton size="small" quaternary type="error" :disabled="!store.desktop || !!busy">
                  <template #icon><Trash2 :size="13" /></template>
                </NButton>
              </template>
              确定要删除该自定义供应商吗？
            </NPopconfirm>
          </div>
        </article>

        <!-- Card to add a new custom provider -->
        <article
          class="provider-card add-custom-card"
          data-testid="ai-add-custom-card"
          @click="openCreateCustomModal"
        >
          <div class="add-custom-content">
            <div class="add-icon-circle"><Plus :size="22" /></div>
            <h3>添加自定义供应商</h3>
            <p>支持配置第三方兼容网关、中转站或本地 Ollama 模型服务</p>
          </div>
        </article>
      </div>
    </div>

    <!-- Details Modal -->
    <NModal
      v-model:show="modalVisible"
      preset="card"
      :title="isEditingCustom ? `配置自定义服务商 · ${modalName || '自定义'}` : `配置服务商 · ${currentMeta.name}`"
      style="max-width: 620px; width: 92vw; max-height: min(480px, 80vh); display: flex; flex-direction: column; overflow: hidden;"
      :header-style="{ flexShrink: 0 }"
      :content-style="{ minHeight: 0, overflowY: 'auto', overscrollBehavior: 'contain' }"
      :footer-style="{ flexShrink: 0 }"
      :mask-closable="false"
      data-testid="ai-provider-modal"
    >
      <div class="modal-form">
        <NAlert v-if="currentMeta.consoleUrl" type="info" :show-icon="false" class="provider-console-tip">
          <div class="tip-content">
            <span>{{ currentMeta.desc }}</span>
            <NButton text type="primary" size="small" @click="openExternal(currentMeta.consoleUrl)">
              <template #icon><ExternalLink :size="13" /></template>
              {{ currentMeta.consoleText || '前往获取 API Key' }}
            </NButton>
          </div>
        </NAlert>

        <!-- Custom provider name field -->
        <div v-if="isEditingCustom" class="field">
          <label for="ai-provider-name">供应商名称</label>
          <NInput
            v-model:value="modalName"
            placeholder="例如：公司内部大模型、本地 Ollama"
            :disabled="!store.desktop || !!busy"
            data-testid="ai-provider-name"
            :input-props="{ id: 'ai-provider-name', autocomplete: 'off', spellcheck: false }"
          />
          <small>自定义该服务商的展示名称，以便在卡片中快速区分。</small>
        </div>

        <div class="field">
          <label for="ai-protocol">接口协议</label>
          <NSelect
            :value="modalProtocol"
            input-id="ai-protocol"
            :options="protocolChoices"
            :disabled="!store.desktop || !!busy || isManaged"
            data-testid="ai-protocol"
            @update:value="onProtocolChange"
          />
          <small>{{ modalOption.hint }}</small>
          <small v-if="!isEditingCustom" class="protocol-switch-hint" data-testid="ai-protocol-endpoint-hint">
            切换协议会自动换成该服务商对应的接口地址；手动改过的地址不会被覆盖。
          </small>
        </div>

        <div class="field">
          <label for="ai-base-url">API 地址</label>
          <NInput
            v-model:value="modalBaseUrl"
            :placeholder="modalOption.placeholder"
            :disabled="!store.desktop || !!busy || isManaged"
            data-testid="ai-base-url"
            :input-props="{ id: 'ai-base-url', autocomplete: 'off', spellcheck: false }"
          />
          <small v-if="modalPreview" data-testid="ai-endpoint-preview">实际请求端点：<code>{{ modalPreview }}</code></small>
          <NAlert v-if="insecureRemote(modalBaseUrl)" type="warning" :show-icon="false" data-testid="ai-http-warning">
            http 地址不加密，密钥和内容会明文传输，建议仅用于本机或可信内网。
          </NAlert>
        </div>

        <div v-if="!isManaged" class="field">
          <label for="ai-api-key">API 密钥</label>
          <NInput
            v-model:value="modalApiKey"
            type="password"
            show-password-on="click"
            :placeholder="modalKeyPlaceholder"
            :disabled="!store.desktop || !!busy"
            data-testid="ai-api-key"
            :input-props="{ id: 'ai-api-key', autocomplete: 'off', spellcheck: false }"
          />
          <small v-if="modalHasKey" class="key-actions">
            <template v-if="!confirmingClear">
              <button type="button" class="link-button" data-testid="ai-clear-key" :disabled="!store.desktop || !!busy" @click="confirmingClear = true">
                <Trash2 :size="13" />删除已保存的密钥
              </button>
            </template>
            <template v-else>
              删除后需重新输入密钥。
              <button type="button" class="link-button danger" data-testid="ai-clear-confirm" :disabled="!!busy" @click="clearProviderKey">确认删除</button>
              <button type="button" class="link-button" data-testid="ai-clear-cancel" :disabled="!!busy" @click="confirmingClear = false">取消</button>
            </template>
          </small>
        </div>

        <div class="field">
          <div class="field-header">
            <label for="ai-model">模型名称</label>
            <NButton
              size="tiny"
              secondary
              type="primary"
              :loading="busy === 'models'"
              :disabled="!store.desktop || !!busy || isManaged || !modalBaseUrl.trim()"
              data-testid="ai-fetch-models"
              @click="fetchModels"
            >
              <template #icon><RefreshCw :size="13" /></template>
              获取模型列表
            </NButton>
          </div>
          <NInput
            v-model:value="modalModel"
            @update:value="modalSelectionMode = 'manual'"
            placeholder="输入或从下方选择模型"
            :disabled="!store.desktop || !!busy"
            data-testid="ai-model"
            :input-props="{ id: 'ai-model', autocomplete: 'off', spellcheck: false }"
          />
          <div v-if="allModelOptions.length > 0" class="model-select-row" data-testid="ai-model-select-wrapper">
            <small>常用或拉取到的可用模型：</small>
            <NSelect
              v-model:value="modalModel"
              @update:value="modalSelectionMode = 'manual'"
              :options="allModelOptions"
              filterable
              placeholder="搜索并选择模型..."
              size="small"
              data-testid="ai-model-select"
            />
          </div>
        </div>

        <details v-if="modalVisible && (!isManaged || canSetReplyPreference)" class="reply-preference" data-testid="ai-reply-advanced">
          <summary>高级设置</summary>
          <div v-if="canSetReplyPreference" class="field">
            <label for="ai-reply-preference">思考等级</label>
            <NSelect
              v-model:value="modalReplyPreference"
              input-id="ai-reply-preference"
              :options="replyPreferenceOptions"
              :disabled="!store.desktop || !!busy"
              data-testid="ai-reply-preference"
            />
            <small>默认沿用服务商设置。更深入可能耗时更久、消耗更多额度。</small>
          </div>
          <div v-if="!isManaged && modalProtocol !== 'anthropic'" class="field">
            <label for="ai-reasoning-mode">模型支持的思考等级</label>
            <NSelect v-model:value="modalReasoningMode" input-id="ai-reasoning-mode" :options="[{label:'自动识别',value:'auto'},{label:'手动声明',value:'manual'},{label:'不支持',value:'off'}]" data-testid="ai-reasoning-mode" />
            <NSelect v-if="modalReasoningMode === 'manual'" v-model:value="modalReasoningLevels" multiple :options="reasoningEfforts.map(value => ({value,label:value}))" data-testid="ai-reasoning-levels" />
            <small>代理地址同样支持自动识别；模型使用别名时，可按服务商文档声明支持的等级。</small>
          </div>
        </details>

        <small v-if="selectedModelDescription">{{ selectedModelDescription }}</small>
        <label v-if="currentMeta.defaultModel && !isEditingCustom" class="field">
          <span>跟随推荐模型 · {{ currentMeta.defaultModel }}</span>
          <NSwitch :value="modalSelectionMode === 'recommended'" @update:value="value => { modalSelectionMode = value ? 'recommended' : 'manual'; if (value) modalModel = currentMeta.defaultModel; }" />
          <small>手动选择的模型不会被推荐更新覆盖。</small>
        </label>
        <NAlert v-if="modalFormError" type="error" :show-icon="false" class="form-error" data-testid="ai-form-error">
          {{ modalFormError }}
        </NAlert>
      </div>

      <template #footer>
        <div class="modal-actions">
          <NButton :disabled="!!busy" @click="modalVisible = false">取消</NButton>
          <NButton
            v-if="!noActiveProvider"
            secondary
            :disabled="!store.desktop || !!modalFormError || !!busy"
            :loading="busy === 'save-provider'"
            data-testid="ai-save"
            @click="saveProvider(false)"
          >
            保存配置
          </NButton>
          <NButton
            type="primary"
            :disabled="!store.desktop || !!modalFormError || !!busy"
            :loading="busy === 'save-provider'"
            data-testid="ai-save-and-activate"
            @click="saveProvider(true)"
          >
            保存并启用
          </NButton>
        </div>
      </template>
    </NModal>

    <!-- Global Preferences Section -->
    <div class="preferences-divider">
      <h3>全局工具与运行设置</h3>
      <p class="muted">影响所有服务商的通用工具、电脑操作与工作目录偏好</p>
    </div>

    <div class="preferences-box">
      <div class="field">
        <div class="setting-row">
          <div>
            <b>隐藏工具调用信息</b>
            <p>在会话中不展示工具调用的中间卡片细节，界面更清爽简洁（不影响工具正常执行）。</p>
          </div>
          <NSwitch v-model:value="hideToolCalls" :disabled="!store.desktop || !!busy" data-testid="ai-hide-tool-calls" />
        </div>
      </div>

      <div class="field">
        <div class="setting-row">
          <div>
            <b>启用电脑屏幕与应用控制（open-computer-use）</b>
            <p>允许小伴操控电脑应用（点击、键入文字等）。macOS 首次使用需授予辅助功能权限。</p>
          </div>
          <NSwitch v-model:value="openComputerUse" :disabled="!store.desktop || !!busy" data-testid="ai-open-computer-use" />
        </div>
      </div>

      <div class="field workspace-field">
        <label>工作目录</label>
        <div class="workspace-row">
          <code data-testid="ai-workspace">{{ shownWorkspace }}</code>
          <NButton :disabled="!store.desktop || !!busy" data-testid="ai-workspace-pick" @click="chooseFolder">
            <template #icon><FolderOpen :size="15" /></template>
            选择文件夹
          </NButton>
          <NButton v-if="workspace" quaternary :disabled="!!busy" data-testid="ai-workspace-reset" @click="workspace = ''">
            恢复默认
          </NButton>
        </div>
        <small>小伴的文件操作仅在此目录中进行。运行命令以当前用户权限执行，请谨慎授权。</small>
      </div>

      <div class="prefs-actions">
        <NButton
          type="primary"
          secondary
          :disabled="!store.desktop || !prefsDirty || !!busy"
          :loading="busy === 'save-prefs'"
          data-testid="ai-save-preferences"
          @click="savePreferences"
        >
          保存通用偏好
        </NButton>
      </div>
    </div>

    <p class="muted ai-note">
      <ShieldCheck :size="15" />
      密钥保存在本机用户数据目录（私密权限保护），不会上传到小伴网站或云端服务器。
    </p>
  </section>
</template>

<style scoped>
.ai-config > .n-alert { margin-top: 16px; }

/* Providers Grid */
.providers-section { margin-top: 22px; }
.providers-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}
.provider-card {
  border: 1px solid var(--ui-border);
  background: var(--ui-surface-alt);
  border-radius: 14px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  transition: all 0.2s ease;
}
.provider-card:hover {
  border-color: var(--ui-line);
}
.provider-card.active {
  border-color: var(--ui-primary);
  background: var(--ui-primary-soft);
}

/* Add custom card */
.add-custom-card {
  border: 2px dashed var(--ui-border);
  background: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
  min-height: 180px;
}
.add-custom-card:hover {
  border-color: var(--ui-primary);
  background: var(--ui-surface-alt);
}
.add-custom-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 12px;
}
.add-icon-circle {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: var(--ui-primary-soft);
  color: var(--ui-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 4px;
}
.add-custom-content h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 650;
  color: var(--ui-text-primary);
}
.add-custom-content p {
  margin: 0;
  font-size: 12px;
  color: var(--ui-text-secondary);
  line-height: 1.4;
}

.card-top {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 8px;
}
.provider-badge {
  display: flex;
  align-items: center;
  gap: 10px;
}
.provider-badge svg {
  color: var(--ui-primary);
  flex-shrink: 0;
}
.provider-title h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 650;
  color: var(--ui-text-primary);
}
.provider-sub {
  font-size: 11px;
  color: var(--ui-text-secondary);
  display: block;
  margin-top: 2px;
}
.provider-desc {
  font-size: 12px;
  color: var(--ui-text-secondary);
  line-height: 1.5;
  margin: 12px 0;
  min-height: 36px;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}
.provider-meta-row {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  margin-bottom: 14px;
}
.meta-label {
  color: var(--ui-muted);
}
.meta-model {
  font-size: 11px;
  background: var(--ui-surface);
  border: 1px solid var(--ui-line);
  padding: 2px 6px;
  border-radius: 6px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 170px;
}
.card-actions {
  display: flex;
  gap: 8px;
  margin-top: auto;
  align-items: center;
}

/* Modal form */
.modal-form { display: grid; gap: 16px; padding: 4px 0; }
.reply-preference summary { cursor: pointer; font-weight: 600; }
.reply-preference[open] summary { margin-bottom: 12px; }
.provider-console-tip .tip-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
  font-size: 12px;
}
.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 12px;
}

/* Fields */
.field { display: grid; gap: 6px; }
.field-header { display: flex; justify-content: space-between; align-items: center; }
.field > label, .field-header > label { font-size: 13px; font-weight: 650; }
.field small { color: var(--ui-muted); font-size: 12px; line-height: 1.6; display: flex; gap: 6px; align-items: center; flex-wrap: wrap; }
.field code { font-size: 12px; overflow-wrap: anywhere; }
.model-select-row { display: grid; gap: 4px; margin-top: 2px; }

/* Preferences Box */
.preferences-divider {
  margin-top: 32px;
  padding-top: 20px;
  border-top: 1px solid var(--ui-line);
}
.preferences-divider h3 { margin: 0 0 4px; font-size: 15px; font-weight: 650; }
.preferences-box {
  display: grid;
  gap: 18px;
  margin-top: 16px;
  background: var(--ui-surface-alt);
  border: 1px solid var(--ui-border);
  border-radius: 14px;
  padding: 18px;
}
.setting-row { display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 4px 0; }
.setting-row b { font-size: 13px; display: block; margin-bottom: 3px; }
.setting-row p { margin: 0; font-size: 12px; color: var(--ui-text-secondary); line-height: 1.5; }

.workspace-field { margin-top: 4px; }
.workspace-row { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.workspace-row code { flex: 1; min-width: 200px; overflow-wrap: anywhere; font-size: 12px; background: var(--ui-surface); border: 1px solid var(--ui-line); border-radius: 8px; padding: 10px 12px; }
.prefs-actions { display: flex; justify-content: flex-end; margin-top: 4px; }

.link-button { border: 0; background: none; padding: 0; color: var(--ui-primary); cursor: pointer; font: inherit; display: inline-flex; gap: 4px; align-items: center; }
.link-button:disabled { opacity: .5; cursor: not-allowed; }
.link-button.danger { color: var(--ui-primary-pressed); }
.form-error { margin-top: 10px; }
.ai-note { display: flex; gap: 7px; align-items: flex-start; font-size: 12px; line-height: 1.7; margin: 20px 0 0; }
.ai-note svg { flex: none; margin-top: 3px; }
</style>
