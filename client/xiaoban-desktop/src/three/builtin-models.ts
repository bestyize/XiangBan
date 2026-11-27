import models from '../../builtin-models.json';

export function cachedModelUrl(sha256: string) {
  const origin = location.protocol === 'tauri:' ? 'wallpaper-asset://localhost' : 'http://wallpaper-asset.localhost';
  return `${origin}/${sha256}`;
}

export async function checkedModelBytes(response: Response, model: { bytes: number; sha256: string }) {
  if (!response.ok) throw new Error('模型下载失败');
  const bytes = await response.arrayBuffer();
  if (bytes.byteLength !== model.bytes) throw new Error('模型大小校验失败');
  const hash = Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', bytes)), value => value.toString(16).padStart(2, '0')).join('');
  if (hash !== model.sha256) throw new Error('模型校验失败');
  return bytes;
}

export async function loadBuiltinModel(id: keyof typeof models, signal?: AbortSignal) {
  const { file, resource } = models[id];
  if (import.meta.env.DEV) {
    const response = await fetch(`/${file}`, { signal });
    if (!response.ok) throw new Error('模型读取失败');
    return response.arrayBuffer();
  }
  const url = window.__TAURI__ ? cachedModelUrl(resource.sha256) : resource.url;
  return checkedModelBytes(await fetch(url, { signal }), resource);
}
