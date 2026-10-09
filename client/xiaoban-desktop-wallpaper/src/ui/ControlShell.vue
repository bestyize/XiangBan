<script setup lang="ts">
import { computed, defineAsyncComponent, onMounted, onBeforeUnmount, ref } from 'vue';
import { NAlert, NButton, NSkeleton, NTag, useMessage } from 'naive-ui';
import { Images, Bot, Gamepad2, Bell, Settings2, Minus, Power, PanelLeftClose, ChevronRight, ArrowUpRight } from 'lucide-vue-next';
import { invoke } from '../bridge.ts';
import { createControlStore } from './control-store.ts';
import type { ControlPage } from './control-types.ts';
const WallpaperPage = defineAsyncComponent(() => import('./pages/WallpaperPage.vue'));
const CompanionPage = defineAsyncComponent(() => import('./pages/CompanionPage.vue'));
const GamesPage = defineAsyncComponent(() => import('./pages/GamesPage.vue'));
const CarePage = defineAsyncComponent(() => import('./pages/CarePage.vue'));
const SettingsPage = defineAsyncComponent(() => import('./pages/SettingsPage.vue'));

const message = useMessage();
const store = createControlStore((text, type = 'success') => message[type](String(text), { duration: type === 'error' ? 6500 : 3500 }));
const collapsed = ref(false);
const items = [
  { id: 'wallpapers', label: '壁纸库', hint: '发现喜欢的风景', icon: Images },
  { id: 'companion', label: '桌面小伴', hint: '外观与陪伴方式', icon: Bot },
  { id: 'games', label: '游戏空间', hint: '给自己放个小假', icon: Gamepad2 },
  { id: 'care', label: '关怀提醒', hint: '照顾日常的小事', icon: Bell },
] as const;
const current = computed(() => store.page === 'settings' ? '设置' : items.find(item => item.id === store.page)?.label ?? '壁纸库');
const activeWallpaper = computed(() => store.settings.wallpaperId === 'companion' ? (store.settings.mode === 'wallpaper' ? '小伴房间' : '系统桌面') : store.library.items.find(item => item.definition.id === store.settings.wallpaperId)?.definition.title ?? '已保存的壁纸');
const unread = computed(() => store.website.announcements.filter(item => store.website.dismissed[item.id] !== item.revision).length);
function navigate(page: ControlPage) { store.navigate(page); document.querySelector('.control-workspace')?.scrollTo(0, 0); }
async function hide() { if (store.desktop) await invoke('hide_control'); else store.notify('浏览器预览不能收起到托盘', 'info'); }
async function quit() { if (store.desktop) await invoke('quit'); else store.notify('这是浏览器预览', 'info'); }
const focus = () => store.refreshSettings().catch(error => store.notify(error, 'error'));
onMounted(() => { void store.initialize().then(async () => { if (store.info.uiSmoke) await (await import('./ui-smoke.ts')).runUiSmoke(store); }); window.addEventListener('focus', focus); });
onBeforeUnmount(() => { store.dispose(); window.removeEventListener('focus', focus); });
</script>
<template>
  <div class="control-app" :class="{ 'is-collapsed': collapsed }">
    <aside class="control-sidebar">
      <div class="control-brand"><div class="control-brand-mark"><img src="/xiaoban.svg" alt="小伴"></div><div class="brand-copy"><strong>小伴桌面</strong><span>你的桌面生活空间</span></div></div>
      <div class="sidebar-group-label">我的桌面</div>
      <nav class="control-navigation" aria-label="功能分区">
        <button v-for="item in items" :key="item.id" :data-testid="`nav-${item.id}`" :class="{ selected: store.page === item.id }" :aria-current="store.page === item.id ? 'page' : undefined" :title="item.label" @click="navigate(item.id)"><component :is="item.icon" :size="19" /><span><b>{{ item.label }}</b><small>{{ item.hint }}</small></span><ChevronRight v-if="store.page === item.id" :size="15" class="nav-arrow" /></button>
      </nav>
      <div class="sidebar-desktop-state"><span class="state-dot" :class="{ idle: store.settings.paused || !store.settings.companionEnabled }"></span><b>{{ store.settings.companionEnabled ? (store.settings.paused ? '小伴休息中' : '小伴正在陪伴') : '小伴已隐藏' }}</b><small>{{ activeWallpaper }}</small></div>
      <div class="sidebar-bottom"><button class="settings-nav" data-testid="nav-settings" :class="{ selected: store.page === 'settings' }" @click="navigate('settings')"><Settings2 :size="18" /><span>设置与关于</span><i v-if="unread || store.website.update?.available" class="notice-dot"></i></button><div class="sidebar-meta"><span>v{{ store.info.version }}</span><NButton quaternary size="tiny" title="收起导航" @click="collapsed = !collapsed"><PanelLeftClose :size="16" /></NButton></div></div>
    </aside>
    <div class="control-main">
      <header class="control-topbar"><div class="breadcrumb"><span>我的桌面</span><ChevronRight :size="13" /><strong>{{ current }}</strong></div><div class="topbar-actions"><span v-if="!store.desktop" class="preview-label">浏览器预览</span><NButton quaternary size="small" title="收起到托盘" aria-label="收起到托盘" @click="hide"><Minus :size="17" /></NButton><NButton quaternary size="small" title="退出小伴" aria-label="退出小伴" @click="quit"><Power :size="15" /></NButton></div></header>
      <main class="control-workspace">
        <NAlert v-if="store.initialError" class="workspace-alert" type="error" title="桌面配置暂时无法读取">{{ store.initialError }} <NButton size="small" @click="store.initialize()">重试</NButton></NAlert>
        <div v-if="store.website.update?.available && store.page !== 'settings'" class="update-strip"><span><span class="notice-dot"></span>小伴有新版本 v{{ store.website.update.version }}</span><button @click="navigate('settings')">查看更新<ArrowUpRight :size="14" /></button></div>
        <div v-if="!store.ready" class="workspace-loading"><NSkeleton height="36px" width="220px" /><NSkeleton height="18px" width="320px" /><NSkeleton height="380px" :sharp="false" /></div>
        <template v-else-if="!store.initialError"><WallpaperPage v-if="store.page === 'wallpapers'" :store="store" /><CompanionPage v-else-if="store.page === 'companion'" :store="store" /><GamesPage v-else-if="store.page === 'games'" :store="store" /><CarePage v-else-if="store.page === 'care'" :store="store" /><SettingsPage v-else :store="store" /></template>
      </main>
      <footer class="control-statusbar"><span><span class="state-dot small"></span>{{ store.desktop ? (store.info.platform === 'windows' ? 'Windows 桌面已连接' : '桌面已连接') : '正在预览界面，桌面操作需在应用内使用' }}</span><span>{{ store.library.items.filter(item => item.installed).length }} 个本地壁纸</span></footer>
    </div>
  </div>
</template>
