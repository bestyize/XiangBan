<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, reactive, ref, watch } from 'vue';
import { NAlert, NButton, NModal } from 'naive-ui';
import { ArrowUpRight, Check, CircleAlert, CircleCheck, LoaderCircle, RefreshCw, Settings, Sparkles, Sprout, UserRound } from 'lucide-vue-next';
import { GATEWAY_PROVIDER, desktop as nativeDesktop, invoke, toAccountError, type AiBalance, type AiConfigView, type AiRedeemed, type AiProbeResult, type AiTier } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import { closeConfigurationWindow, openModelWindow } from '../configuration/windows.ts';
import { bucketLabel, expiryText, formatAiCredits, pickTier } from '../ai-gateway.ts';

const props = defineProps<{ store: ControlStore; config: AiConfigView | null; detached?: boolean; initialOpen?: boolean; hideSummary?: boolean }>();
const emit = defineEmits<{ config: [view: AiConfigView]; close: [] }>();

const open = ref(!!props.detached || !!props.initialOpen);
watch(open, value => { if (!value) emit('close'); });
watch(open, value => { if (props.detached && !value) void closeConfigurationWindow().catch(error => props.store.notify(error, 'error')); });
const balance = ref<AiBalance | null>(null);
const tiers = ref<AiTier[]>([]);
const picked = ref('');
const loading = ref(false);
const loadError = ref('');
const code = ref('');
const redeeming = ref(false);
const redeemError = ref('');
const redeemDone = ref('');
const enabling = ref(false);
const enableError = ref('');
const probes = reactive<Record<string, { state: 'checking' | 'success' | 'failure'; message: string }>>({});
let disposed = false;
onBeforeUnmount(() => { disposed = true; });
const probing = computed(() => Object.values(probes).some(p => p.state === 'checking'));
const purchaseUrl = computed(() => {
  try {
    const url = new URL(balance.value?.creditPurchaseUrl ?? '');
    return ['https:', 'http:'].includes(url.protocol) && !url.username && !url.password ? url.href : '';
  } catch { return ''; }
});
async function purchase() {
  if (!purchaseUrl.value) return;
  try { await invoke('open_download_url', { url: purchaseUrl.value }); }
  catch (error) { props.store.notify(toAccountError(error).message, 'error'); }
}
async function probe(tier: AiTier) {
  if (!uid.value || !tier.available || probing.value) return;
  const owner = uid.value;
  const state = { state: 'checking' as const, message: `正在向 ${tier.displayName} 发送 hi` };
  probes[tier.tier] = state;
  try {
    const result = await invoke<AiProbeResult>('ai_gateway_probe', { tier: tier.tier });
    if (disposed || uid.value !== owner || probes[tier.tier]?.state !== 'checking') return;
    probes[tier.tier] = { state: 'success', message: `检测成功 · ${(result.elapsedMs / 1000).toFixed(2)} 秒 · 回复：${result.reply}` };
  } catch (error) {
    if (disposed || uid.value !== owner || probes[tier.tier]?.state !== 'checking') return;
    probes[tier.tier] = { state: 'failure', message: `检测失败：${toAccountError(error).message}` };
  }
}
function mark(tier: AiTier) {
  const name = tier.displayName.toLowerCase();
  return name.startsWith('claude') ? 'A' : name.startsWith('deepseek') ? 'D' : name.startsWith('gemini') ? 'G' : 'M';
}
function cancel() { picked.value = pickTier(tiers.value, activeTier.value, ''); open.value = false; }
function shortExpiry(value: string | null) {
  if (!value) return '长期有效';
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return expiryText(value);
  const day = `${date.getMonth() + 1}/${date.getDate()}`;
  return date.getFullYear() === new Date().getFullYear() ? `${day} 到期` : `${date.getFullYear()}/${day}`;
}
const modelList = ref<HTMLElement>();

const uid = computed(() => props.store.account?.user.uid ?? '');
const perDollar = computed(() => balance.value?.quotaPerUnit ?? 500_000);
const gatewayActive = computed(() => props.config?.activeProvider === GATEWAY_PROVIDER);
const activeTier = computed(() => (gatewayActive.value ? props.config?.gatewayTier ?? '' : ''));
const activeTierName = computed(() => tiers.value.find(t => t.tier === activeTier.value)?.displayName ?? (activeTier.value ? '已保存的模型' : ''));
const pickedTier = computed(() => tiers.value.find(t => t.tier === picked.value));
const unchanged = computed(() => gatewayActive.value && activeTier.value === picked.value);
const tabbableTier = computed(() => (tiers.value.find(t => t.tier === picked.value && t.available) ?? tiers.value.find(t => t.available))?.tier);
const canEnable = computed(() => !!pickedTier.value?.available && !unchanged.value);

/** Arrow keys select usable models without moving focus from a detection button. */
async function moveSelection(event: KeyboardEvent) {
  if ((event.target as HTMLElement).closest('.probe-button')) return;
  if (!['ArrowDown', 'ArrowUp', 'ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key) || enabling.value) return;
  const usable = tiers.value.filter(t => t.available);
  if (!usable.length) return;
  event.preventDefault();
  const index = usable.findIndex(t => t.tier === picked.value);
  const next = event.key === 'Home' ? 0 : event.key === 'End' ? usable.length - 1
    : index < 0 ? (['ArrowUp', 'ArrowLeft'].includes(event.key) ? usable.length - 1 : 0)
    : (index + (['ArrowUp', 'ArrowLeft'].includes(event.key) ? -1 : 1) + usable.length) % usable.length;
  picked.value = usable[next].tier;
  await nextTick();
  modelList.value?.querySelector<HTMLButtonElement>('[aria-checked="true"]')?.focus();
}

const cardDescription = computed(() => {
  if (!props.store.desktop) return '在桌面应用中使用。';
  if (!uid.value) return '登录小伴账号即可使用，新用户有试用点数，不用自己准备密钥。';
  return '选择小伴 AI 提供的模型，按点数使用，可用兑换码充值。';
});
const cardFact = computed(() => {
  if (gatewayActive.value && activeTierName.value) return { label: '当前模型：', value: activeTierName.value };
  if (uid.value && balance.value) return { label: '可用小伴点：', value: formatAiCredits(balance.value.total, perDollar.value) };
  return null;
});

async function load() {
  const owner = uid.value;
  if (!props.store.desktop || !owner) { balance.value = null; tiers.value = []; return; }
  loading.value = true;
  loadError.value = '';
  const [balanceResult, tiersResult] = await Promise.allSettled([
    invoke<AiBalance>('ai_gateway_balance'),
    invoke<AiTier[]>('ai_gateway_tiers'),
  ]);
  loading.value = false;
  if (uid.value !== owner) return;
  if (balanceResult.status === 'fulfilled') balance.value = balanceResult.value;
  else loadError.value = toAccountError(balanceResult.reason).message;
  if (tiersResult.status === 'fulfilled') {
    tiers.value = tiersResult.value;
    picked.value = pickTier(tiers.value, activeTier.value, picked.value);
  } else if (balanceResult.status === 'fulfilled') {
    loadError.value = toAccountError(tiersResult.reason).message;
  }
}

async function redeem() {
  if (redeeming.value || !code.value.trim()) return;
  redeeming.value = true;
  redeemError.value = '';
  redeemDone.value = '';
  try {
    const done = await invoke<AiRedeemed>('ai_gateway_redeem', { code: code.value });
    redeemDone.value = `兑换成功，到账 ${formatAiCredits(done.quota, perDollar.value)}`;
    code.value = '';
    await load();
  } catch (error) {
    redeemError.value = toAccountError(error).message;
  } finally {
    redeeming.value = false;
  }
}

async function enable() {
  if (enabling.value || !pickedTier.value?.available) return;
  enabling.value = true;
  enableError.value = '';
  try {
    const view = await invoke<AiConfigView>('use_ai_gateway', { tier: picked.value });
    emit('config', view);
    props.store.notify(`聊天已切换到小伴 AI · ${tiers.value.find(t => t.tier === picked.value)?.displayName ?? picked.value}`, 'success');
  } catch (error) {
    enableError.value = toAccountError(error).message;
    // Enabling from the card has no dialog open to show it in.
    if (!open.value) props.store.notify(enableError.value, 'error');
  } finally {
    enabling.value = false;
  }
}

/** The card's own "启用此服务商": the tier that would be preselected in the dialog, with no dialog in between. */
async function activate() {
  if (!props.store.desktop || enabling.value) return;
  if (!uid.value) {
    props.store.notify('请先登录小伴账号，登录后即可启用', 'info');
    openModal();
    return;
  }
  if (!tiers.value.length) await load();
  if (!picked.value) {
    props.store.notify('暂时没有可用的模型，请稍后刷新再试', 'info');
    openModal();
    return;
  }
  await enable();
}

function openModal() {
  if (props.store.desktop && nativeDesktop && !props.detached) { void openModelWindow(GATEWAY_PROVIDER).catch(error => props.store.notify(error, 'error')); return; }
  open.value = true;
  redeemError.value = redeemDone.value = enableError.value = '';
  void load();
}

function signIn() {
  if (props.detached) { props.store.openLogin(); return; }
  open.value = false;
  props.store.openLogin();
}

watch(uid, () => {
  for (const key of Object.keys(probes)) delete probes[key];
  balance.value = null;
  tiers.value = [];
  picked.value = '';
  loadError.value = redeemError.value = redeemDone.value = enableError.value = '';
  void load();
}, { immediate: true });
</script>

<template>
  <template v-if="!detached && !hideSummary">
  <div class="xb-group gateway-card" :class="{ active: gatewayActive }" data-testid="ai-gateway-card">
    <div class="xb-row gw-row">
      <span class="gw-logo" aria-hidden="true"><Sprout :size="22" :stroke-width="2" /></span>
      <div class="xb-row-text" :title="cardDescription" :aria-description="cardDescription">
        <b>小伴 AI</b>
        <span v-if="cardFact" class="gw-fact">{{ cardFact.label }}<strong data-testid="gateway-card-fact">{{ cardFact.value }}</strong></span>
      </div>
      <span v-if="gatewayActive" class="xb-ok" data-testid="gateway-card-status">当前使用</span>
      <span v-else-if="uid" class="gw-status" data-testid="gateway-card-status">可使用</span>
      <span v-else class="gw-status is-warn" data-testid="gateway-card-status">需登录</span>
      <button type="button" class="xb-btn" :disabled="!store.desktop || enabling" data-testid="gateway-open" @click="openModal"><Settings :size="13" />配置</button>
      <button v-if="!gatewayActive" type="button" class="xb-btn-primary" :disabled="!store.desktop || enabling" data-testid="gateway-activate" @click="activate">启用此服务商</button>
      <button v-else type="button" class="xb-btn" disabled data-testid="gateway-in-use">正在使用中</button>
    </div>
  </div>

  </template>
  <component :is="detached ? 'div' : NModal" v-model:show="open" preset="card" title="小伴 AI" :class="{ 'gw-detached': detached }" :style="detached ? undefined : { maxWidth: '640px', width: '92vw' }" data-testid="gateway-modal">
    <div class="mac-config">
      <div v-if="!uid" class="gw-signin" data-testid="gateway-signin">
        <h1>小伴 AI</h1>
        <p>登录小伴账号即可使用模型与共享点数，新用户还有试用点数。</p>
        <button type="button" class="native-button primary" data-testid="gateway-login" @click="signIn">登录 / 注册</button>
      </div>
      <template v-else>
        <NAlert v-if="loadError" class="gw-load-error" type="warning" :show-icon="false" data-testid="gateway-load-error">{{ loadError }}</NAlert>
          <div class="gateway-heading"><h1 class="gateway-page-title"><Sparkles :size="22" :stroke-width="1.7" aria-hidden="true" />小伴 AI</h1><div class="account-actions"><span class="account-name"><UserRound :size="12" aria-hidden="true" />{{ store.account?.user.username }}</span><button type="button" class="refresh-button" :disabled="loading" aria-label="刷新点数" title="刷新点数" data-testid="gateway-refresh" @click="load"><RefreshCw :size="13" :class="{ 'probe-spinner': loading }" aria-hidden="true" /></button></div></div>
        <section class="credit-pane" aria-label="账户与点数">
          <div v-if="balance" class="credit-overview" :title="`${balance.product.name}的所有端共用这份余额；其中付费点数 ${formatAiCredits(balance.paid, perDollar)}`">
            <div class="credit-balance"><div class="section-label credit-label">可用小伴点</div><strong class="credit-total" data-testid="gateway-total">{{ formatAiCredits(balance.total, perDollar) }}</strong></div>
            <span class="sr-only" data-testid="gateway-paid">其中付费点数 {{ formatAiCredits(balance.paid, perDollar) }}</span>
            <span class="sr-only" data-testid="gateway-subtitle">{{ balance.product.name }}的所有端共用这份余额</span>
            <ul v-if="balance.buckets.length" class="credit-packages" aria-label="点数明细" data-testid="gateway-buckets">
              <li v-for="bucket in balance.buckets" :key="bucket.id" class="credit-package" :title="expiryText(bucket.expiresAt)">
                <span class="package-kind">{{ bucketLabel(bucket.kind) }}</span><span class="package-money">{{ formatAiCredits(bucket.remaining, perDollar) }} <em>/ {{ formatAiCredits(bucket.total, perDollar) }}</em></span><time>{{ shortExpiry(bucket.expiresAt) }}</time>
              </li>
            </ul>
            <p v-else-if="balance.trial.available" class="gw-note" data-testid="gateway-trial-hint">首次使用自动发放 {{ formatAiCredits(balance.trial.quota, perDollar) }} 试用点数，{{ balance.trial.days }} 天内有效。</p>
            <p v-else class="gw-note" data-testid="gateway-empty">暂无可用小伴点，可兑换或获取点数。</p>
          </div>
          <div v-else class="credit-overview gw-note" role="status">{{ loading ? '正在加载点数…' : '点数暂不可用' }}</div>
          <form class="redeem-form" @submit.prevent="redeem"><span class="code-field" data-testid="gateway-code"><input v-model="code" aria-label="兑换码" placeholder="输入兑换码" maxlength="64" :disabled="redeeming" /></span><button type="submit" class="native-button" :disabled="redeeming || !code.trim()" data-testid="gateway-redeem"><LoaderCircle v-if="redeeming" class="probe-spinner" :size="12" aria-hidden="true" />兑换</button><button type="button" class="native-button credit-purchase" :disabled="!purchaseUrl" :title="purchaseUrl ? '打开点数获取页面' : '暂未开放'" data-testid="gateway-purchase" @click="purchase">获取点数<ArrowUpRight :size="12" aria-hidden="true" /></button></form>
          <p v-if="redeemError" class="gw-error" role="alert" data-testid="gateway-redeem-error">{{ redeemError }}</p>
          <p v-if="redeemDone" class="gw-done" role="status" data-testid="gateway-redeem-done">{{ redeemDone }}</p>
        </section>
        <section class="model-pane" aria-label="模型" data-testid="gateway-use">
          <div class="pane-heading"><div class="heading-title"><h2>模型</h2><span class="model-count" data-testid="gateway-model-count">{{ tiers.length }} 个</span></div></div>
          <div ref="modelList" class="model-list" role="radiogroup" aria-label="选择模型" @keydown="moveSelection">
            <div v-for="tier in tiers" :key="tier.tier" class="model-row" :class="{ selected: picked === tier.tier }">
              <button type="button" role="radio" class="model-choice" :aria-checked="picked === tier.tier" :tabindex="tier.tier === tabbableTier ? 0 : -1" :disabled="!tier.available || enabling" :title="[tier.displayName, tier.description, !tier.available ? '兑换付费点数后可用' : ''].filter(Boolean).join('；')" :aria-description="[tier.description, !tier.available ? '兑换付费点数后可用' : ''].filter(Boolean).join('；') || undefined" :data-testid="`gateway-tier-${tier.tier}`" @click="picked = tier.tier">
                <span class="model-mark" aria-hidden="true">{{ mark(tier) }}</span><span class="model-name">{{ tier.displayName }}</span><span v-if="tier.tier === activeTier" class="model-state">使用中</span><span v-else-if="!tier.available" class="model-state">需付费点数</span><span v-else-if="tier.isDefault" class="model-state">默认</span><span class="selection-mark" aria-hidden="true"><Check v-if="picked === tier.tier" :size="14" :stroke-width="2.6" /></span>
              </button>
              <span v-if="probes[tier.tier] && probes[tier.tier].state !== 'checking'" class="probe-status" :class="probes[tier.tier].state" role="status" :title="probes[tier.tier].message" :aria-label="probes[tier.tier].message" :data-testid="`gateway-probe-status-${tier.tier}`"><CircleCheck v-if="probes[tier.tier].state === 'success'" :size="14" aria-hidden="true" /><CircleAlert v-else :size="14" aria-hidden="true" /></span>
              <button type="button" class="native-button probe-button" :disabled="!tier.available || probing" :aria-label="`检测 ${tier.displayName}`" :title="probes[tier.tier]?.message ?? `向 ${tier.displayName} 发送 hi，检测是否可用；会消耗少量点数`" :aria-busy="probes[tier.tier]?.state === 'checking'" :data-testid="`gateway-probe-${tier.tier}`" @click="probe(tier)"><LoaderCircle v-if="probes[tier.tier]?.state === 'checking'" class="probe-spinner" :size="12" aria-hidden="true" />{{ probes[tier.tier]?.state === 'checking' ? '检测中' : '检测' }}</button>
            </div>
            <div v-if="!tiers.length" class="model-empty" role="status">{{ loading ? '正在加载模型…' : '暂无可用模型' }}</div>
          </div>
        </section>
        <p v-if="enableError" class="gw-error gw-use-error" role="alert" data-testid="gateway-use-error">{{ enableError }}</p>
        <footer class="window-actions">
          <div class="selection-summary" :title="pickedTier?.displayName"><span>{{ canEnable ? '已选' : '当前模型' }}</span><b>{{ pickedTier?.displayName ?? '请选择模型' }}</b><span v-if="gatewayActive" class="sr-only" data-testid="gateway-active">当前使用 · {{ activeTierName }}</span></div>
          <div class="action-buttons"><button type="button" class="native-button" :disabled="enabling" data-testid="gateway-cancel" @click="cancel">取消</button><button type="button" class="native-button primary" :disabled="!canEnable || enabling" data-testid="gateway-use-btn" @click="enable"><LoaderCircle v-if="enabling" class="probe-spinner" :size="12" aria-hidden="true" />{{ unchanged ? '当前使用中' : '使用此模型' }}</button></div>
        </footer>
      </template>
    </div>
  </component>
</template>

<style scoped>
/* Settings summary card and independent model configuration surface. */
.gateway-card{background:linear-gradient(120deg,var(--ui-surface) 40%,var(--ui-soft))}
.gw-row{padding:16px;gap:14px}
.gw-logo{width:44px;height:44px;flex-shrink:0;border-radius:12px;display:grid;place-items:center;color:#fff;background:linear-gradient(160deg,var(--ui-btn-hi),var(--ui-btn));box-shadow:inset 0 1px 0 rgba(255,255,255,.3)}
.gateway-card .xb-row-text>b{font-size:14px;font-weight:600}
.gw-fact{display:flex;align-items:baseline;gap:4px}
.gw-fact strong{font-weight:600;color:var(--ui-text-primary);font-variant-numeric:tabular-nums}
.gw-status{font-size:12px;color:var(--ui-text-secondary)}
.gw-status.is-warn{color:var(--ui-warning-text)}
.gw-row .xb-btn,.gw-row .xb-btn-primary{flex-shrink:0}
/* The custom window controls stay above the single-page configuration surface. */
.gw-detached{height:100dvh;overflow:hidden}
.gw-signin{padding:32px;display:grid;gap:16px;justify-items:start}.gw-signin p{color:var(--ui-text-secondary);line-height:1.7}
.gw-load-error{margin:16px 32px 0;flex-shrink:0}
.gw-note{font-size:12px;color:var(--ui-text-secondary);line-height:1.6;margin:0}
.gw-error,.gw-done{font-size:12px;line-height:1.5;margin:8px 0 0}.gw-error{color:var(--ui-danger)}.gw-done{color:var(--ui-success-text)}.gw-use-error{padding:0 32px 12px}
.sr-only{position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border:0}
.code-field{display:flex;flex:1;min-width:0}.redeem-form .code-field input{width:100%}
.credit-packages{list-style:none;margin:0;padding:0;max-height:108px;overflow:auto}
.mac-config{min-height:430px;max-height:min(680px,80vh)}.gw-detached .mac-config{min-height:0;max-height:none}

.mac-config{display:flex;flex-direction:column;height:100%;min-height:0;background:var(--ui-configuration-canvas);color:var(--ui-text-primary);font-family:var(--ui-font-family);font-size:13px;-webkit-font-smoothing:antialiased;container-type:inline-size}
.mac-config *{box-sizing:border-box}
button,input{font:inherit}button{cursor:pointer}button:disabled{cursor:not-allowed;color:var(--ui-text-disabled)}
button:focus-visible,input:focus-visible{outline:2px solid var(--ui-btn);outline-offset:2px}
.gateway-heading{display:flex;align-items:center;justify-content:space-between;gap:12px;flex-shrink:0;padding:6px 28px 20px}
.credit-pane{position:relative;flex-shrink:0;margin:0 24px 24px;padding:20px;background:var(--ui-configuration-group);border-radius:14px}
.credit-heading,.account-actions{display:flex;align-items:center;justify-content:space-between;gap:10px}
.section-label{font-size:12px;color:var(--ui-text-secondary)}
.gateway-page-title{display:flex;align-items:center;gap:10px;margin:0;font-size:19px;font-weight:600;letter-spacing:-.3px;line-height:1.3}
.gateway-page-title svg{color:var(--ui-ink)}
.credit-label{margin:0 0 8px}.credit-balance{flex-shrink:0;min-width:140px}
.account-name{display:flex;align-items:center;gap:6px;font-size:12px;color:var(--ui-text-secondary)}
.refresh-button{display:grid;place-items:center;width:24px;height:24px;padding:0;border:0;border-radius:7px;background:transparent;color:var(--ui-text-tertiary)}
.refresh-button:hover{background:var(--ui-surface-hover);color:var(--ui-text-primary)}
.credit-overview{display:flex;align-items:center;gap:24px;margin:0 0 20px}
.credit-total{display:flex;align-items:baseline;flex-shrink:0;min-width:140px;font-size:36px;line-height:1.15;letter-spacing:-1.5px;font-weight:550;font-variant-numeric:tabular-nums}
.credit-packages{flex:1;min-width:0;display:grid;gap:3px}
.credit-package{display:flex;align-items:center;gap:10px;min-height:32px;padding:0;border-radius:0;background:transparent;font-size:12px}
.package-kind{color:var(--ui-text-secondary);white-space:nowrap;font-size:11px}
.package-money{flex:1;font-variant-numeric:tabular-nums;white-space:nowrap}
.package-money em{font-style:normal;color:var(--ui-text-tertiary);font-size:11px}
.credit-package time{color:var(--ui-text-tertiary);font-size:11px;white-space:nowrap}
.redeem-form{display:flex;align-items:center;gap:5px;min-height:46px;padding:5px;background:var(--ui-configuration-canvas);border-radius:10px}
.redeem-form input{flex:1;min-width:0;height:36px;padding:0 12px;border:0;border-radius:7px;background:var(--ui-surface);color:var(--ui-text-primary);font-size:12px;box-shadow:none}
.redeem-form input::placeholder{color:var(--ui-text-tertiary)}
.native-button{display:inline-flex;align-items:center;justify-content:center;gap:5px;height:32px;padding:0 12px;border:0;border-radius:8px;background:transparent;color:var(--ui-text-secondary);box-shadow:none;font-size:12px;font-weight:400;white-space:nowrap}
.native-button:hover:not(:disabled){background:var(--ui-surface-hover);color:var(--ui-text-primary)}
.native-button:active:not(:disabled){transform:translateY(.5px)}
.native-button:disabled{background:transparent;box-shadow:none;color:var(--ui-text-disabled)}
.credit-purchase{color:var(--ui-ink)}
.model-pane{flex:1;min-height:0;display:flex;flex-direction:column;padding:0 24px 0;gap:10px;background:transparent}
.pane-heading{display:flex;align-items:center;justify-content:space-between;gap:16px;flex-shrink:0;padding:0 2px}
.heading-title{display:flex;align-items:baseline;gap:9px}
.heading-title h2{margin:0;font-size:12px;line-height:1.5;font-weight:500;color:var(--ui-text-secondary)}
.model-count{font-size:11px;padding:0;border-radius:0;background:transparent;color:var(--ui-text-secondary);font-variant-numeric:tabular-nums}
.model-list{position:relative;flex:1;min-height:0;overflow-y:auto;scrollbar-gutter:stable;padding:6px;background:var(--ui-configuration-group);border-radius:14px;scrollbar-width:thin;scrollbar-color:var(--ui-scrollbar) transparent}
.model-list::-webkit-scrollbar{width:4px}.model-list::-webkit-scrollbar-thumb{border-radius:5px;background:var(--ui-scrollbar)}
.model-row{position:relative;display:flex;align-items:center;gap:8px;min-height:50px;width:100%;margin-bottom:2px;padding:0 12px;border-radius:9px;background:transparent;color:var(--ui-text-primary)}
.model-choice{display:flex;align-items:center;gap:11px;flex:1;min-width:0;min-height:47px;padding:8px 0;border:0;background:transparent;color:inherit;text-align:left}
.model-row:hover{background:color-mix(in srgb,var(--ui-text-primary) 2.5%,transparent)}
.model-row.selected{background:var(--ui-soft)}
.model-row.selected .model-name{font-weight:550}
.model-row.selected .model-mark{color:var(--ui-ink)}
.model-mark{display:grid;place-items:center;flex-shrink:0;width:28px;height:28px;border-radius:8px;box-shadow:none;background:transparent;font-size:12px;font-weight:500;color:var(--ui-text-secondary)}
.model-name{flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:13px;letter-spacing:-.15px}
.model-state{font-size:11px;color:var(--ui-text-tertiary);white-space:nowrap}
.selection-mark{display:grid;place-items:center;width:16px;height:16px;flex-shrink:0;color:var(--ui-ink)}
.model-choice:disabled .model-mark{opacity:.45}.model-choice:disabled .model-state{color:var(--ui-text-disabled)}
.probe-button{height:26px;padding:0 7px;flex-shrink:0;font-size:11px;color:var(--ui-text-secondary)}
.probe-status{display:flex;flex-shrink:0}.probe-status.success{color:var(--ui-success-text)}.probe-status.failure{color:var(--ui-danger)}
.probe-spinner{animation:probe-spin 1s linear infinite}@keyframes probe-spin{to{transform:rotate(360deg)}}
.window-actions{position:relative;display:flex;align-items:center;justify-content:space-between;gap:18px;flex-shrink:0;min-height:72px;padding:18px 28px 20px;background:transparent}

.selection-summary{display:flex;align-items:center;gap:8px;min-width:0;font-size:11px;color:var(--ui-text-tertiary)}
.selection-summary>span{flex-shrink:0}.selection-summary b{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-weight:400;font-size:12px;color:var(--ui-text-secondary)}
.action-buttons{display:flex;align-items:center;gap:8px;flex-shrink:0}
.native-button.primary{height:34px;padding:0 17px;border-radius:9px;background:var(--ui-btn);color:var(--ui-text-on-btn);box-shadow:none;font-weight:500}
.native-button.primary:hover:not(:disabled){background:var(--ui-btn-hi);color:var(--ui-text-on-btn)}
.native-button.primary:disabled{background:var(--ui-control);color:var(--ui-text-disabled)}
.model-empty{display:grid;place-items:center;min-height:100%;padding:30px;color:var(--ui-text-secondary);font-size:13px}
@container(max-width:480px){.gateway-heading{padding:4px 20px 16px}.gateway-page-title{font-size:18px}.credit-pane{margin:0 16px 20px;padding:16px}.credit-overview{gap:14px;flex-wrap:wrap;margin:0 0 14px}.credit-total{min-width:0;font-size:32px}.credit-packages{flex-basis:100%;gap:0;max-height:66px}.credit-package{min-height:28px;gap:10px}.redeem-form{gap:2px}.redeem-form input{padding-inline:8px}.redeem-form .native-button{padding-inline:7px;font-size:11px}.model-pane{padding:0 16px;gap:10px}.model-row{gap:4px;padding-inline:6px}.model-choice{gap:6px}.model-mark{width:20px}.model-name{font-size:12px}.window-actions{padding:14px 20px 18px;flex-wrap:wrap;gap:10px;min-height:74px}.selection-summary{flex-basis:100%}.action-buttons{margin-left:auto}}
@media(max-height:560px){.gateway-heading{padding-bottom:12px}.credit-pane{flex-shrink:1;min-height:100px;overflow-y:auto;margin-bottom:16px}.model-pane{min-height:90px}.window-actions{min-height:60px;padding-block:12px}}
@media(prefers-reduced-motion:no-preference){button{transition:background-color .15s,color .15s,box-shadow .15s}}
@media(prefers-reduced-motion:reduce){.probe-spinner{animation:none}}

</style>
