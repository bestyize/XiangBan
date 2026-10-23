import type { SpeechStore } from './speech/store.ts';
import type { AsrStore } from './asr/store.ts';
import type { AccountSession, AppInfo, CodeLoginOutcome, LoginResult, SendCodeResult, Settings, SignupMethods, UsernameCheck, WallpaperLibrary, WallpaperOptions } from "../bridge.ts";
import type { EffectsChoice } from "../effects/types.ts";
import type { ImportOutcome, LoadedPacks } from "../effects/catalog.ts";
import type { WebsiteStatus } from "./website-types.ts";
import type { ChatStore } from "./chat/chat-store.ts";
import type { GameCatalog } from "../games/catalog.ts";

export type { GameCatalog };

/** The pages the shell shows (方案四). */
export type MainPage = "new-task" | "chat" | "ai-store" | "library" | "studio" | "games" | "care" | "interactive";
/**
 * Older ids that still work with `navigate`: the four studio kinds open 装扮 on that tab,
 * "settings" / "assistant" open the settings dialog instead of a page.
 */
export type LegacyPage = "wallpapers" | "effects" | "widgets" | "companion" | "settings" | "assistant";
export type ControlPage = MainPage | LegacyPage;
/** The tabs of 装扮. */
export type StudioTab = "wallpapers" | "effects" | "widgets" | "pets";
/** The categories of the settings dialog. */
export type SettingsTab = "general" | "appearance" | "ai" | "assistant" | "voice" | "safety" | "about";
export interface ControlStore {
  settings: Settings;
  info: AppInfo;
  library: WallpaperLibrary;
  website: WebsiteStatus;
  startup: { available: boolean; error: string | null };
  game: { active: boolean; desktopDoubleClickSupported: boolean; desktopEntryError: string | null; /** The desktop-layer game that is running, if any. */ desk?: { active: boolean; id: string | null } };
  account: AccountSession | null;
  showLoginModal: boolean;
  page: ControlPage;
  creationDraft: string | null;
  /** Files handed to the next message along with `creationDraft` (资料库 → 用这份资料提问). */
  creationAttachments: import('./chat/attachments.ts').PendingAttachment[] | null;
  startCreation(kind: "wallpaper" | "effect" | "widget" | "game" | "pet"): Promise<void>;
  /** The settings dialog's category; the chat sends people to "ai" when no model is set up. */
  settingsTab: SettingsTab;
  /** Whether the settings dialog is open. Use `openSettings` / `closeSettings`, or `navigate('settings')`. */
  settingsOpen: boolean;
  /** Opens the settings dialog, on `tab` when given (otherwise on the last category). */
  openSettings(tab?: SettingsTab): void;
  closeSettings(): void;
  /** The tab 装扮 shows. `navigate('wallpapers' | 'effects' | 'widgets' | 'companion')` sets it. */
  studioTab: StudioTab;
  /** The assistant's conversations. Lives here so an answer keeps arriving while another page is open. */
  chat: ChatStore;
  speech: SpeechStore;
  asr: AsrStore;
  ready: boolean;
  initialError: string;
  desktop: boolean;
  notify(message: unknown, type?: "success" | "error" | "info"): void;
  /** Goes to a page. Legacy ids are mapped (see `ControlPage`); "settings"/"assistant" open the dialog and leave the page as it is. */
  navigate(page: ControlPage): void;
  openLogin(): void;
  closeLogin(): void;
  sendAccountCode(identifier: string, purpose?: "login" | "reset_password"): Promise<SendCodeResult>;
  loginAccountWithCode(identifier: string, code: string): Promise<CodeLoginOutcome>;
  loginAccountWithPassword(identifier: string, password: string): Promise<LoginResult>;
  registerAccount(registrationToken: string, username: string, password: string): Promise<LoginResult>;
  resetAccountPassword(identifier: string, code: string, newPassword: string): Promise<void>;
  checkAccountUsername(username: string): Promise<UsernameCheck>;
  getSignupMethods(): Promise<SignupMethods>;
  logoutAccount(): Promise<void>;
  initialize(): Promise<void>;
  dispose(): void;
  refreshSettings(): Promise<void>;
  refreshLibrary(): Promise<void>;
  updateAppearance(patch: Partial<Settings>): Promise<void>;
  setMode(mode: Settings["mode"]): Promise<void>;
  setAppearance(mode: Settings["appearance"]): Promise<void>;
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
  /** Stops the desktop-layer game that is running. */
  stopDeskGame(): Promise<void>;
  /** Shipped games plus the user's own, and the user folders that could not be loaded. */
  listGames(): Promise<GameCatalog>;
  /** Removes a user's own game folder (and closes the game space first if it is open). */
  deleteUserGame(id: string): Promise<void>;
  openUserGamesFolder(): Promise<void>;
  applyWallpaper(id: string, options: WallpaperOptions, preview?: boolean): Promise<void>;
  importMedia(): Promise<string | null>;
  deleteWallpaper(id: string): Promise<void>;
  setAutomaticUpdates(enabled: boolean): Promise<void>;
  checkWebsite(): Promise<void>;
}


