<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { Check, ChevronDown, ChevronRight, Settings } from 'lucide-vue-next';
import { GATEWAY_PROVIDER, invoke, toAccountError, type AiConfigView, type AiReplyPreference, type AiTier, type ProviderConfigView } from '../../bridge.ts';
import { providerReasoningLevels, preferenceOptions } from '../ai-config.ts';

const props = defineProps<{ label: string; disabled?: boolean; /** The service the model comes from, for the small badge; empty when unknown. */ provider?: string; /** The model is 小伴 AI. */ gateway?: boolean }>();
/** Two letters of the service's name (or its first character), like a tiny logo. */
const badge = computed(() => {
  if (props.gateway) return '伴';
  const name = (props.provider || props.label || '').trim();
  const latin = name.replace(/[^A-Za-z0-9]/g, '');
  return latin ? latin.slice(0, 2).toUpperCase() : name.slice(0, 1) || 'AI';
});
const providerBadge = (name: string) => { const latin = name.replace(/[^A-Za-z0-9]/g, ''); return latin ? latin.slice(0, 2).toUpperCase() : name.slice(0, 1); };
const emit = defineEmits<{ switched: []; settings: [] }>();
const root = ref<HTMLElement | null>(null);
const open = ref(false);
const busy = ref(false);
const loading = ref(false);
const loadingTiers = ref(false);
const error = ref('');
const config = ref<AiConfigView | null>(null);
const tiers = ref<AiTier[]>([]);
const gatewayError = ref('');
const gatewayEnabled = computed(() => config.value && config.value.gatewayEnabled !== false);
const currentTier = (tier: AiTier) => config.value?.activeProvider === GATEWAY_PROVIDER
  && (config.value.gatewayTier || config.value.model) === tier.tier;
type ModelChoice = { provider: ProviderConfigView; model: string; name: string };
const providers = computed(() => (config.value?.providers ?? [])
  .filter(p => p.available !== false && p.baseUrl && (p.hasKey || p.isCustom || p.type === 'free'))
  .map(p => ({
    provider: p,
    models: (p.models?.length ? p.models : [{ id: p.model, name: p.model }])
      .filter(m => m.id)
      .map(m => ({ provider: p, model: m.id, name: m.name || m.id })),
  })).filter(p => p.models.length));
const current = (entry: ModelChoice) => config.value?.activeProvider === entry.provider.id && config.value?.model === entry.model;
type ReasoningTarget = { kind: 'model'; entry: ModelChoice } | { kind: 'gateway'; tier: AiTier };
const reasoningTarget = ref<ReasoningTarget | null>(null);
const reasoningPanel = ref<HTMLElement | null>(null);
const reasoningPosition = ref({ left: '0px', top: '0px', width: '210px', maxHeight: '300px' });
const canTune = (entry: ModelChoice) => providerReasoningLevels(entry.provider, entry.model).length > 0;
const reasoningOptions = computed(() => preferenceOptions(reasoningTarget.value?.kind === 'model'
  ? providerReasoningLevels(reasoningTarget.value.entry.provider, reasoningTarget.value.entry.model)
  : reasoningTarget.value?.tier.reasoningEfforts ?? []));
const reasoningNote = computed(() => reasoningOptions.value.length === 1
  ? reasoningTarget.value?.kind === 'gateway' ? '此模式尚未提供可选等级，请更新网关或由管理员配置。'
    : reasoningTarget.value?.entry.provider.protocol === 'anthropic' ? '此协议暂未接入思考等级，可切换为 OpenAI 兼容协议。'
    : '尚未识别此模型的等级，可在模型设置的高级设置中声明。'
  : '默认沿用服务商设置。更高等级可能耗时更久、消耗更多额度。');
const targetModel = (entry: ModelChoice) => reasoningTarget.value?.kind === 'model'
  && reasoningTarget.value.entry.provider.id === entry.provider.id && reasoningTarget.value.entry.model === entry.model;
const targetTier = (tier: AiTier) => reasoningTarget.value?.kind === 'gateway' && reasoningTarget.value.tier.tier === tier.tier;
function selectedStrength(value: AiReplyPreference) {
  const target = reasoningTarget.value;
  return target?.kind === 'gateway' ? currentTier(target.tier) && value === (config.value?.gatewayReplyPreference ?? 'default')
    : !!target && current(target.entry) && (canTune(target.entry) ? target.entry.provider.replyPreference ?? 'default' : 'default') === value;
}
let hoverTimer: ReturnType<typeof setTimeout> | undefined;
function keepReasoning() { clearTimeout(hoverTimer); }
function hideReasoning() { keepReasoning(); reasoningTarget.value = null; }
function leaveReasoning() { keepReasoning(); hoverTimer = setTimeout(hideReasoning, 180); }
async function showReasoning(target: ReasoningTarget, event: Event, focus = false) {
  if (busy.value || props.disabled) return;
  keepReasoning();
  const anchor = event.currentTarget as HTMLElement;
  const rect = anchor.getBoundingClientRect();
  const gap = 6;
  const width = Math.min(210, window.innerWidth - 16);
  const left = rect.right + gap + width <= window.innerWidth - 8 ? rect.right + gap : Math.max(8, rect.left - gap - width);
  reasoningTarget.value = target;
  reasoningPosition.value = { left: `${left}px`, top: `${Math.max(8, rect.top)}px`, width: `${width}px`, maxHeight: `${window.innerHeight - 16}px` };
  await nextTick();
  if (target.kind === 'model' ? !targetModel(target.entry) : !targetTier(target.tier)) return;
  const height = reasoningPanel.value?.getBoundingClientRect().height ?? 0;
  reasoningPosition.value.top = `${Math.max(8, Math.min(rect.top, window.innerHeight - height - 8))}px`;
  if (focus) reasoningPanel.value?.querySelector<HTMLButtonElement>('button')?.focus();
}
async function chooseStrength(preference: AiReplyPreference) {
  const target = reasoningTarget.value;
  if (target?.kind === 'model') await choose(target.entry, preference);
  else if (target?.kind === 'gateway') await chooseGateway(target.tier, preference);
}
let generation = 0;
let pendingGeneration: number | undefined;
function outside(e: PointerEvent) { if (root.value && !root.value.contains(e.target as Node) && !reasoningPanel.value?.contains(e.target as Node)) close(); }
function escape(e: KeyboardEvent) { if (e.key === 'Escape') { if (reasoningTarget.value) hideReasoning(); else close(); } }
function close() { hideReasoning(); open.value = false; ++generation; loading.value = false; loadingTiers.value = false; }
watch(open, value => {
  if (value) { document.addEventListener('pointerdown', outside); document.addEventListener('keydown', escape); window.addEventListener('resize', hideReasoning); }
  else { document.removeEventListener('pointerdown', outside); document.removeEventListener('keydown', escape); window.removeEventListener('resize', hideReasoning); }
});
onBeforeUnmount(() => { ++generation; hideReasoning(); document.removeEventListener('pointerdown', outside); document.removeEventListener('keydown', escape); window.removeEventListener('resize', hideReasoning); });
watch(() => props.disabled, disabled => { if (disabled) close(); });
onMounted(() => { void load(); });
function toggle() {
  if (open.value) { close(); return; }
  open.value = true;
  void load();
}
async function load() {
  if (pendingGeneration === generation) return;
  error.value = ''; gatewayError.value = ''; loading.value = true;
  const id = ++generation;
  pendingGeneration = id;
  try {
    const initial = await invoke<AiConfigView>('get_ai_config');
    if (id !== generation) return;
    config.value = initial;
    loading.value = false;
    loadingTiers.value = initial.gatewayEnabled !== false;
    // Keep the last list visible. Each source updates independently, so a slow
    // catalogue request cannot hold gateway modes or local model selection up.
    await Promise.allSettled([
      (async () => {
        try {
          const refreshed = await invoke<AiConfigView>('refresh_ai_providers', { force: false });
          if (id === generation) config.value = refreshed;
        } catch { /* The locally saved providers remain usable while offline. */ }
      })(),
      (async () => {
        try {
          if (initial.gatewayEnabled === false) return;
          const modes = await invoke<AiTier[]>('ai_gateway_tiers');
          if (id !== generation) return;
          tiers.value = modes;
          loadingTiers.value = false;
          const latest = await invoke<AiConfigView>('get_ai_config');
          if (id === generation) config.value = latest;
        } catch (e) { if (id === generation) gatewayError.value = toAccountError(e).message; }
        finally { if (id === generation) loadingTiers.value = false; }
      })(),
    ]);
  } catch (e) { if (id === generation) error.value = String(e); }
  finally {
    if (id === generation) loading.value = false;
    if (pendingGeneration === id) pendingGeneration = undefined;
  }
}
async function chooseGateway(tier: AiTier, preference?: AiReplyPreference) {
  if (busy.value || loading.value || loadingTiers.value || props.disabled || !tier.available || !gatewayEnabled.value) return;
  if (currentTier(tier) && (preference === undefined || preference === (config.value?.gatewayReplyPreference ?? 'default'))) { close(); return; }
  busy.value = true; error.value = '';
  try {
    await invoke('use_ai_gateway', { tier: tier.tier, ...(preference !== undefined ? { replyPreference: preference } : {}) });
    close();
    emit('switched');
  } catch (e) { error.value = toAccountError(e).message; }
  finally { busy.value = false; }
}
async function choose(entry: ModelChoice, preference?: AiReplyPreference) {
  if (busy.value || loading.value || props.disabled) return;
  if (current(entry) && (preference === undefined || preference === (entry.provider.replyPreference ?? 'default'))) { close(); return; }
  busy.value = true; error.value = '';
  const p = entry.provider;
  try {
    await invoke('save_ai_provider', { providerId: p.id, name: p.name, protocol: p.protocol, baseUrl: p.baseUrl,
      model: entry.model, apiKey: null, clearKey: false, activate: true, selectionMode: 'manual',
      ...(preference !== undefined ? { replyPreference: canTune(entry) ? preference : 'default' } : {}) });
    close();
    emit('switched');
  } catch (e) { error.value = String(e); }
  finally { busy.value = false; }
}
function settings() { close(); emit('settings'); }
</script>

<template>
  <div ref="root" class="model-anchor">
    <button type="button" class="composer-model" data-testid="chat-workbench-model" :title="provider ? `${provider} · ${label}` : label" :disabled="disabled" :aria-expanded="open" aria-haspopup="menu" @click="toggle"><i class="model-badge" :class="{ brand: gateway }" aria-hidden="true">{{ badge }}</i><span>{{ label }}</span><ChevronDown :size="11" :stroke-width="2.4" /></button>
    <div v-if="open" class="model-panel xb-menu" role="menu" aria-label="选择模型" @scroll="hideReasoning">
      <div class="model-heading">选择模型</div>
      <div v-if="loading && !config" class="model-state">读取已配置模型…</div>
      <div v-if="gatewayEnabled" class="model-group" data-testid="chat-model-gateway">
        <div class="provider-name"><i class="model-badge brand" aria-hidden="true">伴</i>小伴 AI</div>
        <button v-for="tier in tiers" :key="tier.tier" type="button" class="model-choice" :title="tier.description" :disabled="busy || loading || loadingTiers || !tier.available" :class="{ selected: currentTier(tier) }" :data-testid="`chat-model-xiaoban-ai-${tier.tier}`" aria-haspopup="menu" :aria-expanded="targetTier(tier)" @mouseenter="showReasoning({ kind: 'gateway', tier }, $event)" @mouseleave="leaveReasoning" @focus="showReasoning({ kind: 'gateway', tier }, $event)" @keydown.right.prevent="showReasoning({ kind: 'gateway', tier }, $event, true)" @click="chooseGateway(tier)">
          <span>{{ tier.displayName }}{{ !tier.available ? '（暂不可用）' : '' }}</span><Check v-if="currentTier(tier)" :size="14" /><ChevronRight :size="12" />
        </button>
        <p v-if="gatewayError" class="model-state error">{{ gatewayError }}</p>
        <div v-else-if="!tiers.length && loadingTiers" class="mode-skeleton" role="status" aria-label="正在加载使用模式"><div v-for="row in 2" :key="row" class="model-choice" aria-hidden="true"><span></span></div></div>
        <p v-else-if="!tiers.length" class="model-state">暂无可用模式，请到模型设置查看。</p>
      </div>
      <div v-for="group in providers" :key="group.provider.id" class="model-group">
        <div class="provider-name"><i class="model-badge" aria-hidden="true">{{ providerBadge(group.provider.name) }}</i>{{ group.provider.name }}</div>
        <button v-for="item in group.models" :key="item.model" type="button" class="model-choice" :title="item.model" :disabled="busy || loading" :class="{ selected: current(item) }" :data-testid="`chat-model-${group.provider.id}-${item.model}`" aria-haspopup="menu" :aria-expanded="targetModel(item)" @mouseenter="showReasoning({ kind: 'model', entry: item }, $event)" @mouseleave="leaveReasoning" @focus="showReasoning({ kind: 'model', entry: item }, $event)" @keydown.right.prevent="showReasoning({ kind: 'model', entry: item }, $event, true)" @click="choose(item)">
          <span>{{ item.name }}</span><Check v-if="current(item)" :size="14" /><ChevronRight :size="12" />
        </button>
      </div>
      <p v-if="error" class="model-state error">{{ error }}</p>
      <p v-if="!loading && !providers.length && !gatewayEnabled && !error" class="model-state">暂无已配置的可用模型</p>
      <button type="button" class="model-settings" @click="settings"><Settings :size="13" />管理模型</button>
    </div>
  </div>
  <Teleport to="body">
    <div v-if="open && reasoningTarget" ref="reasoningPanel" class="reasoning-panel xb-menu" :style="reasoningPosition" role="menu" aria-label="思考强度" data-testid="chat-model-reasoning" @mouseenter="keepReasoning" @mouseleave="leaveReasoning" @keydown.left.prevent="hideReasoning">
      <div class="model-heading">思考强度</div>
      <div class="reasoning-model">{{ reasoningTarget.kind === 'model' ? reasoningTarget.entry.name : `小伴 AI · ${reasoningTarget.tier.displayName}` }}</div>
      <button v-for="option in reasoningOptions" :key="option.value" type="button" class="model-choice" role="menuitemradio" :aria-checked="selectedStrength(option.value)" :disabled="busy" :data-testid="`chat-model-strength-${option.value}`" @click="chooseStrength(option.value)">
        <span>{{ option.label }}</span><Check v-if="selectedStrength(option.value)" :size="14" />
      </button>
      <p class="model-state">{{ reasoningNote }}</p>
    </div>
  </Teleport>
</template>

<style scoped>
.model-anchor{position:relative;display:flex;flex:none;min-width:0}
.composer-model{display:flex;align-items:center;gap:6px;min-width:0;max-width:200px;height:28px;padding:0 8px;border:0;border-radius:8px;background:none;color:var(--ui-text-primary);cursor:pointer;font:inherit;font-size:12px;font-weight:500}
.composer-model:hover,.composer-model[aria-expanded="true"]{background:var(--ui-control)}
.composer-model:disabled{opacity:.5;cursor:not-allowed}
.composer-model span{max-width:150px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-family:var(--ui-font-mono);font-size:12px}
.composer-model svg{flex:none;color:var(--ui-text-secondary)}
.model-badge{flex:none;width:16px;height:16px;border-radius:4px;background:var(--ui-soft);color:var(--ui-ink);font-style:normal;font-size:8.5px;font-weight:600;line-height:16px;text-align:center;letter-spacing:-.02em;font-family:var(--ui-font-family)}
.model-badge.brand{background:linear-gradient(160deg,var(--ui-btn-hi),var(--ui-btn));color:var(--ui-text-on-btn);font-size:9px}
.model-panel{position:absolute;bottom:calc(100% + 8px);right:0;width:max-content;min-width:230px;max-width:min(320px,calc(100vw - 20px));max-height:360px;overflow-y:auto;box-sizing:border-box;scrollbar-width:thin}
.model-heading{font-size:12px;font-weight:500;color:var(--ui-text-tertiary);padding:6px 10px 4px}
.provider-name{display:flex;align-items:center;gap:7px;font-size:11.5px;font-weight:500;color:var(--ui-text-secondary);padding:9px 10px 4px;border-top:1px solid var(--ui-line);margin-top:4px}
.model-choice{width:100%;height:32px;display:flex;align-items:center;justify-content:space-between;gap:12px;padding:0 10px;background:none;color:var(--ui-text-primary);border:0;border-radius:8px;cursor:pointer;text-align:left;font:inherit;font-size:13px}
.model-choice:hover,.model-choice[aria-expanded="true"]{background:var(--ui-surface-hover)}
.model-choice.selected,.model-choice[aria-checked="true"]{background:var(--ui-soft);color:var(--ui-ink)}
.model-choice>span{flex:1;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.model-choice svg{flex:none}
.model-choice svg:last-child{color:var(--ui-text-tertiary)}
.model-choice:disabled{cursor:not-allowed;opacity:.55}
.model-state{color:var(--ui-text-tertiary);font-size:11.5px;line-height:1.6;padding:4px 10px;margin:0}
.model-state.error{color:var(--ui-danger)}
.mode-skeleton .model-choice{cursor:default}.mode-skeleton .model-choice>span{flex:none;width:88px;height:12px;border-radius:4px;background:var(--ui-control)}
.reasoning-panel{position:fixed;z-index:1000;overflow-y:auto;box-sizing:border-box}
.reasoning-model{padding:0 10px 6px;color:var(--ui-text-tertiary);font-size:11.5px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.model-settings{display:flex;align-items:center;gap:8px;width:100%;min-height:32px;padding:0 10px;margin-top:4px;border:0;border-top:1px solid var(--ui-line);border-radius:0 0 8px 8px;background:none;color:var(--ui-text-secondary);cursor:pointer;font:inherit;font-size:12.5px}
.model-settings:hover{color:var(--ui-text-primary)}
@media(max-width:680px){.composer-model{max-width:110px}.composer-model span{max-width:60px}}
</style>
