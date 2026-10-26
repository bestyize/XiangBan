// @vitest-environment happy-dom
import {describe,it,expect} from 'vitest';
import {addBoard,boardsOf,changeBoard,clone,patchNode,patchProp,propsOf,removeBoard,sourceDoc,targetFile,type Project} from './model.ts';
export const fixture=():Project=>({id:'design-012345678901234567890123',title:'多屏作品',sessionId:'s1',revision:1,updatedMs:1,canvas:{v:3,boards:{'Main.dc.html':{x:0,y:0,w:390,h:844,title:'首页',custom:{keep:true}}},order:['Main.dc.html'],notes:{},unknown:123},files:{'Main.dc.html':`<!doctype html><html><head></head><body><x-dc><h1>标题</h1><p>{{title}}</p></x-dc><script type="text/x-dc" data-dc-script data-props='{"tone":{"editor":"enum","default":"light","options":["light","dark"]}}'>class Component extends DCLogic { renderVals(){ return {title:'动态文字'}; } }</script></body></html>`}});
describe('design source adapter',()=>{
 it('edits v3 and legacy indexes while retaining unknown source and canvas metadata',()=>{
  for(const legacy of [false,true]){const p=fixture();if(legacy){p.canvas={artboards:boardsOf(p.canvas),pages:[{id:'p1'}],unknown:123};}
   changeBoard(p,'Main.dc.html',{title:'重命名',w:420});const copy=addBoard(p,boardsOf(p.canvas)[0]);expect(boardsOf(p.canvas)).toHaveLength(2);expect(p.files[copy]).toBe(p.files['Main.dc.html']);removeBoard(p,copy);expect(p.files[copy]).toBeDefined();expect(p.canvas.unknown).toBe(123);expect(boardsOf(p.canvas)[0].custom).toEqual({keep:true});expect(boardsOf(p.canvas)[0].w).toBe(420);expect(()=>removeBoard(p,'Main.dc.html')).toThrow('至少');}
 });
 it('patches a leaf without serializing the rendered DOM over the DC program',()=>{
  const p=fixture(),source=p.files['Main.dc.html'];const edited=patchNode(source,'n0',{text:'手动标题',style:{color:'#ff0000'}});const d=sourceDoc(edited);
  expect(d.querySelector('h1')?.textContent).toBe('手动标题');expect(d.querySelector('p')?.textContent).toBe('{{title}}');expect(d.querySelector('script')?.textContent).toBe(sourceDoc(source).querySelector('script')?.textContent);
  expect(()=>patchNode(edited,'n1',{text:'覆盖变量'})).toThrow('动态');expect(()=>patchNode(edited,'missing',{text:'x'})).toThrow('重新选择');
  const changed=patchProp(edited,'tone','dark');expect(propsOf(changed).tone).toEqual({editor:'enum',default:'dark',options:['light','dark']});expect(sourceDoc(changed).querySelector('h1')?.textContent).toBe('手动标题');
 });
 it('resolves only links to boards belonging to the same project',()=>{
  expect(targetFile('pages/Main.dc.html','../Settings.dc.html',['Settings.dc.html'])).toBe('Settings.dc.html');
  for(const url of ['https://example.com','file:///private','#anchor','/Settings.dc.html','missing.dc.html'])expect(targetFile('Main.dc.html',url,['Settings.dc.html'])).toBeNull();
  expect(clone(fixture())).toEqual(fixture());
 });
});
