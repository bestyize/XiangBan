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
  /** Which official shell the companion wears: pot, cat, bunny or nezha. */
  companionSkin: string;
  /** The floating assistant: a capsule that a global shortcut brings up for a quick spoken or typed question. */
  assistantEnabled: boolean;
  assistantShortcut: string;
  /** The particle effect over the desktop, whatever wallpaper is behind it. */
  effects: EffectsChoice;
}

/** `chat` is /chat/completions, `responses` is /responses and `anthropic` is /v1/messages. */
export type AiProtocol = "chat" | "responses" | "anthropic";

export type ProviderType = "third" | "free" | "paid" | "custom";
export interface ProviderConfigView {
  type?: ProviderType;
  remote?: boolean; available?: boolean; description?: string;
  defaultModel?: string;
  models?: { id: string; name: string; description?: string }[];
  endpoints?: Partial<Record<AiProtocol, string>>;
  consoleUrl?: string; consoleText?: string;
  selectionMode?: "manual" | "recommended";
  id: string;
  name: string;
  protocol: AiProtocol;
  baseUrl: string;
  model: string;
  hasKey: boolean;
  keyHint: string;
  isCustom: boolean;
}

/** What the host tells the web side about the model service. The key itself is never sent back. */
export interface AiConfigView {
  catalogueReceived?: boolean;
  catalogueFetchedAt?: number;
  activeProvider?: string;
  providers?: ProviderConfigView[];
  protocol: AiProtocol;
  baseUrl: string;
  model: string;
  hasKey: boolean;
  /** Dots, plus the last four characters of a long key. Empty when there is no key. */
  keyHint: string;
  /** The folder the assistant's file and command tools work in; empty means the built-in one. */
  workspace: string;
  defaultWorkspace: string;
  /** Whether intermediate tool call cards are hidden in chat. */
  hideToolCalls: boolean;
  /** Whether open-computer-use MCP tools are enabled. */
  openComputerUse: boolean;
  /** The 小伴 AI tier picked in settings; only in use while `activeProvider` is the gateway. */
  gatewayTier?: string;
  /** False once the site has closed the 小伴 AI entry; missing means on. */
  gatewayEnabled?: boolean;
}

/** `activeProvider` while chat is going through 小伴 AI instead of one of the listed providers. */
export const GATEWAY_PROVIDER = "xiaoban-ai";

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

export interface SignupMethods {
  email: boolean;
  phone: boolean;
}

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

/** One pot of AI credit. `kind` is trial, monthly or topup; amounts are in gateway points. */
export interface AiBucket {
  id: number;
  kind: string;
  total: number;
  remaining: number;
  expiresAt: string | null;
  createdAt: string;
}

export interface AiBalance {
  product: { name: string };
  /** Points that make one US dollar. */
  quotaPerUnit: number;
  total: number;
  paid: number;
  buckets: AiBucket[];
  trial: { available: boolean; quota: number; days: number };
}

/** A model level the gateway offers. The real model behind it is never sent to the client. */
export interface AiTier {
  tier: string;
  displayName: string;
  description: string;
  isDefault: boolean;
  requiresPaid: boolean;
  /** False while the account has no paid credit for a paid tier. */
  available: boolean;
}

export interface AiRedeemed {
  kind: string;
  quota: number;
  expiresAt: string | null;
  balance: number;
}

export type WallpaperKind = "image" | "video" | "canvas" | "web";
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
  companionSkin: "pot",
  assistantEnabled: true,
  assistantShortcut: "Alt+Space",
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
