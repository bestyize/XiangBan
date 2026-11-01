<script setup lang="ts">
import { ref, watch } from 'vue';
import { Trash2 } from 'lucide-vue-next';

const props = withDefaults(defineProps<{ name: string; description: string; busy: boolean; removable?: boolean; label?: string }>(), { removable: true, label: '移除' });
const emit = defineEmits<{ confirm: [] }>();
const confirming = ref(false);
watch(() => props.name, () => { confirming.value = false; });
</script>

<template>
  <section class="store-removal">
    <template v-if="confirming && removable">
      <h4>移除「{{ name }}」？</h4>
      <p>{{ description }}</p>
      <slot name="details" />
      <div class="store-removal-actions"><button class="store-secondary" :disabled="busy" @click="confirming = false">取消</button><button class="store-primary store-remove-confirm" :disabled="busy" @click="emit('confirm')">{{ busy ? '移除中…' : '确认移除' }}</button></div>
    </template>
    <div v-else class="store-removal-actions">
      <button v-if="removable" class="store-remove-entry" :disabled="busy" @click="confirming = true"><Trash2 :size="13" aria-hidden="true" />{{ label }}</button>
      <div class="store-removal-primary"><slot /></div>
    </div>
  </section>
</template>

<style scoped>
.store-removal{position:sticky;bottom:0;z-index:1;margin-top:20px;padding:14px 0 2px;background:var(--ui-surface)}.store-removal-actions,.store-removal-primary{display:flex;align-items:center;justify-content:flex-end;gap:8px;flex-wrap:wrap}.store-removal-primary{margin-left:auto}.store-removal h4{font-size:16px;margin:0 0 10px;color:var(--ui-text-primary)}.store-removal p{margin:0 0 16px;color:var(--ui-text-secondary);font-size:13px;line-height:1.7}.store-remove-entry{display:inline-flex;align-items:center;gap:5px;padding:6px 0;border:0;background:transparent;color:var(--ui-text-secondary);font:inherit;font-size:12px;cursor:pointer}.store-remove-entry:hover{color:var(--ui-danger)}.store-removal .store-remove-confirm:not(:disabled){background:var(--ui-danger);color:var(--ui-text-on-btn)}.store-removal .store-remove-confirm:hover:not(:disabled){background:color-mix(in srgb,var(--ui-danger) 90%,var(--ui-text-primary))}
</style>
