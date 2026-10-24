<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { NAlert, NInput, NInputNumber, NSwitch } from 'naive-ui';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import { toolLabel } from '../chat/format.ts';

interface SavedGrant { session: string; title: string; tool: string; kind?: string | null }
interface AgentSettingsView {
  execution?: ExecutionLimits;
  sandboxAvailable: boolean; sandboxNetwork: boolean; hooks: string; hooksPath: string; grants: SavedGrant[];
  platform?: string; windowsSandbox?: boolean;
}
interface ExecutionLimits {
  maxTurns: number | null; maxToolCalls: number | null;
  maxMinutes: number | null; maxReportedTokens: number | null;
}
const limits = ref<ExecutionLimits>({ maxTurns: null, maxToolCalls: null, maxMinutes: null, maxReportedTokens: null });
const limitsChanged = computed(() => !!view.value && JSON.stringify(limits.value) !== JSON.stringify(view.value.execution ?? { maxTurns: null, maxToolCalls: null, maxMinutes: null, maxReportedTokens: null }));

const props = defineProps<{ store: ControlStore }>();
const view = ref<AgentSettingsView | null>(null);
const hooks = ref('');
const busy = ref('');
const hooksChanged = computed(() => !!view.value && hooks.value !== view.value.hooks);
const example = `{ "hooks": [
  { "event": "before_tool", "tools": ["shell"], "command": "python3 ~/check.py", "timeout_seconds": 10 },
  { "event": "after_tool", "tools": ["edit_file"], "command": "npx prettier --check ." }
] }`;

async function load() {
  if (!props.store.desktop) return;
  try {
    view.value = await invoke<AgentSettingsView>('chat_agent_settings');
    hooks.value = view.value.hooks;
    limits.value = { ...(view.value.execution ?? { maxTurns: null, maxToolCalls: null, maxMinutes: null, maxReportedTokens: null }) };
  } catch (error) { props.store.notify(error, 'error'); }
}

async function act(key: string, action: () => Promise<unknown>, success?: string) {
  busy.value = key;
  try {
    await action();
    if (success) props.store.notify(success, 'success');
    await load();
  } catch (error) { props.store.notify(error, 'error'); } finally { busy.value = ''; }
}

const setNetwork = (allowed: boolean) => act('network', () => invoke('chat_set_sandbox_network', { allowed }));
const saveLimits = () => act('limits', () => invoke('chat_set_execution_limits', { limits: { ...limits.value } }), '执行预算已保存，下次开始或继续时生效');
const setWindowsSandbox = (enabled: boolean) => act('windows', () => invoke('chat_set_windows_sandbox', { enabled }),
  enabled ? '已开启 Windows 沙箱，完全信任下的命令改为低完整性运行' : '已关闭 Windows 沙箱');
const saveHooks = () => act('hooks', async () => {
  const count = await invoke<number>('chat_save_hooks', { text: hooks.value });
  props.store.notify(count ? `已保存 ${count} 条 hook，下次工具调用起生效` : '已清除所有 hook', 'success');
});
const revoke = (grant: SavedGrant) => act(`revoke-${grant.session}-${grant.tool}-${grant.kind ?? ''}`,
  () => invoke('chat_revoke_grant', { sessionId: grant.session, tool: grant.tool, kind: grant.kind ?? null }), '已撤销，下次会重新询问');
const grantText = (grant: SavedGrant) => grant.kind ? `${toolLabel(grant.tool)}：${grant.kind}` : toolLabel(grant.tool);

onMounted(load);
</script>

<template>
  <section class="agent-safety" data-testid="agent-safety">
    <NAlert v-if="!store.desktop" type="info" :show-icon="false">请在桌面应用中设置。</NAlert>
    <template v-else-if="view">
      <div class="xb-group-title">任务执行</div>
      <div class="xb-group">
        <div class="xb-row"><div class="xb-row-text"><b>持续执行与预算</b><span>默认不限制执行轮数和工具次数。留空表示不限制；长期目标的时间与用量按多次继续累计，停止按钮始终可用。</span></div></div>
        <div class="xb-row execution-limits">
          <label>执行轮数<NInputNumber v-model:value="limits.maxTurns" :min="1" :max="1000000" :precision="0" clearable placeholder="不限制" aria-label="执行轮数上限" data-testid="execution-turns" /></label>
          <label>工具调用次数<NInputNumber v-model:value="limits.maxToolCalls" :min="1" :max="10000000" :precision="0" clearable placeholder="不限制" aria-label="工具调用次数上限" /></label>
          <label>执行时间（分钟）<NInputNumber v-model:value="limits.maxMinutes" :min="1" :max="43200" :precision="0" clearable placeholder="不限制" aria-label="执行时间上限" /></label>
          <label>累计模型用量（token）<NInputNumber v-model:value="limits.maxReportedTokens" :min="1" :max="1000000000" :precision="0" clearable placeholder="不限制" aria-label="模型用量上限" /></label>
        </div>
        <div class="xb-row"><div class="xb-row-text"><span>模型用量依据服务商回传，包含子任务；命中提示词缓存的输入按 1/10 计入（与 Claude Code 按费用估算一致），并非精确费用上限。后台定时任务保留独立预算。</span></div><button type="button" class="xb-btn-primary is-sm" :disabled="!limitsChanged || !!busy" data-testid="execution-save" @click="saveLimits">{{ busy === 'limits' ? '保存中…' : '保存执行预算' }}</button></div>
      </div>

      <div class="xb-group-title">完全信任时的沙箱</div>
      <div class="xb-group">
        <div v-if="view.sandboxAvailable" class="xb-row">
          <div class="xb-row-text"><b>允许沙箱中的命令联网</b><span>“完全信任”下不逐一询问的命令在系统沙箱里运行：只能写工作目录、临时目录和软件包缓存，读不到密钥目录。安装依赖、拉取代码需要联网；关闭后只能访问本机服务。</span></div>
          <NSwitch :value="view.sandboxNetwork" :loading="busy === 'network'" :disabled="!!busy" aria-label="允许沙箱中的命令联网" data-testid="sandbox-network" @update:value="setNetwork" />
        </div>
        <template v-else-if="view.platform === 'windows'">
          <div class="xb-row">
            <div class="xb-row-text"><b>Windows 沙箱（实验性）</b><span>完全信任下的命令以“低完整性”运行，只能写工作目录和小伴自己的临时目录，不能改动你的其他文件；不限制读取，也不限制联网。</span></div>
            <NSwitch :value="!!view.windowsSandbox" :loading="busy === 'windows'" :disabled="!!busy" aria-label="Windows 沙箱" data-testid="windows-sandbox" @update:value="setWindowsSandbox" />
          </div>
          <div class="xb-row"><NAlert type="warning" :show-icon="false" style="flex:1" data-testid="windows-sandbox-note">开启时会把工作目录及其中的文件标记为“低完整性”（修改的是这些文件的系统安全标签，不改内容），之后换的工作目录也会在第一次使用前标记。标记较多文件需要一点时间。部分需要写入用户目录的工具（如全局安装、某些编译器缓存）可能在沙箱里失败，遇到时可关闭此选项。</NAlert></div>
        </template>
        <div v-else class="xb-row"><NAlert type="warning" :show-icon="false" style="flex:1" data-testid="sandbox-unavailable">当前系统还不支持沙箱（目前仅 macOS）。“完全信任”下的命令不受系统隔离，只有高风险操作会停下来询问。</NAlert></div>
      </div>

      <div class="xb-group-title">已记住的授权</div>
      <div class="xb-group">
        <div v-if="!view.grants.length" class="xb-row" data-testid="grants-empty"><div class="xb-row-text"><span>还没有记住的授权。在确认卡片上选“这段对话都允许”的操作，会一直记到撤销为止。</span></div></div>
        <div v-for="grant in view.grants" :key="`${grant.session}-${grant.tool}-${grant.kind ?? ''}`" class="xb-row" data-testid="grant-row">
          <div class="xb-row-text"><b>{{ grantText(grant) }}</b><span>{{ grant.title }}</span></div>
          <button type="button" class="xb-btn is-sm" :disabled="!!busy" data-testid="grant-revoke" @click="revoke(grant)">{{ busy === `revoke-${grant.session}-${grant.tool}-${grant.kind ?? ''}` ? '撤销中…' : '撤销' }}</button>
        </div>
      </div>

      <div class="xb-group-title">工具 hooks</div>
      <div class="xb-group">
        <div class="xb-row"><div class="xb-row-text"><b>在工具调用前后运行你自己的命令</b><span>调用前的 hook 退出码不为 0 时拦下这次调用；调用后的 hook 失败时把错误输出交给小伴。</span></div></div>
        <div class="xb-row hooks-row">
          <NInput v-model:value="hooks" type="textarea" :autosize="{ minRows: 4, maxRows: 14 }" :placeholder="example" class="hooks-editor" data-testid="hooks-editor" />
        </div>
        <div class="xb-row">
          <div class="xb-row-text"><span>保存在 <code>{{ view.hooksPath }}</code>。命令以你的身份运行，调用信息以 JSON 从标准输入传入。</span></div>
          <button type="button" class="xb-btn is-sm" :disabled="!hooksChanged || !!busy" @click="hooks = view.hooks">还原</button>
          <button type="button" class="xb-btn-primary is-sm" :disabled="!hooksChanged || !!busy" data-testid="hooks-save" @click="saveHooks">{{ busy === 'hooks' ? '保存中…' : '保存 hooks' }}</button>
        </div>
      </div>
    </template>
  </section>
</template>

<style scoped>
.hooks-row{display:block}
.execution-limits{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:12px}.execution-limits label{display:grid;gap:6px;font-size:12px}
.hooks-editor{width:100%}
.hooks-editor :deep(textarea){font-family:var(--ui-font-mono);font-size:12px}
.xb-row-text code{font-family:var(--ui-font-mono);font-size:11.5px;overflow-wrap:anywhere}
</style>
