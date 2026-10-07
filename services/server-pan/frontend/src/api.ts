import {
  FileListResponse,
  FileItem,
  ShareContentResponse,
  ShareDto,
  ShareInfoResponse,
  StorageStats,
  UpdateShareRequest,
  UserDto,
} from './types';

const API_BASE = '/api';

export function getToken(): string | null {
  return localStorage.getItem('pandown_token');
}

export function setToken(token: string) {
  localStorage.setItem('pandown_token', token);
}

export function removeToken() {
  localStorage.removeItem('pandown_token');
}

async function request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
  const token = getToken();
  const headers = new Headers(options.headers || {});

  if (token && !headers.has('Authorization')) {
    headers.set('Authorization', `Bearer ${token}`);
  }

  if (!(options.body instanceof FormData) && !headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json');
  }

  const response = await fetch(`${API_BASE}${endpoint}`, {
    ...options,
    headers,
  });

  if (!response.ok) {
    let errorMsg = '请求发生异常';
    try {
      const data = await response.json();
      errorMsg = data.error || errorMsg;
    } catch {
      errorMsg = `HTTP 错误: ${response.status}`;
    }
    throw new Error(errorMsg);
  }

  return response.json();
}

// 认证模块
export const authApi = {
  login: (username: string, password: string) =>
    request<{ token: string; user: UserDto }>('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ username, password }),
    }),

  getMe: () => request<UserDto>('/auth/me'),

  changePassword: (old_password: string, new_password: string) =>
    request<{ message: string }>('/auth/change-password', {
      method: 'POST',
      body: JSON.stringify({ old_password, new_password }),
    }),
};

// 文件管理模块
export const fileApi = {
  list: (folderId?: string | null) => {
    const url = folderId ? `/files/list?folder_id=${encodeURIComponent(folderId)}` : '/files/list';
    return request<FileListResponse>(url);
  },

  mkdir: (name: string, parentId?: string | null) =>
    request<FileItem>('/files/mkdir', {
      method: 'POST',
      body: JSON.stringify({ name, parent_id: parentId || null }),
    }),

  upload: async (files: File[], folderId?: string | null): Promise<FileItem[]> => {
    const formData = new FormData();
    for (const f of files) {
      formData.append('files', f);
    }
    const url = folderId ? `/files/upload?folder_id=${encodeURIComponent(folderId)}` : '/files/upload';
    return request<FileItem[]>(url, {
      method: 'POST',
      body: formData,
    });
  },

  rename: (id: string, new_name: string) =>
    request<FileItem>(`/files/rename/${encodeURIComponent(id)}`, {
      method: 'POST',
      body: JSON.stringify({ new_name }),
    }),

  delete: (id: string) =>
    request<{ message: string }>(`/files/${encodeURIComponent(id)}`, {
      method: 'DELETE',
    }),

  search: (q: string) =>
    request<FileItem[]>(`/files/search?q=${encodeURIComponent(q)}`),

  getStats: () => request<StorageStats>('/system/stats'),

  // 直链下载 / 预览：浏览器 <a href> / <img src> 无法携带 Authorization 头，
  // 因此把登录 Token 以 ?token= 形式附加，后端鉴权中间件已支持该回退方式。
  getDownloadUrl: (id: string) =>
    `${API_BASE}/files/download/${encodeURIComponent(id)}${authQuery()}`,
  getPreviewUrl: (id: string) =>
    `${API_BASE}/files/preview/${encodeURIComponent(id)}${authQuery()}`,
};

/// 构造登录凭证查询串，供直链使用
function authQuery(): string {
  const token = getToken();
  return token ? `?token=${encodeURIComponent(token)}` : '';
}

// 分享模块（登录态）
export const shareApi = {
  create: (file_id: string, password?: string, expire_days?: number, note?: string) =>
    request<{
      code: string;
      share_url: string;
      has_password: boolean;
      password?: string;
      note?: string;
      expires_at?: number;
    }>('/shares/create', {
      method: 'POST',
      body: JSON.stringify({
        file_id,
        password: password || null,
        expire_days: expire_days ?? null,
        note: note || null,
      }),
    }),

  update: (code: string, payload: UpdateShareRequest) =>
    request<{ message: string }>(`/shares/${encodeURIComponent(code)}`, {
      method: 'PATCH',
      body: JSON.stringify(payload),
    }),

  getMyShares: () => request<ShareDto[]>('/shares/my'),

  delete: (code: string) =>
    request<{ message: string }>(`/shares/${encodeURIComponent(code)}`, {
      method: 'DELETE',
    }),
};

// 公开访客端模块（无需登录账号）
export const publicApi = {
  getInfo: (code: string) =>
    request<ShareInfoResponse>(`/public/shares/${encodeURIComponent(code)}/info`),

  verify: (code: string, password?: string) =>
    request<{ share_token: string }>(`/public/shares/${encodeURIComponent(code)}/verify`, {
      method: 'POST',
      body: JSON.stringify({ password: password || null }),
    }),

  getContent: (code: string, subFolderId?: string | null, token?: string | null) => {
    const params = new URLSearchParams();
    if (subFolderId) params.append('sub_folder_id', subFolderId);
    if (token) params.append('token', token);
    const qs = params.toString() ? `?${params.toString()}` : '';

    return request<ShareContentResponse>(`/public/shares/${encodeURIComponent(code)}/content${qs}`, {
      headers: token ? { 'X-Share-Token': token } : {},
    });
  },

  getDownloadUrl: (code: string, fileId: string, token?: string | null) => {
    const qs = token ? `?token=${encodeURIComponent(token)}` : '';
    return `${API_BASE}/public/shares/${encodeURIComponent(code)}/download/${encodeURIComponent(fileId)}${qs}`;
  },

  getPreviewUrl: (code: string, fileId: string, token?: string | null) => {
    const qs = token ? `?token=${encodeURIComponent(token)}` : '';
    return `${API_BASE}/public/shares/${encodeURIComponent(code)}/preview/${encodeURIComponent(fileId)}${qs}`;
  },
};

// 格式化辅助函数
export function formatBytes(bytes: number, decimals = 1): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
}

export function formatDate(timestamp: number): string {
  if (!timestamp) return '-';
  const date = new Date(timestamp * 1000);
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
}
