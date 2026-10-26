import {readFileSync} from 'node:fs';
import {Window} from 'happy-dom';
import {afterEach,describe,expect,it,vi} from 'vitest';
const runtime=readFileSync('../../libs/agent-tools/open-claude-design/runtime/support.js','utf8');
const windows:Window[]=[];
async function preview(body:string){const w=new Window({url:'https://preview.test/?token=abc'});windows.push(w);w.document.write(`<!doctype html><html><body>${body}</body></html>`);await w.happyDOM.whenAsyncComplete();const messages=vi.spyOn(w,'postMessage');w.eval(runtime);w.dispatchEvent(new w.Event('DOMContentLoaded'));return {w,messages};}
afterEach(async()=>{for(const w of windows.splice(0))await w.happyDOM.close();});
describe('independent DC runtime',()=>{
 it('renders expressions, defaults, conditional and repeated content; selection does not execute actions',async()=>{
  const {w,messages}=await preview(`<x-dc><helmet><style>h1{color:red}</style></helmet><h1>{{tone}}</h1><sc-if value="{{visible}}"><p>显示</p></sc-if><sc-if value="{{count}}"><em>after</em></sc-if><sc-for list="{{items}}" as="item"><span>{{item}}</span></sc-for><button onclick="{{increment}}">{{count}}</button></x-dc><script type="text/x-dc" data-dc-script data-props='{"tone":{"editor":"enum","default":"light"}}'>class Component extends DCLogic{state={count:0};renderVals(){return {tone:this.props.tone,visible:true,items:['A','B'],count:this.state.count,increment:()=>this.setState(s=>({count:s.count+1}))};}}</script>`);
  expect(w.document.querySelector('h1')?.textContent).toBe('light');expect(w.document.querySelectorAll('sc-if,sc-for')).toHaveLength(0);expect([...w.document.querySelectorAll('span')].map(n=>n.textContent)).toEqual(['A','B']);expect(w.document.head.querySelector('style')).not.toBeNull();expect(w.document.querySelector('em')).toBeNull();
  w.document.querySelector('button')!.click();expect(w.document.querySelector('button')?.textContent).toBe('0');expect(messages.mock.calls.some(([data])=>(data as Record<string,unknown>).type==='select'&&(data as Record<string,unknown>).token==='abc')).toBe(true);
  w.eval("dispatchEvent(new MessageEvent('message',{source:parent,data:{channel:'open-claude-design',type:'mode',editing:false}}))");w.document.querySelector('button')!.click();expect(w.document.querySelector('button')?.textContent).toBe('1');expect(w.document.querySelector('em')?.textContent).toBe('after');
 });
 it('reports unsupported components instead of silently showing an empty board',async()=>{
  const {messages}=await preview('<x-dc><dc-import src="Card.dc.html"></dc-import></x-dc>');expect(messages.mock.calls.some(([data])=>(data as Record<string,unknown>).type==='error'&&String((data as Record<string,unknown>).message).includes('dc-import'))).toBe(true);
 });
});
