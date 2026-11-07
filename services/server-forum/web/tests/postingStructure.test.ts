import { test } from 'node:test'
import { deepStrictEqual, strictEqual } from 'node:assert'
import { postingSections, postingSelection } from '../src/lib/postingStructure.ts'
import type { Category } from '../src/api/types.ts'
function row(id:number,slug:string,section:Category['section'],parent_id:number|null=null, extra:Partial<Category>={}):Category {
  return {id,slug,section,parent_id,name:slug,status:'active',posting_enabled:true,description:'',kind:'normal',sort_order:id,min_post_level:1,on_home:true,award_points:false,admin_only:false,preset_tags:[],topic_count:0,...extra}
}
const rows=[row(1,'create','ideas'),row(2,'create-game','ideas',1),row(3,'learn','discuss'),row(4,'learn-codex','discuss',3),row(5,'casual','discuss'),row(6,'casual-travel','discuss',5),row(7,'resources','resources'),row(9,'resource-help','resources',7),row(10,'site','site'),row(11,'site-feedback','site',10),row(12,'site-notice','site',10,{admin_only:true})]
test('posting destinations match the community pages rather than five database roots',()=>{
 const sections=postingSections(rows,1)
 deepStrictEqual(sections.map(s=>s.name),['创意','讨论','资源'])
 deepStrictEqual(sections.find(s=>s.key==='discuss')!.groups.map(g=>g.id),[3,5,10])
 deepStrictEqual(sections.flatMap(s=>s.groups.flatMap(g=>g.categories.map(c=>c.id))),[2,4,6,11,9])
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
test('stopped destinations and permission changes cannot expose new posting choices',()=>{
 const changed=rows.map(c=>c.id===3?{...c,posting_enabled:false}:c)
 strictEqual(postingSections(changed,1).flatMap(s=>s.groups.flatMap(g=>g.categories)).some(c=>c.id===4),false)
 strictEqual(postingSections(rows.map(c=>c.id===7?{...c,status:'archived' as const}:c),1).some(s=>s.key==='resources'),false)
 const edited=postingSections(rows,0,4).flatMap(s=>s.groups.flatMap(g=>g.categories))
 deepStrictEqual(edited.map(c=>c.id),[4])
 strictEqual(postingSections(rows,3).flatMap(s=>s.groups.flatMap(g=>g.categories)).some(c=>c.id===12),true)
})
