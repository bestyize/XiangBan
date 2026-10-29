// @vitest-environment happy-dom
import {mount,flushPromises} from '@vue/test-utils';
import {beforeEach,expect,it,vi} from 'vitest';
import DesignPicker from './DesignPicker.vue';
const invoke=vi.hoisted(()=>vi.fn());vi.mock('../../bridge.ts',()=>({invoke}));
beforeEach(()=>{invoke.mockReset();});
it('searches across sessions and links the chosen original before opening',async()=>{
 const id='design-012345678901234567890123';invoke.mockImplementation(async(cmd)=>cmd==='design_list'?[{id,title:'B站',sessionId:'old',artboards:5,status:'draft'}]:{});
 const w=mount(DesignPicker,{props:{session:'new',notify:vi.fn()}});await flushPromises();
 expect(invoke).toHaveBeenCalledWith('design_list',{session:'*'});expect(w.text()).toContain('草稿');
 await w.get('input').setValue('B站');await w.get('.item').trigger('click');await flushPromises();
 expect(invoke).toHaveBeenCalledWith('design_link',{id,session:'new'});expect(w.emitted('open')?.[0]).toEqual([id]);w.unmount();
});
it('keeps a failed id in the picker instead of presenting an openable design',async()=>{
 invoke.mockImplementation((cmd)=>{if(cmd==='design_list')return [{id:'design-012345678901234567890123',title:'缺失',sessionId:'old',artboards:1}];throw Error('设计稿不存在');});
 const w=mount(DesignPicker,{props:{session:null,notify:vi.fn()}});await flushPromises();await w.get('.item').trigger('click');await flushPromises();
 expect(w.text()).toContain('设计稿不存在');expect(w.emitted('open')).toBeUndefined();w.unmount();
});
