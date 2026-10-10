export interface SiteWallpaper {
  id: string;
  title: string;
  description?: string;
  desktop_id?: string;
  downloaded?: boolean;
  media_type: string;
  preview_url: string;
  downloads: { thumb: string; original?: string; large?: string; medium?: string };
  author: { name: string };
  resolution?: { width: number; height: number };
  video?: {
    preview_url: string;
    duration_secs?: number;
    qualities: { url: string; width: number; height: number }[];
  };
}
export const escapeHtml = (value: unknown) =>
  String(value ?? "").replace(
    /[&<>"']/g,
    (c) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[
        c
      ]!,
  );
export function mediaUrl(value: string | undefined, site: string): string {
  if (!value) return "";
  try {
    const url = new URL(value, site);
    if (url.username || url.password) return "";
    return url.protocol === "https:" ||
      (url.protocol === "http:" &&
        ["localhost", "127.0.0.1", "[::1]"].includes(url.hostname))
      ? url.href
      : "";
  } catch {
    return "";
  }
}
export function siteMedia(item: SiteWallpaper, site: string) {
  const image = [
    item.downloads.thumb,
    item.preview_url,
    ...(item.media_type === "image" ? [item.downloads.original] : []),
  ].find((value) => value && !/\.(mp4|webm|m4v)(?:[?#]|$)/i.test(value));
  const poster = mediaUrl(image, site);
  const qualities = [...(item.video?.qualities ?? [])].sort(
    (a, b) => a.width - b.width,
  );
  const video =
    item.media_type === "video"
      ? ([
          item.video?.preview_url,
          ...qualities.map((q) => q.url),
          item.downloads.original,
        ]
          .map((url) => mediaUrl(url, site))
          .find(Boolean) ?? "")
      : "";
  return { poster, video };
}
/** Widest video rendition still shown at full size; 4K files are too heavy for a preview. */
const FULL_VIDEO_WIDTH = 2560;
/**
 * Media for viewing a website wallpaper full screen: the large rendition instead of the card
 * thumbnail, and the sharpest video up to `FULL_VIDEO_WIDTH`. Falls back to `siteMedia`.
 */
export function siteFullMedia(item: SiteWallpaper, site: string) {
  const preview = siteMedia(item, site);
  if (item.media_type === "image") {
    const image = [item.downloads.large, item.downloads.original, item.downloads.medium]
      .map((url) => mediaUrl(url, site))
      .find(Boolean);
    return { poster: image || preview.poster, video: "" };
  }
  if (item.media_type === "video") {
    const sharp = [...(item.video?.qualities ?? [])]
      .filter((q) => q.width > 0 && q.width <= FULL_VIDEO_WIDTH)
      .sort((a, b) => b.width - a.width)
      .map((q) => mediaUrl(q.url, site))
      .find(Boolean);
    return { poster: preview.poster, video: sharp || preview.video };
  }
  return preview;
}
export function mediaMarkup(
  kind: string,
  source: string,
  poster: string,
  title: string,
  controls = false,
) {
  if (kind === "video" && source)
    return `<video data-video-source="${escapeHtml(source)}" ${poster ? `poster="${escapeHtml(poster)}"` : ""} muted playsinline loop preload="none" ${controls ? "controls" : ""} disablepictureinpicture aria-label="${escapeHtml(title)}"></video><span class="media-status">加载视频画面…</span>`;
  if (poster || (kind === "image" && source))
    return `<img src="${escapeHtml(poster || source)}" alt="${escapeHtml(title)}" loading="lazy"><span class="media-status" hidden>图片暂不可用</span>`;
  return '<span class="fallback-art">✧</span><span class="media-status">暂无预览</span>';
}
// Only visible cards request metadata/first frame. Hover playback stays muted and pauses on exit.
// The inspector has controls; it never starts a full-quality download when merely selected.
export function bindMedia(root: HTMLElement): () => void {
  const videos = [
    ...root.querySelectorAll<HTMLVideoElement>("video[data-video-source]"),
  ];
  const load = (video: HTMLVideoElement) => {
    if (!video.getAttribute("src")) {
      video.src = video.dataset.videoSource!;
      video.preload = "metadata";
      video.load();
    }
  };
  const observer = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        const video = entry.target as HTMLVideoElement;
        if (entry.isIntersecting && (!video.poster || video.controls))
          load(video);
        if (!entry.isIntersecting) video.pause();
      }
    },
    { root: root.closest("#page-body"), rootMargin: "160px" },
  );
  for (const video of videos) {
    const state =
      video.parentElement!.querySelector<HTMLElement>(".media-status");
    if (video.poster && state) state.hidden = true;
    video.addEventListener("loadedmetadata", () => {
      if (
        !video.poster &&
        Number.isFinite(video.duration) &&
        video.duration > 0
      )
        video.currentTime = Math.min(0.001, video.duration / 2);
    });
    video.addEventListener("loadeddata", () => {
      video.dataset.ready = "true";
      if (state) state.hidden = true;
    });
    video.addEventListener("error", () => {
      if (state) {
        state.hidden = false;
        state.textContent = "视频预览暂不可用，仍可下载后预览";
      }
    });
    const card = video.closest("button");
    card?.addEventListener("mouseenter", () => {
      load(video);
      void video.play().catch(() => {});
    });
    card?.addEventListener("mouseleave", () => video.pause());
    observer.observe(video);
  }
  root.querySelectorAll<HTMLImageElement>("img").forEach((image) =>
    image.addEventListener("error", () => {
      image.hidden = true;
      const state =
        image.parentElement!.querySelector<HTMLElement>(".media-status");
      if (state) state.hidden = false;
    }),
  );
  return () => {
    observer.disconnect();
    for (const video of videos) {
      video.pause();
      video.removeAttribute("src");
      video.load();
    }
  };
}
