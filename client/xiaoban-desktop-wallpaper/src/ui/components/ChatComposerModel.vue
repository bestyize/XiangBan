<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { Check, ChevronDown, Code2, Settings } from 'lucide-vue-next';
import { invoke, type AiConfigView, type ProviderConfigView } from '../../bridge.ts';

const props = defineProps<{ label: string; disabled?: boolean }>();
const emit = defineEmits<{ switched: []; settings: [] }>();
const root = ref<HTMLElement | null>(null);
const open = ref(false);
const busy = ref(false);
const error = ref('');
const config = ref<AiConfigView | null>(null);
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
let generation = 0;
function outside(e: PointerEvent) { if (root.value && !root.value.contains(e.target as Node)) close(); }
function escape(e: KeyboardEvent) { if (e.key === 'Escape') close(); }
function close() { open.value = false; ++generation; }
watch(open, value => {
  if (value) { document.addEventListener('pointerdown', outside); document.addEventListener('keydown', escape); }
  else { document.removeEventListener('pointerdown', outside); document.removeEventListener('keydown', escape); }
});
onBeforeUnmount(() => { ++generation; document.removeEventListener('pointerdown', outside); document.removeEventListener('keydown', escape); });
async function toggle() {
  if (open.value) { close(); return; }
  open.value = true; error.value = ''; busy.value = true;
  const id = ++generation;
  try {
    const initial = await invoke<AiConfigView>('get_ai_config');
    if (id !== generation) return;
    config.value = initial;
    try {
      const latest = await invoke<AiConfigView>('refresh_ai_providers', { force: false });
      if (id === generation) config.value = latest;
    } catch { /* Configured local providers remain selectable offline. */ }
  } catch (e) { if (id === generation) error.value = String(e); }
  finally { if (id === generation) busy.value = false; }
}
async function choose(entry: ModelChoice) {
  if (busy.value || current(entry)) { close(); return; }
  busy.value = true; error.value = '';
  const p = entry.provider;
  try {
    await invoke('save_ai_provider', { providerId: p.id, name: p.name, protocol: p.protocol, baseUrl: p.baseUrl,
      model: entry.model, apiKey: null, clearKey: false, activate: true, selectionMode: 'manual' });
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
    <div v-if="open" class="model-panel" role="menu" aria-label="选择编程模型">
      <div class="model-heading">编程模型</div>
      <div v-if="busy && !config" class="model-state">读取已配置模型…</div>
      <div v-for="group in providers" :key="group.provider.id" class="model-group">
        <div class="provider-name">{{ group.provider.name }}</div>
        <button v-for="item in group.models" :key="item.model" type="button" class="model-choice" :title="item.model" :disabled="busy" :class="{ selected: current(item) }" :data-testid="`chat-model-${group.provider.id}-${item.model}`" @click="choose(item)">
          <span>{{ item.name }}</span><Check v-if="current(item)" :size="14" />
        </button>
      </div>
      <p v-if="error" class="model-state error">{{ error }}</p>
      <p v-if="!busy && !providers.length && !error" class="model-state">暂无已配置的可用模型</p>
      <button type="button" class="model-settings" @click="settings"><Settings :size="13" />管理模型</button>
    </div>
  </div>
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
.model-choice>span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.model-choice:disabled{cursor:wait;opacity:.7}
.model-state{color:var(--ui-text-tertiary);font-size:11px;line-height:1.6;padding:4px 9px}
.model-state.error{color:var(--ui-primary)}
.model-settings{display:flex;align-items:center;justify-content:center;gap:6px;width:100%;padding:10px 7px 7px;margin-top:6px;border:0;border-top:1px solid var(--ui-border);background:none;color:var(--ui-primary);cursor:pointer;font:inherit;font-size:12px}
@media(max-width:680px){.composer-model{max-width:96px}.composer-model span{max-width:52px}}
</style>
