import { test } from 'node:test'
import { strictEqual, deepStrictEqual } from 'node:assert'
import { discussionTabs, learnFilters, resourceTabs, categoryDestination, sectionScope, activeCategories } from '../src/lib/communityStructure.ts'
import type { Category } from '../src/api/types.ts'
function category(id:number, slug:string, section:Category['section'], parent_id:number|null=null, extra:Partial<Category>={}):Category {
  return {id,slug,name:slug,section,parent_id,status:'active',posting_enabled:true,description:'',kind:'normal',sort_order:id,min_post_level:1,on_home:true,award_points:false,admin_only:false,preset_tags:[],topic_count:0,...extra}
}
const rows=[category(1,'learn','discuss'),category(2,'learn-codex','discuss',1),category(3,'casual','discuss'),category(4,'casual-travel','discuss',3),category(5,'resources','resources'),category(6,'resource-movies','resources',5),category(7,'resource-help','resources',5),category(8,'resource-open','resources',5)]
test('navigation follows database names, additions and stop state',()=>{
 const changed=[...rows,category(9,'casual-new','discuss',3,{name:'新话题'}),category(10,'old','resources',5,{status:'archived'})]
 strictEqual(discussionTabs(changed).some(c=>c.label==='新话题'),true)
 deepStrictEqual(learnFilters(changed).map(c=>c.key),['','learn-codex'])
 deepStrictEqual(resourceTabs(changed).map(c=>c.key),['software','resource-movies','resource-help','resource-open'])
 strictEqual(sectionScope(changed,'discuss').includes('old'),false)
 strictEqual(categoryDestination(rows[6]!,rows),'/resources?tab=resource-help')
 strictEqual(categoryDestination(rows[1]!,rows),'/discuss?tab=learn&filter=learn-codex')
})
test('a stopped parent hides descendants and legacy routes do not reopen them',()=>{
 const changed=rows.map(c=>c.id===5?{...c,status:'archived' as const}:c)
 strictEqual(activeCategories(changed).some(c=>c.id===6),false)
 strictEqual(categoryDestination(rows[5]!,changed),'/c/resource-movies')
 deepStrictEqual(resourceTabs(changed),[{key:'software',label:'开源软件'}])
})
