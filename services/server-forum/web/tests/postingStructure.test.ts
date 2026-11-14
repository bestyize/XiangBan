import { test } from 'node:test'
import { deepStrictEqual, strictEqual } from 'node:assert'
import { postingLocation, postingSections, postingSelection } from '../src/lib/postingStructure.ts'
import type { Category } from '../src/api/types.ts'
function row(id:number,slug:string,section:Category['section'],parent_id:number|null=null, extra:Partial<Category>={}):Category {
  return {id,slug,section,parent_id,name:slug,status:'active',posting_enabled:true,description:'',kind:'normal',sort_order:id,min_post_level:1,on_home:true,award_points:false,admin_only:false,preset_tags:[],topic_count:0,...extra}
}
const rows=[row(1,'create','ideas'),row(2,'create-game','ideas',1),row(3,'learn','discuss'),row(4,'learn-codex','discuss',3),row(5,'casual','discuss'),row(6,'casual-travel','discuss',5),row(7,'resources','resources'),row(9,'resource-help','resources',7),row(10,'site','site'),row(11,'site-feedback','site',10),row(12,'site-notice','site',10,{admin_only:true})]
test('posting destinations follow current boards and keep restricted boards visible',()=>{
 const sections=postingSections(rows,1)
 deepStrictEqual(sections.map(s=>s.name),['创意','讨论','资源'])
 deepStrictEqual(sections.find(s=>s.key==='discuss')!.groups.map(g=>g.id),[3,6,10])
 deepStrictEqual(sections.flatMap(s=>s.groups.flatMap(g=>g.categories.map(c=>c.id))),[2,4,6,11,12,9])
 strictEqual(sections.flatMap(s=>s.groups.flatMap(g=>g.categories)).find(c=>c.id===12)?.unavailableReason,'需守护者')
})
test('category and parent preselection use real IDs inside the right page',()=>{
 const sections=postingSections(rows,1)
 deepStrictEqual(postingSelection(sections,'learn-codex',rows),{sectionKey:'discuss',categoryId:4})
 deepStrictEqual(postingSelection(sections,'resource-help',rows),{sectionKey:'resources',categoryId:9})
 deepStrictEqual(postingSelection(sections,'site-feedback',rows),{sectionKey:'discuss',categoryId:11})
 deepStrictEqual(postingSelection(sections,'create',rows),{sectionKey:'ideas',categoryId:2})
 // 资源区现在只有一个可发帖板块，进入资源页时直接选中它。
 deepStrictEqual(postingSelection(sections,'',rows,'resources'),{sectionKey:'resources',categoryId:9})
})
test('closed destinations and permission changes prevent selection and allow existing edits',()=>{
 const changed=rows.map(c=>c.id===3?{...c,posting_enabled:false}:c)
 const sections=postingSections(changed,1)
 strictEqual(sections.flatMap(s=>s.groups.flatMap(g=>g.categories)).find(c=>c.id===4)?.unavailableReason,'已关闭发帖')
 deepStrictEqual(postingSelection(sections,'learn-codex',changed),{sectionKey:'discuss',categoryId:0})
 strictEqual(postingSections(rows.map(c=>c.id===7?{...c,status:'archived' as const}:c),1).some(s=>s.key==='resources'),false)
 const edited=postingSections(rows,0,4).flatMap(s=>s.groups.flatMap(g=>g.categories)).filter(c=>!c.unavailableReason)
 deepStrictEqual(edited.map(c=>c.id),[4])
 strictEqual(postingSections(rows,3).flatMap(s=>s.groups.flatMap(g=>g.categories)).find(c=>c.id===12)?.unavailableReason,'')
})
test('the publish shortcut uses the board that the current page is displaying', () => {
  deepStrictEqual(postingLocation('discuss', { tab: 'learn', filter: 'codex' }, rows), { section: 'discuss', category: 'learn-codex' })
  deepStrictEqual(postingLocation('discuss', { tab: 'all', board: 'learn', filter: 'learn-codex' }, rows), { section: 'discuss', category: 'learn-codex' })
  deepStrictEqual(postingLocation('discuss', { tab: 'casual', filter: 'casual-travel' }, rows), { section: 'discuss', category: 'casual-travel' })
  deepStrictEqual(postingLocation('discuss', { tab: 'design' }, rows), { section: 'discuss' })
  deepStrictEqual(postingLocation('resources', { tab: 'software' }, rows), { section: 'resources' })
  deepStrictEqual(postingLocation('resources', { tab: 'help' }, rows), { section: 'resources', category: 'resource-help' })
  deepStrictEqual(postingLocation('category', {}, rows, 'site-feedback'), { section: 'discuss', category: 'site-feedback' })
  deepStrictEqual(postingLocation('ideas', { category: 'xiaoban-game' }, rows), { section: 'ideas', category: 'create-game' })
})
test('no eligible board is substituted for a specific unavailable board', () => {
  const withMovies = [...rows, row(8, 'resource-movies', 'resources', 7, { min_post_level: 0 })]
  const sections = postingSections(withMovies, 0)
  deepStrictEqual(postingSelection(sections, 'resource-help', withMovies), { sectionKey: 'resources', categoryId: 0 })
  deepStrictEqual(postingSelection(sections, 'resources', withMovies), { sectionKey: 'resources', categoryId: 8 })
  deepStrictEqual(postingSelection(sections, '', withMovies, 'resources'), { sectionKey: 'resources', categoryId: 8 })
})
test('discussion choices mirror public order and choose a valid default', () => {
  const withGames = [...rows, row(14, 'casual-games', 'discuss', 5, { min_post_level: 2 })]
  const sections = postingSections(withGames, 1)
  const groups = sections.find(s => s.key === 'discuss')!.groups
  deepStrictEqual(groups.map(({ name, expandable }) => [name, expandable]), [
    ['casual-games', false], ['learn', true], ['casual-travel', false], ['site', true],
  ])
  deepStrictEqual(postingSelection(sections, '', withGames), { sectionKey: 'discuss', categoryId: 4 })
  deepStrictEqual(postingSelection(sections, 'learn', withGames), { sectionKey: 'discuss', categoryId: 4 })
})
