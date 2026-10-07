export interface UserDto {
  id: number;
  username: string;
  is_admin: boolean;
  created_at: number;
}

export interface FileItem {
  id: string;
  user_id: number;
  parent_id: string | null;
  name: string;
  size: number;
  mime_type: string;
  is_dir: boolean;
  storage_path: string | null;
  created_at: number;
  updated_at: number;
}

export interface BreadcrumbItem {
  id: string | null;
  name: string;
}

export interface FileListResponse {
  current_folder: FileItem | null;
  breadcrumbs: BreadcrumbItem[];
  items: FileItem[];
}

export interface StorageStats {
  total_files: number;
  total_folders: number;
  used_bytes: number;
  total_shares: number;
}

export interface ShareDto {
  code: string;
  file_id: string;
  file_name: string;
  is_dir: boolean;
  file_size: number;
  has_password: boolean;
  password: string | null;
  note?: string | null;
  expires_at: number | null;
  is_expired: boolean;
  view_count: number;
  download_count: number;
  created_at: number;
}

export interface UpdateShareRequest {
  note?: string | null;
  password?: string | null;
  expire_days?: number | null;
}

export interface ShareInfoResponse {
  code: string;
  target_name: string;
  is_dir: boolean;
  file_size: number;
  has_password: boolean;
  note?: string | null;
  is_expired: boolean;
  expires_at: number | null;
  creator_name: string;
  created_at: number;
}

export interface ShareContentResponse {
  target_item: FileItem;
  current_folder: FileItem | null;
  breadcrumbs: BreadcrumbItem[];
  items: FileItem[];
}
