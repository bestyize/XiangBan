import { test } from 'node:test'
import { deepStrictEqual, strictEqual } from 'node:assert'
import { postingLocation, postingSections, postingSelection, type PostingSectionKey } from '../src/lib/postingStructure.ts'
import type { Category } from '../src/api/types.ts'
function row(id:number,slug:string,section:Category['section'],parent_id:number|null=null, extra:Partial<Category>={}):Category {
  return {id,slug,section,parent_id,name:slug,status:'active',posting_enabled:true,description:'',kind:'normal',sort_order:id,min_post_level:1,on_home:true,award_points:false,admin_only:false,preset_tags:[],topic_count:0,...extra}
}
const rows=[row(2,'create-game','ideas'),row(3,'learn','discuss'),row(6,'casual-travel','discuss'),row(7,'resources','resources'),row(9,'resource-help','resources',7),row(10,'site','site',null,{admin_only:true})]
test('posting destinations follow current boards and keep restricted boards visible',()=>{
 const sections=postingSections(rows,1)
 deepStrictEqual(sections.map(s=>s.name),['创意','讨论','资源'])
 deepStrictEqual(sections.find(s=>s.key==='discuss')!.boards.map(c=>c.id),[3,6,10])
 deepStrictEqual(sections.flatMap(s=>s.boards.map(c=>c.id)),[2,3,6,10,9])
 strictEqual(sections.flatMap(s=>s.boards).find(c=>c.id===10)?.unavailableReason,'需守护者')
})
test('preselection uses a real board ID in the right section',()=>{
 const sections=postingSections(rows,1)
 deepStrictEqual(postingSelection(sections,'learn-codex',rows),{sectionKey:'discuss',categoryId:3})
 deepStrictEqual(postingSelection(sections,'resource-help',rows),{sectionKey:'resources',categoryId:9})
 deepStrictEqual(postingSelection(sections,'site-feedback',rows),{sectionKey:'discuss',categoryId:0})
 deepStrictEqual(postingSelection(sections,'create-game',rows),{sectionKey:'ideas',categoryId:2})
 deepStrictEqual(postingSelection(sections,'',rows,'resources'),{sectionKey:'resources',categoryId:0})
})
test('closed destinations and permission changes prevent selection and allow existing edits',()=>{
 const changed=rows.map(c=>c.id===3?{...c,posting_enabled:false}:c)
 const sections=postingSections(changed,1)
 strictEqual(sections.flatMap(s=>s.boards).find(c=>c.id===3)?.unavailableReason,'已关闭发帖')
 deepStrictEqual(postingSelection(sections,'learn-codex',changed),{sectionKey:'discuss',categoryId:0})
 strictEqual(postingSections(rows.map(c=>c.id===7?{...c,status:'archived' as const}:c),1).some(s=>s.key==='resources'),false)
 const edited=postingSections(rows,0,3).flatMap(s=>s.boards).filter(c=>!c.unavailableReason)
 deepStrictEqual(edited.map(c=>c.id),[3])
 strictEqual(postingSections(rows,3).flatMap(s=>s.boards).find(c=>c.id===10)?.unavailableReason,'')
})
test('the publish shortcut follows the current page and its active board', () => {
  deepStrictEqual(postingLocation('discuss', { tab: 'learn', filter: 'codex' }, rows), { section: 'discuss', category: 'learn' })
  deepStrictEqual(postingLocation('discuss', { tab: 'all', board: 'learn', filter: 'learn-codex' }, rows), { section: 'discuss', category: 'learn' })
  deepStrictEqual(postingLocation('discuss', { tab: 'casual', filter: 'casual-travel' }, rows), { section: 'discuss', category: 'casual-travel' })
  deepStrictEqual(postingLocation('discuss', { tab: 'design' }, rows), { section: 'discuss' })
  deepStrictEqual(postingLocation('resources', { tab: 'software' }, rows), { section: 'resources' })
  deepStrictEqual(postingLocation('resources', { tab: 'help' }, rows), { section: 'resources', category: 'resource-help' })
  deepStrictEqual(postingLocation('category', {}, rows, 'site-feedback'), { section: 'discuss', category: 'site' })
  deepStrictEqual(postingLocation('ideas', { category: 'xiaoban-game' }, rows), { section: 'ideas' })
  deepStrictEqual(postingLocation('ideas', { tab: 'gallery' }, rows), { section: 'ideas' })
})
test('no eligible board is substituted for a specific unavailable board', () => {
  const withMovies = [...rows, row(8, 'resource-movies', 'resources', 7, { min_post_level: 0 })]
  const sections = postingSections(withMovies, 0)
  deepStrictEqual(postingSelection(sections, 'resource-help', withMovies), { sectionKey: 'resources', categoryId: 0 })
  deepStrictEqual(postingSelection(sections, 'resources', withMovies), { sectionKey: 'resources', categoryId: 0 })
  deepStrictEqual(postingSelection(sections, '', withMovies, 'resources'), { sectionKey: 'resources', categoryId: 0 })
})
test('the default is a separate uncategorized board in each section', () => {
  const keys: PostingSectionKey[] = ['ideas', 'discuss', 'resources']
  const defaults = keys.map((key, index) => row(20 + index, `${key}-uncategorized`, key, null, {
    kind: 'uncategorized', name: '默认',
  }))
  const withGames = [...rows, row(14, 'casual-games', 'discuss', null, { min_post_level: 2 })]
  const all = [...defaults, ...withGames]
  const sections = postingSections(all, 1)
  for (const board of defaults) {
    const key = board.section as PostingSectionKey
    deepStrictEqual(postingSelection(sections, '', all, key), { sectionKey: key, categoryId: board.id })
  }
  const boards = sections.find(s => s.key === 'discuss')!.boards
  deepStrictEqual(boards.map(board => board.name), ['默认', 'casual-games', 'learn', 'casual-travel', 'site'])
  deepStrictEqual(postingSelection(sections, 'learn', all), { sectionKey: 'discuss', categoryId: 3 })
  deepStrictEqual(postingSelection(sections, 'learn-codex', all), { sectionKey: 'discuss', categoryId: 3 })
  deepStrictEqual(postingSelection(postingSections(all, 0), '', all), { sectionKey: 'discuss', categoryId: 0 })
  const withoutResources = all.filter(board => board.section !== 'resources')
  strictEqual(postingSelection(postingSections(withoutResources, 1), '', withoutResources, 'resources').categoryId, 0)
})
