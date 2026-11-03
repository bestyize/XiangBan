import type { OpenSourceProject } from './openSource'
import manifest from '../../../editorial/open-source-software.json'

export const softwareCategories = [
  { key: 'all', label: '全部' },
  { key: 'ai', label: 'AI' },
  { key: 'media', label: '影音' },
  { key: 'network', label: '网络' },
  { key: 'tools', label: '工具' },
  { key: 'productivity', label: '效率' },
  { key: 'development', label: '开发' },
  { key: 'games', label: '游戏' },
  { key: 'other', label: '其它' },
] as const

type SoftwareCategory = Exclude<typeof softwareCategories[number]['key'], 'all'>

const desktop = ['Windows', 'macOS', 'Linux'] as const
const desktopAndAndroid = [...desktop, 'Android'] as const
export type SoftwarePlatform = typeof desktopAndAndroid[number]

interface SoftwareEntry {
  category: SoftwareCategory
  platforms: readonly SoftwarePlatform[]
  releaseRepo: string
  version: string
  language: string
  project?: OpenSourceProject
}

const software = manifest as Record<string, SoftwareEntry>
export const softwareProjects = Object.values(software).flatMap(entry => entry.project ? [entry.project] : [])

export function findSoftwareProject(name: string) {
  return software[name.toLowerCase()]?.project
}

export function softwareDownload(fullName: string) {
  const key = fullName.toLowerCase()
  const entry = software[key]
  if (!entry) return undefined
  const releaseRepo = entry.releaseRepo
  const releases = `https://github.com/${releaseRepo}/releases`
  return { releaseRepo, category: entry.category, platforms: entry.platforms, version: entry.version, language: entry.language, releases, latest: `${releases}/latest` }
}

export const softwareDirectoryPreview = import.meta.env.DEV && import.meta.env.VITE_OPEN_SOURCE_PREVIEW === '1'

export function softwareOwnerIcon(fullName: string) {
  const owner = fullName.split('/')[0]
  return `https://github.com/${encodeURIComponent(owner)}.png?size=80`
}
