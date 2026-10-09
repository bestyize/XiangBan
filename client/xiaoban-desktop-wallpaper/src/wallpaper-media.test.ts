import { describe, it, expect } from "vitest";
import { siteMedia, mediaUrl, type SiteWallpaper } from "./wallpaper-media";
const video: SiteWallpaper = {
  id: "pixabay_v_284566",
  title: "Desert",
  media_type: "video",
  preview_url: "",
  downloads: { thumb: "", original: "https://cdn.example/full.mp4" },
  author: { name: "Author" },
  video: {
    preview_url: "https://cdn.example/tiny.mp4",
    qualities: [
      { url: "https://cdn.example/full.mp4", width: 3840, height: 2160 },
      { url: "https://cdn.example/small.mp4", width: 640, height: 360 },
    ],
  },
};
describe("website media preview", () => {
  it("shows a video first frame when the website has no image cover", () => {
    expect(siteMedia(video, "https://wallpaper.xiangban.xyz")).toEqual({
      poster: "",
      video: "https://cdn.example/tiny.mp4",
    });
  });
  it("falls back to the smallest stream instead of a full 4K download", () => {
    expect(
      siteMedia(
        { ...video, video: { ...video.video!, preview_url: "" } },
        "https://wallpaper.xiangban.xyz",
      ).video,
    ).toBe("https://cdn.example/small.mp4");
  });
  it("does not treat a custom video upload as an image", () => {
    const media = siteMedia(
      {
        ...video,
        preview_url: "/uploads/test.mp4",
        downloads: { thumb: "/uploads/test.mp4" },
        video: { preview_url: "/uploads/test.mp4", qualities: [] },
      },
      "http://127.0.0.1:8087",
    );
    expect(media.poster).toBe("");
    expect(media.video).toBe("http://127.0.0.1:8087/uploads/test.mp4");
  });
  it("uses a real poster and rejects executable or credential-bearing URLs", () => {
    expect(
      siteMedia(
        { ...video, preview_url: "/uploads/poster.png" },
        "https://wallpaper.xiangban.xyz",
      ).poster,
    ).toBe("https://wallpaper.xiangban.xyz/uploads/poster.png");
    expect(mediaUrl("javascript:alert(1)", "https://example.com")).toBe("");
    expect(
      mediaUrl("https://user:password@example.com/a", "https://example.com"),
    ).toBe("");
  });
});
