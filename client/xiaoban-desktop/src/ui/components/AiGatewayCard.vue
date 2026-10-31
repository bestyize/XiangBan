<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { NAlert, NButton, NInput, NModal, NTag } from 'naive-ui';
import { Check, ChevronDown, Search, Settings, Sprout } from 'lucide-vue-next';
import { GATEWAY_PROVIDER, desktop as nativeDesktop, invoke, toAccountError, type AiBalance, type AiConfigView, type AiRedeemed, type AiTier } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import { closeConfigurationWindow, openModelWindow } from '../configuration/windows.ts';
import { bucketLabel, expiryText, money, pickTier } from '../ai-gateway.ts';

const props = defineProps<{ store: ControlStore; config: AiConfigView | null; detached?: boolean }>();
const emit = defineEmits<{ config: [view: AiConfigView] }>();

const open = ref(!!props.detached);
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
const modelQuery = ref('');
const modelList = ref<HTMLElement>();

const uid = computed(() => props.store.account?.user.uid ?? '');
const perDollar = computed(() => balance.value?.quotaPerUnit ?? 500_000);
const gatewayActive = computed(() => props.config?.activeProvider === GATEWAY_PROVIDER);
const activeTier = computed(() => (gatewayActive.value ? props.config?.gatewayTier ?? '' : ''));
const activeTierName = computed(() => tiers.value.find(t => t.tier === activeTier.value)?.displayName ?? (activeTier.value ? '已保存的模型' : ''));
const pickedTier = computed(() => tiers.value.find(t => t.tier === picked.value));
const unchanged = computed(() => gatewayActive.value && activeTier.value === picked.value);
const filteredTiers = computed(() => {
  const query = modelQuery.value.trim().toLocaleLowerCase();
  return tiers.value.filter(t => !query || t.displayName.toLocaleLowerCase().includes(query));
});
const modelCount = computed(() => modelQuery.value.trim() ? `${filteredTiers.value.length} / ${tiers.value.length}` : String(tiers.value.length));
const tabbableTier = computed(() => (filteredTiers.value.find(t => t.tier === picked.value && t.available) ?? filteredTiers.value.find(t => t.available))?.tier);
const canEnable = computed(() => !!pickedTier.value?.available && !unchanged.value);

/** Keep radio navigation within the visible, usable models. Searching never changes the saved selection. */
async function moveSelection(event: KeyboardEvent) {
  if (!['ArrowDown', 'ArrowUp', 'ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key) || enabling.value) return;
  const usable = filteredTiers.value.filter(t => t.available);
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
  if (!uid.value) return '登录小伴账号即可使用，新用户有试用额度，不用自己准备密钥。';
  return '选择小伴 AI 提供的模型，按额度使用，可用兑换码充值。';
});
const cardFact = computed(() => {
  if (gatewayActive.value && activeTierName.value) return { label: '当前模型：', value: activeTierName.value };
  if (uid.value && balance.value) return { label: '可用额度：', value: money(balance.value.total, perDollar.value) };
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
    redeemDone.value = `兑换成功，到账 ${money(done.quota, perDollar.value)}`;
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
  modelQuery.value = '';
  redeemError.value = redeemDone.value = enableError.value = '';
  void load();
}

function signIn() {
  if (props.detached) { props.store.openLogin(); return; }
  open.value = false;
  props.store.openLogin();
}

watch(uid, () => {
  balance.value = null;
  tiers.value = [];
  picked.value = '';
  modelQuery.value = '';
  loadError.value = redeemError.value = redeemDone.value = enableError.value = '';
  void load();
}, { immediate: true });
</script>

<template>
  <template v-if="!detached">
  <div class="xb-group gateway-card" :class="{ active: gatewayActive }" data-testid="ai-gateway-card">
    <div class="xb-row gw-row">
      <span class="gw-logo" aria-hidden="true"><Sprout :size="22" :stroke-width="2" /></span>
      <div class="xb-row-text">
        <b>小伴 AI</b>
        <span>{{ cardDescription }}</span>
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
    <div class="gw-modal">
      <header v-if="detached" class="gw-window-head"><span class="gw-logo" aria-hidden="true"><Sprout :size="20" /></span><h1>小伴 AI</h1></header>
      <div v-if="!uid" class="gw-signin" data-testid="gateway-signin">
        <p>登录小伴账号后，就能直接用小伴 AI 聊天：不用自己找模型、申请密钥，新用户还有试用额度。有兑换码的话，登录后在这里输入即可充值。</p>
        <NButton type="primary" data-testid="gateway-login" @click="signIn">登录 / 注册</NButton>
      </div>

      <template v-else>
        <NAlert v-if="loadError" type="warning" :show-icon="false" data-testid="gateway-load-error">{{ loadError }}</NAlert>

        <section v-if="balance" class="gw-balance">
          <div class="gw-total">
            <span>当前可用</span>
            <strong data-testid="gateway-total">{{ money(balance.total, perDollar) }}</strong>
            <small v-if="balance.paid > 0" data-testid="gateway-paid">其中付费额度 {{ money(balance.paid, perDollar) }}</small>
            <NButton class="gw-refresh" size="tiny" quaternary :loading="loading" data-testid="gateway-refresh" @click="load">刷新</NButton>
          </div>
          <details class="gw-credit-details">
          <summary>额度明细<ChevronDown :size="14" aria-hidden="true" /></summary>
          <p class="gw-note" data-testid="gateway-subtitle">{{ balance.product.name }}的所有端共用这份余额</p>
          <ul v-if="balance.buckets.length" class="gw-buckets" data-testid="gateway-buckets">
            <li v-for="bucket in balance.buckets" :key="bucket.id">
              <NTag size="small" :bordered="false" :type="bucket.kind === 'trial' ? 'default' : 'success'">{{ bucketLabel(bucket.kind) }}</NTag>
              <span class="gw-amount">{{ money(bucket.remaining, perDollar) }} <em>/ {{ money(bucket.total, perDollar) }}</em></span>
              <span class="gw-expiry">{{ expiryText(bucket.expiresAt) }}</span>
            </li>
          </ul>
          <p v-else-if="balance.trial.available" class="gw-note" data-testid="gateway-trial-hint">
            首次使用小伴 AI 时会自动发放 {{ money(balance.trial.quota, perDollar) }} 试用额度，{{ balance.trial.days }} 天内有效。
          </p>
          <p v-else class="gw-note" data-testid="gateway-empty">现在没有可用额度，输入兑换码即可充值。</p>
          </details>
        </section>

        <section class="gw-block gw-redemption">
          <form class="gw-redeem" @submit.prevent="redeem">
            <NInput v-model:value="code" placeholder="输入兑换码" aria-label="兑换码" :maxlength="64" :disabled="redeeming" clearable data-testid="gateway-code" />
            <NButton type="primary" attr-type="submit" :loading="redeeming" :disabled="!code.trim()" data-testid="gateway-redeem">兑换</NButton>
          </form>
          <p v-if="redeemError" class="gw-error" role="alert" data-testid="gateway-redeem-error">{{ redeemError }}</p>
          <p v-if="redeemDone" class="gw-done" role="status" data-testid="gateway-redeem-done">{{ redeemDone }}</p>
        </section>

        <section v-if="tiers.length" class="gw-model-section" data-testid="gateway-use">
          <div class="gw-block-head">
            <h4>模型<span class="gw-count" data-testid="gateway-model-count">{{ modelCount }}</span></h4>
          </div>
          <NInput v-model:value="modelQuery" placeholder="搜索模型" aria-label="搜索模型" clearable data-testid="gateway-model-search"><template #prefix><Search :size="15" aria-hidden="true" /></template></NInput>
          <div ref="modelList" class="gw-tiers xb-scroll" role="radiogroup" aria-label="模型" @keydown="moveSelection">
            <button
              v-for="tier in filteredTiers"
              :key="tier.tier"
              type="button"
              role="radio"
              class="gw-tier"
              :class="{ on: picked === tier.tier }"
              :aria-checked="picked === tier.tier"
              :tabindex="tier.tier === tabbableTier ? 0 : -1"
              :disabled="!tier.available || enabling"
              :title="[tier.displayName, tier.description, !tier.available ? '兑换付费额度后可用' : ''].filter(Boolean).join('；')"
              :aria-description="[tier.description, !tier.available ? '兑换付费额度后可用' : ''].filter(Boolean).join('；') || undefined"
              :data-testid="`gateway-tier-${tier.tier}`"
              @click="picked = tier.tier"
            >
              <span class="gw-radio" aria-hidden="true"><Check v-if="picked === tier.tier" :size="12" :stroke-width="3" /></span>
              <b>{{ tier.displayName }}</b>
              <span v-if="tier.tier === activeTier" class="gw-tier-state">使用中</span>
              <span v-else-if="tier.isDefault" class="gw-tier-state">默认</span>
              <span v-if="!tier.available" class="gw-tier-state">需付费额度</span>
            </button>
            <div v-if="!filteredTiers.length" class="gw-empty" role="status" data-testid="gateway-search-empty">没有匹配的模型<button type="button" class="xb-link" @click="modelQuery = ''">清除搜索</button></div>
          </div>
        </section>
        <div v-else-if="!loading && !loadError" class="gw-empty" role="status">暂无可用模型<NButton text @click="load">刷新</NButton></div>
        <footer v-if="tiers.length" class="gw-actions">
          <div class="gw-selection">
            <span v-if="gatewayActive" class="gw-current" data-testid="gateway-active" :title="activeTierName">当前使用 · {{ activeTierName }}</span>
            <span v-else class="gw-current">尚未启用小伴 AI</span>
            <b v-if="pickedTier && !unchanged" :title="pickedTier.displayName">已选 · {{ pickedTier.displayName }}</b>
          </div>
          <NButton type="primary" :disabled="!canEnable" :loading="enabling" data-testid="gateway-use-btn" @click="enable">
            {{ unchanged ? '当前使用中' : '使用此模型' }}
          </NButton>
          <p v-if="enableError" class="gw-error" role="alert" data-testid="gateway-use-error">{{ enableError }}</p>
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
.gw-detached{height:100dvh;overflow:auto;box-sizing:border-box;padding:24px;background:var(--ui-surface-sunken);color:var(--ui-text-primary)}
.gw-modal{display:flex;flex-direction:column;gap:16px;min-height:0}
.gw-detached .gw-modal{height:100%;min-height:480px}
.gw-window-head{display:flex;align-items:center;gap:10px;flex-shrink:0}
.gw-window-head .gw-logo{width:34px;height:34px;border-radius:10px}
.gw-window-head h1{font-size:17px;font-weight:600;margin:0}
.gw-signin { display: grid; gap: 14px; justify-items: start; }
.gw-signin p { margin: 0; line-height: 1.7; color: var(--ui-text-secondary); }
.gw-balance {
  display: grid;
  gap: 10px;
  padding: 16px;
  border-radius: 14px;
  background: linear-gradient(135deg, var(--ui-primary-soft), var(--ui-surface-alt));
  border: 1px solid var(--ui-border);
}
.gw-total { display: flex; align-items: baseline; gap: 12px; flex-wrap: wrap; }
.gw-total > span { color: var(--ui-text-secondary); }
.gw-total strong { font-size: 32px; line-height: 1.1; font-variant-numeric: tabular-nums; }
.gw-total small { color: var(--ui-text-secondary); }
.gw-refresh { margin-left: auto; }
.gw-note { margin: 0; font-size: 12px; color: var(--ui-text-secondary); }
.gw-buckets { list-style: none; margin: 0; padding: 0; display: grid; gap: 8px; }
.gw-buckets li {
  display: grid;
  grid-template-columns: auto 1fr auto;
  align-items: center;
  gap: 12px;
  padding: 9px 12px;
  border-radius: 10px;
  background: var(--ui-surface);
  border: 1px solid var(--ui-border);
}
.gw-amount { font-variant-numeric: tabular-nums; }
.gw-amount em { font-style: normal; color: var(--ui-muted); }
.gw-expiry { font-size: 13px; color: var(--ui-muted); }

.gw-block { display: grid; gap: 10px; justify-items: start; }
.gw-block h4 { margin: 0; font-size: 13px; font-weight: 650; }
.gw-block-head { display: flex; align-items: center; gap: 10px; }
.gw-redeem { display: flex; gap: 10px; width: 100%; }
.gw-redeem .n-input { flex: 1; }
.gw-error { margin: 0; color: var(--ui-danger); font-size: 13px; }
.gw-done { margin: 0; color: var(--ui-success); font-size: 13px; }
.gw-balance,.gw-redemption,.gw-actions{flex-shrink:0}
.gw-credit-details summary{display:flex;align-items:center;justify-content:space-between;gap:6px;cursor:pointer;font-size:12px;color:var(--ui-text-secondary);list-style:none}
.gw-credit-details{max-height:180px;overflow:auto}
.gw-credit-details summary::-webkit-details-marker{display:none}
.gw-credit-details[open] summary svg{transform:rotate(180deg)}
.gw-credit-details .gw-note,.gw-credit-details .gw-buckets{margin-top:10px}
.gw-model-section{display:flex;flex-direction:column;gap:12px;min-height:0;flex:1}
.gw-block-head h4{display:flex;align-items:center;gap:8px;margin:0;font-size:13px;font-weight:600}
.gw-count{font-weight:400;color:var(--ui-text-secondary);font-variant-numeric:tabular-nums}
.gw-tiers{display:flex;flex-direction:column;overflow:auto;min-height:132px;max-height:320px;border:1px solid var(--ui-border);border-radius:12px;background:var(--ui-surface)}
.gw-detached .gw-tiers{flex:1;max-height:none}
.gw-tier{display:flex;align-items:center;gap:10px;flex-shrink:0;min-height:46px;width:100%;padding:10px 12px;border:0;border-bottom:1px solid var(--ui-line);background:transparent;text-align:left;color:var(--ui-text-primary);font:inherit;cursor:pointer}
.gw-tier:last-child{border-bottom:0}
.gw-tier:hover:not(:disabled){background:var(--ui-surface-hover)}
.gw-tier.on,.gw-tier.on:hover{background:var(--ui-soft)}
.gw-tier b{flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:13px;font-weight:500}
.gw-radio{display:grid;place-items:center;flex-shrink:0;width:16px;height:16px;border:1px solid var(--ui-border-strong);border-radius:50%}
.gw-tier.on .gw-radio{border-color:var(--ui-btn);background:var(--ui-btn);color:var(--ui-text-on-btn)}
.gw-tier-state{font-size:11px;color:var(--ui-text-secondary);white-space:nowrap}
.gw-tier:disabled{cursor:not-allowed;color:var(--ui-text-disabled)}
.gw-tier:focus-visible{outline:2px solid var(--ui-btn);outline-offset:-3px}
.gw-empty{display:flex;flex:1;flex-direction:column;align-items:center;justify-content:center;gap:8px;padding:24px;color:var(--ui-text-secondary);font-size:13px}
.gw-actions{display:flex;align-items:center;gap:14px;flex-wrap:wrap;border-top:1px solid var(--ui-border);padding-top:16px}
.gw-selection{display:grid;gap:4px;flex:1;min-width:0}
.gw-selection>*{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.gw-current{font-size:12px;color:var(--ui-text-secondary)}
.gw-selection b{font-size:13px;font-weight:500}
.gw-actions>.gw-error{width:100%}
@media(max-width:420px){.gw-detached{padding:16px}.gw-total{gap:8px}.gw-total small{flex-basis:100%}.gw-buckets li{gap:8px;grid-template-columns:auto 1fr}.gw-expiry{grid-column:2;font-size:12px}}
</style>
