import {describe,it,expect} from 'vitest';
import {widgetKinds,baseSize,validateLayout,emptyLayout,validStockSymbol,type DesktopWidget} from './model.ts';
const one=(kind:DesktopWidget['kind']='clock'):DesktopWidget=>({id:'one',kind,theme:'ivory',scale:1,x:0,y:0,enabled:true,city:null});
describe('reference widget schema',()=>{
 it('exposes exactly the eight cards and reference proportions',()=>{expect(widgetKinds.map(w=>w.kind)).toEqual(['clock','weather','notes','reminders','stocks','woodfish','calendar','countdown']);expect(baseSize('clock')).toEqual({width:596,height:306});expect(baseSize('countdown')).toEqual({width:790,height:286});});
 it('retires every previous layout, appearance and custom executable',()=>{for(const kind of ['fortune','flower','custom'])expect(()=>validateLayout({...emptyLayout(),widgets:[{...one(),kind}]})).toThrow();expect(()=>validateLayout({version:1,widgets:[]})).toThrow();expect(()=>validateLayout({...emptyLayout(),widgets:[{...one(),theme:'glass'}]})).toThrow();});
 it('validates HK and A-share identities without query injection',()=>{for(const s of ['116.00700','1.000001','0.399006','1.600519'])expect(validStockSymbol(s)).toBe(true);for(const s of ['116.00700&x=1','116.abcde','3.123456'])expect(validStockSymbol(s)).toBe(false);});
 it('validates event time ranges',()=>{const w={...one('reminders'),living:{count:0,symbol:'1.000001',reminders:[{id:'event',title:'评审',at:'2026-10-19T10:00',done:false,end:'09:00'}]}};expect(()=>validateLayout({...emptyLayout(),widgets:[w]})).toThrow();w.living.reminders[0].end='11:00';expect(validateLayout({...emptyLayout(),widgets:[w]}).widgets).toHaveLength(1);});
});
