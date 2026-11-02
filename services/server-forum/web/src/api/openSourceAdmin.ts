import { request, adminAuth } from './admin'
import type { OpenSourceCatalog, OpenSourceProject } from '@/lib/openSource'
import { openSourceCoverRules } from '@/lib/openSourceCovers'

export interface OpenSourceWorkspace {
  scope: 'open-source'; revision: number; published_revision: number
  draft: OpenSourceCatalog; live: OpenSourceCatalog; cover_rules: typeof openSourceCoverRules
}
export interface AssetCandidate { url: string; label: string; role: 'logo' | 'screenshot' | 'image'; source: string }
export interface CuratedAsset { id: string; url: string; source_url: string; mime: string; size: number }
const root = '/open-source'
export const openSourceAdmin = {
  read: () => request<OpenSourceWorkspace>('GET', root),
  save: (revision: number, data: OpenSourceCatalog) => request<OpenSourceWorkspace>('PUT', root, { expected_revision: revision, data }),
  preview: (data: OpenSourceCatalog) => request<OpenSourceCatalog>('POST', `${root}/preview`, data),
  publish: (revision: number) => request<OpenSourceWorkspace>('POST', `${root}/publish`, { expected_revision: revision }),
  import: (repository: string) => request<{ project: OpenSourceProject; assets: AssetCandidate[]; notes: string[] }>('POST', `${root}/import-github`, { repository }),
  upload: (project: string, source_url: string, data_base64: string) => request<CuratedAsset>('POST', `${root}/assets`, { project, source_url, data_base64 }),
  collectAsset: (project: string, url: string) => request<CuratedAsset>('POST', `${root}/assets/fetch`, { project, url }),
}
export async function privateAssetPreview(url: string): Promise<string> {
  const match = /^\/api\/open-source\/assets\/([\da-f-]+)$/i.exec(url)
  if (!match) return url
  const response = await fetch(`/api/admin/open-source/assets/${match[1]}`, { headers: { Authorization: `Bearer ${adminAuth.token}` } })
  if (!response.ok) throw new Error('草稿素材加载失败，请重新读取')
  return URL.createObjectURL(await response.blob())
}
/** SVG and ICO are decoded by the browser as images; uploads are ordinary bounded raster assets. */
export async function rasterAsset(file: File, icon: boolean): Promise<string> {
  if (file.size > 5 * 1024 * 1024) throw new Error('图片需要在 5 MiB 以内')
  const url = URL.createObjectURL(file)
  try {
    const image = await new Promise<HTMLImageElement>((resolve, reject) => {
      const image = new Image(); image.onload = () => resolve(image); image.onerror = () => reject(new Error('无法读取这张图片')); image.src = url
    })
    if (!image.naturalWidth || !image.naturalHeight || image.naturalWidth * image.naturalHeight > 40_000_000) throw new Error('图片尺寸过大或无效')
    const scale = Math.min(1, (icon ? 512 : 1920) / Math.max(image.naturalWidth, image.naturalHeight))
    const canvas = document.createElement('canvas'); canvas.width = Math.max(1, Math.round(image.naturalWidth * scale)); canvas.height = Math.max(1, Math.round(image.naturalHeight * scale))
    canvas.getContext('2d')!.drawImage(image, 0, 0, canvas.width, canvas.height)
    return canvas.toDataURL(icon ? 'image/png' : 'image/webp', .9).split(',')[1]!
  } finally { URL.revokeObjectURL(url) }
}
