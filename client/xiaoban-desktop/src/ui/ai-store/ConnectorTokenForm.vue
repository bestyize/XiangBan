<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { ArrowUpRight, Eye, EyeOff, LockKeyhole } from 'lucide-vue-next';
import { invoke } from '../../bridge.ts';
import type { TokenSchema } from './connector-plugins.ts';

const props = withDefaults(defineProps<{ schema?: TokenSchema; configured: string[]; busy?: boolean; confirmationRequired?: boolean; submitLabel?: string; cancelable?: boolean }>(), { confirmationRequired: true, submitLabel: '保存并连接', cancelable: false });
const emit = defineEmits<{ connect: [values: Record<string, string>]; cancel: [] }>();
const values = ref<Record<string, string>>({});
const confirmed = ref(false);
const attempted = ref(false);
const docError = ref('');
const documentation = computed(() => {
  try { const url = new URL(props.schema?.docUrl ?? ''); return url.protocol === 'https:' && !url.username && !url.password ? url.toString() : null; } catch { return null; }
});
async function openDocumentation() {
  if (!documentation.value) return;
  try { await invoke('open_download_url', { url: documentation.value }); docError.value = ''; }
  catch { docError.value = '无法打开服务商页面，请稍后重试。'; }
}
const revealed = ref<Record<string, boolean>>({});
// This interface has no language preference; use the Chinese schema labels.
const missing = computed(() => props.schema?.fields.filter(f => f.required && !values.value[f.key]?.trim() && !props.configured.includes(f.key)) ?? []);
watch(() => [props.schema, props.configured], () => { values.value = {}; revealed.value = {}; confirmed.value = false; attempted.value = false; });
watch(values, () => { confirmed.value = false; }, { deep: true });
function submit() {
  attempted.value = true;
  if (props.busy || (props.confirmationRequired && !confirmed.value) || missing.value.length) return;
  const submitted = Object.fromEntries(Object.entries(values.value).filter(([, value]) => value.trim() !== ''));
  emit('connect', submitted);
  // Never retain credentials after handing them to the host, including a failed connection.
  values.value = {};
  revealed.value = {};
  confirmed.value = false;
}
</script>

<template>
  <form class="token-form" @submit.prevent="submit">
    <h4>{{ schema?.title || '连接配置' }}</h4>
    <button v-if="documentation" class="token-doc-link" type="button" @click="openDocumentation">{{ schema?.docLabel || '如何获取连接密钥？' }}<ArrowUpRight :size="13" /></button>
    <p v-if="docError" role="alert">{{ docError }}</p>
    <label v-for="field in schema?.fields ?? []" :key="field.key">
      <span>{{ field.label || field.key }}{{ field.required ? ' *' : '' }} <small v-if="configured.includes(field.key)">已保存</small></span>
      <span class="token-input"><input v-model="values[field.key]" :aria-label="field.label || field.key" :title="field.description" :aria-description="field.description" :type="field.type === 'password' && !revealed[field.key] ? 'password' : 'text'" :placeholder="configured.includes(field.key) ? '已保存，留空保留' : field.placeholder" :disabled="busy" :required="field.required && !configured.includes(field.key)" autocomplete="off" spellcheck="false" :aria-invalid="attempted && missing.includes(field)" /><button v-if="field.type === 'password'" type="button" class="token-reveal" :disabled="busy" :aria-label="`${revealed[field.key] ? '隐藏' : '显示'}${field.label || field.key}`" :aria-pressed="!!revealed[field.key]" @click="revealed[field.key] = !revealed[field.key]"><EyeOff v-if="revealed[field.key]" :size="15" /><Eye v-else :size="15" /></button></span>
    </label>
    <p v-if="attempted && missing.length" role="alert">请填写必填字段：{{ missing.map(f => f.label || f.key).join('、') }}</p>
    <p class="token-safety" title="连接后小伴可在任务中调用此服务的工具，请确认服务来源及访问权限。" aria-description="连接后小伴可在任务中调用此服务的工具，请确认服务来源及访问权限。"><LockKeyhole :size="13" />凭据仅保存在本机</p>
    <label v-if="confirmationRequired" class="confirmation"><input v-model="confirmed" type="checkbox" :disabled="busy" />我已查看配置，同意连接此服务。</label>
    <div class="token-actions"><button v-if="cancelable" class="store-secondary" type="button" :disabled="busy" @click="emit('cancel')">取消</button><button class="store-primary" type="submit" :disabled="busy || (confirmationRequired && !confirmed) || missing.length > 0">{{ busy ? '正在连接…' : submitLabel }}</button></div>
  </form>
</template>

<style scoped>
.token-form{display:grid;gap:16px;margin:0;padding:14px 0 4px}.token-form h4,.token-form p{margin:0}.token-form h4{font-size:18px;font-weight:600;color:var(--ui-text-primary)}.token-form p,small{font-size:12px;color:var(--ui-text-secondary)}
label{display:grid;gap:8px;font-size:13px}label>span:first-child{display:flex;align-items:center;gap:8px}small{color:var(--ui-success-text);font-size:11px}.token-input{display:flex;align-items:center;position:relative}input:not([type=checkbox]){box-sizing:border-box;width:100%;height:38px;padding:8px 40px 8px 12px;border:0;border-radius:8px;background:var(--ui-control);color:var(--ui-text-primary);font:inherit;font-size:13px}input:focus-visible{outline:2px solid var(--ui-btn);outline-offset:2px}.confirmation{display:flex;align-items:center;gap:8px}
.token-reveal{position:absolute;right:6px;display:grid;place-items:center;width:28px;height:28px;margin:0!important;padding:0;border:0;border-radius:6px;background:transparent;color:var(--ui-text-tertiary);cursor:pointer}.token-reveal:hover{background:var(--ui-surface-hover);color:var(--ui-text-primary)}.token-safety{display:flex;align-items:center;gap:6px;margin-top:0;font-size:11px!important;color:var(--ui-text-tertiary)!important}
.token-doc-link{display:inline-flex;align-items:center;gap:4px;justify-self:start;margin:0;padding:0;background:transparent;color:var(--ui-ink);border:0;font:inherit;font-size:12px;cursor:pointer}.token-doc-link:hover{text-decoration:underline}.token-actions{display:flex;align-items:center;justify-content:flex-end;gap:8px;margin-top:12px}.token-actions button{margin:0}
</style>
