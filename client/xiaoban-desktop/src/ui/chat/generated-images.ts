export interface GeneratedImage { id: string; name: string; mime: string; width: number; height: number }

/** Only the native image tool's small file references are renderable, never arbitrary Markdown/remote URLs. */
export function generatedImages(value: unknown): GeneratedImage[] {
  if (!value || typeof value !== 'object' || !('generated_images' in value) || !Array.isArray(value.generated_images)) return [];
  return value.generated_images.slice(0, 4).filter((item): item is GeneratedImage => {
    if (!item || typeof item !== 'object') return false;
    return typeof item.id === 'string' && /^[a-f0-9]{64}\.(png|jpg|webp|gif)$/.test(item.id)
      && typeof item.name === 'string' && item.name.length <= 160
      && ['image/png', 'image/jpeg', 'image/webp', 'image/gif'].includes(item.mime)
      && Number.isInteger(item.width) && item.width > 0 && item.width <= 8192
      && Number.isInteger(item.height) && item.height > 0 && item.height <= 8192;
  });
}
