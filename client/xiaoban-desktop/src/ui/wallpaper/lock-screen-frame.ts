import { invoke, type WallpaperOptions } from '../../bridge.ts';
import { drawEffects } from '../../effects/draw.ts';
import { ParticleSystem } from '../../effects/sim.ts';
import type { EffectPack, EffectParams } from '../../effects/types.ts';

interface Size { width: number; height: number }
interface Source { url: string; video: boolean }

async function drawSource(source: Source, draw: (media: CanvasImageSource, size: Size) => void) {
  if (!source.video) {
    const image = new Image();
    image.crossOrigin = 'anonymous';
    image.src = source.url;
    await image.decode();
    draw(image, { width: image.naturalWidth, height: image.naturalHeight });
    return;
  }

  const video = document.createElement('video');
  const events = new AbortController();
  video.crossOrigin = 'anonymous';
  video.preload = 'auto';
  video.muted = true;
  let deadline: ReturnType<typeof setTimeout> | undefined;
  try {
    await new Promise<void>((resolve, reject) => {
      const unavailable = () => reject(new Error('无法读取视频首帧'));
      video.addEventListener('loadeddata', () => resolve(), { once: true, signal: events.signal });
      video.addEventListener('error', unavailable, { once: true, signal: events.signal });
      deadline = setTimeout(unavailable, 12000);
      video.src = source.url;
      video.load();
    });
    draw(video, { width: video.videoWidth, height: video.videoHeight });
  } finally {
    clearTimeout(deadline);
    events.abort();
    video.removeAttribute('src');
    video.load();
  }
}

export async function wallpaperFrame(
  id: string,
  options: WallpaperOptions,
  composition?: { size: Size; effect: EffectPack; params: EffectParams },
) {
  const canvas = document.createElement('canvas');
  const context = canvas.getContext('2d');
  if (!context) throw new Error('无法生成锁屏图片');
  const paint = (size: Size, media?: CanvasImageSource, sourceSize = size) => {
    if (![size.width, size.height, sourceSize.width, sourceSize.height].every(n => Number.isFinite(n) && n > 0)) {
      throw new Error('壁纸的画面尺寸无效');
    }
    const shrink = Math.min(1, 4096 / Math.max(size.width, size.height));
    canvas.width = Math.max(1, Math.round(size.width * shrink));
    canvas.height = Math.max(1, Math.round(size.height * shrink));
    context.fillStyle = options.background || '#000';
    context.fillRect(0, 0, canvas.width, canvas.height);
    if (!media) return;
    const xScale = canvas.width / sourceSize.width;
    const yScale = canvas.height / sourceSize.height;
    const scale = options.fit === 'contain' ? Math.min(xScale, yScale) : Math.max(xScale, yScale);
    const width = options.fit === 'fill' ? canvas.width : sourceSize.width * scale;
    const height = options.fit === 'fill' ? canvas.height : sourceSize.height * scale;
    context.drawImage(media, (canvas.width - width) / 2, (canvas.height - height) / 2, width, height);
  };
  if (id && id !== 'companion') {
    const source = await invoke<Source>('lock_screen_wallpaper_source', { id });
    await drawSource(source, (media, size) => paint(composition?.size ?? size, media, size));
  } else if (composition) {
    paint(composition.size);
  } else {
    throw new Error('请先选择一张壁纸');
  }

  if (composition) {
    const particles = document.createElement('canvas');
    particles.width = canvas.width;
    particles.height = canvas.height;
    const layer = particles.getContext('2d');
    if (!layer) throw new Error('无法合成锁屏特效');
    const system = new ParticleSystem(composition.effect);
    system.configure(particles.width, particles.height, composition.params);
    drawEffects(layer, system);
    context.drawImage(particles, 0, 0);
  }
  return canvas.toDataURL('image/jpeg', 0.88);
}
