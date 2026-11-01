<script setup lang="ts">
import { computed, defineAsyncComponent, ref, type Component } from 'vue';
import '../configuration/surfaces.css';
import { NModal } from 'naive-ui';
import { Info, Palette, Search, Settings, ShieldCheck, Sparkles, Terminal, Volume2, Wifi, X, Zap } from 'lucide-vue-next';
import type { ControlStore, SettingsTab } from '../control-types.ts';
import GeneralSettings from './settings/GeneralSettings.vue';
import AppearanceSettings from './settings/AppearanceSettings.vue';
import AboutSettings from './settings/AboutSettings.vue';
import NetworkSettings from './settings/NetworkSettings.vue';
import WindowChrome from '../configuration/WindowChrome.vue';
import { dragConfigurationWindow, zoomConfigurationWindow } from '../configuration/windows.ts';
const AiConfigCard = defineAsyncComponent(() => import('./AiConfigCard.vue'));
const AssistantPage = defineAsyncComponent(() => import('../pages/AssistantPage.vue'));
const VoiceSettingsCard = defineAsyncComponent(() => import('./VoiceSettingsCard.vue'));
const AgentSafetyCard = defineAsyncComponent(() => import('./AgentSafetyCard.vue'));
const DesignMcpSettings = defineAsyncComponent(() => import('./settings/DesignMcpSettings.vue'));
const SpeechAgentSettings = defineAsyncComponent(() => import('./settings/SpeechAgentSettings.vue'));

const props = defineProps<{ store: ControlStore; detached?: boolean }>();
const emit = defineEmits<{ openSite: [] }>();

/** Grouped navigation; keyword filtering preserves all existing settings routes. */
const tabs: { id: SettingsTab; label: string; icon: Component; keywords: string }[] = [
  { id: 'general', label: '通用', icon: Settings, keywords: '开机 启动 动画 帧率 暂停 数据 目录' },
  { id: 'appearance', label: '外观', icon: Palette, keywords: '深色 浅色 主题 颜色 主题色 跟随系统 对话框 桌宠 小伴' },
  { id: 'ai', label: 'AI 模型', icon: Sparkles, keywords: '模型 供应商 密钥 api deepseek 网关' },
  { id: 'agent', label: '外部 Agent', icon: Terminal, keywords: 'agent claude codex chatgpt 设计 画板 协作 外部 语音编程 hooks 委派 观察 mcp' },
  { id: 'assistant', label: '超级小伴', icon: Zap, keywords: '快捷键 悬浮球 呼出 截图' },
  { id: 'voice', label: '声音', icon: Volume2, keywords: '语音 朗读 识别 麦克风 音色' },
  { id: 'network', label: '连接与网络', icon: Wifi, keywords: '代理 proxy 网络 连接 socks5 http https VPN 测试 手机 扫码 配对 遥控 互联' },
  { id: 'safety', label: '安全与权限', icon: ShieldCheck, keywords: '权限 沙箱 确认 命令 文件' },
  { id: 'about', label: '关于与更新', icon: Info, keywords: '版本 更新 公告 官网' },
];
const query = ref('');
const shown = computed(() => {
  const q = query.value.trim().toLowerCase();
  return q ? tabs.filter(tab => (tab.label + ' ' + tab.keywords).toLowerCase().includes(q)) : tabs;
});
const groups = computed(() => [
  { label: '个性化', ids: ['general', 'appearance'] },
  { label: '智能与交互', ids: ['ai', 'agent', 'assistant', 'voice'] },
  { label: '应用', ids: ['network', 'safety', 'about'] },
].map(group => ({ ...group, tabs: shown.value.filter(tab => group.ids.includes(tab.id)) })).filter(group => group.tabs.length));
const current = computed(() => tabs.find(tab => tab.id === props.store.settingsTab) ?? tabs[0]);
const show = computed({ get: () => props.store.settingsOpen, set: value => { if (!value) props.store.closeSettings(); } });
const hasUpdate = computed(() => !!props.store.website.update?.available);
function pick(id: SettingsTab) { props.store.settingsTab = id; }
</script>
<template>
  <component :is="detached ? 'div' : NModal" v-model:show="show" :auto-focus="false" transform-origin="center" class="xb-settings-modal">
    <div class="xb-settings" :class="{ 'is-detached': detached }" role="dialog" :aria-modal="detached ? undefined : true" aria-label="设置" data-testid="settings-page">
      <nav class="xb-settings-nav" aria-label="设置分类">
        <WindowChrome v-if="detached" :platform="store.info.platform" />
        <label class="xb-search"><Search :size="13" /><input v-model="query" type="search" placeholder="搜索设置" aria-label="搜索设置" data-testid="settings-search" /></label>
        <div class="xb-settings-sections">
          <section v-for="group in groups" :key="group.label" class="xb-settings-nav-group" :aria-label="group.label">
            <h2>{{ group.label }}</h2>
            <button v-for="tab in group.tabs" :key="tab.id" type="button" class="xb-settings-tab" :class="{ on: tab.id === current.id }" :aria-current="tab.id === current.id ? 'page' : undefined" :data-testid="`settings-tab-${tab.id}`" @click="pick(tab.id)">
              <component :is="tab.icon" :size="17" :stroke-width="1.7" />{{ tab.label }}
              <em v-if="tab.id === 'about' && hasUpdate" class="xb-badge" data-testid="about-tab-new">新</em>
            </button>
          </section>
        </div>
        <p v-if="!shown.length" class="nav-empty">没有匹配的设置</p>
        <div class="xb-settings-account" data-testid="settings-account">
          <span class="avatar"><img v-if="store.account?.user.avatarUrl" :src="store.account.user.avatarUrl" alt="" /><Sparkles v-else :size="20" :stroke-width="1.8" aria-hidden="true" /></span>
          <span class="who"><b>{{ store.account?.user.username ?? '未登录' }}</b><small>{{ store.account ? '已登录' : '登录后同步与使用小伴 AI' }}</small></span>
        </div>
      </nav>
      <section class="xb-settings-main">
        <header @mousedown="detached && dragConfigurationWindow($event)" @dblclick="detached && zoomConfigurationWindow($event)"><h1>{{ current.label }}</h1><button v-if="!detached" type="button" class="xb-icon-btn" aria-label="关闭设置" data-testid="settings-close" @click="store.closeSettings()"><X :size="15" :stroke-width="2.2" /></button></header>
        <div :key="current.id" class="xb-settings-body xb-scroll">
          <div class="xb-settings-content">
          <GeneralSettings v-if="current.id === 'general'" :store="store" />
          <AppearanceSettings v-else-if="current.id === 'appearance'" :store="store" />
          <div v-else-if="current.id === 'ai'" data-testid="settings-ai"><AiConfigCard :store="store" /></div>
          <div v-else-if="current.id === 'agent'" data-testid="settings-agent"><DesignMcpSettings /><SpeechAgentSettings /></div>
          <div v-else-if="current.id === 'assistant'" data-testid="settings-assistant"><AssistantPage :store="store" embedded /></div>
          <div v-else-if="current.id === 'voice'" data-testid="settings-voice"><VoiceSettingsCard v-if="store.speech" :store="store" /></div>
          <NetworkSettings v-else-if="current.id === 'network'" />
          <div v-else-if="current.id === 'safety'" data-testid="settings-safety"><AgentSafetyCard :store="store" /></div>
          <AboutSettings v-else :store="store" @open-site="emit('openSite')" />
          </div>
        </div>
      </section>
    </div>
  </component>
</template>
<style>
.xb-settings{container-type:inline-size;width:940px;height:700px;max-width:calc(100vw - 32px);max-height:calc(100vh - 32px);display:flex;border-radius:var(--ui-radius-dialog);overflow:hidden;background:var(--ui-configuration-canvas);box-shadow:var(--ui-shadow-dialog);color:var(--ui-text-primary);font-family:var(--ui-font-family);font-size:13px}
.xb-settings-nav{width:216px;box-sizing:border-box;flex-shrink:0;display:flex;flex-direction:column;padding:16px 14px 14px;background:var(--ui-configuration-sidebar);overflow:hidden}
html:not([data-theme=dark]) .xb-settings-nav{background:var(--ui-tint)}
.xb-settings-nav .xb-search{margin:0 4px 14px;flex-shrink:0;background:var(--ui-configuration-control);box-shadow:none;border:0;border-radius:8px;color:var(--ui-text-secondary)}
.xb-settings-nav .xb-search input{color:var(--ui-text-primary)}
.xb-settings-nav .xb-search input::placeholder{color:var(--ui-text-tertiary)}
.xb-settings-sections{flex:1;min-height:0;overflow:auto;padding:0 2px 8px;scrollbar-width:thin;scrollbar-color:transparent transparent}
.xb-settings-nav-group+ .xb-settings-nav-group{margin-top:12px}
.xb-settings-nav-group h2{margin:0 10px 8px;font-size:11px;font-weight:500;color:var(--ui-text-tertiary);letter-spacing:.5px}
.xb-settings-tab{display:flex;align-items:center;gap:12px;width:100%;height:32px;padding:0 11px;border:0;border-radius:8px;background:transparent;color:var(--ui-text-secondary);font:inherit;text-align:left;cursor:pointer;margin-bottom:2px}
.xb-settings-tab svg{flex-shrink:0}
.xb-settings-tab:hover{background:var(--ui-surface-hover);color:var(--ui-text-primary)}
.xb-settings-tab.on{background:var(--ui-soft);color:var(--ui-ink);font-weight:600}
.xb-settings-tab .xb-badge{margin-left:auto}
.xb-settings-nav .nav-empty{margin:8px;font-size:12px;color:var(--ui-text-secondary)}
.xb-settings .xb-settings-account{display:flex;align-items:center;gap:10px;width:100%;padding:12px 8px 2px;margin-top:10px;border:0;border-radius:8px;background:transparent;box-shadow:none;text-align:left;cursor:default;color:inherit;flex-shrink:0}
.xb-settings-account .avatar{width:34px;height:34px;flex-shrink:0;border-radius:50%;overflow:hidden;background:var(--ui-soft);color:var(--ui-ink);display:grid;place-items:center}
.xb-settings-account .avatar img{width:100%;height:100%;object-fit:cover}
.xb-settings-account .who{display:flex;flex-direction:column;gap:3px;min-width:0;line-height:1.3}
.xb-settings-account b{font-size:12px;font-weight:600;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.xb-settings-account small{font-size:11px;color:var(--ui-text-secondary);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.xb-settings-main{flex:1;min-width:0;display:flex;flex-direction:column;background:var(--ui-configuration-canvas)}
.xb-settings-main>header{height:70px;box-sizing:border-box;flex-shrink:0;display:flex;align-items:flex-end;padding:0 30px 16px;user-select:none}
.xb-settings-main>header h1{margin:0;font-size:14px;line-height:1.4;letter-spacing:0;font-weight:500;color:var(--ui-text-secondary)}
.xb-settings-main>header .xb-icon-btn{margin-left:auto;width:28px;height:28px}
.xb-settings-body{flex:1;min-height:0;padding:0 30px 28px;scrollbar-gutter:stable}
.xb-settings-content{max-width:780px;margin:0 auto}
.xb-settings.is-detached{width:100%;height:100%;max-width:none;max-height:none;border-radius:0;box-shadow:none}
.xb-settings-modal:has(>.xb-settings.is-detached){width:100%;height:100%}
.xb-settings.is-detached .xb-settings-nav{padding-top:0}
.xb-settings-nav>.configuration-chrome{margin:0 -14px;min-height:52px;flex-shrink:0}
@media(max-width:760px){.xb-settings-nav{width:180px;padding-inline:10px}.xb-settings-main>header{padding-inline:24px}.xb-settings-body{padding-inline:24px}.xb-settings-nav>.configuration-chrome{margin-inline:-10px}}
@media(max-width:600px){.xb-settings-nav{width:150px;padding-inline:8px}.xb-settings-tab{gap:8px;padding-inline:8px;font-size:12px}.xb-settings-main>header{height:64px;padding:0 16px 14px}.xb-settings-main>header h1{font-size:14px}.xb-settings-body{padding-inline:16px}.xb-settings-nav>.configuration-chrome{margin-inline:-8px}}
@container(max-width:760px){.xb-settings-nav{width:180px;padding-inline:10px}.xb-settings-main>header{padding-inline:24px}.xb-settings-body{padding-inline:24px}.xb-settings-nav>.configuration-chrome{margin-inline:-10px}}
@container(max-width:600px){.xb-settings-nav{width:150px;padding-inline:8px}.xb-settings-tab{gap:8px;padding-inline:8px;font-size:12px}.xb-settings-main>header{height:64px;padding:0 16px 14px}.xb-settings-main>header h1{font-size:14px}.xb-settings-body{padding-inline:16px}.xb-settings-nav>.configuration-chrome{margin-inline:-8px}}
</style>
