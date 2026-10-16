import { describe, it, expect } from "vitest";
import { siteMedia, siteFullMedia, mediaUrl, mediaMarkup, type SiteWallpaper } from "./wallpaper-media";
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
describe("full-screen media", () => {
  const site = "https://wallpaper.example";
  const image: SiteWallpaper = {
    id: "bing_x",
    title: "Bay",
    media_type: "image",
    preview_url: "https://cdn.example/regular.jpg",
    downloads: { thumb: "https://cdn.example/thumb.jpg", large: "https://cdn.example/large.jpg", original: "https://cdn.example/uhd.jpg" },
    author: { name: "A" },
  };
  it("shows the large image rather than the card thumbnail", () => {
    expect(siteMedia(image, site).poster).toBe("https://cdn.example/thumb.jpg");
    expect(siteFullMedia(image, site)).toEqual({ poster: "https://cdn.example/large.jpg", video: "" });
    expect(siteFullMedia({ ...image, downloads: { thumb: image.downloads.thumb, original: "/uploads/o.jpg" } }, site).poster).toBe(`${site}/uploads/o.jpg`);
    expect(siteFullMedia({ ...image, downloads: { thumb: image.downloads.thumb, large: "http://insecure.example/l.jpg" } }, site).poster).toBe("https://cdn.example/thumb.jpg");
  });
  it("picks the sharpest video rendition up to 2560 px wide", () => {
    const clip: SiteWallpaper = {
      ...image,
      media_type: "video",
      video: {
        preview_url: "https://cdn.example/tiny.mp4",
        qualities: [
          { url: "https://cdn.example/720.mp4", width: 1280, height: 720 },
          { url: "https://cdn.example/4k.mp4", width: 3840, height: 2160 },
          { url: "https://cdn.example/1440.mp4", width: 2560, height: 1440 },
        ],
      },
    };
    expect(siteFullMedia(clip, site).video).toBe("https://cdn.example/1440.mp4");
    expect(siteFullMedia({ ...clip, video: { preview_url: "https://cdn.example/tiny.mp4", qualities: [] } }, site).video).toBe("https://cdn.example/tiny.mp4");
  });
});


it('renders selected HTML animations in an isolated frame and leaves gallery cards static', () => {
  const selected = mediaMarkup('web', 'wallpaper-asset://localhost/hash', '', '星空', true);
  expect(selected).toContain('<iframe');
  expect(selected).toContain('sandbox="allow-scripts"');
  expect(selected).not.toContain('allow-same-origin');
  expect(selected).not.toContain('<img');
  const card = mediaMarkup('web', 'wallpaper-asset://localhost/hash', '', '星空');
  expect(card).not.toContain('<iframe');
  expect(card).toContain('选择后预览动画');
  expect(mediaMarkup('web', '" onload="bad', '', '<test>', true)).not.toContain('src="" onload=');
});
