// @vitest-environment happy-dom
import {mount,flushPromises} from '@vue/test-utils';
import {describe,it,expect,vi} from 'vitest';
import MediaPreview from './MediaPreview.vue';
const spies=vi.hoisted(()=>({create:vi.fn(),update:vi.fn(),destroy:vi.fn(),cleanup:vi.fn()}));
vi.mock('../../wallpaper-media.ts',()=>({mediaMarkup:()=>'<img alt="preview">',bindMedia:()=>spies.cleanup}));
vi.mock('../../wallpaper-effects.ts',()=>({WallpaperEffects:class {constructor(...args:unknown[]){spies.create(...args);}update(...args:unknown[]){spies.update(...args);}destroy(){spies.destroy();}}}));
describe('media preview animation configuration',()=>{
 it('passes global pause and fps to the renderer and reacts to changes',async()=>{const wrapper=mount(MediaPreview,{props:{kind:'image',source:'',poster:'/test.png',title:'test',paused:true,fps:30}});await flushPromises();expect(spies.create).toHaveBeenCalledWith(expect.any(HTMLElement),'image',undefined,{paused:true,fps:30});expect(spies.update).toHaveBeenLastCalledWith('image',undefined,{paused:true,fps:30});await wrapper.setProps({paused:false,fps:60});expect(spies.update).toHaveBeenLastCalledWith('image',undefined,{paused:false,fps:60});wrapper.unmount();expect(spies.destroy).toHaveBeenCalled();expect(spies.cleanup).toHaveBeenCalled();});
});
