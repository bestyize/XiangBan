export interface AccountUser {
  uid: string;
  account_id: string;
  username: string;
  avatar_url?: string | null;
  email?: string | null;
  phone?: string | null;
}

export interface AuthSession {
  accessToken: string;
  refreshToken: string;
  expiresAt: number;
  user: AccountUser;
}

export interface AuthMethods {
  email: boolean;
  phone: boolean;
}

export interface CodeLoginResult {
  registrationRequired: boolean;
  registrationToken?: string;
  expiresIn?: number;
  session?: AuthSession;
}

const SESSION_KEY = 'xb_hall_account_session_v1';
const DEFAULT_BASE = '/account-api';
const DEFAULT_CLIENT_ID = 'xiaban-web';

export function getAccountApiBase(): string {
  const envBase = (import.meta as unknown as { env?: Record<string, string | undefined> }).env?.VITE_ACCOUNT_API_BASE;
  return (envBase || DEFAULT_BASE).replace(/\/+$/, '');
}

function apiUrl(path: string): string {
  const base = getAccountApiBase();
  // 同源反代 /account-api 映射到上游 /api/*，直接指向独立域名时保留 /api 前缀
  if (base.endsWith('/account-api') && path.startsWith('/api/')) {
    return `${base}${path.slice(4)}`;
  }
  return `${base}${path}`;
}

export function getAccountClientId(): string {
  const envClient = (import.meta as unknown as { env?: Record<string, string | undefined> }).env?.VITE_ACCOUNT_CLIENT_ID;
  return envClient || DEFAULT_CLIENT_ID;
}

export function loadSavedSession(): AuthSession | null {
  try {
    const raw = localStorage.getItem(SESSION_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as AuthSession;
    if (!parsed?.accessToken || !parsed?.user?.username) return null;
    return parsed;
  } catch {
    return null;
  }
}

export function persistSession(session: AuthSession | null): void {
  try {
    if (!session) localStorage.removeItem(SESSION_KEY);
    else localStorage.setItem(SESSION_KEY, JSON.stringify(session));
  } catch { /* ignore */ }
}

async function requestJson<T>(path: string, init: RequestInit = {}): Promise<T> {
  const res = await fetch(apiUrl(path), {
    ...init,
    headers: {
      'Content-Type': 'application/json',
      ...(init.headers || {})
    }
  });
  const data = await res.json().catch(() => ({}));
  if (!res.ok) {
    const message = typeof data?.message === 'string' ? data.message : `请求失败 (${res.status})`;
    const err = new Error(message) as Error & { code?: string; retryAfter?: number; status?: number };
    err.code = data?.code;
    err.retryAfter = typeof data?.retry_after === 'number' ? data.retry_after : undefined;
    err.status = res.status;
    throw err;
  }
  return data as T;
}

export async function fetchAuthMethods(): Promise<AuthMethods> {
  try {
    const data = await requestJson<Partial<AuthMethods>>('/api/auth/methods', { method: 'GET' });
    return { email: data.email !== false, phone: Boolean(data.phone) };
  } catch {
    return { email: true, phone: false };
  }
}

export async function sendVerifyCode(identifier: string, purpose: 'login' | 'reset_password' = 'login'): Promise<void> {
  await requestJson('/api/auth/code/send', {
    method: 'POST',
    body: JSON.stringify({ identifier: identifier.trim(), purpose })
  });
}

export async function loginWithCode(identifier: string, code: string): Promise<CodeLoginResult> {
  const data = await requestJson<{
    registration_required?: boolean;
    registration_token?: string;
    expires_in?: number;
    access_token?: string;
    refresh_token?: string;
    user?: AccountUser;
  }>('/api/auth/code/login', {
    method: 'POST',
    body: JSON.stringify({
      identifier: identifier.trim(),
      code: code.trim(),
      client_id: getAccountClientId(),
      device_name: '小伴游戏厅 H5'
    })
  });

  if (data.registration_required && data.registration_token) {
    return {
      registrationRequired: true,
      registrationToken: data.registration_token,
      expiresIn: data.expires_in
    };
  }

  if (!data.access_token || !data.refresh_token || !data.user) {
    throw new Error('账号服务返回异常，请重试');
  }
  const session: AuthSession = {
    accessToken: data.access_token,
    refreshToken: data.refresh_token,
    expiresAt: Date.now() + (data.expires_in || 1800) * 1000,
    user: data.user
  };
  persistSession(session);
  return { registrationRequired: false, session };
}

export async function checkUsernameAvailable(username: string): Promise<{ available: boolean; message: string }> {
  const q = encodeURIComponent(username.trim());
  const data = await requestJson<{ available?: boolean; message?: string }>(
    `/api/auth/username/available?username=${q}`,
    { method: 'GET' }
  );
  return {
    available: Boolean(data.available),
    message: data.message || (data.available ? '用户名可用 · 注册后不可修改' : '该用户名不可用')
  };
}

export async function completeRegistration(
  registrationToken: string,
  username: string,
  password: string
): Promise<AuthSession> {
  const data = await requestJson<{
    access_token: string;
    refresh_token: string;
    expires_in: number;
    user: AccountUser;
  }>('/api/auth/register', {
    method: 'POST',
    body: JSON.stringify({
      registration_token: registrationToken,
      username: username.trim(),
      password,
      client_id: getAccountClientId(),
      device_name: '小伴游戏厅 H5'
    })
  });
  const session: AuthSession = {
    accessToken: data.access_token,
    refreshToken: data.refresh_token,
    expiresAt: Date.now() + (data.expires_in || 1800) * 1000,
    user: data.user
  };
  persistSession(session);
  return session;
}

export async function loginWithPassword(identifier: string, password: string): Promise<AuthSession> {
  const data = await requestJson<{
    access_token: string;
    refresh_token: string;
    expires_in: number;
    user: AccountUser;
  }>('/api/auth/password/login', {
    method: 'POST',
    body: JSON.stringify({
      identifier: identifier.trim(),
      password,
      client_id: getAccountClientId(),
      device_name: '小伴游戏厅 H5'
    })
  });
  const session: AuthSession = {
    accessToken: data.access_token,
    refreshToken: data.refresh_token,
    expiresAt: Date.now() + (data.expires_in || 1800) * 1000,
    user: data.user
  };
  persistSession(session);
  return session;
}

export async function resetPassword(identifier: string, code: string, newPassword: string): Promise<void> {
  await requestJson('/api/auth/password/reset', {
    method: 'POST',
    body: JSON.stringify({
      identifier: identifier.trim(),
      code: code.trim(),
      new_password: newPassword
    })
  });
}

export async function logoutSession(session: AuthSession | null): Promise<void> {
  persistSession(null);
  if (!session?.refreshToken) return;
  try {
    await requestJson('/api/auth/logout', {
      method: 'POST',
      body: JSON.stringify({ refresh_token: session.refreshToken })
    });
  } catch {
    // Local session is already cleared
  }
}
