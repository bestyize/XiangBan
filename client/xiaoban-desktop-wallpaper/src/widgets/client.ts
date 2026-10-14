import { emptyNote, emptyCountdown } from './content.ts';
import { ref } from 'vue';
import { desktop, invoke } from '../bridge.ts';
import { emptyLayout, validateLayout, type WidgetLayout, type WidgetKind, type DesktopWidget, type City } from './model.ts';
const key = 'xiaoban.widgets.v1';
export function createWidgetClient() {
  const layout = ref<WidgetLayout>(emptyLayout()), ready = ref(false), busy = ref(false), error = ref('');
  let queue = Promise.resolve();
  function accept(value: unknown) { layout.value = validateLayout(value); }
  async function load() {
    ready.value = false; error.value = '';
    try { const stored = desktop ? await invoke('get_desktop_widgets') : localStorage.getItem(key); accept(desktop ? stored : stored ? JSON.parse(stored as string) : emptyLayout()); ready.value = true; }
    catch (e) { error.value = String(e); }
  }
  function change(command: string, args: Record<string, unknown>, fallback: (layout: WidgetLayout) => WidgetLayout) {
    const next = queue.catch(() => {}).then(async () => {
      if (!ready.value) throw new Error('组件布局尚未载入'); busy.value = true; error.value = '';
      try {
        if (desktop) accept(await invoke(command, args));
        else { const changed = validateLayout(fallback(structuredClone(JSON.parse(JSON.stringify(layout.value))))); localStorage.setItem(key, JSON.stringify({version:1,widgets:changed.widgets})); accept(changed); }
      } catch (e) { error.value = String(e); throw e; }
      finally { busy.value = false; }
    }); queue = next; return next;
  }
  function add(kind: WidgetKind, customId?: string) { return change('add_desktop_widget', {kind, customId}, current => { if(current.widgets.length >= 12) throw new Error('桌面最多放置 12 个组件'); const count=current.widgets.length; current.widgets.push({id:crypto.randomUUID().replaceAll('-','').slice(0,16),kind,theme:kind==='fortune'?'paper':'glass',scale:1,x:Math.max(.05,(count%3===1?.88:.52)-Math.floor(count/3)*.25),y:count%3===2?.94:.06,enabled:true,city:null,customId,...(kind==='notes'?{note:emptyNote()}:kind==='countdown'?{countdown:emptyCountdown()}:{})}); return current; }); }
  function update(id: string, patch: Partial<Omit<DesktopWidget,'id'|'kind'>>) { return change('update_desktop_widget',{id,patch}, current => ({...current,widgets:current.widgets.map(widget=>widget.id===id?{...widget,...patch}:widget)})); }
  function remove(id: string) { return change('remove_desktop_widget',{id},current=>({...current,widgets:current.widgets.filter(widget=>widget.id!==id)})); }
  function edit(editing: boolean) { return change('set_widgets_editing',{editing},current=>({...current,editing})); }
  function changed(event: Event) { try { accept((event as CustomEvent).detail); } catch (e) { error.value=String(e); } }
  window.addEventListener('widgets-changed',changed);
  function dispose() { window.removeEventListener('widgets-changed',changed); }
  return { layout, ready, busy, error, load, add, update, remove, edit, dispose };
}
export async function searchWidgetCities(query: string): Promise<City[]> {
  if (desktop) return invoke('search_care_cities',{query});
  const url = new URL('https://geocoding-api.open-meteo.com/v1/search'); url.searchParams.set('name',query); url.searchParams.set('language','zh'); url.searchParams.set('count','6');
  const response=await fetch(url,{signal:AbortSignal.timeout(12000)});if(!response.ok)throw new Error('城市查询失败，请稍后再试');
  const data=await response.json();return (data.results??[]).map((city: City)=>({name:city.name,country:city.country??'',admin1:city.admin1??'',latitude:city.latitude,longitude:city.longitude}));
}

