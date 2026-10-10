export interface IdentityView {
  kind: string;
  masked: string;
}

export type UserStatus = 'active' | 'disabled';

export interface UserRow {
  uid: string;
  nickname: string;
  avatar_url: string | null;
  status: UserStatus;
  created_at: string;
  last_login_at: string | null;
  identities: IdentityView[];
}

export interface UserList {
  items: UserRow[];
  total: number;
  page: number;
  page_size: number;
}

export interface SessionRow {
  client_id: string;
  device_name: string | null;
  ip: string | null;
  created_at: string;
  expires_at: string;
}

export interface UserDetail {
  user: {
    uid: string;
    nickname: string;
    avatar_url: string | null;
    has_password: boolean;
    identities: IdentityView[];
    created_at: string;
  };
  status: UserStatus;
  last_login_at: string | null;
  sessions: SessionRow[];
}

export interface Stats {
  users: { total: number; active: number; disabled: number; new_24h: number; new_7d: number; logged_in_24h: number };
  active_sessions: number;
  codes_24h: {
    total: number;
    by_kind: { kind: string; count: number }[];
    top_ips: { ip: string; count: number }[];
  };
}

export interface ClientRow {
  client_id: string;
  name: string;
  enabled: boolean;
  created_at: string;
  active_sessions: number;
}

export interface AuditRow {
  admin: string;
  action: string;
  target: string | null;
  detail: string | null;
  created_at: string;
}
