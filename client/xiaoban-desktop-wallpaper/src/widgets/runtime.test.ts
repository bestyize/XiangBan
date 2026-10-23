// @vitest-environment happy-dom
import {mount,flushPromises} from '@vue/test-utils';
import {describe,it,expect,vi,beforeEach} from 'vitest';
const {invoke}=vi.hoisted(()=>({invoke:vi.fn()}));
vi.mock('../bridge.ts',()=>({desktop:true,invoke}));
import WidgetSurface from './WidgetSurface.vue';
import type {DesktopWidget} from './model.ts';
const one=(kind:DesktopWidget['kind']):DesktopWidget=>({id:'runtime',kind,theme:'ivory',scale:.5,x:0,y:0,enabled:true,city:null});
describe('widget native bridge data',()=>{
 beforeEach(()=>{invoke.mockReset();Object.defineProperty(document,'hidden',{value:false,configurable:true});});
 it('opens the widget settings window directly and exposes the resize grip only on desktop cards',async()=>{
  invoke.mockResolvedValue(undefined);const widget=one('clock');const payload={widget,editing:false,screenX:100,screenY:100};const view=mount(WidgetSurface,{props:{widget,payload}});expect(view.find('[aria-label="按比例调整组件大小"]').exists()).toBe(true);await view.get('[data-testid="widget-settings"]').trigger('click');expect(invoke).toHaveBeenCalledWith('show_widget_settings');expect(invoke).not.toHaveBeenCalledWith('present_control');view.unmount();const preview=mount(WidgetSurface,{props:{widget,payload,preview:true}});expect(preview.find('[aria-label="按比例调整组件大小"]').exists()).toBe(false);preview.unmount();
 });
 it('keeps actual weather when the independent air quality request fails',async()=>{
  invoke.mockImplementation((command:string)=>command==='get_widget_weather'?Promise.resolve({text:'杭州：多云，16°C',source:'天气来源',updatedAt:1}):command==='get_widget_forecast'?Promise.resolve({days:[{date:'2026-10-19',high:21,low:8,code:2}]}):Promise.reject(Error('空气质量服务不可用')));
  const view=mount(WidgetSurface,{props:{widget:{...one('weather'),city:{name:'杭州',latitude:30,longitude:120,country:'中国',admin1:'浙江'}}}});await flushPromises();expect(view.get('.weather-temperature').text()).toContain('16');expect(view.get('.weather-description').text()).toContain('多云');expect(view.get('.air-quality').text()).toContain('空气数据暂不可用');expect(view.text()).not.toContain('空气优');view.unmount();
 });
 it('renders real AQI with its declared standard rather than demo air quality',async()=>{
  invoke.mockImplementation((command:string)=>Promise.resolve(command==='get_widget_weather'?{text:'杭州：晴，20°C',source:'天气来源',updatedAt:1}:command==='get_widget_forecast'?{days:[]}:{aqi:126,pm25:45,pm10:60,modelAt:1,updatedAt:2,standard:'US AQI',source:'Open-Meteo / CAMS'}));
  const view=mount(WidgetSurface,{props:{widget:{...one('weather'),city:{name:'杭州',latitude:30,longitude:120,country:'中国',admin1:'浙江'}}}});await flushPromises();expect(view.get('.air-quality').text()).toContain('US AQI 126');expect(view.get('.air-quality').attributes('title')).toContain('Open-Meteo / CAMS');view.unmount();
 });
 it('loads the watchlist and switches the chart to the selected security',async()=>{
  const living={symbol:'116.00700',symbols:['116.00700','1.000001'],count:0,reminders:[]};
  invoke.mockImplementation((command:string,args?:{symbol:string;period:string})=>Promise.resolve(command==='get_widget_stock_watchlist'?{quotes:[{symbol:'1.000001',name:'上证指数',price:3000,change:1,percent:.03,updatedAt:1}]}:{symbol:args!.symbol,period:args!.period,name:args!.symbol==='1.000001'?'上证指数':'腾讯控股',price:3000,change:1,percent:.03,points:[2999,3000],updatedAt:'2026-10-19 10:00',source:'真实来源'}));
  const save=vi.fn().mockResolvedValue(undefined);const view=mount(WidgetSurface,{props:{widget:{...one('stocks'),living},save}});await flushPromises();await view.get('[aria-label="查看自选列表"]').trigger('click');await flushPromises();const target=view.findAll('.stock-list-panel>button').find(b=>b.text().includes('上证指数'))!;await target.trigger('click');expect(save.mock.calls[0][0]).toMatchObject({living:{symbol:'1.000001',symbols:living.symbols}});await view.setProps({widget:{...one('stocks'),living:{...living,symbol:'1.000001'}}});await flushPromises();expect(invoke).toHaveBeenCalledWith('get_widget_stocks',{period:'1D',symbol:'1.000001'});expect(view.get('.stock-source').text()).toContain('2026-10-19 10:00');view.unmount();
 });
});
