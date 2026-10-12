<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { NAlert, NButton, NInput, NSelect, NSwitch } from 'naive-ui';
import { Bot, FolderOpen, Shield, ShieldCheck, Trash2, Zap } from 'lucide-vue-next';
import { invoke, type AiConfigView, type AiProtocol } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import { aiProtocols, endpointPreview, insecureRemote, normalizeBaseUrl, protocolOption, validateAiForm, validateBaseUrl, validateKey } from '../ai-config.ts';

const props = defineProps<{ store: ControlStore }>();
const blank: AiConfigView = { protocol: 'chat', baseUrl: '', model: '', hasKey: false, keyHint: '', workspace: '', defaultWorkspace: '', permissionMode: 'ask', hideToolCalls: false, openComputerUse: false };
// The last thing the host confirmed. The key is never part of it, only whether one exists.
const saved = ref<AiConfigView>({ ...blank });
const protocol = ref<AiProtocol>('chat');
const baseUrl = ref('');
const model = ref('');
const apiKey = ref('');
// Empty is the built-in folder.
const workspace = ref('');
const permissionMode = ref<'ask' | 'auto'>('ask');
const hideToolCalls = ref(false);
const openComputerUse = ref(false);
const busy = ref('');
const loadError = ref('');
const confirmingClear = ref(false);

const option = computed(() => protocolOption(protocol.value));
const preview = computed(() => validateBaseUrl(baseUrl.value) ? '' : endpointPreview(protocol.value, baseUrl.value));
const workspaceChanged = computed(() => workspace.value !== saved.value.workspace);
const shownWorkspace = computed(() => workspace.value || saved.value.defaultWorkspace || '桌面应用中可查看');
const dirty = computed(() => openComputerUse.value !== saved.value.openComputerUse || hideToolCalls.value !== saved.value.hideToolCalls || permissionMode.value !== saved.value.permissionMode || workspaceChanged.value || protocol.value !== saved.value.protocol || normalizeBaseUrl(baseUrl.value) !== saved.value.baseUrl || model.value.trim() !== saved.value.model || apiKey.value.trim() !== '');
const formError = computed(() => validateAiForm({ baseUrl: baseUrl.value, model: model.value }) || validateKey(apiKey.value));
const canSave = computed(() => props.store.desktop && dirty.value && !formError.value && !busy.value);
const keyPlaceholder = computed(() => saved.value.hasKey ? `已保存 ${saved.value.keyHint}，留空表示不修改` : '没有密钥（本地模型服务可以不填）');
const status = computed(() => saved.value.baseUrl ? `已配置：${saved.value.model}${saved.value.hasKey ? '' : ' · 未设密钥'}` : '尚未配置');

function fill(view: AiConfigView) {
  saved.value = view;
  protocol.value = view.protocol;
  baseUrl.value = view.baseUrl;
  model.value = view.model;
  workspace.value = view.workspace;
  permissionMode.value = view.permissionMode || 'ask';
  hideToolCalls.value = view.hideToolCalls ?? false;
  openComputerUse.value = view.openComputerUse ?? false;
  apiKey.value = '';
}

onMounted(async () => {
  if (!props.store.desktop) return;
  try { fill(await invoke<AiConfigView>('get_ai_config')); } catch (error) { loadError.value = String(error instanceof Error ? error.message : error); }
});

async function send(key: string, args: Record<string, unknown>, after: (view: AiConfigView) => void, success: string) {
  busy.value = key;
  try {
    after(await invoke<AiConfigView>('save_ai_config', args));
    loadError.value = '';
    props.store.notify(success, 'success');
  } catch (error) { props.store.notify(error, 'error'); } finally { busy.value = ''; }
}

const save = () => send('save', {
  protocol: protocol.value, baseUrl: baseUrl.value.trim(), model: model.value.trim(),
  apiKey: apiKey.value.trim() || null, clearKey: false,
  workspace: workspaceChanged.value ? workspace.value : null,
  permissionMode: permissionMode.value,
  hideToolCalls: hideToolCalls.value,
  openComputerUse: openComputerUse.value,
}, fill, '模型设置已保存');

async function chooseFolder() {
  busy.value = 'folder';
  try { const chosen = await invoke<string | null>('pick_workspace'); if (chosen) workspace.value = chosen; } catch (error) { props.store.notify(error, 'error'); } finally { busy.value = ''; }
}

// Deleting works on what is saved, so edits that are still in the form are neither saved nor lost.
const clearKey = () => send('clear', {
  protocol: saved.value.protocol, baseUrl: saved.value.baseUrl, model: saved.value.model, apiKey: null, clearKey: true, workspace: null, permissionMode: saved.value.permissionMode, hideToolCalls: saved.value.hideToolCalls, openComputerUse: saved.value.openComputerUse,
}, view => { saved.value = view; apiKey.value = ''; confirmingClear.value = false; }, '已删除保存的密钥');
</script>

<template>
  <section class="surface ai-config" data-testid="ai-config-card">
    <div class="section-heading"><div><h2>AI 模型</h2><p data-testid="ai-status">{{ status }}</p></div><Bot :size="20" /></div>
    <NAlert v-if="!store.desktop" type="info" :show-icon="false">请在桌面应用中配置模型服务。</NAlert>
    <NAlert v-if="loadError" type="warning" :show-icon="false" data-testid="ai-load-error">{{ loadError }}。重新填写并保存即可覆盖。</NAlert>
    <div class="ai-form">
      <div class="field">
        <label for="ai-protocol">接口协议</label>
        <NSelect v-model:value="protocol" input-id="ai-protocol" :options="aiProtocols.map(o => ({ label: o.label, value: o.value }))" :disabled="!store.desktop || !!busy" data-testid="ai-protocol" />
        <small>{{ option.hint }}</small>
      </div>
      <div class="field">
        <label for="ai-base-url">API 地址</label>
        <NInput v-model:value="baseUrl" :placeholder="option.placeholder" :disabled="!store.desktop || !!busy"  data-testid="ai-base-url" :input-props="{ id: 'ai-base-url', autocomplete: 'off', spellcheck: false }" />
        <small v-if="preview" data-testid="ai-endpoint-preview">实际请求：<code>{{ preview }}</code></small>
        <NAlert v-if="insecureRemote(baseUrl)" type="warning" :show-icon="false" data-testid="ai-http-warning">http 地址不加密，密钥和聊天内容会明文传输，只建议用于本机或可信内网。</NAlert>
      </div>
      <div class="field">
        <label for="ai-api-key">API 密钥</label>
        <NInput v-model:value="apiKey" type="password" show-password-on="click" :placeholder="keyPlaceholder" :disabled="!store.desktop || !!busy"  data-testid="ai-api-key" :input-props="{ id: 'ai-api-key', autocomplete: 'off', spellcheck: false }" />
        <small v-if="saved.hasKey" class="key-actions">
          <template v-if="!confirmingClear"><button type="button" class="link-button" data-testid="ai-clear-key" :disabled="!store.desktop || !!busy" @click="confirmingClear = true"><Trash2 :size="13" />删除已保存的密钥</button></template>
          <template v-else>删除后需要重新填写密钥。<button type="button" class="link-button danger" data-testid="ai-clear-confirm" :disabled="!!busy" @click="clearKey">确认删除</button><button type="button" class="link-button" data-testid="ai-clear-cancel" :disabled="!!busy" @click="confirmingClear = false">取消</button></template>
        </small>
      </div>
      <div class="field">
        <label>权限审批模式</label>
        <div class="permission-options" role="group" aria-label="权限审批模式" data-testid="ai-permission-mode">
          <button type="button" class="perm-choice" :class="{ on: permissionMode === 'ask' }" data-testid="ai-perm-ask" :disabled="!store.desktop || !!busy" @click="permissionMode = 'ask'">
            <Shield :size="16" />
            <div><b>默认权限</b><p>读操作自动，写文件、运行命令与系统调整前需确认</p></div>
          </button>
          <button type="button" class="perm-choice" :class="{ on: permissionMode === 'auto' }" data-testid="ai-perm-auto" :disabled="!store.desktop || !!busy" @click="permissionMode = 'auto'">
            <Zap :size="16" />
            <div><b>完全信任</b><p>自动执行所有操作，无需逐一确认</p></div>
          </button>
        </div>
      </div>
      <div class="field">
        <div class="setting-row">
          <div><b>隐藏工具调用信息</b><p>在会话中不展示工具调用的中间卡片与参数细节，界面更干净简洁（不影响工具正常执行）。</p></div>
          <NSwitch v-model:value="hideToolCalls" :disabled="!store.desktop || !!busy" data-testid="ai-hide-tool-calls" />
        </div>
      </div>
      <div class="field">
        <div class="setting-row">
          <div><b>启用电脑屏幕与应用控制（open-computer-use）</b><p>允许小伴通过系统无障碍操控其他电脑应用（点击按钮、输入文字、按快捷键等）。macOS 首次使用需在系统设置中授予辅助功能权限。</p></div>
          <NSwitch v-model:value="openComputerUse" :disabled="!store.desktop || !!busy" data-testid="ai-open-computer-use" />
        </div>
      </div>
      <div class="field">
        <label for="ai-model">模型</label>
        <NInput v-model:value="model" placeholder="按服务商给出的模型名填写" :disabled="!store.desktop || !!busy"  data-testid="ai-model" :input-props="{ id: 'ai-model', autocomplete: 'off', spellcheck: false }" />
      </div>
    </div>
    <div class="field workspace-field">
      <label>工作目录</label>
      <div class="workspace-row">
        <code data-testid="ai-workspace">{{ shownWorkspace }}</code>
        <NButton :disabled="!store.desktop || !!busy" data-testid="ai-workspace-pick" @click="chooseFolder"><template #icon><FolderOpen :size="15" /></template>选择文件夹</NButton>
        <NButton v-if="workspace" quaternary :disabled="!!busy" data-testid="ai-workspace-reset" @click="workspace = ''">恢复默认</NButton>
      </div>
      <small>小伴的文件工具只在这个文件夹里读写，命令也从这里开始运行。改文件、运行命令前都会先问你。</small>
      <small class="caution" data-testid="ai-workspace-note"><b>命令不受这个文件夹限制：</b>它以你的账户权限运行，选择文件夹只决定从哪里开始。请只批准你看懂的命令。</small>
    </div>
    <NAlert v-if="dirty && formError" type="error" :show-icon="false" class="form-error" data-testid="ai-form-error" role="alert">{{ formError }}</NAlert>
    <div class="ai-actions"><NButton type="primary" :disabled="!canSave" :loading="busy === 'save'" data-testid="ai-save" @click="save">保存</NButton></div>
    <p class="muted ai-note"><ShieldCheck :size="15" />密钥未加密，只保存在本机用户数据目录，不会上传到小伴账号或网站。使用 AI 对话时，聊天内容会发送到你填写的地址。</p>
  </section>
</template>

<style scoped>
.ai-config>.n-alert{margin-top:16px}
.ai-form{display:grid;gap:18px;margin-top:20px}
.ai-actions{display:flex;gap:10px;align-items:center;margin-top:22px}
.field{display:grid;gap:6px}
.field>label{font-size:13px;font-weight:650}
.field small{color:var(--ui-muted);font-size:12px;line-height:1.6;display:flex;gap:6px;align-items:center;flex-wrap:wrap}
.field code{font-size:12px;overflow-wrap:anywhere}
.setting-row{display:flex;align-items:center;justify-content:space-between;gap:16px;padding:4px 0}
.setting-row b{font-size:13px;display:block;margin-bottom:3px}
.setting-row p{margin:0;font-size:12px;color:var(--ui-text-secondary);line-height:1.5}
.permission-options{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:10px}
.perm-choice{border:1px solid var(--ui-border);background:var(--ui-surface-alt);border-radius:12px;padding:12px 14px;cursor:pointer;display:flex;gap:10px;align-items:flex-start;text-align:left;color:var(--ui-text-primary)}
.perm-choice:hover:not(:disabled){border-color:var(--ui-border);background:var(--ui-surface)}
.perm-choice.on{border-color:var(--ui-primary);background:var(--ui-primary-soft)}
.perm-choice b{font-size:13px;display:block;margin-bottom:3px}
.perm-choice p{margin:0;font-size:12px;color:var(--ui-text-secondary);line-height:1.5}
.perm-choice svg{color:var(--ui-primary);flex:none;margin-top:2px}
.perm-choice:disabled{opacity:.6;cursor:not-allowed}
@media(max-width:600px){.permission-options{grid-template-columns:1fr}}
.workspace-field{margin-top:18px}
.workspace-row{display:flex;gap:10px;align-items:center;flex-wrap:wrap}
.workspace-row code{flex:1;min-width:200px;overflow-wrap:anywhere;font-size:12px;background:var(--ui-surface-alt);border:1px solid var(--ui-line);border-radius:8px;padding:10px 12px}
.field small.caution{color:var(--ui-text-secondary)}
.link-button{border:0;background:none;padding:0;color:var(--ui-primary);cursor:pointer;font:inherit;display:inline-flex;gap:4px;align-items:center}
.link-button:disabled{opacity:.5;cursor:not-allowed}
.link-button.danger{color:var(--ui-primary-pressed)}
.form-error{margin-top:14px}
.ai-note{display:flex;gap:7px;align-items:flex-start;font-size:12px;line-height:1.7;margin:16px 0 0}
.ai-note svg{flex:none;margin-top:3px}
</style>
