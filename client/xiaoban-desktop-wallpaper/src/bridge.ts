export interface Settings {
  mode: "floating" | "wallpaper";
  expression: string;
  size: number;
  theme: "night" | "dawn";
  paused: boolean;
  fps: 30 | 60;
  autostart: boolean;
  wallpaperId: string;
  wallpaperOptions: WallpaperOptions;
  catalogUrl: string;
  updateUrl: string;
}

export type WallpaperKind = "image" | "video" | "canvas";
export interface Resource {
  url: string;
  sha256: string;
  bytes: number;
  mime: string;
}
export interface WallpaperControl {
  key: string;
  label: string;
  type: "range" | "color" | "boolean" | "select";
  default: number | string | boolean;
  min?: number;
  max?: number;
  step?: number;
  options?: string[];
}
export interface WallpaperDefinition {
  id: string;
  version: string;
  title: string;
  description?: string;
  author?: string;
  kind: WallpaperKind;
  resource: Resource;
  cover?: Resource;
  assets?: Record<string, Resource>;
  controls?: WallpaperControl[];
}
export interface WallpaperEntry {
  definition: WallpaperDefinition;
  installed: boolean;
  coverUrl: string | null;
  source?: "builtin" | "subscription" | "local";
}
export interface WallpaperOptions {
  fit: "cover" | "contain" | "fill";
  playbackRate: number;
  background: string;
  values: Record<string, string | number | boolean>;
}
export interface WallpaperLibrary {
  items: WallpaperEntry[];
  source: string;
  activeId: string;
}
export interface AppInfo {
  name: string;
  version: string;
  platform: string;
  arch: string;
  updateTarget: string;
  updateUrl: string;
  dataDirectory?: string;
}
export const wallpaperDefaults: WallpaperOptions = {
  fit: "cover",
  playbackRate: 1,
  background: "#edf5f0",
  values: {},
};

export interface CursorSample {
  x: number;
  y: number;
  moved: boolean;
  pressed: boolean;
  released: boolean;
  desktop: boolean;
}

declare global {
  interface Window {
    __TAURI__?: {
      core: {
        invoke: <T>(
          command: string,
          args?: Record<string, unknown>,
        ) => Promise<T>;
      };
      event: {
        listen: <T>(
          event: string,
          callback: (event: { payload: T }) => void,
        ) => Promise<() => void>;
      };
    };
  }
}

export const desktop = Boolean(window.__TAURI__);
export const defaults: Settings = {
  mode: "floating",
  expression: "idle",
  size: 260,
  theme: "night",
  paused: false,
  fps: 60,
  autostart: true,
  wallpaperId: "companion",
  wallpaperOptions: wallpaperDefaults,
  catalogUrl: "",
  updateUrl: "",
};
export function invoke<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (!window.__TAURI__)
    return Promise.reject(new Error("请通过桌面程序运行这个功能"));
  return window.__TAURI__.core.invoke<T>(command, args);
}
export async function listen<T>(
  event: string,
  callback: (payload: T) => void,
): Promise<() => void> {
  return (
    window.__TAURI__?.event.listen<T>(event, ({ payload }) =>
      callback(payload),
    ) ?? (() => {})
  );
}
