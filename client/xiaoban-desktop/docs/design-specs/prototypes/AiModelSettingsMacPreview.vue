<script setup lang="ts">
/** Design-only macOS direction. No imports from runtime UI and no service calls. */
import { computed, nextTick, onBeforeUnmount, reactive, ref, watch } from 'vue';
import { ArrowUpRight, Check, CircleAlert, CircleCheck, LoaderCircle, RefreshCw, UserRound } from 'lucide-vue-next';
import type { AiTier } from '../../../src/bridge.ts';

const props = defineProps<{ models: AiTier[]; active: string; creditPurchaseUrl?: string; probeOutcome?: 'success' | 'failure' }>();
const emit = defineEmits<{ use: [id: string]; notice: [message: string] }>();
const picked = ref(props.active), code = ref('');
const list = ref<HTMLElement>();
const probes = reactive<Record<string, { state: 'checking' | 'success' | 'failure'; message: string }>>({});
const probeTimers = new Map<string, ReturnType<typeof setTimeout>>();
onBeforeUnmount(() => { for (const timer of probeTimers.values()) clearTimeout(timer); });
function probe(model: AiTier) {
  if (!model.available || probes[model.tier]?.state === 'checking') return;
  const outcome = props.probeOutcome ?? 'success';
  probes[model.tier] = { state: 'checking', message: `正在向 ${model.displayName} 发送 hi` };
  // Mock-only response. The implementation will send hi to this model without changing the active model.
  probeTimers.set(model.tier, setTimeout(() => {
    probeTimers.delete(model.tier);
    probes[model.tier] = outcome === 'success'
      ? { state: 'success', message: '演示检测成功：发送 hi，收到“你好！”，耗时 0.82 秒' }
      : { state: 'failure', message: '演示检测失败：发送 hi 后请求超时，可重新检测' };
  }, 900));
}
const pickedModel = computed(() => props.models.find(model => model.tier === picked.value));
const focusId = computed(() => props.models.find(model => model.tier === picked.value && model.available)?.tier ?? props.models.find(model => model.available)?.tier);
const canUse = computed(() => pickedModel.value?.available && picked.value !== props.active);
watch(() => props.active, value => { picked.value = value; });
watch(() => props.models, models => {
  if (!models.some(model => model.tier === picked.value && model.available)) picked.value = models.find(model => model.tier === props.active && model.available)?.tier ?? models.find(model => model.available)?.tier ?? '';
}, { immediate: true });
function mark(model: AiTier) {
  const name = model.displayName.toLowerCase();
  return name.startsWith('claude') ? 'A' : name.startsWith('deepseek') ? 'D' : name.startsWith('gemini') ? 'G' : 'M';
}
function cancel() { picked.value = props.active; emit('notice', '已取消本次选择'); }
function use() { if (canUse.value) { emit('use', picked.value); emit('notice', `预览已切换到 ${pickedModel.value!.displayName}`); } }
async function move(event: KeyboardEvent) {
  if (!['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
  const available = props.models.filter(model => model.available);
  if (!available.length) return;
  event.preventDefault();
  const i = available.findIndex(model => model.tier === picked.value), back = ['ArrowUp', 'ArrowLeft'].includes(event.key);
  const next = event.key === 'Home' ? 0 : event.key === 'End' ? available.length - 1 : i < 0 ? (back ? available.length - 1 : 0) : (i + (back ? -1 : 1) + available.length) % available.length;
  picked.value = available[next].tier;
  await nextTick(); list.value?.querySelector<HTMLButtonElement>('[aria-checked="true"]')?.focus();
}
</script>

<template>
  <div class="mac-config">
    <section class="credit-pane" aria-label="账户与额度">
      <div class="credit-heading"><span class="section-label">可用额度</span><div class="account-actions"><span class="account-name"><UserRound :size="12" aria-hidden="true" />read</span><button type="button" class="refresh-button" aria-label="刷新额度" title="刷新额度" @click="emit('notice', '演示额度已刷新')"><RefreshCw :size="13" aria-hidden="true" /></button></div></div>
      <div class="credit-overview">
        <strong class="credit-total"><span class="currency">$</span>7.30</strong>
        <div class="credit-packages" aria-label="额度明细">
          <div class="credit-package" title="2026/11/20 到期"><span class="package-kind">加油包</span><span class="package-money">$6.57 <em>/ $25.00</em></span><time>11/20 到期</time></div>
          <div class="credit-package" title="2027/10/18 到期"><span class="package-kind">加油包</span><span class="package-money">$0.729 <em>/ $10.00</em></span><time>2027/10/18</time></div>
        </div>
      </div>
      <form class="redeem-form" @submit.prevent="emit('notice', '设计预览：未兑换真实额度')"><input v-model="code" aria-label="兑换码" placeholder="输入兑换码" maxlength="64" /><button type="submit" class="native-button" :disabled="!code.trim()">兑换</button><button type="button" class="native-button credit-purchase" :disabled="!creditPurchaseUrl" :title="creditPurchaseUrl ? '打开额度获取页面' : '暂未开放'" @click="emit('notice', '设计预览：将打开配置的额度获取链接，不发起真实支付')">获取额度<ArrowUpRight :size="12" aria-hidden="true" /></button></form>
    </section>

    <section class="model-pane" aria-label="模型">
      <div class="pane-heading">
        <div class="heading-title"><h1>模型</h1><span class="model-count">{{ models.length }} 个</span></div>
      </div>
      <div ref="list" class="model-list" role="radiogroup" aria-label="选择模型" @keydown="move">
        <div v-for="model in models" :key="model.tier" class="model-row" :class="{ selected: picked === model.tier }">
        <button class="model-choice" type="button" role="radio" :aria-checked="picked === model.tier" :tabindex="focusId === model.tier ? 0 : -1" :disabled="!model.available" :title="[model.displayName, model.description, !model.available ? '兑换付费额度后可用' : ''].filter(Boolean).join('；')" :aria-description="[model.description, !model.available ? '兑换付费额度后可用' : ''].filter(Boolean).join('；')" @click="picked = model.tier">
          <span class="model-mark" aria-hidden="true">{{ mark(model) }}</span>
          <span class="model-name">{{ model.displayName }}</span>
          <span v-if="model.tier === active" class="model-state">使用中</span>
          <span v-else-if="!model.available" class="model-state">需付费额度</span>
          <span v-else-if="model.isDefault" class="model-state">默认</span>
          <span class="selection-mark" aria-hidden="true"><Check v-if="picked === model.tier" :size="14" :stroke-width="2.6" /></span>
        </button>
        <span v-if="probes[model.tier] && probes[model.tier].state !== 'checking'" class="probe-status" :class="probes[model.tier].state" role="status" :title="probes[model.tier].message" :aria-label="probes[model.tier].message"><CircleCheck v-if="probes[model.tier].state === 'success'" :size="14" aria-hidden="true" /><CircleAlert v-else :size="14" aria-hidden="true" /></span>
        <button type="button" class="native-button probe-button" :disabled="!model.available || probes[model.tier]?.state === 'checking'" :aria-label="`检测 ${model.displayName}`" :title="probes[model.tier]?.message ?? `向 ${model.displayName} 发送 hi，检测是否可用`" :aria-busy="probes[model.tier]?.state === 'checking'" @click="probe(model)"><LoaderCircle v-if="probes[model.tier]?.state === 'checking'" class="probe-spinner" :size="12" aria-hidden="true" />{{ probes[model.tier]?.state === 'checking' ? '检测中' : '检测' }}</button>
        </div>
        <div v-if="!models.length" class="model-empty" role="status">暂无可用模型</div>
      </div>
    </section>

    <footer class="window-actions">
        <div class="selection-summary" :title="pickedModel?.displayName"><span>{{ canUse ? '已选' : '当前模型' }}</span><b>{{ pickedModel?.displayName ?? '请选择模型' }}</b></div>
        <div class="action-buttons"><button type="button" class="native-button" @click="cancel">取消</button><button type="button" class="native-button primary" :disabled="!canUse" @click="use">使用此模型</button></div>

    </footer>
  </div>
</template>

<style scoped>
.mac-config{display:flex;flex-direction:column;height:100%;min-height:0;background:color-mix(in srgb,var(--ui-surface) 94%,transparent);color:var(--ui-text-primary);font-family:var(--ui-font-family);font-size:13px;-webkit-font-smoothing:antialiased;container-type:inline-size}
.mac-config *{box-sizing:border-box}
button,input{font:inherit}button{cursor:pointer}button:disabled{cursor:not-allowed;color:var(--ui-text-disabled)}
button:focus-visible,input:focus-visible{outline:3px solid color-mix(in srgb,var(--ui-btn) 23%,transparent);outline-offset:2px}
.credit-pane{position:relative;flex-shrink:0;padding:26px 32px 25px;background:transparent}
.credit-heading,.account-actions{display:flex;align-items:center;justify-content:space-between;gap:10px}
.section-label{font-size:12px;color:var(--ui-text-secondary)}
.account-name{display:flex;align-items:center;gap:6px;font-size:12px;color:var(--ui-text-secondary)}
.refresh-button{display:grid;place-items:center;width:24px;height:24px;padding:0;border:0;border-radius:7px;background:transparent;color:var(--ui-text-tertiary)}
.refresh-button:hover{background:var(--ui-surface-hover);color:var(--ui-text-primary)}
.credit-overview{display:flex;align-items:center;gap:40px;margin:5px 0 20px}
.credit-total{display:flex;align-items:baseline;flex-shrink:0;min-width:140px;font-size:43px;line-height:1.15;letter-spacing:-1.8px;font-weight:500;font-variant-numeric:tabular-nums}
.credit-total .currency{font-size:26px;margin-right:3px;letter-spacing:-.6px;font-weight:400;color:var(--ui-text-secondary)}
.credit-packages{flex:1;min-width:0;display:grid;gap:3px}
.credit-package{display:flex;align-items:center;gap:14px;min-height:27px;font-size:12px}
.package-kind{color:var(--ui-text-secondary);white-space:nowrap;font-size:11px}
.package-money{flex:1;font-variant-numeric:tabular-nums;white-space:nowrap}
.package-money em{font-style:normal;color:var(--ui-text-tertiary);font-size:11px}
.credit-package time{color:var(--ui-text-tertiary);font-size:11px;white-space:nowrap}
.redeem-form{display:flex;align-items:center;gap:5px;min-height:40px;padding:4px 5px 4px 4px;background:color-mix(in srgb,var(--ui-control) 42%,var(--ui-surface));border-radius:10px}
.redeem-form input{flex:1;min-width:0;height:32px;padding:0 11px;border:0;border-radius:7px;background:color-mix(in srgb,var(--ui-surface) 96%,transparent);color:var(--ui-text-primary);font-size:12px;box-shadow:none}
.redeem-form input::placeholder{color:var(--ui-text-tertiary)}
.native-button{display:inline-flex;align-items:center;justify-content:center;gap:5px;height:32px;padding:0 12px;border:0;border-radius:8px;background:transparent;color:var(--ui-text-secondary);box-shadow:none;font-size:12px;font-weight:400;white-space:nowrap}
.native-button:hover:not(:disabled){background:var(--ui-surface-hover);color:var(--ui-text-primary)}
.native-button:active:not(:disabled){transform:translateY(.5px)}
.native-button:disabled{background:transparent;box-shadow:none;color:var(--ui-text-disabled)}
.credit-purchase{color:var(--ui-ink)}
.model-pane{flex:1;min-height:0;display:flex;flex-direction:column;padding:2px 28px 16px;gap:12px;background:transparent}
.pane-heading{display:flex;align-items:center;justify-content:space-between;gap:16px;flex-shrink:0;padding:0 5px}
.heading-title{display:flex;align-items:baseline;gap:9px}
h1{margin:0;font-size:16px;line-height:1.5;font-weight:600;letter-spacing:-.3px}
.model-count{font-size:11px;color:var(--ui-text-tertiary);font-variant-numeric:tabular-nums}
.model-list{position:relative;flex:1;min-height:0;overflow-y:auto;scrollbar-gutter:stable;padding:1px 3px 1px 0;scrollbar-width:thin;scrollbar-color:var(--ui-scrollbar) transparent}
.model-list::-webkit-scrollbar{width:4px}.model-list::-webkit-scrollbar-thumb{border-radius:5px;background:var(--ui-scrollbar)}
.model-row{position:relative;display:flex;align-items:center;gap:8px;min-height:47px;width:100%;margin-bottom:2px;padding:0 11px;border-radius:10px;background:transparent;color:var(--ui-text-primary)}
.model-choice{display:flex;align-items:center;gap:11px;flex:1;min-width:0;min-height:47px;padding:8px 0;border:0;background:transparent;color:inherit;text-align:left}
.model-row:hover{background:color-mix(in srgb,var(--ui-text-primary) 2.5%,transparent)}
.model-row.selected{background:color-mix(in srgb,var(--ui-text-primary) 4.5%,transparent)}
.model-mark{display:grid;place-items:center;flex-shrink:0;width:28px;height:28px;border-radius:8px;box-shadow:none;background:color-mix(in srgb,var(--ui-control) 65%,var(--ui-surface));font-size:12px;font-weight:500;color:var(--ui-text-secondary)}
.model-name{flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:13px;letter-spacing:-.15px}
.model-state{font-size:11px;color:var(--ui-text-tertiary);white-space:nowrap}
.selection-mark{display:grid;place-items:center;width:16px;height:16px;flex-shrink:0;color:var(--ui-ink)}
.model-choice:disabled .model-mark{opacity:.45}.model-choice:disabled .model-state{color:var(--ui-text-disabled)}
.probe-button{height:26px;padding:0 7px;flex-shrink:0;font-size:11px;color:var(--ui-text-secondary)}
.probe-status{display:flex;flex-shrink:0}.probe-status.success{color:var(--ui-success-text)}.probe-status.failure{color:var(--ui-danger)}
.probe-spinner{animation:probe-spin 1s linear infinite}@keyframes probe-spin{to{transform:rotate(360deg)}}
.window-actions{position:relative;display:flex;align-items:center;justify-content:space-between;gap:18px;flex-shrink:0;min-height:77px;padding:18px 32px 20px;background:transparent}
.window-actions::before{content:'';position:absolute;left:32px;right:32px;top:0;height:1px;background:var(--ui-line)}
.selection-summary{display:flex;align-items:center;gap:8px;min-width:0;font-size:11px;color:var(--ui-text-tertiary)}
.selection-summary>span{flex-shrink:0}.selection-summary b{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-weight:400;font-size:12px;color:var(--ui-text-secondary)}
.action-buttons{display:flex;align-items:center;gap:8px;flex-shrink:0}
.native-button.primary{height:34px;padding:0 17px;border-radius:9px;background:var(--ui-btn);color:var(--ui-text-on-btn);box-shadow:none;font-weight:500}
.native-button.primary:hover:not(:disabled){background:var(--ui-btn-hi);color:var(--ui-text-on-btn)}
.native-button.primary:disabled{background:var(--ui-control);color:var(--ui-text-disabled)}
.model-empty{display:grid;place-items:center;min-height:100%;padding:30px;color:var(--ui-text-secondary);font-size:13px}
@container(max-width:480px){.credit-pane{padding:20px 20px 22px}.credit-overview{gap:9px;flex-wrap:wrap;margin:5px 0 15px}.credit-total{min-width:0;font-size:37px}.credit-total .currency{font-size:23px}.credit-packages{flex-basis:100%;gap:0}.credit-package{min-height:27px;gap:10px}.redeem-form{gap:2px}.redeem-form input{padding-inline:8px}.redeem-form .native-button{padding-inline:7px;font-size:11px}.model-pane{padding:2px 16px 12px;gap:10px}.model-row{gap:6px;padding-inline:8px}.model-choice{gap:9px}.model-name{font-size:12px}.window-actions{padding:14px 20px 18px;flex-wrap:wrap;gap:10px;min-height:74px}.window-actions::before{left:20px;right:20px}.selection-summary{flex-basis:100%}.action-buttons{margin-left:auto}}
@media(prefers-reduced-motion:no-preference){button{transition:background-color .15s,color .15s,box-shadow .15s}}
@media(prefers-reduced-motion:reduce){.probe-spinner{animation:none}}
</style>
