import type { AppInfo, Settings, WallpaperLibrary, WallpaperOptions } from "../bridge.ts";
import type { EffectsChoice } from "../effects/types.ts";
import type { LoadedPacks } from "../effects/catalog.ts";
import type { WebsiteStatus } from "./website-types.ts";

export type ControlPage = "wallpapers" | "effects" | "companion" | "games" | "care" | "settings";
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
  setCompanionLinks(enabled: boolean): Promise<void>;
  /** The effects that can be chosen: the built-in ones and the user's own files. */
  effectList: LoadedPacks;
  loadEffects(): Promise<void>;
  openEffectsDirectory(): Promise<void>;
  /** Chooses the desktop-wide particle effect; it applies at once, whichever wallpaper is behind it. */
  setEffects(effects: EffectsChoice): Promise<void>;
  setAutostart(enabled: boolean): Promise<void>;
  setDoubleClick(enabled: boolean): Promise<void>;
  openGames(gameId?: string): Promise<void>;
  applyWallpaper(id: string, options: WallpaperOptions, preview?: boolean): Promise<void>;
  importMedia(): Promise<string | null>;
  configureWebsite(url: string, automatic: boolean): Promise<void>;
  checkWebsite(): Promise<void>;
}
