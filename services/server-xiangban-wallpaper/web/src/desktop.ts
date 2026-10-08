export const targets = [
  { id: "windows-x86_64", label: "Windows 10 / 11 · 64 位" },
  { id: "darwin-aarch64", label: "macOS 12+ · Apple 芯片" },
  { id: "darwin-x86_64", label: "macOS · Intel" },
] as const;
export interface DesktopPackage {
  url: string;
  sha256?: string;
  bytes?: number;
}
export interface DesktopRelease {
  version: string;
  notes: string;
  publishedAt: string;
  published: boolean;
  platforms: Record<string, DesktopPackage>;
}
export interface Announcement {
  id: string;
  title: string;
  body: string;
  published: boolean;
  audience: string[];
  startsAt: string | null;
  endsAt: string | null;
}
export interface DesktopConfig {
  siteUrl: string;
  introduction: string;
  releases: DesktopRelease[];
  announcements: Announcement[];
}
export interface PublicNotice {
  id: string;
  title: string;
  body: string;
  revision: string;
}
export interface DesktopProduct {
  schemaVersion: number;
  introduction: string;
  downloads: Record<
    string,
    { version: string; notes: string; publishedAt: string; url: string }
  >;
  announcements: PublicNotice[];
}
export async function getDesktopProduct(): Promise<DesktopProduct> {
  const response = await fetch("/api/v1/desktop/product", {
    cache: "no-store",
  });
  if (!response.ok) throw new Error("小伴桌面信息加载失败，请稍后重试。");
  return response.json();
}
