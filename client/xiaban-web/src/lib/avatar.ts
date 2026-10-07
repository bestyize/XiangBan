import { ApiError, getToken, requireUserSession } from '../api/client.ts'
import type { Profile } from '../api/types.ts'

export const MAX_AVATAR_BYTES = 64 * 1024
export async function compressAvatar(file: File): Promise<Blob> {
  if (!/^image\/(jpeg|png|webp|gif)$/.test(file.type)) throw new Error('请选择 JPG、PNG、WebP 或 GIF 图片')
  if (file.size > 10 * 1024 * 1024) throw new Error('原图片不能超过 10MB，请先缩小图片')
  const url = URL.createObjectURL(file)
  const image = new Image()
  try {
    image.src = url
    await image.decode()
    if (!image.naturalWidth || !image.naturalHeight) throw new Error('图片无法读取，请换一张')
    const crop = Math.min(image.naturalWidth, image.naturalHeight)
    for (const size of [256, 192, 128]) {
      const canvas = document.createElement('canvas')
      canvas.width = size; canvas.height = size
      const context = canvas.getContext('2d')
      if (!context) throw new Error('浏览器暂不支持头像压缩')
      context.fillStyle = '#ffffff'; context.fillRect(0, 0, size, size)
      context.drawImage(image, (image.naturalWidth - crop) / 2, (image.naturalHeight - crop) / 2, crop, crop, 0, 0, size, size)
      for (const quality of [0.84, 0.7, 0.55, 0.4, 0.25]) {
        const result = await new Promise<Blob | null>((resolve) => canvas.toBlob(resolve, 'image/jpeg', quality))
        if (result && result.type === 'image/jpeg' && result.size <= MAX_AVATAR_BYTES) return result
      }
    }
    throw new Error('图片仍超过 64KB，请选择更简单的图片')
  } catch (e) {
    if (e instanceof Error && e.name === 'EncodingError') throw new Error('图片格式无法读取，请换一张 JPG 或 PNG')
    throw e
  } finally { URL.revokeObjectURL(url) }
}
export async function uploadAvatar(blob: Blob): Promise<Profile> {
  if (blob.size > MAX_AVATAR_BYTES) throw new Error('头像不能超过 64KB')
    if (blob.type !== 'image/jpeg') throw new Error('请先将头像压缩为 JPEG 图片')
  if (!getToken()) await requireUserSession()
  const token = getToken()
  if (!token) throw new Error('登录状态正在检查，请稍后再试')
  const response = await fetch('/api/account/avatar', { method: 'PUT', headers: { Authorization: `Bearer ${token}`, 'Content-Type': 'image/jpeg' }, body: blob })
  const data = await response.json().catch(() => null)
  if (!response.ok) throw new ApiError(response.status, data?.error ?? 'avatar_failed', data?.message ?? '头像上传失败，请稍后重试')
  return data as Profile
}
