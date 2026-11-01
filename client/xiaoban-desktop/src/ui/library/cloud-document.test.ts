// @vitest-environment happy-dom
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '../../bridge.ts';
import { acceptCloudTask } from './cloud-document.ts';
import type { ControlStore } from '../control-types.ts';
vi.mock('../../bridge.ts',()=>({invoke:vi.fn()}));
beforeEach(()=>{ vi.mocked(invoke).mockReset(); });
describe('cloud document task handoff',()=>{
  it('uses host-resolved body attachments and a new task, never raw child prompts',async()=>{
    const store={chat:{select:vi.fn()},navigate:vi.fn()} as unknown as ControlStore;
    const attachment={name:'正文.txt',mime:'text/plain',size:6,dataUrl:'data:text/plain;base64,5q2j5paH'};
    vi.mocked(invoke).mockResolvedValue({draft:'总结附件',attachments:[attachment]});
    await acceptCloudTask(store,{id:'saved',mode:'summary',draft:'untrusted child prompt'});
    expect(invoke).toHaveBeenCalledWith('library_cloud_question',{id:'saved',mode:'summary'});
    expect(store.creationAttachments).toEqual([attachment]); expect(store.creationDraft).toBe('总结附件');
    expect(store.chat.select).toHaveBeenCalledWith(null); expect(store.navigate).toHaveBeenCalledWith('new-task');
  });
  it('keeps the current task intact if content cannot be read',async()=>{
    const store={chat:{select:vi.fn()},navigate:vi.fn()} as unknown as ControlStore;
    vi.mocked(invoke).mockRejectedValue('未读到正文');
    await expect(acceptCloudTask(store,{id:'saved',mode:'ask'})).rejects.toBe('未读到正文');
    expect(store.chat.select).not.toHaveBeenCalled();
    await expect(acceptCloudTask(store,{id:'saved',mode:'write'})).rejects.toThrow('参数无效');
  });
});
