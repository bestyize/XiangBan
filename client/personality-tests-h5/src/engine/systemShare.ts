export interface SystemSharePayload {
  title: string;
  text: string;
  url: string;
  imageDataUrl?: string;
  filename?: string;
}

export type SystemShareOutcome =
  | { status: 'shared-image'; message: string }
  | { status: 'shared-link'; message: string }
  | { status: 'cancelled'; message: string }
  | { status: 'fallback-copied'; message: string }
  | { status: 'unsupported'; message: string };

export function dataUrlToPngFile(dataUrl: string, filename: string): File | null {
  if (typeof File === 'undefined' || !dataUrl.startsWith('data:')) return null;
  try {
    const parts = dataUrl.split(',');
    const mimeMatch = parts[0].match(/:(.*?);/);
    const mime = mimeMatch ? mimeMatch[1] : 'image/png';
    const binary = atob(parts[1] ?? '');
    const len = binary.length;
    const bytes = new Uint8Array(len);
    for (let i = 0; i < len; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    return new File([bytes], filename, { type: mime });
  } catch {
    return null;
  }
}

export function isSystemShareSupported(): boolean {
  return typeof navigator !== 'undefined' && typeof navigator.share === 'function';
}

/**
 * 统一拉起系统原生分享（Web Share API）：
 * 1. 优先尝试连同高清 PNG 海报一起分享（navigator.canShare({ files })）
 * 2. 降级尝试系统分享标题 + 文案 + 合盘链接
 * 3. 不支持 Web Share 的环境（如部分桌面端或微信内 WebView）自动复制文案与链接并给出清晰提示
 */
export async function triggerSystemShare(
  payload: SystemSharePayload
): Promise<SystemShareOutcome> {
  const nav = typeof navigator !== 'undefined' ? navigator : undefined;

  if (nav && typeof nav.share === 'function') {
    // 1. 尝试带图片文件分享
    if (payload.imageDataUrl) {
      const file = dataUrlToPngFile(
        payload.imageDataUrl,
        payload.filename || 'vendme-poster.png'
      );
      if (file && typeof nav.canShare === 'function') {
        try {
          if (nav.canShare({ files: [file] })) {
            await nav.share({
              files: [file],
              title: payload.title,
              text: payload.text,
              url: payload.url
            });
            return { status: 'shared-image', message: '已调起系统分享高清海报' };
          }
        } catch (err: unknown) {
          if (err instanceof Error && err.name === 'AbortError') {
            return { status: 'cancelled', message: '' };
          }
          // 若设备拒绝 files 分享，继续降级到纯链接系统分享
        }
      }
    }

    // 2. 尝试纯文案 + 链接系统分享
    try {
      await nav.share({
        title: payload.title,
        text: payload.text,
        url: payload.url
      });
      return { status: 'shared-link', message: '已调起系统分享' };
    } catch (err: unknown) {
      if (err instanceof Error && err.name === 'AbortError') {
        return { status: 'cancelled', message: '' };
      }
    }
  }

  // 3. 优雅降级：复制「金句 + 链接」到剪贴板
  const shareText = `${payload.text} ${payload.url}`.trim();
  if (nav?.clipboard && typeof nav.clipboard.writeText === 'function') {
    try {
      await nav.clipboard.writeText(shareText);
      return {
        status: 'fallback-copied',
        message: payload.imageDataUrl
          ? '当前环境不支持直拉系统分享，已复制文案与链接（海报可长按保存转发）'
          : '当前浏览器暂不支持系统分享，已自动复制分享文案与链接'
      };
    } catch {
      // ignore
    }
  }

  return {
    status: 'unsupported',
    message: '当前浏览器暂不支持系统分享，请点击下载 PNG 或复制链接转发'
  };
}
