<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { NAlert, NButton, NInput, NModal, NTag } from 'naive-ui';
import { Settings, Sprout } from 'lucide-vue-next';
import { GATEWAY_PROVIDER, invoke, toAccountError, type AiBalance, type AiConfigView, type AiRedeemed, type AiTier } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import { bucketLabel, expiryText, money, pickTier } from '../ai-gateway.ts';

const props = defineProps<{ store: ControlStore; config: AiConfigView | null }>();
const emit = defineEmits<{ config: [view: AiConfigView] }>();

const open = ref(false);
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

const uid = computed(() => props.store.account?.user.uid ?? '');
const perDollar = computed(() => balance.value?.quotaPerUnit ?? 500_000);
const gatewayActive = computed(() => props.config?.activeProvider === GATEWAY_PROVIDER);
const activeTier = computed(() => (gatewayActive.value ? props.config?.gatewayTier ?? '' : ''));
const activeTierName = computed(() => tiers.value.find(t => t.tier === activeTier.value)?.displayName ?? activeTier.value);
const pickedTier = computed(() => tiers.value.find(t => t.tier === picked.value));
const unchanged = computed(() => gatewayActive.value && activeTier.value === picked.value);

const cardDescription = computed(() => {
  if (!props.store.desktop) return '在桌面应用中使用。';
  if (!uid.value) return '登录小伴账号即可使用，新用户有试用额度，不用自己准备密钥。';
  return '选择小伴 AI 提供的模型，按额度使用，可用兑换码充值。';
});
const cardFact = computed(() => {
  if (gatewayActive.value && activeTierName.value) return { label: '当前模式：', value: activeTierName.value };
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
  if (enabling.value || !picked.value) return;
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

/** The mode switch in settings: choosing is enough once 小伴 AI is in use; otherwise it only preselects. */
function pickMode(tier: string) {
  picked.value = tier;
  if (gatewayActive.value && tier !== activeTier.value) void enable();
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
    props.store.notify('暂时没有可用的模式，请稍后刷新再试', 'info');
    openModal();
    return;
  }
  await enable();
}

function openModal() {
  open.value = true;
  redeemError.value = redeemDone.value = enableError.value = '';
  void load();
}

function signIn() {
  open.value = false;
  props.store.openLogin();
}

watch(uid, () => {
  balance.value = null;
  tiers.value = [];
  picked.value = '';
  loadError.value = redeemError.value = redeemDone.value = enableError.value = '';
  void load();
}, { immediate: true });
</script>

<template>
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

  <template v-if="tiers.length && uid">
    <div class="xb-group-title">使用模式</div>
    <div class="xb-group" data-testid="gateway-modes">
      <div class="xb-row gw-modes-row">
        <div class="xb-row-text"><b>小伴 AI 模式</b><span>{{ pickedTier?.description || '选择要使用的模型' }}{{ gatewayActive ? '' : '。选好后点“启用此服务商”生效' }}</span></div>
        <div class="xb-seg" role="radiogroup" aria-label="小伴 AI 模式">
          <button v-for="tier in tiers" :key="tier.tier" type="button" role="radio" :aria-checked="picked === tier.tier" :disabled="!tier.available || enabling" :title="tier.available ? undefined : '兑换付费额度后可用'" :data-testid="`gateway-mode-${tier.tier}`" @click="pickMode(tier.tier)">{{ tier.displayName }}</button>
        </div>
      </div>
    </div>
  </template>

  <NModal v-model:show="open" preset="card" title="小伴 AI" style="max-width: 600px; width: 92vw;" data-testid="gateway-modal">
    <div class="gw-modal">
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
        </section>

        <section class="gw-block">
          <h4>兑换码</h4>
          <form class="gw-redeem" @submit.prevent="redeem">
            <NInput v-model:value="code" placeholder="输入兑换码" :maxlength="64" :disabled="redeeming" clearable data-testid="gateway-code" />
            <NButton type="primary" attr-type="submit" :loading="redeeming" :disabled="!code.trim()" data-testid="gateway-redeem">兑换</NButton>
          </form>
          <p v-if="redeemError" class="gw-error" role="alert" data-testid="gateway-redeem-error">{{ redeemError }}</p>
          <p v-if="redeemDone" class="gw-done" role="status" data-testid="gateway-redeem-done">{{ redeemDone }}</p>
        </section>

        <section v-if="tiers.length" class="gw-block" data-testid="gateway-use">
          <div class="gw-block-head">
            <h4>使用模式</h4>
            <NTag v-if="gatewayActive" type="success" size="small" :bordered="false" data-testid="gateway-active">当前使用中 · {{ activeTierName }}</NTag>
          </div>
          <div class="gw-tiers" role="radiogroup" aria-label="使用模式">
            <button
              v-for="tier in tiers"
              :key="tier.tier"
              type="button"
              role="radio"
              class="gw-tier"
              :class="{ on: picked === tier.tier }"
              :aria-checked="picked === tier.tier"
              :disabled="!tier.available || enabling"
              :data-testid="`gateway-tier-${tier.tier}`"
              @click="picked = tier.tier"
            >
              <b>{{ tier.displayName }}<span v-if="tier.isDefault"> · 默认</span></b>
              <span v-if="tier.description">{{ tier.description }}</span>
              <em v-if="!tier.available">兑换付费额度后可用</em>
            </button>
          </div>
          <NButton type="primary" :disabled="!picked || unchanged" :loading="enabling" data-testid="gateway-use-btn" @click="enable">
            {{ unchanged ? '当前使用中' : gatewayActive ? '切换模式' : '用小伴 AI 聊天' }}
          </NButton>
          <p v-if="enableError" class="gw-error" role="alert" data-testid="gateway-use-error">{{ enableError }}</p>
        </section>
      </template>
    </div>
  </NModal>
</template>

<style scoped>
/* Several root nodes (card, mode group, dialog): everything here is scoped to this file. */
.gateway-card{background:linear-gradient(120deg,var(--ui-surface) 40%,var(--ui-soft))}
.gw-row{padding:16px;gap:14px}
.gw-logo{width:44px;height:44px;flex-shrink:0;border-radius:12px;display:grid;place-items:center;color:#fff;background:linear-gradient(160deg,var(--ui-btn-hi),var(--ui-btn));box-shadow:inset 0 1px 0 rgba(255,255,255,.3)}
.gateway-card .xb-row-text>b{font-size:14px;font-weight:600}
.gw-fact{display:flex;align-items:baseline;gap:4px}
.gw-fact strong{font-weight:600;color:var(--ui-text-primary);font-variant-numeric:tabular-nums}
.gw-status{font-size:12px;color:var(--ui-text-secondary)}
.gw-status.is-warn{color:var(--ui-warning-text)}
.gw-row .xb-btn,.gw-row .xb-btn-primary{flex-shrink:0}
.gw-modes-row{align-items:flex-start;flex-direction:column}
.gw-modes-row .xb-seg{flex-wrap:wrap;max-width:100%;gap:4px}
.gw-modes-row .xb-seg>button{height:auto;min-height:26px;white-space:normal;overflow-wrap:anywhere;text-align:left}
.gw-modal { display: grid; gap: 18px; }
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
.gw-error { margin: 0; color: var(--ui-danger, #d03050); font-size: 13px; }
.gw-done { margin: 0; color: var(--ui-success, #18a058); font-size: 13px; }
.gw-tiers { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 10px; width: 100%; }
.gw-tier {
  display: grid;
  gap: 4px;
  text-align: left;
  padding: 12px 14px;
  border: 1px solid var(--ui-border);
  background: var(--ui-surface-alt);
  border-radius: 12px;
  color: inherit;
  font: inherit;
  cursor: pointer;
}
.gw-tier span, .gw-tier em { font-size: 13px; font-style: normal; color: var(--ui-muted); }
.gw-tier.on { border-color: var(--ui-primary); background: var(--ui-primary-soft); }
.gw-tier:disabled { cursor: not-allowed; opacity: 0.6; }
.gw-tier:focus-visible { outline: 2px solid var(--ui-primary); outline-offset: 2px; }
</style>
