import { createHash } from 'node:crypto'
import { access, mkdir, readFile, rename, rm, writeFile } from 'node:fs/promises'
import { galleryExhibits } from '../src/lib/microGallery.ts'

// RealWorldBuilder commits the self-contained players alongside their source.
const upstream = 'https://raw.githubusercontent.com/bestyize/RealWorldBuilder/'
const galleryRevision = '48eef09d9a1e2546de4329a3d0490170e63699f2'
const fishRevision = 'cb1212a3af11131b69a6cd47d0d0e5d37e84a9da'
const fishSources = new Map([
  ['aquarium.html', 'aquarium.html'],
  ['fish/blue-guppy/preview.html', 'fish/blue-guppy/preview.html'],
  ['fish/blue-guppy/school-preview.html', 'fish/blue-guppy/school-preview.html'],
  // Keep the standalone exhibit URL used by existing mobile clients.
  ['fish/arowana/preview.html', 'fish/vermilion-arowana/preview.html'],
])
const destination = new URL('../public/gallery-exhibits/', import.meta.url)
const marker = new URL('.gallery-source-version', import.meta.url)
const maxFileBytes = 40 * 1024 * 1024

const policy = [
  "default-src 'none'",
  "script-src 'unsafe-inline' 'wasm-unsafe-eval'",
  "style-src 'unsafe-inline'",
  'img-src data: blob:',
  'media-src data: blob:',
  'connect-src data: blob:',
  'worker-src blob:',
  "base-uri 'none'",
  "form-action 'none'",
].join('; ')

function galleryLink(linkClass, label, target) {
  return `<button${linkClass} type="button" onclick="openCommunityExhibit('${target}')">${label}</button>`
}

function embedPlayer(html, source) {
  const head = /<head>/i
  if (!head.test(html)) throw new Error(`展品页面缺少 head：${source}`)
  const bridge = `<script>
    function openCommunityExhibit(action) {
      if (window.parent !== window) window.parent.postMessage(action, '*');
    }

    document.addEventListener('DOMContentLoaded', () => {
      const status = document.querySelector('#status, #preview-status');
      if (!status) return;
      const showFailure = () => {
        if (!status.textContent.trim().startsWith('无法')) return;
        status.textContent = '当前无法显示 3D 预览，可以查看展品图片。';
        status.dataset.previewFailed = 'true';
        observer.disconnect();
        openCommunityExhibit('gallery:unavailable');
      };
      const observer = new MutationObserver(showFailure);
      observer.observe(status, { childList: true, characterData: true, subtree: true });
      showFailure();
    });
  </script>`
  const linkStyle = `<style>
    button[onclick^="openCommunityExhibit"] { font: inherit; cursor: pointer; }
    #background-overlay { display: none !important; }
  </style>`
  return html
    .replace(head, `<head><meta http-equiv="Content-Security-Policy" content="${policy}">${linkStyle}${bridge}`)
    .replace(/<a\b([^>]*)>([\s\S]*?)<\/a>/gi, (_, attributes, label) => {
      const href = /\bhref="([^"]+)"/i.exec(attributes)?.[1]
      const className = /\bclass="([^"]+)"/i.exec(attributes)?.[1]
      const linkClass = className ? ` class="${className}"` : ''
      if (/^(?:\.\.\/|\.\/)*index\.html(?:#.*)?$/.test(href ?? '')) {
        return galleryLink(linkClass, label, 'gallery:back')
      }
      if (/^(?:\.\.\/|\.\/)*aquarium\.html$/.test(href ?? '')) {
        return galleryLink(linkClass, label, 'gallery:aquarium')
      }
      // The background controls still update this element when the transparent preset is selected.
      if (href === '#overlay') return '<span id="background-overlay" hidden></span>'
      throw new Error(`无法识别展品页面的链接：${source} ${href}`)
    })
}

async function download(source) {
  const fish = fishSources.get(source)
  const revision = fish ? fishRevision : galleryRevision
  const response = await fetch(`${upstream}${revision}/${fish ?? source}`, {
    redirect: 'error',
    signal: AbortSignal.timeout(45_000),
  })
  if (!response.ok || !response.body) throw new Error(`展品资源下载失败：${source}（HTTP ${response.status}）`)
  const chunks = []
  let bytes = 0
  for await (const chunk of response.body) {
    bytes += chunk.byteLength
    if (bytes > maxFileBytes) throw new Error(`展品资源超过 40 MiB：${source}`)
    chunks.push(chunk)
  }
  return Buffer.concat(chunks, bytes)
}

async function sync() {
  const files = galleryExhibits.flatMap(({ source, cover }) => [source, `assets/gallery/${cover}.webp`])
  const covers = new Map(await Promise.all(['arowana', 'living-room'].map(async cover => {
    const path = `assets/gallery/${cover}.webp`
    return [path, await readFile(new URL(`../${path}`, import.meta.url))]
  })))
  const hash = createHash('sha256').update(await readFile(new URL(import.meta.url))).update(files.join('\n'))
  for (const cover of covers.values()) hash.update(cover)
  const stamp = hash.digest('hex')
  const current = await readFile(marker, 'utf8').catch(() => '')
  if (current === stamp &&
      await Promise.all(files.map(source => access(new URL(source, destination)))).then(() => true, () => false)) return

  await rm(marker, { force: true })
  await rm(new URL('.source-version', destination), { force: true })
  await Promise.all(files.map(async source => {
    const buffer = covers.get(source) ?? await download(source)
    const output = new URL(source, destination)
    const content = source.endsWith('.html') ? embedPlayer(buffer.toString('utf8'), source) : buffer
    await mkdir(new URL('./', output), { recursive: true })
    const temp = new URL(output.href + '.' + process.pid + '.tmp')
    try {
      await writeFile(temp, content)
      await rename(temp, output)
    } finally {
      await rm(temp, { force: true })
    }
  }))
  await writeFile(marker, stamp)
  console.log(`已同步 ${galleryExhibits.length} 件展品`)
}

await sync()
