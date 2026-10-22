<script setup lang="ts">
import { computed, onDeactivated, onMounted, onBeforeUnmount, ref, watch } from 'vue';
import { NAlert, NInput, NModal, NPopconfirm, NSelect, NSwitch } from 'naive-ui';
import { Check, ChevronsUpDown, ExternalLink, FolderOpen, Plus, RefreshCw, SlidersHorizontal, Star, Trash2, X, Zap } from 'lucide-vue-next';
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
  providerTile,
  reasoningEfforts,
  reasoningLevelLabels,
  reasoningLevels,
  reasoningModes,
  reasoningSummary,
  preferenceOptions,
  providerEndpoint,
  validateAiForm,
  validateBaseUrl,
  validateKey,
  type ProviderMeta,
  type ReasoningMode,
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

type DiscoveredModel = NonNullable<ProviderConfigView['models']>[number];
/** One model in the provider dialog: its id, the thinking levels declared for it, and what the service reported about it. */
interface ModelDraft {
  id: string; name: string; description: string;
  mode: ReasoningMode; levels: AiReasoningEffort[];
  facts: Pick<DiscoveredModel, 'maxOutputTokens' | 'contextWindowTokens' | 'supportsTools' | 'supportsVision'>;
}
const DEFAULT_LEVELS: AiReasoningEffort[] = ['low', 'high', 'max'];

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
/** The default model: the one chat starts on. Always one of `modalModels` once the list is not empty. */
const modalModel = ref('');
const modalModels = ref<ModelDraft[]>([]);
const modelDraft = ref('');
const expandedModel = ref<string | null>(null);
const modalApiKey = ref('');
const modalSelectionMode = ref<'manual' | 'recommended'>('manual');
const modalReplyPreference = ref<AiReplyPreference>('default');
const modalHasKey = ref(false);
const modalKeyHint = ref('');
const confirmingClear = ref(false);
const protoOpen = ref(false);
const testState = ref<{ kind: 'idle' | 'testing' | 'ok' | 'fail'; text: string }>({ kind: 'idle', text: '' });
watch([modalModel, modalProtocol], () => {
  if (modalVisible.value) modalReplyPreference.value = 'default';
}, { flush: 'sync' });
const fetchedModels = ref<DiscoveredModel[]>([]);
const fetchedFor = ref('');
watch([modalBaseUrl, modalProtocol], () => {
  if (modalVisible.value) {
    fetchedModels.value = [];
    fetchedFor.value = '';
    testState.value = { kind: 'idle', text: '' };
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

// ── model drafts ────────────────────────────────────────────────────────────
function draftFrom(model: Pick<DiscoveredModel, 'id'> & Partial<DiscoveredModel>): ModelDraft {
  const declared = model.reasoningEfforts;
  return {
    id: model.id,
    name: model.name && model.name !== model.id ? model.name : '',
    description: model.description ?? '',
    mode: declared == null ? 'auto' : declared.length ? 'manual' : 'off',
    levels: declared?.length ? [...declared] : [...DEFAULT_LEVELS],
    facts: { maxOutputTokens: model.maxOutputTokens, contextWindowTokens: model.contextWindowTokens, supportsTools: model.supportsTools, supportsVision: model.supportsVision },
  };
}
function initialDrafts(p: ProviderConfigView | undefined, meta: ProviderMeta): ModelDraft[] {
  const list = (p?.models ?? []).map(draftFrom);
  const current = p ? p.model : meta.defaultModel;
  if (current && !list.some(d => d.id === current)) list.unshift(draftFrom({ id: current }));
  // A provider-wide declaration (older versions) belongs to the model that was selected.
  if (p?.reasoningEfforts != null) {
    const selected = list.find(d => d.id === current);
    if (selected) { selected.mode = p.reasoningEfforts.length ? 'manual' : 'off'; selected.levels = p.reasoningEfforts.length ? [...p.reasoningEfforts] : [...DEFAULT_LEVELS]; }
  }
  return list;
}
const declaredEfforts = (draft: ModelDraft) => draft.mode === 'off' ? [] : draft.mode === 'manual' ? draft.levels : undefined;
const levelsOf = (draft: ModelDraft) => reasoningLevels(modalProtocol.value, draft.id, declaredEfforts(draft));
const summaryOf = (draft: ModelDraft) => reasoningSummary(draft.mode, draft.levels, reasoningLevels(modalProtocol.value, draft.id));

const defaultDraft = computed(() => modalModels.value.find(d => d.id === modalModel.value));
/** The model the form will save as the default, counting a name typed but not yet added. */
const pendingDefault = computed(() => modalModel.value || modelDraft.value.trim());
const modalLevels = computed(() => defaultDraft.value ? levelsOf(defaultDraft.value) : reasoningLevels(modalProtocol.value, pendingDefault.value));
const canSetReplyPreference = computed(() => modalLevels.value.length > 0);
const replyPreferenceOptions = computed(() => preferenceOptions(modalLevels.value));
watch(replyPreferenceOptions, options => { if (!options.some(o => o.value === modalReplyPreference.value)) modalReplyPreference.value = 'default'; });
const modalOption = computed(() => protocolOption(modalProtocol.value));
const modalPreview = computed(() => validateBaseUrl(modalBaseUrl.value) ? '' : endpointPreview(modalProtocol.value, modalBaseUrl.value));
const thinkingEditable = computed(() => !isManaged.value && modalProtocol.value !== 'anthropic');
const modelChips = computed(() => {
  const have = new Set(modalModels.value.map(d => d.id));
  const fetched = fetchedModels.value.filter(m => !have.has(m.id));
  const known = new Set(fetched.map(m => m.id));
  const common = currentMeta.value.suggestedModels.filter(id => !have.has(id) && !known.has(id));
  return { fetched, common };
});

function commitDraft(): boolean {
  const id = modelDraft.value.trim();
  if (!id) return true;
  return addModel(id);
}
function addModel(id: string, found?: DiscoveredModel): boolean {
  const clean = id.trim();
  if (!clean) return false;
  if (clean.length > 200 || /[\u0000-\u001f\u007f]/.test(clean)) { props.store.notify('模型名不对，请按服务商给出的名字填写', 'info'); return false; }
  if (modalModels.value.some(d => d.id === clean)) { modelDraft.value = ''; return true; }
  const known = found ?? fetchedModels.value.find(m => m.id === clean) ?? currentMeta.value.models?.find(m => m.id === clean);
  modalModels.value = [...modalModels.value, draftFrom(known ?? { id: clean })];
  if (!modalModel.value) modalModel.value = clean;
  modelDraft.value = '';
  return true;
}
function removeModel(id: string) {
  modalModels.value = modalModels.value.filter(d => d.id !== id);
  if (expandedModel.value === id) expandedModel.value = null;
  if (modalModel.value === id) { modalModel.value = modalModels.value[0]?.id ?? ''; modalSelectionMode.value = 'manual'; }
}
function makeDefault(id: string) {
  modalModel.value = id;
  modalSelectionMode.value = 'manual';
}
function toggleLevel(draft: ModelDraft, level: AiReasoningEffort) {
  draft.levels = draft.levels.includes(level) ? draft.levels.filter(l => l !== level) : reasoningEfforts.filter(l => draft.levels.includes(l) || l === level);
}
function setRecommended(value: boolean) {
  modalSelectionMode.value = value ? 'recommended' : 'manual';
  if (value && currentMeta.value.defaultModel) {
    if (!modalModels.value.some(d => d.id === currentMeta.value.defaultModel)) addModel(currentMeta.value.defaultModel);
    modalModel.value = currentMeta.value.defaultModel;
  }
}

// ── protocol ────────────────────────────────────────────────────────────────
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
  protoOpen.value = false;
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
/** The address shown beside each protocol in the picker: the vendor's own, or an example for a custom service. */
const protocolChoices = computed(() => aiProtocols.map(o => ({
  ...o,
  url: isEditingCustom.value ? o.placeholder : providerEndpoint(editingId.value, o.value) || '该服务商未提供此协议的地址，请手动填写',
})));
const protocolRef = ref<HTMLElement | null>(null);
function closeProtocolOnOutside(event: Event) {
  if (protoOpen.value && protocolRef.value && !protocolRef.value.contains(event.target as Node)) protoOpen.value = false;
}
function onWindowKey(event: KeyboardEvent) {
  if (event.key !== 'Escape' || !modalVisible.value) return;
  if (protoOpen.value) protoOpen.value = false;
  else if (!busy.value) modalVisible.value = false;
}
function resetAddress() {
  modalBaseUrl.value = providerEndpoint(editingId.value, modalProtocol.value);
}
const canResetAddress = computed(() => !isEditingCustom.value && !isManaged.value
  && normalizeBaseUrl(modalBaseUrl.value) !== normalizeBaseUrl(providerEndpoint(editingId.value, modalProtocol.value)) && !!providerEndpoint(editingId.value, modalProtocol.value));

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

interface ProviderRow { id: string; name: string; custom: boolean; kind: string; desc: string; mono: string; hue: string }
const providerRows = computed<ProviderRow[]>(() => [
  ...catalogueProviders.value.map(p => ({
    id: p.id, name: p.name, custom: false, desc: p.desc,
    kind: p.type === 'free' ? '免费服务' : p.type === 'paid' ? '付费服务' : '',
    ...providerTile(p.id, p.name),
  })),
  ...customProviders.value.map(p => ({
    id: p.id, name: p.name || '自定义供应商', custom: true, desc: p.baseUrl || '尚未填写 API 端点地址', kind: '自定义',
    ...providerTile(p.id, p.name || '自定义'),
  })),
]);
function rowSummary(row: ProviderRow): string {
  const p = getProviderSaved(row.id);
  if (!p || !isProviderFullyConfigured(row.id)) return row.desc;
  const count = Math.max(p.models?.length ?? 0, p.model ? 1 : 0);
  return `${count} 个模型 · 默认 ${p.model || '未设置'}`;
}

const modalFormError = computed(() => {
  if (isEditingCustom.value && !modalName.value.trim()) return '请填写自定义供应商名称';
  return validateAiForm({ baseUrl: modalBaseUrl.value, model: pendingDefault.value }) || validateKey(modalApiKey.value);
});

const modalKeyPlaceholder = computed(() => {
  if (modalHasKey.value) return `已保存 ${modalKeyHint.value}，留空表示不修改`;
  return '没有密钥（本地模型服务可以不填）';
});

const workspaceChanged = computed(() => workspace.value !== saved.value.workspace);
const shownWorkspace = computed(() => workspace.value || saved.value.defaultWorkspace || '桌面应用中可查看');

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
const editingConfigured = computed(() => isProviderFullyConfigured(editingId.value));
const editingInUse = computed(() => isProviderInUse(editingId.value));
const modalTitle = computed(() => isEditingCustom.value ? (getProviderSaved(editingId.value) ? modalName.value || '自定义供应商' : '添加自定义供应商') : currentMeta.value.name);
const modalDesc = computed(() => isEditingCustom.value ? '任何兼容 OpenAI 或 Anthropic 接口的模型服务' : currentMeta.value.desc);
const modalTile = computed(() => isEditingCustom.value && !getProviderSaved(editingId.value) ? { mono: '+', hue: 'gray' } : providerTile(editingId.value, modalTitle.value));

function fill(view: AiConfigView) {
  saved.value = view;
  workspace.value = view.workspace;
  hideToolCalls.value = view.hideToolCalls ?? false;
  openComputerUse.value = view.openComputerUse ?? false;
}

let unlisten: (() => void) | undefined;
onBeforeUnmount(() => { unlisten?.(); window.removeEventListener('keydown', onWindowKey); document.removeEventListener('mousedown', closeProtocolOnOutside); });
onDeactivated(() => { modalApiKey.value = ''; modalVisible.value = false; });
async function refreshCatalogue() {
  if (!props.store.desktop || busy.value) return;
  busy.value = 'catalogue';
  try { fill(await invoke<AiConfigView>('refresh_ai_providers', { force: true })); loadError.value = ''; }
  catch (e) { loadError.value = String(e instanceof Error ? e.message : e); }
  finally { busy.value = ''; }
}
onMounted(async () => {
  window.addEventListener('keydown', onWindowKey);
  document.addEventListener('mousedown', closeProtocolOnOutside);
  if (!props.store.desktop) return;
  try {
    fill(await invoke<AiConfigView>('get_ai_config'));
    unlisten = await listen<AiConfigView>('ai-config-changed', fill);
    void invoke<AiConfigView>('refresh_ai_providers', { force: false }).then(fill).catch(() => {});
  } catch (error) {
    loadError.value = String(error instanceof Error ? error.message : error);
  }
});

function resetModalState() {
  modalApiKey.value = '';
  modelDraft.value = '';
  expandedModel.value = null;
  protoOpen.value = false;
  confirmingClear.value = false;
  fetchedModels.value = [];
  fetchedFor.value = '';
  testState.value = { kind: 'idle', text: '' };
}

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
  modalModels.value = initialDrafts(p, meta);
  modalSelectionMode.value = p?.selectionMode ?? 'manual';
  modalReplyPreference.value = p?.replyPreference ?? 'default';
  resetModalState();
  modalVisible.value = true;
}

function openCreateCustomModal() {
  modalSelectionMode.value = 'manual';
  modalReplyPreference.value = 'default';
  const newId = `custom_${Date.now()}`;
  editingId.value = newId;
  modalName.value = '';
  modalProtocol.value = 'responses';
  modalBaseUrl.value = '';
  modalModel.value = '';
  modalModels.value = [];
  modalHasKey.value = false;
  modalKeyHint.value = '';
  resetModalState();
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

/** The list as the host stores it. Facts a service reported about another address must not follow a changed endpoint. */
function modelPayload(endpointChanged: boolean) {
  return modalModels.value.map(d => ({
    id: d.id,
    name: d.name || d.id,
    description: d.description,
    ...(endpointChanged ? {} : Object.fromEntries(Object.entries(d.facts).filter(([, v]) => v != null))),
    ...(d.mode === 'manual' ? { reasoningEfforts: [...d.levels] } : d.mode === 'off' ? { reasoningEfforts: [] } : {}),
  }));
}

async function saveProvider(activate: boolean) {
  if (!props.store.desktop) return;
  if (!commitDraft()) return;
  if (modalFormError.value) {
    props.store.notify(modalFormError.value, 'error');
    return;
  }
  busy.value = 'save-provider';
  // 保存前是否已在生效；用于区分“本次保存让它生效”和“它本来就在用”。
  const wasInUse = isProviderInUse(editingId.value);
  const previous = getProviderSaved(editingId.value);
  const endpointChanged = !previous || normalizeBaseUrl(previous.baseUrl) !== normalizeBaseUrl(modalBaseUrl.value) || previous.protocol !== modalProtocol.value;
  const model = modalModel.value.trim();
  try {
    const updated = await invoke<AiConfigView>('save_ai_provider', {
      providerId: editingId.value,
      name: modalName.value.trim() || null,
      protocol: modalProtocol.value,
      baseUrl: modalBaseUrl.value.trim(),
      model,
      apiKey: modalApiKey.value.trim() || null,
      clearKey: false,
      activate,
      selectionMode: modalSelectionMode.value,
      ...(!isManaged.value ? { models: modelPayload(endpointChanged), clearReasoningEfforts: true } : {}),
      replyPreference: canSetReplyPreference.value ? modalReplyPreference.value : 'default',
      discoveredModel: fetchedFor.value === `${modalProtocol.value}\n${modalBaseUrl.value.trim()}`
        ? fetchedModels.value.find(m => m.id === model) ?? null
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
    modalVisible.value = false;
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

/**
 * Ask the service for its model list. Used both to pull the list ("拉取可用模型") and to test the
 * connection: a service that answers the list request with this address and key is reachable and accepts the key.
 */
async function fetchModels(purpose: 'fetch' | 'test' = 'fetch') {
  if (!props.store.desktop) return;
  const cleanUrl = modalBaseUrl.value.trim();
  const note = (text: string) => purpose === 'test' ? (testState.value = { kind: 'fail', text }) : props.store.notify(text, 'info');
  if (!cleanUrl) { note('请先填写 API 地址'); return; }
  const err = validateBaseUrl(cleanUrl) || validateKey(modalApiKey.value);
  if (err) { note(err); return; }
  busy.value = purpose === 'test' ? 'test' : 'models';
  if (purpose === 'test') testState.value = { kind: 'testing', text: '正在连接…' };
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
      if (purpose === 'test') testState.value = { kind: 'ok', text: `连接成功，服务商返回 ${list.length} 个可用模型` };
      else props.store.notify(`成功获取到 ${list.length} 个模型`, 'success');
    } else if (purpose === 'test') {
      testState.value = { kind: 'ok', text: '已连接，但服务商没有返回模型列表，可手动添加模型' };
    } else {
      props.store.notify('未获取到可用模型', 'info');
    }
  } catch (error) {
    const message = String(error instanceof Error ? error.message : error);
    if (purpose === 'test') testState.value = { kind: 'fail', text: message };
    else props.store.notify(message, 'error');
  } finally {
    busy.value = '';
  }
}

/** Switches and the folder apply as soon as they change, like the system settings they imitate. */
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
  } catch (error) {
    fill(saved.value);
    props.store.notify(String(error instanceof Error ? error.message : error), 'error');
  } finally {
    busy.value = '';
  }
}
function setHideToolCalls(value: boolean) { hideToolCalls.value = value; void savePreferences(); }
function setOpenComputerUse(value: boolean) { openComputerUse.value = value; void savePreferences(); }

async function chooseFolder() {
  busy.value = 'folder';
  let chosen: string | null = null;
  try {
    chosen = await invoke<string | null>('pick_workspace');
  } catch (error) {
    props.store.notify(error, 'error');
  } finally {
    busy.value = '';
  }
  if (chosen) { workspace.value = chosen; await savePreferences(); }
}
async function resetWorkspace() {
  workspace.value = '';
  await savePreferences();
}

function openExternal(url?: string) {
  if (!url) return;
  invoke('open_download_url', { url }).catch(() => {});
}
</script>

<template>
  <section class="ai-config" data-testid="ai-config-card">
    <NAlert v-if="!store.desktop" type="info" :show-icon="false">请在桌面应用中配置模型服务。</NAlert>
    <NAlert v-if="loadError" type="warning" :show-icon="false" data-testid="ai-load-error">
      {{ loadError }}。可以稍后刷新，已有配置会保留。
    </NAlert>

    <!-- 小伴 AI is its own kind of provider: account login and credit instead of an address and key -->
    <AiGatewayCard v-if="gatewayOffered" :store="store" :config="saved" @config="fill" />

    <div class="xb-group-title list-title">
      <span>自有模型服务</span>
      <span class="list-note">配置后可在输入框里切换</span>
      <button type="button" class="xb-link" :disabled="!store.desktop || !!busy" data-testid="ai-refresh-catalogue" @click="refreshCatalogue"><RefreshCw :size="12" :class="{ spinning: busy === 'catalogue' }" />刷新供应商</button>
    </div>
    <div class="xb-group providers-list">
      <div v-for="row in providerRows" :key="row.id" class="xb-row provider-row" :class="{ 'is-active': isProviderInUse(row.id) }" :data-testid="`ai-provider-card-${row.id}`">
        <span class="ptile" :class="`hue-${row.hue}`" aria-hidden="true">{{ row.mono }}</span>
        <div class="xb-row-text">
          <b>{{ row.name }}<em v-if="row.kind" class="pkind">{{ row.kind }}</em></b>
          <span>{{ rowSummary(row) }}</span>
        </div>
        <span v-if="isProviderInUse(row.id)" class="xb-ok">当前使用</span>
        <span v-else-if="isProviderFullyConfigured(row.id)" class="pstate">已配置</span>
        <span v-else class="pstate is-warn">待配置</span>
        <button v-if="isProviderFullyConfigured(row.id) && !isProviderInUse(row.id)" type="button" class="xb-btn is-sm" :disabled="!store.desktop || !!busy" :data-testid="`ai-provider-activate-${row.id}`" @click="activateProvider(row.id)">{{ busy === `activate-${row.id}` ? '切换中…' : '启用' }}</button>
        <button type="button" class="xb-btn is-sm" :disabled="!store.desktop || !!busy" :data-testid="`ai-provider-config-${row.id}`" @click="openConfigureModal(row.id)">{{ isProviderFullyConfigured(row.id) ? '编辑' : '配置' }}</button>
        <NPopconfirm v-if="row.custom && !isProviderActive(row.id)" @positive-click="deleteCustomProvider(row.id)">
          <template #trigger>
            <button type="button" class="xb-icon-btn is-sm" aria-label="删除该自定义供应商" :disabled="!store.desktop || !!busy" :data-testid="`ai-provider-delete-${row.id}`"><Trash2 :size="14" /></button>
          </template>
          确定要删除该自定义供应商吗？
        </NPopconfirm>
      </div>
      <button type="button" class="xb-row add-row" data-testid="ai-add-custom-card" @click="openCreateCustomModal"><Plus :size="14" :stroke-width="2.4" />添加自定义供应商（OpenAI 兼容）</button>
    </div>
    <p class="status-line" data-testid="ai-status">{{ activeStatusText }}</p>

    <!-- Provider dialog -->
    <NModal v-model:show="modalVisible" :close-on-esc="false" :mask-closable="false" data-testid="ai-provider-modal" transform-origin="center">
      <div class="xb-provider" role="dialog" aria-modal="true" :aria-label="modalTitle" data-testid="ai-provider-dialog">
        <header class="pv-head">
          <span class="ptile is-lg" :class="`hue-${modalTile.hue}`" aria-hidden="true">{{ modalTile.mono }}</span>
          <div class="pv-title">
            <div class="pv-name"><h2>{{ modalTitle }}</h2><span v-if="editingConfigured" class="pill-ok" data-testid="ai-provider-connected"><i></i>已连接</span></div>
            <span class="pv-desc">{{ modalDesc }}</span>
          </div>
          <button type="button" class="xb-icon-btn" aria-label="关闭" data-testid="ai-provider-close" @click="modalVisible = false"><X :size="15" :stroke-width="2.2" /></button>
        </header>

        <div class="pv-body xb-scroll">
          <section>
            <p class="pv-sec">连接</p>
            <div class="pv-card">
              <div v-if="isEditingCustom" class="fd">
                <label for="ai-provider-name" class="lb">供应商名称</label>
                <NInput v-model:value="modalName" placeholder="例如：公司内部模型、本地 Ollama" :disabled="!store.desktop || !!busy" data-testid="ai-provider-name" :input-props="{ id: 'ai-provider-name', autocomplete: 'off', spellcheck: false }" />
              </div>

              <div class="fd">
                <span class="lb" id="ai-protocol-label">接口协议</span>
                <div ref="protocolRef" class="proto">
                  <button type="button" class="proto-trigger" :class="{ open: protoOpen }" aria-haspopup="listbox" :aria-expanded="protoOpen" aria-labelledby="ai-protocol-label" :disabled="!store.desktop || !!busy || isManaged" data-testid="ai-protocol" @click="protoOpen = !protoOpen">
                    <b>{{ modalOption.label }}</b><span class="sub">{{ modalOption.hint }}</span>
                    <ChevronsUpDown :size="12" :stroke-width="2.4" class="chev" />
                  </button>
                  <div v-if="protoOpen" class="xb-menu proto-menu" role="listbox" aria-label="选择接口协议" data-testid="ai-protocol-menu">
                    <button v-for="option in protocolChoices" :key="option.value" type="button" role="option" class="xb-menu-item proto-option" :class="{ 'is-active': option.value === modalProtocol }" :aria-selected="option.value === modalProtocol" :data-testid="`ai-protocol-${option.value}`" @click="onProtocolChange(option.value)">
                      <span class="tick"><Check v-if="option.value === modalProtocol" :size="13" :stroke-width="3" /></span>
                      <span class="opt-text"><b>{{ option.label }}</b><span>{{ option.hint.replace(/[。.]$/, '') }} · <code>{{ option.url }}</code></span></span>
                    </button>
                  </div>
                </div>
                <small v-if="!isEditingCustom && !isManaged" class="muted" data-testid="ai-protocol-endpoint-hint">切换协议会自动换成该服务商对应的接口地址；手动改过的地址不会被覆盖。</small>
              </div>

              <div class="fd">
                <div class="lb-row"><label for="ai-base-url" class="lb">API 地址</label>
                  <button v-if="canResetAddress" type="button" class="xb-link" data-testid="ai-base-url-reset" @click="resetAddress">恢复默认</button></div>
                <NInput v-model:value="modalBaseUrl" class="mono" :placeholder="modalOption.placeholder" :disabled="!store.desktop || !!busy || isManaged" data-testid="ai-base-url" :input-props="{ id: 'ai-base-url', autocomplete: 'off', spellcheck: false }" />
                <small v-if="modalPreview" class="muted" data-testid="ai-endpoint-preview">实际请求端点：<code>{{ modalPreview }}</code></small>
                <NAlert v-if="insecureRemote(modalBaseUrl)" type="warning" :show-icon="false" data-testid="ai-http-warning">
                  http 地址不加密，密钥和内容会明文传输，建议仅用于本机或可信内网。
                </NAlert>
              </div>

              <div v-if="!isManaged" class="fd">
                <div class="lb-row"><label for="ai-api-key" class="lb">API 密钥</label>
                  <button v-if="currentMeta.consoleUrl && !isEditingCustom" type="button" class="xb-link" data-testid="ai-console-link" @click="openExternal(currentMeta.consoleUrl)">{{ currentMeta.consoleText || '前往获取 API Key' }}<ExternalLink :size="11" /></button></div>
                <NInput v-model:value="modalApiKey" class="mono" type="password" show-password-on="click" :placeholder="modalKeyPlaceholder" :disabled="!store.desktop || !!busy" data-testid="ai-api-key" :input-props="{ id: 'ai-api-key', autocomplete: 'off', spellcheck: false }" />
                <small class="muted key-actions">
                  <span>只保存在这台电脑上，不会上传到小伴服务器</span>
                  <template v-if="modalHasKey">
                    <template v-if="!confirmingClear">
                      <button type="button" class="xb-link" data-testid="ai-clear-key" :disabled="!store.desktop || !!busy" @click="confirmingClear = true"><Trash2 :size="12" />删除已保存的密钥</button>
                    </template>
                    <template v-else>
                      删除后需重新输入密钥。
                      <button type="button" class="xb-link is-danger" data-testid="ai-clear-confirm" :disabled="!!busy" @click="clearProviderKey">确认删除</button>
                      <button type="button" class="xb-link" data-testid="ai-clear-cancel" :disabled="!!busy" @click="confirmingClear = false">取消</button>
                    </template>
                  </template>
                </small>
              </div>

              <div v-if="!isManaged" class="test-row">
                <button type="button" class="xb-btn" :disabled="!store.desktop || !!busy || !modalBaseUrl.trim()" data-testid="ai-test-connection" @click="fetchModels('test')"><Zap :size="14" />测试连接</button>
                <span class="test-text" :class="`is-${testState.kind}`" role="status" data-testid="ai-test-result">{{ testState.text || '填好地址和密钥后可以测试；会向服务商请求一次模型列表' }}</span>
              </div>
            </div>
          </section>

          <section>
            <p class="pv-sec">模型<span>· {{ modalModels.length }} 个，输入框里可切换</span></p>
            <div class="pv-card is-flush">
              <p v-if="!modalModels.length" class="empty">还没有模型。输入模型名称，或拉取服务商的可用模型</p>
              <div v-for="model in modalModels" :key="model.id" class="mrow" :class="{ 'is-open': expandedModel === model.id }" :data-testid="`ai-model-row-${model.id}`">
                <div class="mrow-main">
                  <button type="button" class="star" :class="{ on: model.id === modalModel }" :aria-pressed="model.id === modalModel" :title="model.id === modalModel ? '默认模型' : '设为默认模型'" aria-label="设为默认模型" :disabled="!store.desktop || !!busy" :data-testid="`ai-model-default-${model.id}`" @click="makeDefault(model.id)"><Star :size="15" :stroke-width="1.8" /></button>
                  <div class="mname">
                    <span class="mid"><code>{{ model.id }}</code><em v-if="model.id === modalModel" class="def-tag">默认</em></span>
                    <span class="msub">思考等级：{{ summaryOf(model) }}</span>
                  </div>
                  <button v-if="thinkingEditable" type="button" class="xb-icon-btn" :class="{ on: expandedModel === model.id }" :aria-expanded="expandedModel === model.id" :aria-label="expandedModel === model.id ? `收起 ${model.id} 的设置` : `设置 ${model.id}`" :data-testid="`ai-model-expand-${model.id}`" @click="expandedModel = expandedModel === model.id ? null : model.id"><SlidersHorizontal :size="15" /></button>
                  <button v-if="!isManaged" type="button" class="xb-icon-btn" :aria-label="`移除 ${model.id}`" :disabled="!!busy" :data-testid="`ai-model-remove-${model.id}`" @click="removeModel(model.id)"><Trash2 :size="15" /></button>
                </div>
                <div v-if="thinkingEditable && expandedModel === model.id" class="mpanel">
                  <div class="mpanel-row"><span class="mpanel-label">思考等级</span>
                    <div class="xb-seg" role="radiogroup" aria-label="思考等级声明" data-testid="ai-reasoning-mode">
                      <button v-for="mode in reasoningModes" :key="mode.value" type="button" role="radio" :aria-checked="model.mode === mode.value" :data-testid="`ai-reasoning-mode-${mode.value}`" @click="model.mode = mode.value">{{ mode.label }}</button>
                    </div>
                  </div>
                  <div v-if="model.mode === 'manual'" class="mpanel-row levels" role="group" aria-label="支持的思考等级" data-testid="ai-reasoning-levels">
                    <button v-for="level in reasoningEfforts" :key="level" type="button" role="checkbox" class="xb-chip level" :class="{ on: model.levels.includes(level) }" :aria-checked="model.levels.includes(level)" :data-testid="`ai-reasoning-level-${level}`" @click="toggleLevel(model, level)"><Check v-if="model.levels.includes(level)" :size="10" :stroke-width="3" />{{ reasoningLevelLabels[level] }}</button>
                  </div>
                  <span class="mpanel-hint">{{ reasoningModes.find(m => m.value === model.mode)?.hint }}</span>
                </div>
              </div>

              <div v-if="!isManaged" class="addbar">
                <div class="addline">
                  <NInput v-model:value="modelDraft" class="mono" placeholder="输入模型名称" :disabled="!store.desktop || !!busy" data-testid="ai-model" :input-props="{ id: 'ai-model', 'aria-label': '添加模型', autocomplete: 'off', spellcheck: false }" @keydown.enter.prevent="commitDraft()" />
                  <button type="button" class="xb-btn" :disabled="!store.desktop || !!busy || !modelDraft.trim()" data-testid="ai-model-add" @click="commitDraft()">添加</button>
                  <button type="button" class="xb-btn" :disabled="!store.desktop || !!busy || !modalBaseUrl.trim()" data-testid="ai-fetch-models" @click="fetchModels('fetch')"><RefreshCw :size="14" :class="{ spinning: busy === 'models' }" />拉取可用模型</button>
                </div>
                <div v-if="modelChips.fetched.length || modelChips.common.length" class="chips" data-testid="ai-model-select-wrapper">
                  <template v-if="modelChips.common.length">
                    <span class="chips-label">常用</span>
                    <button v-for="id in modelChips.common" :key="id" type="button" class="xb-chip model-chip" :data-testid="`ai-model-chip-${id}`" @click="addModel(id)"><Plus :size="10" :stroke-width="3" />{{ id }}</button>
                  </template>
                  <template v-if="modelChips.fetched.length">
                    <span class="chips-label">可用</span>
                    <div class="fetched">
                      <button v-for="m in modelChips.fetched" :key="m.id" type="button" class="xb-chip model-chip" :title="m.name" :data-testid="`ai-model-chip-${m.id}`" @click="addModel(m.id, m)"><Plus :size="10" :stroke-width="3" />{{ m.id }}</button>
                    </div>
                  </template>
                </div>
              </div>
              <div v-if="currentMeta.defaultModel && !isEditingCustom" class="mfoot">
                <div class="xb-row-text"><b>跟随推荐模型 · {{ currentMeta.defaultModel }}</b><span>手动选择的模型不会被推荐更新覆盖。</span></div>
                <NSwitch :value="modalSelectionMode === 'recommended'" aria-label="跟随推荐模型" data-testid="ai-follow-recommended" @update:value="setRecommended" />
              </div>
              <div v-if="modalModels.length && (!isManaged || canSetReplyPreference)" class="mfoot" data-testid="ai-reply-advanced">
                <div class="xb-row-text"><b>回答偏好</b><span>{{ canSetReplyPreference ? '默认模型的思考深度；更深入可能耗时更久、消耗更多额度。' : '默认模型不支持调节思考深度（可在上方为它声明思考等级）。' }}</span></div>
                <NSelect v-if="canSetReplyPreference" v-model:value="modalReplyPreference" :options="replyPreferenceOptions" :disabled="!store.desktop || !!busy" style="width:150px" data-testid="ai-reply-preference" />
              </div>
            </div>
          </section>

          <NAlert v-if="modalFormError" type="error" :show-icon="false" class="form-error" data-testid="ai-form-error">{{ modalFormError }}</NAlert>
        </div>

        <footer class="pv-foot">
          <NPopconfirm v-if="isEditingCustom && getProviderSaved(editingId) && !isProviderActive(editingId)" @positive-click="deleteCustomProvider(editingId)">
            <template #trigger><button type="button" class="del-link" :disabled="!!busy" data-testid="ai-provider-delete">删除此配置</button></template>
            确定要删除该自定义供应商吗？
          </NPopconfirm>
          <button type="button" class="xb-btn" :disabled="!!busy" data-testid="ai-cancel" style="margin-left:auto" @click="modalVisible = false">取消</button>
          <button v-if="!noActiveProvider && !editingInUse" type="button" class="xb-btn" :disabled="!store.desktop || !!modalFormError || !!busy" data-testid="ai-save" @click="saveProvider(false)">仅保存</button>
          <button v-if="!editingInUse" type="button" class="xb-btn-primary" :disabled="!store.desktop || !!modalFormError || !!busy" data-testid="ai-save-and-activate" @click="saveProvider(true)">保存并启用</button>
          <button v-else type="button" class="xb-btn-primary" :disabled="!store.desktop || !!modalFormError || !!busy" data-testid="ai-save" @click="saveProvider(false)">保存</button>
        </footer>
      </div>
    </NModal>

    <div class="xb-group-title">对话与工具</div>
    <div class="xb-group">
      <div class="xb-row"><div class="xb-row-text"><b>隐藏工具调用信息</b><span>回答里只保留结果，工具调用的中间卡片不再展示（不影响工具正常执行）</span></div>
        <NSwitch :value="hideToolCalls" aria-label="隐藏工具调用信息" :disabled="!store.desktop || !!busy" data-testid="ai-hide-tool-calls" @update:value="setHideToolCalls" /></div>
      <div class="xb-row"><div class="xb-row-text"><b>启用电脑屏幕与应用控制</b><span>允许小伴操控电脑应用（点击、键入文字等，open-computer-use）。macOS 首次使用需授予辅助功能权限。</span></div>
        <NSwitch :value="openComputerUse" aria-label="启用电脑屏幕与应用控制" :disabled="!store.desktop || !!busy" data-testid="ai-open-computer-use" @update:value="setOpenComputerUse" /></div>
      <div class="xb-row workspace-row"><div class="xb-row-text"><b>默认工作区</b><span class="ws-path"><code data-testid="ai-workspace">{{ shownWorkspace }}</code></span>
        <span>每个对话在自己的文件夹里工作，可以在聊天输入框下方切换；这里选的会作为下一个新对话的文件夹，较早的对话也一直用它。运行命令以当前用户权限执行，请谨慎授权。</span></div>
        <div class="ws-actions">
          <button type="button" class="xb-btn is-sm" :disabled="!store.desktop || !!busy" data-testid="ai-workspace-pick" @click="chooseFolder"><FolderOpen :size="14" />选择文件夹</button>
          <button v-if="workspace" type="button" class="xb-btn is-sm" :disabled="!!busy" data-testid="ai-workspace-reset" @click="resetWorkspace">恢复默认</button>
        </div></div>
    </div>

    <p class="ai-note">密钥保存在本机用户数据目录（私密权限保护），不会上传到小伴网站或云端服务器。</p>
  </section>
</template>

<style scoped>
.ai-config{display:block}
.ai-config>.n-alert{margin-bottom:12px}
.list-title{display:flex;align-items:center;gap:8px}
.list-note{font-weight:400;color:var(--ui-text-tertiary)}
.list-title .xb-link{margin-left:auto;display:inline-flex;align-items:center;gap:4px;font-weight:400}
.spinning{animation:spin 1s linear infinite}
@keyframes spin{to{transform:rotate(360deg)}}
.providers-list{overflow:hidden}
.provider-row{min-height:56px}
.provider-row.is-active{background:linear-gradient(90deg,var(--ui-soft),transparent 55%)}
.pkind{margin-left:7px;font-style:normal;font-weight:400;font-size:11px;color:var(--ui-text-tertiary)}
.pstate{font-size:12px;color:var(--ui-text-secondary);white-space:nowrap}
.pstate.is-warn{color:var(--ui-warning-text)}
.provider-row .xb-row-text>span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.add-row{width:100%;border-right:0;border-bottom:0;border-left:0;background:transparent;justify-content:flex-start;gap:6px;min-height:44px;color:var(--ui-ink);font-size:13px;font-weight:500;cursor:pointer;text-align:left}
.add-row:hover{background:var(--ui-surface-hover)}
.status-line{margin:8px 4px 0;font-size:12px;color:var(--ui-text-secondary)}

/* Provider tiles */
.ptile{--tile-bg:#F1F1F3;--tile-ink:#2C2C30;width:30px;height:30px;flex-shrink:0;border-radius:8px;display:grid;place-items:center;background:var(--tile-bg);color:var(--tile-ink);font-size:12px;font-weight:700;box-shadow:inset 0 0 0 1px rgba(0,0,0,.05)}
.ptile.is-lg{width:40px;height:40px;border-radius:11px;font-size:14px}
.hue-blue{--tile-bg:#E8EEFD;--tile-ink:#3557C9}
.hue-sky{--tile-bg:#E6F3FB;--tile-ink:#1C6E9E}
.hue-gray{--tile-bg:#F1F1F3;--tile-ink:#2C2C30}
.hue-orange{--tile-bg:#FDEEE6;--tile-ink:#B4501C}
.hue-green{--tile-bg:#E3F2E9;--tile-ink:#1E6A45}
.hue-pink{--tile-bg:#FCE8F0;--tile-ink:#B02A62}
html[data-theme=dark] .ptile{box-shadow:inset 0 0 0 1px rgba(255,255,255,.08)}
html[data-theme=dark] .hue-blue{--tile-bg:rgba(143,169,240,.16);--tile-ink:#9DB4F4}
html[data-theme=dark] .hue-sky{--tile-bg:rgba(124,192,230,.16);--tile-ink:#84C6EA}
html[data-theme=dark] .hue-gray{--tile-bg:rgba(255,255,255,.1);--tile-ink:#D0D0D4}
html[data-theme=dark] .hue-orange{--tile-bg:rgba(240,162,122,.16);--tile-ink:#F2AA86}
html[data-theme=dark] .hue-green{--tile-bg:rgba(111,208,160,.16);--tile-ink:#78D6A7}
html[data-theme=dark] .hue-pink{--tile-bg:rgba(238,143,179,.16);--tile-ink:#F095B7}

/* Dialog */
.xb-provider{width:600px;max-width:calc(100vw - 32px);max-height:min(784px,calc(100vh - 32px));display:flex;flex-direction:column;overflow:hidden;border-radius:18px;background:var(--ui-surface-sunken);color:var(--ui-text-primary);font-family:var(--ui-font-family);font-size:13px;box-shadow:0 0 0 1px var(--ui-border),var(--ui-shadow-dialog)}
.pv-head{display:flex;align-items:center;gap:12px;padding:18px 14px 16px 22px;background:var(--ui-surface);border-bottom:1px solid var(--ui-line)}
.pv-head .xb-icon-btn{align-self:flex-start}
.pv-title{flex:1;min-width:0;display:flex;flex-direction:column;gap:2px}
.pv-name{display:flex;align-items:center;gap:8px}
.pv-name h2{margin:0;font-size:16px;font-weight:600}
.pv-desc{font-size:12px;color:var(--ui-text-secondary);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.pill-ok{display:inline-flex;align-items:center;gap:5px;height:20px;padding:0 7px;border-radius:999px;background:color-mix(in srgb,var(--ui-success) 15%,transparent);color:var(--ui-success-text);font-size:11px;font-weight:500}
.pill-ok i{width:6px;height:6px;border-radius:50%;background:var(--ui-success)}
.pv-body{flex:1;min-height:0;padding:18px 22px 10px;display:flex;flex-direction:column;gap:20px}
.pv-sec{margin:0 0 10px;font-size:12px;font-weight:600;color:var(--ui-text-secondary);display:flex;align-items:center;gap:8px}
.pv-sec span{font-weight:400}
.pv-card{display:flex;flex-direction:column;gap:14px;padding:16px;border-radius:12px;background:var(--ui-surface);box-shadow:var(--ui-shadow-card)}
.pv-card.is-flush{gap:0;padding:0;overflow:visible}
.fd{display:flex;flex-direction:column;gap:6px;min-width:0}
.lb{font-size:13px;font-weight:500}
.lb-row{display:flex;align-items:center;justify-content:space-between;gap:10px}
.lb-row .xb-link{display:inline-flex;align-items:center;gap:3px;font-size:12px}
.mono :deep(input),.mono :deep(.n-input__input-el){font-family:var(--ui-font-mono);font-size:12.5px}
.muted{font-size:11.5px;line-height:1.6;color:var(--ui-text-secondary)}
.muted code{font-family:var(--ui-font-mono);font-size:11.5px;overflow-wrap:anywhere}
.key-actions{display:flex;flex-wrap:wrap;align-items:center;gap:4px 12px}
.key-actions .xb-link{display:inline-flex;align-items:center;gap:4px;font-size:11.5px}
.xb-link.is-danger{color:var(--ui-danger)}
.sub{font-size:12px;color:var(--ui-text-secondary);font-weight:400}

.proto{position:relative}
.proto-trigger{width:100%;height:34px;display:flex;align-items:center;gap:8px;padding:0 10px 0 11px;border:0;border-radius:9px;background:var(--ui-surface);box-shadow:inset 0 0 0 1px var(--ui-border-strong);color:var(--ui-text-primary);font:inherit;font-size:13.5px;text-align:left;cursor:pointer}
.proto-trigger b{font-weight:500;white-space:nowrap}
.proto-trigger .sub{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.proto-trigger .chev{margin-left:auto;flex-shrink:0;color:var(--ui-text-tertiary)}
.proto-trigger.open{box-shadow:inset 0 0 0 1.5px var(--ui-btn),0 0 0 3px var(--ui-soft)}
.proto-trigger:disabled{opacity:.6;cursor:not-allowed}
.proto-menu{position:absolute;left:0;right:0;top:40px;z-index:5}
.proto-option{align-items:center;padding:8px 10px;min-height:0}
.proto-option .tick{width:14px;flex-shrink:0;display:grid;place-items:center;color:var(--ui-ink)}
.opt-text{display:flex;flex-direction:column;gap:1px;min-width:0}
.opt-text b{font-size:13.5px;font-weight:500}
.opt-text span{font-size:11.5px;color:var(--ui-text-secondary)}
.opt-text code{font-family:var(--ui-font-mono);font-size:11px;overflow-wrap:anywhere}
.proto-option.is-active .opt-text b{color:var(--ui-ink)}

.test-row{display:flex;align-items:center;gap:10px}
.test-text{font-size:12.5px;color:var(--ui-text-secondary);display:inline-flex;align-items:center;gap:6px;min-width:0}
.test-text.is-ok{color:var(--ui-success-text)}
.test-text.is-fail{color:var(--ui-danger)}

/* Models */
.empty{margin:0;padding:18px 16px;text-align:center;font-size:12px;color:var(--ui-text-secondary)}
.mrow{border-top:1px solid var(--ui-line)}
.mrow:first-child{border-top:0}
.mrow-main{display:flex;align-items:center;gap:10px;min-height:48px;padding:7px 10px 7px 14px}
.star{width:22px;height:22px;border:0;padding:0;background:transparent;display:grid;place-items:center;cursor:pointer;color:var(--ui-text-disabled)}
.star:hover:not(.on){color:var(--ui-text-tertiary)}
.star.on{color:var(--ui-btn)}
.star.on :deep(svg){fill:currentColor}
.mname{flex:1;min-width:0;display:flex;flex-direction:column;gap:1px}
.mid{display:flex;align-items:center;gap:7px;min-width:0}
.mid code{font-family:var(--ui-font-mono);font-size:13px;font-weight:500;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.def-tag{height:18px;padding:0 6px;border-radius:5px;background:var(--ui-soft);color:var(--ui-ink);font-size:10.5px;font-weight:600;font-style:normal;display:inline-flex;align-items:center;flex-shrink:0}
.msub{font-size:11.5px;color:var(--ui-text-secondary)}
.mrow .xb-icon-btn.on{background:var(--ui-soft);color:var(--ui-ink)}
.mpanel{margin:0 14px 12px 46px;padding:12px;border-radius:10px;background:var(--ui-surface-sunken);display:flex;flex-direction:column;gap:9px}
.mpanel-row{display:flex;align-items:center;gap:10px}
.mpanel-row.levels{flex-wrap:wrap;gap:6px;padding-left:66px}
.mpanel-label{width:56px;flex-shrink:0;font-size:12.5px;font-weight:500}
.mpanel .xb-seg{flex:1;background:var(--ui-control)}
.mpanel .xb-seg>button{flex:1}
.mpanel-hint{font-size:11.5px;color:var(--ui-text-secondary);padding-left:66px}
.xb-chip.level{height:26px;cursor:pointer;border:0;font-family:inherit}
.xb-chip.level.on{background:var(--ui-soft);color:var(--ui-ink);box-shadow:inset 0 0 0 1.5px var(--ui-btn)}
.addbar{border-top:1px solid var(--ui-line);padding:12px 14px;display:flex;flex-direction:column;gap:10px}
.mrow+.addbar,.empty+.addbar{border-top:1px solid var(--ui-line)}
.addbar:first-child{border-top:0}
.addline{display:flex;gap:8px}
.addline .n-input{flex:1;min-width:0}
.addline .xb-btn{flex-shrink:0}
.chips{display:flex;flex-wrap:wrap;align-items:center;gap:6px}
.chips-label{font-size:11.5px;color:var(--ui-text-secondary);margin-right:2px}
.fetched{display:flex;flex-wrap:wrap;gap:6px;max-height:112px;overflow-y:auto;flex-basis:100%}
.model-chip{height:26px;border:0;cursor:pointer;display:inline-flex;align-items:center;gap:5px;font-family:var(--ui-font-mono);font-size:11.5px}
.mfoot{display:flex;align-items:center;gap:14px;padding:11px 14px;border-top:1px solid var(--ui-line)}
.form-error{margin-bottom:6px}

.pv-foot{display:flex;align-items:center;gap:8px;padding:14px 18px 16px 22px;background:var(--ui-surface);border-top:1px solid var(--ui-line)}
.del-link{height:32px;padding:0 4px;border:0;background:transparent;font:inherit;font-size:13px;color:var(--ui-danger);cursor:pointer}
.del-link:disabled{opacity:.5;cursor:not-allowed}

/* Tools */
.ws-path code{font-family:var(--ui-font-mono);font-size:11.5px;overflow-wrap:anywhere;color:var(--ui-text-primary)}
.workspace-row{align-items:flex-start}
.ws-actions{display:flex;flex-direction:column;align-items:stretch;gap:6px;flex-shrink:0}
.ai-note{margin:14px 4px 0;font-size:12px;line-height:1.7;color:var(--ui-text-tertiary)}
</style>
