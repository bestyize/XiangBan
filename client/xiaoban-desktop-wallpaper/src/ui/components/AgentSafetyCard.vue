<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { NAlert, NButton, NInput, NSwitch } from 'naive-ui';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import { toolLabel } from '../chat/format.ts';

interface SavedGrant { session: string; title: string; tool: string; kind?: string | null }
interface AgentSettingsView {
  sandboxAvailable: boolean; sandboxNetwork: boolean; hooks: string; hooksPath: string; grants: SavedGrant[];
  platform?: string; windowsSandbox?: boolean;
}

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
  <section class="surface agent-safety" data-testid="agent-safety">
    <NAlert v-if="!store.desktop" type="info" :show-icon="false">请在桌面应用中设置。</NAlert>
    <template v-else-if="view">
      <div class="section-heading"><div><h2>完全信任时的沙箱</h2><p>“完全信任”下不逐一询问的命令，在系统沙箱里运行：只能写工作目录、临时目录和软件包缓存，读不到密钥目录。</p></div></div>
      <div v-if="view.sandboxAvailable" class="setting-row">
        <div><b>允许沙箱中的命令联网</b><p>安装依赖、拉取代码需要联网。关闭后只能访问本机服务。</p></div>
        <NSwitch :value="view.sandboxNetwork" :loading="busy === 'network'" :disabled="!!busy" data-testid="sandbox-network" @update:value="setNetwork" />
      </div>
      <template v-else-if="view.platform === 'windows'">
        <div class="setting-row">
          <div><b>Windows 沙箱（实验性）</b><p>完全信任下的命令以“低完整性”运行，只能写工作目录和小伴自己的临时目录，不能改动你的其他文件；不限制读取，也不限制联网。</p></div>
          <NSwitch :value="!!view.windowsSandbox" :loading="busy === 'windows'" :disabled="!!busy" data-testid="windows-sandbox" @update:value="setWindowsSandbox" />
        </div>
        <NAlert type="warning" :show-icon="false" data-testid="windows-sandbox-note">开启时会把工作目录及其中的文件标记为“低完整性”（修改的是这些文件的系统安全标签，不改内容），之后换的工作目录也会在第一次使用前标记。标记较多文件需要一点时间。部分需要写入用户目录的工具（如全局安装、某些编译器缓存）可能在沙箱里失败，遇到时可关闭此选项。</NAlert>
      </template>
      <NAlert v-else type="warning" :show-icon="false" data-testid="sandbox-unavailable">当前系统还不支持沙箱（目前仅 macOS）。“完全信任”下的命令不受系统隔离，只有高风险操作会停下来询问。</NAlert>

      <div class="section-heading"><div><h2>已记住的授权</h2><p>在确认卡片上选“这段对话都允许”的操作，会一直记到撤销为止。</p></div></div>
      <p v-if="!view.grants.length" class="muted" data-testid="grants-empty">还没有记住的授权。</p>
      <ul v-else class="grants">
        <li v-for="grant in view.grants" :key="`${grant.session}-${grant.tool}-${grant.kind ?? ''}`" data-testid="grant-row">
          <div><b>{{ grantText(grant) }}</b><small>{{ grant.title }}</small></div>
          <NButton size="small" quaternary :disabled="!!busy" :loading="busy === `revoke-${grant.session}-${grant.tool}-${grant.kind ?? ''}`" data-testid="grant-revoke" @click="revoke(grant)">撤销</NButton>
        </li>
      </ul>

      <div class="section-heading"><div><h2>工具 hooks</h2><p>在小伴调用工具前后运行你自己的命令。调用前的 hook 退出码不为 0 时拦下这次调用；调用后的 hook 失败时把错误输出交给小伴。</p></div></div>
      <NInput v-model:value="hooks" type="textarea" :autosize="{ minRows: 4, maxRows: 14 }" :placeholder="example" class="hooks-editor" data-testid="hooks-editor" />
      <p class="muted hooks-path">保存在 <code>{{ view.hooksPath }}</code>。命令以你的身份运行，调用信息以 JSON 从标准输入传入。</p>
      <div class="settings-actions">
        <NButton type="primary" :disabled="!hooksChanged || !!busy" :loading="busy === 'hooks'" data-testid="hooks-save" @click="saveHooks">保存 hooks</NButton>
        <NButton :disabled="!hooksChanged || !!busy" @click="hooks = view.hooks">还原</NButton>
      </div>
    </template>
  </section>
</template>

<style scoped>
.agent-safety{display:grid;gap:14px}
.grants{list-style:none;margin:0;padding:0;display:grid;gap:8px}
.grants li{display:flex;justify-content:space-between;align-items:center;gap:12px;padding:8px 12px;border-radius:10px;background:var(--ui-surface-alt)}
.grants li div{display:grid;gap:2px;min-width:0}
.grants small{color:var(--ui-text-secondary);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.hooks-editor :deep(textarea){font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;font-size:12px}
.hooks-path{font-size:12px;margin:0}
.hooks-path code{word-break:break-all}
</style>
