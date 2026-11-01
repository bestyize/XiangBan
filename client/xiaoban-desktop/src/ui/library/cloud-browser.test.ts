// @vitest-environment happy-dom
import { describe, expect, it, vi } from 'vitest';
import { createCloudBrowser, type CloudSource } from './cloud-browser.ts';
const source: CloudSource = { provider:'tencent',name:'腾讯文档',connectorId:'docs',status:'connected',connected:true,browse:true,search:true };
describe('cloud browsing', () => {
  it('does not list any account files until the user opens that account', async () => {
    const invoke=vi.fn().mockResolvedValue([source]); const browser=createCloudBrowser(invoke);
    await browser.refreshSources(); expect(invoke.mock.calls.map(([command])=>command)).toEqual(['library_cloud_sources']);
  });
  it('discards late pages after switching provider or leaving the browser', async () => {
    let finish!: (value:unknown)=>void;
    const invoke=vi.fn().mockImplementationOnce(()=>new Promise(resolve=>{finish=resolve;})).mockResolvedValue({items:[{id:'new',name:'新文件'}],hasMore:false});
    const browser=createCloudBrowser(invoke); const old=browser.open(source);
    await browser.open({...source,provider:'baidu'});
    finish({items:[{id:'old',name:'旧文件'}],hasMore:true}); await old;
    expect(browser.items[0].id).toBe('new'); expect(browser.active).toBe('baidu');
    browser.close(); expect(browser.items).toEqual([]);
  });
  it('uses provider-specific folder identity and bounded page arguments', async () => {
    const invoke=vi.fn().mockResolvedValue({items:[],hasMore:false}); const browser=createCloudBrowser(invoke);
    await browser.open({...source,provider:'baidu'});
    await browser.enter({id:'123',name:'工作',path:'/工作',url:'',directory:true});
    expect(invoke).toHaveBeenLastCalledWith('library_cloud_browse',{platform:'baidu',query:'',location:'/工作',pageNumber:1});
    await browser.back(); expect(browser.location).toBe('');
    browser.query='合同'; await browser.load(2);
    expect(invoke).toHaveBeenLastCalledWith('library_cloud_browse',{platform:'baidu',query:'合同',location:'',pageNumber:2});
  });
  it('falls back visibly to file names when Baidu semantics cannot return importable files', async () => {
    const invoke=vi.fn().mockResolvedValue({items:[],hasMore:false});
    const browser=createCloudBrowser(invoke); await browser.open({...source,provider:'baidu'});
    browser.query='Java'; browser.searchMode='content';
    invoke.mockRejectedValueOnce('百度语义接口未返回可导入的文件列表，请改用文件名搜索或稍后重试').mockResolvedValueOnce({items:[{id:'book',name:'Java.pdf'}],hasMore:false});
    await browser.load(); expect(browser.items[0].name).toBe('Java.pdf'); expect(browser.searchMode).toBe('title');
    expect(browser.notice).toContain('已改用文件名搜索'); expect(browser.error).toBe('');
    expect(invoke).toHaveBeenLastCalledWith('library_cloud_browse',{platform:'baidu',query:'Java',location:'',pageNumber:1});
  });
  it('uses explicit content search and resets search cursors when its scope changes', async () => {
    const invoke=vi.fn().mockResolvedValue({items:[],hasMore:true,nextCursor:43});
    const browser=createCloudBrowser(invoke); await browser.open(source); browser.query='正文关键词'; await browser.load();
    browser.searchMode='content'; await browser.load(2);
    expect(invoke).toHaveBeenLastCalledWith('library_cloud_browse',{platform:'tencent',query:'正文关键词',location:'',pageNumber:2,contentSearch:true});
    await browser.enter({id:'folder',name:'目录',path:'',url:'',directory:true});
    expect(browser.searchMode).toBe('title');
  });
  it('keeps the space context when entering folders and requires a fresh confirmation for each marked read', async () => {
    const invoke=vi.fn().mockResolvedValue({items:[],hasMore:false,confirmationRequired:true});
    const browser=createCloudBrowser(invoke); await browser.open(source);
    expect(browser.confirmationRequired).toBe(true);
    await browser.load(1,true);
    expect(invoke).toHaveBeenLastCalledWith('library_cloud_browse',expect.objectContaining({confirmed:true}));
    await browser.enter({id:'folder',name:'工作',path:'',url:'',directory:true,location:'{"spaceId":"space","parentId":"folder"}'});
    expect(browser.locationLabel).toBe('工作');
    expect(invoke).toHaveBeenLastCalledWith('library_cloud_browse',{platform:'tencent',query:'',location:'{"spaceId":"space","parentId":"folder"}',pageNumber:1});
    await browser.back(); expect(browser.location).toBe(''); expect(browser.locationLabel).toBe('');
  });
  it('uses the returned search cursor and forgets it when the query changes', async () => {
    const invoke=vi.fn().mockResolvedValue({items:[],hasMore:true,nextCursor:43});
    const browser=createCloudBrowser(invoke); await browser.open(source); browser.query='周报'; await browser.load();
    await browser.load(2);
    expect(invoke).toHaveBeenLastCalledWith('library_cloud_browse',{platform:'tencent',query:'周报',location:'',pageNumber:2,cursor:43});
    browser.query='合同'; await browser.load(2);
    expect(invoke).toHaveBeenLastCalledWith('library_cloud_browse',{platform:'tencent',query:'合同',location:'',pageNumber:2});
  });
});
