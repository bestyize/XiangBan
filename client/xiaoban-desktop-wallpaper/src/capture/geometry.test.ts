import { describe,it,expect } from 'vitest';
import { selectionBetween,resizeSelection,toolbarPosition,sourceRect } from './geometry.ts';
describe('capture geometry',()=>{
  it('allows selection in every drag direction',()=>{
    expect(selectionBetween({x:300,y:200},{x:50,y:20})).toEqual({x:50,y:20,w:250,h:180});
  });
  it('keeps every resize edge within the screen with a minimum crop',()=>{
    const r={x:100,y:80,w:200,h:120};
    expect(resizeSelection(r,'nw',{x:-500,y:-500},{x:800,y:600})).toEqual({x:0,y:0,w:300,h:200});
    expect(resizeSelection(r,'se',{x:999,y:999},{x:800,y:600})).toEqual({x:100,y:80,w:700,h:520});
    expect(resizeSelection(r,'nw',{x:999,y:999},{x:800,y:600})).toEqual({x:288,y:188,w:12,h:12});
  });
  it('moves the toolbar above a crop near the bottom and clamps it horizontally',()=>{
    const p=toolbarPosition({x:0,y:500,w:100,h:80},430,48,{x:800,y:600});
    expect(p.left).toBe(10);expect(p.top).toBe(438);
  });
  it('keeps controls visible for a full-screen crop',()=>{
    const p=toolbarPosition({x:0,y:0,w:800,h:600},430,48,{x:800,y:600});
    expect(p.left).toBeGreaterThanOrEqual(0);expect(p.top).toBeGreaterThanOrEqual(0);expect(p.top+48).toBeLessThan(600);
  });
  it('maps CSS selection into Retina snapshot pixels independently on each axis',()=>{
    expect(sourceRect({x:120,y:90,w:200,h:150},{x:1000,y:800},{x:2000,y:1600})).toEqual({x:240,y:180,w:400,h:300});
  });
});
