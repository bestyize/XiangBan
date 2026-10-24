import { reactive } from 'vue';
import type {
  AdminCategory, AdminItem, AdminMember, AdminVersion, CategoryCount, ItemDetail, Job, JobLog, Kind, ListResponse, Plural, ProbeResult, Source,
  StagedDetail, StagedRow, Stats,
} from './types';

const TOKEN_KEY = 'xiaoban-market-token';

function load(key: string): string {
  try {
    return localStorage.getItem(key) ?? '';
  } catch {
    return '';
  }
}

export const session = reactive({ token: load(TOKEN_KEY), username: '' });

export function setSession(token: string, username = '') {
  session.token = token;
  session.username = username;
  try {
    token ? localStorage.setItem(TOKEN_KEY, token) : localStorage.removeItem(TOKEN_KEY);
  } catch {
    /* 隐私模式下无法持久化，忽略 */
  }
}

export class ApiError extends Error {
  constructor(public status: number, message: string) {
    super(message);
  }
}

/** 后台请求收到 401 时调用：由路由层把用户送回登录页。 */
export const hooks: { onUnauthorized: () => void } = { onUnauthorized: () => {} };

type Query = Record<string, string | number | boolean | undefined | null>;

export function toQuery(q: Query = {}): string {
  const p = new URLSearchParams();
  for (const [k, v] of Object.entries(q)) {
    if (v !== undefined && v !== null && v !== '') p.set(k, String(v));
  }
  const s = p.toString();
  return s ? `?${s}` : '';
}

async function request<T>(method: string, path: string, opts: { query?: Query; body?: unknown; form?: FormData; admin?: boolean } = {}): Promise<T> {
  const headers: Record<string, string> = { accept: 'application/json' };
  if (opts.admin && session.token) headers.authorization = `Bearer ${session.token}`;
  let body: BodyInit | undefined;
  if (opts.form) body = opts.form;
  else if (opts.body !== undefined) {
    headers['content-type'] = 'application/json';
    body = JSON.stringify(opts.body);
  }
  let resp: Response;
  try {
    resp = await fetch(`/api/v1${path}${toQuery(opts.query)}`, { method, headers, body });
  } catch {
    throw new ApiError(0, '无法连接服务，请检查网络后重试');
  }
  const data = await resp.json().catch(() => null);
  if (!resp.ok) {
    if (resp.status === 401 && opts.admin && path !== '/admin/login') {
      setSession('');
      hooks.onUnauthorized();
    }
    throw new ApiError(resp.status, data?.message ?? `请求失败（HTTP ${resp.status}）`);
  }
  return data as T;
}

export interface ListQuery {
  keyword?: string;
  category?: string;
  source?: string;
  variant?: string;
  origin?: string;
  sortBy?: string;
  page?: number;
  pageSize?: number;
}

export const market = {
  list: (kind: Plural, q: ListQuery) => request<ListResponse>('GET', `/market/${kind}`, { query: q as Query }),
  categories: (kind: Plural) => request<{ results: CategoryCount[] }>('GET', '/market/categories', { query: { kind } }),
  suggest: (kind: Plural, q: string) => request<{ results: { name: string; namespace: string; slug: string }[] }>('GET', '/market/suggest', { query: { kind, q } }),
  detail: (kind: Plural, ns: string, slug: string) => request<ItemDetail>('GET', `/market/${kind}/${encodeURIComponent(ns)}/${encodeURIComponent(slug)}`),
};

export interface LoginConfig {
  account: { enabled: boolean; url: string; clientId: string };
}

export interface AccountBinding {
  id: string;
  accountId: string;
  label: string;
  boundBy: string;
  createdAt: string;
}

/**
 * 直接向相伴账号中心用「账号 / 邮箱 / 手机号 + 密码」登录，拿到访问令牌。
 * 密码只发给账号中心；小伴市场后台只收到令牌，并在服务端用 JWKS 验签。
 */
export async function accountPasswordLogin(accountUrl: string, clientId: string, identifier: string, password: string): Promise<string> {
  let resp: Response;
  try {
    resp = await fetch(`${accountUrl.replace(/\/+$/, '')}/api/auth/password/login`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ identifier: identifier.trim(), password, client_id: clientId, device_name: '小伴市场后台' }),
    });
  } catch {
    throw new ApiError(0, '无法连接账号中心，请检查网络后重试');
  }
  const data = await resp.json().catch(() => null);
  if (!resp.ok || !data?.access_token) {
    throw new ApiError(resp.status, data?.message ?? data?.error ?? `账号中心登录失败（HTTP ${resp.status}）`);
  }
  return data.access_token as string;
}

type Paged<T> = { total: number; page: number; pageSize: number; results: T[] };
const a = { admin: true };

export const admin = {
  config: () => request<LoginConfig>('GET', '/admin/config'),
  loginWithAccount: (accessToken: string) => request<{ token: string; username: string }>('POST', '/admin/login/account', { body: { accessToken } }),
  bindings: () => request<{ enabled: boolean; results: AccountBinding[] }>('GET', '/admin/account/bindings', a),
  bindAccount: (accessToken: string, label: string) => request<{ ok: boolean }>('POST', '/admin/account/bindings', { ...a, body: { accessToken, label } }),
  unbindAccount: (id: string) => request<{ ok: boolean }>('DELETE', `/admin/account/bindings/${id}`, a),
  login: (username: string, password: string) => request<{ token: string; username: string }>('POST', '/admin/login', { body: { username, password } }),
  me: () => request<{ username: string }>('GET', '/admin/me', a),
  changePassword: (currentPassword: string, newPassword: string) => request<{ ok: boolean }>('PUT', '/admin/account', { ...a, body: { currentPassword, newPassword } }),
  stats: () => request<Stats>('GET', '/admin/stats', a),

  sources: () => request<{ results: Source[] }>('GET', '/admin/sources', a),
  updateSource: (id: number, body: Partial<{ name: string; enabled: boolean; intervalMinutes: number; autoImport: string; config: unknown }>) =>
    request<Source>('PUT', `/admin/sources/${id}`, { ...a, body }),
  probe: (id: number, params: Record<string, unknown> = {}) => request<ProbeResult>('POST', `/admin/sources/${id}/probe`, { ...a, body: { params } }),
  startJob: (id: number, params: Record<string, unknown> = {}) => request<{ id: string }>('POST', `/admin/sources/${id}/jobs`, { ...a, body: params }),

  jobs: (q: { sourceId?: number; status?: string; page?: number } = {}) => request<Paged<Job>>('GET', '/admin/jobs', { ...a, query: q }),
  job: (id: string, after = 0) => request<{ job: Job; logs: JobLog[] }>('GET', `/admin/jobs/${id}`, { ...a, query: { after } }),
  cancelJob: (id: string) => request<{ ok: boolean }>('POST', `/admin/jobs/${id}/cancel`, { ...a, body: {} }),

  staged: (q: { sourceId?: number; status?: string; change?: string; keyword?: string; page?: number } = {}) =>
    request<Paged<StagedRow> & { counts: Record<string, number> }>('GET', '/admin/staged', { ...a, query: q }),
  stagedDetail: (id: string) => request<StagedDetail>('GET', `/admin/staged/${id}`, a),
  importStaged: (ids: string[], body: { status?: string; categoryId?: number } = {}) =>
    request<{ results: { id: string; ok: boolean; itemId?: string; error?: string }[] }>('POST', '/admin/staged/import', { ...a, body: { ids, ...body } }),
  ignoreStaged: (ids: string[]) => request<{ updated: number }>('POST', '/admin/staged/ignore', { ...a, body: { ids } }),
  restoreStaged: (ids: string[]) => request<{ updated: number }>('POST', '/admin/staged/restore', { ...a, body: { ids } }),

  items: (q: { kind?: Kind; status?: string; variant?: string; categoryId?: number; keyword?: string; page?: number } = {}) => request<Paged<AdminItem>>('GET', '/admin/items', { ...a, query: q }),
  item: (id: string) => request<{ item: AdminItem; versions: AdminVersion[]; members: AdminMember[] }>('GET', `/admin/items/${id}`, a),
  createItem: (body: Record<string, unknown>) => request<{ id: string }>('POST', '/admin/items', { ...a, body }),
  updateItem: (id: string, body: Record<string, unknown>) => request<{ ok: boolean }>('PUT', `/admin/items/${id}`, { ...a, body }),
  setItemStatus: (id: string, status: string) => request<{ ok: boolean }>('POST', `/admin/items/${id}/status`, { ...a, body: { status } }),
  deleteItem: (id: string) => request<{ ok: boolean }>('DELETE', `/admin/items/${id}`, a),
  uploadVersion: (id: string, form: FormData) => request<{ version: string; files: number }>('POST', `/admin/items/${id}/versions`, { ...a, form }),
  yankVersion: (id: string, yanked: boolean) => request<{ ok: boolean }>('PUT', `/admin/versions/${id}`, { ...a, body: { yanked } }),

  categories: (kind?: Kind) => request<{ results: AdminCategory[] }>('GET', '/admin/categories', { ...a, query: { kind } }),
  createCategory: (body: { kind: Kind; name: string; slug?: string }) => request<{ id: number }>('POST', '/admin/categories', { ...a, body }),
  updateCategory: (id: number, body: Partial<{ name: string; nameEn: string; sortOrder: number; active: boolean }>) => request<{ ok: boolean }>('PUT', `/admin/categories/${id}`, { ...a, body }),
  deleteCategory: (id: number) => request<{ ok: boolean }>('DELETE', `/admin/categories/${id}`, a),
};
