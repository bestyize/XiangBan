/* Runs inside the opaque iframe before any code from the ZIP. */
(() => {
  'use strict'
  const boot = document.currentScript
  const policy = document.querySelector('meta[http-equiv="Content-Security-Policy"]')
  const { parentOrigin, instance, preview } = boot.dataset
  const channel = 'xiaoban.app-runtime.v2'
  const send = (type, extra) => parent.postMessage({ channel, instance, type, ...extra }, parentOrigin)
  const root = new URL('https://work.invalid/')
  const mime = {
    css: 'text/css', js: 'text/javascript', mjs: 'text/javascript', json: 'application/json',
    svg: 'image/svg+xml', png: 'image/png', jpg: 'image/jpeg', jpeg: 'image/jpeg', webp: 'image/webp',
    avif: 'image/avif', gif: 'image/gif', ico: 'image/x-icon',
    mp3: 'audio/mpeg', ogg: 'audio/ogg', wav: 'audio/wav', m4a: 'audio/mp4', aac: 'audio/aac',
    mp4: 'video/mp4', webm: 'video/webm',
    woff: 'font/woff', woff2: 'font/woff2', ttf: 'font/ttf', otf: 'font/otf',
    wasm: 'application/wasm', glb: 'model/gltf-binary', gltf: 'model/gltf+json',
  }

  function dataUrl(blob) {
    return new Promise((resolve, reject) => {
      const reader = new FileReader()
      reader.onload = () => resolve(reader.result)
      reader.onerror = () => reject(reader.error)
      reader.readAsDataURL(blob)
    })
  }

  async function show({ entry, files, bridge }) {
    const urls = new Map()
    function resolve(value, from) {
      if (!value || value.startsWith('#')) return
      try {
        const url = new URL(value, new URL(from, root))
        const path = decodeURIComponent(url.pathname.slice(1))
        if (url.origin === root.origin && Object.hasOwn(files, path)) return { path, hash: url.hash }
      } catch { /* Leave invalid URLs to the browser. */ }
    }
    function typedFile(path) {
      const type = mime[path.split('.').at(-1).toLowerCase()] || 'application/octet-stream'
      return files[path].slice(0, files[path].size, type)
    }
    function fileUrl(path) {
      // WebKit cannot load blob:null URLs from the rewritten opaque document.
      if (!urls.has(path)) urls.set(path, dataUrl(typedFile(path)))
      return urls.get(path)
    }
    async function cssUrls(css, from) {
      let output = '', start = 0
      for (const match of css.matchAll(/url\(\s*(?:"([^"]*)"|'([^']*)'|([^\s)]+))\s*\)/gi)) {
        const [, quoted, single, plain] = match
        const asset = resolve(quoted ?? single ?? plain, from)
        if (!asset) continue
        output += css.slice(start, match.index) + `url("${await fileUrl(asset.path)}${asset.hash}")`
        start = match.index + match[0].length
      }
      return output + css.slice(start)
    }

    const html = new TextDecoder('utf-8', { fatal: true }).decode(await files[entry].arrayBuffer())
    const page = new DOMParser().parseFromString(html, 'text/html')
    for (const meta of page.querySelectorAll('meta[http-equiv="refresh" i]')) meta.remove()
    for (const element of page.querySelectorAll('script[src], img[src], input[type="image"][src], audio[src], video[src], source[src], track[src], video[poster], link[rel~="icon"][href], link[rel~="stylesheet"][href], image[href], use[href]')) {
      for (const attribute of ['src', 'href', 'poster']) {
        const asset = resolve(element.getAttribute(attribute), entry)
        if (!asset) continue
        const { path, hash } = asset
        const url = element.matches('link[rel~="stylesheet"]')
          ? await dataUrl(new Blob([await cssUrls(await files[path].text(), path)], { type: 'text/css' }))
          : await fileUrl(path)
        element.setAttribute(attribute, url + hash)
      }
    }
    for (const style of page.querySelectorAll('style')) style.textContent = await cssUrls(style.textContent, entry)
    for (const element of page.querySelectorAll('[style]')) element.setAttribute('style', await cssUrls(element.getAttribute('style'), entry))

    // Keep relative fetch() calls inside the uploaded package. The original
    // fetch still enforces CSP for URLs that do not resolve to a packaged file.
    const browserFetch = window.fetch.bind(window)
    window.fetch = async (input, init) => {
      const method = init?.method ?? (input instanceof Request ? input.method : 'GET')
      if (/^(get|head)$/i.test(method)) {
        const url = input instanceof Request ? input.url : String(input)
        const asset = resolve(url, entry)
        if (asset) {
          const signal = init?.signal ?? (input instanceof Request ? input.signal : undefined)
          signal?.throwIfAborted()
          const file = typedFile(asset.path)
          const body = /^head$/i.test(method) ? null : await file.arrayBuffer()
          signal?.throwIfAborted()
          return new Response(body, {
            headers: { 'Content-Type': file.type, 'Content-Length': String(file.size) },
          })
        }
      }
      return browserFetch(input, init)
    }

    const sdk = page.createElement('script')
    Object.assign(sdk.dataset, { parentOrigin, instance, preview })
    sdk.textContent = bridge
    page.head.prepend(policy.cloneNode(), sdk)
    document.open()
    document.write('<!doctype html>' + page.documentElement.outerHTML)
    document.close()
  }

  function receive(event) {
    if (event.source !== parent || event.origin !== parentOrigin || event.data?.channel !== channel ||
        event.data.instance !== instance || event.data.type !== 'archive-init') return
    window.removeEventListener('message', receive)
    void show(event.data).catch(() => send('archive-error', { message: '无法加载作品入口，请检查 HTML 文件和包内素材' }))
  }
  window.addEventListener('message', receive)
  send('archive-ready')
})()
