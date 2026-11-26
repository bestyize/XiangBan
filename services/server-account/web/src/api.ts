import { session, signOut } from './session';
import type { AuditRow, ClientRow, SenderConfig, SignupMethods, SmtpConfig, Stats, UserDetail, UserList } from './types';

export class ApiError extends Error {
  constructor(message: string, public status: number) {
    super(message);
  }
}

async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
  const headers: Record<string, string> = {};
  if (body !== undefined) headers['Content-Type'] = 'application/json';
  if (session.token) headers.Authorization = `Bearer ${session.token}`;

  let response: Response;
  try {
    response = await fetch(`/api${path}`, {
      method,
      headers,
      body: body === undefined ? undefined : JSON.stringify(body),
    });
  } catch {
    throw new ApiError('无法连接服务器，请检查网络', 0);
  }
  const data = await response.json().catch(() => null);
  if (!response.ok) {
    if (response.status === 401 && session.token && path !== '/admin/login') signOut();
    throw new ApiError(data?.message ?? `请求失败（${response.status}）`, response.status);
  }
  return data as T;
}

export const api = {
  login: (username: string, password: string) =>
    request<{ token: string; admin: { username: string } }>('POST', '/admin/login', { username, password }),
  changePassword: (current_password: string, new_password: string) =>
    request<{ ok: true }>('POST', '/admin/password', { current_password, new_password }),
  stats: () => request<Stats>('GET', '/admin/stats'),
  users: (params: { q: string; status: string; page: number; page_size?: number }) => {
    const query = new URLSearchParams({
      page: String(params.page),
      page_size: String(params.page_size ?? 20),
    });
    if (params.q) query.set('q', params.q);
    if (params.status) query.set('status', params.status);
    return request<UserList>('GET', `/admin/users?${query}`);
  },
  createUser: (body: { request_id: string; username: string; password: string; avatar_url?: string }) =>
    request<{ uid: string; account_id?: string; username?: string }>('POST', '/admin/users', body),
  user: (uid: string) => request<UserDetail>('GET', `/admin/users/${uid}`),
  setUserStatus: (uid: string, status: 'active' | 'disabled') =>
    request<{ ok: true }>('POST', `/admin/users/${uid}/status`, { status }),
  renameUser: (uid: string, username: string) =>
    request<{ ok: true; username: string }>('POST', `/admin/users/${uid}/username`, { username }),
  revokeSessions: (uid: string) => request<{ ok: true }>('POST', `/admin/users/${uid}/revoke-sessions`),
  clients: () => request<{ items: ClientRow[] }>('GET', '/admin/clients'),
  createClient: (client_id: string, name: string) =>
    request<{ ok: true }>('POST', '/admin/clients', { client_id, name }),
  updateClient: (client_id: string, patch: { name?: string; enabled?: boolean }) =>
    request<{ ok: true }>('PATCH', `/admin/clients/${encodeURIComponent(client_id)}`, patch),
  audit: (limit = 100) => request<{ items: AuditRow[] }>('GET', `/admin/audit?limit=${limit}`),
  getSender: () => request<SenderConfig>('GET', '/admin/sender'),
  updateSender: (body: { provider: string; smtp?: SmtpConfig }) =>
    request<{ ok: true }>('PUT', '/admin/sender', body),
  getSignupMethods: () => request<SignupMethods>('GET', '/admin/signup-methods'),
  updateSignupMethods: (body: SignupMethods) =>
    request<SignupMethods>('PUT', '/admin/signup-methods', body),
  testSender: (body: { recipient: string; smtp?: SmtpConfig }) =>
    request<{ ok: true; message: string }>('POST', '/admin/sender/test', body),
};

export function formatTime(value: string | null): string {
  if (!value) return '—';
  const d = new Date(value);
  if (Number.isNaN(d.getTime())) return '—';
  return d.toLocaleString('zh-CN', { hour12: false });
}

export function formatRelativeTime(value: string | null): string {
  if (!value) return '从未';
  const d = new Date(value);
  if (Number.isNaN(d.getTime())) return '—';
  const diff = Date.now() - d.getTime();
  if (diff < 0) return formatTime(value);
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return '刚刚';
  if (mins < 60) return `${mins} 分钟前`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours} 小时前`;
  const days = Math.floor(hours / 24);
  if (days < 7) return `${days} 天前`;
  return d.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' });
}

export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : '操作失败';
}
