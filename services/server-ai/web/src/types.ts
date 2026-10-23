export interface Product {
  product_id: string;
  name: string;
  client_ids: string[];
  trial_quota: number;
  trial_days: number;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface Tier {
  reasoning_efforts?: string[] | null;
  product_id: string;
  tier: string;
  display_name: string;
  description: string;
  model: string;
  requires_paid: boolean;
  is_default: boolean;
  sort_order: number;
  enabled: boolean;
}

export type Capability = 'speech' | 'image' | 'transcribe' | 'realtime';

export interface MediaSetting {
  product_id: string;
  capability: Capability;
  model: string;
  enabled: boolean;
  requires_paid: boolean;
  voices: string[];
  sizes: string[];
  max_secs: number;
  updated_at: string;
}

export interface ModelOption {
  model_name: string;
  model_ratio: number;
  completion_ratio: number;
  ratio_warning: boolean;
}

export interface DayPoint {
  day: string;
  consumed: number;
  granted: number;
}

export interface FlowSegment {
  key: string;
  label: string;
  points: number;
  tone: 'used' | 'lapsed' | 'left';
  note: string;
}

export interface ProductOverview {
  product_id: string;
  name: string;
  bindings: number;
  balance: number;
  consumed_7d: number;
}

export interface Overview {
  bindings: number;
  active_bindings: number;
  funded_bindings: number;
  active_24h: number;
  balance: number;
  consumed_24h: number;
  granted_24h: number;
  granted_total: number;
  consumed_total: number;
  lapsed_total: number;
  series: DayPoint[];
  open_anomalies: number;
  quota_per_unit: number;
  newapi_configured: boolean;
  products: ProductOverview[];
}

export interface BindingRow {
  id: number;
  uid: string;
  product_id: string;
  status: string;
  balance: number;
  synced_quota: number;
  created_at: string;
  last_used_at: string | null;
  last_synced_at: string | null;
}

export interface Bucket {
  id: number;
  kind: string;
  total: number;
  remaining: number;
  expires_at: string | null;
  source: string;
  created_at: string;
  closed_at: string | null;
  closed_reason: string | null;
}

export interface LedgerRow {
  id: number;
  bucket_id: number | null;
  kind: string;
  delta: number;
  balance_after: number;
  note: string;
  created_at: string;
}

export interface Anomaly {
  id: number;
  binding_id: number;
  kind: string;
  detail: string;
  created_at: string;
  resolved_at: string | null;
}

export interface BindingDetail {
  binding: BindingRow;
  buckets: Bucket[];
  ledger: LedgerRow[];
  anomalies: Anomaly[];
}

export interface AuditRow {
  admin: string;
  action: string;
  target: string | null;
  detail: string | null;
  created_at: string;
}

export interface Batch {
  id: number;
  product_id: string;
  kind: 'topup' | 'monthly';
  quota: number;
  valid_days: number;
  note: string;
  created_by: string;
  created_at: string;
  total: number;
  unused: number;
  redeemed: number;
  void: number;
}

export interface CodeLookup {
  id: number;
  status: 'unused' | 'redeemed' | 'void';
  batch_id: number;
  product_id: string;
  kind: 'topup' | 'monthly';
  quota: number;
  valid_days: number;
  note: string;
  redeemed_by: string | null;
  redeemed_at: string | null;
  voided_at: string | null;
  binding_id: number | null;
  bucket_id: number | null;
  bucket_remaining: number | null;
  bucket_expires_at: string | null;
  bucket_closed_reason: string | null;
}

export interface GatewaySettings {
  base_url: string;
  configured: boolean;
  source: 'console' | 'env' | 'none';
  hint: string | null;
  updated_at: string | null;
  problem: string | null;
}
