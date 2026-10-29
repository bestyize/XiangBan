import {expect,it} from 'vitest';
import {zoomAt,wheelPixels} from './viewport.ts';
it('keeps the world point under the cursor while scaling, with bounded zoom',()=>{
 const z=zoomAt(.5,1,300,200,120,80);
 expect((z.left+120)/z.scale).toBe((300+120)/.5);expect((z.top+80)/z.scale).toBe((200+80)/.5);
 expect(zoomAt(1,99,0,0,0,0).scale).toBe(8);expect(zoomAt(1,.001,0,0,0,0).scale).toBe(.1);
});
it('normalizes line and page scroll deltas',()=>{expect(wheelPixels(2,1,500)).toBe(32);expect(wheelPixels(2,2,500)).toBe(1000);});
