type Request = <T>(method: string, path: string, body?: unknown) => Promise<T>
type ErrorFactory = (status: number, message: string) => Error
type Kind = 'image' | 'attachment'
type Ticket = { id: string; upload_url: string; headers: Record<string, string> }

/** The forum authorizes the upload; bytes go only to the fixed resource host. */
export async function directForumUpload<T>(kind: Kind, file: File, request: Request, error: ErrorFactory, onProgress?: (ratio: number) => void): Promise<T> {
  if (!file.size || file.size > 32 * 1024 * 1024) throw error(400, '文件大小超出限制')
  const digest = await crypto.subtle.digest('SHA-256', await file.arrayBuffer())
  const sha256 = Array.from(new Uint8Array(digest), b => b.toString(16).padStart(2, '0')).join('')
  const mime = kind === 'image' ? file.type.toLowerCase() : 'application/octet-stream'
  const ticket = await request<Ticket>('POST', '/resource-uploads/ticket', {kind, name:file.name, mime, size:file.size, sha256})
  const id = ticket.id
  if (!/^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$/.test(id)) throw error(0, '上传凭证无效')
  const expectedPath = kind === 'image' ? `/internal/forum-image/images/${id}` : `/internal/forum-file/${id}`
  let destination: URL
  try { destination = new URL(ticket.upload_url) } catch { throw error(0, '上传地址无效') }
  if (destination.origin !== 'https://res.xiangban.xyz' || destination.pathname !== expectedPath || destination.search || destination.hash || destination.username || destination.password)
    throw error(0, '上传地址无效')
  const headers = ticket.headers
  const allowed = new Set(['Content-Type', 'X-Content-Sha256', 'X-Expiration', 'X-Signature', ...(kind === 'attachment' ? ['X-File-Ext'] : [])])
  if (!headers || Object.keys(headers).length !== allowed.size || Object.keys(headers).some(key => !allowed.has(key)) ||
      headers['Content-Type'] !== mime || headers['X-Content-Sha256'] !== sha256 ||
      !/^\d+$/.test(headers['X-Expiration'] || '') || !/^[a-f0-9]{64}$/.test(headers['X-Signature'] || '') ||
      (kind === 'attachment' && !/^[a-z0-9]{1,8}$/.test(headers['X-File-Ext'] || '')))
    throw error(0, '上传凭证无效')
  const receipt = await new Promise<unknown>((resolve, reject) => {
    const xhr = new XMLHttpRequest()
    xhr.open('POST', destination.href)
    for (const [key, value] of Object.entries(headers)) xhr.setRequestHeader(key, value)
    xhr.upload.onprogress = e => { if (e.lengthComputable) onProgress?.(e.loaded / e.total) }
    xhr.onerror = () => reject(error(0, '网络连接失败，请稍后再试'))
    xhr.onload = () => {
      let data: { message?: string } | null = null
      try { data = JSON.parse(xhr.responseText) } catch { data = null }
      if (xhr.status >= 200 && xhr.status < 300 && data) return resolve(data)
      const fallback = xhr.status === 413 ? '文件太大，被服务器拒绝了' : `上传失败（${xhr.status}）`
      reject(error(xhr.status, data?.message ?? fallback))
    }
    xhr.send(file)
  })
  return request<T>('POST', `/resource-uploads/${id}/complete`, receipt)
}
