// @vitest-environment happy-dom
import { beforeEach, describe, it, expect, vi } from 'vitest';
import { createWidgetClient } from './client.ts';
vi.mock('../bridge.ts',()=>({desktop:false,invoke:vi.fn()}));
beforeEach(()=>localStorage.clear());
describe('widget collection persistence',()=>{
  it('keeps several instances, per-instance settings and positions after reopening',async()=>{
    const client=createWidgetClient();await client.load();
    await Promise.all([client.add('calendar'),client.add('calendar'),client.add('fortune')]);
    expect(client.layout.value.widgets).toHaveLength(3);const first=client.layout.value.widgets[0].id;
    await client.update(first,{x:.1,y:.8,theme:'paper',scale:1.25});await client.edit(true);await client.edit(false);client.dispose();
    const reopened=createWidgetClient();await reopened.load();expect(reopened.layout.value.widgets[0]).toMatchObject({id:first,x:.1,y:.8,theme:'paper',scale:1.25});expect(reopened.layout.value.editing).toBe(false);
    await reopened.remove(first);expect(reopened.layout.value.widgets).toHaveLength(2);reopened.dispose();
  });
  it('preserves unreadable layouts instead of replacing them',async()=>{
    localStorage.setItem('xiaoban.widgets.v1','broken');const client=createWidgetClient();await client.load();
    expect(client.ready.value).toBe(false);await expect(client.add('fortune')).rejects.toThrow();expect(localStorage.getItem('xiaoban.widgets.v1')).toBe('broken');client.dispose();
  });
  it('does not report success when saving fails',async()=>{
    const client=createWidgetClient();await client.load();vi.stubGlobal('localStorage',{getItem:()=>null,setItem:()=>{throw new Error('没有空间');}});
    try{await expect(client.add('weather')).rejects.toThrow('没有空间');expect(client.layout.value.widgets).toHaveLength(0);expect(client.error.value).toContain('没有空间');}finally{vi.unstubAllGlobals();client.dispose();}
  });
});
