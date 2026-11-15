import { test } from 'node:test'
import { strictEqual, deepStrictEqual } from 'node:assert'
import { discussionTabs, discussionTab, resourceTabs, categoryDestination, sectionScope, activeCategories, navigationSections, categoryHref, categoryLabel, discussionCategory, discussionTopicTag, postableCategories } from '../src/lib/communityStructure.ts'
import type { Category } from '../src/api/types.ts'
function category(id:number, slug:string, section:Category['section'], parent_id:number|null=null, extra:Partial<Category>={}):Category {
  return {id,slug,name:slug,section,parent_id,status:'active',posting_enabled:true,description:'',kind:'normal',sort_order:id,min_post_level:1,on_home:true,award_points:false,admin_only:false,preset_tags:[],topic_count:0,...extra}
}
const learn = category(1,'learn','discuss')
const movies = category(6,'resource-movies','resources',5)
const help = category(7,'resource-help','resources',5)
const rows=[learn,category(4,'casual-travel','discuss'),category(5,'resources','resources'),movies,help]
test('mobile tabs keep their order and reflect the published boards and tags',()=>{
 const added=category(9,'writing-lab','discuss',null,{name:'写作实践'})
 const changed=[category(8,'casual-games','discuss',null,{name:'游戏'}),...rows,added,category(10,'old','discuss',null,{status:'archived'})]
 deepStrictEqual(discussionTabs(changed).map(c=>c.key),['all','design','casual-games','3d','learn','casual-travel','writing-lab'])
 deepStrictEqual(discussionTabs(changed).slice(0,4).map(c=>c.label),['全部','设计','游戏','3D'])
 strictEqual(discussionCategory('design',changed),sectionScope(changed,'discuss'))
 strictEqual(discussionCategory('3d',changed),sectionScope(changed,'discuss'))
 strictEqual(discussionCategory('casual-games',changed),'casual-games')
 strictEqual(discussionTopicTag('design'),'设计')
 strictEqual(discussionTopicTag('3d'),'3d')
 strictEqual(discussionTopicTag('casual-games'),undefined)
 strictEqual(categoryHref(added,changed),'/discuss?tab=writing-lab')
 deepStrictEqual(resourceTabs(changed).map(c=>c.key),['software','resource-movies','resource-help'])
 strictEqual(sectionScope(changed,'discuss').includes('old'),false)
 strictEqual(categoryDestination(help,rows),'/resources?tab=resource-help')
 strictEqual(categoryDestination(learn,rows),'/discuss?tab=learn')
})
test('a stopped parent hides descendants and legacy routes do not reopen them',()=>{
 const changed=rows.map(c=>c.id===5?{...c,status:'archived' as const}:c)
 strictEqual(activeCategories(changed).some(c=>c.id===6),false)
 strictEqual(categoryDestination(movies,changed),'/c/resource-movies')
 deepStrictEqual(resourceTabs(changed),[{key:'software',label:'开源软件'}])
})

test('detail navigation links to the available community tabs',()=>{
 const idea = category(10,'create-effect','ideas')
 const all=[...rows,idea,category(11,'site','site')]
 const sections=navigationSections(all)
 deepStrictEqual(sections.map(s=>s.name),['创意','讨论','资源'])
 deepStrictEqual(sections.find(s=>s.key==='discuss')!.children.map(c=>c.slug),['learn','casual-travel','site'])
 deepStrictEqual(sections.find(s=>s.key==='ideas')!.children,[])
 strictEqual(categoryHref(idea,all),'/ideas')
 strictEqual(sectionScope(all,'ideas'),'create-effect')
 strictEqual(categoryHref({id:7,slug:'resource-help'},all),'/resources?tab=resource-help')
 strictEqual(categoryHref({id:2,slug:'learn-codex'},all),'/discuss?tab=learn')
 strictEqual(categoryHref({id:12,slug:'site-feedback'},all),'/discuss?tab=site')
})
test('old discussion URLs select their current board',()=>{
 const all=[...rows,category(9,'site','site')]
 strictEqual(discussionTab(all,'learn','codex'),'learn')
 strictEqual(discussionTab(all,'learn-codex'),'learn')
 strictEqual(discussionTab(all,'all','','site-feedback'),'site')
 strictEqual(discussionTab(all,'casual','casual-travel'),'casual-travel')
 strictEqual(categoryLabel({...help,name:'自定义求助板块'}),'自定义求助板块')
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
