import channels from '../../../editorial/channels.json'
/** 当前栏目是唯一分类树；旧 slug 仅作为输入别名。 */
export function canonicalCategory(slug:string) { return (channels.aliases as Record<string,string>)[slug] || slug }
export const CATEGORY_ALIASES = channels.aliases as Record<string,string>
/** Preserve historical category lookup without offering an obsolete publishing channel. */
const legacyCategories = new Set(channels.channels.filter(c => 'legacy_only' in c && c.legacy_only).map(c => c.id))
export function isLegacyCategory(slug:string) { return legacyCategories.has(canonicalCategory(slug)) }
export const DISCUSSION_TABS = [{key:'create',label:'创作'},{key:'learn',label:'学习和教程'},{key:'casual',label:'休闲'}]
export const LEARN_FILTERS = [{key:'',label:'全部'},{key:'claude-code',label:'Claude Code'},{key:'codex',label:'Codex'},{key:'提示词',label:'提示词'},{key:'工作流',label:'工作流'},{key:'本地部署',label:'本地部署'},{key:'ai-硬件',label:'AI 硬件'}]
export const RESOURCE_TABS = [{key:'tools',label:'AI 工具'},{key:'docs',label:'资料'},{key:'media',label:'影音娱乐'},{key:'help',label:'求助悬赏'}]
export const LEARN_CATEGORIES = 'learn'
export function learnCategory(filter:string) {
  return ({'claude-code':'learn-claude-code',codex:'learn-codex','提示词':'learn-prompts','工作流':'learn-workflows','本地部署':'learn-local','ai-硬件':'learn-hardware'} as Record<string,string>)[filter] || 'learn'
}
export function creationCategory(filter:string) { return filter ? `create-${filter}` : 'create' }

export function resourceTopicCategory(tab:string, filter:string):string {
  if (tab === 'media') return filter === 'music' ? 'resource-music' : filter === 'movies' ? 'resource-movies' : 'resource-movies,resource-music'
  return tab === 'help' ? 'resource-help' : tab === 'tools' ? 'resource-tools' : tab === 'docs' ? 'resource-docs' : 'resource-open'
}
