import { emptyNote, emptyCountdown } from './content.ts';
import { ref } from 'vue';
import { desktop, invoke } from '../bridge.ts';
import { emptyLayout, validateLayout, type WidgetLayout, type WidgetKind, type DesktopWidget, type City, type CustomWidgetPack } from './model.ts';
const key = 'xiaoban.widgets.v2';
const customKey = 'xiaoban.custom_widgets.v1';
const emptyLiving = () => ({reminders: [],count:0,symbol:'1.000001'});
/** Mirrors the host's `default_scale`: the library is drawn for a 1440×900 desktop. */
function defaultScale() {
  const width = window.screen?.width ?? 0, height = window.screen?.height ?? 0;
  if (!(width > 0 && height > 0)) return .5;
  const fit = Math.min(width / 1440, height / 900);
  return Math.min(1.5, Math.max(.5, Math.round(.5 * fit * 100) / 100));
}
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
        else { const changed = validateLayout(fallback(structuredClone(JSON.parse(JSON.stringify(layout.value))))); localStorage.setItem(key, JSON.stringify({version:2,widgets:changed.widgets})); accept(changed); }
      } catch (e) { error.value = String(e); throw e; }
      finally { busy.value = false; }
    }); queue = next; return next;
  }
  function add(kind:WidgetKind,layer:'desktop'|'topmost'='desktop',customId?:string){
    return change('add_desktop_widget',{kind,layer,customId},current=>{
      const count=current.widgets.length;
      let custom: CustomWidgetPack | undefined;
      if (kind === 'custom' && customId) {
        const stored = localStorage.getItem(customKey);
        const packs = stored ? JSON.parse(stored) : [];
        custom = packs.find((p: CustomWidgetPack) => p.id === customId);
      }
      current.widgets.push({
        id:crypto.randomUUID().replaceAll('-','').slice(0,16),
        kind,
        theme: kind === 'custom' ? (custom?.theme ?? 'transparent') : 'ivory',
        layer,
        scale:defaultScale(),
        x:.05+(count%3)*.28,
        y:Math.min(.94,.06+Math.floor(count/3)*.2),
        enabled:true,
        city:null,
        customId,
        custom,
        ...(kind==='notes'?{note:emptyNote()}:kind==='countdown'?{countdown:emptyCountdown()}:['reminders','stocks','woodfish'].includes(kind)?{living:{...emptyLiving(),symbol:kind==='stocks'?'116.00700':'1.000001'}}:{})
      });
      return current;
    });
  }
  function update(id: string, patch: Partial<Omit<DesktopWidget,'id'|'kind'>>) { return change('update_desktop_widget',{id,patch}, current => ({...current,widgets:current.widgets.map(widget=>widget.id===id?{...widget,...patch}:widget)})); }
  function remove(id: string) { return change('remove_desktop_widget',{id},current=>({...current,widgets:current.widgets.filter(widget=>widget.id!==id)})); }
  function edit(editing: boolean) { return change('set_widgets_editing',{editing},current=>({...current,editing})); }
  async function listCustom(): Promise<CustomWidgetPack[]> {
    if (desktop) return invoke<CustomWidgetPack[]>('list_custom_widgets');
    const stored = localStorage.getItem(customKey);
    return stored ? JSON.parse(stored) : [];
  }
  async function saveCustom(pack: CustomWidgetPack, overwrite = false) {
    if (desktop) return invoke('save_custom_widget', { pack, overwrite });
    const stored = localStorage.getItem(customKey);
    let packs: CustomWidgetPack[] = stored ? JSON.parse(stored) : [];
    const idx = packs.findIndex(p => p.id === pack.id);
    if (idx >= 0 && !overwrite) throw new Error(`小组件「${pack.id}」已经存在，请确认覆盖`);
    if (idx >= 0) packs[idx] = pack; else packs.push(pack);
    localStorage.setItem(customKey, JSON.stringify(packs));
  }
  async function removeCustom(id: string) {
    if (desktop) return invoke('remove_custom_widget', { id });
    const stored = localStorage.getItem(customKey);
    if (stored) {
      const packs: CustomWidgetPack[] = JSON.parse(stored);
      localStorage.setItem(customKey, JSON.stringify(packs.filter(p => p.id !== id)));
    }
    layout.value = { ...layout.value, widgets: layout.value.widgets.filter(w => !(w.kind === 'custom' && w.customId === id)) };
    localStorage.setItem(key, JSON.stringify({ version: 2, widgets: layout.value.widgets }));
  }
  function changed(event: Event) { try { accept((event as CustomEvent).detail); } catch (e) { error.value=String(e); } }
  window.addEventListener('widgets-changed',changed);
  function dispose() { window.removeEventListener('widgets-changed',changed); }
  return { layout, ready, busy, error, load, add, update, remove, edit, listCustom, saveCustom, removeCustom, dispose };
}
export async function searchWidgetCities(query: string): Promise<City[]> {
  if (desktop) return invoke('search_care_cities',{query});
  const url = new URL('https://geocoding-api.open-meteo.com/v1/search'); url.searchParams.set('name',query); url.searchParams.set('language','zh'); url.searchParams.set('count','6');
  const response=await fetch(url,{signal:AbortSignal.timeout(12000)});if(!response.ok)throw new Error('城市查询失败，请稍后再试');
  const data=await response.json();return (data.results??[]).map((city: City)=>({name:city.name,country:city.country??'',admin1:city.admin1??'',latitude:city.latitude,longitude:city.longitude}));
}



