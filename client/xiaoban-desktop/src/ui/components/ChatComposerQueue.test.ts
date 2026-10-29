// @vitest-environment happy-dom
import {flushPromises,mount} from '@vue/test-utils';
import {reactive} from 'vue';
import {describe,it,expect,vi} from 'vitest';
import ChatComposer from './ChatComposer.vue';
import {createChatStore} from '../chat/chat-store.ts';
import {newView} from '../chat/view.ts';
const native = vi.hoisted(() => vi.fn(async(command:string):Promise<unknown> => command === 'chat_screenshot' ? 'data:image/png;base64,YQ==' : command === 'get_ai_config' ? {model:'test',providers:[]} : []));
vi.mock('../../bridge.ts',async original=>({...await original<any>(),invoke:native}));
function setup() {
  const host=vi.fn(async(command:string) => command==='chat_enqueue' ? {revision:1,paused:false,reason:'',enqueuedId:'q1',items:[{id:'q1',text:'next',images:1,files:[],mode:'queue',dispatching:false}]} : undefined);
  const chat=createChatStore({desktop:true,invoke:host as any,listen:vi.fn().mockResolvedValue(()=>{}),notify:vi.fn()});
  chat.currentId='s1';chat.sessions=[{id:'s1',title:'chat',createdAt:1,updatedAt:1}];chat.views.s1=newView('s1',true);chat.views.s1.running=true;chat.configured=true;
  const store=reactive({desktop:true,page:'chat',chat,creationDraft:null,notify:vi.fn(),navigate:vi.fn()});
  const wrapper=mount(ChatComposer,{props:{store:store as any,page:'chat',pet:false},global:{stubs:{ChatWorkbenchTools:{props:['fileDisabled','screenshotDisabled'],emits:['screenshot'],template:'<button data-testid="shot" :disabled="screenshotDisabled" @click="$emit(\'screenshot\')">shot</button>'},ChatComposerModel:true,ChatWorkspaceMenu:true,ChatPermissionMenu:true,ChatContextRing:true,ChatExpertChip:true}}});
  return{host,chat,wrapper};
}
describe('composing while a reply is running',()=>{
  it('accepts a screenshot and Enter queues it without stopping the active reply',async()=>{
    const h=setup();await flushPromises();expect(h.wrapper.get('[data-testid=shot]').attributes('disabled')).toBeUndefined();
    await h.wrapper.get('[data-testid=shot]').trigger('click');await flushPromises();expect(h.chat.draft.attachments).toHaveLength(1);
    await h.wrapper.get('textarea').setValue('next');expect(h.wrapper.find('[data-testid=chat-stop]').exists()).toBe(true);expect(h.wrapper.get('[data-testid=chat-send]').attributes('aria-label')).toBe('加入队列');
    await h.wrapper.get('textarea').trigger('keydown',{key:'Enter'});await flushPromises();
    expect(h.host.mock.calls.find(([c])=>c==='chat_enqueue')).toBeDefined();expect(h.host.mock.calls.some(([c])=>c==='chat_cancel')).toBe(false);expect(h.chat.draft.text).toBe('');expect(h.chat.draft.attachments).toHaveLength(0);expect(h.chat.queue?.items).toHaveLength(1);
    h.wrapper.unmount();h.chat.dispose();
  });
  it('keeps a draft intact when Stop is pressed and never queues that draft',async()=>{
    const h=setup();await flushPromises();h.chat.draft.text='not yet submitted';h.chat.draft.attachments=['data:image/png;base64,YQ=='];await flushPromises();
    await h.wrapper.get('[data-testid=chat-stop]').trigger('click');await flushPromises();
    expect(h.host.mock.calls.some(([c])=>c==='chat_cancel')).toBe(true);expect(h.host.mock.calls.some(([c])=>c==='chat_enqueue')).toBe(false);expect(h.chat.draft.text).toBe('not yet submitted');expect(h.chat.draft.attachments).toHaveLength(1);h.wrapper.unmount();h.chat.dispose();
  });
  it('lets the user remove a pending attachment while the model is replying',async()=>{
    const h=setup();h.chat.draft.attachments=['data:image/png;base64,YQ=='];await flushPromises();
    await h.wrapper.get('[data-testid=chat-attachment-remove-0]').trigger('click');expect(h.chat.draft.attachments).toHaveLength(0);h.wrapper.unmount();h.chat.dispose();
  });
});
