import { describe,it,expect } from 'vitest';
import { hitRegion } from './hit-regions.ts';
describe('assistant mouse regions',()=>{
  it('adds only four pixels of margin around the visible surface',()=>{
    expect(hitRegion({left:12,bottom:144,width:456,height:68},156,34)).toEqual({left:8,bottom:8,width:464,height:76,radius:38});
  });
  it('keeps the same bottom anchor when the native window grows',()=>{
    expect(hitRegion({left:12,bottom:144,width:456,height:68},156,34)).toEqual(hitRegion({left:12,bottom:388,width:456,height:68},400,34));
  });
  it('does not publish a surface that is not laid out',()=>{
    expect(hitRegion({left:0,bottom:0,width:0,height:68},156,34)).toBeNull();
  });
});
