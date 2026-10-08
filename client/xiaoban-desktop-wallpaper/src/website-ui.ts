import {
  desktop,
  invoke,
  listen,
  type Settings,
  type WallpaperLibrary,
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
interface SiteWallpaper {
  id: string;
  title: string;
  media_type: string;
  preview_url: string;
  downloads: { thumb: string };
  author: { name: string };
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
function imageUrl(value: string, site: string) {
  try {
    const url = new URL(value, site);
    return url.protocol === "https:" ||
      (url.protocol === "http:" &&
        ["localhost", "127.0.0.1", "[::1]"].includes(url.hostname))
      ? url.href
      : "";
  } catch {
    return "";
  }
}
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
  private root: HTMLElement | null = null;
  constructor(
    private options: {
      config: () => Settings;
      configure: (config: Settings) => void;
      toast: (message: unknown, error?: boolean) => void;
      local: () => void;
      about: () => void;
      selected: (library: WallpaperLibrary, id: string) => void;
      status: (status: WebsiteStatus) => void;
    },
  ) {}
  async start() {
    if (!desktop) return;
    this.receive(await invoke<WebsiteStatus>("get_website_status"));
    await listen<WebsiteStatus>("website-status", (status) =>
      this.receive(status),
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
  render(root: HTMLElement) {
    this.root = root;
    root.innerHTML = `<div class="wallpaper-toolbar"><div class="filter-tabs"><button id="site-local">本地壁纸</button><button class="selected">在线壁纸</button></div><button class="secondary-button" id="site-settings">网站设置</button></div><form class="site-search"><input id="site-q" placeholder="搜索本站壁纸" value="${esc(this.query.q)}"><select id="site-kind"><option value="image">图片壁纸</option><option value="video">动态视频</option></select><select id="site-category"><option value="">全部分类</option>${this.categories.map((c) => `<option value="${esc(c.id)}">${esc(c.name)}</option>`).join("")}</select><button class="primary-button" ${this.loading ? "disabled" : ""}>搜索</button></form><p class="site-summary">${esc(this.options.config().websiteUrl)} · ${this.feed.total} 个结果</p>${this.error ? `<div class="site-error" role="alert">${esc(this.error)}<button id="site-retry">重试</button></div>` : ""}<div class="site-gallery wallpaper-gallery">${this.feed.items
      .map((item) => {
        const url = imageUrl(
          item.downloads.thumb || item.preview_url,
          this.options.config().websiteUrl,
        );
        return `<button class="wallpaper-card" data-site-id="${esc(item.id)}" ${this.loading ? "disabled" : ""}><div class="wallpaper-cover">${url ? `<img src="${esc(url)}" alt="${esc(item.title)}" loading="lazy">` : '<span class="fallback-art">✧</span>'}<span class="wallpaper-type">${item.media_type === "video" ? "视频" : "图片"}</span></div><div class="wallpaper-card-info"><b>${esc(item.title)}</b><small>${esc(item.author.name)}<em>选择壁纸</em></small></div></button>`;
      })
      .join(
        "",
      )}</div>${this.loading ? '<p class="site-summary" role="status">正在加载，请稍后…</p>' : ""}${!this.loading && !this.error && !this.feed.items.length ? '<p class="empty-gallery">暂时没有上架的壁纸。</p>' : ""}${this.feed.has_more ? `<button class="secondary-button site-more" id="site-more" ${this.loading ? "disabled" : ""}>加载更多</button>` : ""}`;
    root.querySelector("#site-local")!.addEventListener("click", () => {
      this.root = null;
      this.options.local();
    });
    root.querySelector("#site-settings")!.addEventListener("click", () => {
      this.root = null;
      this.options.about();
    });
    const kind = root.querySelector<HTMLSelectElement>("#site-kind")!,
      category = root.querySelector<HTMLSelectElement>("#site-category")!;
    kind.value = this.query.media;
    category.value = this.query.category;
    root.querySelector("form")!.addEventListener("submit", (e) => {
      e.preventDefault();
      this.query.q = root
        .querySelector<HTMLInputElement>("#site-q")!
        .value.trim();
      this.query.media = kind.value;
      this.query.category = category.value;
      this.query.page = 1;
      void this.load();
    });
    [kind, category].forEach((select) =>
      select.addEventListener("change", () => {
        this.query.media = kind.value;
        this.query.category = category.value;
        this.query.page = 1;
        void this.load();
      }),
    );
    root.querySelector("#site-more")?.addEventListener("click", () => {
      this.query.page++;
      void this.load(true);
    });
    root.querySelector("#site-retry")?.addEventListener("click", () => {
      this.query.page = 1;
      void this.load();
    });
    root
      .querySelectorAll<HTMLButtonElement>("[data-site-id]")
      .forEach((button) =>
        button.addEventListener("click", () => {
          if (this.loading) return;
          this.loading = true;
          const source = this.options.config().websiteUrl;
          const generation = ++this.generation;
          button.disabled = true;
          button.querySelector("em")!.textContent = "正在准备…";
          void invoke<{ library: WallpaperLibrary; id: string }>(
            "select_site_wallpaper",
            { id: button.dataset.siteId },
          )
            .then((result) => {
              if (
                source !== this.options.config().websiteUrl ||
                generation !== this.generation
              )
                return;
              this.root = null;
              this.options.selected(result.library, result.id);
            })
            .catch((e) => this.options.toast(e, true))
            .finally(() => {
              this.loading = false;
              if (this.root?.querySelector(".site-gallery"))
                this.render(this.root);
            });
        }),
      );
  }
  async open(root: HTMLElement) {
    this.render(root);
    if (!desktop) {
      this.error = "请在桌面应用中浏览在线壁纸";
      this.render(root);
      return;
    }
    if (!this.categories.length) {
      try {
        this.categories = (
          await invoke<{ categories: { id: string; name: string }[] }>(
            "fetch_site_categories",
          )
        ).categories;
      } catch {
        /* Search still works if categories are temporarily unavailable. */
      }
    }
    await this.load();
  }
  private async load(more = false) {
    const generation = ++this.generation;
    const source = this.options.config().websiteUrl;
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
    } catch (e) {
      if (generation === this.generation) {
        this.error = String(e);
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
