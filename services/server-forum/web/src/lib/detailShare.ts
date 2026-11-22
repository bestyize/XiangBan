export type ShareExtra = {
  label: string
  description: string
  icon: string
} & ({ text: () => string } | { href: string })

export interface SharePayload {
  title: string
  url: string
  badge: string
  author?: string | null
  meta?: string
  excerpt?: string | null
  cover?: string | null
  thumbnail?: string | null
  icon?: string
  extra?: ShareExtra
  metric?: string
  linkOnly?: boolean
  verify?: () => Promise<void>
}

export interface ShareDeletion {
  label: string
  confirm: string
  description?: string
  confirmLabel?: string
  run: () => Promise<void>
}

export function copyFull(title: string, html: string): ShareExtra {
  return {
    label: '复制全文',
    description: '复制标题与完整正文内容',
    icon: 'document',
    text: () => {
      const template = document.createElement('template')
      template.innerHTML = html
      for (const node of template.content.querySelectorAll('script,style,template')) node.remove()
      for (const image of template.content.querySelectorAll('img')) image.replaceWith(image.alt)
      for (const node of template.content.querySelectorAll('br, p, h1, h2, h3, h4, h5, h6, li, pre, blockquote, tr, hr')) node.after('\n')
      return [title.trim(), (template.content.textContent ?? '').replace(/\n{3,}/g, '\n\n').trim()].filter(Boolean).join('\n\n')
    },
  }
}

export function copyRepo(url: string): ShareExtra {
  return { label: '复制仓库地址', description: url, icon: 'document', text: () => url }
}
