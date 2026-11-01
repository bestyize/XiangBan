// @vitest-environment happy-dom
import {mount} from '@vue/test-utils';
import {describe,it,expect} from 'vitest';
import ElementCode from './ElementCode.vue';
import {mutateElements} from './element-commands.ts';
describe('element Code drafts',()=>{
 it('keeps a dirty draft on concurrent updates and does not save against the newer source',async()=>{
  const w=mount(ElementCode,{props:{source:'<x-dc><p>old</p></x-dc>',nodeId:'n0',writable:true}});
  await w.get('[aria-label="元素 CSS"]').setValue('color:red');await w.setProps({source:'<x-dc><p>updated</p></x-dc>'});await w.get('form').trigger('submit');
  expect(w.emitted('save')).toBeUndefined();expect(w.emitted('notice')?.[0][0]).toContain('新版本');expect((w.get('textarea').element as HTMLTextAreaElement).value).toBe('color:red');w.unmount();
 });
 it('acknowledges a successful save while preserving expression values',async()=>{
  const source='<x-dc><p style="color:{{theme}}">text</p></x-dc>',w=mount(ElementCode,{props:{source,nodeId:'n0',writable:true}});
  await w.get('[aria-label="元素 CSS"]').setValue('color:{{theme}};padding:20px');await w.get('form').trigger('submit');
  const payload=w.emitted('save')![0][0] as any;await w.setProps({source:mutateElements(source,payload.action).source});
  expect(w.find('[role="alert"]').exists()).toBe(false);expect((w.get('[type="submit"]').element as HTMLButtonElement).disabled).toBe(true);w.unmount();
 });
});
