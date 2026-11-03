import { Marked } from 'marked'
import markedFootnote from 'marked-footnote'
import DOMPurify from 'dompurify'

export interface ProjectReadme {
  schema: 1
  full_name: string
  card: {
    name: string; summary_zh: string; icon_source_url?: string; icon_attribution?: string;
    github_url: string; primary_url?: string; primary_label?: string;
    stars?: number; stars_fetched_at?: string; code_updated_at?: string;
    license?: string; platforms: string[]; github_archived?: boolean; upstream_maintenance_notice?: string
  }
  source_url: string
  commit: string
  fetched_at: string
  translation: string
  zh: string
  en: string
  licenses: { name: string; text: string }[]
}

export async function fetchProjectReadme(owner: string, repo: string): Promise<ProjectReadme | null> {
  const key = `${owner}/${repo}`.toLowerCase()
  const indexResponse = await fetch('/open-source-readmes/index.json', { cache: 'no-cache' })
  if (indexResponse.status === 404) return null
  if (!indexResponse.ok) throw new Error('README 暂时无法加载，请重试')
  if (!indexResponse.headers.get('content-type')?.includes('application/json')) return null
  const index = await indexResponse.json() as { projects: Record<string, string> }
  const file = index.projects[key]
  if (!file) return null
  if (!/^projects\/[a-f0-9]{64}\.json$/.test(file)) throw new Error('README 索引无效')
  const response = await fetch(`/open-source-readmes/${file}`)
  if (!response.ok) throw new Error('README 暂时无法加载，请重试')
  const data = await response.json() as ProjectReadme
  if (data.schema !== 1 || data.full_name.toLowerCase() !== key || typeof data.zh !== 'string' || typeof data.en !== 'string') throw new Error('README 内容校验失败')
  return data
}

/** Upstream Markdown is untrusted, including its raw HTML. Never expose it before sanitizing. */
export function renderProjectReadme(markdown: string): string {
  const parser = new Marked({ gfm: true, renderer: {
    checkbox: ({ checked }) => checked ? '<span aria-label="已完成">☑</span> ' : '<span aria-label="未完成">☐</span> ',
  } }).use(markedFootnote())
  const html = parser.parse(markdown, { async: false })
  const safe = DOMPurify.sanitize(html, {
    USE_PROFILES: { html: true },
    ADD_TAGS: ['details', 'summary'],
    FORBID_TAGS: ['style', 'form', 'input', 'button', 'textarea', 'select', 'iframe', 'object', 'embed'],
    FORBID_ATTR: ['style', 'srcset'],
    ALLOW_DATA_ATTR: false,
  })
  const doc = new DOMParser().parseFromString(safe, 'text/html')
  const used = new Map<string, number>()
  doc.querySelectorAll('h1,h2,h3,h4,h5,h6').forEach(node => {
    if (node.id) return
    const base = (node.textContent ?? '').toLowerCase().replace(/[^\p{L}\p{N}\s_-]/gu, '').replace(/ /g, '-')
    const count = used.get(base) ?? 0
    used.set(base, count + 1)
    node.id = base + (count ? `-${count}` : '')
  })
  doc.querySelectorAll('a').forEach(node => {
    const href = node.getAttribute('href') ?? ''
    if (href.startsWith('#')) { node.removeAttribute('target'); return }
    if (!/^https?:\/\//i.test(href) && !/^(mailto|xmpp|ircs?):/i.test(href)) { node.removeAttribute('href'); return }
    node.setAttribute('target', '_blank')
    node.setAttribute('rel', 'noopener noreferrer nofollow')
  })
  doc.querySelectorAll('img').forEach(node => {
    if (!/^https:\/\//i.test(node.getAttribute('src') ?? '')) { node.remove(); return }
    node.setAttribute('loading', 'lazy')
    node.setAttribute('referrerpolicy', 'no-referrer')
  })
  return doc.body.innerHTML
}
