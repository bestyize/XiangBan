import { describe, it, expect } from 'vitest';
import { calendarDays, validateLayout, emptyLayout, type DesktopWidget } from './model.ts';
const one=():DesktopWidget=>({id:'one',kind:'calendar',theme:'glass',scale:1,x:.6,y:.2,enabled:true,city:null});
describe('desktop widget layout',()=>{
  it('allows multiple instances but rejects duplicate identities and unsafe bounds',()=>{
    const layout={...emptyLayout(),widgets:[one(),{...one(),id:'two'}]};expect(validateLayout(layout).widgets).toHaveLength(2);
    expect(()=>validateLayout({...layout,widgets:[one(),one()]})).toThrow();
    expect(()=>validateLayout({...layout,widgets:[{...one(),x:NaN}]})).toThrow();
    expect(()=>validateLayout({...layout,widgets:[{...one(),id:'../one'}]})).toThrow();
    expect(()=>validateLayout({...layout,widgets:[{...one(),scale:3}]})).toThrow();
  });
  it('validates cities only on weather widgets',()=>{
    const city={name:'上海',admin1:'上海',country:'中国',latitude:31.23,longitude:121.47};
    expect(()=>validateLayout({...emptyLayout(),widgets:[{...one(),city}]})).toThrow();
    expect(validateLayout({...emptyLayout(),widgets:[{...one(),kind:'weather',city}]}).widgets[0].city).toEqual(city);
    expect(()=>validateLayout({...emptyLayout(),widgets:[{...one(),kind:'weather',city:{...city,latitude:100}}]})).toThrow();
  });
  it('uses a Monday-first six-week calendar and handles leap days and year boundaries',()=>{
    const february=calendarDays(2024,1);expect(february).toHaveLength(42);expect(february[0].date.getDay()).toBe(1);expect(february.filter(day=>day.current)).toHaveLength(29);
    const december=calendarDays(2026,11);expect(december.filter(day=>day.current)).toHaveLength(31);expect(december.at(-1)?.date.getFullYear()).toBe(2027);
  });
});
