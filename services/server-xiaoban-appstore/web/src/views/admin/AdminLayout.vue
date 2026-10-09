<template>
  <div class="admin-wrapper">
    <!-- 左侧经典侧边栏菜单 -->
    <aside class="admin-sidebar">
      <div class="sidebar-brand">
        <span class="brand-emoji">❖</span>
        <div class="brand-title-box">
          <h2 class="sidebar-title">小伴精选管理</h2>
          <span class="sidebar-sub">AppStore Admin</span>
        </div>
      </div>

      <!-- 导航菜单 -->
      <nav class="sidebar-nav">
        <router-link to="/admin/dashboard" class="nav-item" active-class="active">
          <LayoutDashboard :size="18" />
          <span>仪表盘概览</span>
        </router-link>
        <router-link to="/admin/apps" class="nav-item" active-class="active">
          <Package :size="18" />
          <span>应用库管理</span>
        </router-link>
        <router-link to="/admin/categories" class="nav-item" active-class="active">
          <Tag :size="18" />
          <span>分类体系</span>
        </router-link>
        <router-link to="/admin/settings" class="nav-item" active-class="active">
          <Shield :size="18" />
          <span>账户与安全</span>
        </router-link>
      </nav>

      <!-- 底部操作区 -->
      <div class="sidebar-footer">
        <router-link to="/" class="footer-link">
          <ExternalLink :size="16" />
          <span>前往前台商店</span>
        </router-link>
        <button class="footer-link btn-logout" @click="handleLogout">
          <LogOut :size="16" />
          <span>退出登录</span>
        </button>
      </div>
    </aside>

    <!-- 右侧主体内容区 -->
    <div class="admin-main-area">
      <!-- 顶栏 Header -->
      <header class="admin-header">
        <div class="header-left">
          <span class="location-label">控制台</span>
          <span class="divider">/</span>
          <span class="current-page">{{ currentRouteTitle }}</span>
        </div>
        <div class="header-right">
          <router-link to="/admin/settings" class="admin-badge admin-badge-link" title="点击进入账户安全设置">
            <User :size="14" />
            <span>{{ authStore.username || 'admin' }}</span>
          </router-link>
        </div>
      </header>

      <!-- 内容页面渲染区 -->
      <main class="admin-content-view">
        <router-view />
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useAuthStore } from '../../stores/auth';
import {
  LayoutDashboard,
  Package,
  Tag,
  Shield,
  ExternalLink,
  LogOut,
  User,
} from 'lucide-vue-next';

const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();

const currentRouteTitle = computed(() => {
  if (route.path.includes('dashboard')) return '仪表盘概览';
  if (route.path.includes('apps')) return '应用库管理（上架与下架）';
  if (route.path.includes('categories')) return '分类体系管理';
  if (route.path.includes('settings')) return '管理员账号与安全设置';
  return '管理控制台';
});

function handleLogout() {
  authStore.logout();
  router.push('/admin/login');
}
</script>

<style scoped>
.admin-wrapper {
  display: flex;
  min-height: 100vh;
  background-color: #f8fafc;
}

/* 左侧经典侧边栏 (固定 240px) */
.admin-sidebar {
  width: 240px;
  min-width: 240px;
  background: #0f172a;
  color: #f8fafc;
  display: flex;
  flex-direction: column;
  position: fixed;
  top: 0;
  bottom: 0;
  left: 0;
  z-index: 100;
  border-right: 1px solid #1e293b;
}

.sidebar-brand {
  height: 64px;
  padding: 0 20px;
  display: flex;
  align-items: center;
  gap: 12px;
  border-bottom: 1px solid #1e293b;
}

.brand-emoji {
  font-size: 26px;
  line-height: 1;
}

.sidebar-title {
  font-size: 15px;
  font-weight: 700;
  color: #f8fafc;
  letter-spacing: -0.2px;
}

.sidebar-sub {
  font-size: 11px;
  color: #64748b;
  display: block;
}

.sidebar-nav {
  padding: 18px 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  border-radius: 8px;
  font-size: 14px;
  color: #94a3b8;
  font-weight: 500;
  transition: all 0.15s ease;
}

.nav-item:hover {
  background: #1e293b;
  color: #f8fafc;
}

.nav-item.active {
  background: #2563eb;
  color: #ffffff;
  font-weight: 600;
  box-shadow: 0 2px 8px rgba(37, 99, 235, 0.3);
}

.sidebar-footer {
  padding: 16px 12px;
  border-top: 1px solid #1e293b;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.footer-link {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 13px;
  color: #94a3b8;
  transition: all 0.15s ease;
}

.footer-link:hover {
  background: #1e293b;
  color: #f8fafc;
}

.btn-logout {
  color: #ef4444;
}

.btn-logout:hover {
  background: #450a0a;
  color: #fca5a5;
}

/* 右侧内容工作区 */
.admin-main-area {
  margin-left: 240px;
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.admin-header {
  height: 64px;
  background: #ffffff;
  border-bottom: 1px solid #e2e8f0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 28px;
  position: sticky;
  top: 0;
  z-index: 90;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
}

.location-label {
  color: #94a3b8;
}

.divider {
  color: #cbd5e1;
}

.current-page {
  font-weight: 600;
  color: #1e293b;
}

.admin-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: #f1f5f9;
  color: #334155;
  padding: 5px 12px;
  border-radius: 20px;
  font-size: 13px;
  font-weight: 500;
  transition: all 0.15s ease;
}

.admin-badge-link:hover {
  background: #eff6ff;
  color: #2563eb;
}

.admin-content-view {
  padding: 28px;
  flex: 1;
}
</style>
