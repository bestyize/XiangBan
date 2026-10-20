import { describe, expect, it } from 'vitest';
import { morphGeometry } from './morph.ts';
describe('edge-safe assistant morph', () => {
  it.each([false, true])('keeps both forms inside their native window, right=%s', right => {
    for (const rect of [{left:12,top:80,width:456,height:64},{left:0,top:0,width:320,height:64},{left:0,top:10,width:40,height:40}]) {
      const {full,ball}=morphGeometry(rect,right);
      expect(ball.left).toBeGreaterThanOrEqual(full.left);
      expect(ball.left+ball.width).toBeLessThanOrEqual(full.left+full.width);
      expect(ball.top).toBeGreaterThanOrEqual(full.top);
      expect(ball.top+ball.height).toBeLessThanOrEqual(full.top+full.height);
    }
  });
  it('expands inward from the appropriate side',()=>{
    const rect={left:12,top:80,width:456,height:64};
    expect(morphGeometry(rect,false).ball.left).toBe(12);
    expect(morphGeometry(rect,true).ball.left).toBe(420);
  });
});
