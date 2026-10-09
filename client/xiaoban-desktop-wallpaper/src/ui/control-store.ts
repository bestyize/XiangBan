import { reactive } from "vue";
import { desktop, defaults, invoke, listen, type Settings, type WallpaperEntry, type WallpaperLibrary, type AppInfo } from "../bridge.ts";
import type { WebsiteStatus } from "./website-types.ts";
import type { ControlStore } from "./control-types.ts";

export const clone = <T>(value: T): T => JSON.parse(JSON.stringify(value));
export function createControlStore(notify: ControlStore["notify"]): ControlStore {
  let disposed = false;
  const unsubscribers: (() => void)[] = [];
  // Serialize partial appearance updates so a fast sequence cannot overwrite another setting.
  let appearanceQueue = Promise.resolve();
  const store: ControlStore = reactive({
    settings: clone(defaults),
    info: { name: "小伴桌面", version: "0.1.0", platform: "preview", arch: "", updateTarget: "", updateUrl: "" },
    library: { items: [], source: "", activeId: "companion" },
    website: { announcements: [], update: null, dismissed: {}, errors: [] },
    startup: { available: false, error: null },
    game: { active: false, desktopDoubleClickSupported: false, desktopEntryError: null },
    page: "wallpapers", ready: false, initialError: "", desktop, notify,
    navigate(page) { store.page = page; },
    async initialize() {
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
          listen<Settings>("settings-changed", (settings) => { if (!disposed) store.settings = settings; }),
          listen<WebsiteStatus>("website-status", (website) => { if (!disposed) store.website = website; }),
          listen<boolean>("games-active", (active) => { if (!disposed) store.game.active = active; }),
          listen<string>("desktop-error", (error) => notify(error, "error")),
        ]);
        if (disposed) { subscriptions.forEach((unlisten) => unlisten()); return; }
        unsubscribers.push(...subscriptions);
        const initial = await Promise.allSettled([
          invoke<Settings>("get_settings"), invoke<AppInfo>("get_app_info"), invoke<WallpaperLibrary>("get_wallpaper_library"),
        ]);
        if (disposed) return;
        if (initial[0].status === "rejected") throw initial[0].reason;
        store.settings = initial[0].value;
        if (initial[1].status === "fulfilled") store.info = initial[1].value;
        else notify(`应用信息读取失败：${initial[1].reason}`, "error");
        if (initial[2].status === "fulfilled") store.library = initial[2].value;
        else notify(`本地壁纸目录读取失败：${initial[2].reason}`, "error");
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
    dispose() { disposed = true; unsubscribers.splice(0).forEach((unlisten) => unlisten()); },
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
    async setMode(mode) { store.settings = desktop ? await invoke<Settings>("set_mode", { mode }) : { ...store.settings, mode }; },
    async setCompanion(enabled) { store.settings = desktop ? await invoke<Settings>("set_companion_enabled", { enabled }) : { ...store.settings, companionEnabled: enabled }; },
    async setAutostart(enabled) {
      if (!desktop) throw new Error("请在桌面应用中设置开机启动");
      store.settings = await invoke<Settings>("set_autostart", { enabled });
    },
    async setDoubleClick(enabled) {
      if (!desktop) throw new Error("请在桌面应用中设置桌面快捷入口");
      store.settings = await invoke<Settings>("set_game_desktop_double_click", { enabled });
      store.game = await invoke<ControlStore["game"]>("get_game_status");
    },
    async openGames() {
      if (!desktop) { window.open("/?scene=games", "_blank", "noopener"); return; }
      await invoke("open_game_space");
      store.game.active = true;
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
    async configureWebsite(url, automatic) {
      if (!desktop) throw new Error("请在桌面应用中设置壁纸来源");
      store.settings = await invoke<Settings>("configure_website", { url, automatic });
      store.website = await invoke<WebsiteStatus>("get_website_status");
    },
    async checkWebsite() {
      if (!desktop) throw new Error("请在桌面应用中检查版本与公告");
      store.website = await invoke<WebsiteStatus>("check_website");
    },
  } as ControlStore);
  return store;
}
