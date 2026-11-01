// @vitest-environment happy-dom
import { describe, it, expect, vi } from 'vitest';
import { createCloudDownloader } from './cloud-download.ts';
import type { LibraryDoc } from './library-store.ts';
describe('original cloud file download',()=>{
  it('uses only a reference ID, tracks matching progress and hands the actual saved document to the library',async()=>{
    let finish!: (doc: LibraryDoc)=>void;
    const invoke=vi.fn().mockImplementation(()=>new Promise<LibraryDoc>(resolve=>{finish=resolve;}));
    const saved=vi.fn(); const download=createCloudDownloader(invoke,saved);
    const pending=download.download('reference');
    download.progress({id:'other',received:100,total:100}); expect(download.received).toBe(0);
    download.progress({id:'reference',received:50,total:100});expect(download.label()).toContain('下载中 50%');
    const doc={id:'local',name:'原文.pdf',origin:'https://pan.baidu.com/disk/main'} as LibraryDoc;
    finish(doc);await pending;
    expect(invoke).toHaveBeenCalledWith('library_cloud_download',{id:'reference'});expect(saved).toHaveBeenCalledWith(doc);
    expect(download.message).toContain('我的文档');expect(download.id).toBe('');
  });
  it('keeps failed downloads out of the library and permits retry',async()=>{
    const invoke=vi.fn().mockRejectedValueOnce('下载中断').mockResolvedValueOnce({id:'local'});const saved=vi.fn();const download=createCloudDownloader(invoke,saved);
    await download.download('reference');expect(saved).not.toHaveBeenCalled();expect(download.error).toBe('下载中断');
    await download.download('reference');expect(saved).toHaveBeenCalledOnce();expect(download.error).toBe('');
  });
  it('cancels the active identity and reports cancellation without a successful import',async()=>{
    let fail!: (error: string)=>void;
    const invoke=vi.fn().mockImplementation(command=>command==='library_cloud_download'?new Promise((_,reject)=>{fail=reject;}):Promise.resolve());
    const saved=vi.fn();const download=createCloudDownloader(invoke,saved);const pending=download.download('reference');
    await download.cancel();expect(invoke).toHaveBeenLastCalledWith('library_cloud_download_cancel',{id:'reference'});
    fail('下载已取消');await pending;expect(saved).not.toHaveBeenCalled();expect(download.message).toBe('下载已取消');expect(download.error).toBe('');
  });
});
