import { session, signOut } from './session';
import type { Anomaly, AuditRow, Batch, BindingDetail, BindingRow, CodeLookup, ModelOption, Overview, Product, Tier } from './types';

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

export type TierInput = Omit<Tier, 'product_id' | 'tier'>;
export type ProductInput = Pick<Product, 'name' | 'client_ids' | 'trial_quota' | 'trial_days' | 'enabled'>;

export const api = {
  login: (username: string, password: string) =>
    request<{ token: string; admin: { username: string } }>('POST', '/admin/login', { username, password }),
  changePassword: (current_password: string, new_password: string) =>
    request<{ ok: true }>('POST', '/admin/password', { current_password, new_password }),
  overview: () => request<Overview>('GET', '/admin/overview'),
  products: () => request<{ items: Product[] }>('GET', '/admin/products'),
  saveProduct: (id: string, body: ProductInput) => request<{ product: Product }>('PUT', `/admin/products/${id}`, body),
  tiers: (productId: string) => request<{ items: Tier[] }>('GET', `/admin/products/${productId}/tiers`),
  saveTier: (productId: string, tier: string, body: TierInput) =>
    request<{ ok: true; warnings: string[] }>('PUT', `/admin/products/${productId}/tiers/${tier}`, body),
  deleteTier: (productId: string, tier: string) => request<{ ok: true }>('DELETE', `/admin/products/${productId}/tiers/${tier}`),
  models: () => request<{ items: ModelOption[]; default_ratio: number }>('GET', '/admin/models'),
  bindings: (params: { q: string; productId: string; page: number }) => {
    const query = new URLSearchParams({ page: String(params.page), page_size: '20' });
    if (params.q) query.set('q', params.q);
    if (params.productId) query.set('product_id', params.productId);
    return request<{ items: BindingRow[]; total: number; page: number; page_size: number }>('GET', `/admin/bindings?${query}`);
  },
  binding: (id: number) => request<BindingDetail>('GET', `/admin/bindings/${id}`),
  grant: (id: number, body: { quota: number; kind: string; valid_days?: number; note: string }) =>
    request<{ ok: true }>('POST', `/admin/bindings/${id}/grant`, body),
  voidBucket: (id: number, note: string) => request<{ ok: true; voided: number }>('POST', `/admin/buckets/${id}/void`, { note }),
  anomalies: () => request<{ items: Anomaly[] }>('GET', '/admin/anomalies'),
  resolveAnomaly: (id: number) => request<{ ok: true }>('POST', `/admin/anomalies/${id}/resolve`),
  audit: () => request<{ items: AuditRow[] }>('GET', '/admin/audit?limit=100'),
  batches: () => request<{ items: Batch[] }>('GET', '/admin/batches'),
  createBatch: (body: { product_id: string; kind: string; quota: number; valid_days: number; count: number; note: string }) =>
    request<{ batch: Batch; codes: string[] }>('POST', '/admin/batches', body),
  voidBatch: (id: number) => request<{ ok: true; voided: number }>('POST', `/admin/batches/${id}/void`),
  lookupCode: (code: string) => request<{ code: CodeLookup }>('POST', '/admin/codes/lookup', { code }),
  voidCode: (id: number) => request<{ ok: true }>('POST', `/admin/codes/${id}/void`),
};

export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : '操作失败';
}
