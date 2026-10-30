// @vitest-environment happy-dom
import {readFileSync} from 'node:fs';
import {describe,it,expect,vi} from 'vitest';

const shell=readFileSync('public/widgets/sandbox.html','utf8');
describe('custom widget isolated shell',()=>{
 it('accepts HTML only from its parent and initializes just once',()=>{
  const listeners=new Map<string,(event:unknown)=>void>();
  const parent={postMessage:vi.fn()},doc={open:vi.fn(),write:vi.fn(),close:vi.fn()};
  const add=(type:string,fn:(event:unknown)=>void)=>listeners.set(type,fn);
  const remove=(type:string)=>listeners.delete(type);
  const script=new DOMParser().parseFromString(shell,'text/html').querySelector('script')!.textContent!;
  new Function('parent','document','addEventListener','removeEventListener',script)(parent,doc,add,remove);
  expect(parent.postMessage).toHaveBeenCalledWith({type:'xiaoban-widget-shell-ready'},'*');
  const send=(source:unknown,data:unknown)=>listeners.get('message')?.({source,data});
  const init={type:'xiaoban-widget-init',html:'<button>+1</button><script>window.count=0</script>'};
  send({},init);send(parent,{...init,type:'other'});send(parent,{...init,html:0});send(parent,{...init,html:'x'.repeat(256*1024+1)});
  expect(doc.open).not.toHaveBeenCalled();
  send(parent,init);send(parent,init);
  expect(doc.open).toHaveBeenCalledOnce();expect(doc.write).toHaveBeenCalledExactlyOnceWith(init.html);expect(doc.close).toHaveBeenCalledOnce();
 });
 it('allows widget scripts only in the unprivileged document, keeping host policy strict',()=>{
  const doc=new DOMParser().parseFromString(shell,'text/html');
  const csp=doc.querySelector('meta[http-equiv="Content-Security-Policy"]')!.getAttribute('content')!;
  expect(csp).toContain("script-src 'unsafe-inline'");
  for(const directive of ['connect-src','frame-src','object-src','base-uri','form-action'])expect(csp).toContain(`${directive} 'none'`);
  const rust=readFileSync('src-tauri/src/library.rs','utf8');
  expect(rust).toContain(`.header("Content-Security-Policy","${csp}")`);
  expect(rust).toContain('include_bytes!("../../public/widgets/sandbox.html")');
  const config=JSON.parse(readFileSync('src-tauri/tauri.conf.json','utf8'));
  expect(config.app.security.csp).toContain("script-src 'self';");
 });
 it('verifies builtin pelican-bike widget pack validity and safe sandbox embedding',()=>{
  const pelicanJson = JSON.parse(readFileSync('public/widgets/pelican-bike.json', 'utf8'));
  expect(pelicanJson.schemaVersion).toBe(1);
  expect(pelicanJson.id).toBe('pelican-bike');
  expect(pelicanJson.name).toBe('鹈鹕骑车');
  expect(pelicanJson.width).toBe(300);
  expect(pelicanJson.height).toBe(260);
  expect(pelicanJson.theme).toBe('transparent');
  expect(pelicanJson.html).toContain('慢慢骑');
  expect(pelicanJson.html).toContain('id="stage"');
 });
});
