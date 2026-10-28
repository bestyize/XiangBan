// 内容包直传：先在本地算 SHA-256，向论坛要上传凭证，再把文件直接传到资源机（主站不经手文件）。
import { contentApi } from '@/api/content'

export async function sha256Hex(file: Blob): Promise<string> {
  const digest = await crypto.subtle.digest('SHA-256', await file.arrayBuffer())
  return Array.from(new Uint8Array(digest), b => b.toString(16).padStart(2, '0')).join('')
}

/** 用 XHR 上传以便报告进度；资源机失败时返回它给出的中文原因。 */
function put(url: string, ticket: string, file: Blob, onProgress: (ratio: number) => void): Promise<void> {
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest()
    xhr.open('PUT', url)
    xhr.setRequestHeader('X-Upload-Ticket', ticket)
    xhr.upload.onprogress = e => e.lengthComputable && onProgress(e.loaded / e.total)
    xhr.onerror = () => reject(new Error('上传失败，请检查网络后重试'))
    xhr.onload = () => {
      if (xhr.status >= 200 && xhr.status < 300) return resolve()
      let message = `上传失败（${xhr.status}）`
      try {
        message = JSON.parse(xhr.responseText).message ?? message
      } catch {
        // 响应不是 JSON 时用状态码
      }
      reject(new Error(message))
    }
    xhr.send(file)
  })
}

/** 上传一个文件，返回资源机上的文件键。kind 是内容包类型，或 `cover`。 */
export async function uploadFile(kind: string, file: File, onProgress: (ratio: number) => void): Promise<string> {
  const sha256 = await sha256Hex(file)
  const ticket = await contentApi.uploadTicket(kind, file.size, sha256)
  await put(ticket.upload_url, ticket.ticket, file, onProgress)
  return ticket.file_key
}
