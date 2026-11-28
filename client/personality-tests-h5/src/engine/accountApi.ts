export interface AccountIdentity {
  kind: 'email' | 'phone' | string;
  masked: string;
}

export interface AccountUser {
  uid: string;
  account_id: string;
  username: string;
  avatar_url: string | null;
  has_password: boolean;
  identities: AccountIdentity[];
  created_at: string;
  email?: string | null;
}

export interface AuthTokens {
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
}

export interface AuthSessionResponse extends AuthTokens {
  is_new_user: boolean;
  user: AccountUser;
}

export interface RegistrationRequiredResponse {
  registration_required: true;
  registration_token: string;
  expires_in: number;
}

export type CodeLoginResponse =
  | RegistrationRequiredResponse
  | (AuthSessionResponse & { registration_required?: false });

export interface SignupMethods {
  email: boolean;
  phone: boolean;
}

export interface SendCodeResponse {
  ok: boolean;
  cooldown_seconds: number;
  expires_in: number;
}

export interface UsernameCheckResponse {
  available: boolean;
  message: string | null;
}

export class AccountApiError extends Error {
  status: number;
  code: string;
  retryAfter?: number;

  constructor(message: string, status = 0, code = 'request_failed', retryAfter?: number) {
    super(message);
    this.name = 'AccountApiError';
    this.status = status;
    this.code = code;
    this.retryAfter = retryAfter;
  }
}

const DEFAULT_API_BASE = '/account-api';
const DEFAULT_CLIENT_ID = 'xiaban-web';

function resolveEnv(key: string): string | undefined {
  try {
    const env = (import.meta as unknown as { env?: Record<string, string | undefined> }).env;
    const val = env?.[key];
    return val && val.trim() ? val.trim() : undefined;
  } catch {
    return undefined;
  }
}

export function getAccountApiBase(): string {
  return (resolveEnv('VITE_ACCOUNT_API_BASE') || DEFAULT_API_BASE).replace(/\/+$/, '');
}

export function getAccountClientId(): string {
  return resolveEnv('VITE_ACCOUNT_CLIENT_ID') || DEFAULT_CLIENT_ID;
}

export function getDeviceLabel(): string {
  if (typeof navigator === 'undefined') return '人格贩卖机 H5';
  const ua = navigator.userAgent || '';
  if (/MicroMessenger/i.test(ua)) return '人格贩卖机 · 微信浏览器';
  if (/iPhone|iPad/i.test(ua)) return '人格贩卖机 · iOS H5';
  if (/Android/i.test(ua)) return '人格贩卖机 · Android H5';
  return '人格贩卖机 · 网页端';
}

async function requestJson<T>(
  path: string,
  options: {
    method?: 'GET' | 'POST' | 'PATCH';
    body?: unknown;
    accessToken?: string;
  } = {}
): Promise<T> {
  const url = `${getAccountApiBase()}${path}`;
  const headers: Record<string, string> = {};
  if (options.body !== undefined) {
    headers['Content-Type'] = 'application/json';
  }
  if (options.accessToken) {
    headers.Authorization = `Bearer ${options.accessToken}`;
  }

  let res: Response;
  try {
    res = await fetch(url, {
      method: options.method || 'GET',
      headers,
      body: options.body !== undefined ? JSON.stringify(options.body) : undefined
    });
  } catch {
    throw new AccountApiError('网络连接失败，请检查网络后重试', 0, 'network_error');
  }

  let payload: Record<string, unknown> | null = null;
  try {
    payload = (await res.json()) as Record<string, unknown>;
  } catch {
    payload = null;
  }

  if (!res.ok) {
    const message =
      (payload && typeof payload.message === 'string' && payload.message) ||
      `请求失败（${res.status}）`;
    const code =
      (payload && typeof payload.code === 'string' && payload.code) || `http_${res.status}`;
    const retryFromBody =
      payload && typeof payload.retry_after === 'number' ? payload.retry_after : undefined;
    const retryFromHeader = Number.parseInt(res.headers.get('Retry-After') || '', 10);
    const retryAfter =
      retryFromBody ?? (Number.isFinite(retryFromHeader) ? retryFromHeader : undefined);
    throw new AccountApiError(message, res.status, code, retryAfter);
  }

  return payload as T;
}

export async function getAuthMethods(): Promise<SignupMethods> {
  try {
    const data = await requestJson<SignupMethods>('/auth/methods');
    return {
      email: Boolean(data?.email),
      phone: Boolean(data?.phone)
    };
  } catch {
    // 账号中心规范：接口不可用时默认只展示邮箱
    return { email: true, phone: false };
  }
}

export async function sendVerificationCode(
  identifier: string,
  purpose: 'login' | 'reset_password' = 'login'
): Promise<SendCodeResponse> {
  return requestJson<SendCodeResponse>('/auth/code/send', {
    method: 'POST',
    body: { identifier: identifier.trim(), purpose }
  });
}

export async function loginWithCode(
  identifier: string,
  code: string
): Promise<CodeLoginResponse> {
  return requestJson<CodeLoginResponse>('/auth/code/login', {
    method: 'POST',
    body: {
      identifier: identifier.trim(),
      code: code.trim(),
      client_id: getAccountClientId(),
      device_name: getDeviceLabel()
    }
  });
}

export async function registerAccount(
  registrationToken: string,
  username: string,
  password: string
): Promise<AuthSessionResponse> {
  return requestJson<AuthSessionResponse>('/auth/register', {
    method: 'POST',
    body: {
      registration_token: registrationToken,
      username: username.trim(),
      password,
      client_id: getAccountClientId(),
      device_name: getDeviceLabel()
    }
  });
}

export async function checkUsernameAvailable(username: string): Promise<UsernameCheckResponse> {
  const q = encodeURIComponent(username.trim());
  return requestJson<UsernameCheckResponse>(`/auth/username/available?username=${q}`);
}

export async function loginWithPassword(
  identifier: string,
  password: string
): Promise<AuthSessionResponse> {
  return requestJson<AuthSessionResponse>('/auth/password/login', {
    method: 'POST',
    body: {
      identifier: identifier.trim(),
      password,
      client_id: getAccountClientId(),
      device_name: getDeviceLabel()
    }
  });
}

export async function resetPassword(
  identifier: string,
  code: string,
  newPassword: string
): Promise<{ ok: boolean }> {
  return requestJson<{ ok: boolean }>('/auth/password/reset', {
    method: 'POST',
    body: {
      identifier: identifier.trim(),
      code: code.trim(),
      new_password: newPassword
    }
  });
}

export async function refreshSession(refreshToken: string): Promise<AuthTokens> {
  return requestJson<AuthTokens>('/auth/refresh', {
    method: 'POST',
    body: {
      refresh_token: refreshToken,
      client_id: getAccountClientId()
    }
  });
}

export async function getMe(accessToken: string): Promise<AccountUser> {
  return requestJson<AccountUser>('/me', { accessToken });
}

export async function changePassword(
  accessToken: string,
  currentPassword: string | undefined,
  newPassword: string
): Promise<{ ok: boolean }> {
  return requestJson<{ ok: boolean }>('/me/password', {
    method: 'POST',
    accessToken,
    body: {
      ...(currentPassword ? { current_password: currentPassword } : {}),
      new_password: newPassword
    }
  });
}

export async function logoutSession(refreshToken: string): Promise<void> {
  try {
    await requestJson<{ ok: boolean }>('/auth/logout', {
      method: 'POST',
      body: { refresh_token: refreshToken }
    });
  } catch {
    // 登出接口失败也不阻塞清理本机登录态
  }
}

/* ========== 与 server-account 一致的本地格式预检 ========== */

const RESERVED_EXACT = new Set([
  'support',
  'service',
  'moderator',
  'staff',
  'guest',
  'null',
  'undefined',
  'anonymous',
  'nobody',
  'everyone',
  'help',
  'mod',
  'owner',
  'bot',
  'api',
  'www',
  'mail',
  'email'
]);

const RESERVED_PREFIX = [
  'admin',
  'root',
  'system',
  'official',
  'xiaoban',
  'xiaban',
  'xiangban',
  '管理员',
  '客服',
  '官方',
  '系统',
  '站长',
  '版主',
  '小伴',
  '相伴',
  '下班了么'
];

function isHan(ch: string): boolean {
  const code = ch.charCodeAt(0);
  return code >= 0x4e00 && code <= 0x9fff;
}

function isAsciiLetter(ch: string): boolean {
  return /^[A-Za-z]$/.test(ch);
}

function isAsciiDigit(ch: string): boolean {
  return /^[0-9]$/.test(ch);
}

export function validateUsernameLocal(raw: string): string | null {
  const name = raw.trim();
  if (!name) return '请输入用户名';
  const chars = Array.from(name);
  if (!isAsciiLetter(chars[0]) && !isHan(chars[0])) {
    return '用户名需以字母或汉字开头';
  }
  for (const ch of chars) {
    if (!isAsciiLetter(ch) && !isAsciiDigit(ch) && ch !== '_' && !isHan(ch)) {
      return '用户名只能包含字母、数字、下划线和汉字';
    }
  }
  const han = chars.filter(isHan).length;
  const letters = chars.filter(isAsciiLetter).length;
  if ((han > 0 && han + letters < 2) || (han === 0 && letters < 3)) {
    return '用户名至少要有 2 个汉字或 3 个英文字母';
  }
  const width = chars.reduce((acc, ch) => acc + (isHan(ch) ? 2 : 1), 0);
  if (width > 20) {
    return '用户名最长 20 个字符（汉字算 2 个，最多 10 个汉字）';
  }
  const key = name.toLowerCase();
  if (RESERVED_EXACT.has(key) || RESERVED_PREFIX.some((p) => key.startsWith(p))) {
    return '这个用户名不能使用';
  }
  return null;
}

export function validatePasswordLocal(password: string, username?: string): string | null {
  const len = Array.from(password).length;
  if (len < 8 || len > 128) {
    return '密码长度需为 8-128 位';
  }
  if (username && password.toLowerCase() === username.trim().toLowerCase()) {
    return '密码不能和用户名相同';
  }
  return null;
}

export function formatPrimaryIdentity(user: AccountUser): string {
  const email = user.identities.find((i) => i.kind === 'email')?.masked;
  const phone = user.identities.find((i) => i.kind === 'phone')?.masked;
  return email || phone || `UID ${user.account_id}`;
}
