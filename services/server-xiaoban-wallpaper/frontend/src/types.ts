export interface Resource {
  url: string;
  sha256: string;
  bytes: number;
  mime: string;
}
export interface Wallpaper {
  id: string;
  version: string;
  title: string;
  description: string;
  author: string;
  kind: "image" | "video" | "canvas";
  resource: Resource;
  cover?: Resource;
  assets: Record<string, Resource>;
  controls: unknown[];
}
export interface Entry {
  id: string;
  definition: Wallpaper;
  published: boolean;
  sortOrder: number;
  revision: number;
  verifiedAt: string | null;
  updatedAt: string;
}
export interface Release {
  schemaVersion: 1;
  version: string;
  notes: string;
  publishedAt?: string;
  platforms: Record<string, Resource>;
}
export interface ReleaseState {
  definition: Release;
  published: Release | null;
  revision: number;
}
export interface Site {
  name: string;
  catalog: { schemaVersion: 1; revision: string; items: Wallpaper[] };
  release: Release | null;
}
export interface Session {
  username: string;
  csrfToken: string;
  initialPassword: boolean;
}
