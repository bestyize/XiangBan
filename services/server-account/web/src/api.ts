import { session, signOut } from './session';
import type { AuditRow, ClientRow, Stats, UserDetail, UserList } from './types';

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
    response = await fetch(`/api${path}`, { method, headers, body: body === undefined ? undefined : JSON.stringify(body) });
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
  users: (params: { q: string; status: string; page: number }) => {
    const query = new URLSearchParams({ page: String(params.page), page_size: '20' });
    if (params.q) query.set('q', params.q);
    if (params.status) query.set('status', params.status);
    return request<UserList>('GET', `/admin/users?${query}`);
  },
  createUser: (body: { request_id: string; username: string; password: string; avatar_url?: string }) =>
    request<{ uid: string }>('POST', '/admin/users', body),
  user: (uid: string) => request<UserDetail>('GET', `/admin/users/${uid}`),
  setUserStatus: (uid: string, status: 'active' | 'disabled') =>
    request<{ ok: true }>('POST', `/admin/users/${uid}/status`, { status }),
  renameUser: (uid: string, username: string) =>
    request<{ ok: true; username: string }>('POST', `/admin/users/${uid}/username`, { username }),
  revokeSessions: (uid: string) => request<{ ok: true }>('POST', `/admin/users/${uid}/revoke-sessions`),
  clients: () => request<{ items: ClientRow[] }>('GET', '/admin/clients'),
  createClient: (client_id: string, name: string) => request<{ ok: true }>('POST', '/admin/clients', { client_id, name }),
  updateClient: (client_id: string, patch: { name?: string; enabled?: boolean }) =>
    request<{ ok: true }>('PATCH', `/admin/clients/${encodeURIComponent(client_id)}`, patch),
  audit: () => request<{ items: AuditRow[] }>('GET', '/admin/audit?limit=100'),
  getSender: () => request<import('./types').SenderConfig>('GET', '/admin/sender'),
  updateSender: (body: { provider: string; smtp?: import('./types').SmtpConfig }) =>
    request<{ ok: true }>('PUT', '/admin/sender', body),
  testSender: (body: { recipient: string; smtp?: import('./types').SmtpConfig }) =>
    request<{ ok: true; message: string }>('POST', '/admin/sender/test', body),
};

export function formatTime(value: string | null): string {
  if (!value) return '—';
  return new Date(value).toLocaleString('zh-CN', { hour12: false });
}

export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : '操作失败';
}

