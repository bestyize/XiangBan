import {readFileSync} from 'node:fs';
import {Window} from 'happy-dom';
import {afterEach,describe,expect,it,vi} from 'vitest';
const runtime=readFileSync('../../libs/agent-tools/open-claude-design/runtime/support.js','utf8');
const windows:Window[]=[];
async function preview(body:string,bundle?:{base?:string;files?:Record<string,string>;tokens?:Record<string,string>}){
  const w=new Window({url:'https://preview.test/?token=abc'});
  windows.push(w);
  const bundleScript=bundle?`<script id="__ocd_bundle__" type="application/json">${JSON.stringify(bundle).replace(/</g, '\\u003c')}</script>`:'';
  w.document.write(`<!doctype html><html><head>${bundleScript}</head><body>${body}</body></html>`);
  await w.happyDOM.whenAsyncComplete();
  const messages=vi.spyOn(w,'postMessage');
  w.eval(runtime);
  w.dispatchEvent(new w.Event('DOMContentLoaded'));
  return {w,messages};
}
afterEach(async()=>{for(const w of windows.splice(0))await w.happyDOM.close();});
describe('independent DC runtime',()=>{
 it('renders expressions, defaults, conditional and repeated content; selection does not execute actions',async()=>{
  const {w,messages}=await preview(`<x-dc><helmet><style>h1{color:red}</style></helmet><h1>{{tone}}</h1><sc-if value="{{visible}}"><p>显示</p></sc-if><sc-if value="{{count}}"><em>after</em></sc-if><sc-for list="{{items}}" as="item" index="idx"><span>{{idx}}:{{item}}</span></sc-for><button onclick="{{increment}}">{{count}}</button></x-dc><script type="text/x-dc" data-dc-script data-props='{"tone":{"editor":"enum","default":"light"}}'>class Component extends DCLogic{state={count:0};renderVals(){return {tone:this.props.tone,visible:true,items:['A','B'],count:this.state.count,increment:()=>this.setState(s=>({count:s.count+1}))};}}</script>`);
  expect(w.document.querySelector('h1')?.textContent).toBe('light');expect(w.document.querySelectorAll('sc-if,sc-for')).toHaveLength(0);expect([...w.document.querySelectorAll('span')].map(n=>n.textContent)).toEqual(['0:A','1:B']);expect(w.document.head.querySelector('style')).not.toBeNull();expect(w.document.querySelector('em')).toBeNull();
  w.document.querySelector('button')!.click();expect(w.document.querySelector('button')?.textContent).toBe('0');expect(messages.mock.calls.some(([data])=>(data as Record<string,unknown>).type==='select'&&(data as Record<string,unknown>).token==='abc')).toBe(true);
  w.eval("dispatchEvent(new MessageEvent('message',{source:parent,data:{channel:'open-claude-design',type:'mode',editing:false}}))");w.document.querySelector('button')!.click();expect(w.document.querySelector('button')?.textContent).toBe('1');expect(w.document.querySelector('em')?.textContent).toBe('after');
 });
 it('handles sc-else-if and sc-else branches',async()=>{
  const {w}=await preview(`<x-dc><sc-if value="{{status==='a'}}"><span>Branch A</span></sc-if><sc-else-if value="{{status==='b'}}"><span>Branch B</span></sc-else-if><sc-else><span>Branch C</span></sc-else></x-dc><script type="text/x-dc" data-dc-script>class Component extends DCLogic{renderVals(){return {status:'b'};}}</script>`);
  expect(w.document.querySelector('span')?.textContent).toBe('Branch B');
 });
 it('formats object style and class bindings',async()=>{
  const {w}=await preview(`<x-dc><div style="{{boxStyle}}" class="{{boxClass}}">Box</div></x-dc><script type="text/x-dc" data-dc-script>class Component extends DCLogic{renderVals(){return {boxStyle:{color:'red',fontSize:16,padding:8},boxClass:{active:true,hidden:false}}};}</script>`);
  const div=w.document.querySelector('div')!;
  expect(div.getAttribute('style')).toContain('color:red');
  expect(div.getAttribute('style')).toContain('font-size:16px');
  expect(div.getAttribute('style')).toContain('padding:8px');
  expect(div.getAttribute('class')).toBe('active');
 });
 it('supports dc-import with slots when bundled',async()=>{
  const cardSource=`<x-dc><div class="card"><h4>{{title}}</h4><slot>默认内容</slot></div></x-dc><script type="text/x-dc" data-dc-script data-props='{"title":{"default":"默认标题"}}'></script>`;
  const {w}=await preview(
    `<x-dc><dc-import src="components/Card.dc.html" title="自定义标题"><p>传入的插槽</p></dc-import></x-dc>`,
    {base:'Main.dc.html',files:{'components/Card.dc.html':cardSource}}
  );
  expect(w.document.querySelector('h4')?.textContent).toBe('自定义标题');
  expect(w.document.querySelector('.card p')?.textContent).toBe('传入的插槽');
 });
 it('reports unsupported components when missing from bundle',async()=>{
  const {messages}=await preview('<x-dc><dc-import src="Card.dc.html"></dc-import></x-dc>');
  expect(messages.mock.calls.some(([data])=>(data as Record<string,unknown>).type==='error'&&String((data as Record<string,unknown>).message).includes('dc-import'))).toBe(true);
 });
 it('updates theme and pins on message event',async()=>{
  const {w}=await preview('<x-dc><h1>Header</h1></x-dc>');
  w.eval("dispatchEvent(new MessageEvent('message',{source:parent,data:{channel:'open-claude-design',type:'theme',theme:'dark'}}))");
  expect(w.document.documentElement.dataset.theme).toBe('dark');
  w.eval("dispatchEvent(new MessageEvent('message',{source:parent,data:{channel:'open-claude-design',type:'pins',pins:[{num:1,text:'批注一'}]}}))");
  expect(w.document.querySelector('[data-ocd-pin-layer]')).not.toBeNull();
 });
});
