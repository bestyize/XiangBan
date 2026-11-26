import { describe, expect, it, vi } from 'vitest';
import { createCloudChat, type CloudRequest } from './cloud-chat.ts';

describe('cloud conversations', () => {
  it('retries a lost acknowledgement with the same id and replays each SSE event once', async () => {
    const session = 'session-1', runId = 'run-1';
    const summary = { id: session, title: '测试', createdAt: 0, updatedAt: 0 };
    const subs: {after: string; push: (text: string) => void; end: () => void}[] = [];
    const posts: string[] = [], cancelled: string[] = [];
    const enc = new TextEncoder();
    const request: CloudRequest = async (path, init) => {
      if (path === '/sessions') return Response.json(init.method === 'POST' ? summary : []);
      if (path.includes('/events?')) {
        const stream = new ReadableStream<Uint8Array>({start(controller) {
          subs.push({after: path.split('after=')[1], push: s => controller.enqueue(enc.encode(s)), end: () => controller.close()});
          controller.enqueue(enc.encode('event: ready\r\ndata: {}\r\n\r\n'));
        }});
        return new Response(stream, {headers:{'Content-Type':'text/event-stream'}});
      }
      if (path.endsWith('/messages')) {
        posts.push(JSON.parse(String(init.body)).requestId);
        if (posts.length === 1) throw TypeError('Network error');
        return Response.json({session: summary, runId});
      }
      if (path.endsWith('/cancel')) { cancelled.push(path); return Response.json({}); }
      throw Error('Unexpected path ' + path);
    };
    const {chat, state} = createCloudChat({owner:'account-1',request,notify:vi.fn()});
    await chat.init();
    state.tier = 'standard'; chat.configured = true;
    expect(await chat.send('hello')).toBe(true);
    expect(posts).toHaveLength(2);
    expect(posts[0]).toBe(posts[1]);
    const frame = (seq: number, event: object) => 'id: '+seq+'\ndata: '+JSON.stringify({runId,event})+'\n\n';
    subs[0].push(frame(1,{type:'user_message',requestId:posts[0],message:{role:'user',parts:[{type:'text',text:'hello'}]},initial:true}));
    const partial = frame(2,{type:'text_delta',text:'前'});
    subs[0].push(partial.slice(0,-2)); subs[0].push(partial.slice(-2)); subs[0].end();
    await vi.waitFor(() => expect(subs[1]?.after).toBe('2'));
    subs[1].push(partial);
    subs[1].push(frame(3,{type:'text_delta',text:'后'}));
    await chat.cancel();
    expect(cancelled).toEqual(['/sessions/' + chat.currentId + '/runs/run-1/cancel']);
    subs[1].push(frame(4,{type:'cancelled'}));
    await vi.waitFor(() => expect(chat.current()?.running).toBe(false));
    expect(chat.current()?.items.filter(i => i.kind === 'user')).toHaveLength(1);
    expect(chat.current()?.items.find(i => i.kind === 'assistant')).toMatchObject({text:'前后',streaming:false});
    chat.dispose(); subs[1].end();
  });
});
