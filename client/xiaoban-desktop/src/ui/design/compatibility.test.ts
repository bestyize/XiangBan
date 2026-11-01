import {readFileSync,readdirSync} from 'node:fs';
import {Window} from 'happy-dom';
import {afterEach,describe,expect,it,vi} from 'vitest';
const runtime=readFileSync('../../libs/agent-tools/open-claude-design/runtime/vendor/react-bridge.js','utf8')+'\n'+readFileSync('../../libs/agent-tools/open-claude-design/runtime/support.js','utf8');
const windows:Window[]=[];
async function preview(source:string,bundle:Record<string,unknown>={}){
 const w=new Window({url:'https://preview.test/?token=abc',settings:{disableJavaScriptFileLoading:true,disableCSSFileLoading:true}});windows.push(w);
 w.document.write(source);await w.happyDOM.whenAsyncComplete();
 const data=w.document.createElement('script');data.id='__ocd_bundle__';data.type='application/json';data.textContent=JSON.stringify(bundle);w.document.head.append(data);
 const messages=vi.spyOn(w,'postMessage');w.eval(runtime);w.dispatchEvent(new w.Event('DOMContentLoaded'));
 const errors=()=>messages.mock.calls.map(([v])=>v as any).filter(v=>v.type==='error').map(v=>v.message);
 return {w,errors};
}
afterEach(async()=>{for(const w of windows.splice(0))await w.happyDOM.close();});
const play=(w:Window)=>w.eval("dispatchEvent(new MessageEvent('message',{source:parent,data:{channel:'open-claude-design',type:'mode',editing:false}}))");
describe('Claude Design source compatibility',()=>{
 const fixture='docs/design-specs/ui-upgrade/';
 const sources=Object.fromEntries(readdirSync(fixture).filter(n=>n.endsWith('.dc.html')).map(n=>[n,readFileSync(fixture+n,'utf8')]));
 for(const [file,source] of Object.entries(sources))it('renders original '+file,async()=>{
  const {w,errors}=await preview(source,{base:file,files:sources});
  expect(errors()).toEqual([]);
  expect(w.document.querySelector('x-dc')?.textContent?.trim().length).toBeGreaterThan(10);
  expect(w.document.querySelectorAll('dc-import,sc-for,sc-if')).toHaveLength(0);
 });
 it('keeps independent component state and lifecycle across parent updates and unmounts removed children',async()=>{
  const child=`<x-dc><button onClick="{{inc}}">{{label}}:{{count}}</button><slot></slot></x-dc><script type="text/x-dc" data-dc-script>class Component extends DCLogic{state={count:0};componentDidMount(){window.mounts=(window.mounts||0)+1;}componentWillUnmount(){window.unmounts=(window.unmounts||0)+1;}renderVals(){return {label:this.props.someLabel,count:this.state.count,inc:()=>this.setState(s=>({count:s.count+1}),()=>window.callbackCount=this.state.count)}}}</script>`;
  const {w,errors}=await preview(`<x-dc><button id="parent" onclick="{{update}}">Parent</button><dc-import name="Counter" some-label="First"></dc-import><sc-if value="{{show}}"><dc-import name="Counter" dc-props="{{ {someLabel:'Second'} }}"><a onclick="{{update}}">slot</a></dc-import></sc-if></x-dc><script type="text/x-dc" data-dc-script>class Component extends DCLogic{state={show:true};renderVals(){return {show:this.state.show,update:()=>this.setState({show:false})}}}</script>`,{base:'Main.dc.html',files:{'Counter.dc.html':child}});
  expect(errors()).toEqual([]);play(w);expect((w as any).mounts).toBe(2);
  w.document.querySelectorAll('button')[1].click();
  expect(w.document.querySelectorAll('button')[1].textContent).toBe('First:1');
  expect(w.document.querySelectorAll('button')[2].textContent).toBe('Second:0');expect((w as any).callbackCount).toBe(1);
  w.document.querySelector('a')!.click();expect(w.document.querySelectorAll('button')).toHaveLength(2);
  expect(w.document.querySelectorAll('button')[1].textContent).toBe('First:1');
  expect((w as any).mounts).toBe(2);expect((w as any).unmounts).toBe(1);
 });
 it('supports object and callback expressions, controlled input, boolean props and pseudo styles',async()=>{
  const {w,errors}=await preview(`<x-dc><button id="btn" style="{{ {color: 'red', padding: 4} }}" style-focus="outline:2px solid blue" onClick="{{() => set('done')}}">{{value}} / {{ ({label:'brace }'}).label }}</button><textarea value="{{value}}" onChange="{{e => set(e.target.value)}}"></textarea><input readonly="{{false}}"/></x-dc><script type="text/x-dc" data-dc-script>class Component extends DCLogic{state={value:'start'};renderVals(){return {value:this.state.value,set:value=>this.setState({value})}}}</script>`);
  expect(errors()).toEqual([]);play(w);w.document.querySelector('button')!.click();expect(w.document.querySelector('textarea')!.value).toBe('done');
  const area=w.document.querySelector('textarea')!;area.value='typed';area.dispatchEvent(new w.Event('input',{bubbles:true}));
  expect(w.document.querySelector('#btn')!.textContent).toBe('typed / brace }');expect(w.document.querySelector('[readonly]')).toBeNull();expect(w.document.head.textContent).toContain(':focus{outline:2px solid blue}');
 });
 it('resolves dynamic image and CSS assets relative to imported components and preserves helmet',async()=>{
  const image='data:image/png;base64,iVBORw0KGgo=';
  const {w,errors}=await preview('<x-dc><dc-import name="parts/Card"></dc-import></x-dc>',{base:'Main.dc.html',files:{'parts/Card.dc.html':`<x-dc><helmet><style>.card{color:blue}</style></helmet><img src="{{image}}"/><div style="background-image:url(../_blob/a)">Card</div></x-dc><script type="text/x-dc" data-dc-script>class Component extends DCLogic{renderVals(){return {image:'/_blob/a'}}}</script>`},assets:{'_blob/a':image}});
  expect(errors()).toEqual([]);expect(w.document.querySelector('img')!.getAttribute('src')).toBe(image);expect(w.document.head.textContent).toContain('.card{color:blue}');expect(w.document.querySelector('[style*="background-image"]')!.getAttribute('style')).toContain(image);
 });
 it('restores serialized official HTML events, controlled inputs and SVG attribute casing',async()=>{
  const {w,errors}=await preview(`<x-dc><button sc-camel-on-click="{{() => set('clicked')}}">{{value}}</button><input value="{{value}}" sc-camel-on-change="{{e => set(e.target.value)}}"/><svg sc-camel-view-box="0 0 24 24" sc-camel-on-click="{{() => set('svg')}}"></svg></x-dc><script type="text/x-dc" data-dc-script>class Component extends DCLogic{state={value:'start'};renderVals(){return {value:this.state.value,set:value=>this.setState({value})}}}</script>`);
  w.document.querySelector('button')!.click();expect(w.document.querySelector('button')!.textContent).toBe('start');
  play(w);w.document.querySelector('button')!.click();expect(w.document.querySelector('button')!.textContent).toBe('clicked');
  const input=w.document.querySelector('input')!;input.value='typed';input.dispatchEvent(new w.Event('input',{bubbles:true}));
  expect(w.document.querySelector('button')!.textContent).toBe('typed');
  const svg=w.document.querySelector('svg')!;expect(svg.getAttribute('viewBox')).toBe('0 0 24 24');
  svg.dispatchEvent(new w.Event('click'));expect(w.document.querySelector('button')!.textContent).toBe('svg');expect(errors()).toEqual([]);
 });
 it('keeps a real imported market screen interactive',async()=>{
  const {w,errors}=await preview(sources['WB-Market-Review.dc.html'],{base:'WB-Market-Review.dc.html',files:sources});play(w);
  const checkbox=w.document.querySelector('input[type="checkbox"]');expect(checkbox).not.toBeNull();(checkbox as unknown as {click():void}).click();
  expect(errors()).toEqual([]);expect(w.document.querySelector('input[type="checkbox"]')?.hasAttribute('checked')).toBe(true);
 });
 it('executes local TSX components with hooks, module imports and props without network dependencies',async()=>{
  const {w,errors}=await preview(`<x-dc><x-import from="./ui/Counter.tsx" component="Counter" dc-props="{{ {label:'Count'} }}"></x-import></x-dc>`,{base:'Main.dc.html',files:{
   'ui/Counter.tsx':`import React,{useState,useEffect} from 'react';import {step} from './step';export function Counter({label}:{label:string}){const [count,setCount]=useState<number>(0);useEffect(()=>{window.reactMounted=true;return ()=>{window.reactUnmounted=true;}},[]);return <button onClick={()=>setCount(count+step)}>{label}:{count}</button>;}`,
   'ui/step.ts':'export const step:number=2;',
  }});
  expect(errors()).toEqual([]);expect(w.document.querySelector('button')!.textContent).toBe('Count:0');
  w.document.querySelector('button')!.click();await w.happyDOM.whenAsyncComplete();expect(w.document.querySelector('button')!.textContent).toBe('Count:0');
  play(w);w.document.querySelector('button')!.click();await w.happyDOM.whenAsyncComplete();expect(w.document.querySelector('button')!.textContent).toBe('Count:2');expect((w as any).reactMounted).toBe(true);
  w.dispatchEvent(new w.Event('unload'));expect((w as any).reactUnmounted).toBe(true);
 });
 it('reports unsupported external React dependencies instead of silently rendering an empty board',async()=>{
  const {errors}=await preview('<x-dc><x-import from="https://example.test/Widget.jsx" component="Widget"></x-import></x-dc>',{base:'Main.dc.html',files:{}});
  expect(errors().join(' ')).toContain('不支持外部模块');
 });

});
