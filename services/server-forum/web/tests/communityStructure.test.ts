import { test } from 'node:test'
import { strictEqual, deepStrictEqual } from 'node:assert'
import { discussionTabs, learnFilters, resourceTabs, categoryDestination, sectionScope, activeCategories, navigationSections, categoryHref, categoryLabel, discussionCategory, discussionFilter, postableCategories } from '../src/lib/communityStructure.ts'
import type { Category } from '../src/api/types.ts'
function category(id:number, slug:string, section:Category['section'], parent_id:number|null=null, extra:Partial<Category>={}):Category {
  return {id,slug,name:slug,section,parent_id,status:'active',posting_enabled:true,description:'',kind:'normal',sort_order:id,min_post_level:1,on_home:true,award_points:false,admin_only:false,preset_tags:[],topic_count:0,...extra}
}
const rows=[category(1,'learn','discuss'),category(2,'learn-codex','discuss',1),category(3,'casual','discuss'),category(4,'casual-travel','discuss',3),category(5,'resources','resources'),category(6,'resource-movies','resources',5),category(7,'resource-help','resources',5)]
test('navigation follows database names, additions and stop state',()=>{
 const changed=[...rows,category(9,'casual-new','discuss',3,{name:'新话题'}),category(10,'old','resources',5,{status:'archived'})]
 strictEqual(discussionTabs(changed).some(c=>c.label==='新话题'),true)
 deepStrictEqual(learnFilters(changed).map(c=>c.key),['','learn-codex'])
 deepStrictEqual(resourceTabs(changed).map(c=>c.key),['software','resource-movies','resource-help'])
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

test('detail navigation groups the backend roots into the same three community destinations',()=>{
 const all=[...rows,category(9,'create','ideas'),category(10,'create-effect','ideas',9),category(11,'site','site'),category(12,'site-feedback','site',11)]
 const sections=navigationSections(all)
 deepStrictEqual(sections.map(s=>s.name),['创意','讨论','资源'])
 deepStrictEqual(sections.find(s=>s.key==='discuss')!.children.map(c=>c.slug),['learn-codex','casual-travel','site-feedback'])
 strictEqual(categoryHref({id:7,slug:'resource-help'},all),'/resources?tab=resource-help')
 strictEqual(categoryHref({id:2,slug:'learn-codex'},all),'/discuss?tab=learn&filter=learn-codex')
})
test('administrator-defined tutorial slugs and labels survive every browse and posting mapping',()=>{
 const custom=[...rows,category(9,'writing-lab','discuss',1,{name:'写作实践'})]
 strictEqual(learnFilters(custom).some(c=>c.key==='writing-lab'),true)
 strictEqual(discussionFilter(custom,'learn','writing-lab'),'writing-lab')
 strictEqual(discussionCategory('learn','writing-lab',custom),'writing-lab')
 strictEqual(categoryDestination(custom[custom.length-1]!,custom),'/discuss?tab=learn&filter=writing-lab')
 strictEqual(categoryLabel({...rows[6]!,name:'自定义求助板块'}),'自定义求助板块')
})
test('administrative posting and move selectors exclude stopped categories and descendants',()=>{
 const stopped=rows.map(c=>c.id===5?{...c,posting_enabled:false}:c)
 strictEqual(postableCategories(stopped).some(c=>c.section==='resources'),false)
 const leafStopped=rows.map(c=>c.id===6?{...c,status:'archived' as const}:c)
 strictEqual(postableCategories(leafStopped).some(c=>c.id===6),false)
 strictEqual(postableCategories(leafStopped).some(c=>c.id===7),true)
})

test('a standalone postable category is also reachable from its assigned page',()=>{
 const standalone=category(10,'tools-chat','resources')
 const all=[...rows,standalone]
 strictEqual(resourceTabs(all).some(c=>c.key==='tools-chat'),true)
 strictEqual(categoryHref(standalone,all),'/resources?tab=tools-chat')
 strictEqual(postableCategories(all).some(c=>c.id===standalone.id),true)
})
