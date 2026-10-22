import { createSpeechStore } from './speech/store.ts';
import { createAsrStore } from './asr/store.ts';
import { reactive } from "vue";
import { desktop, defaults, invoke, listen, toAccountError, type AiConfigView, type Settings, type WallpaperEntry, type WallpaperLibrary, type AppInfo, type AccountSession, type SendCodeResult, type CodeLoginOutcome, type UsernameCheck, type SignupMethods, type LoginResult } from "../bridge.ts";
import type { WebsiteStatus } from "./website-types.ts";
import { loadBuiltinCatalog, readCatalog } from "../games/catalog.ts";
import { loadBuiltinPacks, noPacks, readImported, readList } from "../effects/catalog.ts";
import type { ControlPage, ControlStore, StudioTab } from "./control-types.ts";
import { createChatStore } from "./chat/chat-store.ts";

export const clone = <T>(value: T): T => JSON.parse(JSON.stringify(value));

/** Account commands reject with `{ message, code }`; the UI needs both, so turn it into an AccountError. */
async function accountInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw toAccountError(error);
  }
}

/** Browser preview only: a signed-in session without any network. */
function mockLogin(username: string, isNewUser: boolean): LoginResult {
  return {
    isNewUser,
    session: {
      accessToken: "mock.access.token",
      refreshToken: "mock.refresh.token",
      expiresAt: Math.floor(Date.now() / 1000) + 1800,
      user: { uid: "mock-uid-1234", username },
    },
  };
}
/** Where the older page ids went: the studio kinds are tabs of 装扮 now. */
const studioTabs: Partial<Record<ControlPage, StudioTab>> = { wallpapers: "wallpapers", effects: "effects", widgets: "widgets", companion: "pets" };
/** The new-task page and an open conversation share the composer, so going between them is not leaving the chat. */
const chatLike = (page: ControlPage) => page === "chat" || page === "new-task";
export function createControlStore(notify: ControlStore["notify"]): ControlStore {
  let disposed = false;
  const unsubscribers: (() => void)[] = [];
  // Serialize partial appearance updates so a fast sequence cannot overwrite another setting.
  let appearanceQueue = Promise.resolve();
  // Same for the effect choice: sliders send many updates in a row, and only the last may win.
  let effectsQueue = Promise.resolve();
  const store: ControlStore = reactive({
    settings: clone(defaults),
    info: { name: "小伴桌面", version: "2.1.0", platform: "preview", arch: "", updateTarget: "", updateUrl: "" },
    library: { items: [], source: "", activeId: "companion" },
    website: { announcements: [], update: null, dismissed: {}, errors: [] },
    startup: { available: false, error: null },
    game: { active: false, desktopDoubleClickSupported: false, desktopEntryError: null },
    effectList: noPacks(),
    account: null,
    showLoginModal: false,
    page: "new-task", creationDraft: null, settingsTab: "general", settingsOpen: false, studioTab: "wallpapers", asr: createAsrStore({ desktop, invoke, listen, notify }), speech: createSpeechStore({ desktop, invoke, notify }), chat: createChatStore({ desktop, invoke, listen, notify,
      onActivity: () => { store.asr.deactivate(); void store.speech.stop(); },
      onReply: (session, text, key) => { if (chatLike(store.page) && store.speech.settings.autoRead && document.visibilityState !== 'hidden') void store.speech.speak(text, key ?? 'auto:' + session, undefined, true); },
    }), ready: false, initialError: "", desktop, notify,
    navigate(page) {
      if (page === 'assistant' || page === 'settings') { store.openSettings(page === 'assistant' ? 'assistant' : undefined); return; }
      const tab = studioTabs[page];
      if (tab) { store.studioTab = tab; page = 'studio'; }
      if (store.page !== page && !(chatLike(store.page) && chatLike(page))) { void store.speech.stop(); store.asr.deactivate(); }
      store.page = page;
    },
    openSettings(tab) { if (tab) store.settingsTab = tab; store.settingsOpen = true; },
    closeSettings() { store.settingsOpen = false; },
    async startCreation(kind) {
      const prompts = {
        wallpaper: '帮我创作一张动态壁纸：',
        effect: '帮我创作一个桌面特效：',
        widget: '帮我创作一个桌面小组件：',
        game: '帮我创作一款小游戏：',
        pet: '帮我创作一个桌面宠物主题。先读取 pet_read_format：',
      };
      // With nothing open the draft goes to 新建任务; with a conversation open it starts a new one on the chat page as before.
      const hadConversation = !!store.chat.currentId;
      await store.chat.select(null);
      store.creationDraft = prompts[kind];
      store.navigate(hadConversation ? 'chat' : 'new-task');
    },
    openLogin() { store.showLoginModal = true; },
    closeLogin() { store.showLoginModal = false; },
    async initialize() {
      void store.speech.init();
      void store.asr.init();
      store.initialError = "";
      store.ready = false;
      unsubscribers.splice(0).forEach((unlisten) => unlisten());
      try {
        if (!desktop) {
          const response = await fetch("/wallpapers/builtin/catalog.json");
          if (!response.ok) throw new Error("内置壁纸目录加载失败");
          const catalog = await response.json();
          const asset = (url?: string) => url?.replace("builtin://", "/wallpapers/builtin/") ?? "";
          store.library.items = catalog.items.map((definition: WallpaperEntry["definition"]) => ({
            definition, installed: true, source: "builtin", coverUrl: asset(definition.cover?.url) || (definition.kind === "image" ? asset(definition.resource.url) : null),
            previewUrl: definition.kind === "canvas" ? undefined : asset(definition.resource.url),
          }));
          store.settings.autostart = false;
          return;
        }
        const subscriptions = await Promise.all([
          listen<AiConfigView>("ai-config-changed", () => { if (!disposed) void store.chat.refresh(); }),
          listen<Settings>("settings-changed", (settings) => { if (!disposed) store.settings = settings; }),
          listen<WebsiteStatus>("website-status", (website) => { if (!disposed) store.website = website; }),
          listen<AccountSession | null>("account-changed", (account) => { if (!disposed) { store.account = account; void invoke('refresh_ai_providers', { force: true }).catch(() => {}); } }),
          listen<boolean>("games-active", (active) => { if (!disposed) store.game.active = active; }),
          listen<string | null>("desk-game-active", (id) => { if (!disposed) store.game.desk = { active: !!id, id: id ?? null }; }),
          listen("care-voice-start", () => { if (!disposed) void store.speech.stop(); }),
          listen<string>("desktop-error", (error) => notify(error, "error")),
        ]);
        if (disposed) { subscriptions.forEach((unlisten) => unlisten()); return; }
        unsubscribers.push(...subscriptions);
        const initial = await Promise.allSettled([
          invoke<Settings>("get_settings"), invoke<AppInfo>("get_app_info"), invoke<WallpaperLibrary>("get_wallpaper_library"),
          invoke<AccountSession | null>("get_account_session"),
        ]);
        if (disposed) return;
        if (initial[0].status === "rejected") throw initial[0].reason;
        store.settings = initial[0].value;
        if (initial[1].status === "fulfilled") store.info = initial[1].value;
        else notify(`应用信息读取失败：${initial[1].reason}`, "error");
        if (initial[2].status === "fulfilled") store.library = initial[2].value;
        else notify(`本地壁纸目录读取失败：${initial[2].reason}`, "error");
        if (initial[3].status === "fulfilled") store.account = initial[3].value;
        const secondary = await Promise.allSettled([
          invoke<ControlStore["startup"]>("get_startup_status"), invoke<ControlStore["game"]>("get_game_status"), invoke<WebsiteStatus>("get_website_status"),
        ]);
        if (disposed) return;
        if (secondary[0].status === "fulfilled") store.startup = secondary[0].value;
        else store.startup.error = String(secondary[0].reason);
        if (secondary[1].status === "fulfilled") store.game = secondary[1].value;
        else store.game.desktopEntryError = String(secondary[1].reason);
        if (secondary[2].status === "fulfilled") store.website = secondary[2].value;
        else store.website.errors = [String(secondary[2].reason)];
      } catch (error) { store.initialError = String(error); }
      finally { if (!disposed) store.ready = true; }
    },
    dispose() { disposed = true; store.chat.dispose(); store.speech.dispose(); store.asr.dispose(); unsubscribers.splice(0).forEach((unlisten) => unlisten()); },
    async refreshSettings() {
      if (!desktop || disposed) return;
      store.settings = await invoke<Settings>("get_settings");
      store.game = await invoke<ControlStore["game"]>("get_game_status");
    },
    async refreshLibrary() { if (desktop) store.library = await invoke<WallpaperLibrary>("get_wallpaper_library"); },
    updateAppearance(patch) {
      const pending = appearanceQueue.catch(() => {}).then(async () => {
        const next = { ...store.settings, ...patch };
        if (!desktop) { store.settings = next; return; }
        store.settings = await invoke<Settings>("update_settings", { expression: next.expression, size: next.size, theme: next.theme, paused: next.paused, fps: next.fps });
      });
      appearanceQueue = pending;
      return pending;
    },
    async setAppearance(mode) { store.settings = desktop ? await invoke<Settings>("set_appearance", { mode }) : { ...store.settings, appearance: mode }; },
    async setMode(mode) { store.settings = desktop ? await invoke<Settings>("set_mode", { mode }) : { ...store.settings, mode }; },
    async setCompanion(enabled) { store.settings = desktop ? await invoke<Settings>("set_companion_enabled", { enabled }) : { ...store.settings, companionEnabled: enabled }; },
    async setCompanionSkin(skin) { store.settings = desktop ? await invoke<Settings>("set_companion_skin", { skin }) : { ...store.settings, companionSkin: skin }; },
    async setCompanionLinks(enabled) { store.settings = desktop ? await invoke<Settings>("set_companion_links", { enabled }) : { ...store.settings, companionLinks: enabled }; },
    async loadEffects() {
      store.effectList = desktop ? readList(await invoke("list_desktop_effects")) : await loadBuiltinPacks();
    },
    async openEffectsDirectory() {
      if (!desktop) throw new Error("请在桌面应用中打开特效文件夹");
      await invoke("open_effects_directory");
    },
    async importEffectPack() {
      if (!desktop) throw new Error("请在桌面应用中导入特效包");
      const outcome = readImported(await invoke("import_effect_pack"));
      if (outcome.status === "imported") await store.loadEffects();
      return outcome;
    },
    async deleteEffect(id: string) {
      if (!desktop) throw new Error("请在桌面应用中删除特效");
      await invoke("delete_desktop_effect", { id });
      await store.loadEffects();
      await store.refreshSettings();
    },
    setEffects(effects) {
      const pending = effectsQueue.catch(() => {}).then(async () => {
        const next = clone(effects);
        store.settings = desktop ? await invoke<Settings>("set_desktop_effects", { effects: next }) : { ...store.settings, effects: next };
      });
      effectsQueue = pending;
      return pending;
    },
    async setAutostart(enabled) {
      if (!desktop) throw new Error("请在桌面应用中设置开机启动");
      store.settings = await invoke<Settings>("set_autostart", { enabled });
    },
    async setDoubleClick(enabled) {
      if (!desktop) throw new Error("请在桌面应用中设置桌面快捷入口");
      store.settings = await invoke<Settings>("set_game_desktop_double_click", { enabled });
      store.game = await invoke<ControlStore["game"]>("get_game_status");
    },
    async openGames(gameId?: string) {
      if (!desktop) { window.open(`/?scene=games${gameId ? `&game=${encodeURIComponent(gameId)}` : ""}`, "_blank", "noopener"); return; }
      await invoke("open_game_space", { gameId: gameId ?? null });
      // A game on the desktop layer does not open the game space, so ask the host what is running.
      const status = await invoke<ControlStore["game"]>("get_game_status").catch(() => null);
      if (status) store.game = status;
      else store.game.active = true;
    },
    async stopDeskGame() {
      if (!desktop) return;
      await invoke("close_desktop_game");
      store.game.desk = { active: false, id: null };
    },
    async listGames() {
      return desktop ? readCatalog(await invoke("list_games")) : loadBuiltinCatalog();
    },
    async deleteUserGame(id: string) {
      if (!desktop) throw new Error("请在桌面应用中删除游戏");
      await invoke("delete_user_game", { id });
      await store.refreshSettings();
    },
    async openUserGamesFolder() {
      if (!desktop) throw new Error("请在桌面应用中打开游戏文件夹");
      await invoke("open_user_games_folder");
    },
    async applyWallpaper(id, options, preview = false) {
      if (!desktop) throw new Error("请在桌面应用中预览或应用壁纸");
      const settings = await invoke<Settings>("apply_wallpaper", { id, options: clone(options), preview });
      if (!preview) { store.settings = settings; await store.refreshLibrary(); }
    },
    async importMedia() {
      if (!desktop) throw new Error("请在桌面应用中导入图片或视频");
      const result = await invoke<{ id: string; library: WallpaperLibrary } | null>("import_local_wallpaper");
      if (!result) return null;
      store.library = result.library;
      return result.id;
    },
    async deleteWallpaper(id: string) {
      if (!desktop) throw new Error("请在桌面应用中删除壁纸");
      const res = await invoke<{ library: WallpaperLibrary }>("delete_local_wallpaper", { id });
      store.library = res.library;
      await store.refreshSettings();
    },
    async setAutomaticUpdates(enabled) {
      if (!desktop) {
        store.settings.automaticUpdates = enabled;
        return;
      }
      store.settings = await invoke<Settings>("set_automatic_updates", { enabled });
    },
    async checkWebsite() {
      if (!desktop) throw new Error("请在桌面应用中检查版本与公告");
      store.website = await invoke<WebsiteStatus>("check_website");
    },
    async sendAccountCode(identifier, purpose = "login") {
      if (!desktop) return { ok: true, cooldownSeconds: 60, expiresIn: 300 };
      return await accountInvoke<SendCodeResult>("send_account_code", { identifier, purpose });
    },
    async loginAccountWithCode(identifier, code) {
      if (!desktop) {
        if (identifier.includes("new")) return { kind: "registration", registrationToken: "mock.registration" };
        const result = mockLogin(identifier.split("@")[0] || "Xiaoban_user", false);
        return { kind: "session", ...result };
      }
      const outcome = await accountInvoke<CodeLoginOutcome>("login_account_with_code", { identifier, code });
      if (outcome.kind === "session") store.account = outcome.session;
      return outcome;
    },
    async loginAccountWithPassword(identifier, password) {
      if (!desktop) return mockLogin(identifier.split("@")[0] || "Xiaoban_user", false);
      const result = await accountInvoke<LoginResult>("login_account_with_password", { identifier, password });
      store.account = result.session;
      return result;
    },
    async registerAccount(registrationToken, username, password) {
      if (!desktop) return mockLogin(username, true);
      const result = await accountInvoke<LoginResult>("register_account", { registrationToken, username, password });
      store.account = result.session;
      return result;
    },
    async resetAccountPassword(identifier, code, newPassword) {
      if (!desktop) return;
      await accountInvoke("reset_account_password", { identifier, code, newPassword });
    },
    async checkAccountUsername(username) {
      if (!desktop) return { available: true };
      return await accountInvoke<UsernameCheck>("check_account_username", { username });
    },
    async getSignupMethods() {
      if (!desktop) return { email: true, phone: false };
      return await invoke<SignupMethods>("get_signup_methods");
    },
    async logoutAccount() {
      if (!desktop) {
        store.account = null;
        return;
      }
      await invoke("logout_account");
      store.account = null;
    },
  } as ControlStore);
  return store;
}

