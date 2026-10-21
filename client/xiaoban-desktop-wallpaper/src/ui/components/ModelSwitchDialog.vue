<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { NAlert, NButton, NModal, NSelect } from 'naive-ui';
import { GATEWAY_PROVIDER, invoke, toAccountError, type AiConfigView, type AiTier, type ProviderConfigView } from '../../bridge.ts';
const props = defineProps<{ show: boolean; retry?: boolean }>();
const emit = defineEmits<{ 'update:show': [value: boolean]; switched: [] ; settings: [] }>();
const config = ref<AiConfigView | null>(null);
const selected = ref('');
const busy = ref(false);
const error = ref('');
const tiers = ref<AiTier[]>([]);
const gatewayError = ref('');
const choices = computed(() => [
  ...(config.value && config.value.gatewayEnabled !== false ? tiers.value.map(t => ({
    label: `小伴 AI · ${t.displayName}${!t.available ? '（暂不可用）' : ''}`,
    value: JSON.stringify([GATEWAY_PROVIDER, t.tier]), description: t.description, disabled: !t.available,
  })) : []),
  ...(config.value?.providers ?? [])
  .filter(p => p.available !== false && p.baseUrl && (p.hasKey || p.isCustom || p.type === 'free'))
  .sort((a, b) => rank(a) - rank(b))
  .flatMap(p => (p.models?.length ? p.models : [{ id: p.model, name: p.model }]).filter(m => m.id)
    .map(m => ({ label: p.name + ' · ' + m.name + (p.type === 'free' ? ' · 免费' : p.type === 'paid' ? ' · 付费' : ''),
      value: JSON.stringify([p.id, m.id]), description: m.description ?? '', disabled: false }))),
]);
const description = computed(() => choices.value.find(c => c.value === selected.value)?.description);
function rank(p: ProviderConfigView) { return p.type === 'free' ? 0 : p.type === 'paid' ? 2 : 1; }
let generation = 0;
onBeforeUnmount(() => { ++generation; });
watch(() => props.show, async show => {
  const id = ++generation;
  if (!show) return;
  error.value = ''; gatewayError.value = ''; tiers.value = []; selected.value = ''; busy.value = true;
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
    if (modes.status === 'fulfilled') tiers.value = modes.value;
    else gatewayError.value = toAccountError(modes.reason).message;
  } catch (e) { if (id === generation) error.value = String(e instanceof Error ? e.message : e); }
  finally { if (id === generation) busy.value = false; }
}, { immediate: true });
async function switchModel() {
  if (!selected.value || !config.value || busy.value) return;
  const choice = choices.value.find(c => c.value === selected.value && !c.disabled);
  if (!choice) return;
  const [id, model] = JSON.parse(choice.value) as [string, string];
  const p = config.value.providers?.find(p => p.id === id);
  if (id !== GATEWAY_PROVIDER && !p) return;
  busy.value = true; error.value = '';
  try {
    if (id === GATEWAY_PROVIDER) await invoke('use_ai_gateway', { tier: model });
    else if (p) await invoke('save_ai_provider', { providerId: p.id, name: p.name, protocol: p.protocol, baseUrl: p.baseUrl,
      model, apiKey: null, clearKey: false, activate: true, selectionMode: 'manual' });
    emit('update:show', false); emit('switched');
  } catch (e) { error.value = toAccountError(e).message; }
  finally { busy.value = false; }
}
</script>
<template>
  <NModal :show="show" preset="card" title="切换模型" style="width:min(92vw,460px)" @update:show="emit('update:show', $event)">
    <p>选择后会使用目标供应商继续对话，并发送必要的聊天上下文。</p>
    <NSelect v-model:value="selected" :options="choices" filterable :disabled="busy" placeholder="选择供应商和模型" data-testid="model-switch-select" />
    <p v-if="description">{{ description }}</p>
    <p v-if="config && config.gatewayEnabled !== false && (gatewayError || (!busy && !tiers.length))" data-testid="model-switch-gateway-error">小伴 AI：{{ gatewayError || '暂无可用模式，请到模型设置查看。' }}</p>
    <NAlert v-if="error" type="error" :show-icon="false">{{ error }}</NAlert>
    <p v-if="!busy && !choices.length">暂无已配置的可用模型。请登录以获取官方服务，或添加自己的供应商。</p>
    <template #footer>
      <NButton @click="emit('settings')">模型设置</NButton>
      <NButton type="primary" :disabled="!selected || busy" :loading="busy" @click="switchModel">{{ retry ? '切换并重试' : '确认切换' }}</NButton>
    </template>
  </NModal>
</template>
