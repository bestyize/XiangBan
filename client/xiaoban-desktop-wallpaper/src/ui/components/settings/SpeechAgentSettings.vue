<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { NAlert, NSwitch } from 'naive-ui';
import { Bot, Check, Copy, ExternalLink, Play, RotateCcw, StopCircle, Terminal } from 'lucide-vue-next';
import { desktop, invoke, listen } from '../../../bridge.ts';

interface ToolStatus {
  name: string;
  installed: boolean;
  path: string | null;
  version: string | null;
  loggedIn: boolean | null;
  notes: string[];
}

interface DesktopAppStatus {
  name: string;
  running: boolean;
}

interface EnvironmentReport {
  os: string;
  node: ToolStatus;
  git: ToolStatus;
  claude: ToolStatus;
  codex: ToolStatus;
  desktopApps: DesktopAppStatus[];
  readyForSpeechCoding: boolean;
}

interface InstallGuide {
  tool: string;
  officialCommand: string;
  mirrorCommand: string | null;
  loginCommand: string | null;
  documentationUrl: string;
}

interface HooksStatus {
  scope: string;
  settingsPath: string;
  fileExists: boolean;
  isInstalled: boolean;
  events: string[];
  otherHooks: boolean;
}

interface VoiceServiceStatus {
  enabled: boolean;
  running: boolean;
  speechBroadcast: boolean;
  port: number;
  token: string;
  baseUrl: string;
  mcpUrl: string;
  mcpConfigSnippet: Record<string, unknown>;
}

interface DelegatedTask {
  taskId: string;
  agent: string;
  prompt: string;
  workspace: string;
  startedAtMs: number;
  status: 'running' | 'completed' | 'failed' | 'cancelled';
  recentOutput: string;
  pendingApproval: string | null;
  sessionId: string | null;
}

const envReport = ref<EnvironmentReport | null>(null);
const guides = ref<InstallGuide[]>([]);
const hooks = ref<HooksStatus | null>(null);
const voiceService = ref<VoiceServiceStatus | null>(null);
const tasks = ref<DelegatedTask[]>([]);

const loading = ref(true);
const busy = ref(false);
const message = ref('');
const isError = ref(false);
const copiedKey = ref<string | null>(null);

let unlistenAnnounce: (() => void) | null = null;
let taskPollTimer: ReturnType<typeof setInterval> | null = null;
let disposed = false;

async function copyText(key: string, text: string) {
  try {
    await navigator.clipboard.writeText(text);
    copiedKey.value = key;
    setTimeout(() => {
      if (copiedKey.value === key) copiedKey.value = null;
    }, 2000);
  } catch {
    copiedKey.value = null;
  }
}

async function loadData() {
  if (!desktop) {
    loading.value = false;
    return;
  }
  try {
    const [envData, guidesData, hooksData, voiceData, tasksData] = await Promise.all([
      invoke<EnvironmentReport>('speech_agent_env'),
      invoke<InstallGuide[]>('speech_agent_install_guides'),
      invoke<HooksStatus>('speech_agent_hooks_status', { workspace: null }),
      invoke<VoiceServiceStatus>('voice_service_status'),
      invoke<DelegatedTask[]>('speech_agent_task_list'),
    ]);
    envReport.value = envData;
    guides.value = guidesData;
    hooks.value = hooksData;
    voiceService.value = voiceData;
    tasks.value = tasksData;
  } catch (e) {
    message.value = `加载外部 Agent 环境失败：${e}`;
    isError.value = true;
  } finally {
    loading.value = false;
  }
}

async function installHooks() {
  if (!desktop || busy.value) return;
  busy.value = true;
  message.value = '';
  try {
    const res = await invoke<{ status: HooksStatus; backup: string | null }>('speech_agent_hooks_install', { workspace: null });
    hooks.value = res.status;
    isError.value = false;
    message.value = res.backup ? `已成功安装 Claude Code hooks（备份保存在：${res.backup}）` : '已成功安装 Claude Code hooks';
  } catch (e) {
    isError.value = true;
    message.value = `安装 hooks 失败：${e}`;
  } finally {
    busy.value = false;
  }
}

async function uninstallHooks() {
  if (!desktop || busy.value) return;
  busy.value = true;
  message.value = '';
  try {
    const res = await invoke<{ status: HooksStatus; backup: string | null }>('speech_agent_hooks_uninstall', { workspace: null });
    hooks.value = res.status;
    isError.value = false;
    message.value = '已成功卸载 hooks 并恢复原有配置';
  } catch (e) {
    isError.value = true;
    message.value = `卸载 hooks 失败：${e}`;
  } finally {
    busy.value = false;
  }
}

async function toggleVoiceService(enabled: boolean) {
  if (!desktop || busy.value) return;
  busy.value = true;
  message.value = '';
  try {
    voiceService.value = await invoke<VoiceServiceStatus>('voice_service_set_enabled', { enabled, speechBroadcast: null });
    isError.value = false;
    message.value = enabled ? '本机回环语音服务已启动' : '本机语音服务已停止';
  } catch (e) {
    isError.value = true;
    message.value = `切换语音服务失败：${e}`;
  } finally {
    busy.value = false;
  }
}

async function toggleSpeechBroadcast(broadcast: boolean) {
  if (!desktop || busy.value) return;
  busy.value = true;
  message.value = '';
  try {
    const currentEnabled = voiceService.value?.enabled ?? false;
    voiceService.value = await invoke<VoiceServiceStatus>('voice_service_set_enabled', { enabled: currentEnabled, speechBroadcast: broadcast });
    isError.value = false;
  } catch (e) {
    isError.value = true;
    message.value = `更新播报设置失败：${e}`;
  } finally {
    busy.value = false;
  }
}

async function resetVoiceToken() {
  if (!desktop || busy.value) return;
  busy.value = true;
  message.value = '';
  try {
    voiceService.value = await invoke<VoiceServiceStatus>('voice_service_reset_token');
    isError.value = false;
    message.value = '已重置语音服务 Bearer Token';
  } catch (e) {
    isError.value = true;
    message.value = `重置 Token 失败：${e}`;
  } finally {
    busy.value = false;
  }
}

async function cancelTask(taskId: string) {
  if (!desktop) return;
  try {
    await invoke('speech_agent_task_cancel', { taskId });
    tasks.value = await invoke<DelegatedTask[]>('speech_agent_task_list');
  } catch (e) {
    message.value = `取消任务失败：${e}`;
    isError.value = true;
  }
}

onMounted(async () => {
  await loadData();
  if (disposed) return;
  if (desktop) {
    const unlisten = await listen<{ text: string }>('speech-announcement', payload => {
      message.value = `语音播报：${payload.text}`;
      isError.value = false;
    });
    if (disposed) { unlisten(); return; }
    unlistenAnnounce = unlisten;
  }
  taskPollTimer = setInterval(async () => {
    if (desktop && envReport.value?.readyForSpeechCoding) {
      runCatching(async () => {
        tasks.value = await invoke<DelegatedTask[]>('speech_agent_task_list');
      });
    }
  }, 4000);
});

onUnmounted(() => {
  disposed = true;
  unlistenAnnounce?.();
  if (taskPollTimer) clearInterval(taskPollTimer);
});

function runCatching(fn: () => Promise<void>) {
  fn().catch(() => {});
}
</script>

<template>
  <div class="speech-agent-settings" data-testid="settings-speech-agent">
    <!-- 环境检测与安装 -->
    <div class="xb-group-title">外部 Coding Agent 环境</div>
    <div class="xb-group">
      <div v-if="envReport" class="xb-row">
        <div class="xb-row-text">
          <b>Claude Code CLI</b>
          <span>
            {{ envReport.claude.installed ? `${envReport.claude.version ?? '已安装'} · ${envReport.claude.loggedIn ? '已登录' : '未登录'}` : '未安装' }}
          </span>
        </div>
        <div class="agent-actions">
          <span class="status-chip" :class="{ on: envReport.claude.installed }">
            {{ envReport.claude.installed ? '可用' : '缺失' }}
          </span>
        </div>
      </div>

      <div v-if="envReport" class="xb-row">
        <div class="xb-row-text">
          <b>OpenAI Codex CLI</b>
          <span>
            {{ envReport.codex.installed ? `${envReport.codex.version ?? '已安装'} · ${envReport.codex.loggedIn ? '已登录' : '未登录'}` : '未安装' }}
          </span>
        </div>
        <div class="agent-actions">
          <span class="status-chip" :class="{ on: envReport.codex.installed }">
            {{ envReport.codex.installed ? '可用' : '缺失' }}
          </span>
        </div>
      </div>

      <div v-if="envReport" class="xb-row">
        <div class="xb-row-text">
          <b>运行中的桌面版应用</b>
          <span>检测到运行：{{ (envReport.desktopApps ?? []).filter(a => a.running).map(a => a.name).join('、') || '暂无运行的桌面客户端' }}</span>
        </div>
        <div class="agent-actions">
          <span v-for="app in envReport.desktopApps ?? []" :key="app.name" class="status-chip is-pill" :class="{ on: app.running }">
            {{ app.name }}
          </span>
        </div>
      </div>
    </div>

    <!-- 安装命令参考（有缺失时展开） -->
    <div v-if="envReport && (!envReport.claude.installed || !envReport.codex.installed)" class="xb-group-title">
      安装命令推荐（中国大陆镜像加速）
    </div>
    <div v-if="envReport && (!envReport.claude.installed || !envReport.codex.installed)" class="xb-group">
      <div v-for="guide in (guides ?? []).filter(g => (g.tool === 'claude' && !envReport?.claude.installed) || (g.tool === 'codex' && !envReport?.codex.installed))" :key="guide.tool" class="xb-row cmd-row">
        <div class="xb-row-text">
          <b>{{ guide.tool === 'claude' ? '安装 Claude Code' : '安装 OpenAI Codex' }}</b>
          <code>{{ guide.mirrorCommand || guide.officialCommand }}</code>
        </div>
        <button
          type="button"
          class="xb-btn is-sm is-ghost"
          @click="copyText(guide.tool, guide.mirrorCommand || guide.officialCommand)"
        >
          <Check v-if="copiedKey === guide.tool" :size="13" />
          <Copy v-else :size="13" />
          {{ copiedKey === guide.tool ? '已复制' : '复制命令' }}
        </button>
      </div>
    </div>

    <!-- 观察层 Hooks 管理 -->
    <div class="xb-group-title">Claude Code 观察层 (Hooks)</div>
    <div class="xb-group">
      <div class="xb-row">
        <div class="xb-row-text">
          <b>小伴观察端点</b>
          <span>
            {{ hooks?.isInstalled ? `已配置，正在监听 ${hooks.events?.length ?? 0} 项生命周期事件` : '未配置；一键安装后外部终端执行任务完成可提醒小伴' }}
          </span>
        </div>
        <div class="agent-actions">
          <button
            v-if="!hooks?.isInstalled"
            type="button"
            class="xb-btn is-sm"
            data-testid="hooks-install-btn"
            :disabled="!desktop || busy || loading"
            @click="installHooks"
          >
            一键安装 Hooks
          </button>
          <button
            v-else
            type="button"
            class="xb-btn is-sm is-ghost is-danger"
            data-testid="hooks-uninstall-btn"
            :disabled="!desktop || busy || loading"
            @click="uninstallHooks"
          >
            卸载 Hooks
          </button>
        </div>
      </div>
    </div>

    <!-- 本机回环语音服务与 MCP (P4) -->
    <div class="xb-group-title">本机语音服务与 MCP (127.0.0.1)</div>
    <div class="xb-group">
      <div class="xb-row">
        <div class="xb-row-text">
          <b>启用回环语音服务</b>
          <span>把小伴本地 SenseVoice 识别与 Edge 朗读能力开放为本地 OpenAI 接口及 MCP 语音工具</span>
        </div>
        <NSwitch
          :value="voiceService?.enabled ?? false"
          aria-label="启用回环语音服务"
          data-testid="voice-service-toggle"
          :disabled="!desktop || busy || loading"
          @update:value="toggleVoiceService"
        />
      </div>

      <div v-if="voiceService?.enabled" class="xb-row">
        <div class="xb-row-text">
          <b>外部任务语音播报</b>
          <span>外部 Agent 委派任务完成、需要确认或遇到异常时，由小伴自动语音朗读一句摘要</span>
        </div>
        <NSwitch
          :value="voiceService?.speechBroadcast ?? true"
          aria-label="外部任务语音播报"
          :disabled="!desktop || busy || loading"
          @update:value="toggleSpeechBroadcast"
        />
      </div>

      <div v-if="voiceService?.enabled" class="xb-row cmd-row">
        <div class="xb-row-text">
          <b>MCP 配置片段 (给 Claude Code / Desktop)</b>
          <span>将下方配置写入 <code>.claude/settings.json</code> 即可为外部 Agent 添加中文耳朵与嘴巴：</span>
          <pre class="mcp-code">{{ JSON.stringify(voiceService.mcpConfigSnippet ?? {}, null, 2) }}</pre>
        </div>
        <button
          type="button"
          class="xb-btn is-sm is-ghost"
          @click="copyText('mcp-snippet', JSON.stringify(voiceService.mcpConfigSnippet ?? {}, null, 2))"
        >
          <Check v-if="copiedKey === 'mcp-snippet'" :size="13" />
          <Copy v-else :size="13" />
          {{ copiedKey === 'mcp-snippet' ? '已复制' : '复制 JSON' }}
        </button>
      </div>
    </div>

    <!-- 委派任务列表 (P3) -->
    <div class="xb-group-title">外部 Agent 委派任务</div>
    <div class="xb-group">
      <div v-if="!tasks.length" class="xb-row">
        <div class="xb-row-text">
          <span>暂无正在运行或历史委派任务；可在小伴聊天中通过 <code>agent_delegate</code> 发起</span>
        </div>
      </div>
      <div v-for="t in tasks" :key="t.taskId" class="xb-row task-row">
        <div class="xb-row-text">
          <b>
            {{ t.taskId }} ({{ t.agent }})
            <span class="status-chip is-pill" :class="{ on: t.status === 'running' }">{{ t.status }}</span>
          </b>
          <span>{{ t.prompt }}</span>
          <small v-if="t.recentOutput" class="task-output">{{ t.recentOutput }}</small>
        </div>
        <button
          v-if="t.status === 'running'"
          type="button"
          class="xb-btn is-sm is-ghost is-danger"
          @click="cancelTask(t.taskId)"
        >
          取消任务
        </button>
      </div>
    </div>

    <NAlert v-if="message" :type="isError ? 'error' : 'success'" :show-icon="false" style="margin-top:14px">
      {{ message }}
    </NAlert>
  </div>
</template>

<style scoped>
.speech-agent-settings{padding-bottom:18px}
.agent-actions{display:flex;align-items:center;gap:8px}
.status-chip{padding:2px 8px;border-radius:6px;font-size:11.5px;font-weight:500;background:var(--ui-surface-sunken);color:var(--ui-text-tertiary)}
.status-chip.on{background:var(--ui-soft);color:var(--ui-btn);font-weight:600}
.status-chip.is-pill{border-radius:999px;font-size:11px}
.cmd-row{flex-wrap:wrap;align-items:flex-start;gap:8px}
.cmd-row code{font-family:monospace;font-size:12px;background:var(--ui-surface-sunken);padding:4px 8px;border-radius:6px;display:block;margin-top:4px;word-break:break-all}
.mcp-code{font-family:monospace;font-size:11.5px;line-height:1.4;background:var(--ui-surface-sunken);padding:8px 10px;border-radius:8px;max-height:160px;overflow:auto;margin:6px 0 0;color:var(--ui-text-secondary)}
.task-row{align-items:flex-start}
.task-output{display:block;color:var(--ui-text-tertiary);font-family:monospace;font-size:11px;margin-top:4px;max-height:48px;overflow:hidden;text-overflow:ellipsis}
.is-danger{color:var(--ui-danger)!important}
</style>
