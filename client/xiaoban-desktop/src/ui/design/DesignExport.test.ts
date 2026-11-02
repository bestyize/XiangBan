// @vitest-environment happy-dom
import {mount,flushPromises} from '@vue/test-utils';
import {defineComponent,h} from 'vue';
import {afterEach,beforeEach,describe,it,expect,vi} from 'vitest';
import DesignWorkbench from './DesignWorkbench.vue';
const mocks=vi.hoisted(()=>({invoke:vi.fn(),listen:vi.fn(),captureScene:vi.fn(),cancelCapture:vi.fn(),makePptx:vi.fn()}));
vi.mock('../../bridge.ts',()=>({invoke:mocks.invoke,listen:mocks.listen}));
vi.mock('./pptx-export.ts',()=>({makePptx:mocks.makePptx}));
const fixture={id:'design-012345678901234567890123',title:'设计',sessionId:'s',revision:1,updatedMs:1,canvas:{v:3,boards:{'Main.dc.html':{title:'首页',w:100,h:80,x:0,y:0}},order:['Main.dc.html']},files:{'Main.dc.html':'<x-dc><p>文字</p></x-dc>'}};
const preview=defineComponent({setup(_,{expose}){expose({captureScene:mocks.captureScene,cancelCapture:mocks.cancelCapture});return()=>h('div');}});
let wrapper:ReturnType<typeof mount>|undefined;
beforeEach(()=>{vi.resetAllMocks();mocks.listen.mockResolvedValue(()=>{});mocks.invoke.mockImplementation(async cmd=>cmd==='design_read'?{project:fixture,origin:'design-preview://localhost'}:cmd==='design_export'?'/tmp/design.pptx':undefined);mocks.makePptx.mockResolvedValue('pptx-data');mocks.captureScene.mockResolvedValue({version:1,width:100,height:80,nodes:[],warnings:[]});});
afterEach(()=>wrapper?.unmount());
async function open(){wrapper=mount(DesignWorkbench,{props:{projectId:fixture.id,notify:vi.fn()},global:{stubs:{BoardPreview:preview}}});await flushPromises();return wrapper;}
describe('editable export workbench actions',()=>{
 it('exports the captured saved revision through the native file picker',async()=>{
  const w=await open();await w.findAll('.export button').find(b=>b.text()==='PowerPoint (.pptx)')!.trigger('click');await flushPromises();
  expect(mocks.makePptx).toHaveBeenCalledWith('设计',[expect.objectContaining({title:'首页',file:'Main.dc.html'})]);
  expect(mocks.invoke).toHaveBeenCalledWith('design_export',{id:fixture.id,format:'pptx',project:null,rendered:{revision:1,file:null,data:'pptx-data'}});
  expect(w.find('.export-progress').exists()).toBe(false);
 });
 it('does not open a save dialog when cancelled during scene extraction',async()=>{
  let done!:(scene:any)=>void;mocks.captureScene.mockImplementation(()=>new Promise(r=>{done=r;}));
  const w=await open();await w.findAll('.export button').find(b=>b.text()==='PowerPoint (.pptx)')!.trigger('click');await flushPromises();
  await w.get('.export-progress button').trigger('click');done({version:1,width:100,height:80,nodes:[],warnings:[]});await flushPromises();
  expect(mocks.makePptx).not.toHaveBeenCalled();expect(mocks.invoke.mock.calls.some(([cmd])=>cmd==='design_export')).toBe(false);
 });
});
