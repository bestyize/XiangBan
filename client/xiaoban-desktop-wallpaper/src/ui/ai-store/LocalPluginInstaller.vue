<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue';
import { NModal } from 'naive-ui';
import { invoke, listen } from '../../bridge.ts';

interface PluginReview {
  name: string; version: string; description: string; kind: string; manifestSha256: string;
  warnings: string[]; agents: string[]; skills: string[]; hasMcp: boolean;
  connectorCount?: number; skillCount?: number; agentCount?: number;
}
const props = defineProps<{ desktop: boolean }>();
const emit = defineEmits<{ installed: [] }>();
const show = ref(false), busy = ref(false), confirmed = ref(false), error = ref('');
const path = ref(''), review = ref<PluginReview | null>(null), picker = ref<HTMLInputElement | null>(null);
let reviewedPath = '';
let unlisten: (() => void) | undefined;
let alive = true;
const kindLabel: Record<string, string> = { connector: '连接器', agent: '专家', team: '专家团', skill: '技能包', skills: '技能包', bundle: '技能包' };
async function open() {
  show.value = true; review.value = null; path.value = ''; error.value = ''; confirmed.value = false;
  // Native drag/drop carries absolute paths; the HTML File API intentionally does not.
  if (!unlisten) {
    try {
      const off = await listen<{ paths: string[] }>('tauri://drag-drop', payload => {
        if (show.value && !busy.value && payload.paths.length === 1) { path.value = payload.paths[0]; void inspect(); }
      });
      if (alive) unlisten = off; else off();
    } catch { /* The explicit path input also works when native events are unavailable. */ }
  }
  if (alive && show.value) picker.value?.click();
}
function picked(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0] as (File & { path?: string }) | undefined;
  input.value = '';
  if (!file) return;
  review.value = null; confirmed.value = false;
  if (file.path) { path.value = file.path; void inspect(); }
  else { path.value = ''; error.value = `已选择 ${file.name}。请拖入该 ZIP 文件，或粘贴它的完整路径后审查。`; }
}
async function inspect() {
  if (busy.value) return;
  review.value = null; confirmed.value = false; error.value = '';
  const selectedPath = path.value.trim();
  if (!/\.zip$/i.test(selectedPath) || !/^(?:[a-z]:[\\/]|\\\\|\/)/i.test(selectedPath)) { error.value = '请选择 ZIP 插件包，或填写 ZIP 文件的完整路径。'; return; }
  busy.value = true;
  try {
    const result = await invoke<PluginReview>('ai_plugin_inspect_file', { path: selectedPath });
    if (!result.manifestSha256) throw new Error('审查结果缺少清单摘要，无法确认安装。');
    reviewedPath = selectedPath; review.value = result;
  } catch (e) { error.value = String(e); }
  finally { busy.value = false; }
}
async function install() {
  if (!review.value || !confirmed.value || busy.value) return;
  busy.value = true; error.value = '';
  try {
    // Keep the existing digest lock and also send the documented explicit confirmation.
    await invoke('ai_plugin_install_file', { path: reviewedPath, sha256: review.value.manifestSha256, confirmed: true });
    review.value = null; show.value = false; emit('installed');
  } catch (e) { error.value = String(e); }
  finally { busy.value = false; }
}
onBeforeUnmount(() => { alive = false; unlisten?.(); });
</script>

<template>
  <button class="xb-btn" :disabled="!desktop" @click="open">从本地插件包安装</button>
  <input ref="picker" hidden type="file" accept=".zip" aria-label="选择本地 ZIP 插件包" @change="picked" />
  <NModal :show="show" preset="card" title="从本地插件包安装" class="store-dialog" :mask-closable="!busy" :closable="!busy" style="width:600px;max-width:calc(100vw - 36px)" @update:show="!$event && !busy && (show = false)">
    <form class="local-plugin-form" @submit.prevent="inspect">
      <p>选择或拖入 ZIP 插件包，先查看审查结果再安装。</p>
      <label>插件包完整路径<input v-model="path" aria-label="插件包完整路径" :disabled="busy" placeholder="粘贴 ZIP 文件的完整路径" @input="review = null; confirmed = false" /></label>
      <div><button type="button" class="xb-btn" :disabled="busy" @click="picker?.click()">选择 ZIP 文件</button> <button class="xb-btn" type="submit" :disabled="busy">{{ busy ? '正在处理…' : '审查插件包' }}</button></div>
    </form>
    <p v-if="error" role="alert">{{ error }}</p>
    <template v-if="review">
      <h3>{{ review.name }} · v{{ review.version }}</h3><p>{{ kindLabel[review.kind] || review.kind }} · {{ review.description }}</p>
      <p>技能 {{ review.skillCount ?? review.skills?.length ?? 0 }} · 专家 {{ review.agentCount ?? review.agents?.length ?? 0 }} · 连接器 {{ review.connectorCount ?? (review.hasMcp ? '包含（数量待连接时确认）' : 0) }}</p>
      <ul><li v-for="warning in review.warnings" :key="warning">{{ warning }}</li></ul>
      <p v-if="review.hasMcp || review.kind === 'connector'">安装后需要你确认并配置才会连接。</p>
      <p>安装不会执行包内脚本。</p>
      <label class="confirm"><input v-model="confirmed" type="checkbox" :disabled="busy" />我已查看此插件包的说明和警告，同意安装。</label>
      <div class="dialog-actions"><button class="store-primary" :disabled="busy || !confirmed" @click="install">确认安装</button></div>
    </template>
  </NModal>
</template>

<style scoped>
.local-plugin-form{display:grid;gap:12px}.local-plugin-form label{display:grid;gap:6px;font-size:13px}.local-plugin-form input{padding:9px 12px;border:1px solid var(--ui-border);border-radius:10px;background:var(--ui-control);color:var(--ui-text-primary);font:inherit}.confirm{display:flex;align-items:center;gap:8px;font-size:13px}p,li{font-size:13px;line-height:1.7;overflow-wrap:anywhere}
</style>
