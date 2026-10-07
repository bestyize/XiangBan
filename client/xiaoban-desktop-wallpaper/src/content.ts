import {
  desktop,
  invoke,
  type CursorSample,
  type Settings,
  type WallpaperOptions,
  defaults,
} from "./bridge.ts";
import { PetView } from "./pet.ts";

interface Payload {
  item: {
    id: string;
    title: string;
    kind: string;
    resourceUrl: string;
    assets: Record<string, { url: string; mime: string }>;
  };
  options: WallpaperOptions;
  config: {
    expression: string;
    size: number;
    theme: string;
    paused: boolean;
    fps: 30 | 60;
    overlay: boolean;
  };
  test: boolean;
}
declare global {
  interface Window {
    __XIAOBAN_CONTENT__?: Payload;
  }
}

export async function mountContent() {
  let payload = window.__XIAOBAN_CONTENT__;
  if (!payload) {
    const catalog = await fetch("/wallpapers/builtin/catalog.json").then((r) =>
      r.json(),
    );
    const id =
      new URLSearchParams(location.search).get("builtin") ??
      "builtin-mint-orbits";
    const item =
      catalog.items.find((item: { id: string }) => item.id === id) ??
      catalog.items[0];
    payload = {
      item: {
        ...item,
        resourceUrl: item.resource.url.replace(
          "builtin://",
          "/wallpapers/builtin/",
        ),
        assets: {},
      },
      options: defaults.wallpaperOptions,
      config: { ...defaults, overlay: false },
      test: false,
    };
  }
  let state = payload as Payload;
  document.body.className = "content-scene";
  document.body.innerHTML =
    '<main class="content-stage"></main><div class="wallpaper-pet content-pet"></div><div class="content-error" role="alert" hidden></div>';
  const host = document.querySelector<HTMLElement>(".content-stage")!;
  let pet: PetView | null = null,
    video: HTMLVideoElement | null = null,
    frame: HTMLIFrameElement | null = null;
  const update = (detail: {
    options: WallpaperOptions;
    config: Payload["config"];
  }) => {
    state = { ...state, ...detail };
    host.style.background = state.options.background;
    const media = host.querySelector<HTMLImageElement | HTMLVideoElement>(
      "img,video",
    );
    if (media) media.style.objectFit = state.options.fit;
    if (video) {
      video.playbackRate = state.options.playbackRate;
      if (state.config.paused) video.pause();
      else video.play().catch(fail);
    }
    if (frame)
      frame.contentWindow?.postMessage(
        {
          type: "canvas-config",
          config: state.config,
          settings: state.options.values,
        },
        "*",
      );
    document.body.dataset.theme = state.config.theme;
    document.body.dataset.paused = String(state.config.paused);
    const settings = { ...defaults, ...state.config } as Settings;
    if (state.config.overlay) {
      if (!pet)
        pet = new PetView(
          document.querySelector<HTMLElement>(".content-pet")!,
          settings,
        );
      else pet.update(settings);
    } else {
      pet?.destroy();
      pet = null;
    }
  };
  const ready = async (report: Record<string, unknown>) => {
    if (state.test && desktop) {
      let blocked = false;
      try {
        await invoke("set_autostart", { enabled: false });
      } catch {
        blocked = true;
      }
      report.privilegedIpcBlocked = blocked;
    }
    if (desktop)
      await invoke("scene_ready", {
        report: { kind: state.item.kind, ...report },
      });
    else document.body.dataset.ready = "true";
  };
  const fail = (error: unknown) => {
    const message = String(error).slice(0, 2000),
      view = document.querySelector<HTMLElement>(".content-error")!;
    view.hidden = false;
    view.textContent = `壁纸暂时无法加载：${message}`;
    if (desktop)
      invoke("scene_ready", { report: { error: message } }).catch(() => {});
  };
  window.addEventListener("content-config", (event) =>
    update((event as CustomEvent).detail),
  );
  window.addEventListener("content-cursor", (event) => {
    const cursor = (event as CustomEvent<CursorSample>).detail;
    pet?.look(cursor.x, cursor.y, cursor.moved);
    if (cursor.pressed && cursor.desktop && pet?.hit(cursor.x, cursor.y))
      pet.poke();
  });
  window.addEventListener("content-poke", () => pet?.poke());
  try {
    if (state.item.kind === "image") {
      const image = new Image();
      image.src = state.item.resourceUrl;
      host.append(image);
      await image.decode();
      await ready({ width: image.naturalWidth, height: image.naturalHeight });
    } else if (state.item.kind === "video") {
      video = document.createElement("video");
      video.muted = true;
      video.autoplay = true;
      video.loop = true;
      video.playsInline = true;
      video.preload = "auto";
      video.src = state.item.resourceUrl;
      host.append(video);
      await new Promise<void>((resolve, reject) => {
        video!.addEventListener("loadeddata", () => resolve(), { once: true });
        video!.addEventListener(
          "error",
          () => reject(new Error("视频解码失败，请使用 H.264 MP4")),
          { once: true },
        );
      });
      await video.play();
      await ready({ width: video.videoWidth, height: video.videoHeight });
    } else {
      frame = document.createElement("iframe");
      frame.setAttribute("sandbox", "allow-scripts");
      frame.setAttribute("allow", "autoplay");
      frame.title = state.item.title;
      const shell = desktop
        ? state.item.resourceUrl.startsWith("http://wallpaper-asset.localhost/")
          ? "http://wallpaper-asset.localhost/sandbox"
          : location.protocol === "https:" ||
              (location.protocol === "http:" &&
                location.hostname === "tauri.localhost")
            ? "http://wallpaper-asset.localhost/sandbox"
            : "wallpaper-asset://localhost/sandbox"
        : "/wallpapers/sandbox.html";
      // Windows built-in items use the same unprivileged native asset protocol.
      frame.src =
        desktop && navigator.userAgent.includes("Windows")
          ? "http://wallpaper-asset.localhost/sandbox"
          : shell;
      const code = await fetch(state.item.resourceUrl).then((r) => {
        if (!r.ok) throw new Error("动画资源不可用");
        return r.text();
      });
      const assets: Record<string, { bytes: ArrayBuffer; mime: string }> = {};
      for (const [key, asset] of Object.entries(state.item.assets)) {
        assets[key] = {
          bytes: await fetch(asset.url).then((r) => r.arrayBuffer()),
          mime: asset.mime,
        };
      }
      const pending = new Promise<void>((resolve, reject) => {
        const listener = (event: MessageEvent) => {
          if (
            event.source !== frame!.contentWindow ||
            !event.data ||
            typeof event.data !== "object"
          )
            return;
          if (event.data.type === "canvas-shell-ready")
            frame!.contentWindow!.postMessage(
              {
                type: "canvas-init",
                code,
                assets,
                settings: state.options.values,
                config: state.config,
                width: innerWidth,
                height: innerHeight,
                dpr: Math.min(devicePixelRatio, 2),
              },
              "*",
            );
          else if (event.data.type === "canvas-ready") {
            ready({ width: event.data.width, height: event.data.height }).then(
              resolve,
              reject,
            );
          } else if (event.data.type === "canvas-error") {
            fail(event.data.message);
            reject(new Error(event.data.message));
          }
        };
        window.addEventListener("message", listener);
      });
      host.append(frame);
      await pending;
      window.addEventListener("resize", () =>
        frame!.contentWindow?.postMessage(
          {
            type: "canvas-resize",
            width: innerWidth,
            height: innerHeight,
            dpr: Math.min(devicePixelRatio, 2),
          },
          "*",
        ),
      );
    }
    update(state);
  } catch (error) {
    fail(error);
  }
  window.addEventListener("beforeunload", () => {
    pet?.destroy();
    video?.pause();
  });
}
