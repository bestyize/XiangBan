import type { SpeechStore } from './speech/store.ts';
import type { AccountSession, AppInfo, CodeLoginOutcome, LoginResult, SendCodeResult, Settings, UsernameCheck, WallpaperLibrary, WallpaperOptions } from "../bridge.ts";
import type { EffectsChoice } from "../effects/types.ts";
import type { ImportOutcome, LoadedPacks } from "../effects/catalog.ts";
import type { WebsiteStatus } from "./website-types.ts";
import type { ChatStore } from "./chat/chat-store.ts";

export type ControlPage = "chat" | "wallpapers" | "widgets" | "interactive" | "effects" | "companion" | "games" | "care" | "settings";
export interface ControlStore {
  settings: Settings;
  info: AppInfo;
  library: WallpaperLibrary;
  website: WebsiteStatus;
  startup: { available: boolean; error: string | null };
  game: { active: boolean; desktopDoubleClickSupported: boolean; desktopEntryError: string | null };
  account: AccountSession | null;
  showLoginModal: boolean;
  page: ControlPage;
  /** Which tab of the settings page opens next; the chat sends people to "ai" when no model is set up. */
  settingsTab: "general" | "ai" | "voice" | "about";
  /** The assistant's conversations. Lives here so an answer keeps arriving while another page is open. */
  chat: ChatStore;
  speech: SpeechStore;
  ready: boolean;
  initialError: string;
  desktop: boolean;
  notify(message: unknown, type?: "success" | "error" | "info"): void;
  navigate(page: ControlPage): void;
  openLogin(): void;
  closeLogin(): void;
  sendAccountCode(identifier: string, purpose?: "login" | "reset_password"): Promise<SendCodeResult>;
  loginAccountWithCode(identifier: string, code: string): Promise<CodeLoginOutcome>;
  loginAccountWithPassword(identifier: string, password: string): Promise<LoginResult>;
  registerAccount(registrationToken: string, username: string, password: string): Promise<LoginResult>;
  resetAccountPassword(identifier: string, code: string, newPassword: string): Promise<void>;
  checkAccountUsername(username: string): Promise<UsernameCheck>;
  logoutAccount(): Promise<void>;
  initialize(): Promise<void>;
  dispose(): void;
  refreshSettings(): Promise<void>;
  refreshLibrary(): Promise<void>;
  updateAppearance(patch: Partial<Settings>): Promise<void>;
  setMode(mode: Settings["mode"]): Promise<void>;
  setCompanion(enabled: boolean): Promise<void>;
  setCompanionLinks(enabled: boolean): Promise<void>;
  setCompanionSkin(skin: string): Promise<void>;
  /** The effects that can be chosen: the built-in ones and the user's own files. */
  effectList: LoadedPacks;
  loadEffects(): Promise<void>;
  openEffectsDirectory(): Promise<void>;
  /** Offers a pack file to the effects folder; the pack's own id names the file it is saved as. */
  importEffectPack(): Promise<ImportOutcome>;
  deleteEffect(id: string): Promise<void>;
  /** Chooses the desktop-wide particle effect; it applies at once, whichever wallpaper is behind it. */
  setEffects(effects: EffectsChoice): Promise<void>;
  setAutostart(enabled: boolean): Promise<void>;
  setDoubleClick(enabled: boolean): Promise<void>;
  openGames(gameId?: string): Promise<void>;
  applyWallpaper(id: string, options: WallpaperOptions, preview?: boolean): Promise<void>;
  importMedia(): Promise<string | null>;
  deleteWallpaper(id: string): Promise<void>;
  setAutomaticUpdates(enabled: boolean): Promise<void>;
  checkWebsite(): Promise<void>;
}
