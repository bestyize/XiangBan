// @vitest-environment happy-dom
import {describe,it,expect,beforeEach,vi} from 'vitest';
vi.mock('../bridge.ts',()=>({desktop:false,invoke:vi.fn()}));
import {createWidgetClient} from './client.ts';
describe('new widget persistence',()=>{
 beforeEach(()=>localStorage.clear());
 it('starts empty and leaves v1 records out of the new runtime',async()=>{localStorage.setItem('xiaoban.widgets.v1',JSON.stringify({version:1,widgets:[{kind:'fortune'}]}));const c=createWidgetClient();await c.load();expect(c.ready.value).toBe(true);expect(c.layout.value.widgets).toEqual([]);await c.add('woodfish');expect(JSON.parse(localStorage.getItem('xiaoban.widgets.v2')!).version).toBe(2);expect(localStorage.getItem('xiaoban.widgets.v1')).toContain('fortune');c.dispose();});
 it('persists independent instances and rejects invalid updates without replacing storage',async()=>{const c=createWidgetClient();await c.load();await c.add('notes');await c.add('notes');const [a,b]=c.layout.value.widgets;await c.update(a.id,{note:{title:'待办',mode:'todo',text:'',items:[{id:'t1',text:'写周报',done:true,time:'09:00'}]}});expect(c.layout.value.widgets[1].note?.items).toEqual([]);const stored=localStorage.getItem('xiaoban.widgets.v2');await expect(c.update(b.id,{scale:9})).rejects.toThrow();expect(localStorage.getItem('xiaoban.widgets.v2')).toBe(stored);c.dispose();});
});
