import { computed, reactive, ref, watch, onScopeDispose } from "vue";
import { invoke, listen, wallpaperDefaults, type WallpaperEntry, type WallpaperLibrary, type WallpaperOptions } from "../../bridge.ts";
import type { SiteWallpaper } from "../../wallpaper-media.ts";
import type { ControlStore } from "../control-types.ts";
/** Error text of a download the user stopped; matches `library::CANCELLED` in the native host. */
export const DOWNLOAD_CANCELLED = "已取消下载";
export type Selection = { source: "local"; item: WallpaperEntry } | { source: "online"; item: SiteWallpaper };
export function cloneOptions(options: WallpaperOptions) { return JSON.parse(JSON.stringify(options)) as WallpaperOptions; }
export function useWallpaperWorkspace(store: ControlStore) {
  const tab = ref("local"), media = ref(""), query = ref(""), category = ref(""), page = ref(1), localSource = ref("");
  const selection = ref<Selection | null>(null), draft = ref(cloneOptions(store.settings.wallpaperOptions)), busy = ref(false), loading = ref(false), error = ref("");
  const feed = reactive<{items: SiteWallpaper[]; has_more: boolean; total: number}>({items: [], has_more: false,total: 0});
  const progress = ref<{id:string;received:number;total:number} | null>(null);
  /** True only while files are being fetched from the website, the part that can be cancelled. */
  const downloading = ref(false), cancelling = ref(false);
  let progressId:string | undefined, disposed=false, unlisten:(()=>void) | undefined;
  const progressPercent=computed(()=>progress.value ? Math.max(0,Math.min(100,Math.round(progress.value.received/progress.value.total*100))) : null);
  if(store.desktop) void listen<{id:string;received:number;total:number}>("wallpaper-progress", event=>{if(!disposed && busy.value && event.id===progressId && Number.isFinite(event.received) && Number.isFinite(event.total) && event.total>0) progress.value=event;}).then(cleanup=>{if(disposed)cleanup();else unlisten=cleanup;}).catch(e=>store.notify(e,"error"));
  const categories = ref<{id: string; name: string}[]>([]);
  const prepared = reactive(new Map<string,string>()); let generation = 0;
  const kind = computed(() => selection.value?.source === "local" ? selection.value.item.definition.kind : selection.value?.item.media_type);
  function select(value: Selection) {
    if(busy.value)return; selection.value = value;
    const id = value.source === "local" ? value.item.definition.id : value.item.desktop_id || prepared.get(value.item.id);
    draft.value = cloneOptions(id === store.settings.wallpaperId ? store.settings.wallpaperOptions : wallpaperDefaults);
    if (value.source === "local") for (const control of value.item.definition.controls ?? []) draft.value.values[control.key] ??= control.default;
  }
  function cancel() { if(downloading.value) cancelDownload(); else if(!busy.value) selection.value = null; }
  function cancelDownload() {
    if(!downloading.value || cancelling.value) return;
    cancelling.value = true;
    invoke<boolean>("cancel_site_download").catch(e=>{cancelling.value=false;store.notify(e,"error");});
  }
  async function load() {
    const token = ++generation, site = store.settings.websiteUrl;
    if (!store.desktop) return;
    loading.value = true; error.value = "";
    try {
      const result = await invoke<typeof feed>("fetch_site_wallpapers", {q:query.value,category:category.value,media:media.value,page:page.value});
      if(token !== generation || site !== store.settings.websiteUrl) return;
      Object.assign(feed,result);
    } catch(e) { if(token === generation && site === store.settings.websiteUrl) error.value = String(e); }
    finally { if(token === generation) loading.value = false; }
  }
  async function loadCategories() {
    const site = store.settings.websiteUrl;
    try { const result = await invoke<{categories:{id:string;name:string}[]}>("fetch_site_categories"); if(site === store.settings.websiteUrl) categories.value = result.categories; }
    catch { /* Feed retains actionable retry. */ }
  }
  watch([tab,media,category,page], () => { if(tab.value === "online") void load(); });
  watch([media,category], () => {page.value = 1;});
  watch(() => store.settings.websiteUrl, () => { generation++; prepared.clear(); Object.assign(feed,{items:[],has_more:false,total:0}); categories.value=[]; selection.value=null; page.value=1; if(tab.value === "online") {void load();void loadCategories();} });
  onScopeDispose(() => {generation++;disposed=true;unlisten?.();progress.value=null;progressId=undefined;});
  /** 动画（程序绘制）排在最前，其次图片、视频；排序保持稳定，同类型内仍按原有顺序。 */
  const KIND_ORDER: Record<string, number> = { canvas: 0, image: 1, video: 2 };
  const localItems = computed(() => store.library.items.filter(entry => (!media.value || entry.definition.kind === media.value) && (!localSource.value || entry.source === localSource.value) && entry.definition.title.toLowerCase().includes(query.value.toLowerCase())).sort((a, b) => (KIND_ORDER[a.definition.kind] ?? 3) - (KIND_ORDER[b.definition.kind] ?? 3)));
  async function prepare(value: Selection) {
    if(value.source === "local") return value.item.definition.id;
    const site = store.settings.websiteUrl, key = value.item.id;
    const cached = prepared.get(key); if(cached && store.library.items.some(item => item.definition.id === cached && item.installed)) return cached;
    downloading.value = true;
    let result: {library:WallpaperLibrary;id:string};
    try { result = await invoke<{library:WallpaperLibrary;id:string}>("download_site_wallpaper", {id:key}); }
    finally { downloading.value = false; cancelling.value = false; }
    if(site !== store.settings.websiteUrl) throw new Error("网站已切换，请重新选择壁纸");
    if(!result.library.items.some(item=>item.definition.id === result.id && item.installed)) throw new Error("下载未完成校验，请重试");
    store.library=result.library; prepared.set(key,result.id); return result.id;
  }
  async function action(apply: boolean, preview = false) {
    const chosen=selection.value; if(!chosen || busy.value) return;
    const options=cloneOptions(draft.value); progress.value=null; progressId=chosen.source === "local" ? chosen.item.definition.id : prepared.get(chosen.item.id) || chosen.item.desktop_id; busy.value=true;
    try { const id=await prepare(chosen); progressId=id; if(apply) await store.applyWallpaper(id,options,preview); else store.notify("已保存到我的壁纸","success"); }
    catch(e) {if(String(e)===DOWNLOAD_CANCELLED) store.notify(DOWNLOAD_CANCELLED,"info"); else store.notify(e,"error");} finally {busy.value=false;progress.value=null;progressId=undefined;}
  }
  if(store.desktop && !store.info?.uiSmoke) {void load();void loadCategories();}
  const saved = computed(() => { const chosen=selection.value; if(chosen?.source!=="online") return false; const id=prepared.get(chosen.item.id) || chosen.item.desktop_id; return store.library.items.some(item=>item.definition.id===id && item.installed); });
  return {progress,progressPercent,saved,tab,media,query,category,page,localSource,selection,draft,busy,downloading,cancelling,loading,error,feed,categories,kind,select,cancel,cancelDownload,load,localItems,action};
}
