import test from 'node:test'
import assert from 'node:assert/strict'
import { navigationCategories, navigationGroups, navigationLogo, navigationMatches } from '../src/lib/navigationDirectory.ts'
import { webLink } from '../src/components/community/contract.ts'
const groups = (data: Parameters<typeof navigationGroups>[0]) => navigationGroups(data, url => !!webLink(url))

test('legacy entries are reclassified by purpose without losing edits or duplicating product URLs', () => {
  const custom = {key:'grok',name:'Grok 已编辑',vendor:'xAI',url:'https://grok.com/'}
  const data = {groups:[{name:'海外',items:[custom,{...custom,key:'custom-copilot',name:'Copilot',url:'https://copilot.microsoft.com'}]}]}
  const result = groups(data)
  assert.deepEqual(result.map(group=>group.name), ['大模型','Harness','Skill','API'])
  const entries = result.flatMap(group=>group.items)
  assert.equal(entries.find(item=>item.key==='grok')?.name,'Grok 已编辑')
  assert.equal(entries.find(item=>item.key==='grok')?.category,'harness')
  assert.equal(entries.find(item=>item.key==='openai-models')?.category,'models')
  assert.equal(entries.filter(item=>item.category==='models').some(item=>item.key==='chatgpt'),false)
  assert.equal(entries.filter(item=>item.url.replace(/\/$/,'')==='https://copilot.microsoft.com').length,1)
  assert.equal(data.groups[0]!.items.length,2)
})

test('v1 removals survive the taxonomy upgrade and v2 empty publication stays empty', () => {
  const entries = groups({catalog_version:1,groups:[]}).flatMap(group=>group.items)
  assert.ok(!entries.some(item=>item.key==='grok'))
  assert.ok(entries.some(item=>item.key==='hermes-agent'))
  assert.deepEqual(groups({catalog_version:2,groups:[]}),[])
  assert.deepEqual(groups({catalog_version:2,groups:[{name:'API',items:[{key:'bad',name:'Bad',url:'javascript:alert(1)',category:'api'}]}]}),[])
})

test('published category, order and ownership remain authoritative; filters match service types', () => {
  const row={key:'one',name:'One',url:'https://example.com/',category:'api',subtype:'inference',vendor:'Example'}
  const entries = groups({catalog_version:2,groups:[{name:'API',items:[row,{...row,key:'two'}]}]}).flatMap(group=>group.items)
  assert.deepEqual(entries.map(item=>item.key),['one','two'])
  assert.equal(navigationMatches(row,'api','thirdparty'),true)
  assert.equal(navigationMatches({...row,subtype:'router'},'api','thirdparty'),true)
  assert.equal(navigationMatches({...row,subtype:'cloud'},'api','thirdparty'),false)
  assert.equal(navigationMatches({...row,category:'models',region:'cn'},'models','cn'),true)
  assert.equal(navigationMatches(row,'harness'),false)
  assert.equal(navigationCategories.length,4)
  assert.equal(navigationLogo({key:'yuanbao',name:'腾讯元宝',vendor:'腾讯',url:'https://yuanbao.tencent.com/'}),'Tencent')
  assert.equal(navigationLogo({key:'yuanbao',name:'其他项目',vendor:'其他厂商',url:'https://example.com/'}),'其他项目')
})
