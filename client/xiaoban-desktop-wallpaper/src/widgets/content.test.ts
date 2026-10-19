import { describe, it, expect } from 'vitest';
import { calendarLabel, dateKey, holidayInfo, lunarDate } from './calendar.ts';
import { countdownValue, emptyCountdown, emptyNote, validDate } from './content.ts';
import { emptyLayout, validateLayout, type DesktopWidget } from './model.ts';
const day=(value:string)=>{const [y,m,d]=value.split('-').map(Number);return new Date(y,m-1,d,12);};
const widget=(kind:DesktopWidget['kind']):DesktopWidget=>({id:'abc',kind,theme:'ivory',scale:1,x:0,y:0,enabled:true,city:null});
describe('Chinese calendar metadata',()=>{
  it('distinguishes official holidays from weekend makeup workdays',()=>{
    expect(holidayInfo(day('2026-10-01'))).toMatchObject({holiday:'国庆节',badge:'休'});
    expect(holidayInfo(day('2026-10-10'))).toMatchObject({makeup:'国庆节',badge:'班'});
    expect(holidayInfo(day('2026-10-11')).badge).toBe('');
    expect(holidayInfo(day('2027-10-01'))).toMatchObject({covered:false,badge:''});
  });
  it('does not call the first day of the Spring Festival break lunar New Year',()=>{
    expect(calendarLabel(day('2026-02-15'))).not.toBe('春节');
    expect(lunarDate(day('2026-02-17'))).toMatchObject({month:'正月',day:1,festival:'春节'});
    expect(calendarLabel(day('2026-06-19'))).toBe('端午');
    expect(calendarLabel(day('2026-09-25'))).toBe('中秋');
  });
});
describe('countdown civil dates',()=>{
  it('rejects impossible dates, ambiguous formats and out-of-range years',()=>{
    expect(validDate('2024-02-29')).toBe(true);
    for(const date of ['2026-02-29','2026-2-01','2026-04-31','1899-12-31','2101-01-01'])expect(validDate(date)).toBe(false);
  });
  it('handles today, passed targets, elapsed time and future start dates',()=>{
    const data={...emptyCountdown(),date:'2026-10-12'};
    expect(countdownValue(data,day('2026-10-12'))).toMatchObject({days:0,label:'就是今天'});
    expect(countdownValue(data,day('2026-10-13'))).toMatchObject({days:1,label:'已过去'});
    expect(countdownValue({...data,mode:'elapsed'},day('2026-10-13'))).toMatchObject({days:1,label:'已经'});
    expect(countdownValue({...data,mode:'elapsed'},day('2026-10-11'))).toMatchObject({days:1,label:'还有'});
  });
  it('uses February 28 in non-leap years without losing the original February 29',()=>{
    const data={...emptyCountdown(),date:'2024-02-29',repeat:'yearly' as const};
    expect(countdownValue(data,day('2026-02-27'))).toMatchObject({days:1,date:'2026-02-28'});
    expect(countdownValue(data,day('2026-03-01')).date).toBe('2027-02-28');
    expect(countdownValue(data,day('2028-02-27'))).toMatchObject({days:2,date:'2028-02-29'});
    expect(data.date).toBe('2024-02-29');
  });
  it('counts calendar days over DST and keeps future annual events in their starting year',()=>{
    expect(countdownValue({...emptyCountdown(),date:'2026-03-09'},day('2026-03-07')).days).toBe(2);
    expect(countdownValue({...emptyCountdown(),date:'2028-06-01',repeat:'yearly'},day('2026-06-02')).date).toBe('2028-06-01');
    expect(dateKey(day('2026-12-31'))).toBe('2026-12-31');
  });
});
describe('typed widget content',()=>{
  it('accepts old widgets without new fields and rejects wrong-type content and duplicate todo IDs',()=>{
    expect(validateLayout({...emptyLayout(),widgets:[widget('calendar')]}).widgets).toHaveLength(1);
    expect(()=>validateLayout({...emptyLayout(),widgets:[{...widget('calendar'),note:emptyNote()}]})).toThrow();
    const note={...emptyNote(),items:[{id:'one',text:'读书',done:false},{id:'one',text:'喝水',done:true}]};
    expect(()=>validateLayout({...emptyLayout(),widgets:[{...widget('notes'),note}]})).toThrow();
    expect(()=>validateLayout({...emptyLayout(),widgets:[{...widget('countdown'),countdown:{...emptyCountdown(),mode:'elapsed',repeat:'yearly'}}]})).toThrow();
  });
});

