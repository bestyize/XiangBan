import {readFileSync} from 'node:fs';
import {Window} from 'happy-dom';
import {afterEach,describe,expect,it,vi} from 'vitest';
const runtime=readFileSync('../../libs/agent-tools/open-claude-design/runtime/support.js','utf8')+'\n'+readFileSync('../../libs/agent-tools/open-claude-design/runtime/editor.js','utf8');
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
 it('forwards editing wheel events while leaving normal prototype scrolling available in Play',async()=>{
  const {w,messages}=await preview('<x-dc><p>page</p></x-dc>');
  const wheel=new w.WheelEvent('wheel',{deltaY:80,cancelable:true});Object.defineProperty(wheel,'ctrlKey',{value:true});w.document.dispatchEvent(wheel);
  expect(wheel.defaultPrevented).toBe(true);expect(messages).toHaveBeenCalledWith(expect.objectContaining({type:'canvas-wheel',dy:80,ctrl:true}),'*');
  w.eval("dispatchEvent(new MessageEvent('message',{source:parent,data:{channel:'open-claude-design',type:'mode',editing:false}}))");
  const scroll=new w.WheelEvent('wheel',{deltaY:40,cancelable:true});w.document.dispatchEvent(scroll);expect(scroll.defaultPrevented).toBe(false);
 });

});

describe('canvas direct editing protocol',()=>{
 const mode=(w:Window,editing=true)=>w.eval(`dispatchEvent(new MessageEvent('message',{source:parent,data:{channel:'open-claude-design',type:'mode',editing:${editing},writable:true,zoom:1,guides:false}}))`);
 const last=(m:{mock:{calls:unknown[][]}},type:string)=>m.mock.calls.map(([v])=>v as Record<string,any>).filter(v=>v.type===type).at(-1);
 it('draws selection handles and forwards delete/duplicate/undo instead of mutating program source',async()=>{
  const {w,messages}=await preview('<x-dc><h1>标题</h1><p>正文</p></x-dc>');mode(w);const h=w.document.querySelector('h1')!;h.click();expect(w.document.querySelectorAll('[data-handle]')).toHaveLength(8);
  w.document.dispatchEvent(new w.KeyboardEvent('keydown',{key:'Delete',bubbles:true}));expect(last(messages,'editor-command')?.command).toBe('delete');expect(h.isConnected).toBe(true);
  w.document.dispatchEvent(new w.KeyboardEvent('keydown',{key:'d',ctrlKey:true,bubbles:true}));expect(last(messages,'editor-command')?.command).toBe('duplicate');w.document.dispatchEvent(new w.KeyboardEvent('keydown',{key:'z',metaKey:true,bubbles:true}));expect(last(messages,'editor-command')?.command).toBe('undo');
 });
 it('supports shift multiselect and a cancellable inline text editor',async()=>{
  const {w,messages}=await preview('<x-dc><h1>标题</h1><p>正文</p></x-dc>');mode(w);const h=w.document.querySelector('h1')!,p=w.document.querySelector('p')!;h.click();p.dispatchEvent(new w.MouseEvent('click',{shiftKey:true,bubbles:true}));expect(last(messages,'select')?.nodes).toHaveLength(2);
  h.dispatchEvent(new w.MouseEvent('dblclick',{bubbles:true}));expect(last(messages,'editor-command')?.command).toBe('showProperties');expect(h.getAttribute('contenteditable')).toBe('plaintext-only');h.textContent='直接改';h.dispatchEvent(new w.KeyboardEvent('keydown',{key:'Enter',bubbles:true}));expect(last(messages,'element-edit')?.action).toMatchObject({type:'text',text:'直接改'});
  h.dispatchEvent(new w.MouseEvent('dblclick',{bubbles:true}));h.textContent='取消';h.dispatchEvent(new w.KeyboardEvent('keydown',{key:'Escape',bubbles:true}));expect(h.textContent).toBe('直接改');
 });
 it('commits drag and resize once on pointerup and rolls back cancelled gestures',async()=>{
  const {w,messages}=await preview('<x-dc><div style="position:absolute;width:100px;height:60px">盒子</div></x-dc>');mode(w);const el=w.document.querySelector('x-dc div')!;vi.spyOn(el,'getBoundingClientRect').mockReturnValue({left:10,top:20,right:110,bottom:80,x:10,y:20,width:100,height:60,toJSON:()=>({})} as never);
  el.dispatchEvent(new w.PointerEvent('pointerdown',{button:0,pointerId:1,clientX:30,clientY:30,bubbles:true}));w.document.dispatchEvent(new w.PointerEvent('pointermove',{pointerId:1,clientX:50,clientY:40,bubbles:true}));expect(last(messages,'element-edit')).toBeUndefined();w.document.dispatchEvent(new w.PointerEvent('pointerup',{pointerId:1,bubbles:true}));expect(last(messages,'element-edit')?.action.patches[0].style.translate).toBe('20px 10px');
  const handle=w.document.querySelector('[data-handle="se"]')!;handle.dispatchEvent(new w.PointerEvent('pointerdown',{button:0,pointerId:2,clientX:110,clientY:80,bubbles:true}));w.document.dispatchEvent(new w.PointerEvent('pointermove',{pointerId:2,clientX:130,clientY:100,bubbles:true}));w.document.dispatchEvent(new w.PointerEvent('pointerup',{pointerId:2,bubbles:true}));expect(last(messages,'element-edit')?.action.patches[0].style).toMatchObject({width:'120px',height:'80px'});
  const count=messages.mock.calls.length;el.dispatchEvent(new w.PointerEvent('pointerdown',{button:0,pointerId:3,clientX:30,clientY:30,bubbles:true}));w.document.dispatchEvent(new w.PointerEvent('pointermove',{pointerId:3,clientX:100,clientY:90,bubbles:true}));w.document.dispatchEvent(new w.PointerEvent('pointercancel',{pointerId:3,bubbles:true}));expect(messages.mock.calls.slice(count).some(([v])=>(v as any).type==='element-edit')).toBe(false);
 });
 it('protects generated text and leaves Play controls interactive',async()=>{
  const {w,messages}=await preview('<x-dc><h1>{{title}}</h1><button onclick="{{click}}">{{count}}</button></x-dc><script type="text/x-dc" data-dc-script>class Component extends DCLogic{state={count:0};renderVals(){return {title:"动态",count:this.state.count,click:()=>this.setState(s=>({count:s.count+1}))};}}</script>');mode(w);w.document.querySelector('h1')!.dispatchEvent(new w.MouseEvent('dblclick',{bubbles:true}));expect(last(messages,'editor-notice')?.message).toContain('动态');expect(w.document.querySelector('[contenteditable]')).toBeNull();mode(w,false);w.document.querySelector('button')!.click();expect(w.document.querySelector('button')?.textContent).toBe('1');expect(w.document.querySelectorAll('[data-handle]')).toHaveLength(0);
 });
 it('preserves literal text and style overrides through hover effects',async()=>{
  const {w}=await preview('<x-dc><p data-ocd-literal="true">{{literal}}</p><button style="color:red" style-hover="color:blue" data-ocd-style="translate:24px 10px">按钮</button></x-dc>');expect(w.document.querySelector('p')!.textContent).toBe('{{literal}}');const button=w.document.querySelector('button')!;button.dispatchEvent(new w.MouseEvent('mouseenter'));expect(button.style.translate).toBe('24px 10px');button.dispatchEvent(new w.MouseEvent('mouseleave'));expect(button.style.translate).toBe('24px 10px');
 });
 it('reorders flex children for Alt+arrow and passes focus navigation to the parent',async()=>{
  const {w,messages}=await preview('<x-dc><main style="display:flex"><p>A</p><p>B</p></main></x-dc>');mode(w);w.document.querySelector('p')!.click();w.document.dispatchEvent(new w.KeyboardEvent('keydown',{key:'ArrowRight',altKey:true,bubbles:true}));expect(last(messages,'element-edit')?.action).toMatchObject({type:'reorder',direction:'next'});w.document.dispatchEvent(new w.KeyboardEvent('keydown',{key:'F6',bubbles:true}));expect(last(messages,'editor-command')?.command).toBe('focusNext');
 });
 it('does not commit inline text when Enter only confirms IME composition',async()=>{
  const {w,messages}=await preview('<x-dc><p>标题</p></x-dc>');mode(w);const p=w.document.querySelector('p')!;p.dispatchEvent(new w.MouseEvent('dblclick',{bubbles:true}));p.textContent='中文输入';p.dispatchEvent(new w.KeyboardEvent('keydown',{key:'Enter',isComposing:true,bubbles:true}));expect(last(messages,'element-edit')).toBeUndefined();expect(p.hasAttribute('contenteditable')).toBe(true);p.dispatchEvent(new w.KeyboardEvent('keydown',{key:'Enter',bubbles:true}));expect(last(messages,'element-edit')?.action.text).toBe('中文输入');
 });
 it('keeps inline editing active for the legacy WebView IME confirmation key',async()=>{
  const {w,messages}=await preview('<x-dc><p>标题</p></x-dc>');mode(w);const p=w.document.querySelector('p')!;p.dispatchEvent(new w.MouseEvent('dblclick',{bubbles:true}));p.textContent='中文';p.dispatchEvent(new w.KeyboardEvent('keydown',{key:'Enter',keyCode:229,bubbles:true}));expect(last(messages,'element-edit')).toBeUndefined();expect(p.hasAttribute('contenteditable')).toBe(true);
 });

});
