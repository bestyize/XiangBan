<script setup lang="ts">
import { computed, defineAsyncComponent, ref, type Component } from 'vue';
import { NModal } from 'naive-ui';
import { Info, Palette, Search, Settings, ShieldCheck, Sparkles, Volume2, Wifi, X, Zap } from 'lucide-vue-next';
import type { ControlStore, SettingsTab } from '../control-types.ts';
import GeneralSettings from './settings/GeneralSettings.vue';
import AppearanceSettings from './settings/AppearanceSettings.vue';
import AboutSettings from './settings/AboutSettings.vue';
import NetworkSettings from './settings/NetworkSettings.vue';
const AiConfigCard = defineAsyncComponent(() => import('./AiConfigCard.vue'));
const AssistantPage = defineAsyncComponent(() => import('../pages/AssistantPage.vue'));
const VoiceSettingsCard = defineAsyncComponent(() => import('./VoiceSettingsCard.vue'));
const AgentSafetyCard = defineAsyncComponent(() => import('./AgentSafetyCard.vue'));

const props = defineProps<{ store: ControlStore; detached?: boolean }>();
const emit = defineEmits<{ openSite: [] }>();

/** Category list. `color: null` uses the theme colour. Keywords feed the search box. */
const tabs: { id: SettingsTab; label: string; color: string | null; icon: Component; keywords: string }[] = [
  { id: 'general', label: '通用', color: '#8E8E93', icon: Settings, keywords: '开机 启动 动画 帧率 暂停 数据 目录' },
  { id: 'appearance', label: '外观', color: null, icon: Palette, keywords: '深色 浅色 主题 颜色 主题色 跟随系统' },
  { id: 'ai', label: 'AI 模型', color: '#7152D8', icon: Sparkles, keywords: '模型 供应商 密钥 api deepseek 网关' },
  { id: 'assistant', label: '超级小伴', color: '#E5484D', icon: Zap, keywords: '快捷键 悬浮球 呼出 截图' },
  { id: 'voice', label: '声音', color: '#E0457B', icon: Volume2, keywords: '语音 朗读 识别 麦克风 音色' },
  { id: 'network', label: '连接与网络', color: '#3386C7', icon: Wifi, keywords: '代理 proxy 网络 连接 socks5 http https VPN 测试 手机 扫码 配对 遥控 互联' },
  { id: 'safety', label: '安全与权限', color: '#2F6FDB', icon: ShieldCheck, keywords: '权限 沙箱 确认 命令 文件' },
  { id: 'about', label: '关于与更新', color: '#55555B', icon: Info, keywords: '版本 更新 公告 官网' },
];
const query = ref('');
const shown = computed(() => {
  const q = query.value.trim().toLowerCase();
  return q ? tabs.filter(tab => (tab.label + ' ' + tab.keywords).toLowerCase().includes(q)) : tabs;
});
const current = computed(() => tabs.find(tab => tab.id === props.store.settingsTab) ?? tabs[0]);
const show = computed({ get: () => props.store.settingsOpen, set: value => { if (!value) props.store.closeSettings(); } });
const hasUpdate = computed(() => !!props.store.website.update?.available);
function pick(id: SettingsTab) { props.store.settingsTab = id; }
</script>
<template>
  <component :is="detached ? 'div' : NModal" v-model:show="show" :auto-focus="false" transform-origin="center" class="xb-settings-modal">
    <div class="xb-settings" :class="{ 'is-detached': detached }" role="dialog" :aria-modal="detached ? undefined : true" aria-label="设置" data-testid="settings-page">
      <nav class="xb-settings-nav" aria-label="设置分类">
        <label class="xb-search"><Search :size="13" /><input v-model="query" type="search" placeholder="搜索设置" aria-label="搜索设置" data-testid="settings-search" /></label>
        <button type="button" class="xb-settings-account" data-testid="settings-account" @click="store.account ? undefined : store.openLogin()">
          <span class="avatar"><img v-if="store.account?.user.avatarUrl" :src="store.account.user.avatarUrl" alt="" /><img v-else src="/xiaoban.svg" alt="" class="mascot" /></span>
          <span class="who"><b>{{ store.account?.user.username ?? '未登录' }}</b><small>{{ store.account ? '小伴账号 · 已登录' : '登录后同步与使用小伴 AI' }}</small></span>
        </button>
        <button v-for="tab in shown" :key="tab.id" type="button" class="xb-settings-tab" :class="{ on: tab.id === current.id }" :aria-current="tab.id === current.id ? 'page' : undefined" :data-testid="`settings-tab-${tab.id}`" @click="pick(tab.id)">
          <i :style="{ background: tab.color ?? 'var(--ui-btn)' }"><component :is="tab.icon" :size="14" :stroke-width="2.2" /></i>{{ tab.label }}
          <em v-if="tab.id === 'about' && hasUpdate" class="xb-badge" data-testid="about-tab-new">新</em>
        </button>
        <p v-if="!shown.length" class="nav-empty">没有匹配的设置</p>
      </nav>
      <section class="xb-settings-main">
        <header><h1>{{ current.label }}</h1><button type="button" class="xb-icon-btn" aria-label="关闭设置" data-testid="settings-close" @click="store.closeSettings()"><X :size="15" :stroke-width="2.2" /></button></header>
        <div class="xb-settings-body xb-scroll">
          <GeneralSettings v-if="current.id === 'general'" :store="store" />
          <AppearanceSettings v-else-if="current.id === 'appearance'" />
          <div v-else-if="current.id === 'ai'" data-testid="settings-ai"><AiConfigCard :store="store" /></div>
          <div v-else-if="current.id === 'assistant'" data-testid="settings-assistant"><AssistantPage :store="store" embedded /></div>
          <div v-else-if="current.id === 'voice'" data-testid="settings-voice"><VoiceSettingsCard v-if="store.speech" :store="store" /></div>
          <NetworkSettings v-else-if="current.id === 'network'" />
          <div v-else-if="current.id === 'safety'" data-testid="settings-safety"><AgentSafetyCard :store="store" /></div>
          <AboutSettings v-else :store="store" @open-site="emit('openSite')" />
        </div>
      </section>
    </div>
  </component>
</template>
<style>
.xb-settings{width:940px;height:660px;max-width:calc(100vw - 32px);max-height:calc(100vh - 32px);display:flex;border-radius:var(--ui-radius-dialog);overflow:hidden;background:var(--ui-surface-sunken);box-shadow:var(--ui-shadow-dialog);color:var(--ui-text-primary);font-family:var(--ui-font-family);font-size:13px}
.xb-settings-nav{width:220px;flex-shrink:0;display:flex;flex-direction:column;gap:1px;padding:14px 10px 12px;background:var(--ui-tint);border-right:1px solid var(--ui-line);overflow-y:auto}
.xb-settings-nav .xb-search{margin-bottom:10px;flex-shrink:0}
.xb-settings-account{display:flex;align-items:center;gap:10px;width:100%;padding:8px;margin-bottom:8px;border:0;border-radius:10px;background:var(--ui-surface-raised);box-shadow:0 0 0 1px var(--ui-line);text-align:left;cursor:pointer;color:inherit;flex-shrink:0}
.xb-settings-account .avatar{width:36px;height:36px;flex-shrink:0;border-radius:50%;overflow:hidden;background:#fff;box-shadow:0 0 0 1px rgba(0,0,0,.08);display:grid;place-items:center}
.xb-settings-account .avatar img{width:100%;height:100%;object-fit:cover}
.xb-settings-account .avatar img.mascot{width:30px;height:30px;object-fit:contain}
.xb-settings-account .who{display:flex;flex-direction:column;min-width:0;line-height:1.3}
.xb-settings-account b{font-size:13px;font-weight:600;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.xb-settings-account small{font-size:11px;color:var(--ui-text-secondary);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.xb-settings-tab{display:flex;align-items:center;gap:10px;width:100%;height:34px;flex-shrink:0;padding:0 8px;border:0;border-radius:8px;background:transparent;color:var(--ui-text-primary);font-size:13.5px;text-align:left;cursor:pointer}
.xb-settings-tab:hover{background:var(--ui-surface-hover)}
.xb-settings-tab.on{background:rgba(0,0,0,.075);font-weight:500}
html[data-theme=dark] .xb-settings-tab.on{background:rgba(255,255,255,.1)}
.xb-settings-tab i{width:24px;height:24px;flex-shrink:0;border-radius:7px;display:grid;place-items:center;color:#fff;box-shadow:inset 0 1px 0 rgba(255,255,255,.25),0 1px 1.5px rgba(0,0,0,.12)}
.xb-settings-tab .xb-badge{margin-left:auto}
.xb-settings-nav .nav-empty{margin:8px;font-size:12px;color:var(--ui-text-tertiary)}
.xb-settings-main{flex:1;min-width:0;display:flex;flex-direction:column}
.xb-settings-main>header{height:48px;flex-shrink:0;display:flex;align-items:center;padding:0 12px 0 28px}
.xb-settings-main>header h1{margin:0;font-size:15px;font-weight:600}
.xb-settings-main>header .xb-icon-btn{margin-left:auto;width:28px;height:28px}
.xb-settings-body{flex:1;min-height:0;padding:8px 28px 28px}
.xb-settings.is-detached{width:100%;height:100dvh;max-width:none;max-height:none;border-radius:0;box-shadow:none}
@media(max-width:680px){.xb-settings.is-detached .xb-settings-nav{width:168px;padding:10px 8px}.xb-settings.is-detached .xb-settings-body{padding:8px 16px 20px}.xb-settings.is-detached .xb-settings-main>header{padding-left:16px}}
</style>
