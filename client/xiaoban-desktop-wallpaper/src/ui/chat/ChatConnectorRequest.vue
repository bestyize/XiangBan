<script setup lang="ts">
import { ref } from 'vue';
import { NModal } from 'naive-ui';
import { invoke } from '../../bridge.ts';
import ConnectorConnection from '../ai-store/ConnectorConnection.vue';
import MarketConnectorDetail from '../ai-store/MarketConnectorDetail.vue';
import { connectorStatus, type InstalledConnector } from '../ai-store/connector-plugins.ts';
import type { ConnectorSuggestion, QuestionRequest } from './types.ts';

const props = defineProps<{ request: QuestionRequest }>();
const emit = defineEmits<{ answer: [reference: string | null] }>();
const selected = ref<ConnectorSuggestion | null>(null);
const installed = ref<InstalledConnector | null>(null);
const busy = ref(false), error = ref('');
const loading = ref(false);
async function open(candidate: ConnectorSuggestion) {
  if (busy.value) return;
  selected.value = candidate; installed.value = null; error.value = ''; busy.value = true; loading.value = true;
  try {
    const list = await invoke<InstalledConnector[]>('ai_connector_installed');
    if (selected.value?.reference !== candidate.reference) return;
    installed.value = list.find(c => candidate.installed ? c.id === candidate.installed.id : c.plugin === candidate.slug && c.namespace === candidate.namespace) ?? null;
    if (candidate.installed && !installed.value) error.value = '此连接器已被移除，请重新搜索。';
  } catch (e) { error.value = `读取连接状态失败，请重试：${String(e)}`; }
  finally { busy.value = false; loading.value = false; }
}
function connected() {
  if (selected.value) emit('answer', selected.value.reference);
}
function close() { if (!busy.value) selected.value = null; }
</script>

<template>
  <section class="chat-connectors" aria-label="连接服务后继续任务">
    <p>选择服务，完成连接后小伴会继续当前任务。凭据只在配置窗口填写。</p>
    <button v-for="candidate in request.connectors" :key="candidate.reference" class="connector-choice" :disabled="busy" @click="open(candidate)">
      <b>{{ candidate.name }}</b><span>{{ candidate.description }}</span><small>{{ candidate.installed ? connectorStatus[candidate.installed.status] : '安装并连接' }}</small>
    </button>
    <button class="xb-btn" :disabled="busy" @click="emit('answer', null)">暂不连接</button>
    <NModal :show="!!selected" preset="card" :title="selected?.name" class="store-dialog" :mask-closable="!busy" :closable="!busy" style="width:640px;max-width:calc(100vw - 36px)" @update:show="!$event && close()">
      <template v-if="selected">
        <p v-if="error" role="alert">{{ error }} <button class="xb-btn" :disabled="busy" @click="open(selected)">重试</button></p>
        <template v-else-if="installed">
          <ConnectorConnection :key="installed.id" :connector="installed" @updated="installed = $event" @busy="busy = $event" @connected="connected" />
          <button v-if="installed.status === 'connected'" class="store-primary" :disabled="busy" @click="connected">继续当前任务</button>
        </template>
        <MarketConnectorDetail v-else-if="!loading && selected.connector && selected.namespace && selected.slug" :connector="selected.connector" :namespace="selected.namespace" :slug="selected.slug" :version="selected.version" :installed="false" local-error="" @busy="busy = $event" @connected="connected" />
      </template>
    </NModal>
  </section>
</template>

<style scoped>
.chat-connectors p{font-size:13px;line-height:1.6}.connector-choice{display:grid;gap:5px;width:100%;padding:12px;margin:8px 0;text-align:left;border:1px solid var(--ui-border);border-radius:10px;color:var(--ui-text-primary);background:var(--ui-control);cursor:pointer}.connector-choice span,.connector-choice small{font-size:12px;color:var(--ui-text-secondary)}
</style>
