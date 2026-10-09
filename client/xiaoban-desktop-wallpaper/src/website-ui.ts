import {
  siteMedia,
  mediaMarkup,
  bindMedia,
  type SiteWallpaper,
} from "./wallpaper-media.ts";
import {
  desktop,
  invoke,
  listen,
  type Settings,
  type WallpaperLibrary,
  type WallpaperOptions,
  wallpaperDefaults,
} from "./bridge.ts";
interface Notice {
  id: string;
  title: string;
  body: string;
  revision: string;
}
export interface WebsiteStatus {
  announcements: Notice[];
  update: Update | null;
  dismissed: Record<string, string>;
  errors: string[];
}
export interface Update {
  available: boolean;
  version: string;
  notes: string;
  publishedAt?: string;
  downloadUrl?: string;
}
interface Feed {
  items: SiteWallpaper[];
  has_more: boolean;
  total: number;
}
const esc = (value: unknown) =>
  String(value ?? "").replace(
    /[&<>"']/g,
    (c) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[
        c
      ]!,
  );
export class WebsiteUi {
  private status: WebsiteStatus = {
    announcements: [],
    update: null,
    dismissed: {},
    errors: [],
  };
  private feed: Feed = { items: [], has_more: false, total: 0 };
  private categories: { id: string; name: string }[] = [];
  private query = { q: "", category: "", media: "image", page: 1 };
  private loading = false;
  private error = "";
  private generation = 0;
  private refreshing = false;
  private cleanupMedia: (() => void) | null = null;
  private chosen: SiteWallpaper | null = null;
  private draft: WallpaperOptions = JSON.parse(
    JSON.stringify(wallpaperDefaults),
  );
  private applying = false;
  private prepared: { sourceId: string; id: string } | null = null;
  private progress: { received: number; total: number } | null = null;
  private loadedSource = "";
  private root: HTMLElement | null = null;
  constructor(
    private options: {
      config: () => Settings;
      configure: (config: Settings) => void;
      toast: (message: unknown, error?: boolean) => void;
      local: () => void;
      about: () => void;
      library: (library: WallpaperLibrary) => void;
      applied: (settings: Settings) => void;
      importLocal: () => void;
      status: (status: WebsiteStatus) => void;
    },
  ) {}
  async start() {
    if (!desktop) return;
    this.receive(await invoke<WebsiteStatus>("get_website_status"));
    await listen<WebsiteStatus>("website-status", (status) =>
      this.receive(status),
    );
    await listen<{ id: string; received: number; total: number }>(
      "wallpaper-progress",
      (progress) => {
        if (
          this.applying &&
          progress.id === (this.prepared?.id ?? this.chosen?.desktop_id)
        ) {
          this.progress = progress;
          const host =
            this.root?.querySelector<HTMLElement>(".online-progress");
          if (host) {
            host.querySelector("span")!.textContent =
              `正在下载 ${Math.round((progress.received / progress.total) * 100)}%`;
            host.querySelector("progress")!.value =
              progress.received / progress.total;
          }
        }
      },
    );
  }
  private receive(status: WebsiteStatus) {
    const changed = JSON.stringify(status) !== JSON.stringify(this.status);
    this.status = status;
    this.banner();
    if (changed) this.options.status(status);
  }
  private banner() {
    let host = document.querySelector<HTMLElement>("#website-banner");
    if (!host) {
      host = document.createElement("div");
      host.id = "website-banner";
      document.querySelector(".main-pane")!.prepend(host);
    }
    const notices = this.status.announcements.filter(
      (n) => this.status.dismissed[n.id] !== n.revision,
    );
    const update =
      this.status.update?.available &&
      this.status.dismissed.update !== this.status.update.version
        ? this.status.update
        : null;
    host.innerHTML = `${update ? `<article><div><b>发现新版本 v${esc(update.version)}</b><p>打开关于小伴，查看更新说明并下载。</p></div><button data-about>查看更新</button><button data-dismiss="update" data-revision="${esc(update.version)}" aria-label="关闭更新提醒">×</button></article>` : ""}${notices.map((n) => `<article><div><b>${esc(n.title)}</b><p>${esc(n.body)}</p></div><button data-dismiss="${esc(n.id)}" data-revision="${esc(n.revision)}">知道了</button></article>`).join("")}`;
    host
      .querySelector("[data-about]")
      ?.addEventListener("click", this.options.about);
    host.querySelectorAll<HTMLElement>("[data-dismiss]").forEach((button) =>
      button.addEventListener("click", () => {
        void invoke<WebsiteStatus>("dismiss_website_message", {
          id: button.dataset.dismiss,
          revision: button.dataset.revision,
        })
          .then((status) => this.receive(status))
          .catch((e) => this.options.toast(e, true));
      }),
    );
    document.querySelector<HTMLElement>(
      '[data-page="about"] small',
    )!.textContent = update || notices.length ? "有新版本或公告" : "了解与更新";
  }
  detach() {
    this.inspectorCleanup?.();
    this.inspectorCleanup = null;
    this.cleanupMedia?.();
    this.cleanupMedia = null;
    this.root = null;
  }
  render(root: HTMLElement) {
    this.cleanupMedia?.();
    this.root = root;
    root.innerHTML = `<div class="wallpaper-sourcebar"><div class="filter-tabs source-tabs"><button class="selected" aria-pressed="true">在线壁纸</button><button id="site-local">我的壁纸</button></div><button class="secondary-button" id="site-import">导入图片 / 视频</button></div><form class="site-search"><div class="filter-tabs media-tabs"><button type="button" data-site-kind="image" class="${this.query.media === "image" ? "selected" : ""}">图片</button><button type="button" data-site-kind="video" class="${this.query.media === "video" ? "selected" : ""}">动态视频</button></div><input id="site-q" aria-label="搜索壁纸" placeholder="搜索喜欢的风景" value="${esc(this.query.q)}"><select id="site-category" aria-label="壁纸分类"><option value="">全部分类</option>${this.categories.map((c) => `<option value="${esc(c.id)}">${esc(c.name)}</option>`).join("")}</select><button class="primary-button" ${this.loading || this.applying ? "disabled" : ""}>搜索</button></form><div class="site-summary"><span>${this.loading ? "正在加载…" : `${this.feed.total} 个结果`} · 选中壁纸，在右侧预览和应用</span><button id="site-settings">网站设置</button></div>${this.error ? `<div class="site-error" role="alert">${esc(this.error)} <button id="site-retry">重试</button></div>` : ""}<div class="wallpaper-layout online-layout"><div><div class="site-gallery wallpaper-gallery">${this.feed.items
      .map((item) => {
        const media = siteMedia(item, this.options.config().websiteUrl);
        return `<button class="wallpaper-card ${this.chosen?.id === item.id ? "selected" : ""}" data-site-id="${esc(item.id)}" aria-pressed="${this.chosen?.id === item.id}" ${this.applying ? "disabled" : ""}><div class="wallpaper-cover">${mediaMarkup(item.media_type, media.video, media.poster, item.title)}<span class="wallpaper-type">${item.media_type === "video" ? "视频" : "图片"}</span>${item.media_type === "video" ? '<span class="video-mark">▶</span>' : ""}</div><div class="wallpaper-card-info"><b>${esc(item.title)}</b><small>${esc(item.author.name)}<em>${item.downloaded ? "已在我的壁纸" : "在线"}</em></small></div></button>`;
      })
      .join(
        "",
      )}</div>${this.loading ? '<p class="site-summary" role="status">正在加载，请稍后…</p>' : ""}${!this.loading && !this.error && !this.feed.items.length ? '<div class="empty-gallery">暂时没有这一类壁纸，试试其他分类。</div>' : ""}${this.feed.has_more ? `<button class="secondary-button site-more" id="site-more" ${this.loading || this.applying ? "disabled" : ""}>加载更多</button>` : ""}</div><aside class="wallpaper-inspector online-inspector" id="online-inspector"></aside></div>`;
    root.querySelector("#site-local")!.addEventListener("click", () => {
      this.detach();
      this.options.local();
    });
    root.querySelector("#site-import")!.addEventListener("click", () => {
      if (!this.applying) this.options.importLocal();
    });
    root.querySelector("#site-settings")!.addEventListener("click", () => {
      this.detach();
      this.options.about();
    });
    const category = root.querySelector<HTMLSelectElement>("#site-category")!;
    category.value = this.query.category;
    const search = () => {
      if (this.applying || this.loading) return;
      this.query.q = root
        .querySelector<HTMLInputElement>("#site-q")!
        .value.trim();
      this.query.category = category.value;
      this.query.page = 1;
      void this.load();
    };
    root.querySelector("form")!.addEventListener("submit", (e) => {
      e.preventDefault();
      search();
    });
    category.addEventListener("change", search);
    root
      .querySelectorAll<HTMLButtonElement>("[data-site-kind]")
      .forEach((button) =>
        button.addEventListener("click", () => {
          if (
            this.applying ||
            this.loading ||
            this.query.media === button.dataset.siteKind
          )
            return;
          this.query.media = button.dataset.siteKind!;
          this.chosen = null;
          this.prepared = null;
          search();
        }),
      );
    root.querySelector("#site-more")?.addEventListener("click", () => {
      if (!this.loading && !this.applying) {
        this.query.page++;
        void this.load(true);
      }
    });
    root.querySelector("#site-retry")?.addEventListener("click", () => {
      this.query.page = 1;
      void this.load();
    });
    root
      .querySelectorAll<HTMLButtonElement>("[data-site-id]")
      .forEach((button) =>
        button.addEventListener("click", () => {
          if (this.applying) return;
          this.chosen = this.feed.items.find(
            (item) => item.id === button.dataset.siteId,
          )!;
          this.prepared =
            this.chosen.downloaded && this.chosen.desktop_id
              ? { sourceId: this.chosen.id, id: this.chosen.desktop_id }
              : null;
          this.progress = null;
          this.draft = JSON.parse(JSON.stringify(wallpaperDefaults));
          root
            .querySelectorAll<HTMLElement>("[data-site-id]")
            .forEach((card) => {
              const chosen = card.dataset.siteId === this.chosen!.id;
              card.classList.toggle("selected", chosen);
              card.setAttribute("aria-pressed", String(chosen));
            });
          this.renderInspector();
        }),
      );
    this.renderInspector();
    this.cleanupMedia = bindMedia(
      root.querySelector<HTMLElement>(".site-gallery")!,
    );
  }
  private inspectorCleanup: (() => void) | null = null;
  private renderInspector() {
    this.inspectorCleanup?.();
    this.inspectorCleanup = null;
    const host = this.root?.querySelector<HTMLElement>("#online-inspector");
    if (!host) return;
    const item = this.chosen;
    if (!item) {
      host.innerHTML =
        '<div class="inspector-empty"><span>✧</span><h2>选一张喜欢的壁纸</h2><p>点击卡片查看大图或播放视频，再决定是否应用到桌面。</p><small>浏览预览不会更换当前壁纸。</small></div>';
      return;
    }
    const media = siteMedia(item, this.options.config().websiteUrl);
    const active =
      (this.prepared?.id || item.desktop_id) ===
      this.options.config().wallpaperId;
    host.innerHTML = `<div class="inspector-preview">${mediaMarkup(item.media_type, media.video, media.poster, item.title, true)}</div><span class="inspector-eyebrow">${item.media_type === "video" ? "动态视频" : "图片"}壁纸</span><h2>${esc(item.title)}</h2><p>${esc(item.description || "为桌面留下一点喜欢的风景。")}</p><div class="inspector-meta"><span>${esc(item.author.name)}</span>${item.resolution ? `<span>${item.resolution.width} × ${item.resolution.height}</span>` : ""}</div><div class="wallpaper-actions"><button class="secondary-button wide" id="online-save" ${this.applying || item.downloaded ? "disabled" : ""}>${item.downloaded ? "已保存到我的壁纸" : "保存到我的壁纸"}</button><button class="primary-button wide" id="online-apply" ${this.applying ? "disabled" : ""}>${this.applying ? "正在准备…" : active ? "更新壁纸设置" : "应用到桌面"}</button></div><details class="inspector-controls"><summary>画面与播放设置</summary><div class="compact-field"><label for="online-fit">画面适配</label><select id="online-fit"><option value="cover">铺满屏幕</option><option value="contain">完整显示</option><option value="fill">拉伸适配</option></select></div>${item.media_type === "video" ? `<div class="compact-field"><label for="online-speed">播放速度 <span>${this.draft.playbackRate}×</span></label><input id="online-speed" type="range" min="0.25" max="2" step="0.25" value="${this.draft.playbackRate}"><small>静音循环播放</small></div>` : ""}</details>${this.applying ? `<div class="download-progress online-progress" role="status"><span>${this.progress ? `正在下载 ${Math.round((this.progress.received / this.progress.total) * 100)}%` : "正在准备壁纸文件…"}</span><progress max="1" ${this.progress ? `value="${this.progress.received / this.progress.total}"` : ""}></progress></div>` : ""}<div class="inspector-tip">下载完成后可离线使用。${this.options.config().mode === "floating" ? " 小伴继续悬浮陪伴。" : " 小伴会融入这张壁纸。"}</div>`;
    const fit = host.querySelector<HTMLSelectElement>("#online-fit")!;
    fit.value = this.draft.fit;
    fit.disabled = this.applying;
    fit.addEventListener("change", () => {
      this.draft.fit = fit.value as WallpaperOptions["fit"];
      const image = host.querySelector<HTMLElement>("img,video");
      if (image) image.style.objectFit = fit.value;
    });
    const speed = host.querySelector<HTMLInputElement>("#online-speed");
    if (speed) {
      speed.disabled = this.applying;
      speed.addEventListener("input", () => {
        this.draft.playbackRate = Number(speed.value);
        speed.parentElement!.querySelector("label span")!.textContent =
          `${speed.value}×`;
        const video = host.querySelector("video");
        if (video) video.playbackRate = this.draft.playbackRate;
      });
    }
    host.querySelector("#online-save")!.addEventListener("click", () => {
      void this.saveOnline();
    });
    host.querySelector("#online-apply")!.addEventListener("click", () => {
      void this.applyOnline(false);
    });
    this.inspectorCleanup = bindMedia(host);
  }
  private async saveOnline() {
    if (!desktop || !this.chosen || this.applying || this.chosen.downloaded)
      return;
    const item = this.chosen;
    this.applying = true;
    this.progress = null;
    this.renderInspector();
    try {
      const result = await invoke<{ library: WallpaperLibrary; id: string }>(
        "download_site_wallpaper",
        { id: item.id },
      );
      this.prepared = { sourceId: item.id, id: result.id };
      item.downloaded = true;
      item.desktop_id = result.id;
      this.options.library(result.library);
      this.options.toast("已保存到我的壁纸，可以离线使用；当前桌面未更换");
    } catch (error) {
      this.options.toast(error, true);
    } finally {
      this.applying = false;
      this.progress = null;
      this.renderInspector();
    }
  }
  private async applyOnline(preview: boolean) {
    if (!desktop) {
      this.options.toast("请在桌面应用中下载和应用壁纸");
      return;
    }
    if (this.applying || !this.chosen) return;
    const item = this.chosen,
      source = this.options.config().websiteUrl;
    this.applying = true;
    this.progress = null;
    this.renderInspector();
    try {
      if (this.prepared?.sourceId !== item.id) {
        const result = await invoke<{ library: WallpaperLibrary; id: string }>(
          "download_site_wallpaper",
          { id: item.id },
        );
        if (source !== this.options.config().websiteUrl)
          throw new Error("网站地址已改变，请重新选择壁纸");
        this.prepared = { sourceId: item.id, id: result.id };
        item.downloaded = true;
        item.desktop_id = result.id;
        this.options.library(result.library);
      }
      const result = await invoke<Settings>("apply_wallpaper", {
        id: this.prepared.id,
        options: this.draft,
        preview,
      });
      if (!preview) this.options.applied(result);
      this.options.library(
        await invoke<WallpaperLibrary>("get_wallpaper_library"),
      );
      this.options.toast(
        preview ? "壁纸预览已打开" : "壁纸已应用，仍可继续浏览在线图库",
      );
    } catch (error) {
      this.options.toast(error, true);
    } finally {
      this.applying = false;
      this.progress = null;
      this.renderInspector();
    }
  }
  async open(root: HTMLElement) {
    this.render(root);
    if (!desktop) {
      this.error = "请在桌面应用中浏览在线壁纸";
      this.render(root);
      return;
    }
    const source = this.options.config().websiteUrl;
    if (source === this.loadedSource && !this.error) return;
    if (!this.categories.length) {
      try {
        this.categories = (
          await invoke<{ categories: { id: string; name: string }[] }>(
            "fetch_site_categories",
          )
        ).categories;
      } catch {}
    }
    await this.load();
  }
  private async load(more = false) {
    if (this.applying) return;
    const generation = ++this.generation,
      source = this.options.config().websiteUrl;
    this.loading = true;
    this.error = "";
    if (!more) this.feed = { items: [], has_more: false, total: 0 };
    if (this.root?.querySelector(".site-gallery")) this.render(this.root);
    try {
      const feed = await invoke<Feed>("fetch_site_wallpapers", this.query);
      if (
        generation !== this.generation ||
        source !== this.options.config().websiteUrl
      )
        return;
      this.feed = {
        ...feed,
        items: more ? [...this.feed.items, ...feed.items] : feed.items,
      };
      this.loadedSource = source;
      if (
        this.chosen &&
        !this.feed.items.some((item) => item.id === this.chosen!.id)
      ) {
        this.chosen = null;
        this.prepared = null;
      }
    } catch (error) {
      if (generation === this.generation) {
        this.error = String(error);
        if (more) this.query.page--;
      }
    } finally {
      if (generation === this.generation) {
        this.loading = false;
        if (this.root?.querySelector(".site-gallery")) this.render(this.root);
      }
    }
  }
  appendAbout(root: HTMLElement) {
    root.insertAdjacentHTML(
      "beforeend",
      `<section class="settings-card website-settings"><div class="section-title"><h2>壁纸网站</h2><span>在线图片、动态壁纸、版本与公告</span></div><form id="website-form"><label for="website-url">网站地址</label><input id="website-url" type="url" value="${esc(this.options.config().websiteUrl)}" required><label class="setting-row switch-row"><span><b>自动检查版本与公告</b><small>启动时检查，运行期间每 30 分钟检查一次。</small></span><input id="website-auto" type="checkbox" ${this.options.config().automaticUpdates ? "checked" : ""}></label><button class="secondary-button" ${desktop ? "" : "disabled"}>保存网站设置</button><button id="website-refresh" class="primary-button" type="button" ${desktop ? "" : "disabled"}>获取公告与版本</button><p class="site-summary" id="website-errors">${esc(this.status.errors.join("\n"))}</p></form></section>${this.status.announcements.length ? `<section class="settings-card website-notices"><h2>网站公告</h2>${this.status.announcements.map((n) => `<article><h3>${esc(n.title)}</h3><p>${esc(n.body)}</p></article>`).join("")}</section>` : ""}`,
    );
    root.querySelector("#website-form")!.addEventListener("submit", (event) => {
      event.preventDefault();
      const button = root.querySelector<HTMLButtonElement>(
        "#website-form button",
      )!;
      button.disabled = true;
      void invoke<Settings>("configure_website", {
        url: root.querySelector<HTMLInputElement>("#website-url")!.value,
        automatic:
          root.querySelector<HTMLInputElement>("#website-auto")!.checked,
      })
        .then(async (config) => {
          this.options.configure(config);
          this.categories = [];
          this.feed = { items: [], has_more: false, total: 0 };
          this.loadedSource = "";
          this.chosen = null;
          this.prepared = null;
          ++this.generation;
          this.options.toast("网站设置已保存");
          await this.refresh();
        })
        .catch((e) => this.options.toast(e, true))
        .finally(() => {
          if (button.isConnected) button.disabled = false;
        });
    });
    root.querySelector("#website-refresh")!.addEventListener("click", () => {
      void this.refresh();
    });
  }
  async refresh() {
    if (!desktop || this.refreshing) return;
    this.refreshing = true;
    try {
      this.receive(await invoke<WebsiteStatus>("check_website"));
      if (this.status.errors.length)
        this.options.toast(this.status.errors.join("\n"), true);
      else this.options.toast("版本与公告已同步");
    } catch (e) {
      this.options.toast(e, true);
    } finally {
      this.refreshing = false;
    }
  }
}
