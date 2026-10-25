<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type { TokenSchema } from './connector-plugins.ts';

const props = defineProps<{ schema?: TokenSchema; configured: string[]; busy?: boolean }>();
const emit = defineEmits<{ connect: [values: Record<string, string>] }>();
const values = ref<Record<string, string>>({});
const confirmed = ref(false);
const attempted = ref(false);
// This interface has no language preference; use the Chinese schema labels.
const missing = computed(() => props.schema?.fields.filter(f => f.required && !values.value[f.key]?.trim() && !props.configured.includes(f.key)) ?? []);
watch(() => [props.schema, props.configured], () => { values.value = {}; confirmed.value = false; attempted.value = false; });
watch(values, () => { confirmed.value = false; }, { deep: true });
function submit() {
  attempted.value = true;
  if (props.busy || !confirmed.value || missing.value.length) return;
  const submitted = Object.fromEntries(Object.entries(values.value).filter(([, value]) => value.trim() !== ''));
  emit('connect', submitted);
  // Never retain credentials after handing them to the host, including a failed connection.
  values.value = {};
  confirmed.value = false;
}
</script>

<template>
  <form class="token-form" @submit.prevent="submit">
    <h4>{{ schema?.title || '连接配置' }}</h4>
    <p v-if="schema?.description">{{ schema.description }}</p>
    <label v-for="field in schema?.fields ?? []" :key="field.key">
      <span>{{ field.label || field.key }}{{ field.required ? ' *' : '' }} <small v-if="configured.includes(field.key)">已保存</small></span>
      <input v-model="values[field.key]" :aria-label="field.label || field.key" :type="field.type" :placeholder="field.placeholder" :disabled="busy" :required="field.required && !configured.includes(field.key)" autocomplete="off" spellcheck="false" :aria-invalid="attempted && missing.includes(field)" />
      <small v-if="field.description">{{ field.description }}</small>
    </label>
    <p v-if="attempted && missing.length" role="alert">请填写必填字段：{{ missing.map(f => f.label || f.key).join('、') }}</p>
    <p>连接后小伴可在任务中调用此服务的工具。请确认服务来源及访问权限。</p>
    <label class="confirmation"><input v-model="confirmed" type="checkbox" :disabled="busy" />我已查看配置，同意连接此服务。</label>
    <button class="store-primary" type="submit" :disabled="busy || !confirmed || missing.length > 0">{{ busy ? '正在连接…' : '确认连接' }}</button>
  </form>
</template>

<style scoped>
.token-form{display:grid;gap:12px;margin:16px 0}.token-form h4,.token-form p{margin:0}.token-form p,small{font-size:12px;color:var(--ui-text-secondary)}label{display:grid;gap:6px;font-size:13px}input:not([type=checkbox]){box-sizing:border-box;width:100%;padding:8px 11px;border:1px solid var(--ui-border);border-radius:9px;background:var(--ui-control);color:var(--ui-text-primary);font:inherit}input:focus{outline:1px solid var(--ui-btn)}.confirmation{display:flex;align-items:center;gap:8px}
</style>
