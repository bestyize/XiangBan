// @vitest-environment happy-dom
import {mount,flushPromises} from '@vue/test-utils';
import {describe,it,expect,vi} from 'vitest';
import MediaPreview from './MediaPreview.vue';
const spies=vi.hoisted(()=>({cleanup:vi.fn()}));
vi.mock('../../wallpaper-media.ts',()=>({mediaMarkup:()=>'<img alt="preview">',bindMedia:()=>spies.cleanup}));
describe('media preview animation configuration',()=>{
 it('shows the media only: particle effects belong to the desktop, not to a preview',async()=>{const wrapper=mount(MediaPreview,{props:{kind:'image',source:'',poster:'/test.png',title:'test'}});await flushPromises();expect(wrapper.find('.media-preview').exists()).toBe(true);expect(wrapper.find('canvas').exists()).toBe(false);wrapper.unmount();expect(spies.cleanup).toHaveBeenCalled();});
});
