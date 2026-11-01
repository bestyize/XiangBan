// @vitest-environment happy-dom
import {readFileSync} from 'node:fs';
import {describe,it,expect} from 'vitest';
import {exportOrder,validatePng,makePdf} from './raster-export.ts';
import type {Board} from './model.ts';
const board=(file:string,x=0,y=0,w=1280,h=800):Board=>({file,title:file,x,y,w,h});
function header(w:number,h:number){const data=new Uint8Array(33);data.set([137,80,78,71,13,10,26,10]);data.set([73,72,68,82],12);const view=new DataView(data.buffer);view.setUint32(16,w);view.setUint32(20,h);return 'data:image/png;base64,'+btoa(String.fromCharCode(...data));}
describe('rendered export boundaries',()=>{
 it('orders by canvas position without changing the stored source order',()=>{const items=[board('bottom',0,100),board('right',100,0),board('left',0,0)];expect(exportOrder(items).map(b=>b.file)).toEqual(['left','right','bottom']);expect(items[0].file).toBe('bottom');});
 it('requires the exact artboard pixel dimensions and rejects forged or huge headers',()=>{expect(validatePng(header(2560,1600),board('a'),2)).toEqual({width:2560,height:1600});expect(()=>validatePng(header(1280,800),board('a'),2)).toThrow('尺寸');expect(()=>validatePng(header(20000,20000),board('a',0,0,20000,20000),1)).toThrow('尺寸');expect(()=>validatePng('data:image/png;base64,AAAA',board('a'),1)).toThrow('PNG');});
 it('creates mixed-orientation PDF pages at CSS-pixel physical sizes',async()=>{
  const jpeg='data:image/jpeg;base64,'+readFileSync(process.cwd()+'/src/ui/design/fixtures/export-pixel.jpg').toString('base64');
  const result=await makePdf([{board:board('wide'),jpeg},{board:board('tall',0,0,390,844),jpeg}]);
  const pdf=atob(result.split(',')[1]);expect(pdf.startsWith('%PDF-')).toBe(true);expect(pdf.match(/\/Type \/Page\b/g)).toHaveLength(2);
  expect(pdf).toMatch(/\/MediaBox \[0 0 960\.?0* 600\.?0*\]/);expect(pdf).toMatch(/\/MediaBox \[0 0 292\.5 633\.?0*\]/);expect(pdf).toContain('%%EOF');
 });
});
