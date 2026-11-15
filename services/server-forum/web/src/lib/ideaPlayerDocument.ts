const maxDocumentBytes = 32 * 1024 * 1024
const bridgeStart = '<script>\ndocument.currentScript.dataset.parentOrigin = location.origin;\n'

function documentPolicy(response: Response, sandbox: string): string | undefined {
  const header = response.headers.get('content-security-policy')
  const frameOptions = response.headers.get('x-frame-options')?.toLowerCase()
  if (!header || header.includes(',') || (frameOptions && frameOptions !== 'sameorigin')) return

  const directives = header.split(';').map(value => value.trim()).filter(Boolean)
  const serverSandbox = directives.find(value => value.startsWith('sandbox '))?.split(/\s+/).slice(1)
  if (!directives.includes("frame-ancestors 'self'") || !directives.includes("default-src 'none'") ||
      !sandbox.split(' ').every(permission => serverSandbox?.includes(permission))) return
  // The iframe applies sandboxing and the caller runs in a top-level forum page.
  return directives.filter(value => !/^(sandbox|frame-ancestors|report-uri|report-to)\s/.test(value)).join('; ')
}

function attribute(value: string): string {
  return value.replaceAll('&', '&amp;').replaceAll('"', '&quot;').replaceAll('<', '&lt;')
}

function nginxDocumentBytes(response: Response): number | undefined {
  if (response.headers.get('content-encoding') !== 'gzip' ||
      !/^nginx(?:\/|$)/.test(response.headers.get('server') ?? '')) return
  // Nginx's gzip filter makes the static file's ETag weak and retains its size field.
  const tag = response.headers.get('etag')?.match(/^W\/"([0-9a-f]+)-([0-9a-f]+)"$/i)
  if (!tag || parseInt(tag[1]!, 16) * 1000 !== Date.parse(response.headers.get('last-modified') ?? '')) return
  const size = parseInt(tag[2]!, 16)
  if (Number.isSafeInteger(size) && size > 0) return size
}

export async function readIdeaPlayerDocument(
  url: URL,
  sandbox: string,
  signal: AbortSignal,
  progress: (received: number, total: number | undefined) => void,
): Promise<string | undefined> {
  const response = await fetch(url, {
    credentials: 'omit', referrerPolicy: 'no-referrer', redirect: 'error', signal,
  })
  if (!response.ok) throw new Error(`作品资源请求失败（HTTP ${response.status}）`)
  if (!response.body) return

  const policy = documentPolicy(response, sandbox)
  const encoding = response.headers.get('content-encoding')
  const length = Number(response.headers.get('content-length'))
  const declared = (!encoding || encoding === 'identity') && Number.isSafeInteger(length) && length > 0 ? length : undefined
  let total = declared ?? nginxDocumentBytes(response)
  if (!policy || !/^text\/html(?:;|$)/i.test(response.headers.get('content-type') ?? '') ||
      response.status !== 200 || (total !== undefined && total > maxDocumentBytes)) {
    await response.body.cancel().catch(() => {})
    return
  }

  const decoder = new TextDecoder()
  const reader = response.body.getReader()
  const chunks: string[] = []
  let received = 0
  progress(received, total)
  try {
    while (true) {
      const { done, value } = await reader.read()
      if (done) break
      received += value.byteLength
      if (received > maxDocumentBytes) return
      if (declared !== undefined && received > declared) throw new Error('作品资源大小与响应不一致')
      if (total !== undefined && received > total) total = undefined
      chunks.push(decoder.decode(value, { stream: true }))
      progress(received, total)
    }
  } finally {
    await reader.cancel().catch(() => {})
  }

  if (declared !== undefined && received !== declared) throw new Error('作品资源下载不完整')
  chunks.push(decoder.decode())
  const html = chunks.join('')
  if (!html.includes(bridgeStart)) return
  // An opaque srcdoc keeps the work sandboxed. Apply the response's resource
  // policy before its markup and tell the bridge the actual parent origin.
  const secured = `<!doctype html><meta charset="utf-8"><meta http-equiv="Content-Security-Policy" content="${attribute(policy)}">`
  return secured + html.replace(/^\s*<!doctype html>/i, '')
    .replace(bridgeStart, `<script data-parent-origin="${attribute(url.origin)}">\n`)
}
