<script setup lang="ts">
import { ref, watch } from 'vue';
import { Trash2 } from 'lucide-vue-next';

const props = withDefaults(defineProps<{ name: string; description: string; busy: boolean; removable?: boolean; label?: string }>(), { removable: true, label: '移除' });
const emit = defineEmits<{ confirm: [] }>();
const confirming = ref(false);
watch(() => props.name, () => { confirming.value = false; });
</script>

<template>
  <section class="store-removal" :class="{ 'is-confirming': confirming && removable }">
    <template v-if="confirming && removable">
      <div class="remove-confirm-box" role="alert">
        <h4>移除「{{ name }}」？</h4>
        <p>{{ description }}</p>
        <slot name="details" />
        <div class="store-removal-actions"><button class="store-secondary" :disabled="busy" @click="confirming = false">取消</button><button class="store-primary store-remove-confirm" :disabled="busy" @click="emit('confirm')">{{ busy ? '移除中…' : '确认移除' }}</button></div>
      </div>
    </template>
    <div v-else class="store-removal-actions">
      <button v-if="removable" class="store-remove-entry" :disabled="busy" @click="confirming = true"><Trash2 :size="13" aria-hidden="true" />{{ label }}</button>
      <div class="store-removal-primary"><slot /></div>
    </div>
  </section>
</template>

<style scoped>
.store-removal{position:sticky;bottom:0;z-index:1;margin-top:20px;padding:14px 0 2px;border-top:1px solid var(--ui-line);background:var(--ui-surface)}
.store-removal.is-confirming{border-top:0;padding-top:4px}
.remove-confirm-box{padding:14px 16px;border-radius:12px;background:color-mix(in srgb,var(--ui-danger) 6%,var(--ui-surface-sunken));box-shadow:inset 0 0 0 1px color-mix(in srgb,var(--ui-danger) 24%,transparent)}
.store-removal-actions,.store-removal-primary{display:flex;align-items:center;justify-content:flex-end;gap:8px;flex-wrap:wrap}
.store-removal-primary{margin-left:auto}
.store-removal h4{font-size:14.5px;margin:0 0 8px;color:var(--ui-text-primary)}
.store-removal p{margin:0 0 14px;color:var(--ui-text-secondary);font-size:12.5px;line-height:1.65}
.store-remove-entry{display:inline-flex;align-items:center;gap:5px;padding:5px 8px;border:0;border-radius:7px;background:transparent;color:var(--ui-text-secondary);font:inherit;font-size:12.5px;cursor:pointer}
.store-remove-entry:hover{color:var(--ui-danger);background:color-mix(in srgb,var(--ui-danger) 8%,transparent)}
.store-removal .store-remove-confirm:not(:disabled){background:var(--ui-danger);color:var(--ui-text-on-btn)}
.store-removal .store-remove-confirm:hover:not(:disabled){background:color-mix(in srgb,var(--ui-danger) 90%,var(--ui-text-primary))}
</style>
