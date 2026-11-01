import { reactive } from 'vue';

export type CloudProvider = 'tencent' | 'baidu';
export interface CloudSource { provider: CloudProvider; name: string; connectorId: string | null; status: string; connected: boolean; browse: boolean; search: boolean }
export interface CloudEntry { id: string; name: string; url: string; path: string; directory: boolean; location?: string }
type Invoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;
export function createCloudBrowser(invoke: Invoke) {
  let version = 0;
  let scope = '', cursors: (number | undefined)[] = [];
  const state = reactive({
    sources: [] as CloudSource[], active: null as CloudProvider | null, items: [] as CloudEntry[], query: '', searchMode: 'title' as 'title' | 'content', location: '', locationLabel: '', page: 1, hasMore: false, loading: false, confirmationRequired: false, error: '', sourceError: '', notice: '', history: [] as {location:string;label:string}[],
    async refreshSources() {
      try { const sources = await invoke<CloudSource[]>('library_cloud_sources'); if (!Array.isArray(sources)) throw new Error('无法读取云文档连接状态，请更新客户端后重试'); state.sources = sources; state.sourceError = ''; }
      catch (e) { state.sourceError = String(e); }
    },
    async load(page = 1, confirmed = false) {
      if (!state.active) return;
      const id = ++version;
      const nextScope=JSON.stringify([state.active,state.query,state.location,state.searchMode]);
      if(nextScope!==scope || page===1){scope=nextScope;cursors=[];}
      state.loading = true; state.error = ''; state.notice = ''; state.items = []; state.hasMore = false; state.confirmationRequired = false; state.page = page;
      try {
        type Page = { items: CloudEntry[]; hasMore: boolean; nextCursor?: number | null; confirmationRequired?: boolean };
        const args: Record<string, unknown> = { platform: state.active, query: state.query, ...(state.searchMode === 'content' ? {contentSearch:true} : {}), location: state.location, pageNumber: page, ...(confirmed ? {confirmed:true} : {}), ...(cursors[page-1] != null ? {cursor:cursors[page-1]} : {}) };
        let result: Page;
        try { result = await invoke<Page>('library_cloud_browse', args); }
        catch (error) {
          if (id !== version) return;
          if (args.platform !== 'baidu' || !args.contentSearch || !/百度(语义接口未返回|检索暂不可用)|服务返回了无法识别的文件列表|云端返回了未识别的文件格式|服务未返回结构化文件列表/.test(String(error))) throw error;
          delete args.contentSearch;
          result = await invoke<Page>('library_cloud_browse', args);
          if (id === version) { state.searchMode = 'title'; state.notice = '百度语义搜索暂不可用，已改用文件名搜索。'; }
        }
        if (id === version) { state.items = result.items; state.hasMore = result.hasMore; state.confirmationRequired = !!result.confirmationRequired; if(Number.isInteger(result.nextCursor) && result.nextCursor!>=0) cursors[page]=result.nextCursor!; }
      } catch (e) { if (id === version) state.error = String(e); }
      finally { if (id === version) state.loading = false; }
    },
    async open(source: CloudSource) { state.active = source.provider; state.query = ''; state.searchMode = 'title'; state.location = ''; state.locationLabel = ''; state.confirmationRequired = false; state.history = []; if (source.browse) await state.load(); else { ++version; state.items = []; state.error = ''; state.loading = false; } },
    async enter(entry: CloudEntry) { state.history.push({location:state.location,label:state.locationLabel}); state.location = entry.location || (state.active === 'baidu' ? entry.path : entry.id); state.locationLabel = entry.name; state.query = ''; state.searchMode = 'title'; await state.load(); },
    async back() { const previous = state.history.pop(); state.location = previous?.location ?? ''; state.locationLabel = previous?.label ?? ''; state.query = ''; state.searchMode = 'title'; await state.load(); },
    close() { ++version; state.active = null; state.loading = false; state.items = []; state.error = ''; state.notice = ''; state.confirmationRequired = false; },
  });
  return state;
}
