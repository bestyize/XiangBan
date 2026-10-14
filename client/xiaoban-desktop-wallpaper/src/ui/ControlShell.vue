<script setup lang="ts">
import { computed, defineAsyncComponent, onMounted, onBeforeUnmount, ref } from 'vue';
import { NAlert, NButton, NSkeleton, NTag, useMessage } from 'naive-ui';
import { MessageCircle, Images, Sparkles, Hand, Bot, Gamepad2, Bell, Settings2, PanelLeftClose, PanelLeftOpen, ArrowUpRight, Globe, User, LogIn, LogOut } from 'lucide-vue-next';
import { invoke } from '../bridge.ts';
import { createControlStore } from './control-store.ts';
import type { ControlPage } from './control-types.ts';
const ChatPage = defineAsyncComponent(() => import('./pages/ChatPage.vue'));
const WallpaperPage = defineAsyncComponent(() => import('./pages/WallpaperPage.vue'));
const EffectsPage = defineAsyncComponent(() => import('./pages/EffectsPage.vue'));
const InteractivePage = defineAsyncComponent(() => import('./pages/InteractivePage.vue'));
const WidgetsPage = defineAsyncComponent(() => import('./pages/WidgetsPage.vue'));
const CompanionPage = defineAsyncComponent(() => import('./pages/CompanionPage.vue'));
const GamesPage = defineAsyncComponent(() => import('./pages/GamesPage.vue'));
const CarePage = defineAsyncComponent(() => import('./pages/CarePage.vue'));
const SettingsPage = defineAsyncComponent(() => import('./pages/SettingsPage.vue'));
const LoginModal = defineAsyncComponent(() => import('./components/LoginModal.vue'));

const message = useMessage();
const store = createControlStore((text, type = 'success') => message[type](String(text), { duration: type === 'error' ? 6500 : 3500 }));
const collapsed = ref(false);
function toggleNavigation() { collapsed.value = !collapsed.value; }
const items = [
  { id: 'chat', label: '聊天', hint: '和小伴说话，让它帮忙', icon: MessageCircle },
  { id: 'wallpapers', label: '壁纸库', hint: '发现喜欢的风景', icon: Images },
  { id: 'widgets', label: '桌面组件', hint: '把小物件留在桌面上', icon: Hand },
  { id: 'effects', label: '桌面特效', hint: '雨雪花瓣与流星', icon: Sparkles },
  { id: 'companion', label: '桌面小伴', hint: '外观与陪伴方式', icon: Bot },
  { id: 'games', label: '游戏空间', hint: '给自己放个小假', icon: Gamepad2 },
  { id: 'care', label: '关怀提醒', hint: '照顾日常的小事', icon: Bell },
] as const;
const activeWallpaper = computed(() => store.settings.wallpaperId === 'companion' ? '系统桌面' : store.library.items.find(item => item.definition.id === store.settings.wallpaperId)?.definition.title ?? '已保存的壁纸');
const unread = computed(() => store.website.announcements.filter(item => store.website.dismissed[item.id] !== item.revision).length);
function navigate(page: ControlPage) { store.navigate(page); document.querySelector('.control-workspace')?.scrollTo(0, 0); }
async function openOfficialSite() {
  if (store.desktop) {
    store.notify('正在通过系统浏览器打开小伴官网...', 'info');
    try {
      await invoke('open_official_site');
    } catch (error) {
      store.notify(error, 'error');
    }
  } else {
    window.location.href = 'https://wallpaper.xiangban.xyz';
  }
}
async function handleLogout() {
  try {
    await store.logoutAccount();
    store.notify('已退出当前账号', 'info');
  } catch (error) {
    store.notify(error, 'error');
  }
}
const manageWidgets = () => navigate('widgets');
const focus = () => store.refreshSettings().catch(error => store.notify(error, 'error'));
onMounted(() => {
  if (typeof window !== 'undefined' && store.desktop) {
    // 拦截任何 window.open，强制通过系统浏览器打开，防止 WKWebView 产生 500x500 空白窗口
    window.open = (url) => {
      if (url) void invoke('open_download_url', { url: String(url) });
      return null;
    };
  }
  void store.initialize().then(async () => { if (store.info.uiSmoke) await (await import('./ui-smoke.ts')).runUiSmoke(store); });
  window.addEventListener('focus', focus);
  window.addEventListener('widgets-manage', manageWidgets);
});
onBeforeUnmount(() => { store.dispose(); window.removeEventListener('focus', focus); window.removeEventListener('widgets-manage', manageWidgets); });
</script>
<template>
  <div class="control-app" :class="{ 'is-collapsed': collapsed, 'is-wallpapers': store.page === 'wallpapers' }">
    <aside class="control-sidebar">
      <div class="control-brand"><div class="control-brand-mark"><img src="/xiaoban.svg" alt="小伴"></div><div class="brand-copy"><strong>小伴桌面</strong><span>你的桌面生活空间</span></div></div>

      <div class="sidebar-account-box">
        <div v-if="store.account" class="account-profile-card" data-testid="sidebar-profile-card">
          <div class="account-avatar">
            <img v-if="store.account.user.avatarUrl" :src="store.account.user.avatarUrl" :alt="store.account.user.username" />
            <span v-else>{{ (store.account.user.username || '伴')[0]?.toUpperCase() }}</span>
          </div>
          <div class="account-copy">
            <strong>{{ store.account.user.username }}</strong>
            <span>相伴账号</span>
          </div>
          <button type="button" class="account-logout-btn" data-testid="sidebar-logout-btn" title="退出账号" @click="handleLogout">
            <LogOut :size="15" />
          </button>
        </div>
        <button v-else type="button" class="account-login-card" data-testid="sidebar-login-btn" title="登录 / 注册小伴账号" @click="store.openLogin()">
          <div class="account-avatar-placeholder"><User :size="16" /></div>
          <div class="account-copy">
            <strong>登录 / 注册</strong>
            <span>同步壁纸与数据</span>
          </div>
          <LogIn :size="15" class="login-arrow" />
        </button>
      </div>

      <div class="sidebar-scroll">
        <div class="sidebar-group-label">我的桌面</div>
        <nav id="control-navigation" class="control-navigation" aria-label="功能分区">
          <button v-for="item in items" :key="item.id" :data-testid="`nav-${item.id}`" :class="{ selected: store.page === item.id }" :aria-current="store.page === item.id ? 'page' : undefined" :title="item.label" @click="navigate(item.id)"><component :is="item.icon" :size="19" /><span><b>{{ item.label }}</b></span><i v-if="item.id === 'chat' && store.chat.attention && store.page !== 'chat'" class="notice-dot nav-attention" data-testid="chat-attention" aria-label="小伴在等你确认"></i></button>
        </nav>
        <div class="sidebar-desktop-state"><span class="state-dot" :class="{ idle: store.settings.paused || !store.settings.companionEnabled }"></span><b>{{ store.settings.companionEnabled ? (store.settings.paused ? '小伴休息中' : '小伴正在陪伴') : '小伴已隐藏' }}</b><small>{{ activeWallpaper }}</small></div>
      </div>
      <div class="sidebar-bottom"><button class="website-nav" data-testid="nav-website" @click="openOfficialSite"><Globe :size="18" /><span>小伴官网</span><ArrowUpRight :size="14" class="external-icon" /></button><button class="settings-nav" data-testid="nav-settings" :class="{ selected: store.page === 'settings' }" @click="navigate('settings')"><Settings2 :size="18" /><span>设置</span><em v-if="store.website.update?.available" class="new-badge" data-testid="nav-settings-new">新版本</em><i v-else-if="unread" class="notice-dot"></i></button><div class="sidebar-meta"><span>v{{ store.info.version }}</span><NButton quaternary size="tiny" :title="collapsed ? '展开导航' : '收起导航'" :aria-label="collapsed ? '展开导航' : '收起导航'" :aria-expanded="!collapsed" aria-controls="control-navigation"  @click="toggleNavigation"><component :is="collapsed ? PanelLeftOpen : PanelLeftClose" :size="16" /></NButton></div></div>
    </aside>
    <div class="control-main">
      <main class="control-workspace" :class="{'wallpaper-workspace':store.page==='wallpapers','chat-workspace':store.page==='chat'}">
        <NAlert v-if="store.initialError" class="workspace-alert" type="error" title="桌面配置暂时无法读取">{{ store.initialError }} <NButton size="small" @click="store.initialize()">重试</NButton></NAlert>
        <div v-if="store.website.update?.available && store.page !== 'settings' && store.page !== 'wallpapers' && store.page !== 'chat'" class="update-strip"><span><span class="notice-dot"></span>小伴有新版本 v{{ store.website.update.version }}</span><button @click="navigate('settings')">查看更新<ArrowUpRight :size="14" /></button></div>
        <div v-if="!store.ready" class="workspace-loading"><NSkeleton height="36px" width="220px" /><NSkeleton height="18px" width="320px" /><NSkeleton height="380px" :sharp="false" /></div>
        <template v-else-if="!store.initialError"><ChatPage v-if="store.page === 'chat'" :store="store" /><WallpaperPage v-else-if="store.page === 'wallpapers'" :store="store" /><WidgetsPage v-else-if="store.page === 'widgets'" :store="store" /><InteractivePage v-else-if="store.page === 'interactive'" :store="store" /><EffectsPage v-else-if="store.page === 'effects'" :store="store" /><CompanionPage v-else-if="store.page === 'companion'" :store="store" /><GamesPage v-else-if="store.page === 'games'" :store="store" /><CarePage v-else-if="store.page === 'care'" :store="store" /><SettingsPage v-else :store="store" /></template>
      </main>
      <footer class="control-statusbar"><span><span class="state-dot small"></span>{{ store.desktop ? (store.info.platform === 'windows' ? 'Windows 桌面已连接' : '桌面已连接') : '正在预览界面，桌面操作需在应用内使用' }}</span><span>{{ store.library.items.filter(item => item.installed).length }} 个本地壁纸</span></footer>
    </div>
    <LoginModal :store="store" />
  </div>
</template>
