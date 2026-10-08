import { EXPRESSIONS, EXPRESSION_NAMES } from "./companion/expressions.ts";
import { PetView } from "./pet.ts";
import {
  desktop,
  defaults,
  wallpaperDefaults,
  invoke,
  listen,
  type Settings,
  type WallpaperOptions,
  type WallpaperEntry,
  type WallpaperLibrary,
  type AppInfo,
} from "./bridge.ts";
const paths: Record<string, string> = {
  pet: "M5 9 3 3l6 3h6l6-3-2 6v8a4 4 0 0 1-4 4H9a4 4 0 0 1-4-4ZM8 13h.01M16 13h.01M10 17q2 2 4 0",
  wallpaper: "M4 4h16v16H4ZM4 16l5-5 4 4 3-3 4 4M15 8h.01",
  about: "M12 8h.01M12 11v6M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0",
  arrow: "m9 5 7 7-7 7",
  plus: "M12 5v14M5 12h14",
  close: "m6 6 12 12M18 6 6 18",
  play: "m9 5 10 7-10 7Z",
  check: "m5 12 4 4L19 6",
  download: "M12 3v12m-5-5 5 5 5-5M5 16v5h14v-5",
  heart: "M20 5c-3-3-6 0-8 2-2-2-5-5-8-2s0 7 8 14c8-7 11-11 8-14Z",
  minimize: "M5 12h14",
  power: "M12 3v9M6 6a9 9 0 1 0 12 0",
  refresh: "M20 8V3l-3 3M4 16v5l3-3M20 8a8 8 0 0 0-13-3M4 16a8 8 0 0 0 13 3",
  folder: "M3 7V4h7l2 3h9v13H3Z",
  settings: "M5 7h14M5 17h14M9 4v6M15 14v6",
};
const icon = (name: string) =>
  `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="${paths[name] ?? paths.pet}"/></svg>`;
const esc = (value: unknown) =>
  String(value ?? "").replace(
    /[&<>"']/g,
    (c) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[
        c
      ]!,
  );
const clone = <T>(value: T): T => JSON.parse(JSON.stringify(value));
const kindLabel = (kind: string) =>
  ({ image: "图片", video: "视频", canvas: "动画" })[kind] ?? "默认";
export async function mountControl() {
  let config: Settings = { ...defaults },
    page = "pet",
    pet: PetView | null = null,
    chosen = "companion",
    filter = "all",
    busy = false,
    startupAvailable = false;
  let library: WallpaperLibrary = {
      items: [],
      source: "",
      activeId: "companion",
    },
    draft: WallpaperOptions = clone(wallpaperDefaults);
  let info: AppInfo = {
    name: "小伴桌面",
    version: "0.1.0",
    platform: "preview",
    arch: "",
    updateTarget: "",
    updateUrl: "",
  };
  let lastUpdate: {
    available: boolean;
    version: string;
    notes: string;
    publishedAt?: string;
    downloadUrl?: string;
  } | null = null;
  if (desktop) {
    [config, info] = await Promise.all([
      invoke<Settings>("get_settings"),
      invoke<AppInfo>("get_app_info"),
    ]);
    startupAvailable = (
      await invoke<{ available: boolean }>("get_startup_status")
    ).available;
  }
  const local = await fetch("/wallpapers/builtin/catalog.json").then((r) =>
    r.json(),
  );
  library.items = local.items.map(
    (definition: WallpaperEntry["definition"]) => ({
      definition,
      installed: true,
      coverUrl:
        definition.cover?.url.replace("builtin://", "/wallpapers/builtin/") ??
        null,
    }),
  );
  if (desktop)
    library = await invoke<WallpaperLibrary>("get_wallpaper_library");
  chosen = config.wallpaperId;
  document.body.className = "control";
  document.querySelector<HTMLElement>("#app")!.innerHTML =
    `<div class="app-shell"><aside class="sidebar"><div class="brand"><div class="brand-mark">${icon("pet")}</div><div><strong>小伴桌面</strong><span>一点陪伴，一点美好</span></div></div><div class="nav-label">我的桌面</div><nav aria-label="设置导航">${[
      ["pet", "小伴设置", "让陪伴随你心意"],
      ["wallpaper", "动态壁纸", "给桌面换个心情"],
      ["about", "关于小伴", "了解与更新"],
    ]
      .map(
        ([id, title, description]) =>
          `<button class="nav-item" data-page="${id}"><span class="nav-icon">${icon(id)}</span><span><b>${title}</b><small>${description}</small></span><i></i></button>`,
      )
      .join(
        "",
      )}</nav><div class="sidebar-note"><span>✧</span><p>平常的日子，<br>也可以有一点小美好。</p></div><div class="sidebar-bottom"><div class="presence"><i></i><span id="presence-text">小伴正在陪伴</span></div><div class="sidebar-actions"><button id="hide">${icon("minimize")}收起到托盘</button><button id="quit" aria-label="退出小伴" title="退出小伴">${icon("power")}</button></div><small id="sidebar-version"></small></div></aside><main class="main-pane"><header class="page-heading"><div><div class="eyebrow" id="eyebrow"></div><h1 id="page-title"></h1><p id="page-description"></p></div><span class="desktop-badge">${icon("pet")}你的桌面小伙伴</span></header><div class="page-body" id="page-body"></div></main><div class="toast" id="toast" role="status" hidden></div><div id="modal-root"></div><input id="manifest-file" type="file" accept=".json,application/json" hidden></div>`;
  const body = document.querySelector<HTMLElement>("#page-body")!;
  const toast = (message: unknown, error = false) => {
    const host = document.querySelector<HTMLElement>("#toast")!;
    host.textContent = String(message);
    host.dataset.error = String(error);
    host.hidden = false;
    window.setTimeout(() => (host.hidden = true), error ? 8000 : 4200);
  };
  const failure = (error: unknown) =>
    toast(error instanceof Error ? error.message : error, true);
  const task = (fn: () => Promise<unknown>) => void fn().catch(failure);
  const current = (id = chosen) =>
    library.items.find((entry) => entry.definition.id === id);
  const activeName = () =>
    config.wallpaperId === "companion"
      ? config.mode === "floating"
        ? "系统桌面"
        : "小伴房间"
      : (current(config.wallpaperId)?.definition.title ?? "动态壁纸");
  function settingsApply(next: Settings) {
    config = next;
    document.body.dataset.theme = config.theme;
    document.querySelector("#presence-text")!.textContent = config.paused
      ? "小伴休息中"
      : "小伴正在陪伴";
    document.querySelector<HTMLElement>(".presence")!.dataset.paused = String(
      config.paused,
    );
    pet?.update({ ...config, size: 280 });
    if (page !== "pet") return;
    body.querySelectorAll<HTMLElement>("[data-mode]").forEach((button) => {
      const selected = button.dataset.mode === config.mode;
      button.classList.toggle("selected", selected);
      button.setAttribute("aria-pressed", String(selected));
    });
    body.querySelector<HTMLSelectElement>("#expression")!.value =
      config.expression;
    body.querySelector<HTMLInputElement>("#size")!.value = String(config.size);
    body.querySelector("#size-value")!.textContent = `${config.size}px`;
    body
      .querySelectorAll<HTMLElement>("[data-theme]")
      .forEach((b) =>
        b.classList.toggle("selected", b.dataset.theme === config.theme),
      );
    body
      .querySelectorAll<HTMLElement>("[data-fps]")
      .forEach((b) =>
        b.classList.toggle("selected", Number(b.dataset.fps) === config.fps),
      );
    body.querySelector<HTMLInputElement>("#autostart")!.checked =
      config.autostart;
    body.querySelector<HTMLInputElement>("#paused")!.checked = config.paused;
    body.querySelector("#companion-status")!.textContent =
      EXPRESSIONS[config.expression as keyof typeof EXPRESSIONS]?.label ??
      "平静";
    body.querySelector("#current-background")!.textContent = activeName();
  }
  async function update(patch: Partial<Settings>) {
    if (!desktop) {
      settingsApply({ ...config, ...patch });
      return;
    }
    const next = { ...config, ...patch };
    settingsApply(
      await invoke<Settings>("update_settings", {
        expression: next.expression,
        size: next.size,
        theme: next.theme,
        paused: next.paused,
        fps: next.fps,
      }),
    );
  }
  function renderPet() {
    body.innerHTML = `<div class="pet-layout"><section class="companion-card"><div class="card-top"><span class="soft-pill"><i></i>互动预览</span><span class="pet-nickname">小伴</span></div><div class="pet-preview"><div class="preview-orbit orbit-one"></div><div class="preview-orbit orbit-two"></div><div class="preview-floor"></div><div id="preview-pet"></div><span class="pet-mood" id="companion-status">平静</span></div><div class="preview-caption"><h2>今天，也有小伴陪着你。</h2><p>移动鼠标，它会望向你；戳一下，看看它的反应。</p><button class="poke-button" id="poke">${icon("heart")}戳戳小伴</button></div><div class="preview-details"><span><i></i>当前背景：<b id="current-background"></b></span><button id="go-wallpapers">更换壁纸${icon("arrow")}</button></div></section><div class="pet-settings"><section class="settings-card"><div class="section-title"><h2>陪伴方式</h2><span>在桌面上，找到舒服的位置</span></div><div class="placement-options"><button data-mode="floating"><span class="placement-art float-art"><i></i></span><b>悬浮桌宠</b><small>自由拖动，随时陪伴</small><em>${icon("check")}</em></button><button data-mode="wallpaper"><span class="placement-art wallpaper-art"><i></i></span><b>融入壁纸</b><small>住进背景，安静相伴</small><em>${icon("check")}</em></button></div></section><section class="settings-card"><div class="section-title"><h2>小伴的模样</h2><span>心情和大小，都由你决定</span></div><div class="setting-row"><label for="expression">小伴的心情</label><select id="expression">${EXPRESSION_NAMES.map((name) => `<option value="${name}">${esc(EXPRESSIONS[name].label)}</option>`).join("")}</select></div><div class="range-setting"><div><label for="size">角色大小</label><b id="size-value"></b></div><input id="size" type="range" min="160" max="420" step="10"><div class="range-labels"><span>小巧</span><span>醒目</span></div></div><div class="setting-row"><span>小伴房间</span><div class="theme-options"><button data-theme="night"><i class="night-swatch"></i>深夜</button><button data-theme="dawn"><i class="dawn-swatch"></i>晨光</button></div></div></section><section class="settings-card"><div class="section-title"><h2>日常偏好</h2><span>让陪伴刚刚好</span></div><label class="setting-row switch-row" for="autostart"><span><b>开机自动启动</b><small id="startup-hint">登录电脑时，让小伴自动出现</small></span><input id="autostart" type="checkbox" ${startupAvailable ? "" : "disabled"}></label><div class="setting-row"><span>动画流畅度</span><div class="segmented"><button data-fps="30">30 帧</button><button data-fps="60">60 帧</button></div></div><label class="setting-row switch-row" for="paused"><span><b>暂停动画</b><small>想安静一下时，让桌面也慢下来</small></span><input id="paused" type="checkbox"></label></section></div></div>`;
    pet = new PetView(body.querySelector<HTMLElement>("#preview-pet")!, {
      ...config,
      size: 280,
    });
    pet.host.addEventListener("pointerdown", () => pet?.poke());
    body
      .querySelector("#poke")!
      .addEventListener("click", () =>
        desktop ? task(() => invoke("poke")) : pet?.poke(),
      );
    body
      .querySelector("#go-wallpapers")!
      .addEventListener("click", () => navigate("wallpaper"));
    body.querySelectorAll<HTMLButtonElement>("[data-mode]").forEach((b) =>
      b.addEventListener("click", () =>
        task(async () => {
          if (!desktop) {
            toast("请在桌面应用中切换陪伴方式");
            return;
          }
          b.disabled = true;
          try {
            settingsApply(
              await invoke<Settings>("set_mode", { mode: b.dataset.mode }),
            );
            toast("陪伴方式已更新");
          } finally {
            b.disabled = false;
          }
        }),
      ),
    );
    body
      .querySelector<HTMLSelectElement>("#expression")!
      .addEventListener("change", (e) =>
        task(() =>
          update({ expression: (e.target as HTMLSelectElement).value }),
        ),
      );
    body
      .querySelector<HTMLInputElement>("#size")!
      .addEventListener(
        "input",
        (e) =>
          (body.querySelector("#size-value")!.textContent =
            `${(e.target as HTMLInputElement).value}px`),
      );
    body
      .querySelector<HTMLInputElement>("#size")!
      .addEventListener("change", (e) =>
        task(() =>
          update({ size: Number((e.target as HTMLInputElement).value) }),
        ),
      );
    body
      .querySelectorAll<HTMLButtonElement>("[data-theme]")
      .forEach((b) =>
        b.addEventListener("click", () =>
          task(() => update({ theme: b.dataset.theme as Settings["theme"] })),
        ),
      );
    body
      .querySelectorAll<HTMLButtonElement>("[data-fps]")
      .forEach((b) =>
        b.addEventListener("click", () =>
          task(() => update({ fps: Number(b.dataset.fps) as 30 | 60 })),
        ),
      );
    body
      .querySelector<HTMLInputElement>("#paused")!
      .addEventListener("change", (e) =>
        task(() => update({ paused: (e.target as HTMLInputElement).checked })),
      );
    body
      .querySelector<HTMLInputElement>("#autostart")!
      .addEventListener("change", (e) =>
        task(async () => {
          const input = e.target as HTMLInputElement;
          input.disabled = true;
          try {
            settingsApply(
              await invoke<Settings>("set_autostart", {
                enabled: input.checked,
              }),
            );
          } catch (error) {
            settingsApply(await invoke<Settings>("get_settings"));
            throw error;
          } finally {
            input.disabled = false;
          }
        }),
      );
    if (!startupAvailable)
      body.querySelector("#startup-hint")!.textContent =
        "请在正式桌面应用中设置";
    settingsApply(config);
  }
  function pick(id: string) {
    chosen = id;
    draft = clone(
      id === config.wallpaperId ? config.wallpaperOptions : wallpaperDefaults,
    );
    for (const c of current()?.definition.controls ?? [])
      draft.values[c.key] ??= c.default;
    renderWallpapers();
  }
  async function importLocal() {
    if (!desktop) { toast("请在桌面应用中导入图片或视频"); return; }
    if (busy) return;
    busy = true;
    const button = body.querySelector<HTMLButtonElement>("#import-local")!;
    button.disabled = true;
    button.textContent = "正在导入…";
    try {
      const result = await invoke<{ id: string; library: WallpaperLibrary } | null>("import_local_wallpaper");
      if (result) {
        library = result.library;
        filter = "local";
        chosen = result.id;
        draft = clone(wallpaperDefaults);
        navigate("wallpaper");
        toast("已保存到我的壁纸，可以预览或应用到桌面");
      }
    } finally {
      busy = false;
      if (button.isConnected) { button.disabled = false; button.innerHTML = `${icon("folder")}导入图片 / 视频`; }
    }
  }
  function renderWallpapers() {
    const entries = library.items.filter(
      (entry) => filter === "all" || (filter === "local" ? entry.source === "local" : entry.definition.kind === filter),
    );
    body.innerHTML = `<div class="wallpaper-toolbar"><div class="filter-tabs">${[
      ["all", "全部"],
      ["image", "图片"],
      ["video", "视频"],
      ["canvas", "动画"],
      ["local", "我的导入"],
    ]
      .map(
        ([id, label]) =>
          `<button data-filter="${id}" class="${id === filter ? "selected" : ""}">${label}</button>`,
      )
      .join(
        "",
      )}</div><div class="toolbar-actions"><button class="icon-button" id="refresh-library" title="刷新订阅">${icon("refresh")}</button><button class="icon-button" id="import-manifest" title="导入 JSON 清单" aria-label="导入 JSON 清单">${icon("folder")}</button><button class="secondary-button" id="subscribe">${icon("plus")}添加订阅</button><button class="primary-button" id="import-local">${icon("folder")}导入图片 / 视频</button></div></div><div class="wallpaper-layout"><div class="wallpaper-gallery">${filter === "all" ? `<button class="wallpaper-card ${chosen === "companion" ? "selected" : ""}" data-wallpaper="companion"><div class="wallpaper-cover room-cover"><span class="cover-window"></span><span class="cover-moon"></span><span class="cover-desk"></span><span class="wallpaper-type">默认</span>${chosen === "companion" ? `<span class="selection-check">${icon("check")}</span>` : ""}</div><div class="wallpaper-card-info"><b>小伴房间</b><small>跟随小伴的陪伴方式</small></div></button>` : ""}${entries.map((entry) => `<button class="wallpaper-card ${chosen === entry.definition.id ? "selected" : ""}" data-wallpaper="${esc(entry.definition.id)}"><div class="wallpaper-cover ${entry.definition.kind}">${entry.coverUrl ? `<img src="${esc(entry.coverUrl)}" alt="${esc(entry.definition.title)}">` : '<span class="fallback-art">✧</span>'}<span class="wallpaper-type">${kindLabel(entry.definition.kind)}</span>${chosen === entry.definition.id ? `<span class="selection-check">${icon("check")}</span>` : ""}${entry.definition.kind === "video" ? `<span class="video-mark">${icon("play")}</span>` : ""}</div><div class="wallpaper-card-info"><b>${esc(entry.definition.title)}</b><small>${esc(entry.definition.author || "壁纸提供方")}<em>${entry.installed ? "本地可用" : "可下载"}</em></small></div></button>`).join("")}${entries.length === 0 ? '<div class="empty-gallery">还没有这一类壁纸，导入喜欢的图片、视频，或添加壁纸订阅。</div>' : ""}<div class="gallery-note">${icon("download")}导入会保存原文件副本；推荐 H.264 MP4，壁纸可离线使用。</div></div><aside class="wallpaper-inspector" id="wallpaper-inspector"></aside></div>`;
    body.querySelectorAll<HTMLButtonElement>("[data-filter]").forEach((b) =>
      b.addEventListener("click", () => {
        filter = b.dataset.filter!;
        renderWallpapers();
      }),
    );
    body
      .querySelectorAll<HTMLButtonElement>("[data-wallpaper]")
      .forEach((b) =>
        b.addEventListener("click", () => pick(b.dataset.wallpaper!)),
      );
    body
      .querySelector("#subscribe")!
      .addEventListener("click", () => sourceDialog("catalog"));
    body.querySelector("#import-local")!.addEventListener("click", () => task(importLocal));
    body
      .querySelector("#import-manifest")!
      .addEventListener("click", () =>
        document.querySelector<HTMLInputElement>("#manifest-file")!.click(),
      );
    body
      .querySelector("#refresh-library")!
      .addEventListener("click", () =>
        library.source
          ? task(() => subscribe(library.source))
          : toast("添加订阅后，可在这里同步新壁纸"),
      );
    const inspector = body.querySelector<HTMLElement>("#wallpaper-inspector")!,
      entry = current(),
      item = entry?.definition;
    if (!item) {
      inspector.innerHTML = `<span class="inspector-eyebrow">默认背景</span><h2>小伴房间</h2><p>小伴悬浮时保留系统桌面；融入壁纸时，住进温暖的小房间。</p><div class="inspector-tip">房间氛围可在“小伴设置”中调整。</div><button class="primary-button wide" id="apply-wallpaper">${icon("check")}使用默认背景</button>`;
      inspector
        .querySelector("#apply-wallpaper")!
        .addEventListener("click", () => task(() => apply(false)));
      return;
    }
    inspector.innerHTML = `<span class="inspector-eyebrow">${kindLabel(item.kind)}壁纸</span><h2>${esc(item.title)}</h2><p>${esc(item.description || "为桌面留下一点喜欢的风景。")}</p><div class="inspector-meta"><span>来自 ${esc(item.author || "壁纸提供方")}</span><span>v${esc(item.version)}</span></div><div class="inspector-divider"></div>${item.kind !== "canvas" ? `<div class="compact-field"><label for="fit">画面适配</label><select id="fit"><option value="cover">铺满屏幕</option><option value="contain">完整显示</option><option value="fill">拉伸适配</option></select></div>` : ""}${item.kind === "video" ? `<div class="compact-field"><label for="playback">播放速度 <span id="playback-value">${draft.playbackRate}×</span></label><input id="playback" type="range" min="0.25" max="2" step="0.25" value="${draft.playbackRate}"><small>静音循环播放，让画面自然流动。</small></div>` : ""}${(item.controls ?? []).map((c) => `<div class="compact-field"><label for="property-${esc(c.key)}">${esc(c.label)}${c.type === "range" ? `<span id="value-${esc(c.key)}">${esc(draft.values[c.key])}</span>` : ""}</label>${c.type === "range" ? `<input data-property="${esc(c.key)}" id="property-${esc(c.key)}" type="range" min="${c.min}" max="${c.max}" step="${c.step}" value="${esc(draft.values[c.key])}">` : c.type === "color" ? `<input data-property="${esc(c.key)}" id="property-${esc(c.key)}" type="color" value="${esc(draft.values[c.key])}">` : c.type === "boolean" ? `<input data-property="${esc(c.key)}" id="property-${esc(c.key)}" type="checkbox" ${draft.values[c.key] ? "checked" : ""}>` : `<select data-property="${esc(c.key)}" id="property-${esc(c.key)}">${(c.options ?? []).map((v) => `<option ${draft.values[c.key] === v ? "selected" : ""}>${esc(v)}</option>`).join("")}</select>`}</div>`).join("")}<div class="wallpaper-actions"><button class="secondary-button wide" id="preview-wallpaper">${icon("play")}单独预览</button><button class="primary-button wide" id="apply-wallpaper">${icon("check")}${chosen === config.wallpaperId ? "更新壁纸设置" : entry!.installed ? "应用到桌面" : "下载并应用"}</button></div><div class="download-progress" id="download-progress" hidden><span></span><progress max="1" value="0"></progress></div><div class="inspector-tip">${item.kind === "canvas" ? "动画在独立画面中运行，不连接网络。" : "保留桌面图标的正常操作。"}${config.mode === "floating" ? " 小伴仍会悬浮陪伴。" : " 小伴会融入这张壁纸。"}</div>`;
    const fit = inspector.querySelector<HTMLSelectElement>("#fit");
    if (fit) {
      fit.value = draft.fit;
      fit.addEventListener(
        "change",
        () => (draft.fit = fit.value as WallpaperOptions["fit"]),
      );
    }
    inspector
      .querySelector<HTMLInputElement>("#playback")
      ?.addEventListener("input", (e) => {
        draft.playbackRate = Number((e.target as HTMLInputElement).value);
        inspector.querySelector("#playback-value")!.textContent =
          `${draft.playbackRate}×`;
      });
    inspector
      .querySelectorAll<HTMLInputElement | HTMLSelectElement>("[data-property]")
      .forEach((input) =>
        input.addEventListener("input", () => {
          const c = item.controls!.find(
            (c) => c.key === input.dataset.property,
          )!;
          draft.values[c.key] =
            c.type === "range"
              ? Number(input.value)
              : c.type === "boolean"
                ? (input as HTMLInputElement).checked
                : input.value;
          const value = inspector.querySelector(`#value-${c.key}`);
          if (value) value.textContent = String(draft.values[c.key]);
        }),
      );
    inspector
      .querySelector("#preview-wallpaper")!
      .addEventListener("click", () => task(() => apply(true)));
    inspector
      .querySelector("#apply-wallpaper")!
      .addEventListener("click", () => task(() => apply(false)));
  }
  async function apply(preview: boolean) {
    if (!desktop) {
      toast("请在桌面应用中预览和应用壁纸");
      return;
    }
    if (busy) return;
    busy = true;
    body
      .querySelectorAll<HTMLButtonElement>(
        ".wallpaper-actions button,#apply-wallpaper",
      )
      .forEach((b) => (b.disabled = true));
    try {
      const result = await invoke<Settings>("apply_wallpaper", {
        id: chosen,
        options: draft,
        preview,
      });
      if (!preview) {
        settingsApply(result);
        library = await invoke<WallpaperLibrary>("get_wallpaper_library");
        toast("壁纸已应用到桌面");
        renderWallpapers();
      } else toast("壁纸预览已打开");
    } finally {
      busy = false;
      body
        .querySelectorAll<HTMLButtonElement>(
          ".wallpaper-actions button,#apply-wallpaper",
        )
        .forEach((b) => (b.disabled = false));
    }
  }
  async function subscribe(url: string) {
    if (!desktop) throw new Error("请在桌面应用中添加订阅");
    library = await invoke<WallpaperLibrary>("subscribe_wallpapers", { url });
    library.source = url;
    config = await invoke<Settings>("get_settings");
    renderWallpapers();
    toast(`已同步 ${library.items.filter((entry) => entry.source === "subscription").length} 张订阅壁纸`);
  }
  function renderAbout() {
    const platform =
      info.platform === "windows"
        ? "Windows"
        : info.platform === "macos"
          ? "macOS"
          : "浏览器预览";
    body.innerHTML = `<section class="about-hero"><div class="about-art"><div class="about-orbit"></div><div class="about-mark">${icon("pet")}</div><span class="about-star one">✧</span><span class="about-star two">✦</span></div><span class="soft-pill">陪伴，就在桌面上</span><h2>小伴桌面</h2><p>让日常多一点温暖，让桌面多一点喜欢。</p><span class="version-pill">v${esc(info.version)}<i></i>${platform}${info.arch === "aarch64" ? " · Apple 芯片" : info.arch === "x86_64" ? " · 64 位" : ""}</span></section><div class="about-features"><article>${icon("heart")}<h3>随时相伴</h3><p>会回应的小伙伴，安静陪你工作与休息。</p></article><article>${icon("wallpaper")}<h3>桌面里的风景</h3><p>图片、视频与流动的动画，换个心情再出发。</p></article><article>${icon("pet")}<h3>轻盈，不打扰</h3><p>无须注册账号，下载后的壁纸也能离线使用。</p></article></div><section class="update-card"><div class="update-icon">${icon("refresh")}</div><div class="update-copy"><h3>${lastUpdate?.available ? `发现新版本 v${esc(lastUpdate.version)}` : lastUpdate ? "当前版本已是最新" : "让小伴保持新鲜"}</h3><p>${lastUpdate?.available ? "新版本已经准备好了，看看有哪些变化。" : config.updateUrl ? "手动检查更新，继续享受新的陪伴。" : "更新服务地址稍后接入；当前版本已展示在上方。"}</p></div><button class="primary-button" id="check-update">${icon("refresh")}检查更新</button><button class="icon-button" id="update-settings" title="更新服务设置">${icon("settings")}</button></section>${lastUpdate?.available ? `<section class="release-card"><h3>这次更新</h3><p>${esc(lastUpdate.publishedAt ?? "")}</p><pre>${esc(lastUpdate.notes || "新版本体验优化。")}</pre><button class="primary-button" id="download-update">${icon("download")}前往下载</button></section>` : ""}<div class="about-footer"><span>认真陪伴，慢慢变好。</span><span>小伴桌面 · ${esc(info.version)}</span></div>`;
    if (info.dataDirectory) {
      body.insertAdjacentHTML("beforeend", `<section class="data-location"><div><b>用户数据目录</b><code title="${esc(info.dataDirectory)}">${esc(info.dataDirectory)}</code><small>设置、导入的壁纸与缓存保存在这里。</small></div><button class="secondary-button" id="open-data">${icon("folder")}打开目录</button></section>`);
      body.querySelector("#open-data")!.addEventListener("click", () => task(() => invoke("open_data_directory")));
    }
    body
      .querySelector("#update-settings")!
      .addEventListener("click", () => sourceDialog("update"));
    body.querySelector("#check-update")!.addEventListener("click", () =>
      task(async () => {
        if (!desktop) {
          toast("请在桌面应用中检查更新");
          return;
        }
        if (!config.updateUrl) {
          sourceDialog("update");
          return;
        }
        const button = body.querySelector<HTMLButtonElement>("#check-update")!;
        button.disabled = true;
        button.textContent = "正在检查…";
        try {
          lastUpdate = await invoke("check_updates");
          renderAbout();
        } finally {
          if (button.isConnected) {
            button.disabled = false;
            button.innerHTML = `${icon("refresh")}检查更新`;
          }
        }
      }),
    );
    body
      .querySelector("#download-update")
      ?.addEventListener("click", () =>
        task(() =>
          invoke("open_download_url", { url: lastUpdate!.downloadUrl }),
        ),
      );
  }
  function sourceDialog(type: "catalog" | "update") {
    const host = document.querySelector<HTMLElement>("#modal-root")!;
    host.innerHTML = `<div class="modal-backdrop"><section class="dialog" role="dialog" aria-modal="true" aria-labelledby="dialog-title"><button class="dialog-close icon-button" aria-label="关闭">${icon("close")}</button><div class="dialog-symbol">${icon(type === "catalog" ? "wallpaper" : "refresh")}</div><h2 id="dialog-title">${type === "catalog" ? "添加壁纸订阅" : "更新服务设置"}</h2><p>${type === "catalog" ? "使用壁纸提供方给你的订阅地址，将喜欢的风景收进本地壁纸库。" : "更新接口接入后，就可以在这里检查新版本。"}</p><form><label for="source-url">${type === "catalog" ? "订阅地址" : "更新服务地址"}</label><input id="source-url" type="url" placeholder="https://…" value="${esc(type === "catalog" ? library.source : config.updateUrl)}" ${type === "catalog" ? "required" : ""}><div class="dialog-error" role="alert" hidden></div><div class="dialog-actions"><button class="secondary-button" type="button" id="cancel-dialog">取消</button><button class="primary-button" type="submit">${type === "catalog" ? "添加并同步" : "保存设置"}</button></div></form></section></div>`;
    const close = () => (host.innerHTML = "");
    host.querySelector(".dialog-close")!.addEventListener("click", close);
    host.querySelector("#cancel-dialog")!.addEventListener("click", close);
    host.querySelector("form")!.addEventListener("submit", (event) => {
      event.preventDefault();
      task(async () => {
        const button = host.querySelector<HTMLButtonElement>("[type=submit]")!,
          url = host
            .querySelector<HTMLInputElement>("#source-url")!
            .value.trim();
        button.disabled = true;
        button.textContent = type === "catalog" ? "正在同步…" : "正在保存…";
        try {
          if (type === "catalog") await subscribe(url);
          else {
            if (!desktop) throw new Error("请在桌面应用中配置更新服务");
            config = await invoke<Settings>("set_update_source", { url });
            lastUpdate = null;
            renderAbout();
            toast("更新服务已保存");
          }
          close();
        } catch (error) {
          const view = host.querySelector<HTMLElement>(".dialog-error")!;
          view.hidden = false;
          view.textContent = String(error);
          button.disabled = false;
          button.textContent = type === "catalog" ? "添加并同步" : "保存设置";
        }
      });
    });
    host.querySelector<HTMLInputElement>("#source-url")!.focus();
  }
  function navigate(next: string) {
    pet?.destroy();
    pet = null;
    page = next;
    document.querySelectorAll<HTMLElement>("[data-page]").forEach((b) => {
      const selected = b.dataset.page === page;
      b.classList.toggle("selected", selected);
      b.setAttribute("aria-current", selected ? "page" : "false");
    });
    const text =
      page === "pet"
        ? [
            "COMPANION",
            "小伴设置",
            "选一种喜欢的陪伴方式，把日常交给一点温暖。",
          ]
        : page === "wallpaper"
          ? ["", "动态壁纸", ""]
          : ["ABOUT", "关于小伴", "认识你的桌面小伙伴，一起慢慢变好。"];
    const eyebrow = document.querySelector<HTMLElement>("#eyebrow")!;
    eyebrow.textContent = text[0];
    eyebrow.hidden = !text[0];
    document.querySelector("#page-title")!.textContent = text[1];
    const description = document.querySelector<HTMLElement>("#page-description")!;
    description.textContent = text[2];
    description.hidden = !text[2];
    document.querySelector(".page-heading")!.classList.toggle("compact", page === "wallpaper");
    if (page === "pet") renderPet();
    else if (page === "wallpaper") renderWallpapers();
    else renderAbout();
  }
  document
    .querySelectorAll<HTMLButtonElement>("[data-page]")
    .forEach((b) =>
      b.addEventListener("click", () => navigate(b.dataset.page!)),
    );
  document
    .querySelector("#hide")!
    .addEventListener("click", () =>
      desktop
        ? task(() => invoke("hide_control"))
        : toast("请在桌面应用中收起到托盘"),
    );
  document
    .querySelector("#quit")!
    .addEventListener("click", () =>
      desktop ? task(() => invoke("quit")) : toast("这是浏览器预览"),
    );
  document.querySelector("#sidebar-version")!.textContent =
    `v${info.version} · ${info.platform === "windows" ? "Windows" : info.platform === "macos" ? "macOS" : "预览模式"}`;
  document
    .querySelector<HTMLInputElement>("#manifest-file")!
    .addEventListener("change", (event) =>
      task(async () => {
        const input = event.target as HTMLInputElement,
          file = input.files?.[0];
        if (!file) return;
        if (!desktop) {
          toast("请在桌面应用中导入壁纸清单");
          input.value = "";
          return;
        }
        try {
          if (file.size > 1024 * 1024) throw new Error("清单不能超过 1 MB");
          library = await invoke<WallpaperLibrary>("import_wallpaper_catalog", {
            text: await file.text(),
          });
          renderWallpapers();
          toast("壁纸清单已加入本地库");
        } finally {
          input.value = "";
        }
      }),
    );
  window.addEventListener("pointermove", (event) =>
    pet?.look(event.clientX, event.clientY),
  );
  window.addEventListener("focus", () => {
    if (desktop)
      invoke<Settings>("get_settings").then(settingsApply).catch(failure);
  });
  await listen<Settings>("settings-changed", settingsApply);
  await listen("pet-poke", () => pet?.poke());
  await listen<string>("desktop-error", failure);
  await listen<{ id: string; received: number; total: number }>(
    "wallpaper-progress",
    (progress) => {
      const view = body.querySelector<HTMLElement>("#download-progress");
      if (view && progress.id === chosen) {
        view.hidden = false;
        view.querySelector("span")!.textContent =
          `正在下载 ${Math.round((progress.received / progress.total) * 100)}%`;
        view.querySelector("progress")!.value =
          progress.received / progress.total;
      }
    },
  );
  navigate("pet");
  settingsApply(config);
  if (desktop) {
    const status = await invoke<{ error: string | null }>("get_startup_status");
    if (status.error) failure(status.error);
  }
  window.addEventListener("beforeunload", () => pet?.destroy());
}
