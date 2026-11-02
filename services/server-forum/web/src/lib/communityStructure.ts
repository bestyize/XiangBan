/** 当前网页栏目与既有内容来源的映射；后端分类标识仍用于兼容旧帖子和客户端。 */
export const DISCUSSION_TABS = [{key:'create',label:'创作'},{key:'learn',label:'学习和教程'},{key:'casual',label:'休闲'}]
export const LEARN_FILTERS = [{key:'',label:'全部'},{key:'claude-code',label:'Claude Code'},{key:'codex',label:'Codex'},{key:'提示词',label:'提示词'},{key:'工作流',label:'工作流'},{key:'本地部署',label:'本地部署'},{key:'ai-硬件',label:'AI 硬件'}]
export const RESOURCE_TABS = [{key:'tools',label:'AI 工具'},{key:'docs',label:'资料'},{key:'media',label:'影音娱乐'},{key:'help',label:'求助悬赏'},{key:'open',label:'开源精选'}]
export const LEARN_CATEGORIES = 'tech,share-learn'

export function resourceTopicCategory(tab:string, filter:string):string {
  if (tab === 'media') return filter === 'music' ? 'share-music' : filter === 'movies' ? 'share-movies' : 'share-movies,share-music'
  return tab === 'help' ? 'share-help' : 'share-software'
}
