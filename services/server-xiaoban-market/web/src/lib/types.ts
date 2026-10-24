export type Kind = 'skill' | 'expert' | 'connector';
export type Plural = 'skills' | 'experts' | 'connectors';
export type ItemStatus = 'draft' | 'review' | 'published' | 'offline';
export type SourceLabel = 'official' | 'community' | 'enterprise';

export interface CategoryRef {
  slug: string;
  name: string;
}

export interface Item {
  id: string;
  kind: Kind;
  namespace: string;
  slug: string;
  name: string;
  summary: string;
  category: CategoryRef | null;
  tags: string[];
  iconUrl: string | null;
  source: SourceLabel;
  origin: string;
  version: string | null;
  downloads: number;
  favorites: number;
  heat: number;
  featured: boolean;
  updatedAt: string;
}

export interface ListResponse {
  results: Item[];
  near: Item[];
  page: number;
  pageSize: number;
  total: number;
  hasMore: boolean;
}

export interface FileEntry {
  path: string;
  size: number;
  sha256: string;
}

export interface VersionInfo {
  version: string;
  changelog: string;
  size: number;
  yanked: boolean;
  createdAt: string;
  manifestSha256: string;
  packageSha256: string;
  downloadUrl: string | null;
}

export interface ItemDetail {
  item: Item & {
    description: string;
    originUrl: string | null;
    license: string | null;
    prompts: string[];
    metadata: { configs?: { name: string; config: unknown }[]; dependencies?: { slug: string; namespace?: string; version?: string }[] };
    publishedAt: string | null;
  };
  versions: VersionInfo[];
  files: FileEntry[];
}

export interface CategoryCount extends CategoryRef {
  count: number;
}

// ───────── 后台 ─────────

export interface JobStats {
  pages?: number;
  fetched?: number;
  new?: number;
  changed?: number;
  unchanged?: number;
  skipped?: number;
  failed?: number;
  imported?: number;
  import_failed?: number;
}

export interface Job {
  id: string;
  sourceId: number;
  sourceName: string;
  status: 'queued' | 'running' | 'succeeded' | 'failed' | 'cancelled';
  trigger: string;
  params: Record<string, unknown>;
  stats: JobStats;
  error: string | null;
  createdBy: string | null;
  createdAt: string;
  startedAt: string | null;
  finishedAt: string | null;
  total?: number;
}

export interface JobLog {
  id: number;
  ts: string;
  level: 'info' | 'warn' | 'error';
  message: string;
}

export interface Source {
  id: number;
  key: string;
  name: string;
  kind: Kind;
  adapter: 'skillhub' | 'generic_json';
  enabled: boolean;
  config: Record<string, any>;
  intervalMinutes: number;
  autoImport: 'none' | 'draft' | 'published';
  lastJobAt: string | null;
  configured: boolean;
  pending: number;
  lastJob: { id: string; status: Job['status']; stats: JobStats; error: string | null; createdAt: string; finishedAt: string | null } | null;
}

export interface Normalized {
  upstream_id: string;
  kind: Kind;
  namespace: string;
  slug: string;
  name: string;
  summary: string;
  description: string;
  category: string | null;
  tags: string[];
  icon_url: string | null;
  license: string | null;
  origin_url: string | null;
  source: SourceLabel;
  version: string | null;
  metadata: Record<string, unknown>;
}

export interface ProbeResult {
  ok: boolean;
  error?: string;
  count?: number;
  total?: number | null;
  hasMore?: boolean;
  skipped?: number;
  entries?: { raw: unknown; normalized: Normalized }[];
  mapping?: { field: string; rule: unknown; hit: boolean; sample: string | null }[];
  response?: unknown;
}

export interface StagedRow {
  id: string;
  sourceId: number;
  sourceName: string;
  upstreamId: string;
  kind: Kind;
  name: string;
  summary: string;
  category: string | null;
  status: 'pending' | 'imported' | 'ignored';
  itemId: string | null;
  importError: string | null;
  lastSeenAt: string;
  slug: string | null;
  version: string | null;
  change: 'new' | 'changed' | 'failed' | null;
}

export interface StagedDetail {
  id: string;
  sourceName: string;
  kind: Kind;
  status: StagedRow['status'];
  raw: unknown;
  normalized: Normalized;
  importError: string | null;
  itemId: string | null;
  existing: { id: string; name: string; summary: string; tags: string[]; version: string | null; status: ItemStatus; categoryId: number | null } | null;
}

export interface AdminItem {
  id: string;
  kind: Kind;
  namespace: string;
  slug: string;
  name: string;
  summary: string;
  status: ItemStatus;
  source: SourceLabel;
  origin: string;
  featured: boolean;
  version: string | null;
  downloads: number;
  favorites: number;
  tags: string[];
  iconUrl: string | null;
  categoryId: number | null;
  categoryName: string | null;
  updatedAt: string;
  publishedAt: string | null;
  description?: string;
  license?: string | null;
  originUrl?: string | null;
  prompts?: string[];
  metadata?: Record<string, unknown>;
}

export interface AdminVersion {
  id: string;
  version: string;
  changelog: string;
  size: number;
  manifestSha256: string;
  packageSha256: string;
  yanked: boolean;
  createdAt: string;
  fileCount: number;
}

export interface AdminCategory {
  id: number;
  kind: Kind;
  slug: string;
  name: string;
  nameEn: string;
  sortOrder: number;
  active: boolean;
  items: number;
}

export interface Stats {
  pendingStaged: number;
  upstreamChanged: number;
  importFailed: number;
  failedJobs24h: number;
  publishedThisWeek: Partial<Record<Kind, number>>;
  itemsByStatus: Partial<Record<ItemStatus, number>>;
}
