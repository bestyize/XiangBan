import type { AppItem, Category, AdminStats } from '../types';

const API_BASE = '/api';

function getAuthHeader(): Record<string, string> {
  const token = localStorage.getItem('xiaoban_appstore_admin_token');
  return token ? { Authorization: `Bearer ${token}` } : {};
}

async function request<T>(url: string, options: RequestInit = {}): Promise<T> {
  const headers = {
    'Content-Type': 'application/json',
    ...getAuthHeader(),
    ...options.headers,
  };

  const res = await fetch(`${API_BASE}${url}`, {
    ...options,
    headers,
  });

  if (!res.ok) {
    let errMsg = `请求失败: ${res.status}`;
    try {
      const errData = await res.json();
      if (errData.message) errMsg = errData.message;
    } catch {
      // ignore
    }
    throw new Error(errMsg);
  }

  return res.json();
}

export const api = {
  // 公开接口
  async getApps(params?: { category?: string; search?: string; tag?: string; platform?: string }): Promise<{ apps: AppItem[]; total: number }> {
    const query = new URLSearchParams();
    if (params?.category) query.append('category', params.category);
    if (params?.search) query.append('search', params.search);
    if (params?.tag) query.append('tag', params.tag);
    if (params?.platform) query.append('platform', params.platform);
    return request<{ apps: AppItem[]; total: number }>(`/apps?${query.toString()}`);
  },

  async getApp(slug: string): Promise<AppItem> {
    return request<AppItem>(`/apps/${slug}`);
  },

  async getCategories(): Promise<Category[]> {
    return request<Category[]>('/categories');
  },

  async getFeatured(): Promise<AppItem[]> {
    return request<AppItem[]>('/featured');
  },

  async trackDownload(slug: string): Promise<void> {
    try {
      await request(`/apps/${slug}/download`, { method: 'POST' });
    } catch {
      // silent
    }
  },

  // 管理后台接口
  async login(credentials: { username: string; password: string }): Promise<{ token: string; username: string }> {
    return request<{ token: string; username: string }>('/admin/login', {
      method: 'POST',
      body: JSON.stringify(credentials),
    });
  },

  async getMe(): Promise<{ username: string; role: string }> {
    return request<{ username: string; role: string }>('/admin/me');
  },

  async changeAdminCredentials(data: { current_password: string; new_username?: string; new_password?: string }): Promise<{ success: boolean; message: string; username: string; token: string }> {
    return request<{ success: boolean; message: string; username: string; token: string }>('/admin/account', {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  },

  async getAdminStats(): Promise<AdminStats> {
    return request<AdminStats>('/admin/stats');
  },

  async getAdminApps(params?: { status?: string; category?: string; search?: string }): Promise<{ apps: AppItem[]; total: number }> {
    const query = new URLSearchParams();
    if (params?.status) query.append('status', params.status);
    if (params?.category) query.append('category', params.category);
    if (params?.search) query.append('search', params.search);
    return request<{ apps: AppItem[]; total: number }>(`/admin/apps?${query.toString()}`);
  },

  async createApp(data: Partial<AppItem>): Promise<AppItem> {
    return request<AppItem>('/admin/apps', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  },

  async updateApp(id: number, data: Partial<AppItem>): Promise<{ success: boolean; id: number }> {
    return request<{ success: boolean; id: number }>(`/admin/apps/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  },

  async deleteApp(id: number): Promise<{ success: boolean }> {
    return request<{ success: boolean }>(`/admin/apps/${id}`, {
      method: 'DELETE',
    });
  },

  // 核心特性：一键上架 / 下架
  async updateAppStatus(id: number, status: 'published' | 'unlisted'): Promise<{ success: boolean; id: number; status: string }> {
    return request<{ success: boolean; id: number; status: string }>(`/admin/apps/${id}/status`, {
      method: 'PATCH',
      body: JSON.stringify({ status }),
    });
  },

  // 核心特性：单个应用一键采集最新版本与直接安装包
  async syncAppVersion(id: number): Promise<{ success: boolean; version?: string; mac_download_url?: string; win_download_url?: string; linux_download_url?: string; file_size?: string }> {
    return request<{ success: boolean; version?: string; mac_download_url?: string; win_download_url?: string; linux_download_url?: string; file_size?: string }>(`/admin/apps/${id}/sync-version`, {
      method: 'POST',
    });
  },

  // 核心特性：全量采集同步所有软件的最新版本与直接下载包
  async syncAllVersions(): Promise<{ success: boolean; total: number; updated: number }> {
    return request<{ success: boolean; total: number; updated: number }>('/admin/apps/sync-all', {
      method: 'POST',
    });
  },

  // 直接下载端点 URL (用于文件直接直链下载，非跳去官网)
  getDirectDownloadUrl(slug: string): string {
    return `${API_BASE}/apps/${slug}/direct-download`;
  },

  async getAdminCategories(): Promise<Category[]> {
    return request<Category[]>('/admin/categories');
  },
};
