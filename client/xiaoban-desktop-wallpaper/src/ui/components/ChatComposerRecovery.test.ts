// @vitest-environment happy-dom
import {flushPromises,mount} from '@vue/test-utils';
import {reactive} from 'vue';
import {it,expect,vi} from 'vitest';
import {createChatStore} from "../chat/chat-store.ts";
import ChatComposer from "./ChatComposer.vue";
vi.mock("../../bridge.ts",async original=>({...await original<any>(),invoke:vi.fn(async(command:string)=>command==='get_ai_config'?{model:'test',providers:[]}:[])}));
it('retains a refused task and its attachments when a new draft was typed during sending',async()=>{
 let reject!:(error:unknown)=>void;
 const host=vi.fn(async(command:string)=>{
  if(command==='chat_new_session')return{id:'s1',title:'task',createdAt:1,updatedAt:1};
  if(command==='chat_send')return await new Promise((_resolve,fail)=>{reject=fail;});
  if(command==='chat_delete_session')return[];
  return{used:0,limit:1000,canCompact:false};
 });
 const chat=createChatStore({desktop:true,invoke:host as any,listen:vi.fn().mockResolvedValue(()=>{}),notify:vi.fn()});chat.configured=true;
 const store=reactive({desktop:true,page:'new-task',chat,creationDraft:null,notify:vi.fn(),navigate:vi.fn()});
 const wrapper=mount(ChatComposer,{props:{store:store as any,page:'new-task',pet:false},global:{stubs:{ChatWorkbenchTools:true,ChatComposerModel:true,ChatWorkspaceMenu:true,ChatPermissionMenu:true,ChatContextRing:true,ChatExpertChip:true}}});
 await flushPromises();
 const owner=chat.draft;owner.text='原任务';owner.attachments=['data:image/png;base64,YQ=='];owner.skills=[{name:'check',title:'检查'}];await flushPromises();
 await wrapper.get('[data-testid=chat-send]').trigger('click');await flushPromises();
 expect(host.mock.calls.some(([c])=>c==='chat_send')).toBe(true);
 await wrapper.get('textarea').setValue('接下来要问的新问题');
 reject(Error('服务暂不可用'));await flushPromises();
 expect(owner.text).toBe('原任务\n\n接下来要问的新问题');expect(owner.attachments).toEqual(['data:image/png;base64,YQ==']);expect(owner.skills).toEqual([{name:'check',title:'检查'}]);expect(wrapper.emitted('sent')).toBeUndefined();
 expect(Object.values(chat.views).flatMap(v=>v.items).some(i=>i.kind==='user'&&i.text.includes('原任务'))).toBe(false);
 wrapper.unmount();chat.dispose();
});
