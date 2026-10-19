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
  it('keeps saved monitor identities when their screens are disconnected and validates live monitor metadata',()=>{
    const monitors=[{id:'left',name:'外接屏',width:1920,height:1080,primary:false},{id:'main',name:'主屏幕',width:1440,height:900,primary:true}];
    const layout=validateLayout({...emptyLayout(),widgets:[{...one(),monitor:'disconnected'}],monitors});
    expect(layout.widgets[0].monitor).toBe('disconnected');expect(layout.monitors).toEqual(monitors);
    expect(()=>validateLayout({...emptyLayout(),widgets:[{...one(),monitor:''}]})).toThrow();
  });
  it('keeps flower records only on the flower widget and inside the growth bounds',()=>{
    const flower={water:4,since:'2026-10-01'};
    expect(validateLayout({...emptyLayout(),widgets:[{...one(),kind:'flower',flower}]}).widgets[0].flower).toEqual(flower);
    expect(()=>validateLayout({...emptyLayout(),widgets:[{...one(),kind:'notes',flower}]})).toThrow();
    expect(()=>validateLayout({...emptyLayout(),widgets:[{...one(),kind:'flower',flower:{...flower,water:-2}}]})).toThrow();
  });
  it('validates custom widgets with customId',()=>{
    const customWidget: DesktopWidget = { ...one(), id: 'custom1', kind: 'custom', customId: 'pomodoro-timer' };
    expect(validateLayout({ ...emptyLayout(), widgets: [customWidget] }).widgets).toHaveLength(1);
    expect(() => validateLayout({ ...emptyLayout(), widgets: [{ ...customWidget, customId: '' }] })).toThrow();
    expect(() => validateLayout({ ...emptyLayout(), widgets: [{ ...customWidget, customId: 'Invalid Id!' }] })).toThrow();
  });
  it('validates reminders, market symbols and wooden fish counts',()=>{
    const living={reminders:[{id:'rem1',title:'团队周会',at:'2026-10-20T09:30',done:false}],count:108,symbol:'1.000001'};
    const widget={...one(),kind:'reminders' as const,living};
    expect(validateLayout({...emptyLayout(),widgets:[widget]}).widgets[0].living?.reminders[0].title).toBe('团队周会');
    expect(()=>validateLayout({...emptyLayout(),widgets:[{...widget,living:{...living,count:-1}}]})).toThrow();
    expect(()=>validateLayout({...emptyLayout(),widgets:[{...widget,living:{...living,symbol:'4.unknown'}}]})).toThrow();
    expect(()=>validateLayout({...emptyLayout(),widgets:[{...widget,living:{...living,reminders:[...living.reminders,...living.reminders]}}]})).toThrow();
    expect(()=>validateLayout({...emptyLayout(),widgets:[{...widget,living:{...living,reminders:[{...living.reminders[0],at:'broken'}]}}]})).toThrow();
    expect(()=>validateLayout({...emptyLayout(),widgets:[{...widget,kind:'calendar'}]})).toThrow();
  });

});

