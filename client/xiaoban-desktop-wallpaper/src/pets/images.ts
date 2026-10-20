// Only inert raster images travel through declarative pet themes. Animated WebP/GIF stay intact.
const DATA = /^data:image\/(png|webp|gif);base64,([A-Za-z0-9+/]+={0,2})$/;
const MAX_IMAGE = 4 * 1024 * 1024;
export function imageSource(value: unknown): string {
  return typeof value === 'string' && value.length <= Math.ceil(MAX_IMAGE / 3) * 4 + 32 && DATA.test(value) ? value : '';
}
export function validateImage(value: unknown): number {
  const source = imageSource(value), match = DATA.exec(source);
  if (!match) throw Error('植物图片需为 PNG、WebP 或 GIF 的内嵌素材，不能包含外链或 SVG');
  const encoded = match[2];
  if (encoded.length % 4) throw Error('植物图片编码无效');
  let header: string;
  try { header = atob(encoded.slice(0, 24)); } catch { throw Error('植物图片编码无效'); }
  const valid = match[1] === 'png' ? header.startsWith('\x89PNG\r\n\x1a\n')
    : match[1] === 'gif' ? /^GIF8[79]a/.test(header)
    : header.startsWith('RIFF') && header.slice(8, 12) === 'WEBP';
  const bytes = encoded.length / 4 * 3 - (encoded.endsWith('==') ? 2 : encoded.endsWith('=') ? 1 : 0);
  if (!valid || bytes > MAX_IMAGE) throw Error('植物图片内容与格式不符或超过 4 MiB');
  return bytes;
}

interface Mask { width: number; height: number; pixels: Uint8ClampedArray }
const masks = new WeakMap<SVGImageElement, Mask | null>();
/** Transparent pixels remain click-through, including while the plant sways with its parent group. */
export function hitImage(element: SVGImageElement, x: number, y: number): boolean {
  if (!masks.has(element)) {
    masks.set(element, null);
    const image = new Image();
    image.onload = () => {
      const scale = Math.min(1, 512 / Math.max(image.naturalWidth, image.naturalHeight));
      const canvas = document.createElement('canvas');
      canvas.width = Math.max(1, Math.round(image.naturalWidth * scale));
      canvas.height = Math.max(1, Math.round(image.naturalHeight * scale));
      const context = canvas.getContext('2d', { willReadFrequently: true });
      if (!context) return;
      context.drawImage(image, 0, 0, canvas.width, canvas.height);
      masks.set(element, { width: canvas.width, height: canvas.height, pixels: context.getImageData(0, 0, canvas.width, canvas.height).data });
    };
    image.src = imageSource(element.getAttribute('href'));
  }
  const mask = masks.get(element), matrix = element.getScreenCTM();
  if (!mask || !matrix || Number(element.getAttribute('opacity') ?? 1) === 0) return false;
  const point = new DOMPoint(x, y).matrixTransform(matrix.inverse());
  const u = (point.x - Number(element.getAttribute('x'))) / Number(element.getAttribute('width'));
  const v = (point.y - Number(element.getAttribute('y'))) / Number(element.getAttribute('height'));
  if (u < 0 || v < 0 || u >= 1 || v >= 1) return false;
  return mask.pixels[(Math.floor(v * mask.height) * mask.width + Math.floor(u * mask.width)) * 4 + 3] > 16;
}
