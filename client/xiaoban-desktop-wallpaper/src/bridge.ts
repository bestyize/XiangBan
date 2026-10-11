import type { EffectsChoice } from "./effects/types.ts";

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
  websiteUrl: string;
  automaticUpdates: boolean;
  gameDesktopDoubleClick: boolean;
  companionEnabled: boolean;
  /** Lets the companion answer the desktop effect with expressions only; no popups or sound. */
  companionLinks: boolean;
  /** Which official shell the companion wears: cat, bear or bunny. */
  companionSkin: string;
  /** The particle effect over the desktop, whatever wallpaper is behind it. */
  effects: EffectsChoice;
}

export interface AccountUser {
  uid: string;
  /** Unique and unchangeable; also the name shown everywhere. */
  username: string;
  avatarUrl?: string | null;
}

export interface AccountSession {
  accessToken: string;
  refreshToken: string;
  expiresAt: number;
  user: AccountUser;
}

export interface SendCodeResult {
  ok: boolean;
  cooldownSeconds: number;
  expiresIn: number;
}

export interface LoginResult {
  isNewUser: boolean;
  session: AccountSession;
}

/** A code login either signs an existing account in, or says the account still has to be created. */
export type CodeLoginOutcome =
  | { kind: "session"; isNewUser: boolean; session: AccountSession }
  | { kind: "registration"; registrationToken: string };

export interface UsernameCheck {
  available: boolean;
  message?: string | null;
}

/** The account center's error: `code` is machine readable (username_taken, registration_expired, identity_exists...). */
export class AccountError extends Error {
  constructor(message: string, readonly code?: string | null) {
    super(message);
  }
}

/** Tauri rejects with whatever the command returned: an `{ message, code }` object, a plain string, or an Error. */
export function toAccountError(error: unknown): AccountError {
  if (error instanceof AccountError) return error;
  if (error && typeof error === "object") {
    const { message, code } = error as { message?: unknown; code?: unknown };
    return new AccountError(typeof message === "string" && message ? message : "操作失败，请稍后再试", typeof code === "string" ? code : null);
  }
  return new AccountError(typeof error === "string" && error ? error : "操作失败，请稍后再试");
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
  previewUrl?: string | null;
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
  uiSmoke?: boolean;
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
  updateUrl: "https://wallpaper.xiangban.xyz/api/v1/desktop/updates",
  websiteUrl: "https://wallpaper.xiangban.xyz",
  automaticUpdates: true,
  gameDesktopDoubleClick: false,
  companionEnabled: true,
  companionLinks: true,
  companionSkin: "cat",
  effects: { selected: "", params: {} },
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
