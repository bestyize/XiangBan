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

export interface ModelOption {
  model_name: string;
  model_ratio: number;
  completion_ratio: number;
  ratio_warning: boolean;
}

export interface Overview {
  bindings: number;
  active_bindings: number;
  funded_bindings: number;
  balance: number;
  consumed_24h: number;
  granted_24h: number;
  open_anomalies: number;
  quota_per_unit: number;
  newapi_configured: boolean;
  products: { product_id: string; name: string; bindings: number }[];
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
