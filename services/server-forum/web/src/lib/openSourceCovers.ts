import rules from '../../../editorial/open-source-cover-rules.json' with { type: 'json' }
import type { OpenSourceProject } from './openSource.ts'

/** This manifest and resolver belong only to the curated GitHub channel. */
export const openSourceCoverRules = rules
export function coverHash(name: string): number {
  let value = 0
  for (const byte of new TextEncoder().encode(name.toLowerCase())) value = (Math.imul(value, 31) + byte) >>> 0
  return value
}
export function resolveOpenSourceCover(project: OpenSourceProject): OpenSourceProject {
  let template = project.cover_template
  if (!template) template = project.cover_style === 'screen' ? 'screenshot' : project.cover_style === 'photo' ? 'photo' : project.cover_layout || 'auto'
  const requested = template
  if (template === 'auto') {
    template = rules.rules.find(raw => {
      const rule = raw as { categories?: string[]; tags?: string[]; requires_media?: boolean }
      return (!rule.requires_media || !!project.cover_url) && (!rule.categories || rule.categories.includes(project.category)) && (!rule.tags || rule.tags.some(tag => project.tags.includes(tag)))
    })?.template || 'flow'
  }
  const preset = rules.templates.find(item => item.id === template) || rules.templates.find(item => item.id === 'flow')!
  const render = preset.requires_media && !project.cover_url ? rules.templates.find(item => item.id === 'flow')! : preset
  const hash = coverHash(project.full_name)
  return {
    ...project, cover_template: requested, cover_style: render.style, cover_layout: render.layout,
    cover_variant: rules.variants.some(item => item.id === project.cover_variant) ? project.cover_variant : rules.variants[hash % rules.variants.length]!.id,
    cover_accent: /^#[\da-f]{6}$/i.test(project.cover_accent || '') ? project.cover_accent : rules.palettes[hash % rules.palettes.length],
    cover_lines: project.cover_lines?.length ? project.cover_lines : project.tags.slice(0, 3),
    cover_focus_x: Math.max(0, Math.min(100, project.cover_focus_x ?? 50)),
    cover_focus_y: Math.max(0, Math.min(100, project.cover_focus_y ?? 46)),
    icon_fit: project.icon_fit === 'mark-left' ? 'mark-left' : 'contain',
  }
}
