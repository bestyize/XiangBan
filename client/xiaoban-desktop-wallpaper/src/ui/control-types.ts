import type { AppInfo, Settings, WallpaperLibrary, WallpaperOptions } from "../bridge.ts";
import type { WebsiteStatus } from "./website-types.ts";

export type ControlPage = "wallpapers" | "companion" | "games" | "care" | "settings";
export interface ControlStore {
  settings: Settings;
  info: AppInfo;
  library: WallpaperLibrary;
  website: WebsiteStatus;
  startup: { available: boolean; error: string | null };
  game: { active: boolean; desktopDoubleClickSupported: boolean; desktopEntryError: string | null };
  page: ControlPage;
  ready: boolean;
  initialError: string;
  desktop: boolean;
  notify(message: unknown, type?: "success" | "error" | "info"): void;
  navigate(page: ControlPage): void;
  initialize(): Promise<void>;
  dispose(): void;
  refreshSettings(): Promise<void>;
  refreshLibrary(): Promise<void>;
  updateAppearance(patch: Partial<Settings>): Promise<void>;
  setMode(mode: Settings["mode"]): Promise<void>;
  setCompanion(enabled: boolean): Promise<void>;
  setAutostart(enabled: boolean): Promise<void>;
  setDoubleClick(enabled: boolean): Promise<void>;
  openGames(): Promise<void>;
  applyWallpaper(id: string, options: WallpaperOptions, preview?: boolean): Promise<void>;
  importMedia(): Promise<string | null>;
  configureWebsite(url: string, automatic: boolean): Promise<void>;
  checkWebsite(): Promise<void>;
}
