// @vitest-environment happy-dom
import {describe,it,expect} from 'vitest';
import {copyElements,layersOf,mutateElements} from './element-commands.ts';
import {sourceDoc} from './model.ts';
const source=`<!-- @dsCard name="Card" --><html><body><x-dc><main><h1 id="title">标题</h1><button aria-labelledby="title" onclick="{{go}}">按钮</button><p>{{value}}</p><sc-for list="{{items}}"><span>{{item}}</span></sc-for></main></x-dc><script type="text/x-dc" data-dc-script>class Component extends DCLogic{renderVals(){return {value:'动态'};}}</script></body></html>`;
const id=(s:string,tag:string)=>layersOf(s).find(n=>n.tag===tag.toUpperCase())!.id;
describe('source-backed element commands',()=>{
 it('deletes source nodes, retaining programs, metadata and unrelated elements',()=>{
  const h=id(source,'h1');const {source:s}=mutateElements(source,{type:'delete',ids:[h]});expect(sourceDoc(s).querySelector('h1')).toBeNull();expect(s).toContain('@dsCard');expect(s).toContain('{{value}}');expect(sourceDoc(s).querySelector('script')!.textContent).toBe(sourceDoc(source).querySelector('script')!.textContent);
 });
 it('duplicates with fresh stable node ids and fixes internal DOM references',()=>{
  const main=id(source,'main');const fragment=copyElements(source,[main]);const p=mutateElements(source,{type:'paste',fragment});const nodes=sourceDoc(p.source).querySelectorAll('[data-ocd-node]');expect(new Set([...nodes].map(n=>n.getAttribute('data-ocd-node'))).size).toBe(nodes.length);const headers=sourceDoc(p.source).querySelectorAll('h1');expect(headers[1].id).not.toBe(headers[0].id);expect(sourceDoc(p.source).querySelectorAll('button')[1].getAttribute('aria-labelledby')).toBe(headers[1].id);
 });
 it('groups and ungroups siblings while preserving order and existing layout style',()=>{
  const ids=[id(source,'h1'),id(source,'button')];const group=mutateElements(source,{type:'group',ids});expect(sourceDoc(group.source).querySelectorAll('main > h1')).toHaveLength(1);expect(sourceDoc(group.source).querySelectorAll('[data-ocd-group]')).toHaveLength(2);expect(group.ids).toEqual(ids);const ungroup=mutateElements(group.source,{type:'ungroup',ids:group.ids});expect([...sourceDoc(ungroup.source).querySelector('main')!.children].slice(0,2).map(n=>n.localName)).toEqual(['h1','button']);
 });
 it('keeps style expressions executable by writing separate override metadata',()=>{
  const s='<x-dc><div style="{{boxStyle}}">文字</div></x-dc>';const changed=mutateElements(s,{type:'style',patches:[{id:id(s,'div'),style:{translate:'12px 18px',width:'200px'}}]});const el=sourceDoc(changed.source).querySelector('div')!;expect(el.getAttribute('style')).toBe('{{boxStyle}}');expect(el.getAttribute('data-ocd-style')).toContain('translate: 12px 18px');
 });
 it('blocks ambiguous generated edits, locked elements, program nodes and dynamic text changes',()=>{
  expect(()=>mutateElements(source,{type:'delete',ids:[id(source,'span')]})).toThrow('生成');expect(()=>mutateElements(source,{type:'text',id:id(source,'p'),text:'x'})).toThrow('动态');let locked=mutateElements(source,{type:'lock',ids:[id(source,'h1')],value:true});expect(()=>mutateElements(locked.source,{type:'delete',ids:locked.ids})).toThrow('解锁');
 });
 it('supports insertion, multiple deletion, hide/show and sibling reordering',()=>{
  let a=mutateElements('<x-dc><p>A</p><p>B</p></x-dc>',{type:'insert',kind:'ellipse'});expect(sourceDoc(a.source).querySelector('div')!.style.borderRadius).toBe('50%');a=mutateElements(a.source,{type:'hide',ids:a.ids,value:true});expect(sourceDoc(a.source).querySelector('div')!.getAttribute('data-ocd-hidden')).toBe('true');const ps=layersOf(a.source).filter(n=>n.tag==='P');a=mutateElements(a.source,{type:'reorder',ids:[ps[1].id],direction:'previous'});expect(sourceDoc(a.source).querySelector('p')!.textContent).toBe('B');a=mutateElements(a.source,{type:'delete',ids:ps.map(n=>n.id)});expect(sourceDoc(a.source).querySelectorAll('p')).toHaveLength(0);
 });
});

describe('editing safety and clipboard semantics',()=>{
 it('ungroups a legacy wrapper without changing other legacy groups',()=>{
  const s='<x-dc><div data-ocd-group="true"><p>A</p></div><div data-ocd-group="true"><p>B</p></div></x-dc>';
  const changed=mutateElements(s,{type:'ungroup',ids:[id(s,'div')]});const d=sourceDoc(changed.source);
  expect(d.querySelectorAll('[data-ocd-group="true"]')).toHaveLength(1);expect(d.querySelector('[data-ocd-group="true"]')!.textContent).toBe('B');expect(d.querySelector('x-dc > p')!.textContent).toBe('A');
 });
 it('roundtrips literal braces without turning newly typed text into a program',()=>{
  let changed=mutateElements('<x-dc><p>old</p></x-dc>',{type:'text',id:'n0',text:'{{这是普通文字}}'});expect(sourceDoc(changed.source).querySelector('p')!.getAttribute('data-ocd-literal')).toBe('true');changed=mutateElements(changed.source,{type:'text',id:'n0',text:'继续编辑'});expect(changed.source).toContain('继续编辑');
 });
 it('rejects unknown commands, program clipboard nodes and generated visibility edits',()=>{
  expect(()=>mutateElements(source,{type:'not-real'} as never)).toThrow('不支持');expect(()=>mutateElements(source,{type:'paste',fragment:{kind:'open-claude-design/elements',html:['<script>alert(1)</script>']}})).toThrow('程序节点');expect(()=>mutateElements(source,{type:'hide',ids:[id(source,'span')],value:true})).toThrow('模板');
 });
});
