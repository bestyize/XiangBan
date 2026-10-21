<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue';
import { Check, ChevronDown, ChevronRight, Code2, Settings } from 'lucide-vue-next';
import { GATEWAY_PROVIDER, invoke, toAccountError, type AiConfigView, type AiReplyPreference, type AiTier, type ProviderConfigView } from '../../bridge.ts';
import { providerReasoningLevels, preferenceOptions } from '../ai-config.ts';

const props = defineProps<{ label: string; disabled?: boolean }>();
const emit = defineEmits<{ switched: []; settings: [] }>();
const root = ref<HTMLElement | null>(null);
const open = ref(false);
const busy = ref(false);
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
function outside(e: PointerEvent) { if (root.value && !root.value.contains(e.target as Node) && !reasoningPanel.value?.contains(e.target as Node)) close(); }
function escape(e: KeyboardEvent) { if (e.key === 'Escape') { if (reasoningTarget.value) hideReasoning(); else close(); } }
function close() { hideReasoning(); open.value = false; ++generation; }
watch(open, value => {
  if (value) { document.addEventListener('pointerdown', outside); document.addEventListener('keydown', escape); window.addEventListener('resize', hideReasoning); }
  else { document.removeEventListener('pointerdown', outside); document.removeEventListener('keydown', escape); window.removeEventListener('resize', hideReasoning); }
});
onBeforeUnmount(() => { ++generation; hideReasoning(); document.removeEventListener('pointerdown', outside); document.removeEventListener('keydown', escape); window.removeEventListener('resize', hideReasoning); });
watch(() => props.disabled, disabled => { if (disabled) close(); });
async function toggle() {
  if (open.value) { close(); return; }
  open.value = true; error.value = ''; gatewayError.value = ''; tiers.value = []; busy.value = true;
  const id = ++generation;
  try {
    const initial = await invoke<AiConfigView>('get_ai_config');
    if (id !== generation) return;
    config.value = initial;
    const [refresh, modes] = await Promise.allSettled([
      invoke<AiConfigView>('refresh_ai_providers', { force: false }),
      initial.gatewayEnabled === false ? Promise.resolve([] as AiTier[]) : invoke<AiTier[]>('ai_gateway_tiers'),
    ]);
    if (id !== generation) return;
    if (refresh.status === 'fulfilled') config.value = refresh.value;
    if (modes.status === 'fulfilled') {
      tiers.value = modes.value;
      const latest = await invoke<AiConfigView>('get_ai_config');
      if (id !== generation) return;
      config.value = latest;
    }
    else gatewayError.value = toAccountError(modes.reason).message;
  } catch (e) { if (id === generation) error.value = String(e); }
  finally { if (id === generation) busy.value = false; }
}
async function chooseGateway(tier: AiTier, preference?: AiReplyPreference) {
  if (busy.value || props.disabled || !tier.available || !gatewayEnabled.value) return;
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
  if (busy.value || props.disabled) return;
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
    <button type="button" class="composer-model" data-testid="chat-workbench-model" :title="label" :disabled="disabled" :aria-expanded="open" aria-haspopup="menu" @click="toggle"><Code2 :size="14" /><span>{{ label }}</span><ChevronDown :size="12" /></button>
    <div v-if="open" class="model-panel" role="menu" aria-label="选择模型" @scroll="hideReasoning">
      <div class="model-heading">选择模型</div>
      <div v-if="busy && !config" class="model-state">读取已配置模型…</div>
      <div v-if="gatewayEnabled" class="model-group" data-testid="chat-model-gateway">
        <div class="provider-name">小伴 AI</div>
        <button v-for="tier in tiers" :key="tier.tier" type="button" class="model-choice" :title="tier.description" :disabled="busy || !tier.available" :class="{ selected: currentTier(tier) }" :data-testid="`chat-model-xiaoban-ai-${tier.tier}`" aria-haspopup="menu" :aria-expanded="targetTier(tier)" @mouseenter="showReasoning({ kind: 'gateway', tier }, $event)" @mouseleave="leaveReasoning" @focus="showReasoning({ kind: 'gateway', tier }, $event)" @keydown.right.prevent="showReasoning({ kind: 'gateway', tier }, $event, true)" @click="chooseGateway(tier)">
          <span>{{ tier.displayName }}{{ !tier.available ? '（暂不可用）' : '' }}</span><Check v-if="currentTier(tier)" :size="14" /><ChevronRight :size="12" />
        </button>
        <p v-if="gatewayError" class="model-state error">{{ gatewayError }}</p>
        <p v-else-if="!tiers.length" class="model-state">{{ busy ? '读取使用模式…' : '暂无可用模式，请到模型设置查看。' }}</p>
      </div>
      <div v-for="group in providers" :key="group.provider.id" class="model-group">
        <div class="provider-name">{{ group.provider.name }}</div>
        <button v-for="item in group.models" :key="item.model" type="button" class="model-choice" :title="item.model" :disabled="busy" :class="{ selected: current(item) }" :data-testid="`chat-model-${group.provider.id}-${item.model}`" aria-haspopup="menu" :aria-expanded="targetModel(item)" @mouseenter="showReasoning({ kind: 'model', entry: item }, $event)" @mouseleave="leaveReasoning" @focus="showReasoning({ kind: 'model', entry: item }, $event)" @keydown.right.prevent="showReasoning({ kind: 'model', entry: item }, $event, true)" @click="choose(item)">
          <span>{{ item.name }}</span><Check v-if="current(item)" :size="14" /><ChevronRight :size="12" />
        </button>
      </div>
      <p v-if="error" class="model-state error">{{ error }}</p>
      <p v-if="!busy && !providers.length && !gatewayEnabled && !error" class="model-state">暂无已配置的可用模型</p>
      <button type="button" class="model-settings" @click="settings"><Settings :size="13" />管理模型</button>
    </div>
  </div>
  <Teleport to="body">
    <div v-if="open && reasoningTarget" ref="reasoningPanel" class="reasoning-panel" :style="reasoningPosition" role="menu" aria-label="思考强度" data-testid="chat-model-reasoning" @mouseenter="keepReasoning" @mouseleave="leaveReasoning" @keydown.left.prevent="hideReasoning">
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
.composer-model{display:flex;align-items:center;gap:5px;min-width:0;max-width:166px;height:30px;padding:0 7px;border:0;border-radius:8px;background:none;color:var(--ui-text-secondary);cursor:pointer;font:inherit;font-size:12px}
.composer-model:hover,.composer-model[aria-expanded="true"]{background:var(--ui-surface-alt);color:var(--ui-text-primary)}
.composer-model:disabled{opacity:.5;cursor:not-allowed}
.composer-model span{max-width:118px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.composer-model svg{flex:none}
.model-panel{position:absolute;z-index:40;bottom:calc(100% + 8px);right:0;width:max-content;min-width:210px;max-width:min(305px,calc(100vw - 20px));max-height:350px;overflow-y:auto;padding:7px 5px 4px;border-radius:12px;background:var(--ui-surface);border:1px solid var(--ui-border);box-shadow:0 12px 32px rgba(0,0,0,.14);box-sizing:border-box;scrollbar-width:thin}
.model-heading{font-size:12px;font-weight:650;padding:6px 9px}
.provider-name{font-size:11px;color:var(--ui-text-tertiary);padding:10px 9px 4px;border-top:1px solid var(--ui-border);margin-top:3px}
.model-choice{width:100%;height:33px;display:flex;align-items:center;justify-content:space-between;gap:12px;padding:0 9px;background:none;color:var(--ui-text-primary);border:0;border-radius:7px;cursor:pointer;text-align:left;font:inherit;font-size:12px}
.model-choice:hover,.model-choice.selected{background:var(--ui-surface-alt)}
.model-choice>span{flex:1;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.model-choice svg{flex:none}
.model-choice:disabled{cursor:wait;opacity:.7}
.model-state{color:var(--ui-text-tertiary);font-size:11px;line-height:1.6;padding:4px 9px}
.model-state.error{color:var(--ui-primary)}
.reasoning-panel{position:fixed;z-index:1000;overflow-y:auto;padding:7px 5px 4px;border:1px solid var(--ui-border);border-radius:12px;background:var(--ui-surface);color:var(--ui-text-primary);box-shadow:0 12px 32px rgba(0,0,0,.14);box-sizing:border-box}
.reasoning-model{padding:0 9px 6px;color:var(--ui-text-tertiary);font-size:11px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.model-settings{display:flex;align-items:center;justify-content:center;gap:6px;width:100%;padding:10px 7px 7px;margin-top:6px;border:0;border-top:1px solid var(--ui-border);background:none;color:var(--ui-primary);cursor:pointer;font:inherit;font-size:12px}
@media(max-width:680px){.composer-model{max-width:96px}.composer-model span{max-width:52px}}
</style>
